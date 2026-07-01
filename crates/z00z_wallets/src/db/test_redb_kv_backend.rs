use super::*;
use tempfile::TempDir;

fn assert_impls<T: WalletKvBackend + ?Sized>() {}
fn assert_blob_impls<T: WalletBlobBackend + ?Sized>() {}

#[test]
fn test_acceptance_redb_backend_traits() {
    assert_impls::<RedbWalletKvBackend>();
    assert_blob_impls::<FileWalletBlobBackend>();
}

#[test]
fn test_redb_kv_commit_roundtrip() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("wallet.redb");
    let db = Arc::new(redb::Database::create(&db_path).unwrap());
    let backend = RedbWalletKvBackend::new(Arc::clone(&db));

    let mut write = rt
        .block_on(backend.begin_txn(WalletTxnMode::ReadWrite))
        .expect("begin write txn");

    rt.block_on(write.put(WalletKvTable::Meta, b"wallet.id", b"abc"))
        .expect("put");

    let v = rt
        .block_on(write.get(WalletKvTable::Meta, b"wallet.id"))
        .expect("get in-txn");
    assert_eq!(v.as_deref(), Some(b"abc".as_slice()));

    rt.block_on(write.commit()).expect("commit");

    let read = rt
        .block_on(backend.begin_txn(WalletTxnMode::ReadOnly))
        .expect("begin read txn");
    let v = rt
        .block_on(read.get(WalletKvTable::Meta, b"wallet.id"))
        .expect("get after commit");
    assert_eq!(v.as_deref(), Some(b"abc".as_slice()));

    rt.block_on(read.commit()).expect("commit read txn");
}

#[test]
fn test_redb_kv_rollback_discards() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("wallet.redb");
    let db = Arc::new(redb::Database::create(&db_path).unwrap());
    let backend = RedbWalletKvBackend::new(Arc::clone(&db));

    let mut write = rt
        .block_on(backend.begin_txn(WalletTxnMode::ReadWrite))
        .expect("begin write txn");

    rt.block_on(write.put(WalletKvTable::Meta, b"wallet.id", b"abc"))
        .expect("put");

    rt.block_on(write.rollback()).expect("rollback");

    let read = rt
        .block_on(backend.begin_txn(WalletTxnMode::ReadOnly))
        .expect("begin read txn");
    let v = rt
        .block_on(read.get(WalletKvTable::Meta, b"wallet.id"))
        .expect("get after rollback");
    assert!(v.is_none());

    rt.block_on(read.commit()).expect("commit read txn");
}
