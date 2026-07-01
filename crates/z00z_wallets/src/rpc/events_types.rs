//! Event types for real-time wallet notifications
//!
//! This module defines event structures for WebSocket-based event streaming.
//! Events allow UI to receive real-time updates without polling.
//!
//! # Architecture Compliance
//!
//! - ✅ Serializable types (serde Serialize/Deserialize)
//! - ✅ Timestamp tracking (Unix milliseconds)
//! - ✅ Wallet scoping (all events tied to PersistWalletId)
//! - ✅ Type safety (enums for event variants)
//!
//! # Event Categories
//!
//! 1. **WalletEvent** - Wallet lifecycle (created, unlocked, locked, deleted)
//! 2. **AssetEvent** - Asset operations (received, sent, merged, split)
//! 3. **TransactionEvent** - Transaction status (created, pending, confirmed, failed)
//! 4. **SyncEvent** - Blockchain sync (started, progress, completed, failed)

use serde::{Deserialize, Serialize};
use z00z_core::assets::registry::AssetId;

use super::common::{PersistTxId, PersistWalletId, RuntimePaginatedResponse};
use super::security::SessionToken;

#[path = "events_types_impl.rs"]
mod events_types_impl;

/// Wallet lifecycle events
///
/// Emitted when wallet state changes (creation, locking, deletion).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WalletEvent {
    /// Wallet created
    Created {
        /// Wallet identifier
        wallet_id: PersistWalletId,
        /// Wallet name
        name: String,
        /// Creation timestamp (Unix milliseconds)
        timestamp: u64,
    },
    /// Wallet unlocked
    Unlocked {
        /// Wallet identifier
        wallet_id: PersistWalletId,
        /// Session token
        session_token: SessionToken,
        /// Unlock timestamp (Unix milliseconds)
        timestamp: u64,
    },
    /// Wallet locked (manual or auto-lock)
    Locked {
        /// Wallet identifier
        wallet_id: PersistWalletId,
        /// Lock reason (manual, auto-lock, timeout)
        reason: String,
        /// Lock timestamp (Unix milliseconds)
        timestamp: u64,
    },
    /// Wallet deleted
    Deleted {
        /// Wallet identifier
        wallet_id: PersistWalletId,
        /// Deletion timestamp (Unix milliseconds)
        timestamp: u64,
    },
    /// Balance changed
    BalanceChanged {
        /// Wallet identifier
        wallet_id: PersistWalletId,
        /// Asset identifier
        asset_id: AssetId,
        /// Old balance
        old_balance: u64,
        /// New balance
        new_balance: u64,
        /// Change timestamp (Unix milliseconds)
        timestamp: u64,
    },
}

/// Asset operation events
///
/// Emitted when assets are received, sent, merged, or split.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AssetEvent {
    /// Asset received in wallet
    Received {
        /// Wallet identifier
        wallet_id: PersistWalletId,
        /// Asset identifier
        asset_id: AssetId,
        /// Amount received
        amount: u64,
        /// Associated transaction ID
        tx_id: PersistTxId,
        /// Receipt timestamp (Unix milliseconds)
        timestamp: u64,
    },
    /// Asset sent from wallet
    Sent {
        /// Wallet identifier
        wallet_id: PersistWalletId,
        /// Asset identifier
        asset_id: AssetId,
        /// Amount sent
        amount: u64,
        /// Associated transaction ID
        tx_id: PersistTxId,
        /// Recipient address
        recipient: String,
        /// Send timestamp (Unix milliseconds)
        timestamp: u64,
    },
    /// Assets merged (consolidation)
    Merged {
        /// Wallet identifier
        wallet_id: PersistWalletId,
        /// Asset identifier
        asset_id: AssetId,
        /// Number of inputs merged
        input_count: usize,
        /// Resulting amount
        output_amount: u64,
        /// Merge timestamp (Unix milliseconds)
        timestamp: u64,
    },
    /// Asset split into multiple outputs
    Split {
        /// Wallet identifier
        wallet_id: PersistWalletId,
        /// Asset identifier
        asset_id: AssetId,
        /// Input amount
        input_amount: u64,
        /// Number of outputs
        output_count: usize,
        /// Split timestamp (Unix milliseconds)
        timestamp: u64,
    },
}

