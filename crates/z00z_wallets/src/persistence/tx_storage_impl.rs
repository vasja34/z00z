//! File-backed transaction history store.

use super::{
    TxConfirmationEvidence, TxPolicySpendWindow, TxRecord, TxStatus, TxStorage, TxStorageError,
    TxStorageResult,
};
use crate::backup::{
    decode_tx_history_rows, encode_tx_history_rows, fold_tx_history_rows, WalletTxHistoryEntryKind,
    WalletTxHistoryJsonlEntry,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
};
use z00z_core::assets::registry::AssetId;
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{atomic_write_file_private, create_dir_all, path_exists, read_file, write_file, IoError},
    time::TimeProvider,
};

/// Maximum tx-history JSONL file size accepted by the live store.
pub(crate) const MAX_TX_HISTORY_JSONL_BYTES: u64 = 10 * 1024 * 1024;
const WALLET_HISTORY_PREFIX: &str = "wallet_";
const WALLET_HISTORY_SUFFIX: &str = "_tx_history.jsonl";

type TxFileLockMap = Mutex<HashMap<PathBuf, Arc<Mutex<()>>>>;

pub static TX_FILE_LOCKS: OnceLock<TxFileLockMap> = OnceLock::new();

pub(crate) fn tx_history_path_lock(path: &Path) -> TxStorageResult<Arc<Mutex<()>>> {
    let locks = TX_FILE_LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut map = locks
        .lock()
        .map_err(|_| TxStorageError::Database("tx-history lock map poisoned".to_string()))?;
    Ok(map
        .entry(path.to_path_buf())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone())
}

/// File-backed transaction storage implementation.
///
/// Stores the canonical live tx history in a single JSONL file for the wallet stem.
/// The file is the live store; `.wlt` remains wallet state only.
#[derive(Debug, Clone)]
pub struct TxStorageImpl<T: TimeProvider> {
    history_path: PathBuf,
    time_provider: T,
}

impl<T: TimeProvider> TxStorageImpl<T> {
    /// Create a new transaction storage rooted at the canonical JSONL file.
    pub fn new(history_path: impl Into<PathBuf>, time_provider: T) -> Self {
        Self {
            history_path: history_path.into(),
            time_provider,
        }
    }

    fn validate_tx_hash(tx_hash: &str) -> TxStorageResult<()> {
        if tx_hash.is_empty()
            || tx_hash.contains('/')
            || tx_hash.contains('\\')
            || tx_hash.contains("..")
        {
            return Err(TxStorageError::InvalidTxHash(tx_hash.to_string()));
        }
        Ok(())
    }

    fn validate_wallet_stem(wallet_stem: &str) -> TxStorageResult<()> {
        if wallet_stem.is_empty()
            || wallet_stem.contains('/')
            || wallet_stem.contains('\\')
            || wallet_stem.contains("..")
        {
            return Err(TxStorageError::Database(format!(
                "invalid wallet stem: {wallet_stem}"
            )));
        }
        Ok(())
    }

