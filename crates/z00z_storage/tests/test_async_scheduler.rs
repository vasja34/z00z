#![cfg(debug_assertions)]

use std::sync::{Mutex, OnceLock};

use tempfile::tempdir;
use z00z_core::assets::{AssetLeaf, AssetPackPlain};
use z00z_crypto::ZkPackEncrypted;
use z00z_storage::settlement::{
    AdaptiveProofErr, DefinitionId, ForestSchedulerMetrics, SerialId, SettlementPath,
    SettlementStore, SettlementStoreError, SettlementTreeBackend, StoreItem, StoreOp, TerminalId,
    TerminalLeaf,
};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

const PROOF_BATCH_MODE_ENV: &str = "Z00Z_STORAGE_PROOF_BATCH_MODE";

struct EnvGuard {
    bucket_bits: Option<String>,
    redb_root: Option<String>,
}

impl EnvGuard {
    fn with_bucket_bits(bits: &str) -> Self {
        let guard = Self {
            bucket_bits: std::env::var("Z00Z_SETTLEMENT_BUCKET_BITS").ok(),
            redb_root: std::env::var("Z00Z_STORAGE_REDB_ROOT").ok(),
        };
        std::env::set_var("Z00Z_SETTLEMENT_BUCKET_BITS", bits);
        std::env::remove_var("Z00Z_STORAGE_REDB_ROOT");
        guard
    }

    fn with_redb(bits: &str, root: &std::path::Path) -> Self {
        let guard = Self {
            bucket_bits: std::env::var("Z00Z_SETTLEMENT_BUCKET_BITS").ok(),
            redb_root: std::env::var("Z00Z_STORAGE_REDB_ROOT").ok(),
        };
        std::env::set_var("Z00Z_SETTLEMENT_BUCKET_BITS", bits);
        std::env::set_var("Z00Z_STORAGE_REDB_ROOT", root.display().to_string());
        guard
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(bits) = &self.bucket_bits {
            std::env::set_var("Z00Z_SETTLEMENT_BUCKET_BITS", bits);
        } else {
            std::env::remove_var("Z00Z_SETTLEMENT_BUCKET_BITS");
        }
        if let Some(root) = &self.redb_root {
            std::env::set_var("Z00Z_STORAGE_REDB_ROOT", root);
        } else {
            std::env::remove_var("Z00Z_STORAGE_REDB_ROOT");
        }
    }
}

struct ProofBatchModeGuard {
    prev_mode: Option<String>,
}

impl ProofBatchModeGuard {
    fn unset() -> Self {
        let prev_mode = std::env::var(PROOF_BATCH_MODE_ENV).ok();
        std::env::remove_var(PROOF_BATCH_MODE_ENV);
        Self { prev_mode }
    }

    fn parallel() -> Self {
        let prev_mode = std::env::var(PROOF_BATCH_MODE_ENV).ok();
        std::env::set_var(PROOF_BATCH_MODE_ENV, "parallel");
        Self { prev_mode }
    }
}

impl Drop for ProofBatchModeGuard {
    fn drop(&mut self) {
        if let Some(mode) = &self.prev_mode {
            std::env::set_var(PROOF_BATCH_MODE_ENV, mode);
        } else {
            std::env::remove_var(PROOF_BATCH_MODE_ENV);
        }
    }
}

struct TimingEnvGuard {
    prev_out: Option<String>,
    prev_run: Option<String>,
}

impl TimingEnvGuard {
    fn enable(path: &std::path::Path, run: &str) -> Self {
        let prev_out = std::env::var("Z00Z_SETTLEMENT_TIME_OUT").ok();
        let prev_run = std::env::var("Z00Z_SETTLEMENT_TIME_RUN").ok();
        std::env::set_var("Z00Z_SETTLEMENT_TIME_OUT", path.display().to_string());
        std::env::set_var("Z00Z_SETTLEMENT_TIME_RUN", run);
        Self { prev_out, prev_run }
    }
}

