use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use serde::Deserialize;
use serde_json::Value;
use z00z_simulator::{config::ScenarioCfg, scenario_1::stage_6, StageResult};
use z00z_storage::{
    settlement::{CheckRoot, SettlementStore, StoreItem, StoreOp},
    snapshot::{PrepFsStore, PrepSnapshot, PrepSnapshotId, PrepSnapshotStore},
};
use z00z_utils::io::{create_dir_all, load_json, read_to_string, write_file};

use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::scenario_support;
use z00z_simulator::scenario_1::support::stage_runner_support;

use crate::stage4_paths::assert_absent;
use scenario_support::make_cfg_in;

#[derive(Debug, Deserialize)]
struct PrepRefFile {
    snapshot_id_hex: String,
}

struct OutCase {
    out: PathBuf,
}

struct FailCase {
    out: PathBuf,
    msg: String,
}

fn root_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn lock_root() -> std::sync::MutexGuard<'static, ()> {
    root_lock().lock().unwrap_or_else(|err| err.into_inner())
}

fn good_s4(cfg: &mut ScenarioCfg) {
    cfg.simulation.use_mock_rng = true;
    cfg.simulation.mock_rng_seed = Some(42);
    for asset in &mut cfg.genesis_assets {
        asset.serials = asset.serials.min(6);
    }

    if let Some(stage3) = cfg.stage3_claim.as_mut() {
        stage3.rng_seed = Some(42);
    }

    let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_min = 4;
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_target = 4;
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_max = 4;
    stage4.transaction.outputs.bob_outputs_count = 4;
    stage4.transaction.class = "Coin".to_string();
    stage4.transaction.symbol = "Z00Z".to_string();
    stage4.transaction.mode = "fraction".to_string();
    stage4.transaction.fraction = Some(0.1);
    stage4.transaction.amount = None;
}

fn prep_ref_file(out: &Path) -> PathBuf {
    out.join("transactions/checkpoint_prep.json")
}

fn tx_file(out: &Path) -> PathBuf {
    out.join("transactions/tx_alice_to_bob_pkg.json")
}

fn pending_file(out: &Path) -> PathBuf {
    out.join("transactions/wallets_pending.json")
}

fn root_tamper_file(out: &Path) -> PathBuf {
    out.parent()
        .unwrap_or(out)
        .join("test_hooks/stage4_root_tamper.txt")
}

fn write_root_tamper(out: &Path, mode: &str) {
    let path = root_tamper_file(out);
    create_dir_all(path.parent().expect("tamper dir")).expect("tamper dir");
    write_file(path, mode.as_bytes()).expect("write root tamper file");
}

fn load_snapshot(out: &Path) -> PrepSnapshot {
    let prep_ref: PrepRefFile = load_json(prep_ref_file(out)).expect("prep ref");
    let raw = hex::decode(prep_ref.snapshot_id_hex).expect("snapshot id hex");
    let snap_id = PrepSnapshotId::new(raw.try_into().expect("snapshot id size"));
    let store = PrepFsStore::new(out.join("transactions"));
    store.load_snapshot(&snap_id).expect("prep snapshot")
}

fn storage_root(snapshot: &PrepSnapshot) -> CheckRoot {
    let mut store = SettlementStore::new();
    let ops = snapshot
        .entries
        .iter()
        .map(|entry| {
            let item = StoreItem::new(entry.path(), entry.leaf().clone()).expect("store item");
            StoreOp::Put(Box::new(item))
        })
        .collect::<Vec<_>>();
    store.apply_settlement_ops(ops).expect("apply ops");
    CheckRoot::from(store.settlement_root().expect("storage root"))
}

fn ok_case() -> &'static OutCase {
    static CASE: OnceLock<OutCase> = OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage4_root_support_ok_v1", |base| {
            let (cfg_path, design_path, out) = make_cfg_in(base, good_s4);
            let _ctx = stage_runner_support::run_stage4_setup(&cfg_path, &design_path);
            assert!(prep_ref_file(&out).exists(), "cached prep ref missing");
            assert!(tx_file(&out).exists(), "cached tx package missing");
        });
        OutCase {
            out: root.join("outputs/scenario_1"),
        }
    })
}

fn root_drift_case() -> &'static FailCase {
    static CASE: OnceLock<FailCase> = OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage4_root_support_prev_root_hex_v1", |base| {
            let (cfg_path, design_path, out) = make_cfg_in(base, good_s4);
            let mut ctx = stage_runner_support::run_stage5_session(&cfg_path, &design_path);
            write_root_tamper(&out, "prev_root_hex");
            let stage = stage_runner_support::stage_by_id(&design_path, 6);
            let msg = match stage_6::run_tx_prepare(&mut ctx, &stage) {
                StageResult::Fail(msg) => msg,
                other => panic!("tx_prepare stage must fail, got {other:?}"),
            };
            write_file(base.join("fail_msg.txt"), msg.as_bytes()).expect("write root fail msg");
        });
        FailCase {
            out: root.join("outputs/scenario_1"),
            msg: read_to_string(root.join("fail_msg.txt")).expect("read root fail msg"),
        }
    })
}

#[test]
fn test_stage4_stays_loads_snapshot() {
    let _guard = lock_root();
    let out = &ok_case().out;

    let prep_json: Value = load_json(prep_ref_file(out)).expect("prep ref json");
    let prep_obj = prep_json.as_object().expect("prep ref object");
    assert_eq!(
        prep_obj.len(),
        1,
        "checkpoint_prep.json must stay a thin snapshot reference"
    );
    assert!(
        prep_obj
            .get("snapshot_id_hex")
            .and_then(Value::as_str)
            .is_some(),
        "checkpoint_prep.json must expose snapshot_id_hex"
    );
    assert!(
        prep_obj.get("prev_root_hex").is_none(),
        "checkpoint_prep.json must not inline canonical proof payload"
    );

    let snapshot = load_snapshot(out);
    assert!(
        !snapshot.entries.is_empty(),
        "persisted canonical snapshot must include prep entries"
    );
    assert_eq!(
        storage_root(&snapshot),
        snapshot.prev_root,
        "persisted PrepSnapshot must keep the storage-owned root"
    );
    assert!(tx_file(out).exists(), "tx package must exist on success");
}

#[test]
fn test_stage4_rejects_pre_artifacts() {
    let _guard = lock_root();
    let case = root_drift_case();
    assert!(
        case.msg.contains("stage4: canonical snapshot root drift"),
        "unexpected error: {}",
        case.msg
    );

    assert_absent(&prep_ref_file(&case.out));
    assert_absent(&tx_file(&case.out));
    assert_absent(&pending_file(&case.out));
}
