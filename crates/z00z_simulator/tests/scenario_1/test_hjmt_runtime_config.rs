use std::{collections::BTreeSet, path::PathBuf};

use tempfile::tempdir;
use z00z_utils::io;

#[test]
fn runtime_accessors_project_profile_root() {
    let cfg = z00z_simulator::ScenarioCfg::from_file(repo_scenario_cfg()).expect("scenario cfg");
    let runtime = cfg.hjmt_runtime_ref().expect("hjmt runtime ref");
    let observability = cfg
        .runtime_observability_ref()
        .expect("runtime observability ref");
    let publication = cfg
        .publication_observability_ref()
        .expect("publication observability ref");

    assert_eq!(runtime.profile, "SIM-5A7S");
    assert_eq!(
        cfg.hjmt_config_root().expect("hjmt config root"),
        PathBuf::from("config/hjmt_runtime/sim_5a7s")
    );
    assert_eq!(observability.active_profile, "SIM-SMALL");
    assert_eq!(observability.supported_profiles.len(), 4);
    assert!(observability
        .heavy_only_profiles
        .iter()
        .any(|profile| profile == "SIM-BATCH-1000"));
    assert_eq!(observability.traces.cfg_flow_file, "cfg_flow.json");
    assert_eq!(
        observability.traces.recovery_flow_file,
        "recovery_flow.json"
    );
    assert_eq!(observability.traces.leaf_flow_file, "leaf_flow.json");
    assert_eq!(observability.traces.proof_flow_file, "proof_flow.json");
    assert_eq!(observability.traces.pub_flow_file, "pub_flow.json");
    assert_eq!(observability.traces.val_flow_file, "val_flow.json");
    assert_eq!(observability.traces.watch_flow_file, "watch_flow.json");
    assert_eq!(observability.packet.run_meta_file, "run_meta.json");
    assert_eq!(observability.packet.wallet_scan_file, "wallet_scan.json");
    assert_eq!(observability.packet.sim_summary_file, "sim_summary.md");
    assert_eq!(
        observability.packet.emitted_public_files,
        vec![
            String::from("hist_flow.json"),
            String::from("occ_flow.json"),
            String::from("asset_flow.json"),
            String::from("voucher_flow.json"),
            String::from("right_flow.json"),
        ]
    );
    assert_eq!(publication.acceptance_profile, "SIM-5A7S-PUB");
    assert_eq!(publication.inherited_runtime_profile, "SIM-5A7S");
    assert_eq!(publication.public_leaf_count, 7);
}

#[test]
fn observability_declares_trace_pack() {
    let cfg = z00z_simulator::ScenarioCfg::from_file(repo_scenario_cfg()).expect("scenario cfg");
    let observability = cfg
        .runtime_observability_ref()
        .expect("runtime observability ref");

    let supported_profiles = observability
        .supported_profiles
        .iter()
        .map(|profile| profile.id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        supported_profiles,
        BTreeSet::from([
            "SIM-BATCH-1000",
            "SIM-CACHE-EDGE",
            "SIM-MEDIUM",
            "SIM-SMALL",
        ])
    );
    assert_eq!(
        supported_profiles.len(),
        observability.supported_profiles.len()
    );
    assert!(supported_profiles.contains(observability.active_profile.as_str()));
    assert_eq!(
        observability.heavy_only_profiles,
        vec![String::from("SIM-BATCH-1000")]
    );
    assert_eq!(observability.publication.acceptance_profile, "SIM-5A7S-PUB");
    assert_eq!(
        observability.publication.publication_activation_checkpoint,
        101
    );
    assert_eq!(
        observability.publication.positive_topology_examples.len(),
        3
    );
    assert!(!observability
        .heavy_only_profiles
        .iter()
        .any(|profile| profile == &observability.active_profile));

    let trace_files = BTreeSet::from([
        observability.traces.cfg_flow_file.as_str(),
        observability.traces.tx_flow_file.as_str(),
        observability.traces.route_flow_file.as_str(),
        observability.traces.plan_flow_file.as_str(),
        observability.traces.journal_flow_file.as_str(),
        observability.traces.scope_flow_file.as_str(),
        observability.traces.proc_flow_file.as_str(),
        observability.traces.recovery_flow_file.as_str(),
        observability.traces.leaf_flow_file.as_str(),
        observability.traces.proof_flow_file.as_str(),
        observability.traces.pub_flow_file.as_str(),
        observability.traces.val_flow_file.as_str(),
        observability.traces.watch_flow_file.as_str(),
    ]);
    assert_eq!(
        trace_files,
        BTreeSet::from([
            "cfg_flow.json",
            "journal_flow.json",
            "leaf_flow.json",
            "plan_flow.json",
            "proof_flow.json",
            "proc_flow.json",
            "pub_flow.json",
            "recovery_flow.json",
            "route_flow.json",
            "scope_flow.json",
            "tx_flow.json",
            "val_flow.json",
            "watch_flow.json",
        ])
    );
    assert_eq!(
        observability.packet.emitted_public_files,
        vec![
            String::from("hist_flow.json"),
            String::from("occ_flow.json"),
            String::from("asset_flow.json"),
            String::from("voucher_flow.json"),
            String::from("right_flow.json"),
        ]
    );
}

