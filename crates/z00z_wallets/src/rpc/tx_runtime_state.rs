use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use jsonrpsee::{core::RpcResult, types::ErrorObjectOwned};
use tokio::sync::RwLock;
use z00z_utils::{
    codec::{Codec, JsonCodec},
    config::{ConfigSource, EnvConfig},
    io::{hash_root_inputs, reset_managed_root, IoError},
};

use crate::{
    backup::{WalletTxHistoryEntryKind, WalletTxHistoryJsonlEntry},
    persistence::{TxConfirmationEvidence, TxRecord, TxStatus as StorageTxStatus, TxStorage},
    rpc::types::{
        common::PersistWalletId,
        tx::{
            PersistReceiptInfo, PersistTxId, PersistTxInfo, RuntimeTxDetailsResponse,
            RuntimeTxLifecycle, TxStatus,
        },
    },
    tx::TxPackage,
};

pub(crate) type SharedTxStore = Arc<RwLock<Box<dyn TxStorage + Send + Sync>>>;
pub(crate) type PendingTxStore = Arc<RwLock<Vec<PersistTxInfo>>>;
pub(crate) type PendingTxBytesStore = Arc<RwLock<Vec<PendingTxBytes>>>;
pub(crate) type ConfirmationEvidenceStore = Arc<RwLock<Vec<TxConfirmationEvidence>>>;

const TX_EXPORT_DIR_ENV: &str = "Z00Z_WALLET_TX_EXPORT_DIR";
const TX_EXPORT_KEEP_ENV: &str = "Z00Z_WALLET_TX_EXPORT_KEEP";
const TX_EXPORT_HASH_SCHEMA: &str = "wallet-tx-export-root-v1";

#[derive(Debug, Clone)]
pub(crate) struct PendingTxBytes {
    pub(crate) tx_id: PersistTxId,
    pub(crate) tx_bytes: Vec<u8>,
}

pub(crate) fn tx_export_dir_for_output(output_dir: &Path) -> PathBuf {
    let env = EnvConfig;
    if let Ok(Some(path)) = env.get(TX_EXPORT_DIR_ENV) {
        return PathBuf::from(path);
    }

    output_dir.join("tx_exports")
}

pub(crate) fn prepare_tx_export_root(output_dir: &Path) -> Result<PathBuf, IoError> {
    let env = EnvConfig;
    let path = tx_export_dir_for_output(output_dir);
    prepare_tx_export_path(path, matches!(env.get(TX_EXPORT_DIR_ENV), Ok(Some(_))))
}

fn prepare_tx_export_path(path: PathBuf, caller_owned: bool) -> Result<PathBuf, IoError> {
    if caller_owned {
        z00z_utils::io::create_dir_all(&path)?;
    } else {
        reset_managed_root(
            &path,
            &tx_export_fingerprint(),
            &[],
            Some(TX_EXPORT_KEEP_ENV),
        )?;
    }
    Ok(path)
}

pub(crate) fn overlay_pending_timestamp(
    pending_items: &[PersistTxInfo],
    wallet_id: &PersistWalletId,
    item: &mut PersistTxInfo,
) {
    if let Some(existing) = pending_items
        .iter()
        .find(|tx| tx.wallet_id == *wallet_id && tx.id == item.id)
    {
        item.timestamp = existing.timestamp;
    }
}

fn tx_export_fingerprint() -> String {
    static VALUE: OnceLock<String> = OnceLock::new();
    VALUE
        .get_or_init(|| {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
            hash_root_inputs(
                TX_EXPORT_HASH_SCHEMA,
                &[
                    root.join("Cargo.toml"),
                    root.join("Cargo.lock"),
                    root.join(".cargo/config.toml"),
                    root.join("crates/z00z_core/Cargo.toml"),
                    root.join("crates/z00z_crypto/Cargo.toml"),
                    root.join("crates/z00z_storage/Cargo.toml"),
                    root.join("crates/z00z_utils/Cargo.toml"),
                    root.join("crates/z00z_wallets/Cargo.toml"),
                ],
                &[
                    root.join("crates/z00z_core/src"),
                    root.join("crates/z00z_crypto/src"),
                    root.join("crates/z00z_storage/src"),
                    root.join("crates/z00z_utils/src"),
                    root.join("crates/z00z_wallets/src"),
                ],
            )
            .expect("hash tx export root")
        })
        .clone()
}