/// Transaction lifecycle events
///
/// Emitted when transaction status changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransactionEvent {
    /// Transaction created (built, not yet broadcast)
    Created {
        /// Wallet identifier
        wallet_id: PersistWalletId,
        /// Transaction identifier
        tx_id: PersistTxId,
        /// Transaction amount
        amount: u64,
        /// Transaction fee
        fee: u64,
        /// Creation timestamp (Unix milliseconds)
        timestamp: u64,
    },
    /// Transaction broadcast to network (pending confirmation)
    Pending {
        /// Wallet identifier
        wallet_id: PersistWalletId,
        /// Transaction identifier
        tx_id: PersistTxId,
        /// Broadcast timestamp (Unix milliseconds)
        timestamp: u64,
    },
    /// Transaction confirmed on blockchain
    Confirmed {
        /// Wallet identifier
        wallet_id: PersistWalletId,
        /// Transaction identifier
        tx_id: PersistTxId,
        /// Block height
        block_height: u64,
        /// Number of confirmations
        confirmations: u32,
        /// Confirmation timestamp (Unix milliseconds)
        timestamp: u64,
    },
    /// Transaction failed
    Failed {
        /// Wallet identifier
        wallet_id: PersistWalletId,
        /// Transaction identifier
        tx_id: PersistTxId,
        /// Failure reason
        error: String,
        /// Failure timestamp (Unix milliseconds)
        timestamp: u64,
    },
    /// Transaction cancelled (before broadcast)
    Cancelled {
        /// Wallet identifier
        wallet_id: PersistWalletId,
        /// Transaction identifier
        tx_id: PersistTxId,
        /// Cancellation timestamp (Unix milliseconds)
        timestamp: u64,
    },
}

/// Blockchain synchronization events
///
/// Emitted during blockchain sync operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SyncEvent {
    /// Sync started
    Started {
        /// Wallet identifier
        wallet_id: PersistWalletId,
        /// Starting block height
        from_height: u64,
        /// Target block height
        to_height: u64,
        /// Start timestamp (Unix milliseconds)
        timestamp: u64,
    },
    /// Sync progress update
    Progress {
        /// Wallet identifier
        wallet_id: PersistWalletId,
        /// Current block height
        current_height: u64,
        /// Target block height
        target_height: u64,
        /// Progress ratio from 0.0 (0%) to 1.0 (100%)
        progress: Option<f32>,
        /// Estimated time remaining (seconds)
        eta_seconds: Option<u64>,
        /// Progress timestamp (Unix milliseconds)
        timestamp: u64,
    },
    /// Sync completed successfully
    Completed {
        /// Wallet identifier
        wallet_id: PersistWalletId,
        /// Final block height
        final_height: u64,
        /// Number of transactions found
        transactions_found: u64,
        /// Completion timestamp (Unix milliseconds)
        timestamp: u64,
    },
    /// Sync failed
    Failed {
        /// Wallet identifier
        wallet_id: PersistWalletId,
        /// Failure reason
        error: String,
        /// Last successful height
        last_height: u64,
        /// Failure timestamp (Unix milliseconds)
        timestamp: u64,
    },
    /// Sync paused
    Paused {
        /// Wallet identifier
        wallet_id: PersistWalletId,
        /// Current height when paused
        current_height: u64,
        /// Pause timestamp (Unix milliseconds)
        timestamp: u64,
    },
}

/// Event filter for subscription
///
/// Allows clients to subscribe only to specific event types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEventFilter {
    /// Filter by wallet ID (None = all wallets)
    pub wallet_id: Option<PersistWalletId>,
    /// Include wallet events
    pub wallet_events: bool,
    /// Include asset events
    pub asset_events: bool,
    /// Include transaction events
    pub transaction_events: bool,
    /// Include sync events
    pub sync_events: bool,
}

/// Unified event type for subscription streams
///
/// Wraps all event types in a single enum for WebSocket streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "category", rename_all = "snake_case")]
pub enum Event {
    /// Wallet lifecycle event
    Wallet(WalletEvent),
    /// Asset operation event
    Asset(AssetEvent),
    /// Transaction lifecycle event
    Transaction(TransactionEvent),
    /// Sync progress event
    Sync(SyncEvent),
}

/// Event history query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEventHistoryParams {
    /// Wallet identifier
    pub wallet_id: PersistWalletId,
    /// Start timestamp (Unix milliseconds, inclusive)
    pub from_timestamp: Option<u64>,
    /// End timestamp (Unix milliseconds, inclusive)
    pub to_timestamp: Option<u64>,
    /// Event filter
    pub filter: Option<RuntimeEventFilter>,
    /// Maximum number of events to return
    pub limit: usize,
}

/// Event history response (paginated).
///
/// Uses an opaque string cursor (consistent with other modules).
pub type EventHistoryResponse = RuntimePaginatedResponse<Event>;

pub use events_types_impl::{decode_event_cursor_ms, encode_event_cursor_ms};

#[cfg(test)]
#[path = "test_events_types.rs"]
mod tests;