    fn wallet_stem(&self) -> TxStorageResult<String> {
        let file_name = self
            .history_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                TxStorageError::Database(format!(
                    "canonical tx-history path has no file name: {}",
                    self.history_path.to_string_lossy()
                ))
            })?;

        if !file_name.starts_with(WALLET_HISTORY_PREFIX)
            || !file_name.ends_with(WALLET_HISTORY_SUFFIX)
        {
            return Err(TxStorageError::Database(format!(
                "canonical tx-history path must be wallet_<stem>_tx_history.jsonl: {}",
                self.history_path.to_string_lossy()
            )));
        }

        let stem_start = WALLET_HISTORY_PREFIX.len();
        let stem_end = file_name.len() - WALLET_HISTORY_SUFFIX.len();
        let wallet_stem = file_name[stem_start..stem_end].to_string();
        Self::validate_wallet_stem(&wallet_stem)?;
        Ok(wallet_stem)
    }

    fn is_not_found(err: &IoError) -> bool {
        matches!(err, IoError::Io(e) if e.kind() == std::io::ErrorKind::NotFound)
    }

    fn ensure_parent_dir(&self) -> TxStorageResult<()> {
        if let Some(parent) = self.history_path.parent() {
            create_dir_all(parent)?;
        }
        Ok(())
    }

    fn ensure_history_file_unlocked(&self) -> TxStorageResult<()> {
        self.ensure_parent_dir()?;
        if !path_exists(&self.history_path)? {
            write_file(&self.history_path, &[])?;
        }
        Ok(())
    }

    fn load_rows_unlocked(&self) -> TxStorageResult<Vec<WalletTxHistoryJsonlEntry>> {
        let bytes = match read_file(&self.history_path) {
            Ok(bytes) => bytes,
            Err(err) if Self::is_not_found(&err) => {
                return Err(TxStorageError::NotFound(
                    self.history_path.to_string_lossy().to_string(),
                ));
            }
            Err(err) => return Err(TxStorageError::Io(err)),
        };

        if bytes.len() as u64 > MAX_TX_HISTORY_JSONL_BYTES {
            return Err(TxStorageError::Serialization(format!(
                "tx-history JSONL file too large: {} bytes",
                bytes.len()
            )));
        }

        decode_tx_history_rows(&bytes).map_err(TxStorageError::Serialization)
    }

    fn load_or_init_rows_unlocked(&self) -> TxStorageResult<Vec<WalletTxHistoryJsonlEntry>> {
        self.ensure_history_file_unlocked()?;
        self.load_rows_unlocked()
    }

    fn persist_rows_unlocked(&self, rows: &[WalletTxHistoryJsonlEntry]) -> TxStorageResult<()> {
        self.ensure_parent_dir()?;
        let bytes = encode_tx_history_rows(rows).map_err(TxStorageError::Serialization)?;
        if bytes.len() as u64 > MAX_TX_HISTORY_JSONL_BYTES {
            return Err(TxStorageError::Serialization(format!(
                "tx-history JSONL file too large: {} bytes",
                bytes.len()
            )));
        }
        atomic_write_file_private(&self.history_path, &bytes)?;
        Ok(())
    }

    fn path_lock(&self) -> TxStorageResult<Arc<Mutex<()>>> {
        tx_history_path_lock(&self.history_path)
    }

    fn with_write_lock<R>(&self, op: impl FnOnce() -> TxStorageResult<R>) -> TxStorageResult<R> {
        let lock = self.path_lock()?;
        let _guard = lock
            .lock()
            .map_err(|_| TxStorageError::Database("tx-history file lock poisoned".to_string()))?;
        op()
    }

    fn append_row_unlocked(
        &self,
        rows: &mut Vec<WalletTxHistoryJsonlEntry>,
        entry_kind: WalletTxHistoryEntryKind,
        mut record: TxRecord,
    ) -> TxStorageResult<()> {
        Self::validate_tx_hash(&record.tx_hash)?;
        let wallet_stem = self.wallet_stem()?;
        let sequence = rows.len() as u64 + 1;
        let recorded_at_ms = self.time_provider.compat_unix_timestamp_millis();
        let previous_entry_hash = rows.last().map(|row| row.entry_hash);

        if matches!(
            entry_kind,
            WalletTxHistoryEntryKind::Confirmed
                | WalletTxHistoryEntryKind::Failed
                | WalletTxHistoryEntryKind::Submitted
                | WalletTxHistoryEntryKind::Admitted
                | WalletTxHistoryEntryKind::Cancelled
                | WalletTxHistoryEntryKind::Conflicted
                | WalletTxHistoryEntryKind::AlreadySpent
                | WalletTxHistoryEntryKind::Tombstoned
        ) {
            record.timestamp_ms = recorded_at_ms;
        }

        let row = WalletTxHistoryJsonlEntry::build_event(
            &wallet_stem,
            sequence,
            recorded_at_ms,
            entry_kind,
            previous_entry_hash,
            record,
        )
        .map_err(TxStorageError::Serialization)?;
        rows.push(row);
        Ok(())
    }

    fn status_entry_kind(status: TxStatus) -> WalletTxHistoryEntryKind {
        match status {
            TxStatus::Pending => WalletTxHistoryEntryKind::Submitted,
            TxStatus::Confirmed => WalletTxHistoryEntryKind::Confirmed,
            TxStatus::Failed => WalletTxHistoryEntryKind::Failed,
            TxStatus::Cancelled => WalletTxHistoryEntryKind::Cancelled,
        }
    }

    fn current_records(rows: &[WalletTxHistoryJsonlEntry]) -> Vec<TxRecord> {
        fold_tx_history_rows(rows)
    }

    fn tx_amounts_for_policy(
        record: &TxRecord,
        definition_id: &AssetId,
    ) -> TxStorageResult<(u64, u64)> {
        if record.imported {
            return Ok((0, 0));
        }

        let package = JsonCodec
            .deserialize::<crate::tx::TxPackage>(&record.tx_bytes)
            .map_err(|error| {
                TxStorageError::Serialization(format!(
                    "policy spend window decode failed for {}: {error}",
                    record.tx_hash
                ))
            })?;

        let mut total_recipient_amount = 0u64;
        let mut matching_asset_amount = 0u64;

        for output in &package.tx.outputs {
            if output.role != crate::tx::TxOutRole::Recipient {
                continue;
            }

            let asset = output.asset_wire.clone().to_asset().map_err(|error| {
                TxStorageError::Serialization(format!(
                    "policy spend window output decode failed for {}: {error}",
                    record.tx_hash
                ))
            })?;

            total_recipient_amount = total_recipient_amount.saturating_add(asset.amount);
            if &asset.definition.id == definition_id {
                matching_asset_amount = matching_asset_amount.saturating_add(asset.amount);
            }
        }

        Ok((total_recipient_amount, matching_asset_amount))
    }

    /// Derive the live daily spend window and pending-confirmation count for a
    /// concrete asset definition id from the canonical tx-history JSONL store.
    pub fn policy_spend_window(
        &self,
        definition_id: AssetId,
        day_start_ms: u64,
        day_end_ms: u64,
    ) -> TxStorageResult<TxPolicySpendWindow> {
        self.with_write_lock(|| {
            let rows = self.load_or_init_rows_unlocked()?;
            Self::policy_spend_window_from_rows(&rows, definition_id, day_start_ms, day_end_ms)
        })
    }

    fn policy_spend_window_from_rows(
        rows: &[WalletTxHistoryJsonlEntry],
        definition_id: AssetId,
        day_start_ms: u64,
        day_end_ms: u64,
    ) -> TxStorageResult<TxPolicySpendWindow> {
        #[derive(Debug, Clone, Copy)]
        struct TrackedSpend {
            created_at_ms: u64,
            status: TxStatus,
            total_recipient_amount: u64,
            matching_asset_amount: u64,
        }

        let mut tracked: HashMap<&str, TrackedSpend> = HashMap::new();

        for row in rows {
            let tracked_spend = tracked.entry(&row.tx_hash).or_insert(TrackedSpend {
                created_at_ms: row.recorded_at_ms,
                status: row.record.status,
                total_recipient_amount: 0,
                matching_asset_amount: 0,
            });
            let (total_recipient_amount, matching_asset_amount) =
                Self::tx_amounts_for_policy(&row.record, &definition_id)?;
            tracked_spend.status = row.record.status;
            tracked_spend.total_recipient_amount = total_recipient_amount;
            tracked_spend.matching_asset_amount = matching_asset_amount;
        }

        let mut window = TxPolicySpendWindow::default();

        for spend in tracked.values() {
            if spend.total_recipient_amount == 0 {
                continue;
            }

            if spend.status == TxStatus::Pending {
                window.pending_confirmation_count =
                    window.pending_confirmation_count.saturating_add(1);
            }

            if spend.created_at_ms < day_start_ms || spend.created_at_ms >= day_end_ms {
                continue;
            }

            if matches!(spend.status, TxStatus::Pending | TxStatus::Confirmed) {
                window.spent_amount = window
                    .spent_amount
                    .saturating_add(spend.matching_asset_amount);
            }
        }

        Ok(window)
    }
}