pub(crate) async fn load_wallet_tx_items(
    tx_store: Option<&SharedTxStore>,
    pending_txs: &PendingTxStore,
    wallet_id: &PersistWalletId,
) -> RpcResult<Vec<PersistTxInfo>> {
    if let Some(store) = tx_store {
        let store = store.read().await;
        let records = store.list().map_err(|error| {
            ErrorObjectOwned::owned(-32603, format!("TxStore error: {error}"), None::<()>)
        })?;
        let history_rows = store.list_history_rows().map_err(|error| {
            ErrorObjectOwned::owned(-32603, format!("TxStore error: {error}"), None::<()>)
        })?;
        let pending = pending_txs.read().await;

        return Ok(project_tx_infos(
            wallet_id,
            records,
            &history_rows,
            &pending,
        ));
    }

    let records = pending_txs.read().await;
    Ok(records
        .iter()
        .filter(|tx| tx.wallet_id == *wallet_id)
        .cloned()
        .collect())
}

pub(crate) async fn load_wallet_pending_tx_items(
    tx_store: Option<&SharedTxStore>,
    pending_txs: &PendingTxStore,
    wallet_id: &PersistWalletId,
) -> RpcResult<Vec<PersistTxInfo>> {
    if let Some(store) = tx_store {
        let store = store.read().await;
        let records = store
            .list_by_status(StorageTxStatus::Pending)
            .map_err(|error| {
                ErrorObjectOwned::owned(-32603, format!("TxStorage error: {error}"), None::<()>)
            })?;
        let history_rows = store.list_history_rows().map_err(|error| {
            ErrorObjectOwned::owned(-32603, format!("TxStorage error: {error}"), None::<()>)
        })?;
        let pending = pending_txs.read().await;

        return Ok(project_tx_infos(
            wallet_id,
            records,
            &history_rows,
            &pending,
        ));
    }

    let records = pending_txs.read().await;
    Ok(records
        .iter()
        .filter(|tx| tx.wallet_id == *wallet_id && matches!(tx.status, TxStatus::Pending))
        .cloned()
        .collect())
}

fn project_tx_infos(
    wallet_id: &PersistWalletId,
    records: Vec<TxRecord>,
    history_rows: &[WalletTxHistoryJsonlEntry],
    pending: &[PersistTxInfo],
) -> Vec<PersistTxInfo> {
    records
        .into_iter()
        .map(|record| {
            let latest_kind = latest_tx_history_kind(history_rows, &record.tx_hash);
            let mut info = tx_record_to_tx_info(wallet_id.clone(), record, latest_kind);
            overlay_pending_timestamp(pending, wallet_id, &mut info);
            info
        })
        .collect()
}

pub(crate) fn latest_tx_history_kind(
    rows: &[WalletTxHistoryJsonlEntry],
    tx_hash: &str,
) -> Option<WalletTxHistoryEntryKind> {
    rows.iter()
        .rev()
        .find(|row| row.tx_hash == tx_hash)
        .map(|row| row.entry_kind)
}

