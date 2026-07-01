use std::{collections::BTreeSet, path::PathBuf};

use serde::Deserialize;
use tempfile::tempdir;
use z00z_aggregators::{AggregatorId, ShardId};
use z00z_rollup_node::{NodeConfig, PublicationHandoffMeta, StartupPreflightInput};
use z00z_storage::{
    checkpoint::CheckpointId,
    settlement::{
        RootGeneration, SettlementRecoveryState, SettlementRouteCtx, SettlementStateRoot,
        HJMT_PROOF_ENVELOPE_VERSION,
    },
};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io,
};

#[path = "support/test_hjmt_home.rs"]
mod hjmt_test_home;

use hjmt_test_home::{agg, repo_hjmt_home, write_hjmt_home};

#[test]
fn repo_preflight_accepts_canonical_runtime() {
    let cfg = NodeConfig::from_hjmt_home(repo_hjmt_home()).expect("load repo home");
    let route_digest = route_digest_bytes(&cfg);
    let proof_bytes = positive_proof_bytes("BPB-G-001");
    let handoff = handoff_rows(route_digest, 7);
    let scenario_cfg = repo_scenario_cfg();

    let report = cfg
        .startup_preflight(
            AggregatorId::new(0),
            StartupPreflightInput {
                recovery: &recovery_state([0u8; 32]),
                proof_bytes: &proof_bytes,
                handoff: &handoff,
                scenario_cfg_path: Some(&scenario_cfg),
            },
        )
        .expect("preflight report");

    assert_eq!(
        report.route_table_digest,
        "000c78634c31e624c5e194378e6c7613e916e1975ca901e5d6416325c1d617e1"
    );
    assert_eq!(report.checks.len(), 7);
    assert_eq!(report.config_digests.len(), 10);
    let labels = report
        .config_digests
        .iter()
        .map(|item| item.label.as_str())
        .collect::<BTreeSet<_>>();
    assert!(labels.contains("planner-config"));
    assert!(labels.contains("storage-config"));
    assert!(labels.contains("route-source"));
    assert!(labels.contains("runtime-manifest"));
    assert!(labels.contains("scenario-config"));
    assert!(labels.contains("aggregator-config-0"));
}

#[test]
fn route_digest_drift_rejects() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_2a2s");
    write_hjmt_home(
        &home,
        1,
        &[agg(0, 7200, &[(0, &[1])]), agg(1, 7201, &[(1, &[0])])],
    );
    rewrite_all_route_digests(&home, &route_digest_text(&home), "0".repeat(64).as_str());

    let err = NodeConfig::from_hjmt_home(&home).expect_err("route digest drift must reject");
    assert!(err.to_string().contains("route table digest mismatch"));
}

#[test]
fn wrong_journal_lineage_rejects_preflight() {
    let cfg = NodeConfig::from_hjmt_home(repo_hjmt_home()).expect("load repo home");
    let route_digest = route_digest_bytes(&cfg);
    let proof_bytes = positive_proof_bytes("BPB-G-001");
    let handoff = handoff_rows(route_digest, 7);

    let err = cfg
        .startup_preflight(
            AggregatorId::new(0),
            StartupPreflightInput {
                recovery: &recovery_state([0x11; 32]),
                proof_bytes: &proof_bytes,
                handoff: &handoff,
                scenario_cfg_path: None,
            },
        )
        .expect_err("wrong lineage must reject");

    assert!(err.to_string().contains("expected journal lineage"));
}

#[test]
fn test_missing_route_recovery() {
    let cfg = NodeConfig::from_hjmt_home(repo_hjmt_home()).expect("load repo home");
    let route_digest = route_digest_bytes(&cfg);
    let proof_bytes = positive_proof_bytes("BPB-G-001");
    let handoff = handoff_rows(route_digest, 7);
    let mut recovery = recovery_state([0u8; 32]);
    recovery.version = 1;

    let err = cfg
        .startup_preflight(
            AggregatorId::new(0),
            StartupPreflightInput {
                recovery: &recovery,
                proof_bytes: &proof_bytes,
                handoff: &handoff,
                scenario_cfg_path: None,
            },
        )
        .expect_err("non-zero recovery without route must reject");

    assert!(err
        .to_string()
        .contains("recovery state must export route identity"));
}

