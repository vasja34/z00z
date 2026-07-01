use crate::output_roots;
use z00z_simulator::scenario_1::stage_6::sim_pkg_support;
use z00z_simulator::scenario_1::stage_6::verifier_support;
use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::stage_runner_support;

use std::{path::PathBuf, sync::OnceLock};

use z00z_simulator::{config::ScenarioCfg, StageResult};
use z00z_utils::{
    codec::{Codec, YamlCodec},
    io::{create_dir_all, write_file},
};
use z00z_wallets::tx::TxVerifierImpl;

use sim_pkg_support::load_pkg_bundle;
use verifier_support::{check_empty_out, check_verifier, write_ver_log};

struct RunCase {
    out: PathBuf,
    stage4: StageResult,
}

static E2E18_RUN: OnceLock<RunCase> = OnceLock::new();

fn out_dir() -> PathBuf {
    output_roots::stage4_output_root()
}

fn make_cfg_in(base: &std::path::Path) -> (PathBuf, PathBuf, PathBuf) {
    let out = base.join("outputs/scenario_1");
    let mut cfg = ScenarioCfg::from_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_config.yaml"),
    )
    .expect("load cfg");

    cfg.stage1_genesis
        .get_or_insert_with(Default::default)
        .genesis_config = z00z_core::config_paths::devnet_genesis_path()
        .to_string_lossy()
        .to_string();
    cfg.outputs.dir = out.to_string_lossy().to_string();

    if let Some(s3) = cfg.stage3_claim.as_mut() {
        s3.consume_bins = Some(false);
    }

    if let Some(s4) = cfg.stage4_tx_prepare.as_mut() {
        s4.transaction
            .input_assets_selection
            .distinct_serial_ids_min = 4;
        s4.transaction
            .input_assets_selection
            .distinct_serial_ids_target = 4;
        s4.transaction
            .input_assets_selection
            .distinct_serial_ids_max = 10;
        s4.transaction.outputs.bob_outputs_count = 4;
        s4.transaction.class = "Coin".to_string();
        s4.transaction.symbol = "Z00Z".to_string();
        s4.transaction.mode = "fraction".to_string();
        s4.transaction.fraction = Some(0.1);
    }

    let cfg_path = base.join("scenario_config.yaml");
    let cfg_bytes = YamlCodec.serialize(&cfg).expect("cfg bytes");
    write_file(&cfg_path, &cfg_bytes).expect("write cfg");

    let design_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_design.yaml");
    (cfg_path, design_path, out)
}

fn run_case() -> &'static RunCase {
    E2E18_RUN.get_or_init(|| {
        let root = fixture_cache::ensure_case("tx_handoff_integration_v1", |base| {
            let (cfg_path, design_path, out) = make_cfg_in(base);
            let _ctx = stage_runner_support::run_stage4_setup(&cfg_path, &design_path);
            assert!(out.join("transactions/tx_alice_to_bob_pkg.json").exists());
        });
        RunCase {
            out: root.join("outputs/scenario_1"),
            stage4: StageResult::Ok,
        }
    })
}

#[test]
fn test_stage4_structure() {
    if cfg!(debug_assertions) {
        return;
    }

    let run = run_case();
    assert!(
        matches!(run.stage4, StageResult::Ok),
        "stage 4 must succeed"
    );
    let (sim_file, can_bytes, can_pkg) = load_pkg_bundle(&run.out);
    let ver = TxVerifierImpl::new();
    let can_meta = check_verifier(&ver, &can_bytes);
    let (wallet_bad, bad_meta) = check_empty_out(&ver, &can_pkg);
    create_dir_all(out_dir()).expect("mkdir outputs/e2e18");
    write_ver_log(
        &out_dir(),
        &sim_file,
        &can_bytes,
        &wallet_bad,
        &can_meta,
        &bad_meta,
    );
}
