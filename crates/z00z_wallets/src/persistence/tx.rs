//! Transaction history storage.

#[path = "tx_storage.rs"]
mod tx_storage;
#[path = "tx_storage_impl.rs"]
mod tx_storage_impl;

pub use tx_storage::{
    TxConfirmationEvidence, TxPolicySpendWindow, TxRecord, TxStatus, TxStorage, TxStorageError,
    TxStorageResult,
};
pub(crate) use tx_storage_impl::tx_history_path_lock;
pub use tx_storage_impl::TxStorageImpl;
pub(crate) use tx_storage_impl::MAX_TX_HISTORY_JSONL_BYTES;
