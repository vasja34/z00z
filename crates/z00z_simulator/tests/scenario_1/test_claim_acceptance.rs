use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use z00z_utils::codec::{Codec, Value, YamlCodec};
use z00z_utils::io::{load_json, read_to_string, write_file};
use z00z_wallets::claim::registry as claim_registry;

use z00z_simulator::{
    scenario_1::{support::claim_shared_cases, support::stage_runner_support},
    ScenarioCfg, StageResult,
};

fn load_source(rel_path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join(rel_path);
    fs::read_to_string(&src).unwrap_or_else(|e| panic!("read {} failed: {e}", src.display()))
}

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct OutCase {
    out: PathBuf,
}

fn det_case(case_name: &str) -> &'static OutCase {
    static CASE_A: OnceLock<OutCase> = OnceLock::new();
    static CASE_B: OnceLock<OutCase> = OnceLock::new();
    let cell = match case_name {
        "a" => &CASE_A,
        "b" => &CASE_B,
        other => panic!("unexpected deterministic case: {other}"),
    };
    cell.get_or_init(|| OutCase {
        out: match case_name {
            "a" => claim_shared_cases::deterministic_stage3_a_out(),
            "b" => claim_shared_cases::deterministic_stage3_b_out(),
            _ => unreachable!(),
        },
    })
}

fn publish_paths_case() -> &'static OutCase {
    static CASE: OnceLock<OutCase> = OnceLock::new();
    CASE.get_or_init(|| OutCase {
        out: claim_shared_cases::publish_paths_stage4_out(),
    })
}

#[test]
fn test_shared_cfg_bootstrap() {
    let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
    claim_registry::clear_rows();

    let base = tempfile::tempdir().expect("tempdir must create");
    let out = base.path().join("outputs/scenario_1");
    let mut cfg = ScenarioCfg::from_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_config.yaml"),
    )
    .expect("load scenario config");
    cfg.stage1_genesis
        .get_or_insert_with(Default::default)
        .genesis_config = z00z_core::config_paths::devnet_genesis_path()
        .to_string_lossy()
        .to_string();
    cfg.outputs.dir = out.to_string_lossy().to_string();
    cfg.stage3_claim.as_mut().expect("stage3 cfg").consume_bins = Some(false);

    let cfg_path = base.path().join("scenario_config.yaml");
    let cfg_bytes = YamlCodec.serialize(&cfg).expect("serialize cfg");
    write_file(&cfg_path, &cfg_bytes).expect("write cfg");
    let design_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_design.yaml");

    let run = stage_runner_support::run_stage_plan_subset(&cfg_path, &design_path, &[1_u32, 2]);
    assert!(
        matches!(stage_runner_support::stage_res(&run, 1), StageResult::Ok),
        "stage 1 failed in claim shared cfg bootstrap: {:?}",
        stage_runner_support::stage_res(&run, 1)
    );
    if !matches!(stage_runner_support::stage_res(&run, 2), StageResult::Ok) {
        let stage_log = read_to_string(out.join("logs").join("logger.json"))
            .unwrap_or_else(|_| "<missing logger.json>".to_string());
        let rpc_log = read_to_string(out.join("logs").join("rpc_logger.json"))
            .unwrap_or_else(|_| "<missing rpc_logger.json>".to_string());
        panic!(
            "stage 2 failed in claim shared cfg bootstrap: {:?}\nlogger.json:\n{}\nrpc_logger.json:\n{}",
             stage_runner_support::stage_res(&run, 2),
            stage_log,
            rpc_log
        );
    }
}

#[test]
fn test_claim_service_single_entrypoint() {
    let src = load_source("src/scenario_1/stage_3/mod.rs");
    let publish_src = load_source("src/scenario_1/stage_4/mod.rs");
    let publish_impl_src = load_source("src/scenario_1/stage_4/publish.rs");
    let publish_view_src = load_source("src/scenario_1/stage_4/storage_view.rs");

    // Stage-3 orchestration entrypoint must stay explicit.
    assert!(
        src.contains("pub fn run_claim_genesis("),
        "stage_3 must expose run_claim_genesis entrypoint"
    );
    assert!(
        src.contains("fn run_core("),
        "stage_3 must keep a single internal orchestration core"
    );
    assert!(
        !src.contains("pub fn run_claim_publish("),
        "stage_3 must not keep run_claim_publish after the split"
    );
    assert!(
        publish_src.contains("pub use publish::run_claim_publish;")
            || publish_src.contains("pub fn run_claim_publish("),
        "stage_4 must expose run_claim_publish as the canonical entrypoint export"
    );
    assert!(
        !src.contains("publish_claims_store("),
        "stage_3 must not keep claim publish store ownership after the split"
    );
    assert!(
        publish_impl_src.contains("publish_claims_store("),
        "stage_4 must own claim store publication"
    );
    assert!(
        publish_view_src.contains("export_claim_post_view("),
        "stage_4 must export the post-claim store view"
    );

    // Prior simulator wrappers must not come back after migration.
    let banned = [
        "fn read_claim_state(",
        "fn write_claim_state(",
        "fn merge_claim_state(",
        "fn has_claim_row(",
        "fn add_claim_row(",
        "fn rehydrate_claim_rows(",
    ];
    for item in banned {
        assert!(
            !src.contains(item),
            "prior stage_3 wrapper reintroduced: {item}"
        );
    }

    // Stage-3 must route claim primitives through the canonical wallet claim API.
    assert!(
        src.contains("z00z_wallets::claim"),
        "stage_3 must depend on the canonical wallet claim module"
    );

    assert!(
        src.contains("build_card_output_serial_checked("),
        "stage_3 must route approved sender outputs through the validated serial builder"
    );
    assert!(
        !src.contains("build_tx_stealth_output_serial("),
        "stage_3 must not fall back to the raw serial builder"
    );
}