#[test]
fn publication_yaml_topology_metadata() {
    let cfg = z00z_simulator::ScenarioCfg::from_file(repo_scenario_cfg()).expect("scenario cfg");
    let publication = cfg
        .publication_observability_ref()
        .expect("publication observability ref");
    let example = publication
        .positive_topology_examples
        .first()
        .expect("topology example");

    assert_eq!(example.old_topology, "5x7");
    assert_eq!(example.new_topology, "6x7");
    assert_eq!(example.owner_aggregator_id, 5);
    assert_eq!(example.standby_aggregator_ids, vec![0, 4]);
    assert_eq!(example.old_aggregator_count, 5);
    assert_eq!(example.old_shard_count, 7);
    assert_eq!(example.new_aggregator_count, 6);
    assert_eq!(example.new_shard_count, 7);
    assert_eq!(example.route_generation_from, 1);
    assert_eq!(example.route_generation_to, 2);
    assert_eq!(example.join_mode, "owner_activation");
    assert_eq!(example.transfer_target, "aggregator-5");
    assert_eq!(example.activation_checkpoint, 101);

    let secondary = publication
        .positive_topology_examples
        .get(1)
        .expect("secondary topology example");
    assert_eq!(secondary.fixture_id, "SIM-4A3S-OWNER");
    assert_eq!(secondary.old_topology, "3x3");
    assert_eq!(secondary.new_topology, "4x3");

    let staged = publication
        .positive_topology_examples
        .get(2)
        .expect("staged topology example");
    assert_eq!(staged.fixture_id, "SIM-3A7S-2A7S-5A7S");
    assert_eq!(staged.old_topology, "3x7");
    assert_eq!(staged.new_topology, "5x7");
    assert_eq!(staged.route_generation_from, 1);
    assert_eq!(staged.route_generation_to, 3);
    assert_eq!(staged.removed_aggregator_ids, vec![5]);
    assert!(staged.removed_aggregator_absent_from_owner_tables);
    assert!(staged.removed_aggregator_absent_from_standby_tables);
    assert!(staged.all_shards_owned_across_stages);
    assert!(staged.prior_lineage_preserved);
    assert!(staged.publication_continuity_preserved);
    assert_eq!(staged.transition_stages.len(), 3);
    assert_eq!(staged.transition_stages[0].topology, "3x7");
    assert_eq!(staged.transition_stages[0].owner_aggregator_id, 5);
    assert_eq!(staged.transition_stages[1].topology, "2x7");
    assert_eq!(staged.transition_stages[1].owner_aggregator_id, 0);
    assert_eq!(staged.transition_stages[2].topology, "5x7");
    assert_eq!(
        staged.transition_stages[2].standby_aggregator_ids,
        vec![2, 4]
    );
}

#[test]
fn scenario_digest_tracks_runtime_selection() {
    let temp = tempdir().expect("tempdir");
    let cfg_path = temp.path().join("scenario_config.yaml");
    let source = io::read_file(repo_scenario_cfg()).expect("read scenario cfg");
    io::write_file(&cfg_path, &source).expect("copy scenario cfg");

    let before =
        z00z_simulator::ScenarioCfg::config_digest(&cfg_path).expect("scenario digest before");
    let body = String::from_utf8(io::read_file(&cfg_path).expect("read cfg")).expect("utf8");
    let body = body.replace("SIM-5A7S", "SIM-ALT");
    io::write_file(&cfg_path, body.as_bytes()).expect("write cfg");
    let after =
        z00z_simulator::ScenarioCfg::config_digest(&cfg_path).expect("scenario digest after");

    assert_eq!(before.len(), 64);
    assert_eq!(after.len(), 64);
    assert_ne!(before, after);
}

fn repo_scenario_cfg() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_config.yaml")
}
