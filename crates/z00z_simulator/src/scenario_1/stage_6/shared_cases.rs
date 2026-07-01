use std::{path::PathBuf, sync::OnceLock};

use crate::config::ScenarioCfg;
use crate::scenario_1::support::{fixture_cache, scenario_support, stage_runner_support};

fn tx_file(out: &std::path::Path) -> PathBuf {
    out.join("transactions/tx_alice_to_bob_pkg.json")
}

fn build_stage6_out(case_name: &str, edit_cfg: impl FnOnce(&mut ScenarioCfg)) -> PathBuf {
    let root = fixture_cache::ensure_shared_case_precise(case_name, |base| {
        let (cfg_path, design_path, out) = scenario_support::make_cfg_in(base, edit_cfg);
        let _ctx = stage_runner_support::run_stage4_setup(&cfg_path, &design_path);
        assert!(tx_file(&out).exists(), "shared stage6 tx package missing");
    });
    root.join("outputs/scenario_1")
}

pub fn default_stage6_out() -> PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| build_stage6_out("stage6_default_shared_v2", |_| {}))
        .clone()
}

pub fn default_stage6_rerun_out() -> PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| build_stage6_out("stage6_default_rerun_shared_v2", |_| {}))
        .clone()
}

pub fn fraction_02_stage6_out() -> PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| {
        build_stage6_out("stage6_fraction_02_shared_v2", |cfg| {
            let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
            stage4.transaction.fraction = Some(0.2);
            stage4.transaction.amount = None;
        })
    })
    .clone()
}

pub fn e2e18_stage6_out() -> PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| {
        build_stage6_out("stage6_e2e18_shared_v2", |cfg| {
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
        })
    })
    .clone()
}