#[test]
fn test_wrong_route_digest() {
    let cfg = NodeConfig::from_hjmt_home(repo_hjmt_home()).expect("load repo home");
    let route_digest = route_digest_bytes(&cfg);
    let proof_bytes = positive_proof_bytes("BPB-G-001");
    let handoff = handoff_rows(route_digest, 7);
    let recovery = route_bound_recovery_state([0u8; 32], 0, 1, [0x77; 32]);

    let err = cfg
        .startup_preflight(
            AggregatorId::new(0),
            StartupPreflightInput {
                recovery: &recovery,
                proof_bytes: &proof_bytes,
                handoff: &handoff,
                scenario_cfg_path: None,
            },
        )
        .expect_err("wrong route digest must reject");

    assert!(err
        .to_string()
        .contains("recovery state route table digest"));
}

#[test]
fn foreign_recovery_shard_rejects_preflight() {
    let cfg = NodeConfig::from_hjmt_home(repo_hjmt_home()).expect("load repo home");
    let route_digest = route_digest_bytes(&cfg);
    let proof_bytes = positive_proof_bytes("BPB-G-001");
    let handoff = handoff_rows(route_digest, 7);
    let recovery = route_bound_recovery_state([0u8; 32], 99, 1, route_digest);

    let err = cfg
        .startup_preflight(
            AggregatorId::new(0),
            StartupPreflightInput {
                recovery: &recovery,
                proof_bytes: &proof_bytes,
                handoff: &handoff,
                scenario_cfg_path: None,
            },
        )
        .expect_err("foreign shard must reject");

    assert!(err
        .to_string()
        .contains("recovery state shard is not owned"));
}

#[test]
fn wrong_root_generation_rejects_preflight() {
    let cfg = NodeConfig::from_hjmt_home(repo_hjmt_home()).expect("load repo home");
    let route_digest = route_digest_bytes(&cfg);
    let proof_bytes = positive_proof_bytes("BPB-G-001");
    let handoff = handoff_rows(route_digest, 7);
    let mut recovery = recovery_state([0u8; 32]);
    recovery.root_generation = 0;

    let err = cfg
        .startup_preflight(
            AggregatorId::new(0),
            StartupPreflightInput {
                recovery: &recovery,
                proof_bytes: &proof_bytes,
                handoff: &handoff,
                scenario_cfg_path: None,
            },
        )
        .expect_err("wrong root_generation must reject");

    assert!(err
        .to_string()
        .contains("unsupported settlement root generation"));
}

#[test]
fn wrong_proof_version_rejects_preflight() {
    let cfg = NodeConfig::from_hjmt_home(repo_hjmt_home()).expect("load repo home");
    let route_digest = route_digest_bytes(&cfg);
    let proof_bytes = positive_proof_bytes("BPB-G-001");
    let handoff = handoff_rows(route_digest, 7);
    let mut recovery = recovery_state([0u8; 32]);
    recovery.proof_version = recovery.proof_version.saturating_add(1);

    let err = cfg
        .startup_preflight(
            AggregatorId::new(0),
            StartupPreflightInput {
                recovery: &recovery,
                proof_bytes: &proof_bytes,
                handoff: &handoff,
                scenario_cfg_path: None,
            },
        )
        .expect_err("wrong proof_version must reject");

    assert!(err
        .to_string()
        .contains("unsupported settlement proof version"));
}

#[test]
fn malformed_proof_bytes_reject_preflight() {
    let cfg = NodeConfig::from_hjmt_home(repo_hjmt_home()).expect("load repo home");
    let route_digest = route_digest_bytes(&cfg);
    let handoff = handoff_rows(route_digest, 7);

    let err = cfg
        .startup_preflight(
            AggregatorId::new(0),
            StartupPreflightInput {
                recovery: &recovery_state([0u8; 32]),
                proof_bytes: &[0u8; 8],
                handoff: &handoff,
                scenario_cfg_path: None,
            },
        )
        .expect_err("malformed proof must reject");

    assert!(err.to_string().contains("batch proof decode failed"));
}

