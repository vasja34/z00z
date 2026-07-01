//! Transaction broadcast abstraction.

use thiserror::Error;

/// Broadcast errors.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum BroadcastError {
    /// Generic broadcast failure.
    #[error("broadcast failed: {0}")]
    Failed(String),

    /// Network error.
    #[error("network error: {0}")]
    Network(String),

    /// Transaction rejected by the node.
    #[error("transaction rejected: {0}")]
    Rejected(String),

    /// Transaction was replaced before confirmation.
    #[error("transaction replaced: {0}")]
    Replaced(String),

    /// Transaction lost confirmation due to a reorg or rollback.
    #[error("transaction reorged: {0}")]
    Reorg(String),

    /// Operation timed out.
    #[error("timeout")]
    Timeout,
}

/// Broadcast result type.
pub type BroadcastResultType<T> = std::result::Result<T, BroadcastError>;

/// Broadcast lifecycle contract over the canonical wallet chain seam.
pub trait Broadcast {
    /// Submit a serialized transaction and persist the initial lifecycle row.
    fn broadcast(&self, tx_bytes: &[u8]) -> BroadcastResultType<BroadcastResult>;

    /// Submit a serialized transaction with retry handling for transient failures.
    fn broadcast_with_retry(
        &self,
        tx_bytes: &[u8],
        max_retries: u32,
    ) -> BroadcastResultType<BroadcastResult>;

    /// Check whether a transaction is confirmed and persist terminal lifecycle changes.
    fn is_confirmed(&self, tx_hash: &str) -> BroadcastResultType<bool>;

    /// Poll for confirmation until a terminal status or timeout is reached.
    fn wait_for_confirmation(&self, tx_hash: &str, timeout_ms: u64) -> BroadcastResultType<u64>;
}

/// Broadcast result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BroadcastResult {
    /// Transaction hash.
    pub tx_hash: String,
    /// Submission time (Unix time in milliseconds).
    pub submitted_at: u64,
}
