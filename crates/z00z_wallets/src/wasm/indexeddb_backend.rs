#![cfg(target_arch = "wasm32")]

use async_trait::async_trait;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use js_sys::Uint8Array;
use wasm_bindgen::JsValue;

use rexie::{ObjectStore, Rexie, TransactionMode};

use crate::wasm::{WalletBlobBackend, WalletKvBackend, WalletKvTable, WalletKvTxn, WalletTxnMode};
use crate::{WalletError, WalletResult};

const KV_STORE: &str = "kv";
const BLOB_STORE: &str = "blob";
const LEASE_STORE: &str = "lease";

const LEASE_KEY: &str = "global";
const BLOB_KEY: &str = "main";

fn db_err(e: impl std::fmt::Display) -> WalletError {
    WalletError::DatabaseError(format!("indexeddb error: {e}"))
}

fn kv_key(table: WalletKvTable, key: &[u8]) -> String {
    let encoded = URL_SAFE_NO_PAD.encode(key);
    format!("{}:{}", table.store_name(), encoded)
}

#[derive(Debug, Clone)]
pub struct IndexedDbWalletBackend {
    db: Rexie,
}

impl IndexedDbWalletBackend {
    pub async fn open(name: &str, version: u32) -> WalletResult<Self> {
        let version = version.max(2);
        let db = Rexie::builder(name)
            .version(version)
            .add_object_store(ObjectStore::new(KV_STORE))
            .add_object_store(ObjectStore::new(BLOB_STORE))
            .add_object_store(ObjectStore::new(LEASE_STORE))
            .build()
            .await
            .map_err(db_err)?;
        Ok(Self { db })
    }
}

struct IndexedDbTxn {
    mode: WalletTxnMode,
    tx: Option<rexie::Transaction>,
    kv: rexie::Store,
}

impl IndexedDbTxn {
    fn ensure_write(&self) -> WalletResult<()> {
        if self.mode != WalletTxnMode::ReadWrite {
            return Err(WalletError::DatabaseError(
                "write not allowed in read-only transaction".to_string(),
            ));
        }
        Ok(())
    }
}

#[async_trait(?Send)]
impl WalletKvTxn for IndexedDbTxn {
    async fn get(&self, table: WalletKvTable, key: &[u8]) -> WalletResult<Option<Vec<u8>>> {
        let k = kv_key(table, key);
        let key = JsValue::from_str(&k);
        let v = self.kv.get(&key).await.map_err(db_err)?;

        if v.is_undefined() {
            return Ok(None);
        }

        Ok(Some(Uint8Array::new(&v).to_vec()))
    }

    async fn put(&mut self, table: WalletKvTable, key: &[u8], value: &[u8]) -> WalletResult<()> {
        self.ensure_write()?;
        let k = kv_key(table, key);
        let v = Uint8Array::from(value).into();
        let key = JsValue::from_str(&k);
        let _ = self.kv.put(&v, Some(&key)).await.map_err(db_err)?;
        Ok(())
    }

    async fn delete(&mut self, table: WalletKvTable, key: &[u8]) -> WalletResult<()> {
        self.ensure_write()?;
        let k = kv_key(table, key);
        let key = JsValue::from_str(&k);
        self.kv.delete(&key).await.map_err(db_err)?;
        Ok(())
    }

    async fn commit(mut self: Box<Self>) -> WalletResult<()> {
        let Some(tx) = self.tx.take() else {
            return Ok(());
        };
        tx.done().await.map_err(db_err)
    }

    async fn rollback(mut self: Box<Self>) -> WalletResult<()> {
        let Some(tx) = self.tx.take() else {
            return Ok(());
        };
        tx.abort().await.map_err(db_err)
    }
}

#[async_trait(?Send)]
impl WalletKvBackend for IndexedDbWalletBackend {
    async fn begin_txn(&self, mode: WalletTxnMode) -> WalletResult<Box<dyn WalletKvTxn>> {
        let tx_mode = match mode {
            WalletTxnMode::ReadOnly => TransactionMode::ReadOnly,
            WalletTxnMode::ReadWrite => TransactionMode::ReadWrite,
        };
        let stores: &[&str] = match mode {
            WalletTxnMode::ReadOnly => &[KV_STORE],
            WalletTxnMode::ReadWrite => &[LEASE_STORE, KV_STORE],
        };

        let tx = self.db.transaction(stores, tx_mode).map_err(db_err)?;

        if mode == WalletTxnMode::ReadWrite {
            let lease = tx.store(LEASE_STORE).map_err(db_err)?;
            let key = JsValue::from_str(LEASE_KEY);
            let _ = lease.get(&key).await.map_err(db_err)?;
        }

        let kv = tx.store(KV_STORE).map_err(db_err)?;
        Ok(Box::new(IndexedDbTxn {
            mode,
            tx: Some(tx),
            kv,
        }))
    }
}

