mod test_batch_proof_support;

use std::{fs, path::PathBuf};

use serde::de::DeserializeOwned;
use z00z_storage::settlement::{
    batch_proof_transcript_domain_v1, BatchProofBlobV1, BatchProofFamilyTagV1, BatchProofLimits,
    DefinitionId, DeletionFactV1, HjmtProofFamily, InclusionOpeningV1, LeafFamilyTagV1,
    NodeDomainTagV1, NonExistenceOpeningV1, OpeningEntryV1, PriorProofContextV1, ProofBlob,
    ProofChkErr, RightClass, RightLeaf, RootGenerationTagV1, SerialId, SettlementLeaf,
    SettlementLeafFamily, SettlementPath, TerminalFamilyTagV1, TerminalId,
    HJMT_DEFAULT_COMMITMENT_VERSION,
};

use test_batch_proof_support::{
    bytes, live_prior_context_from_blob, sample_batch, sample_deletion_batch,
    sample_nonexistence_batch, sample_path, sample_policy, sample_voucher_nonexistence_batch,
    terminal_leaf,
};
use z00z_storage::settlement::{SettlementStore, StoreItem, StoreOp};
use z00z_utils::codec::{Codec, JsonCodec};

#[test]
fn test_v1_roundtrip_stays_deterministic() {
    let batch = sample_batch();
    batch.check_contract_v1().expect("contract");

    let bytes = batch.encode().expect("encode batch");
    let decoded =
        BatchProofBlobV1::decode_with_limits(&bytes, BatchProofLimits::v1()).expect("decode batch");

    assert_eq!(decoded, batch);
    assert_eq!(decoded.encode().expect("re-encode"), bytes);
    assert_eq!(
        decoded.header.transcript_domain,
        batch_proof_transcript_domain_v1()
    );
    assert_eq!(
        decoded.header.proof_family.into_live(),
        HjmtProofFamily::Inclusion
    );
}

#[test]
fn test_wire_names_match() {
    assert_eq!(
        LeafFamilyTagV1::from_live(SettlementLeafFamily::Terminal),
        LeafFamilyTagV1::Asset
    );
    assert_eq!(
        LeafFamilyTagV1::Asset.into_live(),
        SettlementLeafFamily::Terminal
    );
    assert_eq!(
        TerminalFamilyTagV1::from_live(SettlementLeafFamily::Right),
        TerminalFamilyTagV1::Right
    );
    assert_eq!(
        TerminalFamilyTagV1::Right.into_live(),
        SettlementLeafFamily::Right
    );
    assert_eq!(
        LeafFamilyTagV1::from_live(SettlementLeafFamily::Voucher),
        LeafFamilyTagV1::Voucher
    );
    assert_eq!(
        LeafFamilyTagV1::Voucher.into_live(),
        SettlementLeafFamily::Voucher
    );
    assert_eq!(
        TerminalFamilyTagV1::from_live(SettlementLeafFamily::Voucher),
        TerminalFamilyTagV1::Voucher
    );
    assert_eq!(
        TerminalFamilyTagV1::Voucher.into_live(),
        SettlementLeafFamily::Voucher
    );
    assert_eq!(
        BatchProofFamilyTagV1::from_live(HjmtProofFamily::NonExistence),
        BatchProofFamilyTagV1::NonExistence
    );
}

#[test]
fn test_rejects_bad_root_generation() {
    let mut batch = sample_batch();
    batch.header.root_generation = RootGenerationTagV1::RootGeneration0;

    let err = BatchProofBlobV1::decode(&batch.encode().expect("encode"))
        .expect_err("unsupported root generation must reject");
    assert_eq!(err, ProofChkErr::BatchRootGenerationMix);
}

#[test]
fn test_rejects_partial_shard_context() {
    let mut batch = sample_batch();
    batch.path_table[0].shard_id = Some(9);

    let err = BatchProofBlobV1::decode(&batch.encode().expect("encode"))
        .expect_err("partial shard context must reject");
    assert_eq!(err, ProofChkErr::BatchShardCtxMix);
}

#[test]
fn test_rejects_bad_ref_index() {
    let mut batch = sample_batch();
    batch.path_table[0].reference_index = 1;

    let err = BatchProofBlobV1::decode(&batch.encode().expect("encode"))
        .expect_err("reference index must stay in bounds");
    assert_eq!(err, ProofChkErr::BatchIndexMix);
}

#[test]
fn test_rejects_bad_witness_index() {
    let mut batch = sample_batch();
    batch.reference_table[0].witness_indexes = vec![1];

    let err = BatchProofBlobV1::decode(&batch.encode().expect("encode"))
        .expect_err("witness index must stay in bounds");
    assert_eq!(err, ProofChkErr::BatchIndexMix);
}

#[test]
fn test_v1_rejects_orphan_opening() {
    let mut batch = sample_batch();
    batch.opening_table.push(batch.opening_table[0].clone());

    let err = BatchProofBlobV1::decode(&batch.encode().expect("encode"))
        .expect_err("unreferenced opening entry must reject");
    assert_eq!(err, ProofChkErr::BatchIndexMix);
}

