use std::sync::{Mutex, OnceLock};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use serde_json::Value;
use z00z_simulator::StageResult;
use z00z_simulator::{
    config::ScenarioCfg,
    scenario_1::claim_pkg_consumer::{load_claim_packages, ClaimStorePublishSummary},
    scenario_1::stage_3::Stage3Snapshot,
};
use z00z_utils::io::save_json;
use z00z_utils::{
    codec::{Codec, YamlCodec},
    io::{load_json, write_file},
};
use z00z_wallets::claim::registry as claim_registry;

use z00z_simulator::scenario_1::{
    support::claim_shared_cases, support::fixture_cache, support::stage_runner_support,
};

#[cfg(feature = "wallet_debug_tools")]
use serde::Deserialize;

#[cfg(feature = "wallet_debug_tools")]
use z00z_core::AssetClass;

#[cfg(feature = "wallet_debug_tools")]
use z00z_utils::io::load_json_bounded;

#[cfg(feature = "wallet_debug_tools")]
#[derive(Clone, Deserialize)]
struct ClaimRow {
    asset_id: String,
    class: String,
    amount: u64,
}

#[cfg(feature = "wallet_debug_tools")]
#[derive(Clone, Deserialize)]
struct DebugAsset {
    asset_id_hex: String,
    definition: DebugDef,
    amount: u64,
}

#[cfg(feature = "wallet_debug_tools")]
#[derive(Clone, Deserialize)]
struct DebugDef {
    class: AssetClass,
}

#[cfg(feature = "wallet_debug_tools")]
type AssetTriple = (String, String, u64);

use stage_runner_support::run_stage_plan_subset;

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct TestEnvVarGuard {
    key: &'static str,
    previous: Option<OsString>,
}

impl Drop for TestEnvVarGuard {
    fn drop(&mut self) {
        if let Some(value) = &self.previous {
            std::env::set_var(self.key, value);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

fn set_test_env_var(key: &'static str, value: &str) -> TestEnvVarGuard {
    let previous = std::env::var_os(key);
    std::env::set_var(key, value);
    TestEnvVarGuard { key, previous }
}

struct ClaimPersistCase {
    out: PathBuf,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct FailRunState {
    is_aborted: bool,
    stage3_ok: bool,
    stage4_ok: Option<bool>,
}

struct FailCase {
    out: PathBuf,
    state: FailRunState,
}

fn persist_case() -> &'static ClaimPersistCase {
    static CASE: OnceLock<ClaimPersistCase> = OnceLock::new();
    CASE.get_or_init(|| ClaimPersistCase {
        out: claim_shared_cases::stage6_out("claim_persist"),
    })
}

fn mk_cfg_in(base: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let out = base.join("outputs/scenario_1");

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

    if let Some(stage3) = cfg.stage3_claim.as_mut() {
        stage3.consume_bins = Some(false);
    }

    let cfg_path = base.join("scenario_config.yaml");
    let cfg_bytes = YamlCodec.serialize(&cfg).expect("serialize cfg");
    write_file(&cfg_path, &cfg_bytes).expect("write cfg");

    let design_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_design.yaml");
    (cfg_path, design_path, out)
}

fn fail_claim_pub_case() -> &'static FailCase {
    static CASE: OnceLock<FailCase> = OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("claim_persist_fail_claim_pub_v1", |base| {
            claim_registry::clear_rows();
            let (cfg_path, design_path, out) = mk_cfg_in(base);
            let _env_guard = set_test_env_var("Z00Z_FAIL_CLAIM_PUB", "1");
            let res = run_stage_plan_subset(&cfg_path, &design_path, &[1_u32, 2, 3, 4]);
            let st3 = res
                .stages
                .iter()
                .find(|s| s.stage == 3)
                .expect("stage 3 exists");
            let st4 = res
                .stages
                .iter()
                .find(|s| s.stage == 4)
                .expect("stage 4 exists");
            let state = FailRunState {
                is_aborted: res.is_aborted,
                stage3_ok: matches!(st3.result, StageResult::Ok),
                stage4_ok: Some(matches!(st4.result, StageResult::Ok)),
            };
            save_json(base.join("fail_state.json"), &state).expect("save fail claim pub state");
            assert!(
                !out.join("claim").join("claim_store_pub.json").exists(),
                "claim publish summary must be absent on publish failure"
            );
            assert!(
                out.join("stage_3_snapshot.json").exists(),
                "stage 3 snapshot must exist because prepare completed before publish failed"
            );
        });
        FailCase {
            out: root.join("outputs/scenario_1"),
            state: load_json(root.join("fail_state.json")).expect("load fail claim pub state"),
        }
    })
}

