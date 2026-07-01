//! Persistent storages.
//!
//! This module contains wallet persistence abstractions (traits + error types).
//! Implementations are added separately for the active native and browser backends.

pub mod receipts;
pub mod scans;
pub mod tracked_asset_storage;
pub mod tracked_asset_storage_impl;
pub mod tx;
pub mod wallet_metadata_storage;
pub mod wallet_metadata_storage_impl;

pub use receipts::{
    Receipt, ReceiptStorage, ReceiptStorageError, ReceiptStorageImpl, ReceiptStorageResult,
};
pub use scans::{ScanState, ScanStorage, ScanStorageError, ScanStorageImpl, ScanStorageResult};
pub use tracked_asset_storage::{AssetStorage, AssetStorageError, AssetStorageResult};
pub use tracked_asset_storage_impl::AssetStorageImpl;
pub use tx::{
    TxConfirmationEvidence, TxPolicySpendWindow, TxRecord, TxStatus, TxStorage, TxStorageError,
    TxStorageImpl, TxStorageResult,
};
pub use wallet_metadata_storage::{WalletStorage, WalletStorageError, WalletStorageResult};
pub use wallet_metadata_storage_impl::WalletStorageImpl;