#[test]
fn unordered_handoff_rejects_preflight() {
    let cfg = NodeConfig::from_hjmt_home(repo_hjmt_home()).expect("load repo home");
    let route_digest = route_digest_bytes(&cfg);
    let proof_bytes = positive_proof_bytes("BPB-G-001");
    let mut handoff = handoff_rows(route_digest, 7);
    handoff.swap(0, 1);

    let err = cfg
        .startup_preflight(
            AggregatorId::new(0),
            StartupPreflightInput {
                recovery: &recovery_state([0u8; 32]),
                proof_bytes: &proof_bytes,
                handoff: &handoff,
                scenario_cfg_path: None,
            },
        )
        .expect_err("unordered handoff must reject");

    assert!(err.to_string().contains("strict ascending order"));
}

#[test]
fn missing_handoff_rejects_preflight() {
    let cfg = NodeConfig::from_hjmt_home(repo_hjmt_home()).expect("load repo home");
    let route_digest = route_digest_bytes(&cfg);
    let proof_bytes = positive_proof_bytes("BPB-G-001");
    let mut handoff = handoff_rows(route_digest, 7);
    handoff.pop();

    let err = cfg
        .startup_preflight(
            AggregatorId::new(0),
            StartupPreflightInput {
                recovery: &recovery_state([0u8; 32]),
                proof_bytes: &proof_bytes,
                handoff: &handoff,
                scenario_cfg_path: None,
            },
        )
        .expect_err("missing handoff must reject");

    assert!(err
        .to_string()
        .contains("every active route-table shard exactly once"));
}

#[test]
fn unsupported_generation_rejects() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_2a2s");
    write_hjmt_home(
        &home,
        1,
        &[agg(0, 7300, &[(0, &[1])]), agg(1, 7301, &[(1, &[0])])],
    );
    replace_text(
        &home.join("storage/storage-config.yaml"),
        "generation: 1",
        "generation: 9",
    );

    let err = NodeConfig::from_hjmt_home(&home).expect_err("unsupported generation must reject");
    assert!(err
        .to_string()
        .contains("unsupported settlement backend generation"));
}

#[test]
fn unsupported_backend_rejects() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_2a2s");
    write_hjmt_home(
        &home,
        1,
        &[agg(0, 7310, &[(0, &[1])]), agg(1, 7311, &[(1, &[0])])],
    );
    replace_text(
        &home.join("storage/storage-config.yaml"),
        "backend: hjmt",
        "backend: mirror",
    );

    let err = NodeConfig::from_hjmt_home(&home).expect_err("unsupported backend must reject");
    assert!(err
        .to_string()
        .contains("unsupported settlement backend mode"));
}

#[test]
fn missing_startup_block_rejects() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_2a2s");
    write_hjmt_home(
        &home,
        1,
        &[agg(0, 7400, &[(0, &[1])]), agg(1, 7401, &[(1, &[0])])],
    );
    strip_section(
        &home.join("aggregators/agg-0/aggregator-config.yaml"),
        "startup:",
        "evidence:",
    );

    let err = NodeConfig::from_hjmt_home(&home).expect_err("missing startup block must reject");
    assert!(err.to_string().contains("failed to decode YAML"));
}

#[test]
fn planner_mode_updates_digest() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_2a2s");
    write_hjmt_home(
        &home,
        1,
        &[agg(0, 7500, &[(0, &[1])]), agg(1, 7501, &[(1, &[0])])],
    );
    let before = planner_digest(&NodeConfig::from_hjmt_home(&home).expect("load before"));

    replace_text(
        &home.join("planner/planner-config.yaml"),
        "mode: central",
        "mode: per_agg",
    );

    let after = planner_digest(&NodeConfig::from_hjmt_home(&home).expect("load after"));
    assert_ne!(before, after);
}

#[test]
fn listen_addr_updates_digest() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_2a2s");
    write_hjmt_home(
        &home,
        1,
        &[agg(0, 7550, &[(0, &[1])]), agg(1, 7551, &[(1, &[0])])],
    );
    let before = agg_digest(&NodeConfig::from_hjmt_home(&home).expect("load before"), 0);

    replace_text(
        &home.join("aggregators/agg-0/aggregator-config.yaml"),
        "127.0.0.1:7550",
        "127.0.0.1:8550",
    );

    let after = agg_digest(&NodeConfig::from_hjmt_home(&home).expect("load after"), 0);
    assert_ne!(before, after);
}