#[test]
fn test_v1_rejects_orphan_reference() {
    let mut batch = sample_batch();
    batch.reference_table.push(batch.reference_table[0].clone());

    let err = BatchProofBlobV1::decode(&batch.encode().expect("encode"))
        .expect_err("unreferenced reference entry must reject");
    assert_eq!(err, ProofChkErr::BatchIndexMix);
}

#[test]
fn test_v1_rejects_orphan_witness() {
    let mut batch = sample_batch();
    batch.witness_dag.push(batch.witness_dag[0].clone());

    let err = BatchProofBlobV1::decode(&batch.encode().expect("encode"))
        .expect_err("unreferenced witness node must reject");
    assert_eq!(err, ProofChkErr::BatchIndexMix);
}

#[test]
fn test_rejects_opening_mismatch() {
    let mut batch = sample_batch();
    let other_path = SettlementPath::new(
        z00z_storage::settlement::DefinitionId::new(bytes(1)),
        z00z_storage::settlement::SerialId::new(99),
        z00z_storage::settlement::TerminalId::new(bytes(8)),
    );
    let other_leaf = SettlementLeaf::Terminal(terminal_leaf(other_path));
    batch.opening_table[0] =
        OpeningEntryV1::from_inclusion(InclusionOpeningV1::new(&other_leaf).expect("leaf bytes"));

    let err = BatchProofBlobV1::decode(&batch.encode().expect("encode"))
        .expect_err("opening leaf must stay bound to its path");
    assert_eq!(err, ProofChkErr::BatchPathMix);
}

#[test]
fn test_rejects_bad_path_order() {
    let batch_a = sample_batch();
    let mut batch_b = sample_batch();
    let path_b = SettlementPath::new(
        z00z_storage::settlement::DefinitionId::new(bytes(1)),
        z00z_storage::settlement::SerialId::new(9),
        z00z_storage::settlement::TerminalId::new(bytes(4)),
    );
    let leaf_b = SettlementLeaf::Terminal(terminal_leaf(path_b));
    batch_b.path_table[0].path = path_b;
    batch_b.opening_table[0] =
        OpeningEntryV1::from_inclusion(InclusionOpeningV1::new(&leaf_b).expect("leaf bytes"));
    let mut batch = BatchProofBlobV1::new(
        batch_a.header.clone(),
        vec![batch_b.path_table[0].clone(), batch_a.path_table[0].clone()],
        vec![batch_a.witness_dag[0].clone()],
        vec![
            batch_b.opening_table[0].clone(),
            batch_a.opening_table[0].clone(),
        ],
        batch_a.reference_table.clone(),
    );
    batch.path_table[0].opening_index = 0;
    batch.path_table[1].opening_index = 1;
    batch.header.leaf_family_set = vec![LeafFamilyTagV1::Asset];

    let err = BatchProofBlobV1::decode(&batch.encode().expect("encode"))
        .expect_err("path order must stay canonical");
    assert_eq!(err, ProofChkErr::BatchOrderMix);
}

#[test]
fn test_v1_rejects_duplicate_paths() {
    let batch = sample_batch();
    let dup = batch.path_table[0].clone();
    let batch = BatchProofBlobV1::new(
        batch.header.clone(),
        vec![dup.clone(), dup],
        batch.witness_dag.clone(),
        batch.opening_table.clone(),
        batch.reference_table.clone(),
    );

    let err = BatchProofBlobV1::decode(&batch.encode().expect("encode"))
        .expect_err("duplicate paths must reject");
    assert_eq!(err, ProofChkErr::BatchDupPath);
}

#[test]
fn test_rejects_tampered_default() {
    let mut batch = sample_nonexistence_batch();
    let mut opening = batch.opening_table[0]
        .decode_nonexistence()
        .expect("decode opening");
    opening.default_commitment = bytes(42);
    batch.opening_table[0] = OpeningEntryV1::from_nonexistence(opening);

    let err = BatchProofBlobV1::decode(&batch.encode().expect("encode"))
        .expect_err("default commitment drift must reject");
    assert_eq!(err, ProofChkErr::BatchDefaultCommitmentMix);
}

#[test]
fn test_rejects_unsharded_domain() {
    let mut batch = sample_batch();
    batch.witness_dag[0].node_domain = NodeDomainTagV1::Shard;

    let err = BatchProofBlobV1::decode(&batch.encode().expect("encode"))
        .expect_err("shard witness domain must reject in live v1 mode");
    assert_eq!(err, ProofChkErr::BatchWitnessDomainMix);
}

#[test]
fn test_v1_accepts_placeholder_root() {
    let batch = sample_nonexistence_batch();

    let checked = BatchProofBlobV1::decode(&batch.encode().expect("encode"))
        .expect("placeholder nonexistence batch must verify");
    assert_eq!(checked, batch);
}

