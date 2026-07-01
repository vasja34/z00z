#![cfg(not(target_arch = "wasm32"))]

use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use redb::{ReadableDatabase, TableDefinition, TableError};

use crate::db::wallet_store::WalletIo;
use crate::wasm::{WalletBlobBackend, WalletKvBackend, WalletKvTable, WalletKvTxn, WalletTxnMode};
use crate::{WalletError, WalletResult};

#[cfg(test)]
#[path = "test_redb_kv_backend.rs"]
mod tests;

const META_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("meta");
const SECRETS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("secrets");
const OBJECTS_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("objects");
const INDEX_MANIFEST_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("index_manifest");

fn index_table_def(store: &'static str) -> TableDefinition<'static, &'static [u8], &'static [u8]> {
    TableDefinition::new(store)
}

fn as_utf8_key(key: &[u8]) -> WalletResult<&str> {
    std::str::from_utf8(key).map_err(|_| WalletError::DatabaseError("invalid utf8 key".to_string()))
}

#[derive(Clone)]
pub(crate) struct RedbWalletKvBackend {
    db: Arc<redb::Database>,
}

impl RedbWalletKvBackend {
    pub(crate) fn new(db: Arc<redb::Database>) -> Self {
        Self { db }
    }
}

#[derive(Debug, Clone)]
enum PendingOp {
    Put {
        table: WalletKvTable,
        key: Vec<u8>,
        value: Vec<u8>,
    },
    Delete {
        table: WalletKvTable,
        key: Vec<u8>,
    },
}

#[derive(Debug)]
struct RedbWalletKvTxn {
    db: Arc<redb::Database>,
    mode: WalletTxnMode,
    pending: Vec<PendingOp>,
}

impl RedbWalletKvTxn {
    fn pending_get(&self, table: WalletKvTable, key: &[u8]) -> Option<Option<Vec<u8>>> {
        for op in self.pending.iter().rev() {
            match op {
                PendingOp::Put {
                    table: t,
                    key: k,
                    value,
                } if *t == table && k.as_slice() == key => return Some(Some(value.clone())),
                PendingOp::Delete { table: t, key: k } if *t == table && k.as_slice() == key => {
                    return Some(None);
                }
                _ => {}
            }
        }
        None
    }

    fn db_get(&self, table: WalletKvTable, key: &[u8]) -> WalletResult<Option<Vec<u8>>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| WalletError::DatabaseError(format!("redb begin_read failed: {e}")))?;

        match table {
            WalletKvTable::Meta => {
                let meta = match read_txn.open_table(META_TABLE) {
                    Ok(table) => table,
                    Err(TableError::TableDoesNotExist(_)) => return Ok(None),
                    Err(e) => {
                        return Err(WalletError::DatabaseError(format!(
                            "redb open_table(meta) failed: {e}"
                        )));
                    }
                };
                let key = as_utf8_key(key)?;
                Ok(meta
                    .get(key)
                    .map_err(|e| WalletError::DatabaseError(format!("redb get(meta) failed: {e}")))?
                    .map(|v| v.value().to_vec()))
            }
            WalletKvTable::Secrets => {
                let secrets = match read_txn.open_table(SECRETS_TABLE) {
                    Ok(table) => table,
                    Err(TableError::TableDoesNotExist(_)) => return Ok(None),
                    Err(e) => {
                        return Err(WalletError::DatabaseError(format!(
                            "redb open_table(secrets) failed: {e}"
                        )));
                    }
                };
                let key = as_utf8_key(key)?;
                Ok(secrets
                    .get(key)
                    .map_err(|e| {
                        WalletError::DatabaseError(format!("redb get(secrets) failed: {e}"))
                    })?
                    .map(|v| v.value().to_vec()))
            }
            WalletKvTable::Objects => {
                let objects = match read_txn.open_table(OBJECTS_TABLE) {
                    Ok(table) => table,
                    Err(TableError::TableDoesNotExist(_)) => return Ok(None),
                    Err(e) => {
                        return Err(WalletError::DatabaseError(format!(
                            "redb open_table(objects) failed: {e}"
                        )));
                    }
                };
                Ok(objects
                    .get(key)
                    .map_err(|e| {
                        WalletError::DatabaseError(format!("redb get(objects) failed: {e}"))
                    })?
                    .map(|v| v.value().to_vec()))
            }
            WalletKvTable::IndexManifest => {
                let idx = match read_txn.open_table(INDEX_MANIFEST_TABLE) {
                    Ok(table) => table,
                    Err(TableError::TableDoesNotExist(_)) => return Ok(None),
                    Err(e) => {
                        return Err(WalletError::DatabaseError(format!(
                            "redb open_table(index_manifest) failed: {e}"
                        )));
                    }
                };
                Ok(idx
                    .get(key)
                    .map_err(|e| {
                        WalletError::DatabaseError(format!("redb get(index_manifest) failed: {e}"))
                    })?
                    .map(|v| v.value().to_vec()))
            }
            WalletKvTable::Index(_) => {
                let def = index_table_def(table.store_name());
                let table = match read_txn.open_table(def) {
                    Ok(table) => table,
                    Err(TableError::TableDoesNotExist(_)) => return Ok(None),
                    Err(e) => {
                        return Err(WalletError::DatabaseError(format!(
                            "redb open_table(index) failed: {e}"
                        )));
                    }
                };
                Ok(table
                    .get(key)
                    .map_err(|e| {
                        WalletError::DatabaseError(format!("redb get(index) failed: {e}"))
                    })?
                    .map(|v| v.value().to_vec()))
            }
        }
    }

    fn apply_ops(self) -> WalletResult<()> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| WalletError::DatabaseError(format!("redb begin_write failed: {e}")))?;

        for op in &self.pending {
            match op {
                PendingOp::Put { table, key, value } => match *table {
                    WalletKvTable::Meta => {
                        let mut t = write_txn.open_table(META_TABLE).map_err(|e| {
                            WalletError::DatabaseError(format!("redb open_table(meta) failed: {e}"))
                        })?;
                        let key = as_utf8_key(key)?;
                        t.insert(key, value.as_slice()).map_err(|e| {
                            WalletError::DatabaseError(format!("redb insert(meta) failed: {e}"))
                        })?;
                    }
                    WalletKvTable::Secrets => {
                        let mut t = write_txn.open_table(SECRETS_TABLE).map_err(|e| {
                            WalletError::DatabaseError(format!(
                                "redb open_table(secrets) failed: {e}"
                            ))
                        })?;
                        let key = as_utf8_key(key)?;
                        t.insert(key, value.as_slice()).map_err(|e| {
                            WalletError::DatabaseError(format!("redb insert(secrets) failed: {e}"))
                        })?;
                    }
                    WalletKvTable::Objects => {
                        let mut t = write_txn.open_table(OBJECTS_TABLE).map_err(|e| {
                            WalletError::DatabaseError(format!(
                                "redb open_table(objects) failed: {e}"
                            ))
                        })?;
                        t.insert(key.as_slice(), value.as_slice()).map_err(|e| {
                            WalletError::DatabaseError(format!("redb insert(objects) failed: {e}"))
                        })?;
                    }
                    WalletKvTable::IndexManifest => {
                        let mut t = write_txn.open_table(INDEX_MANIFEST_TABLE).map_err(|e| {
                            WalletError::DatabaseError(format!(
                                "redb open_table(index_manifest) failed: {e}"
                            ))
                        })?;
                        t.insert(key.as_slice(), value.as_slice()).map_err(|e| {
                            WalletError::DatabaseError(format!(
                                "redb insert(index_manifest) failed: {e}"
                            ))
                        })?;
                    }
                    WalletKvTable::Index(_) => {
                        let def = index_table_def(table.store_name());
                        let mut t = write_txn.open_table(def).map_err(|e| {
                            WalletError::DatabaseError(format!(
                                "redb open_table(index) failed: {e}"
                            ))
                        })?;
                        t.insert(key.as_slice(), value.as_slice()).map_err(|e| {
                            WalletError::DatabaseError(format!("redb insert(index) failed: {e}"))
                        })?;
                    }
                },
                PendingOp::Delete { table, key } => match *table {
                    WalletKvTable::Meta => {
                        let mut t = write_txn.open_table(META_TABLE).map_err(|e| {
                            WalletError::DatabaseError(format!("redb open_table(meta) failed: {e}"))
                        })?;
                        let key = as_utf8_key(key)?;
                        let _ = t.remove(key).map_err(|e| {
                            WalletError::DatabaseError(format!("redb remove(meta) failed: {e}"))
                        })?;
                    }
                    WalletKvTable::Secrets => {
                        let mut t = write_txn.open_table(SECRETS_TABLE).map_err(|e| {
                            WalletError::DatabaseError(format!(
                                "redb open_table(secrets) failed: {e}"
                            ))
                        })?;
                        let key = as_utf8_key(key)?;
                        let _ = t.remove(key).map_err(|e| {
                            WalletError::DatabaseError(format!("redb remove(secrets) failed: {e}"))
                        })?;
                    }
                    WalletKvTable::Objects => {
                        let mut t = write_txn.open_table(OBJECTS_TABLE).map_err(|e| {
                            WalletError::DatabaseError(format!(
                                "redb open_table(objects) failed: {e}"
                            ))
                        })?;
                        let _ = t.remove(key.as_slice()).map_err(|e| {
                            WalletError::DatabaseError(format!("redb remove(objects) failed: {e}"))
                        })?;
                    }
                    WalletKvTable::IndexManifest => {
                        let mut t = write_txn.open_table(INDEX_MANIFEST_TABLE).map_err(|e| {
                            WalletError::DatabaseError(format!(
                                "redb open_table(index_manifest) failed: {e}"
                            ))
                        })?;
                        let _ = t.remove(key.as_slice()).map_err(|e| {
                            WalletError::DatabaseError(format!(
                                "redb remove(index_manifest) failed: {e}"
                            ))
                        })?;
                    }
                    WalletKvTable::Index(_) => {
                        let def = index_table_def(table.store_name());
                        let mut t = write_txn.open_table(def).map_err(|e| {
                            WalletError::DatabaseError(format!(
                                "redb open_table(index) failed: {e}"
                            ))
                        })?;
                        let _ = t.remove(key.as_slice()).map_err(|e| {
                            WalletError::DatabaseError(format!("redb remove(index) failed: {e}"))
                        })?;
                    }
                },
            }
        }

        write_txn
            .commit()
            .map_err(|e| WalletError::DatabaseError(format!("redb commit failed: {e}")))?;
        Ok(())
    }
}