pub(crate) fn tx_lifecycle_from_record(
    record: &TxRecord,
    latest_kind: Option<WalletTxHistoryEntryKind>,
) -> RuntimeTxLifecycle {
    if matches!(latest_kind, Some(WalletTxHistoryEntryKind::Conflicted)) {
        return RuntimeTxLifecycle::Conflicted;
    }
    if matches!(latest_kind, Some(WalletTxHistoryEntryKind::AlreadySpent)) {
        return RuntimeTxLifecycle::AlreadySpent;
    }
    if record.status == StorageTxStatus::Confirmed
        || matches!(latest_kind, Some(WalletTxHistoryEntryKind::Confirmed))
    {
        return RuntimeTxLifecycle::Confirmed;
    }
    if record.status == StorageTxStatus::Failed
        || matches!(latest_kind, Some(WalletTxHistoryEntryKind::Failed))
    {
        return RuntimeTxLifecycle::Failed;
    }
    if record.status == StorageTxStatus::Cancelled
        || matches!(latest_kind, Some(WalletTxHistoryEntryKind::Cancelled))
    {
        return RuntimeTxLifecycle::Cancelled;
    }

    match latest_kind {
        Some(WalletTxHistoryEntryKind::Conflicted) => RuntimeTxLifecycle::Conflicted,
        Some(WalletTxHistoryEntryKind::AlreadySpent) => RuntimeTxLifecycle::AlreadySpent,
        Some(WalletTxHistoryEntryKind::Confirmed) => RuntimeTxLifecycle::Confirmed,
        Some(WalletTxHistoryEntryKind::Failed) => RuntimeTxLifecycle::Failed,
        Some(WalletTxHistoryEntryKind::Cancelled) => RuntimeTxLifecycle::Cancelled,
        Some(WalletTxHistoryEntryKind::Created) => RuntimeTxLifecycle::Created,
        Some(WalletTxHistoryEntryKind::Imported) => RuntimeTxLifecycle::Imported,
        Some(WalletTxHistoryEntryKind::Exported) => RuntimeTxLifecycle::Exported,
        Some(WalletTxHistoryEntryKind::Submitted) => RuntimeTxLifecycle::Submitted,
        Some(WalletTxHistoryEntryKind::Admitted) => RuntimeTxLifecycle::Admitted,
        Some(WalletTxHistoryEntryKind::Tombstoned) => {
            if record.imported {
                RuntimeTxLifecycle::Imported
            } else {
                RuntimeTxLifecycle::Created
            }
        }
        None if record.imported => RuntimeTxLifecycle::Imported,
        None => RuntimeTxLifecycle::Created,
    }
}

pub(crate) fn lifecycle_from_status(status: TxStatus) -> RuntimeTxLifecycle {
    match status {
        TxStatus::Pending => RuntimeTxLifecycle::Created,
        TxStatus::Confirmed => RuntimeTxLifecycle::Confirmed,
        TxStatus::Failed => RuntimeTxLifecycle::Failed,
        TxStatus::Cancelled => RuntimeTxLifecycle::Cancelled,
    }
}

pub(crate) fn tx_record_to_tx_info(
    wallet_id: PersistWalletId,
    record: TxRecord,
    latest_kind: Option<WalletTxHistoryEntryKind>,
) -> PersistTxInfo {
    let lifecycle = tx_lifecycle_from_record(&record, latest_kind);
    let package_summary = tx_package_summary(&record.tx_bytes);
    let (amount, fee, timestamp_ms) = package_summary
        .as_ref()
        .map(|summary| (summary.amount, summary.fee, record.timestamp_ms))
        .unwrap_or((0, 0, record.timestamp_ms));
    let receipt = if matches!(record.status, StorageTxStatus::Confirmed) {
        record
            .confirmation_evidence
            .as_ref()
            .map(receipt_from_evidence)
            .or_else(|| receipt_from_record_block(&record))
    } else {
        None
    };

    PersistTxInfo {
        id: PersistTxId::new(record.tx_hash.clone()),
        wallet_id,
        status: match record.status {
            StorageTxStatus::Pending => TxStatus::Pending,
            StorageTxStatus::Confirmed => TxStatus::Confirmed,
            StorageTxStatus::Failed => TxStatus::Failed,
            StorageTxStatus::Cancelled => TxStatus::Cancelled,
        },
        lifecycle,
        amount,
        fee,
        timestamp: timestamp_ms,
        receipt,
    }
}

fn receipt_from_evidence(evidence: &TxConfirmationEvidence) -> PersistReceiptInfo {
    PersistReceiptInfo {
        tx_id: PersistTxId::new(evidence.tx_id.clone()),
        block_height: evidence.block_height,
        block_hash: evidence.checkpoint_id_hex.clone(),
        tx_index: 0,
        confirmations: 1,
        confirmed_at: evidence.confirmed_at,
        verified: evidence.verified,
        merkle_proof: None,
    }
}

fn receipt_from_record_block(record: &TxRecord) -> Option<PersistReceiptInfo> {
    record.block_height.map(|block_height| PersistReceiptInfo {
        tx_id: PersistTxId::new(record.tx_hash.clone()),
        block_height,
        block_hash: tx_receipt_hash("block", &record.tx_hash),
        tx_index: 0,
        confirmations: 1,
        confirmed_at: record.timestamp_ms,
        verified: true,
        merkle_proof: None,
    })
}