impl<T: TimeProvider> TxStorage for TxStorageImpl<T> {
    fn put(&mut self, record: TxRecord) -> TxStorageResult<()> {
        Self::validate_tx_hash(&record.tx_hash)?;

        self.with_write_lock(|| {
            let mut rows = self.load_or_init_rows_unlocked()?;
            self.append_row_unlocked(&mut rows, WalletTxHistoryEntryKind::Created, record)?;
            self.persist_rows_unlocked(&rows)
        })
    }

    fn record_imported(&mut self, record: TxRecord) -> TxStorageResult<()> {
        Self::validate_tx_hash(&record.tx_hash)?;

        self.with_write_lock(|| {
            let mut rows = self.load_or_init_rows_unlocked()?;
            self.append_row_unlocked(&mut rows, WalletTxHistoryEntryKind::Imported, record)?;
            self.persist_rows_unlocked(&rows)
        })
    }

    fn record_exported(&mut self, tx_hash: &str) -> TxStorageResult<()> {
        Self::validate_tx_hash(tx_hash)?;
        self.with_write_lock(|| {
            let mut rows = self.load_or_init_rows_unlocked()?;
            let record = Self::current_records(&rows)
                .into_iter()
                .find(|item| item.tx_hash == tx_hash)
                .ok_or_else(|| TxStorageError::NotFound(tx_hash.to_string()))?;

            self.append_row_unlocked(&mut rows, WalletTxHistoryEntryKind::Exported, record)?;
            self.persist_rows_unlocked(&rows)
        })
    }

    fn get(&self, tx_hash: &str) -> TxStorageResult<TxRecord> {
        Self::validate_tx_hash(tx_hash)?;
        let records = self.list()?;

        records
            .into_iter()
            .find(|record| record.tx_hash == tx_hash)
            .ok_or_else(|| TxStorageError::NotFound(tx_hash.to_string()))
    }

    fn list(&self) -> TxStorageResult<Vec<TxRecord>> {
        self.with_write_lock(|| {
            let rows = self.load_or_init_rows_unlocked()?;
            Ok(Self::current_records(&rows))
        })
    }

    fn list_history_rows(&self) -> TxStorageResult<Vec<WalletTxHistoryJsonlEntry>> {
        self.with_write_lock(|| self.load_or_init_rows_unlocked())
    }