#[test]
fn standby_set_updates_digest() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_3a2s");
    write_hjmt_home(
        &home,
        1,
        &[
            agg(0, 7560, &[(0, &[1])]),
            agg(1, 7561, &[(1, &[0])]),
            agg(2, 7562, &[]),
        ],
    );
    let before = agg_digest(&NodeConfig::from_hjmt_home(&home).expect("load before"), 0);

    replace_text(
        &home.join("aggregators/agg-0/aggregator-config.yaml"),
        "standby_ids: [1]",
        "standby_ids: [1, 2]",
    );

    let after = agg_digest(&NodeConfig::from_hjmt_home(&home).expect("load after"), 0);
    assert_ne!(before, after);
}

#[test]
fn journal_path_updates_digest() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_2a2s");
    write_hjmt_home(
        &home,
        1,
        &[agg(0, 7600, &[(0, &[1])]), agg(1, 7601, &[(1, &[0])])],
    );
    let before = agg_digest(&NodeConfig::from_hjmt_home(&home).expect("load before"), 0);

    replace_text(
        &home.join("aggregators/agg-0/aggregator-config.yaml"),
        "journal.redb",
        "journal-v2.redb",
    );

    let after = agg_digest(&NodeConfig::from_hjmt_home(&home).expect("load after"), 0);
    assert_ne!(before, after);
}

#[test]
fn non_aggregator_role_rejects() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_2a2s");
    write_hjmt_home(
        &home,
        1,
        &[agg(0, 7605, &[(0, &[1])]), agg(1, 7606, &[(1, &[0])])],
    );
    replace_text(
        &home.join("aggregators/agg-0/aggregator-config.yaml"),
        "role: \"aggregator\"",
        "role: \"observer\"",
    );

    let err = NodeConfig::from_hjmt_home(&home).expect_err("non-aggregator role must reject");
    assert!(err
        .to_string()
        .contains("aggregator 0 role must stay aggregator"));
}

#[test]
fn unknown_standby_rejects_at_load() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_2a2s");
    write_hjmt_home(
        &home,
        1,
        &[agg(0, 7610, &[(0, &[1])]), agg(1, 7611, &[(1, &[0])])],
    );
    replace_text(
        &home.join("aggregators/agg-0/aggregator-config.yaml"),
        "standby_ids: [1]",
        "standby_ids: [9]",
    );

    let err = NodeConfig::from_hjmt_home(&home).expect_err("unknown standby must reject");
    assert!(err
        .to_string()
        .contains("standby 9 is not a declared aggregator"));
}

#[test]
fn zero_shard_topology_rejects() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_2a0s");
    write_hjmt_home(&home, 1, &[agg(0, 7620, &[(0, &[1])]), agg(1, 7621, &[])]);
    collapse_shards_to_empty(&home.join("aggregators/agg-0/aggregator-config.yaml"));

    let err = NodeConfig::from_hjmt_home(&home).expect_err("zero shard topology must reject");
    assert!(err
        .to_string()
        .contains("topology must own at least one shard"));
}

fn planner_digest(cfg: &NodeConfig) -> String {
    cfg.config_digests()
        .expect("config digests")
        .into_iter()
        .find(|item| item.label == "planner-config")
        .expect("planner digest")
        .digest_hex
}

fn agg_digest(cfg: &NodeConfig, aggregator_id: u16) -> String {
    cfg.config_digests()
        .expect("config digests")
        .into_iter()
        .find(|item| item.label == format!("aggregator-config-{aggregator_id}"))
        .expect("agg digest")
        .digest_hex
}

fn route_digest_bytes(cfg: &NodeConfig) -> [u8; 32] {
    let raw = cfg
        .hjmt
        .as_ref()
        .and_then(|hjmt| hjmt.planner.route.expected_digest.as_ref())
        .expect("route digest");
    decode_hex32(raw)
}