#[derive(Debug, Clone)]
pub(crate) struct TxPackageSummary {
    pub(crate) amount: u64,
    pub(crate) fee: u64,
    pub(crate) inputs: Vec<z00z_core::assets::registry::AssetId>,
    pub(crate) outputs: Vec<z00z_core::assets::registry::AssetId>,
}

pub(crate) fn tx_package_summary(tx_bytes: &[u8]) -> Option<TxPackageSummary> {
    let package = JsonCodec.deserialize::<TxPackage>(tx_bytes).ok()?;
    let inputs = package
        .tx
        .inputs
        .iter()
        .filter_map(|input| decode_input_asset_id(&input.asset_id_hex))
        .collect::<Vec<_>>();
    let mut outputs = Vec::new();
    let mut amount = 0u64;

    for output in &package.tx.outputs {
        let Ok(asset) = output.asset_wire.clone().to_asset() else {
            continue;
        };
        if output.role == crate::tx::TxOutRole::Recipient {
            amount = amount.saturating_add(asset.amount);
        }
        outputs.push(asset.asset_id());
    }

    Some(TxPackageSummary {
        amount,
        fee: package.tx.fee,
        inputs,
        outputs,
    })
}

pub(crate) fn pending_input_asset_ids(
    records: impl IntoIterator<Item = TxRecord>,
) -> BTreeSet<z00z_core::assets::registry::AssetId> {
    let mut reserved = BTreeSet::new();

    for record in records {
        if !matches!(record.status, StorageTxStatus::Pending) {
            continue;
        }

        if let Some(summary) = tx_package_summary(&record.tx_bytes) {
            reserved.extend(summary.inputs);
        }
    }

    reserved
}

fn tx_receipt_hash(label: &str, tx_hash: &str) -> String {
    hex::encode(blake3::hash(format!("{label}:{tx_hash}").as_bytes()).as_bytes())
}

fn decode_package_asset_ids(
    tx_bytes: &[u8],
) -> (
    Vec<z00z_core::assets::registry::AssetId>,
    Vec<z00z_core::assets::registry::AssetId>,
) {
    tx_package_summary(tx_bytes)
        .map(|summary| (summary.inputs, summary.outputs))
        .unwrap_or_default()
}

fn decode_input_asset_id(value: &str) -> Option<z00z_core::assets::registry::AssetId> {
    let bytes = hex::decode(value).ok()?;
    let bytes: [u8; 32] = bytes.try_into().ok()?;
    if hex::encode(bytes) != value {
        return None;
    }
    Some(bytes)
}

pub(crate) fn tx_info_to_details(
    wallet_id: PersistWalletId,
    info: PersistTxInfo,
    tx_bytes: Option<&[u8]>,
) -> RuntimeTxDetailsResponse {
    let (inputs, outputs) = tx_bytes.map(decode_package_asset_ids).unwrap_or_default();

    RuntimeTxDetailsResponse {
        tx_id: info.id,
        wallet_id,
        status: info.status,
        lifecycle: info.lifecycle,
        amount: info.amount,
        fee: info.fee,
        inputs,
        outputs,
        timestamp: info.timestamp,
        confirmations: info
            .receipt
            .as_ref()
            .map(|receipt| receipt.confirmations)
            .unwrap_or(0),
        receipt_verified: info
            .receipt
            .as_ref()
            .is_some_and(|receipt| receipt.verified),
        receipt: info.receipt,
    }
}

pub(crate) async fn upsert_tx_bytes(
    pending_tx_bytes: &PendingTxBytesStore,
    tx_id: &PersistTxId,
    tx_bytes: Vec<u8>,
) {
    let mut pending = pending_tx_bytes.write().await;
    if let Some(existing) = pending.iter_mut().find(|item| &item.tx_id == tx_id) {
        existing.tx_bytes = tx_bytes;
        return;
    }

    pending.push(PendingTxBytes {
        tx_id: tx_id.clone(),
        tx_bytes,
    });
}