impl Drop for TimingEnvGuard {
    fn drop(&mut self) {
        if let Some(path) = &self.prev_out {
            std::env::set_var("Z00Z_SETTLEMENT_TIME_OUT", path);
        } else {
            std::env::remove_var("Z00Z_SETTLEMENT_TIME_OUT");
        }
        if let Some(run) = &self.prev_run {
            std::env::set_var("Z00Z_SETTLEMENT_TIME_RUN", run);
        } else {
            std::env::remove_var("Z00Z_SETTLEMENT_TIME_RUN");
        }
    }
}

fn path(definition: u8, serial: u32, asset: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new([definition; 32]),
        SerialId::new(serial),
        TerminalId::new([asset; 32]),
    )
}

fn leaf(path: SettlementPath, value: u64) -> TerminalLeaf {
    let payload = AssetPackPlain {
        value,
        blinding: [3u8; 32],
        s_out: [4u8; 32],
    }
    .to_bytes();

    AssetLeaf {
        asset_id: path.terminal_id().into_bytes(),
        serial_id: path.serial_id.get(),
        r_pub: [1u8; 32],
        owner_tag: [2u8; 32],
        c_amount: [5u8; 32],
        enc_pack: ZkPackEncrypted {
            version: 1,
            ciphertext: payload,
            tag: [0u8; 16],
        },
        range_proof: vec![9u8; 4],
        tag16: 11,
    }
    .into()
}

fn item(path: SettlementPath, value: u64) -> StoreItem {
    StoreItem::new(path, leaf(path, value)).expect("store item")
}

fn put_ops(paths: &[SettlementPath], base: u64) -> Vec<StoreOp> {
    paths
        .iter()
        .map(|path| {
            let value = base.saturating_add(u64::from(path.terminal_id().into_bytes()[0]));
            StoreOp::Put(Box::new(item(*path, value)))
        })
        .collect()
}

fn commit(store: &mut SettlementStore, ops: Vec<StoreOp>) {
    SettlementTreeBackend::apply_settlement_ops(store, ops).expect("settlement batch");
}

fn seed_paths() -> Vec<SettlementPath> {
    vec![
        path(51, 1, 1),
        path(51, 2, 2),
        path(51, 3, 3),
        path(52, 1, 4),
        path(52, 2, 5),
        path(53, 1, 6),
    ]
}

fn hot_paths() -> Vec<SettlementPath> {
    vec![
        path(61, 1, 1),
        path(61, 1, 2),
        path(61, 2, 3),
        path(61, 3, 4),
        path(62, 1, 5),
        path(62, 2, 6),
    ]
}

fn same_bucket_group(store: &mut SettlementStore, needed: usize) -> Vec<SettlementPath> {
    let policy = store.bucket_policy();
    let first = path(71, 9, 1);
    let bucket_id = first.bucket_id(policy);
    let mut selected = vec![(1u8, first)];
    for seed in 2..=255 {
        let candidate = path(71, 9, seed);
        if candidate.bucket_id(policy) == bucket_id {
            selected.push((seed, candidate));
            if selected.len() == needed {
                break;
            }
        }
    }

    assert_eq!(selected.len(), needed, "failed to find same-bucket group");
    let ops = selected
        .iter()
        .map(|(seed, candidate)| {
            StoreOp::Put(Box::new(item(*candidate, 11_100 + u64::from(*seed))))
        })
        .collect();
    commit(store, ops);
    selected.into_iter().map(|(_, path)| path).collect()
}

fn assert_blocking(metrics: ForestSchedulerMetrics) {
    let thread_name = metrics
        .last_blocking_thread
        .expect("blocking thread name recorded");
    assert!(thread_name.starts_with("z00z-hjmt-blocking-"));
}

#[test]
fn test_terminal_commits_stable_skew() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let _env = EnvGuard::with_bucket_bits("2");

    let paths = seed_paths();
    let mut serial = SettlementStore::new();
    serial.reset_sched_for_test();
    serial.set_sched_limits_for_test(1, 64);
    commit(&mut serial, put_ops(&paths, 5_100));
    let serial_root = serial.settlement_root().expect("serial root");

    let mut scheduled = SettlementStore::new();
    scheduled.reset_sched_for_test();
    scheduled.set_sched_limits_for_test(2, 64);
    scheduled.set_sched_test_skew_ms(8);
    commit(&mut scheduled, put_ops(&paths, 5_100));
    let target = paths[0];
    let metrics = scheduled.forest_scheduler_metrics();

    assert_eq!(
        scheduled.settlement_root().expect("scheduled root"),
        serial_root
    );
    assert_eq!(
        scheduled
            .settlement_proof_blob(&target)
            .expect("scheduled proof"),
        serial.settlement_proof_blob(&target).expect("serial proof")
    );
    assert!(metrics.max_active >= 2);
    assert!(metrics.max_active <= 2);
}