    fn list_by_status(&self, status: TxStatus) -> TxStorageResult<Vec<TxRecord>> {
        Ok(self
            .list()?
            .into_iter()
            .filter(|record| record.status == status)
            .collect())
    }

    fn update_status(&mut self, tx_hash: &str, status: TxStatus) -> TxStorageResult<()> {
        Self::validate_tx_hash(tx_hash)?;
        self.with_write_lock(|| {
            let mut rows = self.load_or_init_rows_unlocked()?;
            let mut record = Self::current_records(&rows)
                .into_iter()
                .find(|item| item.tx_hash == tx_hash)
                .ok_or_else(|| TxStorageError::NotFound(tx_hash.to_string()))?;

            record.status = status;
            if !matches!(status, TxStatus::Confirmed) {
                record.block_height = None;
                record.confirmation_evidence = None;
            }
            self.append_row_unlocked(&mut rows, Self::status_entry_kind(status), record)?;
            self.persist_rows_unlocked(&rows)
        })
    }

    fn record_submitted(&mut self, tx_hash: &str) -> TxStorageResult<()> {
        Self::validate_tx_hash(tx_hash)?;
        self.with_write_lock(|| {
            let mut rows = self.load_or_init_rows_unlocked()?;
            let mut record = Self::current_records(&rows)
                .into_iter()
                .find(|item| item.tx_hash == tx_hash)
                .ok_or_else(|| TxStorageError::NotFound(tx_hash.to_string()))?;

            record.status = TxStatus::Pending;
            record.block_height = None;
            record.confirmation_evidence = None;
            self.append_row_unlocked(&mut rows, WalletTxHistoryEntryKind::Submitted, record)?;
            self.persist_rows_unlocked(&rows)
        })
    }

    fn record_admitted(&mut self, tx_hash: &str) -> TxStorageResult<()> {
        Self::validate_tx_hash(tx_hash)?;
        self.with_write_lock(|| {
            let mut rows = self.load_or_init_rows_unlocked()?;
            let mut record = Self::current_records(&rows)
                .into_iter()
                .find(|item| item.tx_hash == tx_hash)
                .ok_or_else(|| TxStorageError::NotFound(tx_hash.to_string()))?;

            record.status = TxStatus::Pending;
            record.block_height = None;
            record.confirmation_evidence = None;
            self.append_row_unlocked(&mut rows, WalletTxHistoryEntryKind::Admitted, record)?;
            self.persist_rows_unlocked(&rows)
        })
    }

    fn record_confirmed(&mut self, tx_hash: &str, block_height: u64) -> TxStorageResult<()> {
        Self::validate_tx_hash(tx_hash)?;
        self.with_write_lock(|| {
            let mut rows = self.load_or_init_rows_unlocked()?;
            let mut record = Self::current_records(&rows)
                .into_iter()
                .find(|item| item.tx_hash == tx_hash)
                .ok_or_else(|| TxStorageError::NotFound(tx_hash.to_string()))?;

            record.status = TxStatus::Confirmed;
            record.block_height = Some(block_height);
            record.confirmation_evidence = None;
            self.append_row_unlocked(&mut rows, WalletTxHistoryEntryKind::Confirmed, record)?;
            self.persist_rows_unlocked(&rows)
        })
    }

    fn record_confirmation_evidence(
        &mut self,
        tx_hash: &str,
        evidence: TxConfirmationEvidence,
    ) -> TxStorageResult<()> {
        Self::validate_tx_hash(tx_hash)?;
        if evidence.tx_id != tx_hash {
            return Err(TxStorageError::Database(format!(
                "confirmation evidence tx id mismatch: expected {tx_hash}, got {}",
                evidence.tx_id
            )));
        }

        self.with_write_lock(|| {
            let mut rows = self.load_or_init_rows_unlocked()?;
            let mut record = Self::current_records(&rows)
                .into_iter()
                .find(|item| item.tx_hash == tx_hash)
                .ok_or_else(|| TxStorageError::NotFound(tx_hash.to_string()))?;

            record.status = TxStatus::Confirmed;
            record.block_height = Some(evidence.block_height);
            record.confirmation_evidence = Some(evidence);
            self.append_row_unlocked(&mut rows, WalletTxHistoryEntryKind::Confirmed, record)?;
            self.persist_rows_unlocked(&rows)
        })
    }

    fn record_cancelled(&mut self, tx_hash: &str) -> TxStorageResult<()> {
        Self::validate_tx_hash(tx_hash)?;
        self.with_write_lock(|| {
            let mut rows = self.load_or_init_rows_unlocked()?;
            let mut record = Self::current_records(&rows)
                .into_iter()
                .find(|item| item.tx_hash == tx_hash)
                .ok_or_else(|| TxStorageError::NotFound(tx_hash.to_string()))?;

            record.status = TxStatus::Cancelled;
            record.block_height = None;
            record.confirmation_evidence = None;
            self.append_row_unlocked(&mut rows, WalletTxHistoryEntryKind::Cancelled, record)?;
            self.persist_rows_unlocked(&rows)
        })
    }

