#![cfg(not(target_arch = "wasm32"))]

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use tokio::sync::RwLock;
use z00z_core::assets::AssetPkgWire;
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_utils::io::{read_file, write_file};
use z00z_utils::time::{MockTimeProvider, TimeProvider};
use z00z_wallets::{
    backup::{BackupImporterImpl, WalletTxHistoryEntryKind, WalletTxHistoryJsonlEntry},
    persistence::tx::{
        TxRecord, TxStatus as StorageTxStatus, TxStorage, TxStorageError, TxStorageImpl,
        TxStorageResult,
    },
    rpc::{
        methods::{
            AppRpcImpl, AppRpcServer, TxRpcImpl, TxRpcServer, WalletRpcImpl, WalletRpcServer,
        },
        types::{
            tx::{PersistTxId, RuntimePaginationParams},
            wallet::SessionToken,
        },
    },
    tx::TxPackage,
    WalletService,
};

fn history_kinds(path: &std::path::Path) -> Vec<WalletTxHistoryEntryKind> {
    let bytes = read_file(path).expect("read tx history");
    z00z_wallets::backup::decode_tx_history_rows(&bytes)
        .expect("decode tx history rows")
        .into_iter()
        .map(|entry| entry.entry_kind)
        .collect()
}

fn jsonl_bytes(wallet_stem: &str, records: &[TxRecord]) -> Vec<u8> {
    z00z_wallets::backup::encode_tx_history_jsonl(wallet_stem, records)
        .expect("encode tx-history JSONL")
}

fn jsonl_entry(wallet_stem: &str, record: TxRecord) -> WalletTxHistoryJsonlEntry {
    WalletTxHistoryJsonlEntry::build_event(
        wallet_stem,
        1,
        record.timestamp_ms,
        WalletTxHistoryEntryKind::Created,
        None,
        record,
    )
    .expect("build tx-history JSONL row")
}

fn sample_policy_tx_bytes(asset: &z00z_core::Asset) -> Vec<u8> {
    let package = z00z_wallets::tx::TxPackage {
        kind: "TxPackage".to_string(),
        package_type: "regular_tx".to_string(),
        version: 1,
        chain_id: 3,
        chain_type: "devnet".to_string(),
        chain_name: "devnet".to_string(),
        tx: z00z_wallets::tx::TxWire {
            tx_type: "regular_tx".to_string(),
            inputs: Vec::new(),
            outputs: vec![z00z_wallets::tx::TxOutputWire {
                role: z00z_wallets::tx::TxOutRole::Recipient,
                asset_wire: AssetPkgWire::from_asset(asset),
            }],
            fee: 0,
            nonce: 1,
            context: z00z_wallets::tx::TxContextWire::default(),
            proof: z00z_wallets::tx::TxProofWire::default(),
            auth: z00z_wallets::tx::TxAuthWire::default(),
        },
        tx_digest_hex: hex::encode([3u8; 32]),
        status: "pending".to_string(),
    };

    JsonCodec.serialize(&package).expect("serialize tx package")
}

#[derive(Default)]
struct TxStoreCallCounts {
    list: AtomicUsize,
    list_by_status: AtomicUsize,
    get: AtomicUsize,
}

struct MockTxStore {
    calls: Arc<TxStoreCallCounts>,
    records: Vec<TxRecord>,
}

impl MockTxStore {
    fn new(calls: Arc<TxStoreCallCounts>, records: Vec<TxRecord>) -> Self {
        Self { calls, records }
    }
}

impl TxStorage for MockTxStore {
    fn put(&mut self, _record: TxRecord) -> TxStorageResult<()> {
        Ok(())
    }

    fn record_imported(&mut self, record: TxRecord) -> TxStorageResult<()> {
        self.put(record)
    }

    fn record_exported(&mut self, _tx_hash: &str) -> TxStorageResult<()> {
        Ok(())
    }

    fn get(&self, tx_hash: &str) -> TxStorageResult<TxRecord> {
        self.calls.get.fetch_add(1, Ordering::SeqCst);
        self.records
            .iter()
            .find(|r| r.tx_hash == tx_hash)
            .cloned()
            .ok_or_else(|| TxStorageError::NotFound(tx_hash.to_string()))
    }

    fn list(&self) -> TxStorageResult<Vec<TxRecord>> {
        self.calls.list.fetch_add(1, Ordering::SeqCst);
        Ok(self.records.clone())
    }

