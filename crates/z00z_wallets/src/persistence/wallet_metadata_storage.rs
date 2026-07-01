//! Wallet metadata storage.

use crate::wallet::WalletRecord;
use thiserror::Error;
use z00z_utils::io::IoError;

/// Wallet storage errors.
#[derive(Debug, Error)]
pub enum WalletStorageError {
    /// Wallet metadata does not exist.
    #[error("wallet not found: {0}")]
    NotFound(String),

    /// Wallet metadata already exists.
    #[error("wallet already exists: {0}")]
    AlreadyExists(String),

    /// Invalid wallet identifier.
    #[error("invalid wallet id: {0}")]
    InvalidWalletId(String),

    /// I/O error from the underlying storage.
    #[error("I/O error: {0}")]
    Io(#[from] IoError),
}

/// Wallet storage result type.
pub type WalletStorageResult<T> = std::result::Result<T, WalletStorageError>;

/// Wallet storage trait.
///
/// Stores wallet metadata only. Secrets MUST NOT be stored here.
///
/// Implementation contract:
///
/// - Must use z00z_utils::io for file operations (NOT std::fs)
/// - Must use z00z_utils::codec::JsonCodec for serialization
/// - This trait is side-effect free and has no logging dependencies
/// - Must validate wallet_id to prevent path traversal attacks
/// - Must handle concurrent access (atomic writes)
pub trait WalletStorage {
    /// Save wallet record.
    fn save(&mut self, record: WalletRecord) -> WalletStorageResult<()>;

    /// Load wallet record.
    fn load(&self, wallet_id: &str) -> WalletStorageResult<WalletRecord>;

    /// List all wallets.
    fn list(&self) -> WalletStorageResult<Vec<WalletRecord>>;

    /// Delete wallet metadata.
    fn delete(&mut self, wallet_id: &str) -> WalletStorageResult<()>;

    /// Check whether a wallet exists.
    fn exists(&self, wallet_id: &str) -> bool;
}