    fn record_conflicted(&mut self, tx_hash: &str) -> TxStorageResult<()> {
        Self::validate_tx_hash(tx_hash)?;
        self.with_write_lock(|| {
            let mut rows = self.load_or_init_rows_unlocked()?;
            let mut record = Self::current_records(&rows)
                .into_iter()
                .find(|item| item.tx_hash == tx_hash)
                .ok_or_else(|| TxStorageError::NotFound(tx_hash.to_string()))?;

            record.status = TxStatus::Failed;
            record.block_height = None;
            record.confirmation_evidence = None;
            self.append_row_unlocked(&mut rows, WalletTxHistoryEntryKind::Conflicted, record)?;
            self.persist_rows_unlocked(&rows)
        })
    }

    fn record_already_spent(&mut self, tx_hash: &str) -> TxStorageResult<()> {
        Self::validate_tx_hash(tx_hash)?;
        self.with_write_lock(|| {
            let mut rows = self.load_or_init_rows_unlocked()?;
            let mut record = Self::current_records(&rows)
                .into_iter()
                .find(|item| item.tx_hash == tx_hash)
                .ok_or_else(|| TxStorageError::NotFound(tx_hash.to_string()))?;

            record.status = TxStatus::Failed;
            record.block_height = None;
            record.confirmation_evidence = None;
            self.append_row_unlocked(&mut rows, WalletTxHistoryEntryKind::AlreadySpent, record)?;
            self.persist_rows_unlocked(&rows)
        })
    }

    fn restore_snapshot(
        &mut self,
        record: TxRecord,
        latest_kind: WalletTxHistoryEntryKind,
    ) -> TxStorageResult<()> {
        Self::validate_tx_hash(&record.tx_hash)?;
        self.with_write_lock(|| {
            let mut rows = self.load_or_init_rows_unlocked()?;
            self.append_row_unlocked(&mut rows, latest_kind, record)?;
            self.persist_rows_unlocked(&rows)
        })
    }