#[test]
fn test_tx_prepare_core_only() {
    let src = load_source("src/scenario_1/stage_6/tx_lane_impl.rs");
    let flow_src = load_source("src/scenario_1/stage_6/tx_lane_runtime_flow.rs");
    let validation_src = load_source("src/scenario_1/stage_6/tx_validation_gates.rs");
    let reports_src = load_source("src/scenario_1/stage_6/reports_rows.rs");

    // The tx lane implementation must route through the narrowed core tx facade.
    let must_use_tx_lane = [
        "core_build_output_bundle(",
        "core_verify_self_decrypt(",
        "core_plain_balance(",
    ];
    for item in must_use_tx_lane {
        assert!(
            src.contains(item),
            "tx lane impl must route through core tx API: missing {item}"
        );
    }

    assert!(
        flow_src.contains("build_tx_package_digest("),
        "tx prepare runtime flow must use build_tx_package_digest as the public tx root"
    );
    assert!(
        !src.contains("core_tx_digest("),
        "tx lane impl must not route through the removed core_tx_digest helper"
    );

    assert!(
        validation_src.contains("z00z_wallets::tx::verify_commitment_balance_gate("),
        "tx validation gates must route through core tx API: missing z00z_wallets::tx::verify_commitment_balance_gate("
    );
    assert!(
        validation_src.contains("z00z_wallets::tx::verify_spend_witness_gate("),
        "tx validation gates must route through core tx API: missing z00z_wallets::tx::verify_spend_witness_gate("
    );

    let must_use_reports = [
        "core_build_pending(",
        "core_build_confirm(",
        "core_validate_confirm(",
    ];
    for item in must_use_reports {
        assert!(
            reports_src.contains(item),
            "tx lane reports must route through core tx API: missing {item}"
        );
    }

    // Old local crypto pipeline implementations must not reappear.
    let banned = [
        "fn create_output_bundle(",
        "fn verify_self_decrypt(",
        "fn verify_plaintext_balance_with_fee(",
        "fn derive_fee_commitment(",
    ];
    for item in banned {
        assert!(
            !src.contains(item),
            "local tx lane implementation reintroduced: {item}"
        );
    }
}

#[test]
fn test_deterministic_outputs_same_seed() {
    let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
    claim_registry::clear_rows();
    claim_registry::clear_rows();
    let snap_a: Value =
        load_json(det_case("a").out.join("stage_3_snapshot.json")).expect("snapshot a");
    let snap_b: Value =
        load_json(det_case("b").out.join("stage_3_snapshot.json")).expect("snapshot b");
    assert_eq!(snap_a, snap_b, "stage-3 snapshot must be deterministic");
}

#[test]
fn test_claim_publish_stage3_paths() {
    let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
    claim_registry::clear_rows();
    let out = &publish_paths_case().out;
    assert!(
        out.join("stage_3_custom_snapshot.json").exists(),
        "stage 4 publish must read the configured stage 3 snapshot file"
    );
    assert!(
        out.join("claim_custom").join("tx_claim_pkg.json").exists(),
        "stage 4 publish must consume the configured stage 3 claim package directory"
    );
    assert!(
        out.join("claim_custom")
            .join("claim_store_pub.json")
            .exists(),
        "stage 4 publish must write the claim publish summary beside the configured claim package"
    );

    let publish_audit: Value = load_json(out.join("claim_publish").join("audit_log.json"))
        .expect("load claim publish audit");
    assert_eq!(
        publish_audit
            .get("source_snapshot_file")
            .and_then(|value| value.as_str()),
        Some("stage_3_custom_snapshot.json")
    );
}

#[test]
fn test_simulator_tests_orchestration_only() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/scenario_1");
    let entries = fs::read_dir(&root).expect("read simulator tests dir");

    let mut found = 0usize;
    for item in entries {
        let path = item.expect("entry").path();
        if path.extension().and_then(|x| x.to_str()) != Some("rs") {
            continue;
        }

        found = found.saturating_add(1);
        let name = path
            .file_name()
            .and_then(|x| x.to_str())
            .expect("test file name");
        let migrated_core = [
            "test_tx_assetpack.rs",
            "test_tx_balance.rs",
            "test_tx_drift.rs",
            "test_tx_fee.rs",
            "test_tx_interop.rs",
            "test_tx_parity.rs",
            "test_tx_pass.rs",
            "test_tx_pedersen.rs",
            "test_tx_poison.rs",
            "test_tx_prefilter.rs",
            "test_tx_roundtrip.rs",
            "test_tx_serial.rs",
            "test_tx_spent_gate.rs",
            "test_tx_stealth_flow.rs",
            "test_tx_tamper.rs",
            "test_tx_wrong_root.rs",
        ];
        assert!(
            !migrated_core.contains(&name),
            "migrated core-domain tx test leaked into simulator crate: {name}"
        );

        let _src = fs::read_to_string(&path).expect("read test source");
    }

    assert!(found > 0, "simulator tests directory must not be empty");
}