#[test]
fn test_v1_accepts_voucher_root() {
    let batch = sample_voucher_nonexistence_batch();

    let checked = BatchProofBlobV1::decode(&batch.encode().expect("encode"))
        .expect("voucher nonexistence batch must verify");
    assert_eq!(checked, batch);
    assert_eq!(
        checked.header.leaf_family_set,
        vec![LeafFamilyTagV1::Voucher]
    );
    assert_eq!(
        checked.path_table[0].terminal_family,
        TerminalFamilyTagV1::Voucher
    );
    assert_eq!(checked.path_table[0].leaf_family, LeafFamilyTagV1::Voucher);
}

#[test]
fn test_v1_rejects_policy_drift() {
    let mut batch = sample_batch();
    batch.header.bucket_policy_digest = bytes(88);

    let err = BatchProofBlobV1::decode(&batch.encode().expect("encode"))
        .expect_err("unsupported policy digest must reject");
    assert_eq!(err, ProofChkErr::BatchPolicyMix);
}

#[test]
fn test_v1_rejects_missing_checkpoint() {
    let mut batch = sample_batch();
    batch.header.journal_checkpoint = None;

    let err = BatchProofBlobV1::decode(&batch.encode().expect("encode"))
        .expect_err("missing checkpoint must reject");
    assert_eq!(err, ProofChkErr::BatchCheckpointMix);
}

#[test]
fn test_rejects_bytes_limit() {
    let mut batch = sample_batch();
    let bytes_len = batch.encode().expect("encode").len() as u32;
    batch.header.batch_limits.max_total_bytes = bytes_len.saturating_sub(1);

    let err = BatchProofBlobV1::decode(&batch.encode().expect("encode"))
        .expect_err("header byte limit must reject oversized payload");
    assert_eq!(err, ProofChkErr::BatchLimitMix);
}

#[test]
fn test_rejects_path_limit() {
    let mut batch = sample_batch();
    batch.header.batch_limits.max_path_count = 0;

    let err = BatchProofBlobV1::decode(&batch.encode().expect("encode"))
        .expect_err("header path limit must reject oversize path table");
    assert_eq!(err, ProofChkErr::BatchLimitMix);
}

#[test]
fn test_rejects_small_decode() {
    let batch = sample_batch();
    let bytes = batch.encode().expect("encode");
    let mut ceil = BatchProofLimits::v1();
    ceil.max_total_bytes = (bytes.len() as u32).saturating_sub(1);

    let err = BatchProofBlobV1::decode_with_limits(&bytes, ceil)
        .expect_err("caller decode ceiling must reject oversized bytes");
    assert_eq!(err, ProofChkErr::BatchLimitMix);
}

#[test]
fn test_rejects_folded_root() {
    let mut batch = sample_batch();
    batch.witness_dag[0].hash_material[0] = bytes(99);

    let err = BatchProofBlobV1::decode(&batch.encode().expect("encode"))
        .expect_err("root mismatch must reject");
    assert_eq!(err, ProofChkErr::BatchRootMix);
}

#[test]
fn test_accepts_deletion_blob() {
    let mut store = SettlementStore::new();
    let deleted_path = SettlementPath::new(
        z00z_storage::settlement::DefinitionId::new(bytes(52)),
        z00z_storage::settlement::SerialId::new(1),
        z00z_storage::settlement::TerminalId::new(bytes(11)),
    );
    let deleted_item =
        StoreItem::new(deleted_path, terminal_leaf(deleted_path)).expect("deleted item");
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(deleted_item.clone()))])
        .expect("seed deleted item");

    let prior_blob = store
        .settlement_proof_blob(&deleted_path)
        .expect("prior inclusion blob");
    let prior_context = live_prior_context_from_blob(&prior_blob);
    let leaf = SettlementLeaf::Terminal(terminal_leaf(deleted_path));
    let batch = sample_deletion_batch(deleted_path, leaf, prior_context);

    let checked = BatchProofBlobV1::decode(&batch.encode().expect("encode deletion batch"))
        .expect("deletion batch with live prior context must verify");
    assert_eq!(checked, batch);
}

#[test]
fn test_rejects_deletion_bind_drift() {
    let mut store = SettlementStore::new();
    let deleted_path = SettlementPath::new(
        z00z_storage::settlement::DefinitionId::new(bytes(61)),
        z00z_storage::settlement::SerialId::new(2),
        z00z_storage::settlement::TerminalId::new(bytes(14)),
    );
    let deleted_item =
        StoreItem::new(deleted_path, terminal_leaf(deleted_path)).expect("deleted item");
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(deleted_item))])
        .expect("seed deleted item");

    let prior_blob = store
        .settlement_proof_blob(&deleted_path)
        .expect("prior inclusion blob");
    let mut prior_context = live_prior_context_from_blob(&prior_blob);
    prior_context.prior_root_bind[0] ^= 1;
    let leaf = SettlementLeaf::Terminal(terminal_leaf(deleted_path));
    let batch = sample_deletion_batch(deleted_path, leaf, prior_context);

    let err = BatchProofBlobV1::decode(&batch.encode().expect("encode deletion batch"))
        .expect_err("tampered prior root bind must reject");
    assert_eq!(err, ProofChkErr::PriorRootMix);
}

