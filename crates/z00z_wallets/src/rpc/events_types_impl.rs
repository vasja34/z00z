use super::{
    AssetEvent, Event, PersistWalletId, RuntimeEventFilter, SyncEvent, TransactionEvent,
    WalletEvent,
};

impl Default for RuntimeEventFilter {
    fn default() -> Self {
        Self {
            wallet_id: None,
            wallet_events: true,
            asset_events: true,
            transaction_events: true,
            sync_events: true,
        }
    }
}

impl Event {
    pub fn wallet_id(&self) -> &PersistWalletId {
        match self {
            Event::Wallet(event) => match event {
                WalletEvent::Created { wallet_id, .. }
                | WalletEvent::Unlocked { wallet_id, .. }
                | WalletEvent::Locked { wallet_id, .. }
                | WalletEvent::Deleted { wallet_id, .. }
                | WalletEvent::BalanceChanged { wallet_id, .. } => wallet_id,
            },
            Event::Asset(event) => match event {
                AssetEvent::Received { wallet_id, .. }
                | AssetEvent::Sent { wallet_id, .. }
                | AssetEvent::Merged { wallet_id, .. }
                | AssetEvent::Split { wallet_id, .. } => wallet_id,
            },
            Event::Transaction(event) => match event {
                TransactionEvent::Created { wallet_id, .. }
                | TransactionEvent::Pending { wallet_id, .. }
                | TransactionEvent::Confirmed { wallet_id, .. }
                | TransactionEvent::Failed { wallet_id, .. }
                | TransactionEvent::Cancelled { wallet_id, .. } => wallet_id,
            },
            Event::Sync(event) => match event {
                SyncEvent::Started { wallet_id, .. }
                | SyncEvent::Progress { wallet_id, .. }
                | SyncEvent::Completed { wallet_id, .. }
                | SyncEvent::Failed { wallet_id, .. }
                | SyncEvent::Paused { wallet_id, .. } => wallet_id,
            },
        }
    }

    pub fn timestamp(&self) -> u64 {
        match self {
            Event::Wallet(event) => match event {
                WalletEvent::Created { timestamp, .. }
                | WalletEvent::Unlocked { timestamp, .. }
                | WalletEvent::Locked { timestamp, .. }
                | WalletEvent::Deleted { timestamp, .. }
                | WalletEvent::BalanceChanged { timestamp, .. } => *timestamp,
            },
            Event::Asset(event) => match event {
                AssetEvent::Received { timestamp, .. }
                | AssetEvent::Sent { timestamp, .. }
                | AssetEvent::Merged { timestamp, .. }
                | AssetEvent::Split { timestamp, .. } => *timestamp,
            },
            Event::Transaction(event) => match event {
                TransactionEvent::Created { timestamp, .. }
                | TransactionEvent::Pending { timestamp, .. }
                | TransactionEvent::Confirmed { timestamp, .. }
                | TransactionEvent::Failed { timestamp, .. }
                | TransactionEvent::Cancelled { timestamp, .. } => *timestamp,
            },
            Event::Sync(event) => match event {
                SyncEvent::Started { timestamp, .. }
                | SyncEvent::Progress { timestamp, .. }
                | SyncEvent::Completed { timestamp, .. }
                | SyncEvent::Failed { timestamp, .. }
                | SyncEvent::Paused { timestamp, .. } => *timestamp,
            },
        }
    }
}

pub fn encode_event_cursor_ms(timestamp_ms: u64) -> String {
    timestamp_ms.to_string()
}

pub fn decode_event_cursor_ms(cursor: &str) -> Option<u64> {
    cursor.parse::<u64>().ok()
}
