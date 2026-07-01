use async_trait::async_trait;

use crate::db::schema_keys::IndexTable;
use crate::WalletResult;

/// Logical KV tables for wallet persistence.
///
/// This is a wasm-safe abstraction that both native (RedB) and browser (IndexedDB)
/// backends can implement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalletKvTable {
    Meta,
    Secrets,
    Objects,
    IndexManifest,
    Index(IndexTable),
}

impl WalletKvTable {
    #[cfg(any(test, target_arch = "wasm32"))]
    pub(crate) fn store_name(self) -> &'static str {
        match self {
            Self::Meta => "meta",
            Self::Secrets => "secrets",
            Self::Objects => "objects",
            Self::IndexManifest => "index_manifest",
            Self::Index(table) => table.store_name(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalletTxnMode {
    ReadOnly,
    ReadWrite,
}

/// A transactional KV view.
///
/// The transaction must enforce atomicity for write operations on targets that support it
/// (RedB write transaction; IndexedDB readwrite transaction).
#[async_trait(?Send)]
pub trait WalletKvTxn {
    async fn get(&self, table: WalletKvTable, key: &[u8]) -> WalletResult<Option<Vec<u8>>>;

    async fn put(&mut self, table: WalletKvTable, key: &[u8], value: &[u8]) -> WalletResult<()>;

    async fn delete(&mut self, table: WalletKvTable, key: &[u8]) -> WalletResult<()>;

    async fn commit(self: Box<Self>) -> WalletResult<()>;

    async fn rollback(self: Box<Self>) -> WalletResult<()>;
}

/// KV backend that can create transactions.
#[async_trait(?Send)]
pub trait WalletKvBackend {
    async fn begin_txn(&self, mode: WalletTxnMode) -> WalletResult<Box<dyn WalletKvTxn>>;
}

/// Whole-container persistence boundary.
///
/// This abstraction is intentionally minimal: load the current blob (if any),
/// and atomically persist a new blob.
#[async_trait(?Send)]
pub trait WalletBlobBackend {
    async fn load_blob(&self) -> WalletResult<Option<Vec<u8>>>;

    async fn atomic_persist(&self, blob_bytes: &[u8]) -> WalletResult<()>;
}
