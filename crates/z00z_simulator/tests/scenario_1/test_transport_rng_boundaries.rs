use std::sync::{Mutex, OnceLock};

use z00z_simulator::config::ScenarioCfg;
use z00z_utils::io::load_json;

use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::scenario_support;
use z00z_simulator::scenario_1::support::stage_runner_support;

const CONFIG_SRC: &str = include_str!("../../src/config.rs");
const RNG_MODE_SRC: &str = include_str!("../../src/rng_mode.rs");
const TRANSPORT_SRC: &str = include_str!("../../src/scenario_1/stage_2/transport.rs");

const ACTORS: [&str; 3] = ["alice", "bob", "charlie"];

fn transport_fixture_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn read_wallet_ids(out: &std::path::Path) -> Vec<String> {
    ACTORS
        .iter()
        .map(|name| {
            let path = out.join("keys").join(format!("{name}_keys.json"));
            let value: serde_json::Value = load_json(&path).expect("load actor keys json");
            value["wallet_id"]
                .as_str()
                .unwrap_or_else(|| panic!("wallet_id missing for {name}"))
                .to_string()
        })
        .collect()
}

fn cache_case_name(mock_rng_seed: Option<u64>) -> String {
    match mock_rng_seed {
        None => "transport_rng_boundaries_seed_none_v1".to_string(),
        Some(seed) => format!("transport_rng_boundaries_seed_{seed}_v1"),
    }
}

fn run_wallet_ids(mock_rng_seed: Option<u64>) -> Vec<String> {
    let _guard = transport_fixture_lock()
        .lock()
        .unwrap_or_else(|err| err.into_inner());
    let case_name = cache_case_name(mock_rng_seed);
    let root = fixture_cache::ensure_case(&case_name, |base| {
        let (cfg_path, design_path, out) =
            scenario_support::make_cfg_in(base, |cfg: &mut ScenarioCfg| {
                cfg.simulation.use_mock_rng = true;
                cfg.simulation.mock_rng_seed = mock_rng_seed;
            });
        let _ctx = stage_runner_support::run_stage_setup(&cfg_path, &design_path, &[1_u32, 2]);
        assert!(out.join("keys").exists());
    });
    let out = root.join("outputs/scenario_1");

    let wallet_ids = read_wallet_ids(&out);
    assert_eq!(
        wallet_ids.len(),
        ACTORS.len(),
        "expected one wallet id per actor"
    );
    wallet_ids
}

#[test]
fn test_transport_same_is_reproducible() {
    let first = run_wallet_ids(Some(42));
    let second = run_wallet_ids(Some(42));

    assert_eq!(
        first, second,
        "same mock_rng_seed must reproduce stage-2 wallet ids"
    );
}

#[test]
fn test_transport_none_zero_seed() {
    let implicit_zero = run_wallet_ids(None);
    let explicit_zero = run_wallet_ids(Some(0));

    assert_eq!(
        implicit_zero, explicit_zero,
        "mock RNG without an explicit seed must match the stage-2 zero-seed fallback"
    );
}

#[test]
fn test_transport_seeds_wallet_ids() {
    let seed_42 = run_wallet_ids(Some(42));
    let seed_43 = run_wallet_ids(Some(43));

    assert_ne!(
        seed_42, seed_43,
        "different mock_rng_seed values must not collapse to the same stage-2 wallet ids"
    );
}

#[test]
fn test_transport_contract_simulator_scoped() {
    assert!(
        CONFIG_SRC.contains("Stage-2 simulator reproducibility toggle")
            && CONFIG_SRC.contains(
                "consolidation pass over live abstractions rather than a brand-new design"
            )
            && CONFIG_SRC.contains("deterministic zero-seed fallback"),
        "scenario config must keep mock_rng_seed framed as a stage-2 simulator lane"
    );
    assert!(
        RNG_MODE_SRC.contains("bounded to CI and simulator")
            && RNG_MODE_SRC.contains("does not claim one unified randomness selector")
            && RNG_MODE_SRC.contains("not a brand-new design"),
        "rng mode helper must keep deterministic selection simulator-scoped and non-universal"
    );
    assert!(
        TRANSPORT_SRC.contains("SIMULATOR-ONLY: DO NOT MOVE TO CORE")
            && TRANSPORT_SRC.contains("zero-seed fallback")
            && TRANSPORT_SRC.contains("not a request for production randomness"),
        "stage-2 transport must keep the seeded mock lane simulator-only and non-production"
    );
}
