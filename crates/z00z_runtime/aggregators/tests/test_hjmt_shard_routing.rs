mod test_common;

use serde::{Deserialize, Serialize};
use z00z_aggregators::{
    AggregatorId, PlanDigest, RejectClass, RouteErr, SecondaryState, ShardExecState, ShardExecutor,
    ShardPlacement, ShardPlacementTable, ShardRouteTable,
};
use z00z_utils::codec::{Codec, JsonCodec};

use self::test_common::{
    batch_id, bridge_table, hex_decode, hex_encode, next_hash, split_table, tx_item,
};

const MANIFEST_JSON: &str = include_str!(
    "../../../z00z_storage/tests/fixtures/hjmt_upgrade/shard_route_table_v1/manifest.json"
);
const REGEN_CMD: &str = "Z00Z_REGEN_DUMP=1 cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_shard_routing test_route_manifest_matches_contract -- --exact --nocapture";
const EVIDENCE_PTR: &str = "crates/z00z_runtime/aggregators/tests/test_hjmt_shard_routing.rs::test_route_manifest_matches_contract";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RouteManifest {
    version: u8,
    golden: Vec<GoldCase>,
    tamper: Vec<TamperCase>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct GoldCase {
    id: String,
    kind: String,
    table_hex: Option<String>,
    digest_hex: Option<String>,
    old_table_hex: Option<String>,
    old_digest_hex: Option<String>,
    new_table_hex: Option<String>,
    new_digest_hex: Option<String>,
    activation_checkpoint: u64,
    regen_command: String,
    evidence_pointer: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TamperCase {
    id: String,
    kind: String,
    source_id: String,
    table_hex: Option<String>,
    old_table_hex: Option<String>,
    new_table_hex: Option<String>,
    claimed_digest_hex: Option<String>,
    expected_stage: String,
    expected_error: String,
    regen_command: String,
    evidence_pointer: String,
}

#[test]
fn test_route_manifest_matches_contract() {
    let expected: RouteManifest = JsonCodec
        .deserialize(MANIFEST_JSON.as_bytes())
        .expect("manifest json");
    let live = live_manifest();
    if std::env::var_os("Z00Z_REGEN_DUMP").is_some() {
        let json = JsonCodec.serialize_pretty(&live).expect("manifest json");
        println!("{}", String::from_utf8(json).expect("manifest utf8"));
    }
    assert_eq!(expected, live);
}

#[test]
fn test_route_goldens_prove_codec() {
    let manifest = live_manifest();

    for case in manifest.golden {
        match case.id.as_str() {
            "SRT-G-001" | "SRT-G-002" => {
                let table = decode_table(case.table_hex.as_deref().expect("table hex"));
                let digest = hex_encode(table.digest().as_bytes());
                assert_eq!(Some(digest), case.digest_hex);
            }
            "SRT-G-003" => {
                let old_table = decode_table(case.old_table_hex.as_deref().expect("old table"));
                let new_table = decode_table(case.new_table_hex.as_deref().expect("new table"));
                new_table
                    .validate_migration(&old_table)
                    .expect("historical migration pair");
                assert_eq!(
                    Some(hex_encode(old_table.digest().as_bytes())),
                    case.old_digest_hex
                );
                assert_eq!(
                    Some(hex_encode(new_table.digest().as_bytes())),
                    case.new_digest_hex
                );
            }
            "SRT-G-004" => {
                let table = decode_table(case.table_hex.as_deref().expect("table hex"));
                let bytes = table.canonical_bytes();
                let redecoded = ShardRouteTable::from_canon(&bytes).expect("canonical decode");
                assert_eq!(bytes, redecoded.canonical_bytes());
                assert_eq!(
                    Some(hex_encode(redecoded.digest().as_bytes())),
                    case.digest_hex
                );
            }
            other => panic!("unexpected golden case {other}"),
        }
    }
}

#[test]
fn test_route_tamper_vectors_fail() {
    let manifest = live_manifest();
    let bridge = bridge_table();

    for case in manifest.tamper {
        match case.id.as_str() {
            "SRT-T-001" | "SRT-T-002" | "SRT-T-003" | "SRT-T-004" | "SRT-T-005" | "SRT-T-006" => {
                let err = ShardRouteTable::from_canon(&hex_decode(
                    case.table_hex.as_deref().expect("table hex"),
                ))
                .expect_err("tamper vector must reject");
                assert_eq!(err.to_string(), case.expected_error);
            }
            "SRT-T-007" => {
                let old_table = decode_table(case.old_table_hex.as_deref().expect("old table"));
                let new_table = decode_table(case.new_table_hex.as_deref().expect("new table"));
                let err = new_table
                    .validate_migration(&old_table)
                    .expect_err("wrong previous_generation_digest must reject");
                assert_eq!(err.to_string(), case.expected_error);
            }
            "SRT-T-008" => {
                let table = decode_table(case.table_hex.as_deref().expect("table hex"));
                let live_digest = hex_encode(table.digest().as_bytes());
                assert_ne!(Some(live_digest), case.claimed_digest_hex);
            }
            other => panic!("unexpected tamper case {other}"),
        }
    }

    assert_eq!(
        bridge.lookup([0x11; 32]).expect("bridge lookup"),
        bridge.rules[0].shard_id
    );
}

#[test]
fn test_rejects_route_generation() {
    let item = tx_item("gen-bound");
    let table = split_table();
    let planner = z00z_aggregators::BatchPlanner::new(table.clone());
    let planned = planner
        .plan_batch(batch_id("gen-bound"), &[item])
        .expect("single shard plan");

    let mut matching = ShardPlacementTable::default();
    matching.insert(ShardPlacement::new(
        planned.route,
        AggregatorId::new(7),
        vec![SecondaryState::ready(AggregatorId::new(8))],
        [0x51; 32],
    ));
    let ticket = ShardExecutor::new(matching)
        .route(&planned)
        .expect("matching generation route");
    assert_eq!(ticket.state, ShardExecState::Routed);

    let mut drifted = ShardPlacementTable::default();
    drifted.insert(ShardPlacement::new(
        z00z_aggregators::BatchRoute {
            shard_id: planned.route.shard_id,
            routing_generation: planned.route.routing_generation + 1,
        },
        AggregatorId::new(7),
        vec![SecondaryState::ready(AggregatorId::new(8))],
        [0x61; 32],
    ));
    let err = ShardExecutor::new(drifted)
        .route(&planned)
        .expect_err("wrong generation must reject");
    assert_eq!(err.class, RejectClass::PolicyReject);
}

fn live_manifest() -> RouteManifest {
    let bridge = bridge_table();
    let split = split_table();
    let t001 = unsorted_shard_set();
    let t002 = dup_shard_set();
    let t003 = unsorted_rules();
    let t004 = overlapping_rules();
    let t005 = gap_rules();
    let t006 = foreign_shard();
    let t007 = wrong_prev_digest();

    RouteManifest {
        version: 1,
        golden: vec![
            GoldCase {
                id: "SRT-G-001".to_string(),
                kind: "bridge".to_string(),
                table_hex: Some(hex_encode(&bridge.canonical_bytes())),
                digest_hex: Some(hex_encode(bridge.digest().as_bytes())),
                old_table_hex: None,
                old_digest_hex: None,
                new_table_hex: None,
                new_digest_hex: None,
                activation_checkpoint: bridge.activation_checkpoint,
                regen_command: REGEN_CMD.to_string(),
                evidence_pointer: EVIDENCE_PTR.to_string(),
            },
            GoldCase {
                id: "SRT-G-002".to_string(),
                kind: "split".to_string(),
                table_hex: Some(hex_encode(&split.canonical_bytes())),
                digest_hex: Some(hex_encode(split.digest().as_bytes())),
                old_table_hex: None,
                old_digest_hex: None,
                new_table_hex: None,
                new_digest_hex: None,
                activation_checkpoint: split.activation_checkpoint,
                regen_command: REGEN_CMD.to_string(),
                evidence_pointer: EVIDENCE_PTR.to_string(),
            },
            GoldCase {
                id: "SRT-G-003".to_string(),
                kind: "migration_pair".to_string(),
                table_hex: None,
                digest_hex: None,
                old_table_hex: Some(hex_encode(&bridge.canonical_bytes())),
                old_digest_hex: Some(hex_encode(bridge.digest().as_bytes())),
                new_table_hex: Some(hex_encode(&split.canonical_bytes())),
                new_digest_hex: Some(hex_encode(split.digest().as_bytes())),
                activation_checkpoint: split.activation_checkpoint,
                regen_command: REGEN_CMD.to_string(),
                evidence_pointer: EVIDENCE_PTR.to_string(),
            },
            GoldCase {
                id: "SRT-G-004".to_string(),
                kind: "reencode".to_string(),
                table_hex: Some(hex_encode(&split.canonical_bytes())),
                digest_hex: Some(hex_encode(split.digest().as_bytes())),
                old_table_hex: None,
                old_digest_hex: None,
                new_table_hex: None,
                new_digest_hex: None,
                activation_checkpoint: split.activation_checkpoint,
                regen_command: REGEN_CMD.to_string(),
                evidence_pointer: EVIDENCE_PTR.to_string(),
            },
        ],
        tamper: vec![
            TamperCase {
                id: "SRT-T-001".to_string(),
                kind: "unsorted_shard_set".to_string(),
                source_id: "SRT-G-002".to_string(),
                table_hex: Some(hex_encode(&t001.canonical_bytes())),
                old_table_hex: None,
                new_table_hex: None,
                claimed_digest_hex: None,
                expected_stage: "parser_reject".to_string(),
                expected_error: RouteErr::UnsortedShardSet.to_string(),
                regen_command: REGEN_CMD.to_string(),
                evidence_pointer: EVIDENCE_PTR.to_string(),
            },
            TamperCase {
                id: "SRT-T-002".to_string(),
                kind: "dup_shard_set".to_string(),
                source_id: "SRT-G-002".to_string(),
                table_hex: Some(hex_encode(&t002.canonical_bytes())),
                old_table_hex: None,
                new_table_hex: None,
                claimed_digest_hex: None,
                expected_stage: "parser_reject".to_string(),
                expected_error: RouteErr::DupShardId.to_string(),
                regen_command: REGEN_CMD.to_string(),
                evidence_pointer: EVIDENCE_PTR.to_string(),
            },
            TamperCase {
                id: "SRT-T-003".to_string(),
                kind: "unsorted_rules".to_string(),
                source_id: "SRT-G-002".to_string(),
                table_hex: Some(hex_encode(&t003.canonical_bytes())),
                old_table_hex: None,
                new_table_hex: None,
                claimed_digest_hex: None,
                expected_stage: "parser_reject".to_string(),
                expected_error: RouteErr::UnsortedRules.to_string(),
                regen_command: REGEN_CMD.to_string(),
                evidence_pointer: EVIDENCE_PTR.to_string(),
            },
            TamperCase {
                id: "SRT-T-004".to_string(),
                kind: "overlap".to_string(),
                source_id: "SRT-G-002".to_string(),
                table_hex: Some(hex_encode(&t004.canonical_bytes())),
                old_table_hex: None,
                new_table_hex: None,
                claimed_digest_hex: None,
                expected_stage: "parser_reject".to_string(),
                expected_error: RouteErr::Overlap.to_string(),
                regen_command: REGEN_CMD.to_string(),
                evidence_pointer: EVIDENCE_PTR.to_string(),
            },
            TamperCase {
                id: "SRT-T-005".to_string(),
                kind: "gap".to_string(),
                source_id: "SRT-G-002".to_string(),
                table_hex: Some(hex_encode(&t005.canonical_bytes())),
                old_table_hex: None,
                new_table_hex: None,
                claimed_digest_hex: None,
                expected_stage: "parser_reject".to_string(),
                expected_error: RouteErr::Gap.to_string(),
                regen_command: REGEN_CMD.to_string(),
                evidence_pointer: EVIDENCE_PTR.to_string(),
            },
            TamperCase {
                id: "SRT-T-006".to_string(),
                kind: "foreign_shard".to_string(),
                source_id: "SRT-G-002".to_string(),
                table_hex: Some(hex_encode(&t006.canonical_bytes())),
                old_table_hex: None,
                new_table_hex: None,
                claimed_digest_hex: None,
                expected_stage: "parser_reject".to_string(),
                expected_error: RouteErr::ForeignShard.to_string(),
                regen_command: REGEN_CMD.to_string(),
                evidence_pointer: EVIDENCE_PTR.to_string(),
            },
            TamperCase {
                id: "SRT-T-007".to_string(),
                kind: "wrong_previous_generation_digest".to_string(),
                source_id: "SRT-G-003".to_string(),
                table_hex: None,
                old_table_hex: Some(hex_encode(&bridge.canonical_bytes())),
                new_table_hex: Some(hex_encode(&t007.canonical_bytes())),
                claimed_digest_hex: None,
                expected_stage: "migration_reject".to_string(),
                expected_error: RouteErr::BadPrevGen.to_string(),
                regen_command: REGEN_CMD.to_string(),
                evidence_pointer: EVIDENCE_PTR.to_string(),
            },
            TamperCase {
                id: "SRT-T-008".to_string(),
                kind: "digest_mismatch".to_string(),
                source_id: "SRT-G-002".to_string(),
                table_hex: Some(hex_encode(&split.canonical_bytes())),
                old_table_hex: None,
                new_table_hex: None,
                claimed_digest_hex: Some(hex_encode(&[0u8; 32])),
                expected_stage: "digest_reject".to_string(),
                expected_error: "route table digest mismatch".to_string(),
                regen_command: REGEN_CMD.to_string(),
                evidence_pointer: EVIDENCE_PTR.to_string(),
            },
        ],
    }
}

fn decode_table(raw: &str) -> ShardRouteTable {
    ShardRouteTable::from_canon(&hex_decode(raw)).expect("canonical table decode")
}

fn unsorted_shard_set() -> ShardRouteTable {
    let mut table = split_table();
    table.shard_set.swap(0, 1);
    table
}

fn dup_shard_set() -> ShardRouteTable {
    let mut table = split_table();
    table.shard_set[1] = table.shard_set[0];
    table
}

fn unsorted_rules() -> ShardRouteTable {
    let mut table = split_table();
    table.rules[2].start = table.rules[1].start;
    table
}

fn overlapping_rules() -> ShardRouteTable {
    let mut table = split_table();
    table.rules[1].start = table.rules[0].end;
    table
}

fn gap_rules() -> ShardRouteTable {
    let mut table = split_table();
    table.rules[1].start = next_hash(next_hash(table.rules[0].end).expect("gap 1")).expect("gap 2");
    table
}

fn foreign_shard() -> ShardRouteTable {
    let mut table = split_table();
    table.rules[1].shard_id = z00z_aggregators::ShardId::new(99);
    table
}

fn wrong_prev_digest() -> ShardRouteTable {
    let mut table = split_table();
    table.previous_generation_digest = Some(PlanDigest::new([0x44; 32]));
    table
}
