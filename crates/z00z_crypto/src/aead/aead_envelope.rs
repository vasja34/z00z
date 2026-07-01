#[cfg(not(target_arch = "wasm32"))]
use rand::{CryptoRng, RngCore};
#[cfg(not(target_arch = "wasm32"))]
use z00z_utils::rng::SystemRngProvider;

use crate::CryptoError;

use super::{
    aead_aad::validate_aad_size,
    aead_primitives::{xchacha20poly1305_decrypt, xchacha20poly1305_encrypt},
    MAX_AAD_SIZE, MAX_AAD_SIZE_EXTENDED, MAX_AEAD_ENVELOPE_SIZE, MAX_AEAD_PLAINTEXT_SIZE,
    MIN_ENVELOPE_SIZE, XCHACHA20_POLY1305_ID, XCHACHA_KEY_SIZE, XCHACHA_NONCE_SIZE,
};

#[cfg(target_arch = "wasm32")]
use super::aead_primitives::random_nonce;
#[cfg(not(target_arch = "wasm32"))]
use super::aead_primitives::random_nonce_with_rng;

pub fn seal(
    key: &[u8; XCHACHA_KEY_SIZE],
    aad: &[u8],
    plaintext: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    seal_with_aad_limit(key, aad, plaintext, MAX_AAD_SIZE)
}

#[doc(hidden)]
#[cfg(not(target_arch = "wasm32"))]
pub fn seal_with_rng<R: CryptoRng + RngCore>(
    rng: &mut R,
    key: &[u8; XCHACHA_KEY_SIZE],
    aad: &[u8],
    plaintext: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    seal_with_rng_and_limit(rng, key, aad, plaintext, MAX_AAD_SIZE)
}

#[doc(hidden)]
pub fn seal_extended_aad(
    key: &[u8; XCHACHA_KEY_SIZE],
    aad: &[u8],
    plaintext: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    seal_with_aad_limit(key, aad, plaintext, MAX_AAD_SIZE_EXTENDED)
}

fn seal_with_aad_limit(
    key: &[u8; XCHACHA_KEY_SIZE],
    aad: &[u8],
    plaintext: &[u8],
    aad_limit: usize,
) -> Result<Vec<u8>, CryptoError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let provider = SystemRngProvider;
        let mut rng = provider.rng();
        seal_with_rng_and_limit(&mut rng, key, aad, plaintext, aad_limit)
    }

    #[cfg(target_arch = "wasm32")]
    {
        if plaintext.len() > MAX_AEAD_PLAINTEXT_SIZE {
            return Err(CryptoError::InvalidParameters {
                param: "plaintext_size",
            });
        }

        validate_aad_size(aad, aad_limit)?;

        let nonce = random_nonce().map_err(|_| CryptoError::RngFailure)?;
        let ciphertext_with_tag = xchacha20poly1305_encrypt(
            chacha20poly1305::Key::from_slice(key),
            &nonce,
            aad,
            plaintext,
        )
        .map_err(|_| CryptoError::CryptoOperationFailed)?;

        let mut envelope = Vec::with_capacity(1 + XCHACHA_NONCE_SIZE + ciphertext_with_tag.len());
        envelope.push(XCHACHA20_POLY1305_ID);
        envelope.extend_from_slice(nonce.as_slice());
        envelope.extend_from_slice(&ciphertext_with_tag);

        Ok(envelope)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn seal_with_rng_and_limit<R: CryptoRng + RngCore>(
    rng: &mut R,
    key: &[u8; XCHACHA_KEY_SIZE],
    aad: &[u8],
    plaintext: &[u8],
    aad_limit: usize,
) -> Result<Vec<u8>, CryptoError> {
    if plaintext.len() > MAX_AEAD_PLAINTEXT_SIZE {
        return Err(CryptoError::InvalidParameters {
            param: "plaintext_size",
        });
    }

    validate_aad_size(aad, aad_limit)?;
    let nonce = random_nonce_with_rng(rng).map_err(|_| CryptoError::RngFailure)?;

    let ciphertext_with_tag = xchacha20poly1305_encrypt(
        chacha20poly1305::Key::from_slice(key),
        &nonce,
        aad,
        plaintext,
    )
    .map_err(|_| CryptoError::CryptoOperationFailed)?;

    let mut envelope = Vec::with_capacity(1 + XCHACHA_NONCE_SIZE + ciphertext_with_tag.len());
    envelope.push(XCHACHA20_POLY1305_ID);
    envelope.extend_from_slice(nonce.as_slice());
    envelope.extend_from_slice(&ciphertext_with_tag);

    Ok(envelope)
}

pub fn open(
    key: &[u8; XCHACHA_KEY_SIZE],
    aad: &[u8],
    envelope: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    open_with_aad_limit(key, aad, envelope, MAX_AAD_SIZE)
}

#[doc(hidden)]
pub fn open_extended_aad(
    key: &[u8; XCHACHA_KEY_SIZE],
    aad: &[u8],
    envelope: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    open_with_aad_limit(key, aad, envelope, MAX_AAD_SIZE_EXTENDED)
}

fn open_with_aad_limit(
    key: &[u8; XCHACHA_KEY_SIZE],
    aad: &[u8],
    envelope: &[u8],
    aad_limit: usize,
) -> Result<Vec<u8>, CryptoError> {
    if envelope.len() < MIN_ENVELOPE_SIZE {
        return Err(CryptoError::InvalidParameters { param: "envelope" });
    }
    if envelope.len() > MAX_AEAD_ENVELOPE_SIZE {
        return Err(CryptoError::InvalidParameters {
            param: "envelope_size",
        });
    }

    validate_aad_size(aad, aad_limit)?;

    if envelope[0] != XCHACHA20_POLY1305_ID {
        return Err(CryptoError::InvalidParameters { param: "algorithm" });
    }

    let (nonce_bytes, ciphertext_with_tag) = envelope[1..].split_at(XCHACHA_NONCE_SIZE);
    let nonce = *chacha20poly1305::XNonce::from_slice(nonce_bytes);

    xchacha20poly1305_decrypt(
        chacha20poly1305::Key::from_slice(key),
        &nonce,
        aad,
        ciphertext_with_tag,
    )
    .map_err(|_| CryptoError::CryptoOperationFailed)
}