    fn list_history_rows(&self) -> TxStorageResult<Vec<WalletTxHistoryJsonlEntry>> {
        let mut rows = Vec::with_capacity(self.records.len());

        for record in self.records.iter().cloned() {
            let previous_entry_hash = rows
                .last()
                .map(|row: &WalletTxHistoryJsonlEntry| row.entry_hash);
            let entry_kind = if record.imported {
                WalletTxHistoryEntryKind::Imported
            } else {
                match record.status {
                    StorageTxStatus::Pending => WalletTxHistoryEntryKind::Created,
                    StorageTxStatus::Confirmed => WalletTxHistoryEntryKind::Confirmed,
                    StorageTxStatus::Failed => WalletTxHistoryEntryKind::Failed,
                    StorageTxStatus::Cancelled => WalletTxHistoryEntryKind::Cancelled,
                }
            };
            rows.push(
                WalletTxHistoryJsonlEntry::build_event(
                    "mock",
                    rows.len() as u64 + 1,
                    record.timestamp_ms,
                    entry_kind,
                    previous_entry_hash,
                    record,
                )
                .map_err(TxStorageError::Serialization)?,
            );
        }

        Ok(rows)
    }

    fn list_by_status(&self, status: StorageTxStatus) -> TxStorageResult<Vec<TxRecord>> {
        self.calls.list_by_status.fetch_add(1, Ordering::SeqCst);
        Ok(self
            .records
            .iter()
            .filter(|r| r.status == status)
            .cloned()
            .collect())
    }

    fn update_status(&mut self, _tx_hash: &str, _status: StorageTxStatus) -> TxStorageResult<()> {
        Ok(())
    }

    fn record_submitted(&mut self, tx_hash: &str) -> TxStorageResult<()> {
        self.update_status(tx_hash, StorageTxStatus::Pending)
    }

    fn record_admitted(&mut self, tx_hash: &str) -> TxStorageResult<()> {
        self.update_status(tx_hash, StorageTxStatus::Pending)
    }

    fn record_confirmed(&mut self, tx_hash: &str, block_height: u64) -> TxStorageResult<()> {
        let _ = block_height;
        self.update_status(tx_hash, StorageTxStatus::Confirmed)
    }

    fn record_cancelled(&mut self, tx_hash: &str) -> TxStorageResult<()> {
        self.update_status(tx_hash, StorageTxStatus::Cancelled)
    }

    fn record_conflicted(&mut self, tx_hash: &str) -> TxStorageResult<()> {
        self.update_status(tx_hash, StorageTxStatus::Failed)?;
        self.get(tx_hash).map(|_| ())
    }

    fn record_already_spent(&mut self, tx_hash: &str) -> TxStorageResult<()> {
        self.update_status(tx_hash, StorageTxStatus::Failed)?;
        self.get(tx_hash).map(|_| ())
    }

