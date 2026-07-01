//! RedB-SPEC.v4 wallet key manager (unified).
//!
//! Combines RedbKeyManager (core) and WalletRedbKeyManager (wrapper) into one module.
//! Native RedB-backed key manager.
//!
//! This module provides wallet-facing orchestration for:
//! - Password KDF -> `PW_KEY`
//! - `MASTER_KEY` generation and wrapping (stored as `secrets["master_key"]`)
//! - HKDF-derived wallet keys (DATA/INDEX/INTEGRITY)
//!
//! The crypto primitives are implemented in `crate::db::wallet_store_crypto`.

#![cfg(not(target_arch = "wasm32"))]

use thiserror::Error;
use zeroize::Zeroizing;

use crate::db::wallet_store_crypto::{
    aad_master_key, derive_pw_key, derive_wallet_keys, AeadEnvelope, KdfParams, MasterKeyRecord,
    RedbKey32, WalletDerivedKeys,
};
use z00z_crypto::expert::encoding::SafePassword;
use z00z_crypto::{CryptoError, Hidden};
use z00z_utils::rng::{RngCoreExt, SystemRngProvider};

use crate::rpc::types::common::PersistWalletId;

/// KDF version for future upgrade path.
pub const KDF_VERSION: u32 = KdfParams::VERSION as u32;

/// Errors returned by `RedbKeyManager`.
#[derive(Debug, Error)]
pub enum RedbKeyManagerError {
    /// Invalid parameters or unsupported configuration.
    #[error("invalid parameters: {0}")]
    InvalidParameters(String),
    /// Migration failed.
    #[error("migration failed: {0}")]
    MigrationFailed(String),
    /// Authentication failed (wrong password or tampered record).
    #[error("authentication failed")]
    AuthenticationFailed,
    /// Underlying cryptographic error.
    #[error("crypto error: {0}")]
    Crypto(#[from] CryptoError),
}

/// Result type for RedB key manager operations.
pub type Result<T> = std::result::Result<T, RedbKeyManagerError>;

/// Map crypto errors to key manager errors with authentication focus.
fn map_auth_error(err: CryptoError) -> RedbKeyManagerError {
    match err {
        CryptoError::CryptoOperationFailed => RedbKeyManagerError::AuthenticationFailed,
        CryptoError::InvalidParameters { param } => {
            RedbKeyManagerError::InvalidParameters(param.to_string())
        }
        other => RedbKeyManagerError::Crypto(other),
    }
}

/// RedB key manager for storage operations.
#[derive(Debug, Clone)]
pub struct RedbKeyManager {}

impl Default for RedbKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RedbKeyManager {
    /// Create a new RedB key manager.
    pub fn new() -> Self {
        Self {}
    }

    /// Create default KDF parameters with fresh random salt.
    pub fn create_default_kdf_params(&self) -> Result<KdfParams> {
        let mut salt = vec![0u8; 32];
        SystemRngProvider.rng().fill_bytes_ext(&mut salt);
        Ok(KdfParams::default_argon2id_with_salt(salt))
    }

    /// Generate a fresh 32-byte master key.
    pub fn generate_master_key(&self) -> Hidden<RedbKey32> {
        let mut key = [0u8; 32];
        SystemRngProvider.rng().fill_bytes_ext(&mut key);
        Hidden::hide(key)
    }

    /// Wrap and encrypt master key under a password-derived key.
    pub fn wrap_master_key(
        &self,
        wallet_id: &[u8],
        password: &SafePassword,
        master_key: &Hidden<RedbKey32>,
        kdf_params: &KdfParams,
    ) -> Result<MasterKeyRecord> {
        if kdf_params.version != KdfParams::VERSION {
            return Err(RedbKeyManagerError::InvalidParameters(format!(
                "unsupported KDF version: {}. Only current schema supported",
                kdf_params.version
            )));
        }

        kdf_params.validate_untrusted_persisted().map_err(|e| {
            RedbKeyManagerError::InvalidParameters(format!("weak KDF params: {}", e))
        })?;

        let pw_key = derive_pw_key(password, kdf_params).map_err(map_auth_error)?;

        let aad = aad_master_key(wallet_id);
        use z00z_crypto::aead::seal;
        let envelope_bytes =
            seal(pw_key.reveal(), &aad, master_key.reveal()).map_err(map_auth_error)?;
        let envelope = AeadEnvelope {
            envelope: envelope_bytes,
        };

        Ok(MasterKeyRecord {
            envelope,
            kdf_params: Some(kdf_params.clone()),
        })
    }

    fn validate_record_params(
        &self,
        kdf_params: &KdfParams,
        record: &MasterKeyRecord,
    ) -> Result<()> {
        let record_params = record.kdf_params.as_ref().ok_or_else(|| {
            RedbKeyManagerError::InvalidParameters("missing kdf params".to_string())
        })?;
        if record_params != kdf_params {
            return Err(RedbKeyManagerError::InvalidParameters(
                "kdf params mismatch".to_string(),
            ));
        }

        if kdf_params.version != KdfParams::VERSION {
            return Err(RedbKeyManagerError::InvalidParameters(format!(
                "unsupported KDF version: {}. Only current schema supported",
                kdf_params.version
            )));
        }

        kdf_params
            .validate_untrusted_persisted()
            .map_err(|e| RedbKeyManagerError::InvalidParameters(format!("weak KDF params: {}", e)))
    }

    fn decrypt_master_key_bytes(
        &self,
        wallet_id: &[u8],
        password: &SafePassword,
        kdf_params: &KdfParams,
        record: &MasterKeyRecord,
    ) -> Result<Zeroizing<Vec<u8>>> {
        let pw_key = derive_pw_key(password, kdf_params).map_err(map_auth_error)?;
        let aad = aad_master_key(wallet_id);
        use z00z_crypto::aead::open;

        open(pw_key.reveal(), &aad, &record.envelope.envelope)
            .map(Zeroizing::new)
            .map_err(map_auth_error)
    }

    fn key_from_plaintext(&self, plaintext: &[u8]) -> Result<Hidden<RedbKey32>> {
        if plaintext.len() != 32 {
            return Err(RedbKeyManagerError::InvalidParameters(
                "invalid key length".to_string(),
            ));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(plaintext);
        Ok(Hidden::hide(key))
    }

    /// Unwrap and decrypt a stored master key record.
    pub fn unwrap_master_key(
        &self,
        wallet_id: &[u8],
        password: &SafePassword,
        kdf_params: &KdfParams,
        record: &MasterKeyRecord,
    ) -> Result<Hidden<RedbKey32>> {
        self.validate_record_params(kdf_params, record)?;
        let plaintext = self.decrypt_master_key_bytes(wallet_id, password, kdf_params, record)?;
        self.key_from_plaintext(plaintext.as_slice())
    }

    /// Decrypt the persisted master key record.
    pub fn decrypt_master_key(
        &self,
        wallet_id: &[u8],
        password: &SafePassword,
        kdf_params: &KdfParams,
        record: &MasterKeyRecord,
    ) -> Result<Hidden<RedbKey32>> {
        self.unwrap_master_key(wallet_id, password, kdf_params, record)
    }

    /// Derive wallet keys (DATA/INDEX/INTEGRITY) from the master key.
    pub fn derive_wallet_keys(&self, master_key: &Hidden<RedbKey32>) -> Result<WalletDerivedKeys> {
        derive_wallet_keys(master_key.reveal()).map_err(RedbKeyManagerError::Crypto)
    }
}

include!("manager_redb_wallet.rs");
include!("test_manager_redb_suite.rs");
