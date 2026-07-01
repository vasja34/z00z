use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use z00z_simulator::{config::ScenarioCfg, scenario_1::stage_6, StageResult};
use z00z_utils::io::{create_dir_all, read_to_string, write_file};

use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::scenario_support;
use z00z_simulator::scenario_1::support::stage_runner_support;

use crate::stage4_paths::{assert_absent, lock_stage4_tamper};
use scenario_support::make_cfg_in;

fn lock_tamper() -> std::sync::MutexGuard<'static, ()> {
    lock_stage4_tamper()
}

fn good_s4(cfg: &mut ScenarioCfg) {
    for asset in &mut cfg.genesis_assets {
        asset.serials = asset.serials.min(6);
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
        .distinct_serial_ids_max = 10;
    stage4.transaction.outputs.bob_outputs_count = 4;
    stage4.transaction.class = "Coin".to_string();
    stage4.transaction.symbol = "Z00Z".to_string();
    stage4.transaction.mode = "fraction".to_string();
    stage4.transaction.fraction = Some(0.1);
    stage4.transaction.amount = None;
}

fn tamper_file(out: &Path) -> PathBuf {
    out.parent()
        .unwrap_or(out)
        .join("test_hooks/stage4_output_tamper.txt")
}

fn tx_file(out: &Path) -> PathBuf {
    out.join("transactions/tx_alice_to_bob_pkg.json")
}

fn snap_file(out: &Path) -> PathBuf {
    out.join("stage_4_snapshot.json")
}

fn after_file(out: &Path) -> PathBuf {
    out.join("transactions/wallets_state_after.json")
}

fn diff_file(out: &Path) -> PathBuf {
    out.join("transactions/wallets_state_diff.json")
}

fn pend_file(out: &Path) -> PathBuf {
    out.join("transactions/wallets_pending.json")
}

fn conf_file(out: &Path) -> PathBuf {
    out.join("transactions/wallets_confirmed.json")
}

fn no_post(out: &Path) {
    assert_absent(&tx_file(out));
    assert!(
        snap_file(out).exists(),
        "claim-publish snapshot must remain present"
    );
    assert_absent(&after_file(out));
    assert_absent(&diff_file(out));
    assert_absent(&pend_file(out));
    assert_absent(&conf_file(out));
}

fn write_tamper(out: &Path, mode: &str) {
    let path = tamper_file(out);
    create_dir_all(path.parent().expect("tamper dir")).expect("tamper dir");
    write_file(path, mode.as_bytes()).expect("write tamper file");
}

struct OutCase {
    out: PathBuf,
}

struct FailCase {
    out: PathBuf,
    msg: String,
}

fn ok_case() -> &'static OutCase {
    static CASE: OnceLock<OutCase> = OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage4_tamper_ok_v1", |base| {
            let (cfg_path, design_path, out) = make_cfg_in(base, good_s4);
            let _ctx = stage_runner_support::run_stage4_setup(&cfg_path, &design_path);
            assert!(tx_file(&out).exists(), "tamper control tx missing");
            assert!(snap_file(&out).exists(), "tamper control snapshot missing");
        });
        OutCase {
            out: root.join("outputs/scenario_1"),
        }
    })
}

fn build_tamper_case(base: &Path, mode: &str) {
    let (cfg_path, design_path, out) = make_cfg_in(base, good_s4);
    let mut ctx = stage_runner_support::run_stage5_session(&cfg_path, &design_path);
    write_tamper(&out, mode);
    let stage = stage_runner_support::stage_by_id(&design_path, 6);
    let msg = match stage_6::run_tx_prepare(&mut ctx, &stage) {
        StageResult::Fail(msg) => msg,
        other => panic!("tx_prepare stage must fail, got {other:?}"),
    };
    write_file(base.join("fail_msg.txt"), msg.as_bytes()).expect("write tamper fail msg");
}

fn load_fail_case(root: &Path) -> FailCase {
    FailCase {
        out: root.join("outputs/scenario_1"),
        msg: read_to_string(root.join("fail_msg.txt")).expect("read tamper fail msg"),
    }
}

fn tag16_case() -> &'static FailCase {
    static CASE: OnceLock<FailCase> = OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage4_tamper_tag16_v1", |base| {
            build_tamper_case(base, "tag16");
        });
        load_fail_case(&root)
    })
}

fn witness_case() -> &'static FailCase {
    static CASE: OnceLock<FailCase> = OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage4_tamper_witness_v1", |base| {
            build_tamper_case(base, "witness");
        });
        load_fail_case(&root)
    })
}

#[test]
fn test_stage4_tamper_ok() {
    let _guard = lock_tamper();
    let out = &ok_case().out;
    assert!(
        tx_file(out).exists(),
        "tx package must exist for control case"
    );
    assert!(
        snap_file(out).exists(),
        "snapshot must exist for control case"
    );
}

#[test]
fn test_stage4_rejects_tampered_output() {
    let _guard = lock_tamper();
    let case = tag16_case();

    assert!(
        case.msg.contains("tag16 mismatch"),
        "unexpected error: {}",
        case.msg
    );
    no_post(&case.out);
}

#[test]
fn test_stage4_rejects_bad_witness() {
    let _guard = lock_tamper();
    let case = witness_case();

    assert!(
        case.msg.contains("SpendWitness gate"),
        "unexpected error: {}",
        case.msg
    );
    no_post(&case.out);
}