#[test]
fn test_rejects_definition_mismatch() {
    let mut store = SettlementStore::new();
    let deleted_path = SettlementPath::new(
        z00z_storage::settlement::DefinitionId::new(bytes(62)),
        z00z_storage::settlement::SerialId::new(2),
        z00z_storage::settlement::TerminalId::new(bytes(15)),
    );
    let deleted_item =
        StoreItem::new(deleted_path, terminal_leaf(deleted_path)).expect("deleted item");
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(deleted_item))])
        .expect("seed deleted item");

    let prior_blob = store
        .settlement_proof_blob(&deleted_path)
        .expect("prior inclusion blob");
    let mut prior_context = live_prior_context_from_blob(&prior_blob);
    prior_context.definition_root_leaf_bytes[..32].copy_from_slice(&bytes(91));
    let leaf = SettlementLeaf::Terminal(terminal_leaf(deleted_path));
    let batch = sample_deletion_batch(deleted_path, leaf, prior_context);

    let err = BatchProofBlobV1::decode(&batch.encode().expect("encode deletion batch"))
        .expect_err("tampered prior definition leaf must reject");
    assert_eq!(err, ProofChkErr::PriorDefMix);
}

#[test]
fn test_rejects_serial_mismatch() {
    let mut store = SettlementStore::new();
    let deleted_path = SettlementPath::new(
        z00z_storage::settlement::DefinitionId::new(bytes(63)),
        z00z_storage::settlement::SerialId::new(2),
        z00z_storage::settlement::TerminalId::new(bytes(16)),
    );
    let deleted_item =
        StoreItem::new(deleted_path, terminal_leaf(deleted_path)).expect("deleted item");
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(deleted_item))])
        .expect("seed deleted item");

    let prior_blob = store
        .settlement_proof_blob(&deleted_path)
        .expect("prior inclusion blob");
    let mut prior_context = live_prior_context_from_blob(&prior_blob);
    prior_context.serial_root_leaf_bytes[32..36].copy_from_slice(&9u32.to_le_bytes());
    let leaf = SettlementLeaf::Terminal(terminal_leaf(deleted_path));
    let batch = sample_deletion_batch(deleted_path, leaf, prior_context);

    let err = BatchProofBlobV1::decode(&batch.encode().expect("encode deletion batch"))
        .expect_err("tampered prior serial leaf must reject");
    assert_eq!(err, ProofChkErr::PriorSerMix);
}

#[test]
fn test_rejects_bucket_policy() {
    let mut store = SettlementStore::new();
    let deleted_path = SettlementPath::new(
        z00z_storage::settlement::DefinitionId::new(bytes(64)),
        z00z_storage::settlement::SerialId::new(2),
        z00z_storage::settlement::TerminalId::new(bytes(17)),
    );
    let deleted_item =
        StoreItem::new(deleted_path, terminal_leaf(deleted_path)).expect("deleted item");
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(deleted_item))])
        .expect("seed deleted item");

    let prior_blob = store
        .settlement_proof_blob(&deleted_path)
        .expect("prior inclusion blob");
    let mut prior_context = live_prior_context_from_blob(&prior_blob);
    prior_context.bucket_root_leaf_bytes[100..132].copy_from_slice(&bytes(92));
    let leaf = SettlementLeaf::Terminal(terminal_leaf(deleted_path));
    let batch = sample_deletion_batch(deleted_path, leaf, prior_context);

    let err = BatchProofBlobV1::decode(&batch.encode().expect("encode deletion batch"))
        .expect_err("tampered prior bucket policy must reject");
    assert_eq!(err, ProofChkErr::PriorBucketMix);
}

#[test]
fn test_v1_header_matches_default() {
    let batch = sample_batch();
    let policy = sample_policy();

    assert_eq!(
        batch.header.policy_generation,
        u64::from(policy.compatibility_generation())
    );
    assert_eq!(batch.header.bucket_policy_digest, policy.bucket_policy_id());
}

