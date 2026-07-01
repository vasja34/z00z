use chacha20poly1305::{
    aead::{Aead, Payload},
    Key, KeyInit, XChaCha20Poly1305, XNonce,
};
#[cfg(not(target_arch = "wasm32"))]
use rand::{CryptoRng, RngCore};
#[cfg(not(target_arch = "wasm32"))]
use z00z_utils::rng::SystemRngProvider;

use super::AeadError;

pub const XCHACHA_KEY_SIZE: usize = 32;
pub const XCHACHA_NONCE_SIZE: usize = 24;
pub const POLY1305_TAG_SIZE: usize = 16;
pub const XCHACHA20_POLY1305_ID: u8 = 0x01;
pub const MIN_ENVELOPE_SIZE: usize = 1 + XCHACHA_NONCE_SIZE + POLY1305_TAG_SIZE;
pub const ENVELOPE_HEADER_SIZE: usize = 1 + XCHACHA_NONCE_SIZE;
pub const MAX_AEAD_ENVELOPE_SIZE: usize = 10 * 1024 * 1024;
pub const MAX_AEAD_PLAINTEXT_SIZE: usize = 8 * 1024 * 1024;
pub const MAX_AAD_SIZE: usize = 8 * 1024;
pub const MAX_AAD_SIZE_EXTENDED: usize = 64 * 1024;

pub(crate) fn xchacha20poly1305_encrypt(
    key: &Key,
    nonce: &XNonce,
    aad: &[u8],
    plaintext: &[u8],
) -> Result<Vec<u8>, AeadError> {
    let cipher = XChaCha20Poly1305::new(key);
    let payload = Payload {
        msg: plaintext,
        aad,
    };
    cipher
        .encrypt(nonce, payload)
        .map_err(|_| AeadError::Crypto)
}

pub(crate) fn xchacha20poly1305_decrypt(
    key: &Key,
    nonce: &XNonce,
    aad: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>, AeadError> {
    let cipher = XChaCha20Poly1305::new(key);
    let payload = Payload {
        msg: ciphertext,
        aad,
    };
    cipher
        .decrypt(nonce, payload)
        .map_err(|_| AeadError::Crypto)
}

pub fn random_nonce() -> Result<XNonce, AeadError> {
    let mut bytes = [0u8; XCHACHA_NONCE_SIZE];
    fill_nonce_bytes(&mut bytes)?;
    Ok(*XNonce::from_slice(&bytes))
}

#[cfg(not(target_arch = "wasm32"))]
fn fill_nonce_bytes(bytes: &mut [u8; XCHACHA_NONCE_SIZE]) -> Result<(), AeadError> {
    let provider = SystemRngProvider;
    let mut rng = provider.rng();
    rng.try_fill_bytes(bytes).map_err(|_| AeadError::Random)
}

#[cfg(target_arch = "wasm32")]
fn fill_nonce_bytes(bytes: &mut [u8; XCHACHA_NONCE_SIZE]) -> Result<(), AeadError> {
    getrandom::getrandom(bytes).map_err(|_| AeadError::Random)
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn random_nonce_with_rng<R: CryptoRng + RngCore>(
    rng: &mut R,
) -> Result<XNonce, AeadError> {
    let mut bytes = [0u8; XCHACHA_NONCE_SIZE];
    rng.try_fill_bytes(&mut bytes)
        .map_err(|_| AeadError::Random)?;
    Ok(*XNonce::from_slice(&bytes))
}

#[cfg(test)]
pub(crate) fn encrypt_with_random_nonce(
    key: &Key,
    aad: &[u8],
    plaintext: &[u8],
) -> Result<(XNonce, Vec<u8>), AeadError> {
    let nonce = random_nonce()?;
    let ciphertext = xchacha20poly1305_encrypt(key, &nonce, aad, plaintext)?;
    Ok((nonce, ciphertext))
}
