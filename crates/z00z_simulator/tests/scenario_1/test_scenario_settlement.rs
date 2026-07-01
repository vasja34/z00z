use z00z_core::genesis::GenesisSettlementManifest;
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{read_file, read_to_string},
};

use z00z_simulator::scenario_1::stage_13::shared_cases;

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
struct Fixture {
    assets: Vec<serde_json::Value>,
    rights: Vec<FixtureRight>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
struct FixtureRight {
    right_class: String,
}

const FIXTURE_JSON: &str =
    include_str!("../../../z00z_storage/tests/fixtures/test_settlement_corpus_fixture.json");

fn live_case() -> &'static std::path::PathBuf {
    static OUT: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    OUT.get_or_init(|| shared_cases::stage13_out("scenario_settlement"))
}

fn live_case_cfg() -> std::path::PathBuf {
    live_case()
        .parent()
        .and_then(|path| path.parent())
        .expect("live case base")
        .join("scenario_config.yaml")
}

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repo root")
        .to_path_buf()
}

fn assert_path_suffix(value: &serde_json::Value, suffix: &str) {
    let path = value.as_str().expect("path string");
    assert!(
        std::path::Path::new(path).ends_with(suffix),
        "expected path `{path}` to end with `{suffix}`"
    );
}

#[test]
fn test_cover_mixed_fixture_scope() {
    let fixture: Fixture = JsonCodec
        .deserialize(FIXTURE_JSON.as_bytes())
        .expect("fixture");
    let out_dir = live_case();

    let report: serde_json::Value = JsonCodec
        .deserialize(
            read_to_string(out_dir.join("hjmt/hjmt_settlement_examples.json"))
                .expect("stage13 report")
                .as_bytes(),
        )
        .expect("parse report");
    let manifest: GenesisSettlementManifest = JsonCodec
        .deserialize(
            read_to_string(out_dir.join("hjmt/genesis_settlement_manifest.json"))
                .expect("manifest")
                .as_bytes(),
        )
        .expect("parse manifest");

    let examples = report["examples"].as_array().expect("examples array");
    let comparison_rows = report["comparison_rows"]
        .as_array()
        .expect("comparison rows");
    let proof_families = examples
        .iter()
        .filter_map(|value| value["proof_family"].as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let leaf_families = examples
        .iter()
        .filter_map(|value| value["leaf_family"].as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert!(manifest.asset_count >= fixture.assets.len());
    assert!(manifest.right_count >= fixture.rights.len());
    assert_eq!(
        manifest.terminal_collision_checks.duplicate_right_terminals,
        0
    );
    assert_eq!(manifest.terminal_collision_checks.asset_right_collisions, 0);
    assert!(proof_families.contains("inclusion"));
    assert!(proof_families.contains("deletion"));
    assert!(proof_families.contains("nonexistence"));
    assert!(proof_families.contains("split"));
    assert!(proof_families.contains("policy_transition"));
    assert!(leaf_families.contains("asset"));
    assert!(leaf_families.contains("right"));
    let comparison_surfaces = comparison_rows
        .iter()
        .filter_map(|value| value["proof_surface"].as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let mut source_root_by_path = std::collections::BTreeMap::new();
    for example in examples {
        if let (Some(path), Some(root_hex)) = (
            example["settlement_path"].as_str(),
            example["settlement_state_root_hex"].as_str(),
        ) {
            if !path.is_empty() && path != "none" {
                source_root_by_path.insert(path.to_string(), root_hex.to_string());
            }
        }
    }
    for row in comparison_rows {
        let Some(root_hex) = row["settlement_state_root_hex"].as_str() else {
            continue;
        };
        let Some(paths) = row["settlement_paths"].as_array() else {
            continue;
        };
        for path in paths.iter().filter_map(serde_json::Value::as_str) {
            if !path.is_empty() && path != "none" {
                source_root_by_path
                    .entry(path.to_string())
                    .or_insert_with(|| root_hex.to_string());
            }
        }
    }
    assert!(comparison_surfaces.contains("proof_blob_single"));
    assert!(comparison_surfaces.contains("proof_blob_vec"));
    assert!(comparison_surfaces.contains("batch_proof_v1"));
    let batch_counts = comparison_rows
        .iter()
        .filter(|value| value["proof_surface"].as_str() == Some("batch_proof_v1"))
        .filter_map(|value| value["path_count"].as_u64())
        .collect::<std::collections::BTreeSet<_>>();
    for required in [2u64, 8u64, 32u64] {
        assert!(batch_counts.contains(&required));
    }

    z00z_simulator::scenario_1::runner::validate_runtime_observability_artifacts(
        live_case_cfg(),
        "src/scenario_1/scenario_design.yaml",
        out_dir,
    )
    .expect("runtime trace pack");

    let cfg_flow: serde_json::Value = JsonCodec
        .deserialize(
            read_to_string(out_dir.join("cfg_flow.json"))
                .expect("cfg_flow")
                .as_bytes(),
        )
        .expect("parse cfg_flow");
    let journal_flow: serde_json::Value = JsonCodec
        .deserialize(
            read_to_string(out_dir.join("journal_flow.json"))
                .expect("journal_flow")
                .as_bytes(),
        )
        .expect("parse journal_flow");
    let plan_flow: serde_json::Value = JsonCodec
        .deserialize(
            read_to_string(out_dir.join("plan_flow.json"))
                .expect("plan_flow")
                .as_bytes(),
        )
        .expect("parse plan_flow");
    let proc_flow: serde_json::Value = JsonCodec
        .deserialize(
            read_to_string(out_dir.join("proc_flow.json"))
                .expect("proc_flow")
                .as_bytes(),
        )
        .expect("parse proc_flow");
    let leaf_flow: serde_json::Value = JsonCodec
        .deserialize(
            read_to_string(out_dir.join("leaf_flow.json"))
                .expect("leaf_flow")
                .as_bytes(),
        )
        .expect("parse leaf_flow");
    let proof_flow: serde_json::Value = JsonCodec
        .deserialize(
            read_to_string(out_dir.join("proof_flow.json"))
                .expect("proof_flow")
                .as_bytes(),
        )
        .expect("parse proof_flow");
    let pub_flow: serde_json::Value = JsonCodec
        .deserialize(
            read_to_string(out_dir.join("pub_flow.json"))
                .expect("pub_flow")
                .as_bytes(),
        )
        .expect("parse pub_flow");
    let val_flow: serde_json::Value = JsonCodec
        .deserialize(
            read_to_string(out_dir.join("val_flow.json"))
                .expect("val_flow")
                .as_bytes(),
        )
        .expect("parse val_flow");
    let watch_flow: serde_json::Value = JsonCodec
        .deserialize(
            read_to_string(out_dir.join("watch_flow.json"))
                .expect("watch_flow")
                .as_bytes(),
        )
        .expect("parse watch_flow");
    let hist_flow: serde_json::Value = JsonCodec
        .deserialize(
            read_to_string(out_dir.join("hist_flow.json"))
                .expect("hist_flow")
                .as_bytes(),
        )
        .expect("parse hist_flow");
    let occ_flow: serde_json::Value = JsonCodec
        .deserialize(
            read_to_string(out_dir.join("occ_flow.json"))
                .expect("occ_flow")
                .as_bytes(),
        )
        .expect("parse occ_flow");
    let run_meta: serde_json::Value = JsonCodec
        .deserialize(
            read_to_string(out_dir.join("run_meta.json"))
                .expect("run_meta")
                .as_bytes(),
        )
        .expect("parse run_meta");
    let wallet_scan: serde_json::Value = JsonCodec
        .deserialize(
            read_to_string(out_dir.join("wallet_scan.json"))
                .expect("wallet_scan")
                .as_bytes(),
        )
        .expect("parse wallet_scan");
    let sim_summary = read_to_string(out_dir.join("sim_summary.md")).expect("sim_summary");
    let stage7_summary: serde_json::Value = JsonCodec
        .deserialize(
            read_to_string(out_dir.join("transactions/checkpoint_s7.json"))
                .expect("stage7 summary")
                .as_bytes(),
        )
        .expect("parse stage7 summary");
    let sim_manifest: serde_json::Value = JsonCodec
        .deserialize(
            &read_file(repo_root().join("config/hjmt_runtime/sim_5a7s/manifest.json"))
                .expect("sim manifest"),
        )
        .expect("parse sim manifest");
    let failover_manifest: serde_json::Value =
        JsonCodec
            .deserialize(
                &read_file(repo_root().join(
                    "crates/z00z_runtime/aggregators/tests/fixtures/failover_v1/manifest.json",
                ))
                .expect("failover manifest"),
            )
            .expect("parse failover manifest");

    assert_eq!(cfg_flow["active_profile"].as_str(), Some("SIM-SMALL"));
    assert_eq!(
        cfg_flow["trace_files"]["scope_flow_file"].as_str(),
        Some("scope_flow.json")
    );
    assert_eq!(
        cfg_flow["trace_files"]["val_flow_file"].as_str(),
        Some("val_flow.json")
    );
    assert_eq!(
        cfg_flow["trace_files"]["watch_flow_file"].as_str(),
        Some("watch_flow.json")
    );
    assert_eq!(
        cfg_flow["heavy_only_profiles"][0].as_str(),
        Some("SIM-BATCH-1000")
    );
    assert_eq!(
        cfg_flow["route_table_digest"].as_str(),
        journal_flow["route_table_digest"].as_str()
    );
    assert_eq!(
        cfg_flow["route_table_digest"].as_str(),
        proc_flow["route_table_digest"].as_str()
    );
    assert_eq!(plan_flow["planner_mode"].as_str(), Some("central"));
    assert_path_suffix(
        &plan_flow["planner_config_path"],
        sim_manifest["planner_config_path"]
            .as_str()
            .expect("manifest planner path"),
    );
    assert_eq!(
        pub_flow["publication_profile"].as_str(),
        Some("SIM-5A7S-PUB")
    );
    assert_eq!(
        leaf_flow["publication_profile"].as_str(),
        Some("SIM-5A7S-PUB")
    );
    assert_eq!(
        proof_flow["publication_profile"].as_str(),
        Some("SIM-5A7S-PUB")
    );
    assert_eq!(
        val_flow["publication_profile"].as_str(),
        Some("SIM-5A7S-PUB")
    );
    assert_eq!(
        watch_flow["publication_profile"].as_str(),
        Some("SIM-5A7S-PUB")
    );
    assert_eq!(sim_manifest["profile"].as_str(), Some("SIM-5A7S"));
    assert_eq!(
        sim_manifest["route_table_digest"].as_str(),
        cfg_flow["route_table_digest"].as_str()
    );
    assert_eq!(
        sim_manifest["publication"]["acceptance_profile"].as_str(),
        Some("SIM-5A7S-PUB")
    );
    assert_eq!(
        sim_manifest["publication"]["public_leaf_count"].as_u64(),
        Some(7)
    );
    assert_eq!(
        sim_manifest["shard_mapping"].as_str(),
        Some("aggregator_owned")
    );
    assert_eq!(sim_manifest["agg_ids"].as_array().map(Vec::len), Some(5));
    assert_eq!(sim_manifest["shard_ids"].as_array().map(Vec::len), Some(7));
    assert_eq!(
        proc_flow["process_topology"]["process_model"].as_str(),
        Some("os_process")
    );
    assert_eq!(
        proc_flow["process_topology"]["shard_mapping"].as_str(),
        Some("aggregator_owned")
    );
    assert_eq!(leaf_flow["leaf_rows"].as_array().map(Vec::len), Some(7));
    assert_eq!(proof_flow["proof_rows"].as_array().map(Vec::len), Some(7));
    assert_eq!(pub_flow["public_leaf_count"].as_u64(), Some(7));
    assert_eq!(
        pub_flow["process_verdicts"].as_array().map(Vec::len),
        Some(5)
    );
    let proc_rows = proc_flow["process_topology"]["aggregators"]
        .as_array()
        .expect("proc rows");
    let manifest_rows = sim_manifest["aggregators"]
        .as_array()
        .expect("manifest rows");
    assert_eq!(proc_rows.len(), manifest_rows.len());
    let mut process_ids = std::collections::BTreeSet::new();
    let mut listen_addrs = std::collections::BTreeSet::new();
    let mut data_dirs = std::collections::BTreeSet::new();
    let mut journal_paths = std::collections::BTreeSet::new();
    for (proc_row, manifest_row) in proc_rows.iter().zip(manifest_rows.iter()) {
        assert_eq!(
            proc_row["aggregator_id"].as_u64(),
            manifest_row["aggregator_id"].as_u64()
        );
        assert_eq!(
            proc_row["process_id"].as_str(),
            manifest_row["process_id"].as_str()
        );
        assert_eq!(
            proc_row["listen_addr"].as_str(),
            manifest_row["listen_addr"].as_str()
        );
        assert_eq!(proc_row["shard_ids"], manifest_row["shard_ids"]);
        assert_eq!(proc_row["start_cmd"], manifest_row["start_cmd"]);
        assert_eq!(proc_row["restart_cmd"], manifest_row["restart_cmd"]);
        assert_path_suffix(
            &proc_row["data_dir"],
            manifest_row["data_dir"]
                .as_str()
                .expect("manifest data dir"),
        );
        assert_path_suffix(
            &proc_row["journal_path"],
            manifest_row["journal_path"]
                .as_str()
                .expect("manifest journal path"),
        );
        process_ids.insert(proc_row["process_id"].as_str().expect("process id"));
        listen_addrs.insert(proc_row["listen_addr"].as_str().expect("listen addr"));
        data_dirs.insert(proc_row["data_dir"].as_str().expect("data dir"));
        journal_paths.insert(proc_row["journal_path"].as_str().expect("journal path"));
    }
    assert_eq!(process_ids.len(), 5);
    assert_eq!(listen_addrs.len(), 5);
    assert_eq!(data_dirs.len(), 5);
    assert_eq!(journal_paths.len(), 5);
    assert_eq!(leaf_flow["publication_checkpoint"].as_u64(), Some(101));
    assert_eq!(pub_flow["activation_checkpoint"].as_u64(), Some(101));
    assert_eq!(
        leaf_flow["prior_public_root_hex"].as_str(),
        report["settlement_state_root_hex"].as_str()
    );
    assert_eq!(
        leaf_flow["topology_examples"].as_array().map(Vec::len),
        Some(3)
    );
    assert_eq!(
        leaf_flow["topology_examples"][0]["fixture_id"].as_str(),
        Some("SIM-5A7S-OWNER")
    );
    assert_eq!(
        leaf_flow["topology_examples"][0]["new_topology"].as_str(),
        Some("6x7")
    );
    assert_eq!(
        leaf_flow["topology_examples"][2]["fixture_id"].as_str(),
        Some("SIM-3A7S-2A7S-5A7S")
    );
    assert_eq!(
        leaf_flow["topology_examples"][2]["removed_aggregator_ids"][0].as_u64(),
        Some(5)
    );
    assert_eq!(
        leaf_flow["topology_examples"][2]["removed_aggregator_absent_from_owner_tables"].as_bool(),
        Some(true)
    );
    assert_eq!(
        leaf_flow["topology_examples"][2]["removed_aggregator_absent_from_standby_tables"]
            .as_bool(),
        Some(true)
    );
    assert_eq!(
        leaf_flow["topology_examples"][2]["all_shards_owned_across_stages"].as_bool(),
        Some(true)
    );
    assert_eq!(
        leaf_flow["topology_examples"][2]["prior_lineage_preserved"].as_bool(),
        Some(true)
    );
    assert_eq!(
        leaf_flow["topology_examples"][2]["publication_continuity_preserved"].as_bool(),
        Some(true)
    );
    assert_eq!(
        leaf_flow["topology_examples"][2]["transition_stages"]
            .as_array()
            .map(Vec::len),
        Some(3)
    );
    assert_eq!(
        leaf_flow["topology_examples"][2]["transition_stages"][0]["aggregator_count"].as_u64(),
        Some(3)
    );
    assert_eq!(
        leaf_flow["topology_examples"][2]["transition_stages"][0]["route_generation"].as_u64(),
        Some(1)
    );
    assert_eq!(
        leaf_flow["topology_examples"][2]["transition_stages"][1]["aggregator_count"].as_u64(),
        Some(2)
    );
    assert_eq!(
        leaf_flow["topology_examples"][2]["transition_stages"][1]["route_generation"].as_u64(),
        Some(2)
    );
    assert_eq!(
        leaf_flow["topology_examples"][2]["transition_stages"][2]["aggregator_count"].as_u64(),
        Some(5)
    );
    assert_eq!(
        leaf_flow["topology_examples"][2]["transition_stages"][2]["route_generation"].as_u64(),
        Some(3)
    );
    assert_eq!(
        leaf_flow["topology_examples"][2]["transition_stages"][0]["owner_aggregator_id"].as_u64(),
        Some(5)
    );
    assert_eq!(
        leaf_flow["topology_examples"][2]["transition_stages"][1]["owner_aggregator_id"].as_u64(),
        Some(0)
    );
    assert!(
        leaf_flow["topology_examples"][2]["transition_stages"][1]["standby_aggregator_ids"]
            .as_array()
            .is_some_and(|rows| rows.iter().all(|value| value.as_u64() != Some(5)))
    );
    assert!(
        leaf_flow["topology_examples"][2]["transition_stages"][2]["standby_aggregator_ids"]
            .as_array()
            .is_some_and(|rows| rows.iter().all(|value| value.as_u64() != Some(5)))
    );
    assert_eq!(
        leaf_flow["topology_examples"][2]["transition_stages"][2]["standby_aggregator_ids"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    assert_eq!(
        leaf_flow["publication_digest_hex"].as_str(),
        pub_flow["publication_digest_hex"].as_str()
    );
    assert_eq!(
        proof_flow["publication_digest_hex"].as_str(),
        pub_flow["publication_digest_hex"].as_str()
    );
    assert_eq!(
        val_flow["publication_digest_hex"].as_str(),
        pub_flow["publication_digest_hex"].as_str()
    );
    assert_eq!(
        watch_flow["publication_digest_hex"].as_str(),
        pub_flow["publication_digest_hex"].as_str()
    );
    assert_eq!(run_meta["execution_mode"].as_str(), Some("release"));
    assert_eq!(
        run_meta["process_map_file"].as_str(),
        Some("proc_flow.json")
    );
    assert_eq!(
        run_meta["wallet_scan_file"].as_str(),
        Some("wallet_scan.json")
    );
    assert_eq!(run_meta["summary_file"].as_str(), Some("sim_summary.md"));
    assert_eq!(
        run_meta["artifact_inventory"].as_array().map(Vec::len),
        Some(21)
    );
    assert!(run_meta["artifact_inventory"]
        .as_array()
        .is_some_and(|rows| rows
            .iter()
            .any(|row| row["file"].as_str() == Some("wallet_scan.json")
                && row["status"].as_str() == Some("emitted"))));
    assert!(run_meta["artifact_inventory"]
        .as_array()
        .is_some_and(|rows| rows
            .iter()
            .any(|row| row["file"].as_str() == Some("hist_flow.json")
                && row["status"].as_str() == Some("emitted"))));
    assert!(run_meta["artifact_inventory"]
        .as_array()
        .is_some_and(|rows| rows
            .iter()
            .any(|row| row["file"].as_str() == Some("right_flow.json")
                && row["status"].as_str() == Some("emitted"))));
    assert!(run_meta["artifact_inventory"]
        .as_array()
        .is_some_and(|rows| rows
            .iter()
            .any(|row| row["file"].as_str() == Some("voucher_flow.json")
                && row["status"].as_str() == Some("emitted"))));
    assert_eq!(wallet_scan["actor"].as_str(), Some("charlie"));
    assert_eq!(wallet_scan["status"].as_str(), Some("ok"));
    assert_eq!(
        wallet_scan["store_root_hex"].as_str(),
        stage7_summary["new_root_hex"].as_str()
    );
    assert_eq!(
        stage7_summary["wallet_scan_file"].as_str(),
        Some("wallet_scan.json")
    );
    assert!(sim_summary.contains("## Release Packet"));
    assert!(sim_summary.contains("- emitted: run_meta.json"));
    assert!(sim_summary.contains("- emitted: hist_flow.json"));
    assert!(sim_summary.contains("- emitted: voucher_flow.json"));
    assert!(sim_summary.contains("- emitted: right_flow.json"));
    assert_eq!(hist_flow["trace_kind"].as_str(), Some("hist_flow"));
    assert_eq!(
        hist_flow["trace_mode"].as_str(),
        Some("imported_artifact_contract")
    );
    assert_eq!(
        hist_flow["route_migration_rows"].as_array().map(Vec::len),
        Some(3)
    );
    assert_eq!(
        hist_flow["route_migration_rows"][0]["old_public_root_hex"].as_str(),
        pub_flow["prior_public_root_hex"].as_str()
    );
    assert_eq!(
        hist_flow["route_migration_rows"][0]["new_public_root_hex"].as_str(),
        pub_flow["public_root_hex"].as_str()
    );
    assert_eq!(
        hist_flow["route_migration_rows"][0]["old_settlement_root_hex"].as_str(),
        val_flow["prev_settlement_root_hex"].as_str()
    );
    assert_eq!(
        hist_flow["route_migration_rows"][0]["new_settlement_root_hex"].as_str(),
        val_flow["new_settlement_root_hex"].as_str()
    );
    assert_eq!(
        hist_flow["route_migration_rows"][2]["fixture_id"].as_str(),
        Some("SIM-3A7S-2A7S-5A7S")
    );
    assert_eq!(
        hist_flow["route_migration_rows"][2]["old_route_generation"].as_u64(),
        Some(1)
    );
    assert_eq!(
        hist_flow["route_migration_rows"][2]["new_route_generation"].as_u64(),
        Some(3)
    );
    assert!(hist_flow["historical_proof_verdicts"]
        .as_array()
        .is_some_and(|rows| rows.iter().any(|row| row["example_id"].as_str()
            == Some("E2_right_inclusion")
            && row["verifier_status"].as_str() == Some("verified"))));
    assert!(hist_flow["live_reject_rows"]
        .as_array()
        .is_some_and(|rows| rows.iter().any(|row| row["case_id"].as_str()
            == Some("wrong_root_generation")
            && row["typed_error_class"].as_str() == Some("RootGenerationMix"))));
    assert!(hist_flow["owner_reject_rows"]
        .as_array()
        .is_some_and(|rows| rows
            .iter()
            .any(|row| row["contract_id"].as_str() == Some("route_generation_drift_reject"))));
    assert!(occ_flow["occupancy_disclosure_verdicts"]
        .as_array()
        .is_some_and(|rows| rows.iter().any(|row| row["example_id"].as_str()
            == Some("E6_adaptive_split")
            && row["disclosure_guard"].as_str() == Some("coarse_only"))));
    assert!(occ_flow["live_reject_rows"]
        .as_array()
        .is_some_and(|rows| rows
            .iter()
            .any(|row| row["case_id"].as_str() == Some("stale_policy_transition_id"))));
    assert_eq!(
        val_flow["binding_digest_hex"].as_str(),
        watch_flow["binding_digest_hex"].as_str()
    );
    assert_eq!(
        val_flow["checkpoint_id_hex"].as_str(),
        watch_flow["checkpoint_id_hex"].as_str()
    );
    assert_eq!(
        val_flow["checkpoint_id_hex"].as_str().map(str::len),
        Some(64)
    );
    assert_eq!(
        watch_flow["checkpoint_id_hex"].as_str().map(str::len),
        Some(64)
    );
    assert_eq!(
        val_flow["draft_id_hex"].as_str(),
        watch_flow["draft_id_hex"].as_str()
    );
    assert_eq!(val_flow["verdict_kind"].as_str(), Some("accepted"));
    assert_eq!(watch_flow["verdict_kind"].as_str(), Some("accepted"));
    assert_eq!(watch_flow["publication_state"].as_str(), Some("accepted"));
    assert_eq!(val_flow["route_generation"].as_u64(), Some(1));
    assert!(failover_manifest["cases"]
        .as_array()
        .is_some_and(|rows| !rows.is_empty()));
    assert!(failover_manifest["cases"]
        .as_array()
        .is_some_and(|rows| rows
            .iter()
            .any(|row| row["fixture_id"].as_str() == Some("FOV-001"))));
    assert!(failover_manifest["cases"]
        .as_array()
        .is_some_and(|rows| rows
            .iter()
            .any(|row| row["fixture_class"].as_str() == Some("Route migration fixture"))));
    assert!(journal_flow["journal_contract"]["cache_capacity"]
        .as_u64()
        .is_some_and(|value| value > 0));
    assert!(leaf_flow["leaf_rows"].as_array().is_some_and(|rows| rows
        .iter()
        .all(|row| row["source_settlement_path"]
            .as_str()
            .is_some_and(|value| !value.is_empty()))));
    assert!(proof_flow["proof_rows"].as_array().is_some_and(|rows| rows
        .iter()
        .all(|row| row["source_settlement_path"]
            .as_str()
            .is_some_and(|value| !value.is_empty()))));
    let unique_source_paths = leaf_flow["leaf_rows"]
        .as_array()
        .expect("leaf rows")
        .iter()
        .filter_map(|row| row["source_settlement_path"].as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(unique_source_paths.len(), 7);
    assert!(leaf_flow["leaf_rows"]
        .as_array()
        .is_some_and(|rows| rows.iter().all(|row| {
            let Some(path) = row["source_settlement_path"].as_str() else {
                return false;
            };
            let Some(root_hex) = row["state_root_hex"].as_str() else {
                return false;
            };
            source_root_by_path
                .get(path)
                .is_some_and(|expected_root| expected_root == root_hex)
        })));
    assert!(pub_flow["process_verdicts"]
        .as_array()
        .is_some_and(|rows| rows.iter().all(|row| row["restart_verdict"].as_str()
            == Some("config_bound_restart_ready")
            && process_ids.contains(row["process_id"].as_str().expect("verdict process id"))
            && journal_paths
                .contains(row["journal_path"].as_str().expect("verdict journal path")))));
    assert_eq!(
        watch_flow["process_verdicts"].as_array().map(Vec::len),
        Some(5)
    );
    assert_eq!(
        val_flow["topology_examples"].as_array().map(Vec::len),
        Some(3)
    );
    assert_eq!(
        watch_flow["topology_examples"].as_array().map(Vec::len),
        Some(3)
    );
    assert_eq!(proc_flow["process_topology"]["agg_count"].as_u64(), Some(5));
    assert_eq!(
        proc_flow["process_topology"]["shard_count"].as_u64(),
        Some(7)
    );
}
