//! Transaction history storage.

use crate::backup::{WalletTxHistoryEntryKind, WalletTxHistoryJsonlEntry};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Transaction status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TxStatus {
    /// Transaction created but not yet confirmed.
    Pending,
    /// Transaction is confirmed on-chain.
    Confirmed,
    /// Transaction failed (rejected, invalid, or dropped).
    Failed,
    /// Transaction was cancelled before confirmation.
    Cancelled,
}

/// Typed checkpoint evidence proving a pending wallet transaction was confirmed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxConfirmationEvidence {
    /// Stored wallet transaction id.
    pub tx_id: String,
    /// Canonical transaction digest without the wallet `tx_` prefix.
    pub tx_hash_hex: String,
    /// Chain id the confirmation evidence belongs to.
    pub chain_id: u32,
    /// Confirming checkpoint height.
    pub block_height: u64,
    /// Checkpoint id, hex-encoded.
    pub checkpoint_id_hex: String,
    /// Previous state root, hex-encoded.
    pub prev_root_hex: String,
    /// New state root, hex-encoded.
    pub new_root_hex: String,
    /// Input asset ids proven deleted by the checkpoint.
    pub spent_asset_ids_hex: Vec<String>,
    /// Output asset ids proven created by the checkpoint.
    pub created_asset_ids_hex: Vec<String>,
    /// Confirmation timestamp (Unix milliseconds).
    pub confirmed_at: u64,
    /// Whether evidence verification succeeded at the admission boundary.
    pub verified: bool,
}

/// Aggregated spend facts derived from the durable tx-history journal.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TxPolicySpendWindow {
    /// Total recipient-side amount for the queried asset within the active day
    /// window, across locally-originated pending or confirmed transactions.
    pub spent_amount: u64,
    /// Number of locally-originated pending spend transactions that still
    /// require confirmation before a new send may proceed.
    pub pending_confirmation_count: usize,
}

/// Stored transaction record.
///
/// NOTE: `tx_bytes` is an opaque payload for now, until the canonical protocol-level
/// transaction type is stabilized in `z00z_core`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxRecord {
    /// Transaction identifier/hash.
    pub tx_hash: String,
    /// Serialized transaction bytes.
    pub tx_bytes: Vec<u8>,
    /// True when the wallet learned about the transaction via portable import.
    #[serde(default)]
    pub imported: bool,
    /// Current transaction status.
    pub status: TxStatus,
    /// Record timestamp (Unix time in milliseconds).
    pub timestamp_ms: u64,
    /// Optional block height for confirmed transactions.
    pub block_height: Option<u64>,
    /// Optional typed confirmation evidence attached by wallet reconciliation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmation_evidence: Option<TxConfirmationEvidence>,
}

/// Transaction storage errors.
#[derive(Debug, Error)]
pub enum TxStorageError {
    /// Record does not exist.
    #[error("transaction not found: {0}")]
    NotFound(String),

    /// Invalid transaction hash identifier.
    #[error("invalid tx hash: {0}")]
    InvalidTxHash(String),

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

/// Transaction storage result type.
pub type TxStorageResult<T> = std::result::Result<T, TxStorageError>;

/// Transaction storage trait.
///
/// Implementation requirements:
/// - use `z00z_utils::io` for file operations instead of `std::fs`
/// - use `z00z_utils::codec::JsonCodec` for serialization
/// - remain side-effect free and logging-independent at the trait boundary
/// - use `z00z_utils::time::TimeProvider` for timestamps
/// - support efficient status-based queries
/// - handle transaction-hash collisions
pub trait TxStorage {
    /// Store a transaction record.
    fn put(&mut self, record: TxRecord) -> TxStorageResult<()>;

    /// Store an externally imported transaction record.
    fn record_imported(&mut self, record: TxRecord) -> TxStorageResult<()>;

    /// Record that a transaction package was exported from the wallet.
    fn record_exported(&mut self, tx_hash: &str) -> TxStorageResult<()>;

    /// Get a transaction record by hash.
    fn get(&self, tx_hash: &str) -> TxStorageResult<TxRecord>;

    /// List all transaction records.
    fn list(&self) -> TxStorageResult<Vec<TxRecord>>;

    /// List physical append-only tx-history rows in sequence order.
    fn list_history_rows(&self) -> TxStorageResult<Vec<WalletTxHistoryJsonlEntry>>;

    /// List transaction records by status.
    fn list_by_status(&self, status: TxStatus) -> TxStorageResult<Vec<TxRecord>>;

    /// Update a transaction status.
    fn update_status(&mut self, tx_hash: &str, status: TxStatus) -> TxStorageResult<()>;

    /// Record that a pending transaction was submitted to an admission boundary.
    fn record_submitted(&mut self, tx_hash: &str) -> TxStorageResult<()>;

    /// Record that a pending transaction was accepted by an admission boundary.
    fn record_admitted(&mut self, tx_hash: &str) -> TxStorageResult<()>;

    /// Record confirmed transaction evidence.
    fn record_confirmed(&mut self, tx_hash: &str, block_height: u64) -> TxStorageResult<()>;

    /// Record that a transaction reached a terminal failed state.
    fn record_failed(&mut self, tx_hash: &str) -> TxStorageResult<()> {
        self.update_status(tx_hash, TxStatus::Failed)
    }

    /// Record that a transaction conflicted with durable wallet or asset state.
    fn record_conflicted(&mut self, tx_hash: &str) -> TxStorageResult<()>;

    /// Record that a transaction tried to spend an already-spent input.
    fn record_already_spent(&mut self, tx_hash: &str) -> TxStorageResult<()>;

    /// Record full typed confirmation evidence for a pending transaction.
    fn record_confirmation_evidence(
        &mut self,
        tx_hash: &str,
        evidence: TxConfirmationEvidence,
    ) -> TxStorageResult<()> {
        let _ = evidence;
        self.record_confirmed(tx_hash, evidence.block_height)
    }

    /// Record cancelled transaction evidence.
    fn record_cancelled(&mut self, tx_hash: &str) -> TxStorageResult<()>;

    /// Append one compensating row that restores the exact prior snapshot kind.
    fn restore_snapshot(
        &mut self,
        _record: TxRecord,
        _latest_kind: WalletTxHistoryEntryKind,
    ) -> TxStorageResult<()> {
        Err(TxStorageError::Database(
            "restore_snapshot is not implemented for this tx store".to_string(),
        ))
    }

    /// Delete a transaction record.
    fn delete(&mut self, tx_hash: &str) -> TxStorageResult<()>;
}