pub(crate) async fn upsert_tx_record(
    pending_txs: &PendingTxStore,
    wallet_id: &PersistWalletId,
    tx_id: &PersistTxId,
    status: TxStatus,
    amount: u64,
    fee: u64,
    timestamp_ms: u64,
) {
    let mut pending = pending_txs.write().await;
    if let Some(existing) = pending
        .iter_mut()
        .find(|tx| &tx.wallet_id == wallet_id && &tx.id == tx_id)
    {
        existing.status = status;
        existing.lifecycle = lifecycle_from_status(status);
        existing.amount = amount;
        existing.fee = fee;
        return;
    }

    pending.push(PersistTxInfo {
        id: tx_id.clone(),
        wallet_id: wallet_id.clone(),
        status,
        lifecycle: lifecycle_from_status(status),
        amount,
        fee,
        timestamp: timestamp_ms,
        receipt: None,
    });
}

pub(crate) async fn attach_tx_receipt(
    pending_txs: &PendingTxStore,
    tx_id: &PersistTxId,
    receipt: PersistReceiptInfo,
) {
    let mut pending = pending_txs.write().await;
    if let Some(existing) = pending.iter_mut().find(|tx| &tx.id == tx_id) {
        existing.receipt = Some(receipt);
    }
}

pub(crate) async fn upsert_confirmation_evidence(
    evidence_store: &ConfirmationEvidenceStore,
    evidence: TxConfirmationEvidence,
) {
    let mut items = evidence_store.write().await;
    if let Some(existing) = items.iter_mut().find(|item| item.tx_id == evidence.tx_id) {
        *existing = evidence;
        return;
    }

    items.push(evidence);
}

