//! Wallet encryption (XChaCha20-Poly1305).
//!
//! 📌 Security Overview:
//! - XChaCha20-Poly1305 encryption for wallet payloads
//! - Argon2id + HKDF-SHA256 key derivation from password
//! - Canonical integral-nonce envelope: `nonce || ciphertext_and_tag`
//! - Versioned container format with checksum
//!
//! # Security Parameters
//!
//! ## Argon2id (Production)
//! - Memory: 128 MiB
//! - Iterations: 3
//! - Parallelism: 6
//! - Output: 32 bytes
//!
//! ## Argon2id (Test-fast feature)
//! - Memory: 16 KiB
//! - Iterations: 1
//! - Parallelism: 1
//! - Output: 32 bytes
//!
//! ⚠️ INSECURE: must never be used in production builds.
//!
//! ## XChaCha20-Poly1305
//! - Key: 256 bits
//! - Nonce: 192 bits (random, unique per encryption)
//! - Tag: 128 bits
//!
//! # Encryption Flow
//!
//! ```text
//! Password + Salt
//!     ↓
//! Argon2id (memory-hard KDF)
//!     ↓
//! Intermediate Key (32 bytes)
//!     ↓
//! HKDF-SHA256 + Domain Info
//!     ↓
//! Final Encryption Key (32 bytes)
//!     ↓
//! XChaCha20-Poly1305
//!     ↓
//! Encrypted Container
//! ```

#![cfg(not(target_arch = "wasm32"))]
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use thiserror::Error;

use crate::domains::WalletEncryptionHkdfInfoDomain;
use crate::{WalletError, WalletResult};
use z00z_crypto::expert::encoding::SafePassword;
use z00z_crypto::{
    aead,
    kdf::{derive_argon2id32_key, hkdf_expand_32, Argon2Params},
    secret::SecretBytes,
    DomainHasher,
};
use z00z_utils::rng::{RngCoreExt, SystemRngProvider};

/// Errors that can occur during wallet encryption/decryption.
#[derive(Debug, Error)]
pub enum WalletEncryptionError {
    /// Unsupported algorithm.
    #[error("unsupported algorithm: {got} (expected {expected})")]
    UnsupportedAlgorithm {
        /// Algorithm name received
        got: String,
        /// Expected algorithm name
        expected: &'static str,
    },

    /// Unsupported payload/container version.
    #[error("unsupported payload version: {version}")]
    UnsupportedPayloadVersion {
        /// Version number
        version: u32,
    },

    /// Invalid password (or corrupted ciphertext).
    #[error("invalid password")]
    InvalidPassword,

    /// Plaintext checksum mismatch (corrupted container).
    #[error("checksum mismatch")]
    ChecksumMismatch {
        /// Expected checksum value
        expected: [u8; 32],
        /// Actual checksum value
        actual: [u8; 32],
    },

    /// Any other cryptographic or format error.
    #[error("cryptographic operation failed: {0}")]
    CryptoError(String),
}

/// Encrypted wallet container format (V1).
///
/// - Uses XChaCha20-Poly1305 for encryption.
/// - Uses the canonical "integral nonce" envelope: `nonce || ciphertext_and_tag`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedWalletContainer {
    /// Container format version (1)
    pub version: u32,

    /// Encryption algorithm identifier.
    pub algorithm: String,

    /// Random 128-bit salt (16 bytes) for Argon2id.
    pub salt: [u8; 16],

    /// Canonical ciphertext envelope: `nonce || ciphertext_and_tag`.
    pub envelope: Vec<u8>,

    /// BLAKE2b-256 checksum of the plaintext wallet record.
    pub checksum: [u8; 32],
}

impl EncryptedWalletContainer {
    /// Current container format version.
    pub const VERSION: u32 = 1;

    /// Algorithm identifier.
    pub const ALGORITHM: &'static str = "xchacha20poly1305";
}

/// Wallet encryption service implementing XChaCha20-Poly1305 with Argon2id + HKDF-SHA256 KDF.
pub struct WalletEncryption;

impl WalletEncryption {
    /// Constant-time comparison for 32-byte arrays using `subtle` crate.
    ///
    /// Prevents timing attacks by ensuring comparison time is independent of input values.
    /// Uses `subtle::ConstantTimeEq` for cryptographic-grade constant-time guarantees.
    fn ct_cmp_32(a: &[u8; 32], b: &[u8; 32]) -> bool {
        bool::from(a.ct_eq(b))
    }

    fn hkdf_info(salt: &[u8; 16]) -> [u8; 32] {
        let hash = DomainHasher::<WalletEncryptionHkdfInfoDomain>::new_with_label(
            "wallet_encryption_hkdf_info",
        )
        .chain(salt)
        .finalize();

        let mut out = [0u8; 32];
        out.copy_from_slice(&hash.as_ref()[..32]);
        out
    }

