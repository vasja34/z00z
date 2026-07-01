use std::path::PathBuf;

use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::scenario_support;
use z00z_simulator::scenario_1::support::stage_runner_support;
use z00z_simulator::{scenario_1::stage_5, StageResult};

use crate::stage4_paths::assert_absent;
use scenario_support::make_cfg_in;
use z00z_utils::io::{path_exists, read_to_string, write_file};

struct OutCase {
    out: PathBuf,
}

struct FailCase {
    out: PathBuf,
    msg: String,
}

fn paths_work_case() -> &'static OutCase {
    static CASE: std::sync::OnceLock<OutCase> = std::sync::OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage4_cfg_paths_work_v1", |base| {
            let (cfg_path, design_path, out) = make_cfg_in(base, |cfg| {
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
                stage4.paths.logs_dir =
                    "crates/z00z_simulator/outputs/scenario_1/stage4_logs".to_string();
                stage4.paths.transactions_dir =
                    "crates/z00z_simulator/outputs/scenario_1/stage4_tx".to_string();
                stage4.paths.tx_pkg_file =
                    "crates/z00z_simulator/outputs/scenario_1/stage4_tx/pkg.json".to_string();
                stage4.paths.snapshot_file =
                    "crates/z00z_simulator/outputs/scenario_1/stage4_snap.json".to_string();
                stage4.paths.logger_file =
                    "crates/z00z_simulator/outputs/scenario_1/stage4_logs/stage4_logger.json"
                        .to_string();
                stage4.paths.rpc_logger_file =
                    "crates/z00z_simulator/outputs/scenario_1/stage4_logs/stage4_rpc.json"
                        .to_string();
                stage4.paths.wallets_state_before_file = Some("before.json".to_string());
                stage4.paths.wallets_state_after_file = Some("after.json".to_string());
                stage4.paths.wallets_state_diff_file = Some("diff.json".to_string());
                stage4.paths.wallets_state_report_md_file = Some("report.md".to_string());
                stage4.paths.wallets_state_report_xlsx_file = Some("report.xlsx".to_string());
            });
            let _ctx = stage_runner_support::run_stage4_setup(&cfg_path, &design_path);
            assert!(path_exists(out.join("stage4_tx/pkg.json")).expect("pkg exists"));
        });
        OutCase {
            out: root.join("outputs/scenario_1"),
        }
    })
}

fn short_paths_case() -> &'static OutCase {
    static CASE: std::sync::OnceLock<OutCase> = std::sync::OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage4_cfg_short_paths_work_v1", |base| {
            let (cfg_path, design_path, out) = make_cfg_in(base, |cfg| {
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
                stage4.paths.logs_dir = "logs2".to_string();
                stage4.paths.transactions_dir = "tx2".to_string();
                stage4.paths.tx_pkg_file = "tx2/pkg.json".to_string();
                stage4.paths.snapshot_file = "snap2.json".to_string();
                stage4.paths.logger_file = "logs2/log.json".to_string();
                stage4.paths.rpc_logger_file = "logs2/rpc.json".to_string();
                stage4.paths.wallets_state_before_file = Some("before2.json".to_string());
                stage4.paths.wallets_state_after_file = Some("after2.json".to_string());
                stage4.paths.wallets_state_diff_file = Some("diff2.json".to_string());
                stage4.paths.wallets_state_report_md_file = Some("report2.md".to_string());
                stage4.paths.wallets_state_report_xlsx_file = Some("report2.xlsx".to_string());
            });
            let _ctx = stage_runner_support::run_stage4_setup(&cfg_path, &design_path);
            assert!(path_exists(out.join("tx2/pkg.json")).expect("pkg exists"));
        });
        OutCase {
            out: root.join("outputs/scenario_1"),
        }
    })
}

fn load_fail_case(root: &std::path::Path) -> FailCase {
    FailCase {
        out: root.join("outputs/scenario_1"),
        msg: read_to_string(root.join("fail_msg.txt")).expect("read cfg paths fail msg"),
    }
}

fn bad_mode_case() -> &'static FailCase {
    static CASE: std::sync::OnceLock<FailCase> = std::sync::OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage4_cfg_paths_bad_mode_v1", |base| {
            let (cfg_path, design_path, out) = make_cfg_in(base, |cfg| {
                let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
                stage4.transaction.mode = "bad".to_string();
            });

            let mut ctx = stage_runner_support::run_stage_setup_session(
                &cfg_path,
                &design_path,
                &[1_u32, 2, 3, 4],
            );
            let stage = stage_runner_support::stage_by_id(&design_path, 5);
            let msg = match stage_5::run_tx_plan(&mut ctx, &stage) {
                StageResult::Fail(msg) => msg,
                other => panic!("tx_plan stage must fail, got {other:?}"),
            };

            write_file(base.join("fail_msg.txt"), msg.as_bytes())
                .expect("write cfg paths fail msg");
            assert_absent(&out.join("transactions/tx_alice_to_bob_pkg.json"));
            assert!(
                path_exists(out.join("stage_4_snapshot.json")).expect("claim-publish snap exists")
            );
        });
        load_fail_case(&root)
    })
}

