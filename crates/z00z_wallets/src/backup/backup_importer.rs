//! Backup import abstraction.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use z00z_crypto::expert::encoding::SafePassword;

use crate::wallet::persistence::WalletExportPack;

use super::backup_wire::{ForensicImportMode, WalletForensicPack};

/// Imported wallet data for the current chain-bound backup contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedWalletData {
    /// Wallet identifier.
    pub wallet_id: String,
    /// Network identifier.
    pub network: String,
    /// Chain identifier.
    pub chain: String,
    /// Full restore payload for the current backup format.
    pub export_pack: Option<WalletExportPack>,
    /// Optional forensic archive payload.
    pub forensic_archive: Option<WalletForensicPack>,
    /// Encrypted/serialized keys blob.
    pub keys: Vec<u8>,
    /// Canonical tx-history JSONL blob preserved as raw bytes.
    pub transactions: Vec<u8>,
    /// Serialized owned-asset payload set from the encrypted export pack.
    pub assets: Vec<u8>,
}

/// Backup metadata.
///
/// Same format as the exporter.
pub use super::backup_exporter::BackupMetadata;

/// Backup importer errors.
#[derive(Debug, Error)]
pub enum BackupImporterError {
    /// Import failed.
    #[error("import failed: {0}")]
    ImportFailed(String),

    /// Decryption failed.
    #[error("decryption failed: {0}")]
    DecryptionFailed(String),

    /// Backup format is invalid.
    #[error("invalid backup format: {0}")]
    InvalidFormat(String),

    /// Backup version mismatch.
    #[error("version mismatch: expected {expected}, found {found}")]
    VersionMismatch {
        /// Expected backup version.
        expected: u32,
        /// Actual backup version found in the container.
        found: u32,
    },

    /// Backup integrity check failed.
    #[error("integrity check failed")]
    IntegrityMismatch,

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Deserialization error.
    #[error("deserialization error: {0}")]
    Deserialization(String),
}

/// Backup importer result type.
pub type BackupImporterResult<T> = std::result::Result<T, BackupImporterError>;

/// Backup importer trait.
///
/// Implementation contract:
///
/// - Must use z00z_crypto::AEAD for backup decryption
/// - Must use z00z_crypto::Argon2 for password-based key derivation
/// - Must use z00z_utils::io for file operations (NOT std::fs)
/// - Must use z00z_utils::codec::JsonCodec for metadata deserialization
/// - Must verify backup integrity hash
/// - Must validate backup version compatibility
pub trait BackupImporter {
    /// Import wallet from a file.
    fn import_with_mode(
        &self,
        path: &str,
        password: &SafePassword,
        mode: ForensicImportMode,
    ) -> BackupImporterResult<ImportedWalletData>;

    /// Import from encrypted bytes.
    fn import_from_bytes_with_mode(
        &self,
        data: &[u8],
        password: &SafePassword,
        mode: ForensicImportMode,
    ) -> BackupImporterResult<ImportedWalletData>;

    /// Import wallet from a file using the canonical wallet-only mode.
    fn import(
        &self,
        path: &str,
        password: &SafePassword,
    ) -> BackupImporterResult<ImportedWalletData> {
        self.import_with_mode(path, password, ForensicImportMode::WalletOnly)
    }

    /// Import from encrypted bytes using the canonical wallet-only mode.
    fn import_from_bytes(
        &self,
        data: &[u8],
        password: &SafePassword,
    ) -> BackupImporterResult<ImportedWalletData> {
        self.import_from_bytes_with_mode(data, password, ForensicImportMode::WalletOnly)
    }

    /// Read backup metadata without decrypting.
    fn read_metadata(&self, path: &str) -> BackupImporterResult<BackupMetadata>;

    /// Verify backup password.
    fn verify_password(&self, path: &str, password: &SafePassword) -> BackupImporterResult<bool>;
}