fn fail_no_persist_case() -> &'static FailCase {
    static CASE: OnceLock<FailCase> = OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("claim_persist_fail_no_persist_v1", |base| {
            claim_registry::clear_rows();
            let (cfg_path, design_path, _out) = mk_cfg_in(base);
            let _env_guard = set_test_env_var("Z00Z_FAIL_ASSET_SAVE", "1");
            let res = run_stage_plan_subset(&cfg_path, &design_path, &[1_u32, 2, 3]);
            let st3 = res
                .stages
                .iter()
                .find(|s| s.stage == 3)
                .expect("stage 3 exists");
            let state = FailRunState {
                is_aborted: res.is_aborted,
                stage3_ok: matches!(st3.result, StageResult::Ok),
                stage4_ok: None,
            };
            save_json(base.join("fail_state.json"), &state).expect("save fail no persist state");
        });
        FailCase {
            out: root.join("outputs/scenario_1"),
            state: load_json(root.join("fail_state.json")).expect("load fail no persist state"),
        }
    })
}

fn assert_no_sidecar(out: &Path) {
    let side_db = out.join("wallets").join("wallet_assets.db");
    let side_dir = out.join("wallets").join("wallet_assets.wallets");
    assert!(!side_db.exists(), "sidecar db must be absent");
    assert!(!side_dir.exists(), "sidecar wallet dir must be absent");
}

fn assert_path_exists(path: &Path, message: &str) {
    assert!(path.exists(), "{message}");
}

fn assert_claim_actor_artifacts(out: &Path) {
    for name in ["alice", "bob", "charlie"] {
        let path = out.join("claim").join(format!("claim_rows_{name}.json"));
        assert_path_exists(&path, &format!("claim rows file must exist for {name}"));
    }

    #[cfg(feature = "wallet_debug_tools")]
    for name in ["alice", "bob", "charlie"] {
        let path = out
            .join("claim")
            .join(format!("export_wallet_debug_{name}.json"));
        assert_path_exists(&path, &format!("debug export file must exist for {name}"));
    }
}

fn assert_stage_output_files(out: &Path) {
    let required = [
        (
            out.join("claim").join("claim_source_store.redb"),
            "claim source store must exist",
        ),
        (
            out.join("stage_4_snapshot.json"),
            "stage 4 snapshot must exist",
        ),
        (
            out.join("claim").join("claim_store_pub.json"),
            "claim publish summary must exist",
        ),
        (
            out.join("claim").join("audit_log.json"),
            "audit log must exist",
        ),
        (
            out.join("events").join("claim_genesis.event.json"),
            "claim event file must exist",
        ),
        (
            out.join("claim_publish").join("audit_log.json"),
            "stage 4 audit log must exist",
        ),
        (
            out.join("logs").join("rpc_logger.json"),
            "rpc logger file must exist",
        ),
        (
            out.join("logs_publish").join("claim_publish_logger.json"),
            "stage 4 logger file must exist",
        ),
        (out.join("wallets").join("wlt_map.md"), "wlt_map must exist"),
        (
            out.join("wallets_export_import")
                .join("export_wallet_encrypted_payload.json"),
            "wallet encrypted export must exist",
        ),
        (
            out.join("wallets_export_import")
                .join("export_wallet_encrypted_payload_post_claim.json"),
            "post-claim wallet encrypted export must exist",
        ),
    ];

    for (path, message) in required {
        assert_path_exists(&path, message);
    }
}

fn assert_required_artifacts(out: &Path) {
    let snap = out.join("stage_3_snapshot.json");
    assert_path_exists(&snap, "stage 3 snapshot must exist");
    assert_claim_actor_artifacts(out);
    assert_stage_output_files(out);
}

