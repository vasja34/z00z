use std::sync::{Mutex, OnceLock};

use z00z_storage::backend::{JournalBackend, ReadTxn, StorageBackend, WriteTxn};
use z00z_storage::fixture_support::guardrail::{assert_absent, assert_present};
use z00z_storage::settlement::{
    check_live_startup_contract, RootGeneration, SettlementStore, HJMT_PROOF_ENVELOPE_VERSION,
};

const BACKEND_REDB: &str = include_str!("../src/backend/redb/mod.rs");
const STORE_MOD: &str = include_str!("../src/settlement/store.rs");
const README_DOC: &str = include_str!("../src/settlement/README.md");
const BACKEND_ENV: &str = "Z00Z_SETTLEMENT_BACKEND_MODE";
const REDB_ROOT_ENV: &str = "Z00Z_STORAGE_REDB_ROOT";
const BUCKET_BITS_ENV: &str = "Z00Z_SETTLEMENT_BUCKET_BITS";

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvReset {
    key: &'static str,
    previous: Option<String>,
}

impl EnvReset {
    fn capture(key: &'static str) -> Self {
        Self {
            key,
            previous: std::env::var(key).ok(),
        }
    }
}

impl Drop for EnvReset {
    fn drop(&mut self) {
        if let Some(value) = self.previous.as_deref() {
            std::env::set_var(self.key, value);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

#[test]
fn test_hjmt_backend_traits_compile() {
    struct TestRead;

    impl ReadTxn for TestRead {
        type Error = &'static str;

        fn get(&self, _table: &'static str, _key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
            Ok(None)
        }

        fn scan(&self, _table: &'static str) -> Result<Vec<(Vec<u8>, Vec<u8>)>, Self::Error> {
            Ok(Vec::new())
        }
    }

    struct TestWrite;

    impl ReadTxn for TestWrite {
        type Error = &'static str;

        fn get(&self, _table: &'static str, _key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
            Ok(None)
        }

        fn scan(&self, _table: &'static str) -> Result<Vec<(Vec<u8>, Vec<u8>)>, Self::Error> {
            Ok(Vec::new())
        }
    }

    impl WriteTxn for TestWrite {
        fn put(
            &mut self,
            _table: &'static str,
            _key: &[u8],
            _value: &[u8],
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn delete(&mut self, _table: &'static str, _key: &[u8]) -> Result<(), Self::Error> {
            Ok(())
        }

        fn commit(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    struct TestStore;

    impl StorageBackend for TestStore {
        type Error = &'static str;
        type Reader = TestRead;
        type Writer = TestWrite;

        fn read_txn(&self) -> Result<Self::Reader, Self::Error> {
            Ok(TestRead)
        }

        fn write_txn(&self) -> Result<Self::Writer, Self::Error> {
            Ok(TestWrite)
        }
    }

    struct TestJournal;

    impl JournalBackend for TestJournal {
        type Error = &'static str;

        fn recover_journal(&self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    let store = TestStore;
    let read = store.read_txn().expect("reader");
    let _ = read.get("meta", b"root").expect("read row");
    let _ = read.scan("meta").expect("scan rows");

    let mut write = store.write_txn().expect("writer");
    write.put("meta", b"root", b"value").expect("put row");
    write.delete("meta", b"root").expect("delete row");
    write.commit().expect("commit row");

    let journal = TestJournal;
    journal.recover_journal().expect("recover journal");
}

#[test]
fn test_startup_contract_rejects_drift() {
    check_live_startup_contract(
        "hjmt",
        1,
        RootGeneration::SettlementV1.version(),
        HJMT_PROOF_ENVELOPE_VERSION as u16,
    )
    .expect("live startup contract");

    let err = check_live_startup_contract(
        "wal",
        1,
        RootGeneration::SettlementV1.version(),
        HJMT_PROOF_ENVELOPE_VERSION as u16,
    )
    .expect_err("non-live backend");
    assert!(err
        .to_string()
        .contains("unsupported settlement backend mode"));
    assert!(!err.to_string().contains("wal"));

    let err = check_live_startup_contract(
        "hjmt",
        2,
        RootGeneration::SettlementV1.version(),
        HJMT_PROOF_ENVELOPE_VERSION as u16,
    )
    .expect_err("backend generation drift");
    assert!(err
        .to_string()
        .contains("unsupported settlement backend generation"));

    let err = check_live_startup_contract("hjmt", 1, 0, HJMT_PROOF_ENVELOPE_VERSION as u16)
        .expect_err("root generation drift");
    assert!(err
        .to_string()
        .contains("unsupported settlement root generation"));

    let err =
        check_live_startup_contract("hjmt", 1, RootGeneration::SettlementV1.version(), u16::MAX)
            .expect_err("proof version drift");
    assert!(err
        .to_string()
        .contains("unsupported settlement proof version"));
}

#[test]
fn test_backend_env_fails_closed() {
    let _env_lock = env_lock().lock().expect("env lock");
    let _backend_env = EnvReset::capture(BACKEND_ENV);
    let _redb_root_env = EnvReset::capture(REDB_ROOT_ENV);
    let _bucket_bits_env = EnvReset::capture(BUCKET_BITS_ENV);

    std::env::remove_var(REDB_ROOT_ENV);
    std::env::remove_var(BUCKET_BITS_ENV);

    std::env::remove_var(BACKEND_ENV);
    SettlementStore::try_new().expect("unset backend mode defaults to hjmt");

    std::env::set_var(BACKEND_ENV, "hjmt");
    SettlementStore::try_new().expect("explicit hjmt backend mode");

    let bad_mode = "forest-secret-mode";
    std::env::set_var(BACKEND_ENV, bad_mode);
    let err = match SettlementStore::try_new() {
        Ok(_) => panic!("non-hjmt backend mode must fail closed"),
        Err(err) => err,
    };
    let msg = err.to_string();
    assert!(msg.contains("unsupported settlement backend mode"));
    assert!(!msg.contains(bad_mode));
}

#[test]
fn test_redb_baseline_stays_single() {
    assert_present(
        "redb adapter",
        BACKEND_REDB,
        "impl JournalBackend for StoragePlane",
    );
    assert_present("store mod", STORE_MOD, "StoragePlane::new(root.into())");
    assert_present(
        "store mod",
        STORE_MOD,
        "crate::backend::JournalBackend::recover_journal(&backend)?;",
    );
    assert_present(
        "settlement readme",
        README_DOC,
        "A shared cross-aggregator WAL is not live protocol truth.",
    );
    assert_absent("redb adapter", BACKEND_REDB, "WalBackend");
    assert_absent("redb adapter", BACKEND_REDB, "ReplicatedLog");
}