#[async_trait(?Send)]
impl WalletBlobBackend for IndexedDbWalletBackend {
    async fn load_blob(&self) -> WalletResult<Option<Vec<u8>>> {
        let tx = self
            .db
            .transaction(&[BLOB_STORE], TransactionMode::ReadOnly)
            .map_err(db_err)?;
        let store = tx.store(BLOB_STORE).map_err(db_err)?;
        let key = JsValue::from_str(BLOB_KEY);
        let v = store.get(&key).await.map_err(db_err)?;

        let res = if v.is_undefined() {
            None
        } else {
            Some(Uint8Array::new(&v).to_vec())
        };

        tx.done().await.map_err(db_err)?;
        Ok(res)
    }

    async fn atomic_persist(&self, blob_bytes: &[u8]) -> WalletResult<()> {
        let tx = self
            .db
            .transaction(&[LEASE_STORE, BLOB_STORE], TransactionMode::ReadWrite)
            .map_err(db_err)?;

        let lease = tx.store(LEASE_STORE).map_err(db_err)?;
        let key = JsValue::from_str(LEASE_KEY);
        let _ = lease.get(&key).await.map_err(db_err)?;

        let store = tx.store(BLOB_STORE).map_err(db_err)?;
        let v: JsValue = Uint8Array::from(blob_bytes).into();
        let key = JsValue::from_str(BLOB_KEY);
        let _ = store.put(&v, Some(&key)).await.map_err(db_err)?;
        tx.done().await.map_err(db_err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{cell::Cell, rc::Rc};

    use wasm_bindgen::{closure::Closure, JsCast};
    use wasm_bindgen_futures::JsFuture;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    fn assert_impls<T: WalletKvBackend + ?Sized>() {}
    fn assert_blob_impls<T: WalletBlobBackend + ?Sized>() {}

    #[test]
    fn test_indexeddb_impls_traits() {
        assert_impls::<IndexedDbWalletBackend>();
        assert_blob_impls::<IndexedDbWalletBackend>();
    }

    wasm_bindgen_test_configure!(run_in_browser);

    fn unique_db_name(prefix: &str) -> String {
        let now_ms = js_sys::Date::now() as u64;
        format!("{prefix}_{now_ms}")
    }

    async fn yield_task() {
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            let window = web_sys::window().expect("window");
            let resolve_fn = resolve.clone();

            let cb = Closure::once_into_js(move || {
                let _ = resolve_fn.call0(&JsValue::UNDEFINED);
            });

            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(cb.unchecked_ref(), 0)
                .expect("setTimeout");
        });

        let _ = JsFuture::from(promise).await;
    }

    #[wasm_bindgen_test(async)]
    async fn indexeddb_kv_rollback_discards() {
        let name = unique_db_name("z00z_wallet_kv_rollback");
        let backend = IndexedDbWalletBackend::open(&name, 1).await.expect("open");

        let mut write = backend
            .begin_txn(WalletTxnMode::ReadWrite)
            .await
            .expect("begin write txn");

        write
            .put(WalletKvTable::Meta, b"wallet.id", b"abc")
            .await
            .expect("put");
        write.rollback().await.expect("rollback");

        let read = backend
            .begin_txn(WalletTxnMode::ReadOnly)
            .await
            .expect("begin read txn");
        let v = read
            .get(WalletKvTable::Meta, b"wallet.id")
            .await
            .expect("get");
        assert!(v.is_none());
        read.commit().await.expect("commit read txn");

        let _ = Rexie::delete(&name).await;
    }

    #[wasm_bindgen_test(async)]
    async fn indexeddb_blob_roundtrip() {
        let name = unique_db_name("z00z_wallet_blob_roundtrip");
        let backend = IndexedDbWalletBackend::open(&name, 1).await.expect("open");

        backend.atomic_persist(b"hello").await.expect("persist");
        let v = backend.load_blob().await.expect("load");
        assert_eq!(v.as_deref(), Some(b"hello".as_slice()));

        let _ = Rexie::delete(&name).await;
    }

    #[wasm_bindgen_test(async)]
    async fn indexeddb_writer_blocks_blob() {
        let name = unique_db_name("z00z_wallet_single_writer");
        let backend = IndexedDbWalletBackend::open(&name, 1).await.expect("open");

        let mut write = backend
            .begin_txn(WalletTxnMode::ReadWrite)
            .await
            .expect("begin write txn");

        write
            .put(WalletKvTable::Meta, b"wallet.id", b"abc")
            .await
            .expect("put");

        let done = Rc::new(Cell::new(false));
        let done_flag = done.clone();
        let backend_for_task = backend.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = backend_for_task.atomic_persist(b"hello").await;
            done_flag.set(true);
        });

        yield_task().await;

        assert!(!done.get(), "blob write should be blocked by KV write txn");

        write.commit().await.expect("commit write txn");

        for _ in 0..200 {
            if done.get() {
                break;
            }
            yield_task().await;
        }

        assert!(done.get(), "blob write should complete after KV txn commit");

        let _ = Rexie::delete(&name).await;
    }
}