fn assert_claim_pub(out: &Path, snap: &Stage3Snapshot) {
    let claim_pub: ClaimStorePublishSummary =
        load_json(out.join("claim").join("claim_store_pub.json"))
            .expect("load claim publish summary");
    let packages = load_claim_packages(&out.join("claim").join("tx_claim_pkg.json"))
        .expect("load claim packages");
    let leaf_count: usize = packages.iter().map(|pkg| pkg.tx.outputs.len()).sum();

    assert_eq!(claim_pub.package_count, packages.len());
    assert_eq!(claim_pub.leaf_count, leaf_count);
    assert_eq!(claim_pub.inserted_count, leaf_count);
    assert!(leaf_count <= snap.distributed_assets_count);
}

fn assert_stage4_snapshot(out: &Path) {
    let publish_audit: Value = load_json(out.join("claim_publish").join("audit_log.json"))
        .expect("load claim publish audit");
    assert_eq!(publish_audit.get("stage").and_then(|v| v.as_u64()), Some(4));
    assert_eq!(
        publish_audit
            .get("source_snapshot_file")
            .and_then(|v| v.as_str()),
        Some("stage_3_snapshot.json")
    );
    assert_eq!(
        publish_audit.get("status").and_then(|v| v.as_str()),
        Some("ok")
    );

    let snap: Value = load_json(out.join("stage_4_snapshot.json")).expect("load stage4 snapshot");
    assert_eq!(snap.get("stage").and_then(|v| v.as_u64()), Some(6));
    assert_eq!(snap.get("status").and_then(|v| v.as_str()), Some("ok"));
    assert!(
        snap.get("tx_count")
            .and_then(|v| v.as_u64())
            .expect("stage4 tx_count")
            > 0,
        "stage 4 snapshot must record prepared tx package count"
    );
    assert!(
        snap.get("output_count")
            .and_then(|v| v.as_u64())
            .expect("stage4 output_count")
            > 0,
        "stage 4 snapshot must record prepared tx outputs"
    );
}

fn assert_claim_pkg_bundle_shape(out: &Path) {
    let bundle: Value =
        load_json(out.join("claim").join("tx_claim_pkg.json")).expect("load claim package bundle");
    let root = bundle.as_object().expect("claim package bundle object");

    assert_eq!(
        root.get("kind").and_then(Value::as_str),
        Some("TxPackageBundle")
    );
    assert_eq!(
        root.get("package_type").and_then(Value::as_str),
        Some("claim_tx")
    );
    assert_eq!(root.get("version").and_then(Value::as_u64), Some(1));
    assert!(
        root.get("packages")
            .and_then(Value::as_array)
            .is_some_and(|packages| !packages.is_empty()),
        "claim package bundle must persist explicit packages"
    );
}

#[cfg(feature = "wallet_debug_tools")]
fn assert_debug_dump_meta(name: &str, dump: &serde_json::Value) {
    let fields = dump.as_object().map(|m| m.len()).unwrap_or_default();
    assert!(fields > 2, "debug dump must keep wallet fields for {name}");
    assert!(
        dump.get("wallet_id").is_some(),
        "wallet_id must exist in debug dump for {name}"
    );
    for err in dump
        .get("table_errors")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
    {
        let msg = err
            .get("error")
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        assert!(
            !msg.contains("does not exist"),
            "debug dump must not treat optional missing tables as errors for {name}: {msg}"
        );
    }
}

#[cfg(feature = "wallet_debug_tools")]
fn claim_rows_to_set(claimed: Vec<ClaimRow>) -> std::collections::HashSet<AssetTriple> {
    let mut rows = std::collections::HashSet::new();
    for row in claimed {
        rows.insert((row.asset_id, row.class, row.amount));
    }
    rows
}

#[cfg(feature = "wallet_debug_tools")]
fn debug_assets_to_set(imported: Vec<DebugAsset>) -> std::collections::HashSet<AssetTriple> {
    let mut rows = std::collections::HashSet::new();
    for row in imported {
        rows.insert((
            row.asset_id_hex,
            row.definition.class.to_string(),
            row.amount,
        ));
    }
    rows
}

