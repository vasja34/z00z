//! Blockchain scanning state storage.

#[path = "scan_storage.rs"]
mod scan_storage;
#[path = "scan_storage_impl.rs"]
mod scan_storage_impl;

pub use scan_storage::{ScanState, ScanStorage, ScanStorageError, ScanStorageResult};
pub use scan_storage_impl::ScanStorageImpl;