    fn delete(&mut self, _tx_hash: &str) -> TxStorageResult<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_rpc_uses_injected_tx() {
    let calls = Arc::new(TxStoreCallCounts::default());
    let records = vec![
        TxRecord {
            tx_hash: "tx-1".to_string(),
            tx_bytes: Vec::new(),
            imported: false,
            status: StorageTxStatus::Pending,
            timestamp_ms: 1_700_000_010_000,
            block_height: None,
            confirmation_evidence: None,
        },
        TxRecord {
            tx_hash: "tx-2".to_string(),
            tx_bytes: Vec::new(),
            imported: false,
            status: StorageTxStatus::Confirmed,
            timestamp_ms: 1_700_000_020_000,
            block_height: Some(123),
            confirmation_evidence: None,
        },
    ];

    let tx_store: Arc<RwLock<Box<dyn TxStorage + Send + Sync>>> = Arc::new(RwLock::new(Box::new(
        MockTxStore::new(calls.clone(), records),
    )));

    let time: Arc<dyn TimeProvider> = Arc::new(MockTimeProvider::default());
    let temp = tempfile::tempdir().expect("tempdir");
    let output_dir = temp.path().join("wallets");
    let service = Arc::new(WalletService::with_output_dir_and_time(
        output_dir,
        time.clone(),
    ));

    let app_service = Arc::new(z00z_wallets::services::AppService::with_wallet_service(
        Arc::clone(&service),
    ));
    let app_rpc = AppRpcImpl::new(app_service);
    let wallet_rpc = WalletRpcImpl::new(Arc::clone(&service));

    let create = app_rpc
        .create_wallet("wallet-1".to_string(), "StrongPassw0rd!".to_string(), None)
        .await
        .expect("app.wallet.create_wallet should succeed");
    let session: SessionToken = wallet_rpc
        .unlock_wallet(create.wallet_id, "StrongPassw0rd!".to_string())
        .await
        .expect("wallet.session.unlock_wallet should succeed");

    let rpc = TxRpcImpl::with_dependencies_and_tx_store(service, time, tx_store);

    let history = rpc
        .get_transaction_history(
            session.clone(),
            RuntimePaginationParams {
                limit: Some(50),
                cursor: None,
                include_total: Some(true),
            },
            None,
            None,
        )
        .await
        .expect("tx.get_history should succeed");

    assert_eq!(history.items.len(), 2);
    assert!(history
        .items
        .iter()
        .any(|tx| tx.id == PersistTxId::new("tx-1".to_string())));
    assert!(history
        .items
        .iter()
        .any(|tx| tx.id == PersistTxId::new("tx-2".to_string())));
    assert_eq!(calls.list.load(Ordering::SeqCst), 1);

    let pending = rpc
        .list_pending_transactions(
            session.clone(),
            RuntimePaginationParams {
                limit: Some(50),
                cursor: None,
                include_total: Some(true),
            },
        )
        .await
        .expect("tx.list_pending should succeed");

    assert_eq!(pending.items.len(), 1);
    assert_eq!(pending.items[0].id, PersistTxId::new("tx-1".to_string()));

    // tx.list_pending should query pending records via TxStorage.
    assert_eq!(calls.list_by_status.load(Ordering::SeqCst), 1);
}

#[test]
fn jsonl_import_is_explicit() {
    let temp = tempfile::tempdir().unwrap();
    let source_path = temp.path().join("wallet_abc_tx_history_source.jsonl");
    let live_path = temp.path().join("wallet_abc_tx_history.jsonl");
    let noncanonical_history_dir = temp.path().join("wallet_abc_tx_history");
    let record = TxRecord {
        tx_hash: "tx-1".to_string(),
        tx_bytes: vec![1, 2, 3],
        imported: false,
        status: StorageTxStatus::Confirmed,
        timestamp_ms: 1_700_000_000,
        block_height: Some(7),
        confirmation_evidence: None,
    };
    let bytes = jsonl_bytes("abc", &[record.clone()]);
    write_file(&source_path, &bytes).unwrap();

    let imported =
        BackupImporterImpl::import_history_jsonl(source_path.to_string_lossy().as_ref()).unwrap();
    assert_eq!(imported, vec![record.clone()]);
    assert!(!noncanonical_history_dir.exists());

    write_file(&live_path, &bytes).unwrap();
    let store = TxStorageImpl::new(&live_path, MockTimeProvider::default());
    assert_eq!(store.list().unwrap(), vec![record]);
    assert!(live_path.exists());
    assert!(!noncanonical_history_dir.exists());
}

#[test]
fn test_keeps_created_day_restart() {
    let temp = tempfile::tempdir().unwrap();
    let history_path = temp.path().join("wallet_abc_tx_history.jsonl");
    let time = MockTimeProvider::from_unix_secs(86_399);
    let mut store = TxStorageImpl::new(&history_path, time.clone());
    let asset =
        z00z_core::genesis::asset_std::asset_from_dev_cfg("z00z", 9, 5).expect("valid std asset");

    let record = TxRecord {
        tx_hash: "tx-policy-1".to_string(),
        tx_bytes: sample_policy_tx_bytes(&asset),
        imported: false,
        status: StorageTxStatus::Pending,
        timestamp_ms: 1_700_000_000,
        block_height: None,
        confirmation_evidence: None,
    };
    store.put(record).unwrap();
    time.advance_by(std::time::Duration::from_secs(2));
    store.record_confirmed("tx-policy-1", 11).unwrap();

    let reopened = TxStorageImpl::new(&history_path, time);
    let day_one = reopened
        .policy_spend_window(asset.definition.id, 0, 86_400_000)
        .unwrap();
    let day_two = reopened
        .policy_spend_window(asset.definition.id, 86_400_000, 172_800_000)
        .unwrap();

    assert_eq!(day_one.spent_amount, 5);
    assert_eq!(day_one.pending_confirmation_count, 0);
    assert_eq!(day_two.spent_amount, 0);
}

#[test]
fn jsonl_replay_preserves_record() {
    let temp = tempfile::tempdir().unwrap();
    let source_path = temp.path().join("wallet_abc_tx_history_source.jsonl");
    let live_path = temp.path().join("wallet_abc_tx_history.jsonl");
    let noncanonical_history_dir = temp.path().join("wallet_abc_tx_history");
    let record = TxRecord {
        tx_hash: "tx-full-view".to_string(),
        tx_bytes: vec![9, 8, 7, 6],
        imported: false,
        status: StorageTxStatus::Confirmed,
        timestamp_ms: 1_700_000_123,
        block_height: Some(99),
        confirmation_evidence: None,
    };
    let bytes = jsonl_bytes("abc", &[record.clone()]);
    write_file(&source_path, &bytes).unwrap();

    let imported =
        BackupImporterImpl::import_history_jsonl(source_path.to_string_lossy().as_ref()).unwrap();
    assert_eq!(imported, vec![record.clone()]);
    write_file(&live_path, &bytes).unwrap();
    let store = TxStorageImpl::new(&live_path, MockTimeProvider::default());

    let stored = store.get("tx-full-view").unwrap();
    assert_eq!(stored.tx_hash, record.tx_hash);
    assert_eq!(stored.tx_bytes, record.tx_bytes);
    assert_eq!(stored.status, record.status);
    assert_eq!(stored.timestamp_ms, record.timestamp_ms);
    assert_eq!(stored.block_height, record.block_height);
    assert!(live_path.exists());
    assert!(!noncanonical_history_dir.exists());
}

#[test]
fn jsonl_replay_keeps_tx_pkg() {
    let temp = tempfile::tempdir().unwrap();
    let source_path = temp.path().join("wallet_abc_tx_history_source.jsonl");
    let live_path = temp.path().join("wallet_abc_tx_history.jsonl");
    let noncanonical_history_dir = temp.path().join("wallet_abc_tx_history");
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("tx_package.json");
    let fixture_bytes = read_file(&fixture_path).expect("read tx package fixture");
    let pkg: TxPackage = JsonCodec
        .deserialize(&fixture_bytes)
        .expect("decode tx package fixture");
    let tx_bytes = JsonCodec.serialize(&pkg).unwrap();
    let record = TxRecord {
        tx_hash: "tx-package-full".to_string(),
        tx_bytes: tx_bytes.clone(),
        imported: false,
        status: StorageTxStatus::Confirmed,
        timestamp_ms: 1_700_000_222,
        block_height: Some(17),
        confirmation_evidence: None,
    };
    let bytes = jsonl_bytes("abc", &[record.clone()]);
    write_file(&source_path, &bytes).unwrap();

    let imported =
        BackupImporterImpl::import_history_jsonl(source_path.to_string_lossy().as_ref()).unwrap();
    assert_eq!(imported, vec![record.clone()]);

    write_file(&live_path, &bytes).unwrap();
    let store = TxStorageImpl::new(&live_path, MockTimeProvider::default());
    let stored = store.get("tx-package-full").unwrap();

    assert_eq!(stored.tx_bytes, tx_bytes);
    assert_eq!(
        JsonCodec
            .deserialize::<TxPackage>(&stored.tx_bytes)
            .expect("decode stored tx package"),
        pkg
    );
    assert!(live_path.exists());
    assert!(!noncanonical_history_dir.exists());
}

#[test]
fn jsonl_replay_rejects_tamper() {
    let temp = tempfile::tempdir().unwrap();
    let source_path = temp.path().join("wallet_abc_tx_history_source.jsonl");
    let live_path = temp.path().join("wallet_abc_tx_history.jsonl");
    let noncanonical_history_dir = temp.path().join("wallet_abc_tx_history");
    let existing = TxRecord {
        tx_hash: "tx-existing".to_string(),
        tx_bytes: vec![1, 1, 1],
        imported: false,
        status: StorageTxStatus::Pending,
        timestamp_ms: 1_700_000_001,
        block_height: None,
        confirmation_evidence: None,
    };
    let tampered = TxRecord {
        tx_hash: "tx-tampered".to_string(),
        tx_bytes: vec![2, 2, 2],
        imported: false,
        status: StorageTxStatus::Confirmed,
        timestamp_ms: 1_700_000_002,
        block_height: Some(2),
        confirmation_evidence: None,
    };
    let mut entry = jsonl_entry("abc", tampered);
    entry.record_hash[0] ^= 0x01;
    let mut bytes = JsonCodec.serialize(&entry).unwrap();
    bytes.push(b'\n');
    write_file(&source_path, &bytes).unwrap();

    let mut store = TxStorageImpl::new(&live_path, MockTimeProvider::default());
    store.put(existing.clone()).unwrap();
    let err = BackupImporterImpl::import_history_jsonl(source_path.to_string_lossy().as_ref())
        .unwrap_err();

    assert!(err.to_string().contains("record hash mismatch"));
    assert_eq!(store.list().unwrap(), vec![existing]);
    assert!(store.get("tx-tampered").is_err());
    assert!(live_path.exists());
    assert!(!noncanonical_history_dir.exists());
}

#[test]
fn artifact_paths_stay_distinct() {
    let temp = tempfile::tempdir().unwrap();
    let live_path = temp.path().join("wallet_abc_tx_history.jsonl");
    let noncanonical_history_dir = temp.path().join("wallet_abc_tx_history");
    let wlt_path = temp.path().join("wallet_abc.wlt");
    let archive_path = temp.path().join("wallet_abc.backup");
    let rpc_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("outputs")
        .join("tx_exports");

    assert_ne!(live_path, noncanonical_history_dir);
    assert_ne!(noncanonical_history_dir, wlt_path);
    assert_ne!(noncanonical_history_dir, archive_path);
    assert_ne!(noncanonical_history_dir, rpc_dir);
    assert_ne!(live_path, wlt_path);
    assert_ne!(live_path, archive_path);
    assert_ne!(live_path, rpc_dir);
    assert_ne!(wlt_path, archive_path);
    assert_ne!(wlt_path, rpc_dir);
    assert_ne!(archive_path, rpc_dir);
    assert!(!noncanonical_history_dir.starts_with(&rpc_dir));
    assert!(!live_path.starts_with(&rpc_dir));
    assert!(!archive_path.starts_with(&rpc_dir));

    let record = TxRecord {
        tx_hash: "tx-boundary".to_string(),
        tx_bytes: vec![7, 7, 7],
        imported: false,
        status: StorageTxStatus::Confirmed,
        timestamp_ms: 1_700_000_001,
        block_height: Some(8),
        confirmation_evidence: None,
    };
    let mut store = TxStorageImpl::new(&live_path, MockTimeProvider::default());
    store.put(record.clone()).unwrap();

    assert!(live_path.exists());
    assert!(!noncanonical_history_dir.exists());
    assert_eq!(store.list().unwrap(), vec![record]);
}

#[test]
fn tx_history_appends_admission_sequence() {
    let temp = tempfile::tempdir().unwrap();
    let live_path = temp.path().join("wallet_abc_tx_history.jsonl");
    let record = TxRecord {
        tx_hash: "tx-admission".to_string(),
        tx_bytes: vec![1, 2, 3, 4],
        imported: false,
        status: StorageTxStatus::Pending,
        timestamp_ms: 1_700_000_001,
        block_height: None,
        confirmation_evidence: None,
    };
    let mut store = TxStorageImpl::new(&live_path, MockTimeProvider::default());

    store.put(record.clone()).unwrap();
    store.record_submitted(&record.tx_hash).unwrap();
    store.record_admitted(&record.tx_hash).unwrap();
    store.record_exported(&record.tx_hash).unwrap();
    store.record_confirmed(&record.tx_hash, 42).unwrap();

    let stored = store.get(&record.tx_hash).unwrap();
    assert_eq!(stored.status, StorageTxStatus::Confirmed);
    assert_eq!(stored.block_height, Some(42));
    assert_eq!(
        history_kinds(&live_path),
        vec![
            WalletTxHistoryEntryKind::Created,
            WalletTxHistoryEntryKind::Submitted,
            WalletTxHistoryEntryKind::Admitted,
            WalletTxHistoryEntryKind::Exported,
            WalletTxHistoryEntryKind::Confirmed,
        ]
    );
}
