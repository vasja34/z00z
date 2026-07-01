//! Backup export abstraction.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use z00z_crypto::expert::encoding::SafePassword;

/// Backup metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupMetadata {
    /// Backup format version.
    pub version: u32,
    /// Creation time (Unix timestamp in milliseconds).
    pub created_at: u64,
    /// Wallet identifier.
    pub wallet_id: String,
    /// Network identifier.
    ///
    /// Format v4 redacts this field in the public header. Older formats may
    /// still expose the network in archived export packs.
    pub network: String,
}

/// Backup exporter errors.
#[derive(Debug, Error)]
pub enum BackupExporterError {
    /// Export failed.
    #[error("export failed: {0}")]
    ExportFailed(String),

    /// Encryption failed.
    #[error("encryption failed: {0}")]
    EncryptionFailed(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Backup format is invalid.
    #[error("invalid backup format: {0}")]
    InvalidFormat(String),

    /// Backup integrity check failed.
    #[error("integrity check failed")]
    IntegrityMismatch,
}

/// Backup exporter result type.
pub type BackupExporterResult<T> = std::result::Result<T, BackupExporterError>;

/// Backup exporter trait.
///
/// Implementation contract:
///
/// - Must use z00z_crypto::AEAD for backup encryption
/// - Must use z00z_crypto::Argon2 for password-based key derivation
/// - Must use z00z_utils::io for file operations (NOT std::fs)
/// - Must use z00z_utils::codec::JsonCodec for metadata serialization
/// - Must use z00z_utils::time::TimeProvider for timestamps
/// - Must include integrity hash in backup
pub trait BackupExporter {
    /// Export wallet to a file.
    fn export(&self, path: &str, password: &SafePassword) -> BackupExporterResult<BackupMetadata>;

    /// Export to encrypted bytes.
    fn export_to_bytes(&self, password: &SafePassword) -> BackupExporterResult<Vec<u8>>;

    /// Get backup metadata.
    fn get_metadata(&self) -> BackupExporterResult<BackupMetadata>;

    /// Verify backup integrity.
    fn verify_backup(&self, path: &str, password: &SafePassword) -> BackupExporterResult<bool>;
}
