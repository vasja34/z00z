use std::path::{Path, PathBuf};

use z00z_simulator::{
    scenario_1::{stage_5, stage_6},
    StageResult,
};
use z00z_utils::{
    codec::{Codec, YamlCodec},
    io::write_file,
};

use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::scenario_support;
use z00z_simulator::scenario_1::support::stage_runner_support;

use crate::stage4_paths::assert_absent;
use scenario_support::make_cfg_in;

fn good_s4(cfg: &mut z00z_simulator::config::ScenarioCfg) {
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

fn tx_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/tx_alice_to_bob_pkg.json")
}

fn snap_file(out: &Path) -> std::path::PathBuf {
    out.join("stage_4_snapshot.json")
}

fn after_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/wallets_state_after.json")
}

fn before_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/wallets_state_before.json")
}

fn diff_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/wallets_state_diff.json")
}

fn pend_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/wallets_pending.json")
}

fn conf_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/wallets_confirmed.json")
}

fn report_md(out: &Path) -> std::path::PathBuf {
    out.join("transactions/wallets_state_report.md")
}

fn report_xlsx(out: &Path) -> std::path::PathBuf {
    out.join("transactions/wallets_state_report.xlsx")
}

fn no_post(out: &Path) {
    assert_absent(&tx_file(out));
    assert_absent(&after_file(out));
    assert_absent(&diff_file(out));
    assert_absent(&pend_file(out));
    assert_absent(&conf_file(out));
}

fn no_cfg_art(out: &Path) {
    no_post(out);
    assert_absent(&before_file(out));
    assert_absent(&report_md(out));
    assert_absent(&report_xlsx(out));
}

fn no_run_art(out: &Path) {
    no_post(out);
    assert_absent(&report_md(out));
    assert_absent(&report_xlsx(out));
}

struct OutCase {
    out: PathBuf,
}

fn cached_stage5_fail(
    case_name: &str,
    edit_cfg: impl FnOnce(&mut z00z_simulator::config::ScenarioCfg),
) -> (String, std::path::PathBuf) {
    let root = fixture_cache::ensure_shared_case(case_name, |base| {
        let (cfg_path, design_path, out) = make_cfg_in(base, |cfg| {
            good_s4(cfg);
            edit_cfg(cfg);
        });
        let mut ctx = stage_runner_support::run_stage_setup_session(
            &cfg_path,
            &design_path,
            &[1_u32, 2, 3, 4],
        );
        let stage = stage_runner_support::stage_by_id(&design_path, 5);
        let msg = match stage_5::run_tx_plan(&mut ctx, &stage) {
            StageResult::Fail(msg) => msg,
            other => panic!("stage 5 guard must fail, got {other:?}"),
        };
        write_file(
            root_meta(base),
            serde_json::to_vec_pretty(&serde_json::json!({ "msg": msg }))
                .expect("encode stage5 guard meta")
                .as_slice(),
        )
        .expect("write stage5 guard meta");
        let _ = out;
    });
    let meta: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root_meta(&root)).expect("read stage5 guard meta"))
            .expect("decode stage5 guard meta");
    (
        meta["msg"].as_str().expect("stage5 guard msg").to_string(),
        root.join("outputs/scenario_1"),
    )
}

fn make_repo_cfg_in(base: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let out = base.join("outputs/scenario_1");
    let mut cfg = z00z_simulator::config::ScenarioCfg::from_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_config.yaml"),
    )
    .expect("load scenario config");
    cfg.stage1_genesis
        .get_or_insert_with(Default::default)
        .genesis_config = z00z_core::config_paths::devnet_genesis_path()
        .to_string_lossy()
        .to_string();
    cfg.outputs.dir = out.to_string_lossy().to_string();

    if let Some(stage3) = cfg.stage3_claim.as_mut() {
        stage3.consume_bins = Some(false);
    }

    let cfg_path = base.join("scenario_config.yaml");
    let cfg_bytes = YamlCodec.serialize(&cfg).expect("cfg bytes");
    write_file(&cfg_path, &cfg_bytes).expect("write cfg");

    let design_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_design.yaml");
    (cfg_path, design_path, out)
}

fn root_meta(root: &Path) -> PathBuf {
    root.join("guard_meta.json")
}