#[test]
fn test_batch_opening_payload_roundtrips() {
    let path = sample_path();
    let leaf = SettlementLeaf::Terminal(terminal_leaf(path));

    let inclusion = InclusionOpeningV1::new(&leaf).expect("inclusion");
    let inclusion_roundtrip =
        InclusionOpeningV1::decode(&inclusion.encode()).expect("decode inclusion");
    assert_eq!(inclusion_roundtrip.version, 1);
    assert_eq!(
        inclusion_roundtrip
            .decode_leaf()
            .expect("decode inclusion leaf"),
        leaf
    );

    let miss = NonExistenceOpeningV1::new(&leaf).expect("miss");
    let miss_roundtrip = NonExistenceOpeningV1::decode(&miss.encode()).expect("decode miss");
    assert_eq!(miss_roundtrip.version, 1);
    assert_eq!(
        miss_roundtrip.default_commitment_version,
        HJMT_DEFAULT_COMMITMENT_VERSION
    );
    assert_eq!(
        miss_roundtrip
            .decode_marker_leaf()
            .expect("decode marker leaf"),
        leaf
    );

    let prior = PriorProofContextV1 {
        version: 1,
        prior_hjmt_version: 17,
        prior_settlement_root: sample_batch().header.settlement_root,
        prior_backend_root: bytes(10),
        prior_root_bind_version: 1,
        prior_root_bind: bytes(11),
        prior_journal_digest: bytes(12),
        prior_checkpoint_bind: bytes(13),
        definition_root_leaf_bytes: vec![1, 2, 3],
        serial_root_leaf_bytes: vec![4, 5, 6],
        bucket_root_leaf_bytes: vec![7, 8, 9],
        definition_proof_bytes: vec![10],
        serial_proof_bytes: vec![11],
        bucket_proof_bytes: vec![12],
        prior_terminal_proof_bytes: vec![13],
    };
    let deletion = DeletionFactV1::new(&leaf, prior).expect("deletion");
    let deletion_roundtrip = DeletionFactV1::decode(&deletion.encode()).expect("decode deletion");
    assert_eq!(deletion_roundtrip.version, 1);
    assert_eq!(
        deletion_roundtrip.default_commitment_version,
        HJMT_DEFAULT_COMMITMENT_VERSION
    );
    assert_eq!(
        deletion_roundtrip
            .decode_deleted_leaf()
            .expect("decode deleted leaf"),
        leaf
    );
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
struct PositiveManifest {
    version: u8,
    cases: Vec<PositiveCase>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
struct PositiveCase {
    id: String,
    proof_family: String,
    path_shape: String,
    path_count: u32,
    expected_verdict: String,
    expected_root_hex: String,
    canonical_bytes_hex: String,
    witness_count: u32,
    reference_count: u32,
    regen_command: String,
    evidence_pointer: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
struct RootGenManifest {
    version: u8,
    cases: Vec<RootGenCase>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
struct RootGenCase {
    id: String,
    source_id: String,
    expected_verdict: String,
    root_generation_tag: String,
    expected_root_hex: Option<String>,
    expected_error: Option<String>,
    canonical_bytes_hex: String,
    regen_command: String,
    evidence_pointer: String,
}

struct LiveCase {
    batch: BatchProofBlobV1,
    baseline: Vec<ProofBlob>,
    proof_family: &'static str,
    path_shape: &'static str,
}

#[test]
fn test_inclusion_matches_baseline() {
    let case = live_case("BPB-G-001");
    assert_case_matches_baseline(&case);
    assert_eq!(case.proof_family, "Inclusion");
    assert_eq!(case.path_shape, "clustered");
    assert_eq!(case.batch.path_table.len(), 2);
}

#[test]
fn test_nonexistence_matches_surface() {
    let case = live_case("BPB-G-002");
    assert_case_matches_baseline(&case);
    assert_eq!(case.proof_family, "NonExistence");
    assert_eq!(case.batch.path_table.len(), case.baseline.len());
}

#[test]
fn test_deletion_matches_surface() {
    let case = live_case("BPB-G-003");
    assert_case_matches_baseline(&case);
    assert_eq!(case.proof_family, "Deletion");
    assert_eq!(case.batch.path_table.len(), case.baseline.len());
}

#[test]
fn test_clustered_witness_reuse() {
    let case_a = live_case("BPB-G-004");
    let case_b = live_case("BPB-G-004");
    assert_eq!(
        case_a.batch.encode().expect("encode case a"),
        case_b.batch.encode().expect("encode case b")
    );
    let baseline_bytes = case_a
        .baseline
        .iter()
        .map(|blob| blob.encode().expect("encode baseline blob").len())
        .sum::<usize>();
    let batch_bytes = case_a.batch.encode().expect("encode batch").len();
    assert!(
        batch_bytes < baseline_bytes,
        "clustered batch bytes {batch_bytes} must beat baseline {baseline_bytes}"
    );
    assert!(
        case_a.batch.witness_dag.len() < case_a.batch.reference_table.len() * 4,
        "clustered batch must reuse witnesses"
    );
}

#[test]
fn test_scattered_ref_indexes() {
    let case_a = live_case("BPB-G-005");
    let case_b = live_case("BPB-G-005");
    assert_case_matches_baseline(&case_a);
    assert_eq!(case_a.batch.reference_table, case_b.batch.reference_table);
    assert_eq!(case_a.batch.path_table, case_b.batch.path_table);
    assert_eq!(
        case_a.batch.encode().expect("encode case a"),
        case_b.batch.encode().expect("encode case b")
    );
}

#[test]
fn test_positive_manifest_covers_bpb() {
    let manifest = load_positive_manifest();
    let ids = manifest
        .cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for required in [
        "BPB-G-001",
        "BPB-G-002",
        "BPB-G-003",
        "BPB-G-004",
        "BPB-G-005",
    ] {
        assert!(
            ids.contains(required),
            "missing positive fixture id {required}"
        );
    }
}

#[test]
fn test_positive_fixtures_match_builders() {
    let manifest = load_positive_manifest();
    if std::env::var_os("Z00Z_REGEN_DUMP").is_some() {
        let live = PositiveManifest {
            version: 1,
            cases: manifest
                .cases
                .iter()
                .map(|case| render_positive_case(&case.id))
                .collect(),
        };
        let json = JsonCodec
            .serialize_pretty(&live)
            .expect("serialize positive manifest");
        println!(
            "{}",
            String::from_utf8(json).expect("utf8 positive manifest")
        );
    }
    assert_eq!(manifest.version, 1);
    for case in &manifest.cases {
        let rendered = render_positive_case(&case.id);
        assert_eq!(rendered, *case, "{} fixture drifted", case.id);
        let decoded = BatchProofBlobV1::decode(&decode_hex(&case.canonical_bytes_hex))
            .expect("decode positive fixture");
        assert_eq!(
            encode_hex(&decoded.header.settlement_root.into_bytes()),
            case.expected_root_hex,
            "{} root drifted",
            case.id
        );
    }
}

#[test]
fn test_root_generation_contract() {
    let manifest = load_root_gen_manifest();
    if std::env::var_os("Z00Z_REGEN_DUMP").is_some() {
        let live = render_root_gen_manifest();
        let json = JsonCodec
            .serialize_pretty(&live)
            .expect("serialize root-generation manifest");
        println!(
            "{}",
            String::from_utf8(json).expect("utf8 root-generation manifest")
        );
    }
    assert_eq!(manifest.version, 1);
    for case in &manifest.cases {
        let bytes = decode_hex(&case.canonical_bytes_hex);
        match case.expected_verdict.as_str() {
            "accept" => {
                let decoded = BatchProofBlobV1::decode(&bytes).expect("decode accepted migration");
                assert_eq!(
                    Some(encode_hex(&decoded.header.settlement_root.into_bytes())),
                    case.expected_root_hex,
                    "{} root drifted",
                    case.id
                );
            }
            "verifier_reject" => {
                let err = BatchProofBlobV1::decode(&bytes)
                    .expect_err("future generation migration must reject");
                assert_eq!(
                    Some(expected_error(
                        case.expected_error
                            .as_deref()
                            .expect("expected error for reject case")
                    )),
                    Some(err),
                    "{} reject drifted",
                    case.id
                );
            }
            other => panic!("unknown migration verdict {other}"),
        }
    }
}

fn load_positive_manifest() -> PositiveManifest {
    load_json(
        fixture_root()
            .join("batch_proof_v1_positive")
            .join("manifest.json"),
    )
}

fn load_root_gen_manifest() -> RootGenManifest {
    load_json(
        fixture_root()
            .join("root_generation_migration")
            .join("manifest.json"),
    )
}

fn load_json<T>(path: PathBuf) -> T
where
    T: DeserializeOwned,
{
    let bytes =
        fs::read(&path).unwrap_or_else(|err| panic!("read {} failed: {err}", path.display()));
    JsonCodec
        .deserialize(&bytes)
        .unwrap_or_else(|err| panic!("decode {} failed: {err}", path.display()))
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hjmt_upgrade")
}

fn live_case(case_id: &str) -> LiveCase {
    match case_id {
        "BPB-G-001" => build_inclusion_case_2(),
        "BPB-G-002" => build_nonexistence_case(),
        "BPB-G-003" => build_deletion_case(),
        "BPB-G-004" => build_clustered_case_8(),
        "BPB-G-005" => build_scattered_case_8(),
        other => panic!("unknown live case {other}"),
    }
}

fn build_inclusion_case_2() -> LiveCase {
    let mut store = SettlementStore::new();
    let paths = same_bucket_paths(&store, 41, 7, 2, 1);
    seed_terminals(&mut store, &paths);
    let baseline = store
        .settlement_proof_blobs(&paths)
        .expect("baseline proof blobs");
    let batch = store
        .settlement_inclusion_batch_v1(&paths)
        .expect("live inclusion batch");
    LiveCase {
        batch,
        baseline,
        proof_family: "Inclusion",
        path_shape: "clustered",
    }
}

fn build_nonexistence_case() -> LiveCase {
    let mut store = SettlementStore::new();
    let paths = same_bucket_paths(&store, 52, 9, 4, 1);
    seed_rights(&mut store, &paths[..2], 61);
    let missing = vec![paths[2], paths[3]];
    let baseline = missing
        .iter()
        .map(|path| {
            store
                .settlement_nonexistence_proof_blob(path, SettlementLeafFamily::Right)
                .expect("nonexistence baseline")
        })
        .collect::<Vec<_>>();
    let batch = store
        .settlement_nonexistence_batch_v1(&missing, SettlementLeafFamily::Right)
        .expect("live nonexistence batch");
    LiveCase {
        batch,
        baseline,
        proof_family: "NonExistence",
        path_shape: "clustered",
    }
}

fn build_deletion_case() -> LiveCase {
    let mut store = SettlementStore::new();
    let paths = same_bucket_paths(&store, 63, 5, 4, 1);
    seed_terminals(&mut store, &paths);
    let deleted = vec![paths[0], paths[1]];
    delete_paths(&mut store, &deleted);
    let baseline = store
        .settlement_proof_blobs(&deleted)
        .expect("deletion baseline");
    let batch = store
        .settlement_deletion_batch_v1(&deleted)
        .expect("live deletion batch");
    LiveCase {
        batch,
        baseline,
        proof_family: "Deletion",
        path_shape: "clustered",
    }
}

fn build_clustered_case_8() -> LiveCase {
    let mut store = SettlementStore::new();
    let paths = same_bucket_paths(&store, 74, 11, 8, 1);
    seed_terminals(&mut store, &paths);
    let baseline = store
        .settlement_proof_blobs(&paths)
        .expect("clustered baseline");
    let batch = store
        .settlement_inclusion_batch_v1(&paths)
        .expect("clustered batch");
    LiveCase {
        batch,
        baseline,
        proof_family: "Inclusion",
        path_shape: "clustered",
    }
}

fn build_scattered_case_8() -> LiveCase {
    let mut store = SettlementStore::new();
    let paths = (0..8)
        .map(|index| {
            fixture_path(
                90u8.wrapping_add(index as u8),
                (index + 1) as u32,
                130u8.wrapping_add(index as u8),
            )
        })
        .collect::<Vec<_>>();
    seed_terminals(&mut store, &paths);
    let baseline = store
        .settlement_proof_blobs(&paths)
        .expect("scattered baseline");
    let batch = store
        .settlement_inclusion_batch_v1(&paths)
        .expect("scattered batch");
    LiveCase {
        batch,
        baseline,
        proof_family: "Inclusion",
        path_shape: "scattered",
    }
}

fn assert_case_matches_baseline(case: &LiveCase) {
    let encoded = case.batch.encode().expect("encode live batch");
    let decoded = BatchProofBlobV1::decode(&encoded).expect("decode live batch");
    assert_eq!(decoded, case.batch);
    assert_eq!(
        decoded.header.settlement_root,
        case.batch.header.settlement_root
    );
    assert_eq!(case.batch.path_table.len(), case.baseline.len());
    for proof in &case.baseline {
        assert_eq!(
            proof.item().settlement_root(),
            case.batch.header.settlement_root
        );
    }
}

fn render_positive_case(case_id: &str) -> PositiveCase {
    let live = live_case(case_id);
    let bytes = live.batch.encode().expect("encode live case");
    PositiveCase {
        id: case_id.to_string(),
        proof_family: live.proof_family.to_string(),
        path_shape: live.path_shape.to_string(),
        path_count: live.batch.path_table.len() as u32,
        expected_verdict: "accept".to_string(),
        expected_root_hex: encode_hex(&live.batch.header.settlement_root.into_bytes()),
        canonical_bytes_hex: encode_hex(&bytes),
        witness_count: live.batch.witness_dag.len() as u32,
        reference_count: live.batch.reference_table.len() as u32,
        regen_command: "Z00Z_REGEN_DUMP=1 cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof test_positive_fixtures_match_builders -- --exact --nocapture".to_string(),
        evidence_pointer: "crates/z00z_storage/tests/test_hjmt_batch_proof.rs::test_positive_fixtures_match_builders".to_string(),
    }
}

fn render_root_gen_manifest() -> RootGenManifest {
    let live = live_case("BPB-G-003");
    let accept_bytes = live.batch.encode().expect("encode live deletion case");
    let mut future = live.batch.clone();
    future.header.root_generation = RootGenerationTagV1::RootGeneration0;
    let reject_bytes = future.encode().expect("encode future generation case");
    RootGenManifest {
        version: 1,
        cases: vec![
            RootGenCase {
                id: "RGM-G-001".to_string(),
                source_id: "BPB-G-003".to_string(),
                expected_verdict: "accept".to_string(),
                root_generation_tag: "RootGeneration1".to_string(),
                expected_root_hex: Some(encode_hex(
                    &live.batch.header.settlement_root.into_bytes(),
                )),
                expected_error: None,
                canonical_bytes_hex: encode_hex(&accept_bytes),
                regen_command: "Z00Z_REGEN_DUMP=1 cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof test_root_generation_contract -- --exact --nocapture".to_string(),
                evidence_pointer: "crates/z00z_storage/tests/test_hjmt_batch_proof.rs::test_root_generation_contract".to_string(),
            },
            RootGenCase {
                id: "RGM-T-001".to_string(),
                source_id: "BPB-G-003".to_string(),
                expected_verdict: "verifier_reject".to_string(),
                root_generation_tag: "RootGeneration0".to_string(),
                expected_root_hex: None,
                expected_error: Some("BatchRootGenerationMix".to_string()),
                canonical_bytes_hex: encode_hex(&reject_bytes),
                regen_command: "Z00Z_REGEN_DUMP=1 cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof test_root_generation_contract -- --exact --nocapture".to_string(),
                evidence_pointer: "crates/z00z_storage/tests/test_hjmt_batch_proof.rs::test_root_generation_contract".to_string(),
            },
        ],
    }
}

fn fixture_path(def_mark: u8, serial: u32, term_mark: u8) -> SettlementPath {
    fixture_path_seed(def_mark, serial, u16::from(term_mark))
}

fn fixture_path_seed(def_mark: u8, serial: u32, seed: u16) -> SettlementPath {
    let mut term = [0u8; 32];
    term[0] = (seed >> 8) as u8;
    term[1] = seed as u8;
    term[2] = def_mark;
    term[3] = serial as u8;
    SettlementPath::new(
        DefinitionId::new(bytes(def_mark)),
        SerialId::new(serial),
        TerminalId::new(term),
    )
}

fn same_bucket_paths(
    store: &SettlementStore,
    def_mark: u8,
    serial: u32,
    need: usize,
    start_seed: u16,
) -> Vec<SettlementPath> {
    let mut target = None;
    let mut out = Vec::with_capacity(need);
    for seed in start_seed..=u16::MAX {
        let path = fixture_path_seed(def_mark, serial, seed);
        let bucket = store.bucket_policy().derive_bucket_id(path);
        if target.is_none() {
            target = Some(bucket);
            out.push(path);
        } else if target == Some(bucket) {
            out.push(path);
        }
        if out.len() == need {
            return out;
        }
    }
    panic!("missing same-bucket fixture set with {need} paths");
}

fn seed_terminals(store: &mut SettlementStore, paths: &[SettlementPath]) {
    let ops = paths
        .iter()
        .map(|path| {
            StoreOp::Put(Box::new(
                StoreItem::new(*path, terminal_leaf(*path)).expect("terminal item"),
            ))
        })
        .collect::<Vec<_>>();
    store
        .apply_settlement_ops(ops)
        .expect("seed terminal fixture paths");
}

fn seed_rights(store: &mut SettlementStore, paths: &[SettlementPath], start_mark: u8) {
    let ops = paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let mark = start_mark.wrapping_add(index as u8);
            let right = RightLeaf {
                version: 1,
                terminal_id: path.terminal_id(),
                right_class: RightClass::MachineCapability,
                issuer_scope: bytes(mark.wrapping_add(1)),
                provider_scope: bytes(mark.wrapping_add(2)),
                holder_commitment: bytes(mark.wrapping_add(3)),
                control_commitment: bytes(mark.wrapping_add(4)),
                beneficiary_commitment: bytes(mark.wrapping_add(5)),
                payload_commitment: bytes(mark.wrapping_add(6)),
                valid_from: 10,
                valid_until: 20,
                challenge_from: 12,
                challenge_until: 18,
                use_nonce: bytes(mark.wrapping_add(7)),
                revocation_policy_id: bytes(mark.wrapping_add(8)),
                transition_policy_id: bytes(mark.wrapping_add(9)),
                challenge_policy_id: bytes(mark.wrapping_add(10)),
                disclosure_policy_id: bytes(mark.wrapping_add(11)),
                retention_policy_id: bytes(mark.wrapping_add(12)),
            };
            StoreOp::Put(Box::new(
                StoreItem::new(*path, SettlementLeaf::Right(right)).expect("right item"),
            ))
        })
        .collect::<Vec<_>>();
    store
        .apply_settlement_ops(ops)
        .expect("seed right fixture paths");
}

fn delete_paths(store: &mut SettlementStore, paths: &[SettlementPath]) {
    store
        .apply_settlement_ops(paths.iter().copied().map(StoreOp::Delete).collect())
        .expect("delete fixture paths");
}

fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;

    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut out, "{byte:02x}").expect("write hex");
    }
    out
}

fn decode_hex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0, "hex must have even length");
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|chunk| {
            let text = std::str::from_utf8(chunk).expect("utf8 hex");
            u8::from_str_radix(text, 16).expect("decode hex byte")
        })
        .collect()
}

fn expected_error(name: &str) -> ProofChkErr {
    match name {
        "BatchCheckpointMix" => ProofChkErr::BatchCheckpointMix,
        "BatchLeafFamilyMix" => ProofChkErr::BatchLeafFamilyMix,
        "UnsupportedBatchProofVersion" => ProofChkErr::UnsupportedBatchProofVersion,
        "BatchWitnessStepMix" => ProofChkErr::BatchWitnessStepMix,
        "BatchIndexMix" => ProofChkErr::BatchIndexMix,
        "BatchOpeningKindMix" => ProofChkErr::BatchOpeningKindMix,
        "BatchHashCountMix" => ProofChkErr::BatchHashCountMix,
        "BatchRootGenerationMix" => ProofChkErr::BatchRootGenerationMix,
        other => panic!("unknown expected error {other}"),
    }
}
