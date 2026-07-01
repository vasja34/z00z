#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

use serde_json::Value;
use z00z_utils::io::read_file;

use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::scenario_support;
use z00z_simulator::scenario_1::support::stage_runner_support;

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

fn load_json(path: &Path) -> Value {
    serde_json::from_slice(&read_file(path).expect("read json")).expect("decode json")
}

struct OutCase {
    out: PathBuf,
}

fn ok_case() -> &'static OutCase {
    static CASE: std::sync::OnceLock<OutCase> = std::sync::OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("s7_examples_v1", |base| {
            let (cfg_path, design_path, out) = make_cfg_in(base, good_s4);
            let _ctx = stage_runner_support::run_stage_setup(
                &cfg_path,
                &design_path,
                &[1_u32, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            );
            assert!(out.join("transactions/checkpoint_bridge_s6.json").exists());
            assert!(out.join("transactions/checkpoint_s7.json").exists());
        });
        OutCase {
            out: root.join("outputs/scenario_1"),
        }
    })
}

#[test]
fn test_stage7_wires_storage_apply() {
    let out = &ok_case().out;

    let bridge = load_json(&out.join("transactions/checkpoint_bridge_s6.json"));
    let cp = load_json(&out.join("transactions/checkpoint_s7.json"));

    assert_eq!(bridge["stage"].as_u64(), Some(9));
    assert_eq!(bridge["status"].as_str(), Some("ok"));
    assert_eq!(cp["stage"].as_u64(), Some(11));
    assert_eq!(cp["status"].as_str(), Some("ok"));
    assert_eq!(cp["draft_id_hex"].as_str().map(str::len), Some(64));
    assert_eq!(cp["exec_input_id_hex"].as_str().map(str::len), Some(64));
    assert_eq!(cp["snapshot_id_hex"].as_str().map(str::len), Some(64));
    assert_eq!(cp["exec_input_id_hex"], bridge["exec_input_id_hex"]);
    assert_ne!(cp["prev_root_hex"], cp["new_root_hex"]);
}
