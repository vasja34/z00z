//! Payment receipt storage.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use z00z_crypto::KernelSignature;

/// Payment receipt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Receipt {
    /// Receipt identifier.
    pub receipt_id: String,
    /// Transaction hash.
    pub tx_hash: String,
    /// Amount in native units.
    pub amount: u64,
    /// Recipient or receiver identifier.
    pub recipient: String,
    /// Receipt creation time (Unix time in milliseconds).
    pub timestamp_ms: u64,
    /// Optional proof-of-payment.
    pub proof: Option<KernelSignature>,
}

/// Receipt storage errors.
#[derive(Debug, Error)]
pub enum ReceiptStorageError {
    /// Receipt does not exist.
    #[error("receipt not found: {0}")]
    NotFound(String),

    /// Invalid receipt identifier.
    #[error("invalid receipt ID: {0}")]
    InvalidReceiptId(String),

    /// Backing store error.
    #[error("database error: {0}")]
    Database(String),

    /// I/O error from the underlying storage.
    #[error("I/O error: {0}")]
    Io(#[from] z00z_utils::io::IoError),

    /// Serialization failure.
    #[error("serialization error: {0}")]
    Serialization(String),
}

/// Receipt storage result type.
pub type ReceiptStorageResult<T> = std::result::Result<T, ReceiptStorageError>;

/// Receipt storage trait.
///
/// Implementation contract:
///
/// - Must use z00z_utils::io for file operations (NOT std::fs)
/// - Must use z00z_utils::codec::JsonCodec for serialization
/// - This trait is side-effect free and has no logging dependencies
/// - Must support efficient tx_hash lookup (index or scan)
/// - Must validate receipt_id uniqueness
pub trait ReceiptStorage {
    /// Store receipt.
    fn put(&mut self, receipt: Receipt) -> ReceiptStorageResult<()>;

    /// Get receipt by ID.
    fn get(&self, receipt_id: &str) -> ReceiptStorageResult<Receipt>;

    /// List all receipts.
    fn list(&self) -> ReceiptStorageResult<Vec<Receipt>>;

    /// Find receipts by transaction hash.
    fn find_by_tx(&self, tx_hash: &str) -> ReceiptStorageResult<Vec<Receipt>>;

    /// Delete receipt.
    fn delete(&mut self, receipt_id: &str) -> ReceiptStorageResult<()>;
}
