use crate::CryptoError;

use super::{
    xchacha20poly1305_encrypt, MAX_AAD_SIZE, MAX_AEAD_PLAINTEXT_SIZE, XCHACHA20_POLY1305_ID,
    XCHACHA_KEY_SIZE, XCHACHA_NONCE_SIZE,
};

/// ⚠️ **TEST-ONLY:** Encrypt with deterministic nonce.
///
/// # CRITICAL SECURITY WARNING
///
/// This function allows caller-supplied nonces, which enables catastrophic
/// nonce-reuse vulnerabilities if misused. **NEVER use in production code.**
///
/// Nonce reuse with XChaCha20-Poly1305 leads to:
/// - Complete plaintext recovery
/// - Authentication tag forgery
/// - Full compromise of encrypted data
///
/// This function exists ONLY for:
/// - Unit tests requiring deterministic outputs
/// - Test vectors from cryptographic standards
/// - Reproducible test failures for debugging
///
/// # For Production
///
/// Use `seal()` which ALWAYS generates random nonces.
///
/// # Parameters
///
/// - `key`: 32-byte encryption key
/// - `aad`: Additional authenticated data
/// - `plaintext`: Data to encrypt
/// - `nonce`: 24-byte DETERMINISTIC nonce (⚠️ **DANGEROUS**)
///
/// # Examples
///
/// ```rust,ignore
/// #[cfg(test)]
/// use z00z_crypto::aead::test_only::seal_with_nonce_TEST_ONLY;
///
/// #[test]
/// fn test_encryption_deterministic() {
///     let key = [0x42u8; 32];
///     let nonce = [0x01u8; 24];
///     let envelope = seal_with_nonce_TEST_ONLY(&key, b"", b"test", nonce).unwrap();
///     assert_eq!(envelope[0], 0x01);
/// }
/// ```
#[allow(non_snake_case)]
pub fn seal_with_nonce_TEST_ONLY(
    key: &[u8; XCHACHA_KEY_SIZE],
    aad: &[u8],
    plaintext: &[u8],
    nonce: [u8; XCHACHA_NONCE_SIZE],
) -> Result<Vec<u8>, CryptoError> {
    if plaintext.len() > MAX_AEAD_PLAINTEXT_SIZE {
        return Err(CryptoError::InvalidParameters {
            param: "plaintext_size",
        });
    }

    if aad.len() > MAX_AAD_SIZE {
        return Err(CryptoError::AadTooLarge {
            size: aad.len(),
            limit: MAX_AAD_SIZE,
        });
    }

    let nonce_obj = *chacha20poly1305::XNonce::from_slice(&nonce);
    let ciphertext_with_tag = xchacha20poly1305_encrypt(
        chacha20poly1305::Key::from_slice(key),
        &nonce_obj,
        aad,
        plaintext,
    )
    .map_err(|_| CryptoError::CryptoOperationFailed)?;

    let mut envelope = Vec::with_capacity(1 + XCHACHA_NONCE_SIZE + ciphertext_with_tag.len());
    envelope.push(XCHACHA20_POLY1305_ID);
    envelope.extend_from_slice(&nonce);
    envelope.extend_from_slice(&ciphertext_with_tag);
    Ok(envelope)
}