fn empty_import_case() -> &'static FailCase {
    static CASE: std::sync::OnceLock<FailCase> = std::sync::OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage4_cfg_paths_empty_import_v1", |base| {
            let (cfg_path, design_path, out) = make_cfg_in(base, |cfg| {
                let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
                stage4.rpc.import_asset_method.clear();
            });

            let mut ctx = stage_runner_support::run_stage_setup_session(
                &cfg_path,
                &design_path,
                &[1_u32, 2, 3, 4],
            );
            let stage = stage_runner_support::stage_by_id(&design_path, 5);
            let msg = match stage_5::run_tx_plan(&mut ctx, &stage) {
                StageResult::Fail(msg) => msg,
                other => panic!("tx_plan stage must fail, got {other:?}"),
            };

            write_file(base.join("fail_msg.txt"), msg.as_bytes())
                .expect("write cfg paths fail msg");
            assert_absent(&out.join("transactions/tx_alice_to_bob_pkg.json"));
            assert!(
                path_exists(out.join("stage_4_snapshot.json")).expect("claim-publish snap exists")
            );
        });
        load_fail_case(&root)
    })
}

#[test]
fn test_stage4_paths_work() {
    let out = &paths_work_case().out;

    assert!(path_exists(out.join("stage4_tx/pkg.json")).expect("pkg exists"));
    assert!(path_exists(out.join("stage4_snap.json")).expect("snap exists"));
    assert!(path_exists(out.join("stage4_logs/stage4_logger.json")).expect("log exists"));
    assert!(path_exists(out.join("stage4_logs/stage4_rpc.json")).expect("rpc exists"));
    assert!(path_exists(out.join("stage4_tx/before.json")).expect("before exists"));
    assert!(path_exists(out.join("stage4_tx/after.json")).expect("after exists"));
    assert!(path_exists(out.join("stage4_tx/diff.json")).expect("diff exists"));
    assert!(path_exists(out.join("stage4_tx/report.md")).expect("md exists"));
    assert!(path_exists(out.join("stage4_tx/report.xlsx")).expect("xlsx exists"));

    assert_absent(&out.join("transactions/tx_alice_to_bob_pkg.json"));
    assert!(path_exists(out.join("stage_4_snapshot.json")).expect("claim-publish snap exists"));
    assert_absent(&out.join("transactions/wallets_state_before.json"));
    assert_absent(&out.join("transactions/wallets_state_after.json"));
    assert_absent(&out.join("transactions/wallets_state_diff.json"));
}

#[test]
fn test_stage4_short_paths_work() {
    let out = &short_paths_case().out;

    assert!(path_exists(out.join("tx2/pkg.json")).expect("pkg exists"));
    assert!(path_exists(out.join("snap2.json")).expect("snap exists"));
    assert!(path_exists(out.join("logs2/log.json")).expect("log exists"));
    assert!(path_exists(out.join("logs2/rpc.json")).expect("rpc exists"));
    assert!(path_exists(out.join("tx2/before2.json")).expect("before exists"));
    assert!(path_exists(out.join("tx2/after2.json")).expect("after exists"));
    assert!(path_exists(out.join("tx2/diff2.json")).expect("diff exists"));
    assert!(path_exists(out.join("tx2/report2.md")).expect("md exists"));
    assert!(path_exists(out.join("tx2/report2.xlsx")).expect("xlsx exists"));

    assert_absent(&out.join("pkg.json"));
    assert_absent(&out.join("log.json"));
    assert_absent(&out.join("rpc.json"));
}

#[test]
fn test_stage4_bad_mode_fails() {
    let case = bad_mode_case();
    assert!(
        case.msg
            .contains("transaction.mode must be 'fraction' or 'amount'"),
        "unexpected error: {}",
        case.msg
    );
    assert_absent(&case.out.join("transactions/tx_alice_to_bob_pkg.json"));
    assert!(path_exists(case.out.join("stage_4_snapshot.json")).expect("claim-publish snap exists"));
}

#[test]
fn test_stage4_empty_import_fails() {
    let case = empty_import_case();
    assert!(
        case.msg.contains("rpc.import_asset_method must be set"),
        "unexpected error: {}",
        case.msg
    );
    assert_absent(&case.out.join("transactions/tx_alice_to_bob_pkg.json"));
    assert!(path_exists(case.out.join("stage_4_snapshot.json")).expect("claim-publish snap exists"));
}
