use std::path::{Path, PathBuf};

use z00z_simulator::{scenario_1::stage_6, StageResult};
use z00z_utils::{
    codec::Codec,
    io::{read_file, read_to_string, write_file},
};
use z00z_wallets::tx::{TxOutRole, TxPackage};

use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::scenario_support;
use z00z_simulator::scenario_1::support::stage_runner_support;

use crate::stage4_paths::assert_absent;
use scenario_support::make_cfg_in;

fn stage4_split_test_lock() -> &'static std::sync::Mutex<()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
}

fn tx_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/tx_alice_to_bob_pkg.json")
}

fn after_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/wallets_state_after.json")
}

fn diff_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/wallets_state_diff.json")
}

fn pending_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/wallets_pending.json")
}

fn confirm_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/wallets_confirmed.json")
}

fn load_tx_package(out: &Path) -> TxPackage {
    z00z_utils::codec::JsonCodec
        .deserialize(&read_file(tx_file(out)).expect("read tx"))
        .expect("decode tx")
}

fn assert_no_post_state(out: &Path) {
    assert_absent(&tx_file(out));
    assert_absent(&after_file(out));
    assert_absent(&diff_file(out));
    assert_absent(&pending_file(out));
    assert_absent(&confirm_file(out));
}

fn assert_split_file(rel_path: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel_path);
    assert!(path.exists(), "missing split seam {}", path.display());
}

struct OutCase {
    out: PathBuf,
}

struct FailCase {
    out: PathBuf,
    msg: String,
}

fn partial_case() -> &'static OutCase {
    static CASE: std::sync::OnceLock<OutCase> = std::sync::OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage4_split_partial_v1", |base| {
            let (cfg_path, design_path, out) = make_cfg_in(base, |cfg| {
                let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
                stage4
                    .transaction
                    .input_assets_selection
                    .distinct_serial_ids_min = 3;
                stage4
                    .transaction
                    .input_assets_selection
                    .distinct_serial_ids_target = 3;
                stage4
                    .transaction
                    .input_assets_selection
                    .distinct_serial_ids_max = 3;
                stage4.transaction.mode = "fraction".to_string();
                stage4.transaction.amount = None;
                stage4.transaction.fraction = Some(0.5);
                stage4.transaction.outputs.bob_outputs_count = 3;
            });
            let _ctx = stage_runner_support::run_stage4_setup(&cfg_path, &design_path);
            assert!(tx_file(&out).exists(), "partial split tx missing");
        });
        OutCase {
            out: root.join("outputs/scenario_1"),
        }
    })
}

fn load_fail_case(root: &Path) -> FailCase {
    FailCase {
        out: root.join("outputs/scenario_1"),
        msg: read_to_string(root.join("fail_msg.txt")).expect("read split fail msg"),
    }
}

fn amount_fee_case() -> &'static FailCase {
    static CASE: std::sync::OnceLock<FailCase> = std::sync::OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage4_split_amount_fee_v1", |base| {
            let (cfg_bad, design_bad, out_bad) = make_cfg_in(base, |cfg| {
                let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
                stage4.transaction.mode = "amount".to_string();
                stage4.transaction.fraction = None;
                stage4.transaction.amount = Some(u64::MAX / 2);
            });

            let mut ctx = stage_runner_support::run_stage5_session(&cfg_bad, &design_bad);
            let stage = stage_runner_support::stage_by_id(&design_bad, 6);
            let msg = match stage_6::run_tx_prepare(&mut ctx, &stage) {
                StageResult::Fail(msg) => msg,
                other => panic!("tx_prepare stage must fail, got {other:?}"),
            };

            write_file(base.join("fail_msg.txt"), msg.as_bytes()).expect("write split fail msg");
            assert_no_post_state(&out_bad);
        });
        load_fail_case(&root)
    })
}

fn zero_send_case() -> &'static FailCase {
    static CASE: std::sync::OnceLock<FailCase> = std::sync::OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage4_split_zero_send_v1", |base| {
            let (cfg_bad, design_bad, out_bad) = make_cfg_in(base, |cfg| {
                let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
                stage4.transaction.mode = "fraction".to_string();
                stage4.transaction.amount = None;
                stage4.transaction.fraction = Some(0.000_000_1);
            });

            let mut ctx = stage_runner_support::run_stage5_session(&cfg_bad, &design_bad);
            let stage = stage_runner_support::stage_by_id(&design_bad, 6);
            let msg = match stage_6::run_tx_prepare(&mut ctx, &stage) {
                StageResult::Fail(msg) => msg,
                other => panic!("tx_prepare stage must fail, got {other:?}"),
            };

            write_file(base.join("fail_msg.txt"), msg.as_bytes()).expect("write split fail msg");
            assert_no_post_state(&out_bad);
        });
        load_fail_case(&root)
    })
}

#[test]
fn test_stage4_split_files_exist() {
    let _guard = stage4_split_test_lock()
        .lock()
        .expect("stage4 split test lock");
    for rel_path in [
        "src/scenario_1/stage_6/input_selection_scan.rs",
        "src/scenario_1/stage_6/output_construction.rs",
        "src/scenario_1/stage_6/reporting.rs",
        "src/scenario_1/stage_6/tx_preparation_core.rs",
        "src/scenario_1/stage_6/tx_validation_gates.rs",
        "src/scenario_1/stage_6/wallet_state_capture.rs",
        "src/scenario_1/stage_9/prep_snapshot_loader.rs",
        "src/scenario_1/stage_9/fragment_construction.rs",
        "src/scenario_1/stage_9/exec_input_builder.rs",
        "src/scenario_1/stage_9/bridge_output_router.rs",
        "src/scenario_1/stage_9/demo_checkpoint_agg.rs",
        "src/scenario_1/stage_9/logging.rs",
    ] {
        assert_split_file(rel_path);
    }
}

#[test]
fn test_stage4_amount_fee() {
    let _guard = stage4_split_test_lock()
        .lock()
        .expect("stage4 split test lock");
    let case = amount_fee_case();
    assert!(
        case.msg.contains("transaction.amount=") && case.msg.contains("exceeds input_amount="),
        "unexpected error: {}",
        case.msg
    );
    assert_no_post_state(&case.out);
}

#[test]
fn test_stage4_zero_send() {
    let _guard = stage4_split_test_lock()
        .lock()
        .expect("stage4 split test lock");
    let case = zero_send_case();
    assert!(
        case.msg.contains("transaction produced zero send amount"),
        "unexpected error: {}",
        case.msg
    );
    assert_no_post_state(&case.out);
}

#[test]
fn test_stage4_partial_has_change() {
    let _guard = stage4_split_test_lock()
        .lock()
        .expect("stage4 split test lock");
    let pkg = load_tx_package(&partial_case().out);
    let has_change = pkg
        .tx
        .outputs
        .iter()
        .any(|out| matches!(out.role, TxOutRole::Change));

    assert!(has_change, "partial spend must emit Change output");
}