#[async_trait(?Send)]
impl WalletKvTxn for RedbWalletKvTxn {
    async fn get(&self, table: WalletKvTable, key: &[u8]) -> WalletResult<Option<Vec<u8>>> {
        if let Some(v) = self.pending_get(table, key) {
            return Ok(v);
        }
        self.db_get(table, key)
    }

    async fn put(&mut self, table: WalletKvTable, key: &[u8], value: &[u8]) -> WalletResult<()> {
        if self.mode != WalletTxnMode::ReadWrite {
            return Err(WalletError::DatabaseError(
                "put not allowed in read-only transaction".to_string(),
            ));
        }
        self.pending.push(PendingOp::Put {
            table,
            key: key.to_vec(),
            value: value.to_vec(),
        });
        Ok(())
    }

    async fn delete(&mut self, table: WalletKvTable, key: &[u8]) -> WalletResult<()> {
        if self.mode != WalletTxnMode::ReadWrite {
            return Err(WalletError::DatabaseError(
                "delete not allowed in read-only transaction".to_string(),
            ));
        }
        self.pending.push(PendingOp::Delete {
            table,
            key: key.to_vec(),
        });
        Ok(())
    }

    async fn commit(self: Box<Self>) -> WalletResult<()> {
        self.apply_ops()
    }

    async fn rollback(self: Box<Self>) -> WalletResult<()> {
        Ok(())
    }
}

#[async_trait(?Send)]
impl WalletKvBackend for RedbWalletKvBackend {
    async fn begin_txn(&self, mode: WalletTxnMode) -> WalletResult<Box<dyn WalletKvTxn>> {
        Ok(Box::new(RedbWalletKvTxn {
            db: Arc::clone(&self.db),
            mode,
            pending: Vec::new(),
        }))
    }
}

#[derive(Clone)]
pub(crate) struct FileWalletBlobBackend {
    path: PathBuf,
    io: Arc<dyn WalletIo>,
}

impl FileWalletBlobBackend {}

#[async_trait(?Send)]
impl WalletBlobBackend for FileWalletBlobBackend {
    async fn load_blob(&self) -> WalletResult<Option<Vec<u8>>> {
        if !self.io.path_exists(&self.path)? {
            return Ok(None);
        }
        let bytes = self.io.read_file(&self.path)?;
        Ok(Some(bytes))
    }

    async fn atomic_persist(&self, blob_bytes: &[u8]) -> WalletResult<()> {
        crate::db::wallet_io::atomic_write_file_private(&self.path, blob_bytes)
    }
}
