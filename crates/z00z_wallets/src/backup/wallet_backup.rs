//! Wallet backup cryptography helpers.
//!
//! Phase 1 usage: encrypted, password-based wallet backups.
//!
//! Notes:
//! - This module centralizes symmetric crypto dependencies to preserve the
//!   ONE SOURCE OF TRUTH principle.
//! - Downstream business logic should not import `argon2` or `chacha20poly1305`.
//!

use z00z_crypto::expert::encoding::SafePassword;
use z00z_crypto::{
    aead::{open, seal},
    CryptoError, DomainHasher,
};

use crate::db::wallet_store_crypto::{derive_pw_key, KdfAlgo, KdfParams};
use crate::domains::{WalletBackupAadTagDomain, WalletBackupChecksumDomain};

// Hash domains live in `crate::domains`.

/// Facade for wallet backup cryptography.
///
/// This provides short, ergonomic method names at call sites while keeping the
/// cryptographic contract explicit and stable inside `z00z_crypto`.
///
/// Internally, it uses mandatory domain separation via `hash_domain!()`.
///
/// # Security Parameters (Production Recommendations)
///
/// ## Argon2id Settings
/// - **Memory**: 128 MiB (134,217,728 bytes)
/// - **Iterations**: 3
/// - **Parallelism**: 6 (threads)
///
/// ## XChaCha20-Poly1305
/// - **Key size**: 32 bytes (256-bit)
/// - **Nonce size**: 24 bytes (192-bit)
/// - **Authentication tag**: 16 bytes (128-bit)
///
/// # Example: Complete Backup Workflow
///
/// ```
/// use z00z_wallets::backup::WalletBackupCrypto;
/// use z00z_crypto::expert::encoding::SafePassword;
///
/// let password = SafePassword::from("my-secure-password");
/// let salt = [0x42u8; 16];
/// let wallet_data = b"encrypted-wallet-data";
/// let metadata = b"{\"version\":1,\"timestamp\":1234567890}";
///
/// let key = WalletBackupCrypto::derive_key(&password, &salt).unwrap();
/// let aad_tag = WalletBackupCrypto::aad_tag(metadata);
/// let aad = [&aad_tag[..], metadata].concat();
/// let ciphertext = WalletBackupCrypto::encrypt(&key, &aad, wallet_data).unwrap();
/// let _nonce: [u8; 24] = ciphertext[1..25].try_into().unwrap();
/// let _checksum = WalletBackupCrypto::checksum(&aad, &ciphertext);
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct WalletBackupCrypto;

include!("wallet_backup_kdf.rs");

impl WalletBackupCrypto {
    /// Derive a 32-byte encryption key from a password (Argon2id).
    pub fn derive_key(password: &SafePassword, salt: &[u8; 16]) -> Result<[u8; 32], CryptoError> {
        Self::derive_key_with_kdf(password, &BackupKdf::default(*salt))
    }

    /// Derive a backup key from validated self-describing KDF metadata.
    pub fn derive_key_with_kdf(
        password: &SafePassword,
        kdf: &BackupKdf,
    ) -> Result<[u8; 32], CryptoError> {
        let params = kdf.to_params()?;
        let pw_key = derive_pw_key(password, &params)?;
        let mut out = [0u8; 32];
        out.copy_from_slice(pw_key.reveal());
        Ok(out)
    }

    /// Create a domain-separated 32-byte tag for associated data JSON.
    pub fn aad_tag(aad_json: &[u8]) -> [u8; 32] {
        wallet_backup_aad_tag(aad_json)
    }

    /// Compute a domain-separated checksum over AAD bytes + ciphertext.
    pub fn checksum(aad_bytes: &[u8], ciphertext: &[u8]) -> [u8; 32] {
        wallet_backup_checksum(aad_bytes, ciphertext)
    }

    /// Encrypt bytes using XChaCha20-Poly1305.
    pub fn encrypt(key: &[u8; 32], aad: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        seal(key, aad, plaintext)
    }

    /// Decrypt bytes using XChaCha20-Poly1305.
    pub fn decrypt(key: &[u8; 32], aad: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        open(key, aad, ciphertext)
    }
}

/// Derive a 32-byte key from a password using Argon2id.
pub use z00z_crypto::kdf::derive_argon2id32_key;

/// Create a domain-separated 32-byte tag for backup associated data.
pub fn wallet_backup_aad_tag(aad_json: &[u8]) -> [u8; 32] {
    let hash = DomainHasher::<WalletBackupAadTagDomain>::new_with_label("wallet_backup_aad")
        .chain(aad_json)
        .finalize();

    let mut out = [0u8; 32];
    out.copy_from_slice(&hash.as_ref()[..32]);
    out
}

/// Compute a domain-separated checksum for a backup container.
pub fn wallet_backup_checksum(aad_bytes: &[u8], ciphertext: &[u8]) -> [u8; 32] {
    let hash = DomainHasher::<WalletBackupChecksumDomain>::new_with_label("wallet_backup_checksum")
        .chain(aad_bytes)
        .chain(ciphertext)
        .finalize();

    let mut out = [0u8; 32];
    out.copy_from_slice(&hash.as_ref()[..32]);
    out
}

#[cfg(test)]
#[path = "test_wallet_backup.rs"]
mod tests;