    /// Derive 32-byte encryption key from password using Argon2id + HKDF-SHA256.
    ///
    /// # Security Parameters
    ///
    /// ## Production (Release)
    /// - Memory: 128 MiB (134,217,728 bytes)
    /// - Iterations: 3
    /// - Parallelism: 6
    ///
    /// ## Test-fast (feature)
    /// - Memory: 16 KiB (16,384 bytes)
    /// - Iterations: 1
    /// - Parallelism: 1
    ///
    /// ⚠️ INSECURE: must never be used in production builds.
    pub fn derive_key(password: &SafePassword, salt: &[u8; 16]) -> WalletResult<[u8; 32]> {
        // Parameter selection based on mode:
        // - Production (release, no test-params-fast): `moderate()` preset
        // - Test-fast (release + test-params-fast): 16 KiB, 1 iteration, 1 parallelism
        #[cfg(feature = "test-params-fast")]
        let params = Argon2Params::test_fast();

        #[cfg(not(feature = "test-params-fast"))]
        let params = Argon2Params::moderate();

        // Step 1: Argon2id - derive intermediate key (extend salt to 32 bytes)
        let mut salt32 = [0u8; 32];
        salt32[..16].copy_from_slice(salt);
        salt32[16..].copy_from_slice(salt);
        let intermediate_key = derive_argon2id32_key(password.reveal(), &salt32, &params)
            .map_err(|e| WalletError::CryptoError(format!("Argon2id failed: {}", e)))?;

        // Step 2: HKDF - expand to final key
        let info = Self::hkdf_info(salt);
        let final_key = hkdf_expand_32(intermediate_key.reveal(), salt, &info)
            .map_err(|e| WalletError::CryptoError(format!("HKDF failed: {}", e)))?;

        Ok(final_key.into_inner())
    }

    /// Encrypt wallet-record bytes into an `EncryptedWalletContainer`.
    ///
    /// # Flow
    /// 1. Compute BLAKE2b-256 checksum of plaintext
    /// 2. Derive encryption key from password + random salt
    /// 3. Generate random 192-bit nonce
    /// 4. Encrypt with XChaCha20-Poly1305
    /// 5. Build canonical envelope: `nonce || ciphertext_and_tag`
    /// 6. Return container with metadata
    pub fn encrypt_wallet(
        password: &SafePassword,
        aad: &[u8],
        plaintext: &[u8],
    ) -> WalletResult<EncryptedWalletContainer> {
        use crate::domains::hashing::compute_encryption_checksum;

        // 1. Compute checksum using domain-separated hashing
        let checksum = compute_encryption_checksum(plaintext);

        // 2. Generate salt and derive key
        let mut salt = [0u8; 16];
        {
            let provider = SystemRngProvider;
            let mut rng = provider.rng();
            rng.fill_bytes_ext(&mut salt);
        }

        let key = zeroize::Zeroizing::new(Self::derive_key(password, &salt)?);

        // 3. Encrypt with XChaCha20-Poly1305
        let envelope = aead::seal(&key, aad, plaintext)
            .map_err(|e| WalletError::CryptoError(format!("Encryption failed: {}", e)))?;

        // 4. Build container
        Ok(EncryptedWalletContainer {
            version: EncryptedWalletContainer::VERSION,
            algorithm: EncryptedWalletContainer::ALGORITHM.to_string(),
            salt,
            envelope,
            checksum,
        })
    }

    /// Decrypt wallet-record bytes from an `EncryptedWalletContainer`.
    ///
    /// # Returns
    /// - `Ok(SecretBytes)` if decryption and checksum verification succeed
    /// - `Err(InvalidPassword)` if password is wrong or ciphertext is corrupted
    /// - `Err(ChecksumMismatch)` if checksum doesn't match (corrupted data)
    pub fn decrypt_wallet(
        password: &SafePassword,
        aad: &[u8],
        container: &EncryptedWalletContainer,
    ) -> WalletResult<SecretBytes> {
        use crate::domains::hashing::compute_encryption_checksum;

        // 1. Validate container version
        if container.version != EncryptedWalletContainer::VERSION {
            return Err(WalletError::InvalidConfig(format!(
                "Unsupported container/payload version: {}",
                container.version
            )));
        }

        // 2. Validate algorithm
        if container.algorithm != EncryptedWalletContainer::ALGORITHM {
            return Err(WalletError::InvalidConfig(format!(
                "Unsupported algorithm: {} (expected {})",
                container.algorithm,
                EncryptedWalletContainer::ALGORITHM
            )));
        }

        // 3. Derive key
        let key = zeroize::Zeroizing::new(Self::derive_key(password, &container.salt)?);

        // 4. Decrypt
        let plaintext =
            aead::open(&key, aad, &container.envelope).map_err(|_| WalletError::InvalidPassword)?;

        // 5. Verify checksum using domain-separated hashing
        let computed_checksum = compute_encryption_checksum(&plaintext);
        if !Self::ct_cmp_32(&computed_checksum, &container.checksum) {
            return Err(WalletError::ChecksumMismatch {
                expected: hex::encode(container.checksum),
                actual: hex::encode(computed_checksum),
            });
        }

        Ok(SecretBytes::new(plaintext))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_cmp_32_equal() {
        let a = [1u8; 32];
        let b = [1u8; 32];
        assert!(WalletEncryption::ct_cmp_32(&a, &b));
    }

    #[test]
    fn test_ct_cmp_32_different() {
        let a = [1u8; 32];
        let b = [2u8; 32];
        assert!(!WalletEncryption::ct_cmp_32(&a, &b));
    }

    #[test]
    fn test_ct_cmp32_all_zeros() {
        let a = [0u8; 32];
        let b = [0u8; 32];
        assert!(WalletEncryption::ct_cmp_32(&a, &b));
    }

    #[test]
    fn test_ct_cmp32_all_ones() {
        let a = [0xFFu8; 32];
        let b = [0xFFu8; 32];
        assert!(WalletEncryption::ct_cmp_32(&a, &b));
    }

    #[test]
    fn test_cmp32_one_bit_diff() {
        let a = [0u8; 32];
        let mut b = [0u8; 32];
        b[31] = 1;
        assert!(!WalletEncryption::ct_cmp_32(&a, &b));
    }
}