#[test]
fn test_batch_verifies_input_order() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let _env = EnvGuard::with_bucket_bits("2");
    let _mode = ProofBatchModeGuard::parallel();

    let paths = hot_paths();
    let mut store = SettlementStore::new();
    commit(&mut store, put_ops(&paths, 6_100));
    store.reset_sched_for_test();
    store.set_sched_limits_for_test(3, 64);
    store.set_sched_test_skew_ms(6);

    let batch = store
        .settlement_proof_blobs(&paths)
        .expect("proof batch through scheduler");
    let metrics = store.forest_scheduler_metrics();
    assert_eq!(batch.len(), paths.len());

    for (idx, blob) in batch.iter().enumerate() {
        assert_eq!(blob.item().path(), paths[idx]);
        store
            .validate_settlement_proof_blob(blob)
            .expect("batch proof validates");
    }
    assert!(metrics.max_active >= 2);
    assert!(metrics.max_active <= 3);
}

#[test]
fn test_timing_is_observability() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let _env = EnvGuard::with_bucket_bits("2");
    let temp = tempdir().expect("tempdir");
    let timing_path = temp.path().join("settlement.timing.tsv");
    let _timing = TimingEnvGuard::enable(&timing_path, "timing-observability");

    let paths = hot_paths();
    let target = paths[0];
    let mut timed = SettlementStore::new();
    timed.reset_sched_for_test();
    timed.set_sched_limits_for_test(2, 64);
    commit(&mut timed, put_ops(&paths, 8_100));
    let timed_root = timed.settlement_root().expect("timed root");
    let timed_blob = timed
        .settlement_proof_blob(&target)
        .expect("timed proof blob");

    let mut plain = SettlementStore::new();
    plain.reset_sched_for_test();
    plain.set_sched_limits_for_test(2, 64);
    commit(&mut plain, put_ops(&paths, 8_100));

    assert_eq!(plain.settlement_root().expect("plain root"), timed_root);
    assert_eq!(
        plain
            .settlement_proof_blob(&target)
            .expect("plain proof blob"),
        timed_blob
    );

    let timing_rows = std::fs::read_to_string(&timing_path).expect("timing rows");
    assert!(timing_rows.contains("run=timing-observability"));
    assert!(timing_rows.contains("stage=hjmt_apply_ops"));
}

#[test]
fn test_batch_defaults_serial_mode() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let _env = EnvGuard::with_bucket_bits("2");
    let _mode = ProofBatchModeGuard::unset();

    let paths = hot_paths();
    let mut store = SettlementStore::new();
    commit(&mut store, put_ops(&paths, 6_500));
    store.reset_sched_for_test();
    store.set_sched_limits_for_test(4, 64);
    store.set_sched_test_skew_ms(6);

    let batch = store
        .settlement_proof_blobs(&paths)
        .expect("proof batch through default serial mode");
    let metrics = store.forest_scheduler_metrics();

    assert_eq!(batch.len(), paths.len());
    assert_eq!(metrics.last_batch, paths.len());
    assert!(metrics.max_active <= 1);
}

#[test]
fn test_parallel_keeps_serial() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let _env = EnvGuard::with_bucket_bits("2");
    let _mode = ProofBatchModeGuard::parallel();

    let paths = hot_paths().into_iter().take(3).collect::<Vec<_>>();
    let mut store = SettlementStore::new();
    commit(&mut store, put_ops(&paths, 6_800));
    store.reset_sched_for_test();
    store.set_sched_limits_for_test(4, 64);
    store.set_sched_test_skew_ms(6);

    let batch = store
        .settlement_proof_blobs(&paths)
        .expect("small proof batch stays serial");
    let metrics = store.forest_scheduler_metrics();

    assert_eq!(batch.len(), paths.len());
    assert_eq!(metrics.last_batch, paths.len());
    assert!(metrics.max_active <= 1);
}