fn cached_stage6_fail(
    case_name: &str,
    edit_cfg: impl FnOnce(&mut z00z_simulator::config::ScenarioCfg),
) -> (String, std::path::PathBuf) {
    let root = fixture_cache::ensure_shared_case(case_name, |base| {
        let (cfg_path, design_path, out) = make_cfg_in(base, |cfg| {
            good_s4(cfg);
            edit_cfg(cfg);
        });
        let mut ctx = stage_runner_support::run_stage5_session(&cfg_path, &design_path);
        let stage = stage_runner_support::stage_by_id(&design_path, 6);
        let msg = match stage_6::run_tx_prepare(&mut ctx, &stage) {
            StageResult::Fail(msg) => msg,
            other => panic!("stage 6 guard must fail, got {other:?}"),
        };
        write_file(
            root_meta(base),
            serde_json::to_vec_pretty(&serde_json::json!({ "msg": msg }))
                .expect("encode stage6 guard meta")
                .as_slice(),
        )
        .expect("write stage6 guard meta");
        let _ = out;
    });
    let meta: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root_meta(&root)).expect("read stage6 guard meta"))
            .expect("decode stage6 guard meta");
    (
        meta["msg"].as_str().expect("stage6 guard msg").to_string(),
        root.join("outputs/scenario_1"),
    )
}

fn valid_cfg_case() -> &'static OutCase {
    static CASE: std::sync::OnceLock<OutCase> = std::sync::OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage4_cfg_guards_valid_v1", |base| {
            let (cfg_path, design_path, out) = make_cfg_in(base, good_s4);
            let _ctx = stage_runner_support::run_stage4_setup(&cfg_path, &design_path);
            assert!(tx_file(&out).exists(), "valid cfg tx package missing");
            assert!(snap_file(&out).exists(), "valid cfg snapshot missing");
        });
        OutCase {
            out: root.join("outputs/scenario_1"),
        }
    })
}

fn repo_cfg_case() -> &'static OutCase {
    static CASE: std::sync::OnceLock<OutCase> = std::sync::OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage4_cfg_guards_repo_v1", |base| {
            let (cfg_path, design_path, out) = make_repo_cfg_in(base);
            let _ctx = stage_runner_support::run_stage4_setup(&cfg_path, &design_path);
            assert!(tx_file(&out).exists(), "repo cfg tx package missing");
            assert!(snap_file(&out).exists(), "repo cfg snapshot missing");
        });
        OutCase {
            out: root.join("outputs/scenario_1"),
        }
    })
}

#[test]
fn test_stage4_valid_cfg_ok() {
    let out = &valid_cfg_case().out;
    assert!(
        tx_file(out).exists(),
        "tx package must exist for valid config"
    );
    assert!(
        snap_file(out).exists(),
        "snapshot must exist for valid config"
    );
}

#[test]
fn test_stage4_repo_cfg_ok() {
    let out = &repo_cfg_case().out;
    assert!(
        tx_file(out).exists(),
        "tx package must exist for repo default config"
    );
    assert!(
        snap_file(out).exists(),
        "snapshot must exist for repo default config"
    );
}

#[test]
fn test_stage4_rejects_bad_mode() {
    let (msg, out) = cached_stage5_fail("stage4_cfg_guards_bad_mode_v1", |cfg| {
        let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
        stage4.transaction.mode = "bad".to_string();
        stage4.transaction.fraction = None;
        stage4.transaction.amount = None;
    });

    assert!(
        msg.contains("stage4: transaction.mode must be 'fraction' or 'amount'"),
        "unexpected error: {msg}"
    );
    no_cfg_art(&out);
}

#[test]
fn test_stage4_rejects_big_amount() {
    let (msg, out) = cached_stage6_fail("stage4_cfg_guards_big_amount_v1", |cfg| {
        let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
        stage4.transaction.mode = "amount".to_string();
        stage4.transaction.amount = Some(u64::MAX);
        stage4.transaction.fraction = None;
    });

    assert!(
        msg.contains("stage4: transaction.amount=") && msg.contains("exceeds input_amount="),
        "unexpected error: {msg}"
    );
    no_run_art(&out);
}

#[test]
fn test_stage4_rejects_no_pass() {
    let (msg, out) = cached_stage5_fail("stage4_cfg_guards_no_pass_v1", |cfg| {
        let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
        stage4.transaction.fee_sink.receiver_card_hex = None;
        stage4.transaction.fee_sink.password = None;
    });

    assert!(
        msg.contains("stage4: external fee sink requires transaction.fee_sink.password"),
        "unexpected error: {msg}"
    );
    no_cfg_art(&out);
}
