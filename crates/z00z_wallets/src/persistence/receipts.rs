//! Payment receipt storage.

#[path = "receipt_storage.rs"]
mod receipt_storage;
#[path = "receipt_storage_impl.rs"]
mod receipt_storage_impl;

pub use receipt_storage::{Receipt, ReceiptStorage, ReceiptStorageError, ReceiptStorageResult};
pub use receipt_storage_impl::ReceiptStorageImpl;