pub(crate) async fn find_confirmation_evidence(
    evidence_store: &ConfirmationEvidenceStore,
    tx_id: &PersistTxId,
) -> Option<TxConfirmationEvidence> {
    evidence_store
        .read()
        .await
        .iter()
        .find(|item| item.tx_id == tx_id.0)
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tx::{TxInputWire, TxOutRole, TxOutputWire, TxPackage, TxWire};
    use std::{ffi::OsString, path::PathBuf};
    use tempfile::TempDir;
    use z00z_core::assets::{AssetClass, AssetPkgWire};
    use z00z_utils::{
        codec::{Codec, JsonCodec},
        io::{create_dir_all, read_to_string, write_file},
    };

    const SRC: &str = include_str!("tx_runtime_state.rs");

    struct EnvGuard {
        key: &'static str,
        previous: Option<OsString>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(value) = &self.previous {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    #[test]
    fn test_dir_respects_env_override() {
        let guard = EnvGuard::set(TX_EXPORT_DIR_ENV, "/tmp/z00z-wallet-export-override");
        let path = tx_export_dir_for_output(Path::new("/ignored-output-dir"));
        assert_eq!(path, PathBuf::from("/tmp/z00z-wallet-export-override"));
        drop(guard);
    }

    #[test]
    fn test_prepare_export_root() {
        let dir = TempDir::new().expect("temp dir");
        let export_dir = dir.path().join("tx_exports");
        create_dir_all(&export_dir).expect("create export dir");
        write_file(export_dir.join(".managed-root-fingerprint"), b"stale-root")
            .expect("write stale mark");
        write_file(export_dir.join("old.json"), b"old").expect("write old export");

        let out = prepare_tx_export_path(export_dir.clone(), false).expect("prepare export root");

        assert_eq!(out, export_dir);
        assert!(!out.join("old.json").exists());
        assert_ne!(
            read_to_string(out.join(".managed-root-fingerprint")).expect("read mark"),
            "stale-root"
        );
    }

    #[test]
    fn test_fingerprint_on_first_use() {
        let dir = TempDir::new().expect("temp dir");
        let export_dir = dir.path().join("tx_exports");
        create_dir_all(&export_dir).expect("create export dir");
        write_file(
            export_dir.join(".managed-root-fingerprint"),
            tx_export_fingerprint().as_bytes(),
        )
        .expect("write live mark");
        write_file(export_dir.join("keep.json"), b"keep").expect("write keep file");

        let out = prepare_tx_export_path(export_dir.clone(), false).expect("prepare export root");

        assert_eq!(out, export_dir);
        assert!(
            !out.join("keep.json").exists(),
            "new process scope must clear stale matching exports"
        );
    }

    #[test]
    fn test_prepare_export_root_root() {
        let dir = TempDir::new().expect("temp dir");
        let export_dir = dir.path().join("tx_exports");

        prepare_tx_export_path(export_dir.clone(), false).expect("prepare first export root");
        write_file(export_dir.join("keep.json"), b"keep").expect("write keep file");

        let out = prepare_tx_export_path(export_dir.clone(), false).expect("prepare export root");

        assert_eq!(out, export_dir);
        assert!(
            !out.join("keep.json").exists(),
            "same-process rerun must clear prior tx export payload"
        );
    }

    #[test]
    fn test_prepare_export_override_files() {
        let dir = TempDir::new().expect("temp dir");
        let export_dir = dir.path().join("tx_exports");
        create_dir_all(&export_dir).expect("create export dir");
        write_file(export_dir.join("trace.jsonl"), b"keep").expect("write trace");

        let out = prepare_tx_export_path(export_dir.clone(), true).expect("prepare export root");

        assert_eq!(out, export_dir);
        assert_eq!(
            read_to_string(out.join("trace.jsonl")).expect("read trace"),
            "keep"
        );
        assert!(
            !out.join(".managed-root-fingerprint").exists(),
            "override root must stay caller-owned"
        );
    }

    #[test]
    fn test_tx_export_fingerprint_scope() {
        for needle in [
            "const TX_EXPORT_HASH_SCHEMA: &str = \"wallet-tx-export-root-v1\";",
            "\"Cargo.toml\"",
            "\"Cargo.lock\"",
            "\".cargo/config.toml\"",
            "\"crates/z00z_core/src\"",
            "\"crates/z00z_crypto/src\"",
            "\"crates/z00z_storage/src\"",
            "\"crates/z00z_utils/src\"",
            "\"crates/z00z_wallets/src\"",
        ] {
            assert!(
                SRC.contains(needle),
                "wallet tx export fingerprint contract must include {needle}"
            );
        }
    }

    #[test]
    fn tx_info_decodes_pkg_rows() {
        let input_id = [7u8; 32];
        let asset = z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 1, 10)
            .expect("valid asset");
        let output_wire = AssetPkgWire::from_asset(&asset);
        let package = TxPackage {
            kind: "TxPackage".to_string(),
            package_type: "regular_tx".to_string(),
            version: 1,
            chain_id: 7,
            chain_type: "devnet".to_string(),
            chain_name: "z00z".to_string(),
            tx: TxWire {
                tx_type: "regular_tx".to_string(),
                inputs: vec![TxInputWire {
                    asset_id_hex: hex::encode(input_id),
                    serial_id: 1,
                }],
                outputs: vec![TxOutputWire {
                    role: TxOutRole::Recipient,
                    asset_wire: output_wire,
                }],
                fee: 1,
                nonce: 1,
                context: Default::default(),
                proof: Default::default(),
                auth: Default::default(),
            },
            tx_digest_hex: "a".repeat(64),
            status: "prepared".to_string(),
        };
        let tx_bytes = JsonCodec.serialize(&package).expect("serialize package");

        let info = PersistTxInfo {
            id: PersistTxId::new("tx-1".to_string()),
            wallet_id: PersistWalletId("wallet-1".to_string()),
            status: TxStatus::Pending,
            lifecycle: RuntimeTxLifecycle::Admitted,
            amount: 10,
            fee: 1,
            timestamp: 123,
            receipt: None,
        };

        let details = tx_info_to_details(info.wallet_id.clone(), info, Some(&tx_bytes));

        assert_eq!(details.inputs, vec![input_id]);
        assert_eq!(details.outputs.len(), 1);
        assert_eq!(details.tx_id.0, "tx-1");
        assert_eq!(details.wallet_id.0, "wallet-1");
        assert_eq!(details.amount, 10);
        assert_eq!(details.fee, 1);
        assert_eq!(details.lifecycle, RuntimeTxLifecycle::Admitted);
    }

    #[test]
    fn test_failed_record_no_receipt() {
        let info = tx_record_to_tx_info(
            PersistWalletId("wallet-1".to_string()),
            TxRecord {
                tx_hash: "tx_failed".to_string(),
                tx_bytes: Vec::new(),
                imported: false,
                status: StorageTxStatus::Failed,
                timestamp_ms: 123,
                block_height: Some(99),
                confirmation_evidence: None,
            },
            Some(WalletTxHistoryEntryKind::Failed),
        );

        assert!(matches!(info.status, TxStatus::Failed));
        assert!(info.receipt.is_none());
        assert_eq!(info.lifecycle, RuntimeTxLifecycle::Failed);
    }

    #[test]
    fn test_prefers_latest_kind() {
        let record = TxRecord {
            tx_hash: "tx1".to_string(),
            tx_bytes: Vec::new(),
            imported: false,
            status: StorageTxStatus::Pending,
            timestamp_ms: 123,
            block_height: None,
            confirmation_evidence: None,
        };
        let cases = [
            (
                Some(WalletTxHistoryEntryKind::Created),
                RuntimeTxLifecycle::Created,
            ),
            (
                Some(WalletTxHistoryEntryKind::Imported),
                RuntimeTxLifecycle::Imported,
            ),
            (
                Some(WalletTxHistoryEntryKind::Exported),
                RuntimeTxLifecycle::Exported,
            ),
            (
                Some(WalletTxHistoryEntryKind::Submitted),
                RuntimeTxLifecycle::Submitted,
            ),
            (
                Some(WalletTxHistoryEntryKind::Admitted),
                RuntimeTxLifecycle::Admitted,
            ),
            (
                Some(WalletTxHistoryEntryKind::Conflicted),
                RuntimeTxLifecycle::Conflicted,
            ),
            (
                Some(WalletTxHistoryEntryKind::AlreadySpent),
                RuntimeTxLifecycle::AlreadySpent,
            ),
        ];

        for (latest_kind, expected) in cases {
            assert_eq!(tx_lifecycle_from_record(&record, latest_kind), expected);
        }
    }

    #[test]
    fn test_uses_terminal_fallbacks() {
        let confirmed = TxRecord {
            tx_hash: "tx-confirmed".to_string(),
            tx_bytes: Vec::new(),
            imported: false,
            status: StorageTxStatus::Confirmed,
            timestamp_ms: 1,
            block_height: Some(7),
            confirmation_evidence: None,
        };
        assert_eq!(
            tx_lifecycle_from_record(&confirmed, Some(WalletTxHistoryEntryKind::Created)),
            RuntimeTxLifecycle::Confirmed
        );

        let failed = TxRecord {
            tx_hash: "tx-failed".to_string(),
            tx_bytes: Vec::new(),
            imported: false,
            status: StorageTxStatus::Failed,
            timestamp_ms: 1,
            block_height: None,
            confirmation_evidence: None,
        };
        assert_eq!(
            tx_lifecycle_from_record(&failed, Some(WalletTxHistoryEntryKind::Submitted)),
            RuntimeTxLifecycle::Failed
        );

        let cancelled = TxRecord {
            tx_hash: "tx-cancelled".to_string(),
            tx_bytes: Vec::new(),
            imported: false,
            status: StorageTxStatus::Cancelled,
            timestamp_ms: 1,
            block_height: None,
            confirmation_evidence: None,
        };
        assert_eq!(
            tx_lifecycle_from_record(&cancelled, Some(WalletTxHistoryEntryKind::Admitted)),
            RuntimeTxLifecycle::Cancelled
        );
    }

    #[test]
    fn test_returns_last_history_row() {
        let created = WalletTxHistoryJsonlEntry::build_event(
            "abc",
            1,
            100,
            WalletTxHistoryEntryKind::Created,
            None,
            TxRecord {
                tx_hash: "tx-latest".to_string(),
                tx_bytes: Vec::new(),
                imported: false,
                status: StorageTxStatus::Pending,
                timestamp_ms: 100,
                block_height: None,
                confirmation_evidence: None,
            },
        )
        .expect("build created row");
        let admitted = WalletTxHistoryJsonlEntry::build_event(
            "abc",
            2,
            200,
            WalletTxHistoryEntryKind::Admitted,
            Some(created.entry_hash),
            TxRecord {
                tx_hash: "tx-latest".to_string(),
                tx_bytes: Vec::new(),
                imported: false,
                status: StorageTxStatus::Pending,
                timestamp_ms: 200,
                block_height: None,
                confirmation_evidence: None,
            },
        )
        .expect("build admitted row");

        assert_eq!(
            latest_tx_history_kind(&[created, admitted], "tx-latest"),
            Some(WalletTxHistoryEntryKind::Admitted)
        );
    }
}