fn route_digest_text(home: &std::path::Path) -> String {
    let planner = String::from_utf8(
        io::read_file(home.join("planner/planner-config.yaml")).expect("planner cfg"),
    )
    .expect("utf8");
    planner
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("expected_digest: ")
                .map(|value| value.trim_matches('"').to_string())
        })
        .expect("route digest text")
}

fn rewrite_all_route_digests(home: &std::path::Path, from: &str, to: &str) {
    replace_text(&home.join("planner/planner-config.yaml"), from, to);
    for id in 0..2 {
        replace_text(
            &home.join(format!("aggregators/agg-{id}/aggregator-config.yaml")),
            from,
            to,
        );
    }
}

fn replace_text(path: &std::path::Path, from: &str, to: &str) {
    let body = String::from_utf8(io::read_file(path).expect("read file")).expect("utf8");
    let body = body.replace(from, to);
    io::write_file(path, body.as_bytes()).expect("write file");
}

fn strip_section(path: &std::path::Path, start: &str, end: &str) {
    let body = String::from_utf8(io::read_file(path).expect("read file")).expect("utf8");
    let mut out = Vec::new();
    let mut skip = false;
    for line in body.lines() {
        if line.starts_with(start) {
            skip = true;
            continue;
        }
        if skip && line.starts_with(end) {
            skip = false;
        }
        if !skip {
            out.push(line.to_string());
        }
    }
    let body = out.join("\n") + "\n";
    io::write_file(path, body.as_bytes()).expect("write file");
}

fn collapse_shards_to_empty(path: &std::path::Path) {
    let body = String::from_utf8(io::read_file(path).expect("read file")).expect("utf8");
    let start = body.find("shards:\n").expect("shards section start");
    let end = body[start..]
        .find("network:\n")
        .map(|index| start + index)
        .expect("network section start");
    let mut out = String::with_capacity(body.len());
    out.push_str(&body[..start]);
    out.push_str("shards: []\n");
    out.push_str(&body[end..]);
    io::write_file(path, out.as_bytes()).expect("write file");
}

fn handoff_rows(route_digest: [u8; 32], shard_count: u16) -> Vec<PublicationHandoffMeta> {
    (0..shard_count)
        .map(|shard_id| PublicationHandoffMeta {
            shard_id: ShardId::new(shard_id),
            routing_generation: 1,
            route_table_digest: route_digest,
            checkpoint_id: CheckpointId::new([shard_id as u8 + 1; 32]),
        })
        .collect()
}

fn recovery_state(lineage: [u8; 32]) -> SettlementRecoveryState {
    SettlementRecoveryState::new(
        0,
        SettlementStateRoot::settlement_v1([0u8; 32]),
        RootGeneration::SettlementV1.version(),
        HJMT_PROOF_ENVELOPE_VERSION as u16,
        0,
        [0u8; 32],
        lineage,
    )
}

fn route_bound_recovery_state(
    lineage: [u8; 32],
    shard_id: u16,
    routing_generation: u64,
    route_digest: [u8; 32],
) -> SettlementRecoveryState {
    let mut recovery = recovery_state(lineage);
    recovery.version = 1;
    recovery.route = Some(SettlementRouteCtx::new(
        [0x51; 32],
        u32::from(shard_id),
        routing_generation,
        route_digest,
    ));
    recovery
}

fn positive_proof_bytes(case_id: &str) -> Vec<u8> {
    let manifest = io::read_file(repo_proof_manifest()).expect("read proof manifest");
    let parsed: ProofManifest = JsonCodec
        .deserialize(&manifest)
        .expect("proof manifest json");
    let case = parsed
        .cases
        .into_iter()
        .find(|case| case.id == case_id)
        .expect("proof case");
    hex::decode(case.canonical_bytes_hex).expect("proof bytes")
}

fn repo_proof_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_positive/manifest.json",
    )
}

fn repo_scenario_cfg() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/z00z_simulator/src/scenario_1/scenario_config.yaml")
}

fn decode_hex32(raw: &str) -> [u8; 32] {
    let bytes = hex::decode(raw).expect("hex");
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    out
}

#[derive(Debug, Deserialize)]
struct ProofManifest {
    cases: Vec<ProofCase>,
}

#[derive(Debug, Deserialize)]
struct ProofCase {
    id: String,
    canonical_bytes_hex: String,
}