#[test]
fn test_rejects_no_root_drift() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let _env = EnvGuard::with_bucket_bits("2");

    let paths = seed_paths();
    let mut store = SettlementStore::new();
    let root_before = store.settlement_root().expect("root before");
    store.reset_sched_for_test();
    store.set_sched_limits_for_test(2, 1);

    let err = SettlementTreeBackend::apply_settlement_ops(&mut store, put_ops(&paths, 7_100))
        .expect_err("terminal batch queue must backpressure");
    match err {
        SettlementStoreError::SchedBackpressure {
            stage,
            queued,
            limit,
        } => {
            assert_eq!(stage, "hjmt_plan_ops");
            assert!(queued > limit);
        }
        other => panic!("unexpected error: {other}"),
    }
    assert_eq!(store.settlement_root().expect("root after"), root_before);
}

#[test]
fn test_sched_cancel_keeps_root() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let _env = EnvGuard::with_bucket_bits("2");

    let mut store = SettlementStore::new();
    let baseline = hot_paths()[0];
    commit(
        &mut store,
        vec![StoreOp::Put(Box::new(item(baseline, 8_001)))],
    );
    let root_before = store.settlement_root().expect("root before");

    store.reset_sched_for_test();
    store.set_sched_limits_for_test(2, 64);
    store.set_sched_cancel_for_test(Some(0));

    let err = SettlementTreeBackend::apply_settlement_ops(&mut store, put_ops(&hot_paths(), 8_100))
        .expect_err("scheduler cancel");
    assert!(matches!(
        err,
        SettlementStoreError::SchedCancel {
            stage: "hjmt_plan_ops"
        }
    ));
    assert_eq!(store.settlement_root().expect("root after"), root_before);
    assert_eq!(
        SettlementTreeBackend::get_settlement_item(&store, &baseline).expect("baseline lookup"),
        Some(item(baseline, 8_001))
    );
    assert_eq!(store.forest_scheduler_metrics().cancel_count, 1);
}

#[test]
fn test_redb_sync_dedicated_thread() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let temp = tempdir().expect("tempdir");
    let _env = EnvGuard::with_redb("2", temp.path());

    let mut store = SettlementStore::new();
    store.reset_sched_for_test();
    store.set_sched_limits_for_test(2, 64);
    commit(&mut store, put_ops(&seed_paths(), 9_100));

    assert_blocking(store.forest_scheduler_metrics());
}

#[test]
fn test_root_order_stable_perms() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let _env = EnvGuard::with_bucket_bits("1");

    let paths = hot_paths();
    let mut serial = SettlementStore::new();
    serial.reset_sched_for_test();
    serial.set_sched_limits_for_test(1, 64);
    commit(&mut serial, put_ops(&paths, 10_100));

    let mut skewed = SettlementStore::new();
    skewed.reset_sched_for_test();
    skewed.set_sched_limits_for_test(2, 64);
    skewed.set_sched_test_skew_ms(10);
    let mut reversed = paths.clone();
    reversed.reverse();
    commit(&mut skewed, put_ops(&reversed, 10_100));

    assert_eq!(
        skewed.settlement_root().expect("skewed root"),
        serial.settlement_root().expect("serial root")
    );
    assert_eq!(
        skewed
            .settlement_proof_blob(&paths[1])
            .expect("skewed proof"),
        serial
            .settlement_proof_blob(&paths[1])
            .expect("serial proof")
    );
}

#[test]
fn test_rebuild_respects_sched_backpressure() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let _env = EnvGuard::with_bucket_bits("1");

    let mut store = SettlementStore::new();
    let needed = usize::try_from(store.bucket_policy().min_bucket_count()).expect("usize") + 1;
    let paths = same_bucket_group(&mut store, needed);

    store.reset_sched_for_test();
    store.set_sched_limits_for_test(2, 1);

    let err = store
        .split_proof(&paths[0])
        .expect_err("policy rebuild backpressure");
    match err {
        AdaptiveProofErr::Store(SettlementStoreError::SchedBackpressure {
            stage,
            queued,
            limit,
        }) => {
            assert_eq!(stage, "hjmt_plan_ops");
            assert!(queued > limit);
        }
        other => panic!("unexpected error: {other}"),
    }
}