    fn delete(&mut self, tx_hash: &str) -> TxStorageResult<()> {
        Self::validate_tx_hash(tx_hash)?;
        self.with_write_lock(|| {
            let mut rows = self.load_or_init_rows_unlocked()?;
            let record = Self::current_records(&rows)
                .into_iter()
                .find(|item| item.tx_hash == tx_hash)
                .ok_or_else(|| TxStorageError::NotFound(tx_hash.to_string()))?;

            self.append_row_unlocked(&mut rows, WalletTxHistoryEntryKind::Tombstoned, record)?;
            self.persist_rows_unlocked(&rows)
        })
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::backup::{decode_tx_history_rows, WalletTxHistoryEntryKind};
    use z00z_core::assets::AssetPkgWire;
    use z00z_utils::io::{create_dir_all, path_exists, read_file, write_file};
    use z00z_utils::time::MockTimeProvider;

    fn sample_record(tx_hash: &str) -> TxRecord {
        TxRecord {
            tx_hash: tx_hash.to_string(),
            tx_bytes: vec![1, 2, 3, 4],
            imported: false,
            status: TxStatus::Pending,
            timestamp_ms: 1000,
            block_height: None,
            confirmation_evidence: None,
        }
    }

    fn sample_tx_asset(serial_id: u32, amount: u64) -> z00z_core::Asset {
        z00z_core::genesis::asset_std::asset_from_dev_cfg("z00z", serial_id, amount)
            .expect("valid std asset")
    }

    fn sample_tx_bytes(asset: &z00z_core::Asset) -> Vec<u8> {
        let package = crate::tx::TxPackage {
            kind: "TxPackage".to_string(),
            package_type: "regular_tx".to_string(),
            version: 1,
            chain_id: 3,
            chain_type: "devnet".to_string(),
            chain_name: "devnet".to_string(),
            tx: crate::tx::TxWire {
                tx_type: "regular_tx".to_string(),
                inputs: Vec::new(),
                outputs: vec![crate::tx::TxOutputWire {
                    role: crate::tx::TxOutRole::Recipient,
                    asset_wire: AssetPkgWire::from_asset(asset),
                }],
                fee: 0,
                nonce: 1,
                context: crate::tx::TxContextWire::default(),
                proof: crate::tx::TxProofWire::default(),
                auth: crate::tx::TxAuthWire::default(),
            },
            tx_digest_hex: hex::encode([9u8; 32]),
            status: "pending".to_string(),
        };

        JsonCodec.serialize(&package).unwrap()
    }

    #[test]
    fn test_put_get_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::default();
        let history_path = dir.path().join("wallet_abc_tx_history.jsonl");
        let mut store = TxStorageImpl::new(&history_path, time);

        let record = sample_record("tx1");
        store.put(record.clone()).unwrap();

        let loaded = store.get("tx1").unwrap();
        assert_eq!(loaded.tx_hash, "tx1");
        assert_eq!(loaded.status, TxStatus::Pending);
        assert!(path_exists(&history_path).unwrap());

        let rows = decode_tx_history_rows(&read_file(&history_path).unwrap()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].wallet_stem, "abc");
        assert_eq!(rows[0].sequence, 1);
        assert_eq!(rows[0].entry_kind, WalletTxHistoryEntryKind::Created);
        assert!(rows[0].previous_entry_hash.is_none());
    }

    #[test]
    fn test_persist_rejects_oversized_rows() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::default();
        let history_path = dir.path().join("wallet_abc_tx_history.jsonl");
        let store = TxStorageImpl::new(&history_path, time);
        let mut record = sample_record("tx1");
        record.tx_bytes = vec![7; MAX_TX_HISTORY_JSONL_BYTES as usize];
        let row = WalletTxHistoryJsonlEntry::build_event(
            "abc",
            1,
            record.timestamp_ms,
            WalletTxHistoryEntryKind::Created,
            None,
            record,
        )
        .unwrap();

        let err = store.persist_rows_unlocked(&[row]).unwrap_err();
        assert!(matches!(
            err,
            TxStorageError::Serialization(ref msg)
                if msg.contains("tx-history JSONL file too large")
        ));
    }

    #[test]
    fn test_update_status() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::from_unix_secs(2000);
        let history_path = dir.path().join("wallet_abc_tx_history.jsonl");
        let mut store = TxStorageImpl::new(&history_path, time.clone());

        store.put(sample_record("tx1")).unwrap();

        store.update_status("tx1", TxStatus::Confirmed).unwrap();
        let updated = store.get("tx1").unwrap();
        assert_eq!(updated.status, TxStatus::Confirmed);
        assert_eq!(updated.timestamp_ms, 2000000); // 2000 sec in ms
        assert!(path_exists(&history_path).unwrap());

        let rows = decode_tx_history_rows(&read_file(&history_path).unwrap()).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].entry_kind, WalletTxHistoryEntryKind::Created);
        assert_eq!(rows[0].record.status, TxStatus::Pending);
        assert_eq!(rows[1].entry_kind, WalletTxHistoryEntryKind::Confirmed);
        assert_eq!(rows[1].record.status, TxStatus::Confirmed);
        assert_eq!(rows[1].previous_entry_hash, Some(rows[0].entry_hash));
    }

    #[test]
    fn test_failed_clears_confirmation() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::from_unix_secs(2001);
        let history_path = dir.path().join("wallet_abc_tx_history.jsonl");
        let mut store = TxStorageImpl::new(&history_path, time);

        let mut record = sample_record("tx1");
        record.status = TxStatus::Confirmed;
        record.block_height = Some(42);
        record.confirmation_evidence = Some(TxConfirmationEvidence {
            tx_id: "tx1".to_string(),
            tx_hash_hex: "11".repeat(32),
            chain_id: 7,
            block_height: 42,
            checkpoint_id_hex: "22".repeat(32),
            prev_root_hex: "33".repeat(32),
            new_root_hex: "44".repeat(32),
            spent_asset_ids_hex: vec!["55".repeat(32)],
            created_asset_ids_hex: vec!["66".repeat(32)],
            confirmed_at: 2_001_000,
            verified: true,
        });
        store.put(record).unwrap();

        store.record_failed("tx1").unwrap();

        let failed = store.get("tx1").unwrap();
        assert_eq!(failed.status, TxStatus::Failed);
        assert_eq!(failed.block_height, None);
        assert!(failed.confirmation_evidence.is_none());

        let rows = decode_tx_history_rows(&read_file(&history_path).unwrap()).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].entry_kind, WalletTxHistoryEntryKind::Failed);
        assert_eq!(rows[1].record.status, TxStatus::Failed);
        assert_eq!(rows[1].record.block_height, None);
        assert!(rows[1].record.confirmation_evidence.is_none());
    }

    #[test]
    fn test_keeps_created_day_restart() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::from_unix_secs(86_399);
        let history_path = dir.path().join("wallet_abc_tx_history.jsonl");
        let mut store = TxStorageImpl::new(&history_path, time.clone());

        let asset = sample_tx_asset(7, 6);
        let mut record = sample_record("tx1");
        record.tx_bytes = sample_tx_bytes(&asset);

        store.put(record).unwrap();
        time.advance_by(std::time::Duration::from_secs(2));
        store.record_confirmed("tx1", 9).unwrap();

        let reopened = TxStorageImpl::new(&history_path, time);
        let day_one = reopened
            .policy_spend_window(asset.definition.id, 0, 86_400_000)
            .unwrap();
        let day_two = reopened
            .policy_spend_window(asset.definition.id, 86_400_000, 172_800_000)
            .unwrap();

        assert_eq!(day_one.spent_amount, 6);
        assert_eq!(day_one.pending_confirmation_count, 0);
        assert_eq!(day_two.spent_amount, 0);
    }

    #[test]
    fn test_counts_pending_skips_cancelled() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::from_unix_secs(10);
        let history_path = dir.path().join("wallet_abc_tx_history.jsonl");
        let mut store = TxStorageImpl::new(&history_path, time.clone());

        let first_asset = sample_tx_asset(1, 4);
        let mut first = sample_record("tx1");
        first.tx_bytes = sample_tx_bytes(&first_asset);
        store.put(first).unwrap();

        let second_asset = sample_tx_asset(2, 3);
        let mut second = sample_record("tx2");
        second.tx_bytes = sample_tx_bytes(&second_asset);
        store.put(second).unwrap();
        store.record_cancelled("tx2").unwrap();

        let window = store
            .policy_spend_window(first_asset.definition.id, 0, 86_400_000)
            .unwrap();

        assert_eq!(window.spent_amount, 4);
        assert_eq!(window.pending_confirmation_count, 1);
    }

    #[test]
    fn test_list_by_status() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::default();
        let history_path = dir.path().join("wallet_abc_tx_history.jsonl");
        let mut store = TxStorageImpl::new(&history_path, time);

        let mut record1 = sample_record("tx1");
        record1.status = TxStatus::Pending;
        store.put(record1).unwrap();

        let mut record2 = sample_record("tx2");
        record2.status = TxStatus::Confirmed;
        store.put(record2).unwrap();

        let mut record3 = sample_record("tx3");
        record3.status = TxStatus::Pending;
        store.put(record3).unwrap();

        let pending = store.list_by_status(TxStatus::Pending).unwrap();
        assert_eq!(pending.len(), 2);

        let confirmed = store.list_by_status(TxStatus::Confirmed).unwrap();
        assert_eq!(confirmed.len(), 1);
        assert!(path_exists(&history_path).unwrap());
    }

    #[test]
    fn test_returns_history_rows() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::default();
        let history_path = dir.path().join("wallet_abc_tx_history.jsonl");
        let mut store = TxStorageImpl::new(&history_path, time);

        store.put(sample_record("tx1")).unwrap();
        store.record_submitted("tx1").unwrap();
        store.record_admitted("tx1").unwrap();

        let rows = store.list_history_rows().unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].sequence, 1);
        assert_eq!(rows[1].sequence, 2);
        assert_eq!(rows[2].sequence, 3);
        assert_eq!(rows[0].entry_kind, WalletTxHistoryEntryKind::Created);
        assert_eq!(rows[1].entry_kind, WalletTxHistoryEntryKind::Submitted);
        assert_eq!(rows[2].entry_kind, WalletTxHistoryEntryKind::Admitted);
    }

    #[test]
    fn test_append_in_sequence_order() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::default();
        let history_path = dir.path().join("wallet_abc_tx_history.jsonl");
        let mut store = TxStorageImpl::new(&history_path, time);

        store.put(sample_record("tx1")).unwrap();
        store.put(sample_record("tx2")).unwrap();
        store.put(sample_record("tx3")).unwrap();

        let rows = decode_tx_history_rows(&read_file(&history_path).unwrap()).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].sequence, 1);
        assert_eq!(rows[1].sequence, 2);
        assert_eq!(rows[2].sequence, 3);
        assert_eq!(rows[0].tx_hash, "tx1");
        assert_eq!(rows[1].tx_hash, "tx2");
        assert_eq!(rows[2].tx_hash, "tx3");
        assert_eq!(rows[1].previous_entry_hash, Some(rows[0].entry_hash));
        assert_eq!(rows[2].previous_entry_hash, Some(rows[1].entry_hash));
    }

    #[test]
    fn test_delete_missing_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::default();
        let history_path = dir.path().join("wallet_abc_tx_history.jsonl");
        let mut store = TxStorageImpl::new(&history_path, time);

        let err = store.delete("missing_tx").unwrap_err();
        assert!(matches!(err, TxStorageError::NotFound(_)));
    }

    #[test]
    fn test_appends_fold_latest_row() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::default();
        let history_path = dir.path().join("wallet_abc_tx_history.jsonl");
        let mut store = TxStorageImpl::new(&history_path, time);

        let mut first = sample_record("tx1");
        first.tx_bytes = vec![1, 1, 1];
        first.timestamp_ms = 1000;
        store.put(first).unwrap();

        let mut second = sample_record("tx1");
        second.tx_bytes = vec![2, 2, 2];
        second.timestamp_ms = 2000;
        second.status = TxStatus::Confirmed;
        store.put(second.clone()).unwrap();

        let rows = decode_tx_history_rows(&read_file(&history_path).unwrap()).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].tx_hash, "tx1");
        assert_eq!(rows[1].tx_hash, "tx1");
        assert_eq!(rows[0].entry_kind, WalletTxHistoryEntryKind::Created);
        assert_eq!(rows[1].entry_kind, WalletTxHistoryEntryKind::Created);
        assert_eq!(store.get("tx1").unwrap(), second);
    }

    #[test]
    fn test_appends_conflicted_kind() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::default();
        let history_path = dir.path().join("wallet_abc_tx_history.jsonl");
        let mut store = TxStorageImpl::new(&history_path, time);

        store.put(sample_record("tx1")).unwrap();
        store.record_conflicted("tx1").unwrap();

        let current = store.get("tx1").unwrap();
        assert_eq!(current.status, TxStatus::Failed);

        let rows = store.list_history_rows().unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].entry_kind, WalletTxHistoryEntryKind::Conflicted);
        assert_eq!(rows[1].record.status, TxStatus::Failed);
    }

    #[test]
    fn test_appends_already_spent_kind() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::default();
        let history_path = dir.path().join("wallet_abc_tx_history.jsonl");
        let mut store = TxStorageImpl::new(&history_path, time);

        store.put(sample_record("tx1")).unwrap();
        store.record_already_spent("tx1").unwrap();

        let current = store.get("tx1").unwrap();
        assert_eq!(current.status, TxStatus::Failed);

        let rows = store.list_history_rows().unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].entry_kind, WalletTxHistoryEntryKind::AlreadySpent);
        assert_eq!(rows[1].record.status, TxStatus::Failed);
    }

    #[test]
    fn test_snapshot_restores_failed_kind() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::default();
        let history_path = dir.path().join("wallet_abc_tx_history.jsonl");
        let mut store = TxStorageImpl::new(&history_path, time);

        store.put(sample_record("tx1")).unwrap();
        store.record_submitted("tx1").unwrap();
        store.record_admitted("tx1").unwrap();
        let previous = store.get("tx1").unwrap();

        store.record_confirmed("tx1", 42).unwrap();
        store
            .restore_snapshot(previous.clone(), WalletTxHistoryEntryKind::Admitted)
            .unwrap();

        let rows = store.list_history_rows().unwrap();
        assert_eq!(
            rows.last().map(|row| row.entry_kind),
            Some(WalletTxHistoryEntryKind::Admitted)
        );
        assert_eq!(store.get("tx1").unwrap(), previous);
    }

    #[test]
    fn test_tombstone_no_erasing_pkg() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::from_unix_secs(3000);
        let history_path = dir.path().join("wallet_abc_tx_history.jsonl");
        let mut store = TxStorageImpl::new(&history_path, time);

        let record = sample_record("tx1");
        let tx_bytes = record.tx_bytes.clone();
        store.put(record).unwrap();
        store.delete("tx1").unwrap();

        assert!(matches!(
            store.get("tx1").unwrap_err(),
            TxStorageError::NotFound(_)
        ));
        assert!(store.list().unwrap().is_empty());

        let rows = decode_tx_history_rows(&read_file(&history_path).unwrap()).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].entry_kind, WalletTxHistoryEntryKind::Tombstoned);
        assert_eq!(rows[1].record.tx_bytes, tx_bytes);
        assert_eq!(rows[1].record.timestamp_ms, 3000000);
    }

    #[test]
    fn test_history_dir_not_loaded() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::from_unix_secs(4000);
        let history_path = dir.path().join("wallet_abc_tx_history.jsonl");
        let noncanonical_dir = dir.path().join("wallet_abc_tx_history");
        create_dir_all(&noncanonical_dir).unwrap();
        write_file(noncanonical_dir.join("ignored-noncanonical.json"), b"{}").unwrap();

        let mut store = TxStorageImpl::new(&history_path, time);
        assert!(store.list().unwrap().is_empty());
        assert!(path_exists(&noncanonical_dir).unwrap());

        let rows = decode_tx_history_rows(&read_file(&history_path).unwrap()).unwrap();
        assert!(rows.is_empty());

        store.put(sample_record("tx-new")).unwrap();
        let rows = decode_tx_history_rows(&read_file(&history_path).unwrap()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].entry_kind, WalletTxHistoryEntryKind::Created);
        assert!(path_exists(noncanonical_dir.join("ignored-noncanonical.json")).unwrap());
    }

    #[test]
    fn test_invalid_tx_hash_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let time = MockTimeProvider::default();
        let history_path = dir.path().join("wallet_abc_tx_history.jsonl");
        let store = TxStorageImpl::new(&history_path, time);

        let err = store.get("../bad_tx").unwrap_err();
        assert!(matches!(err, TxStorageError::InvalidTxHash(_)));
    }
}
