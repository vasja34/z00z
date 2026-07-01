use std::path::PathBuf;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use z00z_crypto::expert::encoding::{from_hex, to_hex};
use z00z_storage::settlement::{
    check_checkpoint_publication_contract_v1, check_publication_route_v1, check_shard_root_leaf_v1,
    CheckpointPublicationV1, PolicySetCommitmentV1, ProofChkErr, PublicationModeTagV1,
    PublicationRouteSnapshotV1, RootGenerationTagV1, SettlementRecoveryState, SettlementStateRoot,
    ShardRootLeafV1,
};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io,
};

const LEAF_REGEN_CMD: &str = "cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_root_generation print_shard_root_leaf_manifest_json -- --ignored --nocapture";
const PUB_REGEN_CMD: &str = "cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_root_generation print_checkpoint_publication_manifest_json -- --ignored --nocapture";
const LEAF_EVIDENCE_PTR: &str =
    "crates/z00z_storage/tests/test_hjmt_root_generation.rs::test_shard_root_leaf_manifest_matches_live_contract";
const PUB_EVIDENCE_PTR: &str =
    "crates/z00z_storage/tests/test_hjmt_root_generation.rs::test_checkpoint_publication_manifest_matches_live_contract";

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct LeafManifest {
    version: u8,
    golden: Vec<LeafGoldenCase>,
    tamper: Vec<LeafTamperCase>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct LeafGoldenCase {
    id: String,
    kind: String,
    canonical_bytes_hex: Option<String>,
    expected_digest_hex: Option<String>,
    prior_leaf_hex: Option<String>,
    prior_digest_hex: Option<String>,
    regen_command: String,
    evidence_pointer: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct LeafTamperCase {
    id: String,
    kind: String,
    source_id: String,
    canonical_bytes_hex: Option<String>,
    prior_leaf_hex: Option<String>,
    claimed_digest_hex: Option<String>,
    mutation_point: String,
    expected_stage: String,
    expected_error: String,
    regen_command: String,
    evidence_pointer: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct PublicationManifest {
    version: u8,
    golden: Vec<PublicationGoldenCase>,
    tamper: Vec<PublicationTamperCase>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct PublicationGoldenCase {
    id: String,
    kind: String,
    canonical_bytes_hex: Option<String>,
    expected_digest_hex: Option<String>,
    prior_publication_hex: Option<String>,
    prior_digest_hex: Option<String>,
    regen_command: String,
    evidence_pointer: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct PublicationTamperCase {
    id: String,
    kind: String,
    source_id: String,
    canonical_bytes_hex: Option<String>,
    prior_publication_hex: Option<String>,
    mutation_point: String,
    expected_stage: String,
    expected_error: String,
    regen_command: String,
    evidence_pointer: String,
}

#[test]
fn policy_set_keeps_digest_alias() {
    let recovery = sample_recovery();
    let policy_set = recovery.live_policy_set_v1(9);
    let digest = policy_set.digest().expect("policy-set digest");
    let member = policy_set.members[0];

    assert_eq!(member.policy_digest(), recovery.bucket_policy_id);
    assert_eq!(
        member.policy_generation,
        u64::from(recovery.bucket_policy_generation)
    );
    policy_set
        .verify_member(member.policy_generation, member.policy_digest(), 9)
        .expect("member lookup");

    let leaf = ShardRootLeafV1::new(
        3,
        recovery.state_root.into_bytes(),
        21,
        7,
        [0x42; 32],
        digest,
        33,
        44,
        0,
    );
    leaf.verify_policy_member(
        &policy_set,
        member.policy_generation,
        member.policy_digest(),
        9,
    )
    .expect("leaf policy binding");
}

#[test]
fn leaf_rejects_policy_digest_drift() {
    let recovery = sample_recovery();
    let policy_set = recovery.live_policy_set_v1(9);
    let mut leaf = sample_leaf_case().1;
    leaf.policy_set_digest[0] ^= 0x01;

    let err = leaf
        .verify_policy_member(
            &policy_set,
            u64::from(recovery.bucket_policy_generation),
            recovery.bucket_policy_id,
            9,
        )
        .expect_err("policy-set digest drift must reject");
    assert_eq!(err, ProofChkErr::PublicationPolicyMix);
}

#[test]
fn leaf_manifest_matches_contract() {
    let manifest = load_leaf_manifest();
    assert_eq!(manifest, build_leaf_manifest());
}

#[test]
fn publication_manifest_matches_contract() {
    let manifest = load_publication_manifest();
    assert_eq!(manifest, build_publication_manifest());
}

#[test]
fn leaf_vectors_prove_codec_contract() {
    let manifest = build_leaf_manifest();

    for case in manifest.golden {
        match case.id.as_str() {
            "SRL-G-001" | "SRL-G-003" => {
                let leaf = decode_leaf(
                    case.canonical_bytes_hex
                        .as_deref()
                        .expect("leaf canonical bytes"),
                );
                check_shard_root_leaf_v1(&leaf).expect("leaf contract");
                assert_eq!(
                    case.expected_digest_hex,
                    Some(to_hex(&leaf.digest().expect("leaf digest")))
                );
                if case.id == "SRL-G-003" {
                    assert_eq!(leaf.transition_flags, 0b001);
                }
            }
            "SRL-G-002" => {
                let prior = decode_leaf(case.prior_leaf_hex.as_deref().expect("prior leaf"));
                let next = decode_leaf(case.canonical_bytes_hex.as_deref().expect("next leaf"));
                assert_leaf_successor_ok(prior, next);
                assert_eq!(
                    case.expected_digest_hex,
                    Some(to_hex(&next.digest().expect("leaf digest")))
                );
            }
            "SRL-G-004" => {
                let leaf = decode_leaf(
                    case.canonical_bytes_hex
                        .as_deref()
                        .expect("leaf canonical bytes"),
                );
                let bytes = leaf.canonical_bytes().expect("leaf bytes");
                let redecoded = ShardRootLeafV1::from_canon(&bytes).expect("redecode leaf");
                assert_eq!(bytes, redecoded.canonical_bytes().expect("leaf reencode"));
                assert_eq!(
                    case.expected_digest_hex,
                    Some(to_hex(&redecoded.digest().expect("leaf digest")))
                );
            }
            other => panic!("unexpected leaf golden case {other}"),
        }
    }
}

#[test]
fn leaf_tamper_vectors_fail_closed() {
    let manifest = build_leaf_manifest();
    let expected_route_table_digest = sample_leaf_case().1.route_table_digest;

    for case in manifest.tamper {
        match case.id.as_str() {
            "SRL-T-001" => {
                let err = ShardRootLeafV1::from_canon(&decode_hex(
                    case.canonical_bytes_hex
                        .as_deref()
                        .expect("mutated leaf bytes"),
                ))
                .expect_err("reserved flags must reject");
                assert_eq!(err_name(&err), case.expected_error);
            }
            "SRL-T-002" => {
                let leaf = decode_leaf(
                    case.canonical_bytes_hex
                        .as_deref()
                        .expect("mutated leaf bytes"),
                );
                let err = leaf
                    .check_route_binding_v1(expected_route_table_digest)
                    .expect_err("route drift must reject");
                assert_eq!(err_name(&err), case.expected_error);
            }
            "SRL-T-003" | "SRL-T-004" | "SRL-T-005" => {
                let prior = decode_leaf(case.prior_leaf_hex.as_deref().expect("prior leaf"));
                let next = decode_leaf(
                    case.canonical_bytes_hex
                        .as_deref()
                        .expect("mutated leaf bytes"),
                );
                let err = assert_leaf_successor_err(prior, next);
                assert_eq!(err_name(&err), case.expected_error);
            }
            "SRL-T-006" => {
                let leaf = decode_leaf(case.canonical_bytes_hex.as_deref().expect("leaf bytes"));
                let live_digest = to_hex(&leaf.digest().expect("leaf digest"));
                assert_ne!(
                    case.claimed_digest_hex,
                    Some(live_digest),
                    "claimed digest must drift for {}",
                    case.id
                );
            }
            other => panic!("unexpected leaf tamper case {other}"),
        }
    }
}

#[test]
fn publication_vectors_prove_codec_contract() {
    let manifest = build_publication_manifest();

    for case in manifest.golden {
        match case.id.as_str() {
            "CPP-G-001" => {
                let publication = decode_publication(
                    case.canonical_bytes_hex
                        .as_deref()
                        .expect("publication bytes"),
                );
                check_checkpoint_publication_contract_v1(&publication)
                    .expect("publication contract");
                assert_eq!(
                    case.expected_digest_hex,
                    Some(to_hex(&publication.digest().expect("publication digest")))
                );
            }
            "CPP-G-002" => {
                let prior = decode_publication(
                    case.prior_publication_hex
                        .as_deref()
                        .expect("prior publication bytes"),
                );
                let next = decode_publication(
                    case.canonical_bytes_hex
                        .as_deref()
                        .expect("next publication bytes"),
                );
                next.check_monotonic_successor_v1(&prior)
                    .expect("carry-forward publication");
                assert_eq!(
                    next.shard_leaves[0]
                        .canonical_bytes()
                        .expect("next leaf bytes"),
                    prior.shard_leaves[0]
                        .canonical_bytes()
                        .expect("prior leaf bytes")
                );
                assert_eq!(
                    case.expected_digest_hex,
                    Some(to_hex(&next.digest().expect("publication digest")))
                );
            }
            "CPP-G-003" => {
                let prior = decode_publication(
                    case.prior_publication_hex
                        .as_deref()
                        .expect("prior publication bytes"),
                );
                let next = decode_publication(
                    case.canonical_bytes_hex
                        .as_deref()
                        .expect("next publication bytes"),
                );
                next.check_monotonic_successor_v1(&prior)
                    .expect("changed-shard publication");
                assert_eq!(
                    next.shard_leaves[0]
                        .canonical_bytes()
                        .expect("next leaf bytes"),
                    prior.shard_leaves[0]
                        .canonical_bytes()
                        .expect("prior leaf bytes")
                );
                assert_ne!(
                    next.shard_leaves[1]
                        .canonical_bytes()
                        .expect("next leaf bytes"),
                    prior.shard_leaves[1]
                        .canonical_bytes()
                        .expect("prior leaf bytes")
                );
            }
            "CPP-G-004" => {
                let prior = decode_publication(
                    case.prior_publication_hex
                        .as_deref()
                        .expect("prior publication bytes"),
                );
                let next = decode_publication(
                    case.canonical_bytes_hex
                        .as_deref()
                        .expect("next publication bytes"),
                );
                next.check_monotonic_successor_v1(&prior)
                    .expect("migration publication");
                assert_ne!(next.route_table_digest, prior.route_table_digest);
                assert!(next
                    .shard_leaves
                    .iter()
                    .all(|leaf| leaf.route_table_digest == next.route_table_digest));
            }
            "CPP-G-005" => {
                let prior = decode_publication(
                    case.prior_publication_hex
                        .as_deref()
                        .expect("prior publication bytes"),
                );
                let next = decode_publication(
                    case.canonical_bytes_hex
                        .as_deref()
                        .expect("next publication bytes"),
                );
                next.check_prior_root_v1(prior.public_root_v1().expect("prior root"))
                    .expect("prior root chain");
            }
            other => panic!("unexpected publication golden case {other}"),
        }
    }
}

#[test]
fn publication_tamper_vectors_fail_closed() {
    let manifest = build_publication_manifest();

    for case in manifest.tamper {
        match case.id.as_str() {
            "CPP-T-001" | "CPP-T-003" | "CPP-T-004" | "CPP-T-006" => {
                let err = CheckpointPublicationV1::from_canon(&decode_hex(
                    case.canonical_bytes_hex
                        .as_deref()
                        .expect("tampered publication bytes"),
                ))
                .expect_err("tampered publication must reject");
                assert_eq!(err_name(&err), case.expected_error);
            }
            "CPP-T-002" | "CPP-T-007" => {
                let prior = decode_publication(
                    case.prior_publication_hex
                        .as_deref()
                        .expect("prior publication bytes"),
                );
                let next = decode_publication(
                    case.canonical_bytes_hex
                        .as_deref()
                        .expect("tampered publication bytes"),
                );
                let err = next
                    .check_monotonic_successor_v1(&prior)
                    .expect_err("tampered successor must reject");
                assert_eq!(err_name(&err), case.expected_error);
            }
            "CPP-T-005" => {
                let prior = decode_publication(
                    case.prior_publication_hex
                        .as_deref()
                        .expect("prior publication bytes"),
                );
                let next = decode_publication(
                    case.canonical_bytes_hex
                        .as_deref()
                        .expect("tampered publication bytes"),
                );
                let err = next
                    .check_prior_root_v1(prior.public_root_v1().expect("prior root"))
                    .expect_err("wrong prior root must reject");
                assert_eq!(err_name(&err), case.expected_error);
            }
            other => panic!("unexpected publication tamper case {other}"),
        }
    }
}

#[test]
fn publication_rejects_wrong_prior_root() {
    let publication = sample_publication();
    let err = publication
        .check_prior_root_v1(SettlementStateRoot::settlement_v1([0xEE; 32]))
        .expect_err("wrong prior root must reject");
    assert_eq!(err, ProofChkErr::PublicationPriorRootMix);
}

#[test]
fn monotonic_publication_rejects_stale_tuple() {
    let prev = sample_publication();
    let mut next = sample_publication();
    next.publication_checkpoint = prev.publication_checkpoint + 1;
    next.shard_leaves[1].shard_root = [0xCC; 32];

    let err = next
        .check_monotonic_successor_v1(&prev)
        .expect_err("changed leaf without tuple advance must reject");
    assert_eq!(err, ProofChkErr::PublicationMonotonicityMix);

    next.shard_leaves[1].local_sequence += 1;
    next.check_monotonic_successor_v1(&prev)
        .expect("advanced tuple stays monotonic");
}

#[test]
fn publication_keeps_carry_leaf_bytes() {
    let prev = sample_publication();
    let next = carry_forward_publication(&prev);

    next.check_monotonic_successor_v1(&prev)
        .expect("carry-forward and changed leaf coexist");
    assert_eq!(
        next.shard_leaves[0].canonical_bytes().expect("leaf bytes"),
        prev.shard_leaves[0].canonical_bytes().expect("leaf bytes")
    );
}

#[test]
fn publication_rejects_missing_route_shard() {
    let prior = sample_publication();
    let missing = missing_same_route_publication(&prior);

    let err = missing
        .check_monotonic_successor_v1(&prior)
        .expect_err("same-route missing shard must reject");
    assert_eq!(err, ProofChkErr::PublicationCountMix);
}

#[test]
fn publication_route_accepts() {
    let publication = sample_publication();

    check_publication_route_v1(
        &publication,
        &PublicationRouteSnapshotV1::new(7, publication.route_table_digest, 100, vec![3, 9]),
    )
    .expect("route snapshot must accept");
}

#[test]
fn publication_route_rejects_gap() {
    let publication = sample_publication();
    let err = check_publication_route_v1(
        &publication,
        &PublicationRouteSnapshotV1::new(7, publication.route_table_digest, 100, vec![3]),
    )
    .expect_err("missing route shard must reject");

    assert_eq!(err, ProofChkErr::PublicationCountMix);
}

#[test]
fn publication_route_rejects_stale() {
    let publication = sample_publication();
    let err = check_publication_route_v1(
        &publication,
        &PublicationRouteSnapshotV1::new(7, publication.route_table_digest, 102, vec![3, 9]),
    )
    .expect_err("stale activation checkpoint must reject");

    assert_eq!(err, ProofChkErr::PublicationCheckpointMix);
}

#[ignore]
#[test]
fn print_shard_leaf_manifest() {
    let json = JsonCodec
        .serialize_pretty(&build_leaf_manifest())
        .expect("encode leaf manifest");
    println!("{}", String::from_utf8(json).expect("leaf manifest utf8"));
}

#[ignore]
#[test]
fn print_checkpoint_publication_manifest_json() {
    let json = JsonCodec
        .serialize_pretty(&build_publication_manifest())
        .expect("encode publication manifest");
    println!(
        "{}",
        String::from_utf8(json).expect("publication manifest utf8")
    );
}

fn build_leaf_manifest() -> LeafManifest {
    let (policy_set, bridge) = sample_leaf_case();
    let policy_digest = policy_set.digest().expect("policy-set digest");
    let bridge_bytes = bridge.canonical_bytes().expect("bridge bytes");
    let bridge_digest = bridge.digest().expect("bridge digest");

    let same_generation = ShardRootLeafV1::new(
        bridge.shard_id,
        [0x1C; 32],
        bridge.shard_epoch + 1,
        bridge.routing_generation,
        bridge.route_table_digest,
        policy_digest,
        bridge.journal_checkpoint + 1,
        bridge.local_sequence + 1,
        0,
    );
    let transition = ShardRootLeafV1::new(
        bridge.shard_id,
        [0x1D; 32],
        bridge.shard_epoch + 2,
        bridge.routing_generation,
        bridge.route_table_digest,
        policy_digest,
        bridge.journal_checkpoint + 2,
        bridge.local_sequence + 2,
        0b001,
    );
    let mut invalid_flags = bridge_bytes.clone();
    invalid_flags[135] = 0x08;
    let mut route_drift = bridge_bytes.clone();
    route_drift[52] ^= 0x01;
    let stale_epoch = ShardRootLeafV1::new(
        bridge.shard_id,
        [0x1E; 32],
        bridge.shard_epoch,
        bridge.routing_generation,
        bridge.route_table_digest,
        policy_digest,
        same_generation.journal_checkpoint,
        same_generation.local_sequence,
        0,
    );
    let stale_journal = ShardRootLeafV1::new(
        bridge.shard_id,
        [0x1F; 32],
        same_generation.shard_epoch,
        same_generation.routing_generation,
        same_generation.route_table_digest,
        policy_digest,
        same_generation.journal_checkpoint - 1,
        same_generation.local_sequence + 1,
        0,
    );
    let stale_sequence = ShardRootLeafV1::new(
        bridge.shard_id,
        [0x20; 32],
        same_generation.shard_epoch,
        same_generation.routing_generation,
        same_generation.route_table_digest,
        policy_digest,
        same_generation.journal_checkpoint,
        same_generation.local_sequence - 1,
        0,
    );
    let mut wrong_digest = bridge_digest;
    wrong_digest[0] ^= 0x01;

    LeafManifest {
        version: 1,
        golden: vec![
            LeafGoldenCase {
                id: "SRL-G-001".to_string(),
                kind: "bridge".to_string(),
                canonical_bytes_hex: Some(to_hex(&bridge_bytes)),
                expected_digest_hex: Some(to_hex(&bridge_digest)),
                prior_leaf_hex: None,
                prior_digest_hex: None,
                regen_command: LEAF_REGEN_CMD.to_string(),
                evidence_pointer: LEAF_EVIDENCE_PTR.to_string(),
            },
            LeafGoldenCase {
                id: "SRL-G-002".to_string(),
                kind: "same_generation_changed".to_string(),
                canonical_bytes_hex: Some(to_hex(
                    &same_generation
                        .canonical_bytes()
                        .expect("same-generation bytes"),
                )),
                expected_digest_hex: Some(to_hex(
                    &same_generation.digest().expect("same-generation digest"),
                )),
                prior_leaf_hex: Some(to_hex(&bridge_bytes)),
                prior_digest_hex: Some(to_hex(&bridge_digest)),
                regen_command: LEAF_REGEN_CMD.to_string(),
                evidence_pointer: LEAF_EVIDENCE_PTR.to_string(),
            },
            LeafGoldenCase {
                id: "SRL-G-003".to_string(),
                kind: "transition_state".to_string(),
                canonical_bytes_hex: Some(to_hex(
                    &transition.canonical_bytes().expect("transition bytes"),
                )),
                expected_digest_hex: Some(to_hex(&transition.digest().expect("transition digest"))),
                prior_leaf_hex: None,
                prior_digest_hex: None,
                regen_command: LEAF_REGEN_CMD.to_string(),
                evidence_pointer: LEAF_EVIDENCE_PTR.to_string(),
            },
            LeafGoldenCase {
                id: "SRL-G-004".to_string(),
                kind: "reencode".to_string(),
                canonical_bytes_hex: Some(to_hex(
                    &same_generation
                        .canonical_bytes()
                        .expect("same-generation bytes"),
                )),
                expected_digest_hex: Some(to_hex(
                    &same_generation.digest().expect("same-generation digest"),
                )),
                prior_leaf_hex: None,
                prior_digest_hex: None,
                regen_command: LEAF_REGEN_CMD.to_string(),
                evidence_pointer: LEAF_EVIDENCE_PTR.to_string(),
            },
        ],
        tamper: vec![
            LeafTamperCase {
                id: "SRL-T-001".to_string(),
                kind: "reserved_transition_bits".to_string(),
                source_id: "SRL-G-001".to_string(),
                canonical_bytes_hex: Some(to_hex(&invalid_flags)),
                prior_leaf_hex: None,
                claimed_digest_hex: None,
                mutation_point: "transition_flags".to_string(),
                expected_stage: "parser_reject".to_string(),
                expected_error: "PublicationFlagMix".to_string(),
                regen_command: LEAF_REGEN_CMD.to_string(),
                evidence_pointer: LEAF_EVIDENCE_PTR.to_string(),
            },
            LeafTamperCase {
                id: "SRL-T-002".to_string(),
                kind: "stale_route_table_digest".to_string(),
                source_id: "SRL-G-001".to_string(),
                canonical_bytes_hex: Some(to_hex(&route_drift)),
                prior_leaf_hex: None,
                claimed_digest_hex: None,
                mutation_point: "route_table_digest".to_string(),
                expected_stage: "route_binding_reject".to_string(),
                expected_error: "PublicationRouteMix".to_string(),
                regen_command: LEAF_REGEN_CMD.to_string(),
                evidence_pointer: LEAF_EVIDENCE_PTR.to_string(),
            },
            LeafTamperCase {
                id: "SRL-T-003".to_string(),
                kind: "stale_shard_epoch".to_string(),
                source_id: "SRL-G-002".to_string(),
                canonical_bytes_hex: Some(to_hex(
                    &stale_epoch.canonical_bytes().expect("stale epoch bytes"),
                )),
                prior_leaf_hex: Some(to_hex(
                    &same_generation
                        .canonical_bytes()
                        .expect("same-generation bytes"),
                )),
                claimed_digest_hex: None,
                mutation_point: "shard_epoch".to_string(),
                expected_stage: "monotonicity_reject".to_string(),
                expected_error: "PublicationMonotonicityMix".to_string(),
                regen_command: LEAF_REGEN_CMD.to_string(),
                evidence_pointer: LEAF_EVIDENCE_PTR.to_string(),
            },
            LeafTamperCase {
                id: "SRL-T-004".to_string(),
                kind: "decreasing_journal_checkpoint".to_string(),
                source_id: "SRL-G-002".to_string(),
                canonical_bytes_hex: Some(to_hex(
                    &stale_journal
                        .canonical_bytes()
                        .expect("stale journal bytes"),
                )),
                prior_leaf_hex: Some(to_hex(
                    &same_generation
                        .canonical_bytes()
                        .expect("same-generation bytes"),
                )),
                claimed_digest_hex: None,
                mutation_point: "journal_checkpoint".to_string(),
                expected_stage: "monotonicity_reject".to_string(),
                expected_error: "PublicationMonotonicityMix".to_string(),
                regen_command: LEAF_REGEN_CMD.to_string(),
                evidence_pointer: LEAF_EVIDENCE_PTR.to_string(),
            },
            LeafTamperCase {
                id: "SRL-T-005".to_string(),
                kind: "decreasing_local_sequence".to_string(),
                source_id: "SRL-G-002".to_string(),
                canonical_bytes_hex: Some(to_hex(
                    &stale_sequence
                        .canonical_bytes()
                        .expect("stale sequence bytes"),
                )),
                prior_leaf_hex: Some(to_hex(
                    &same_generation
                        .canonical_bytes()
                        .expect("same-generation bytes"),
                )),
                claimed_digest_hex: None,
                mutation_point: "local_sequence".to_string(),
                expected_stage: "monotonicity_reject".to_string(),
                expected_error: "PublicationMonotonicityMix".to_string(),
                regen_command: LEAF_REGEN_CMD.to_string(),
                evidence_pointer: LEAF_EVIDENCE_PTR.to_string(),
            },
            LeafTamperCase {
                id: "SRL-T-006".to_string(),
                kind: "digest_mismatch".to_string(),
                source_id: "SRL-G-001".to_string(),
                canonical_bytes_hex: Some(to_hex(&bridge_bytes)),
                prior_leaf_hex: None,
                claimed_digest_hex: Some(to_hex(&wrong_digest)),
                mutation_point: "claimed_digest_hex".to_string(),
                expected_stage: "digest_reject".to_string(),
                expected_error: "digest_mismatch".to_string(),
                regen_command: LEAF_REGEN_CMD.to_string(),
                evidence_pointer: LEAF_EVIDENCE_PTR.to_string(),
            },
        ],
    }
}

fn build_publication_manifest() -> PublicationManifest {
    let bridge = sample_publication();
    let bridge_bytes = bridge.canonical_bytes().expect("bridge publication bytes");
    let bridge_digest = bridge.digest().expect("bridge publication digest");
    let carry = carry_forward_publication(&bridge);
    let carry_bytes = carry.canonical_bytes().expect("carry publication bytes");
    let carry_digest = carry.digest().expect("carry publication digest");
    let migration = route_migration_publication(&carry);
    let migration_bytes = migration
        .canonical_bytes()
        .expect("migration publication bytes");
    let migration_digest = migration.digest().expect("migration publication digest");

    let mut reversed = bridge.shard_leaves.clone();
    reversed.reverse();
    let order_bytes = manual_publication_bytes(
        bridge.root_generation as u8,
        bridge.publication_mode as u8,
        bridge.publication_checkpoint,
        bridge.route_table_digest,
        bridge.prior_public_root,
        &reversed,
    );

    let missing = missing_same_route_publication(&bridge);
    let duplicate = duplicate_shard_publication(&bridge);
    let duplicate_bytes = manual_publication_bytes(
        duplicate.root_generation as u8,
        duplicate.publication_mode as u8,
        duplicate.publication_checkpoint,
        duplicate.route_table_digest,
        duplicate.prior_public_root,
        &duplicate.shard_leaves,
    );
    let mut route_drift = bridge_bytes.clone();
    route_drift[10] ^= 0x01;
    let wrong_prior = wrong_prior_root_publication(&bridge);
    let wrong_prior_bytes = wrong_prior
        .canonical_bytes()
        .expect("wrong prior publication bytes");
    let mut generation_bytes = manual_publication_bytes(
        0x00,
        bridge.publication_mode as u8,
        bridge.publication_checkpoint,
        bridge.route_table_digest,
        bridge.prior_public_root,
        &bridge.shard_leaves,
    );
    generation_bytes[0] = 0x00;
    let carried_leaf_drift = carried_leaf_drift_publication(&bridge);
    let carried_leaf_drift_bytes = carried_leaf_drift
        .canonical_bytes()
        .expect("carried leaf drift bytes");

    PublicationManifest {
        version: 1,
        golden: vec![
            PublicationGoldenCase {
                id: "CPP-G-001".to_string(),
                kind: "bridge".to_string(),
                canonical_bytes_hex: Some(to_hex(&bridge_bytes)),
                expected_digest_hex: Some(to_hex(&bridge_digest)),
                prior_publication_hex: None,
                prior_digest_hex: None,
                regen_command: PUB_REGEN_CMD.to_string(),
                evidence_pointer: PUB_EVIDENCE_PTR.to_string(),
            },
            PublicationGoldenCase {
                id: "CPP-G-002".to_string(),
                kind: "carry_forward".to_string(),
                canonical_bytes_hex: Some(to_hex(&carry_bytes)),
                expected_digest_hex: Some(to_hex(&carry_digest)),
                prior_publication_hex: Some(to_hex(&bridge_bytes)),
                prior_digest_hex: Some(to_hex(&bridge_digest)),
                regen_command: PUB_REGEN_CMD.to_string(),
                evidence_pointer: PUB_EVIDENCE_PTR.to_string(),
            },
            PublicationGoldenCase {
                id: "CPP-G-003".to_string(),
                kind: "changed_one_shard".to_string(),
                canonical_bytes_hex: Some(to_hex(&carry_bytes)),
                expected_digest_hex: Some(to_hex(&carry_digest)),
                prior_publication_hex: Some(to_hex(&bridge_bytes)),
                prior_digest_hex: Some(to_hex(&bridge_digest)),
                regen_command: PUB_REGEN_CMD.to_string(),
                evidence_pointer: PUB_EVIDENCE_PTR.to_string(),
            },
            PublicationGoldenCase {
                id: "CPP-G-004".to_string(),
                kind: "route_generation_transition".to_string(),
                canonical_bytes_hex: Some(to_hex(&migration_bytes)),
                expected_digest_hex: Some(to_hex(&migration_digest)),
                prior_publication_hex: Some(to_hex(&carry_bytes)),
                prior_digest_hex: Some(to_hex(&carry_digest)),
                regen_command: PUB_REGEN_CMD.to_string(),
                evidence_pointer: PUB_EVIDENCE_PTR.to_string(),
            },
            PublicationGoldenCase {
                id: "CPP-G-005".to_string(),
                kind: "prior_root_chain".to_string(),
                canonical_bytes_hex: Some(to_hex(&carry_bytes)),
                expected_digest_hex: Some(to_hex(&carry_digest)),
                prior_publication_hex: Some(to_hex(&bridge_bytes)),
                prior_digest_hex: Some(to_hex(&bridge_digest)),
                regen_command: PUB_REGEN_CMD.to_string(),
                evidence_pointer: PUB_EVIDENCE_PTR.to_string(),
            },
        ],
        tamper: vec![
            PublicationTamperCase {
                id: "CPP-T-001".to_string(),
                kind: "reordered_shard_leaves".to_string(),
                source_id: "CPP-G-001".to_string(),
                canonical_bytes_hex: Some(to_hex(&order_bytes)),
                prior_publication_hex: None,
                mutation_point: "shard_leaves order".to_string(),
                expected_stage: "parser_reject".to_string(),
                expected_error: "PublicationOrderMix".to_string(),
                regen_command: PUB_REGEN_CMD.to_string(),
                evidence_pointer: PUB_EVIDENCE_PTR.to_string(),
            },
            PublicationTamperCase {
                id: "CPP-T-002".to_string(),
                kind: "missing_active_shard_leaf".to_string(),
                source_id: "CPP-G-002".to_string(),
                canonical_bytes_hex: Some(to_hex(
                    &missing
                        .canonical_bytes()
                        .expect("missing publication bytes"),
                )),
                prior_publication_hex: Some(to_hex(&bridge_bytes)),
                mutation_point: "shard_leaves missing shard_id=3".to_string(),
                expected_stage: "monotonicity_reject".to_string(),
                expected_error: "PublicationCountMix".to_string(),
                regen_command: PUB_REGEN_CMD.to_string(),
                evidence_pointer: PUB_EVIDENCE_PTR.to_string(),
            },
            PublicationTamperCase {
                id: "CPP-T-003".to_string(),
                kind: "duplicate_shard_leaf".to_string(),
                source_id: "CPP-G-001".to_string(),
                canonical_bytes_hex: Some(to_hex(&duplicate_bytes)),
                prior_publication_hex: None,
                mutation_point: "shard_leaves[1].shard_id".to_string(),
                expected_stage: "parser_reject".to_string(),
                expected_error: "PublicationDupShard".to_string(),
                regen_command: PUB_REGEN_CMD.to_string(),
                evidence_pointer: PUB_EVIDENCE_PTR.to_string(),
            },
            PublicationTamperCase {
                id: "CPP-T-004".to_string(),
                kind: "route_table_digest_mismatch".to_string(),
                source_id: "CPP-G-001".to_string(),
                canonical_bytes_hex: Some(to_hex(&route_drift)),
                prior_publication_hex: None,
                mutation_point: "route_table_digest".to_string(),
                expected_stage: "parser_reject".to_string(),
                expected_error: "PublicationRouteMix".to_string(),
                regen_command: PUB_REGEN_CMD.to_string(),
                evidence_pointer: PUB_EVIDENCE_PTR.to_string(),
            },
            PublicationTamperCase {
                id: "CPP-T-005".to_string(),
                kind: "wrong_prior_public_root".to_string(),
                source_id: "CPP-G-002".to_string(),
                canonical_bytes_hex: Some(to_hex(&wrong_prior_bytes)),
                prior_publication_hex: Some(to_hex(&bridge_bytes)),
                mutation_point: "prior_public_root".to_string(),
                expected_stage: "prior_root_reject".to_string(),
                expected_error: "PublicationPriorRootMix".to_string(),
                regen_command: PUB_REGEN_CMD.to_string(),
                evidence_pointer: PUB_EVIDENCE_PTR.to_string(),
            },
            PublicationTamperCase {
                id: "CPP-T-006".to_string(),
                kind: "invalid_root_generation_tag".to_string(),
                source_id: "CPP-G-001".to_string(),
                canonical_bytes_hex: Some(to_hex(&generation_bytes)),
                prior_publication_hex: None,
                mutation_point: "root_generation_tag".to_string(),
                expected_stage: "parser_reject".to_string(),
                expected_error: "PublicationRootGenerationMix".to_string(),
                regen_command: PUB_REGEN_CMD.to_string(),
                evidence_pointer: PUB_EVIDENCE_PTR.to_string(),
            },
            PublicationTamperCase {
                id: "CPP-T-007".to_string(),
                kind: "carried_forward_leaf_byte_mutation".to_string(),
                source_id: "CPP-G-002".to_string(),
                canonical_bytes_hex: Some(to_hex(&carried_leaf_drift_bytes)),
                prior_publication_hex: Some(to_hex(&bridge_bytes)),
                mutation_point: "shard_leaves[0].shard_root[0]".to_string(),
                expected_stage: "monotonicity_reject".to_string(),
                expected_error: "PublicationMonotonicityMix".to_string(),
                regen_command: PUB_REGEN_CMD.to_string(),
                evidence_pointer: PUB_EVIDENCE_PTR.to_string(),
            },
        ],
    }
}

fn sample_recovery() -> SettlementRecoveryState {
    SettlementRecoveryState::new(
        33,
        SettlementStateRoot::settlement_v1([0x19; 32]),
        1,
        1,
        7,
        [0x2A; 32],
        [0x3B; 32],
    )
}

fn sample_leaf_case() -> (PolicySetCommitmentV1, ShardRootLeafV1) {
    let recovery = sample_recovery();
    let policy_set = recovery.live_policy_set_v1(9);
    let leaf = ShardRootLeafV1::new(
        3,
        recovery.state_root.into_bytes(),
        21,
        7,
        [0x42; 32],
        policy_set.digest().expect("policy-set digest"),
        33,
        44,
        0,
    );
    leaf.verify_policy_member(
        &policy_set,
        u64::from(recovery.bucket_policy_generation),
        recovery.bucket_policy_id,
        9,
    )
    .expect("leaf policy binding");
    (policy_set, leaf)
}

fn sample_publication() -> CheckpointPublicationV1 {
    let (_, leaf_a) = sample_leaf_case();
    let leaf_b = sample_leaf_b([0x1B; 32], 31, 7, [0x42; 32], 41, 52);

    CheckpointPublicationV1::new(
        RootGenerationTagV1::RootGeneration1,
        PublicationModeTagV1::CheckpointWindow,
        101,
        [0x42; 32],
        SettlementStateRoot::settlement_v1([0x11; 32]),
        vec![leaf_a, leaf_b],
    )
}

fn carry_forward_publication(prev: &CheckpointPublicationV1) -> CheckpointPublicationV1 {
    let unchanged = prev.shard_leaves[0];
    let changed = sample_leaf_b([0x2C; 32], 32, 7, prev.route_table_digest, 42, 53);

    CheckpointPublicationV1::new(
        RootGenerationTagV1::RootGeneration1,
        PublicationModeTagV1::CheckpointWindow,
        prev.publication_checkpoint + 1,
        prev.route_table_digest,
        prev.public_root_v1().expect("prior public root"),
        vec![unchanged, changed],
    )
}

fn route_migration_publication(prev: &CheckpointPublicationV1) -> CheckpointPublicationV1 {
    let (_, base_leaf) = sample_leaf_case();
    let migrated_a = ShardRootLeafV1::new(
        base_leaf.shard_id,
        [0x2D; 32],
        23,
        8,
        [0x52; 32],
        base_leaf.policy_set_digest,
        35,
        46,
        0,
    );
    let migrated_b = sample_leaf_b([0x2E; 32], 33, 8, [0x52; 32], 43, 54);

    CheckpointPublicationV1::new(
        RootGenerationTagV1::RootGeneration1,
        PublicationModeTagV1::CheckpointWindow,
        prev.publication_checkpoint + 1,
        [0x52; 32],
        prev.public_root_v1().expect("prior public root"),
        vec![migrated_a, migrated_b],
    )
}

fn missing_same_route_publication(prev: &CheckpointPublicationV1) -> CheckpointPublicationV1 {
    let changed = sample_leaf_b([0x2C; 32], 32, 7, prev.route_table_digest, 42, 53);
    CheckpointPublicationV1::new(
        RootGenerationTagV1::RootGeneration1,
        PublicationModeTagV1::CheckpointWindow,
        prev.publication_checkpoint + 1,
        prev.route_table_digest,
        prev.public_root_v1().expect("prior public root"),
        vec![changed],
    )
}

fn duplicate_shard_publication(prev: &CheckpointPublicationV1) -> CheckpointPublicationV1 {
    let left = prev.shard_leaves[0];
    let mut dup = prev.shard_leaves[1];
    dup.shard_id = left.shard_id;
    CheckpointPublicationV1::new(
        RootGenerationTagV1::RootGeneration1,
        PublicationModeTagV1::CheckpointWindow,
        prev.publication_checkpoint,
        prev.route_table_digest,
        prev.prior_public_root,
        vec![left, dup],
    )
}

fn wrong_prior_root_publication(prev: &CheckpointPublicationV1) -> CheckpointPublicationV1 {
    let mut next = carry_forward_publication(prev);
    next.prior_public_root = SettlementStateRoot::settlement_v1([0xEE; 32]);
    next
}

fn carried_leaf_drift_publication(prev: &CheckpointPublicationV1) -> CheckpointPublicationV1 {
    let mut next = carry_forward_publication(prev);
    next.shard_leaves[0].shard_root[0] ^= 0x01;
    next
}

fn sample_leaf_b(
    shard_root: [u8; 32],
    shard_epoch: u64,
    routing_generation: u64,
    route_table_digest: [u8; 32],
    journal_checkpoint: u64,
    local_sequence: u64,
) -> ShardRootLeafV1 {
    let leaf_b_policy = PolicySetCommitmentV1::singleton_live(8, [0x2B; 32], 11);
    ShardRootLeafV1::new(
        9,
        shard_root,
        shard_epoch,
        routing_generation,
        route_table_digest,
        leaf_b_policy.digest().expect("leaf-b policy digest"),
        journal_checkpoint,
        local_sequence,
        0,
    )
}

fn manual_publication_bytes(
    root_generation: u8,
    publication_mode: u8,
    publication_checkpoint: u64,
    route_table_digest: [u8; 32],
    prior_public_root: SettlementStateRoot,
    shard_leaves: &[ShardRootLeafV1],
) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(root_generation);
    out.push(publication_mode);
    out.extend_from_slice(&publication_checkpoint.to_be_bytes());
    out.extend_from_slice(&route_table_digest);
    out.push(prior_public_root.generation_version());
    out.extend_from_slice(prior_public_root.as_bytes());
    out.extend_from_slice(&(shard_leaves.len() as u32).to_be_bytes());
    for leaf in shard_leaves {
        out.extend_from_slice(
            &leaf
                .canonical_bytes()
                .expect("manual publication uses valid leaves"),
        );
    }
    out
}

fn assert_leaf_successor_ok(prior: ShardRootLeafV1, next: ShardRootLeafV1) {
    let prev_pub = leaf_publication(101, prior);
    let next_pub = leaf_publication(102, next);
    prev_pub
        .check_contract_v1()
        .expect("prior leaf publication");
    next_pub
        .check_monotonic_successor_v1(&prev_pub)
        .expect("leaf successor");
}

fn assert_leaf_successor_err(prior: ShardRootLeafV1, next: ShardRootLeafV1) -> ProofChkErr {
    let prev_pub = leaf_publication(101, prior);
    let next_pub = leaf_publication(102, next);
    next_pub
        .check_monotonic_successor_v1(&prev_pub)
        .expect_err("leaf successor must reject")
}

fn leaf_publication(checkpoint: u64, leaf: ShardRootLeafV1) -> CheckpointPublicationV1 {
    CheckpointPublicationV1::new(
        RootGenerationTagV1::RootGeneration1,
        PublicationModeTagV1::CheckpointWindow,
        checkpoint,
        leaf.route_table_digest,
        SettlementStateRoot::settlement_v1([0x55; 32]),
        vec![leaf],
    )
}

fn err_name(err: &ProofChkErr) -> &'static str {
    match err {
        ProofChkErr::PublicationCountMix => "PublicationCountMix",
        ProofChkErr::PublicationDupShard => "PublicationDupShard",
        ProofChkErr::PublicationFlagMix => "PublicationFlagMix",
        ProofChkErr::PublicationMonotonicityMix => "PublicationMonotonicityMix",
        ProofChkErr::PublicationOrderMix => "PublicationOrderMix",
        ProofChkErr::PublicationPolicyMix => "PublicationPolicyMix",
        ProofChkErr::PublicationPriorRootMix => "PublicationPriorRootMix",
        ProofChkErr::PublicationRouteMix => "PublicationRouteMix",
        ProofChkErr::PublicationRootGenerationMix => "PublicationRootGenerationMix",
        other => panic!("unexpected error {other:?}"),
    }
}

fn load_leaf_manifest() -> LeafManifest {
    load_json(
        fixture_root()
            .join("shard_root_leaf_v1")
            .join("manifest.json"),
    )
}

fn load_publication_manifest() -> PublicationManifest {
    load_json(
        fixture_root()
            .join("checkpoint_publication_v1")
            .join("manifest.json"),
    )
}

fn load_json<T>(path: PathBuf) -> T
where
    T: DeserializeOwned,
{
    let bytes =
        io::read_file(&path).unwrap_or_else(|err| panic!("read {} failed: {err}", path.display()));
    JsonCodec
        .deserialize(&bytes)
        .unwrap_or_else(|err| panic!("decode {} failed: {err}", path.display()))
}

fn decode_leaf(hex: &str) -> ShardRootLeafV1 {
    let bytes = decode_hex(hex);
    ShardRootLeafV1::from_canon(&bytes).expect("leaf decode")
}

fn decode_publication(hex: &str) -> CheckpointPublicationV1 {
    let bytes = decode_hex(hex);
    CheckpointPublicationV1::from_canon(&bytes).expect("publication decode")
}

fn decode_hex(hex: &str) -> Vec<u8> {
    from_hex(hex).expect("hex bytes")
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hjmt_upgrade")
}
