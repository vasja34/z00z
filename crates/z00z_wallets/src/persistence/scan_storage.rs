//! Blockchain scanning state storage.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Scan state data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanState {
    /// Last scanned block height.
    pub last_scanned_height: u64,
    /// Last scanned block hash.
    pub last_scanned_hash: String,
    /// Last scan time (Unix time in milliseconds).
    pub last_scan_timestamp_ms: u64,
    /// Whether scan is currently in progress.
    pub is_scanning: bool,
}

/// Scan state storage errors.
#[derive(Debug, Error)]
pub enum ScanStorageError {
    /// Scan state has not been initialized.
    #[error("scan state not found")]
    NotFound,

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

/// Scan-state storage result type.
pub type ScanStorageResult<T> = std::result::Result<T, ScanStorageError>;

/// Scan state storage trait.
///
/// Implementation contract:
///
/// - Must use z00z_utils::io for file operations (NOT std::fs)
/// - Must use z00z_utils::codec::JsonCodec for serialization
/// - This trait is side-effect free and has no logging dependencies
/// - Must use z00z_utils::time::TimeProvider for timestamps
/// - Must handle atomic updates (race conditions during sync)
pub trait ScanStorage {
    /// Save scan state.
    fn save(&mut self, state: ScanState) -> ScanStorageResult<()>;

    /// Load scan state.
    fn load(&self) -> ScanStorageResult<ScanState>;

    /// Update last scanned block.
    fn update_last_scanned(&mut self, height: u64, hash: String) -> ScanStorageResult<()>;

    /// Set scanning flag.
    fn set_scanning(&mut self, is_scanning: bool) -> ScanStorageResult<()>;

    /// Reset scan state.
    fn reset(&mut self) -> ScanStorageResult<()>;
}