#[cfg(feature = "wallet_debug_tools")]
fn assert_debug_dump_matches_actor(out: &Path, name: &str) {
    let claimed: Vec<ClaimRow> =
        load_json(out.join("claim").join(format!("claim_rows_{name}.json")))
            .expect("load claimed rows");

    let dump: serde_json::Value = load_json_bounded(
        out.join("claim")
            .join(format!("export_wallet_debug_{name}.json")),
        64 * 1024 * 1024,
    )
    .expect("read debug dump");
    assert_debug_dump_meta(name, &dump);

    let imported: Vec<DebugAsset> = serde_json::from_value(
        dump.get("imported_assets_full")
            .cloned()
            .expect("imported_assets_full"),
    )
    .expect("decode imported assets");

    let lhs = claim_rows_to_set(claimed);
    let rhs = debug_assets_to_set(imported);

    assert_eq!(lhs, rhs, "debug dump must match persisted state for {name}");
}

#[test]
fn test_claim_persist_restart() {
    let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
    let case = persist_case();
    assert_no_sidecar(&case.out);
    assert_required_artifacts(&case.out);

    let snap: Stage3Snapshot =
        load_json(case.out.join("stage_3_snapshot.json")).expect("load stage3 snapshot");
    assert_claim_pub(&case.out, &snap);
    assert_claim_pkg_bundle_shape(&case.out);
    assert_stage4_snapshot(&case.out);
    assert_eq!(snap.wallet_persist_stats.len(), 3);
    for row in snap.wallet_persist_stats {
        assert!(row.is_ok, "persist check must pass for {}", row.actor);
        assert_eq!(row.expected_count, row.listed_count);
        assert!(row.listed_count > 0);
    }
}

#[test]
#[cfg(feature = "wallet_debug_tools")]
fn test_debug_dump_state_match() {
    let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
    let case = persist_case();
    assert_no_sidecar(&case.out);
    assert_required_artifacts(&case.out);

    let snap: Stage3Snapshot =
        load_json(case.out.join("stage_3_snapshot.json")).expect("load stage3 snapshot");
    assert_claim_pub(&case.out, &snap);
    assert_claim_pkg_bundle_shape(&case.out);
    assert_stage4_snapshot(&case.out);

    for name in ["alice", "bob", "charlie"] {
        assert_debug_dump_matches_actor(&case.out, name);
    }
}

#[test]
fn test_stage3_fail_claim_pub() {
    let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
    let case = fail_claim_pub_case();
    assert!(case.state.stage3_ok);
    assert_eq!(case.state.stage4_ok, Some(false));
    assert!(
        case.state.is_aborted,
        "scenario must abort on stage 4 publish failure"
    );
    assert!(
        !case.out.join("claim").join("claim_store_pub.json").exists(),
        "claim publish summary must be absent on publish failure"
    );
    assert!(
        case.out.join("stage_3_snapshot.json").exists(),
        "stage 3 snapshot must exist because prepare completed before publish failed"
    );
}

#[test]
fn test_stage3_fail_no_persist() {
    let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
    let case = fail_no_persist_case();
    assert!(case.state.stage3_ok);
    assert_no_sidecar(&case.out);

    let snap: Stage3Snapshot =
        load_json(case.out.join("stage_3_snapshot.json")).expect("load stage3 snapshot");
    assert_eq!(snap.wallet_import_stats.len(), 3);
    for row in snap.wallet_import_stats {
        assert_eq!(row.inserted, 0, "{} must not insert assets", row.actor);
        assert_eq!(
            row.already_exists, 0,
            "{} must not mark already_exists",
            row.actor
        );
        assert!(row.rejected > 0, "{} must reject imports", row.actor);
    }

    let audit_rows: Vec<serde_json::Value> =
        load_json(case.out.join("claim").join("audit_log.json")).expect("read audit log");
    let mut seen = 0usize;
    for row in audit_rows {
        let reason = row
            .get("reason_code")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        assert_eq!(reason, "IMPORT_CONSERVATION_VIOLATION");
        seen = seen.saturating_add(1);
    }
    assert_eq!(
        seen, snap.distributed_assets_count,
        "all stage3 imports must be rejected"
    );
}
