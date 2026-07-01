#[allow(dead_code)]
mod test_batch_proof_support;

use test_batch_proof_support::{bytes, sample_path};
use z00z_storage::settlement::{
    BatchProofBlobV1, BatchProofFamilyTagV1, InclusionOpeningV1, OpeningEntryV1, ProofChkErr,
    SettlementLeafFamily,
};
use z00z_utils::codec::{Codec, JsonCodec};

const NEGATIVE_MANIFEST: &str =
    include_str!("fixtures/hjmt_upgrade/batch_proof_v1_negative/manifest.json");
const POSITIVE_MANIFEST: &str =
    include_str!("fixtures/hjmt_upgrade/batch_proof_v1_positive/manifest.json");

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
struct NegativeManifest {
    version: u8,
    cases: Vec<NegativeCase>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
struct NegativeCase {
    id: String,
    base_case: String,
    mutation: String,
    mutation_point: String,
    expected_verdict: String,
    reject_stage: String,
    expected_error: String,
    regen_command: String,
    evidence_pointer: String,
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
    expected_verdict: String,
    expected_root_hex: String,
    canonical_bytes_hex: String,
    regen_command: String,
    evidence_pointer: String,
}

#[test]
fn test_manifest_covers_bpb() {
    let manifest = load_manifest();
    let ids = manifest
        .cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for required in [
        "BPB-T-001",
        "BPB-T-002",
        "BPB-T-003",
        "BPB-T-004",
        "BPB-T-005",
        "BPB-T-006",
        "BPB-T-007",
        "BPB-T-008",
    ] {
        assert!(
            ids.contains(required),
            "missing negative fixture id {required}"
        );
    }
}

#[test]
fn test_manifest_entries_are_nonempty() {
    let manifest = load_manifest();
    let sources = load_positive_manifest();
    assert_eq!(manifest.version, 1);
    for case in &manifest.cases {
        assert!(
            !case.base_case.is_empty(),
            "base_case missing for {}",
            case.id
        );
        assert!(
            !case.mutation.is_empty(),
            "mutation missing for {}",
            case.id
        );
        assert!(
            !case.mutation_point.is_empty(),
            "mutation_point missing for {}",
            case.id
        );
        assert!(
            !case.expected_verdict.is_empty(),
            "expected_verdict missing for {}",
            case.id
        );
        assert_eq!(
            case.expected_verdict, "verifier_reject",
            "unexpected verdict for {}",
            case.id
        );
        assert!(
            !case.reject_stage.is_empty(),
            "reject_stage missing for {}",
            case.id
        );
        assert!(
            !case.expected_error.is_empty(),
            "expected_error missing for {}",
            case.id
        );
        assert!(
            !case.regen_command.is_empty(),
            "regen_command missing for {}",
            case.id
        );
        assert!(
            !case.evidence_pointer.is_empty(),
            "evidence_pointer missing for {}",
            case.id
        );
        assert!(
            sources
                .cases
                .iter()
                .any(|source| source.id == case.base_case),
            "unknown base_case {} for {}",
            case.base_case,
            case.id
        );
        assert!(
            case.base_case.starts_with("BPB-G-"),
            "negative case {} must mutate a positive BPB-G source",
            case.id
        );
    }
}

#[test]
fn test_base_cases_map_manifest() {
    let sources = load_positive_manifest();
    assert_eq!(sources.version, 1);
    for source in &sources.cases {
        assert!(!source.id.is_empty(), "source id missing");
        assert!(!source.proof_family.is_empty(), "proof_family missing");
        assert!(
            matches!(
                source.proof_family.as_str(),
                "Inclusion" | "NonExistence" | "Deletion"
            ),
            "unknown proof_family for {}",
            source.id
        );
        assert!(
            !source.expected_verdict.is_empty(),
            "expected_verdict missing for {}",
            source.id
        );
        assert_eq!(
            source.expected_verdict, "accept",
            "unexpected verdict for {}",
            source.id
        );
        assert!(
            !source.expected_root_hex.is_empty(),
            "expected_root_hex missing for {}",
            source.id
        );
        assert!(
            !source.canonical_bytes_hex.is_empty(),
            "canonical_bytes_hex missing for {}",
            source.id
        );
        assert!(
            !source.regen_command.is_empty(),
            "regen_command missing for {}",
            source.id
        );
        assert!(
            !source.evidence_pointer.is_empty(),
            "evidence_pointer missing for {}",
            source.id
        );
    }
}

#[test]
fn test_base_cases_decode_manifest() {
    let sources = load_positive_manifest();
    for source in &sources.cases {
        let bytes = decode_hex(&source.canonical_bytes_hex);
        let decoded = BatchProofBlobV1::decode(&bytes).expect("decode positive fixture source");
        assert_eq!(
            encode_hex(&decoded.header.settlement_root.into_bytes()),
            source.expected_root_hex,
            "{} root drifted",
            source.id
        );
        assert_eq!(
            source.expected_verdict, "accept",
            "{} verdict drifted",
            source.id
        );
    }
}

#[test]
fn test_fixtures_reject_expected() {
    let manifest = load_manifest();
    for case in &manifest.cases {
        let err = match run_negative_case(case) {
            Ok(()) => panic!("{} must reject at {}", case.id, case.reject_stage),
            Err(err) => err,
        };
        assert_eq!(
            err,
            expected_error(&case.expected_error),
            "{} rejected with wrong error",
            case.id
        );
    }
}

fn load_manifest() -> NegativeManifest {
    JsonCodec
        .deserialize(NEGATIVE_MANIFEST.as_bytes())
        .expect("negative manifest")
}

fn load_positive_manifest() -> PositiveManifest {
    JsonCodec
        .deserialize(POSITIVE_MANIFEST.as_bytes())
        .expect("positive manifest")
}

fn run_negative_case(case: &NegativeCase) -> Result<(), ProofChkErr> {
    let source_bytes = positive_source_bytes(&case.base_case);
    let mut batch = BatchProofBlobV1::decode(&source_bytes).expect("decode source batch");
    match case.mutation.as_str() {
        "header_missing_checkpoint" => {
            batch.header.journal_checkpoint = None;
            BatchProofBlobV1::decode(&batch.encode()?).map(|_| ())
        }
        "path_terminal_family_right" => {
            batch.path_table[0].terminal_family =
                z00z_storage::settlement::TerminalFamilyTagV1::Right;
            BatchProofBlobV1::decode(&batch.encode()?).map(|_| ())
        }
        "opening_version_2" => {
            let mut payload = batch.opening_table[0].decode_inclusion()?;
            payload.version = 2;
            batch.opening_table[0] = OpeningEntryV1::from_inclusion(payload);
            BatchProofBlobV1::decode(&batch.encode()?).map(|_| ())
        }
        "witness_child_index_flip" => {
            batch.witness_dag[0].child_index ^= 1;
            BatchProofBlobV1::decode(&batch.encode()?).map(|_| ())
        }
        "reference_witness_oob" => {
            batch.reference_table[0].witness_indexes = vec![9];
            BatchProofBlobV1::decode(&batch.encode()?).map(|_| ())
        }
        "proof_family_deletion_with_inclusion_opening" => {
            batch.header.proof_family = BatchProofFamilyTagV1::Deletion;
            BatchProofBlobV1::decode(&batch.encode()?).map(|_| ())
        }
        "opening_payload_family_right" => {
            let wrong_leaf = SettlementLeafFamily::Right.marker_leaf(sample_path());
            batch.opening_table[0] =
                OpeningEntryV1::from_inclusion(InclusionOpeningV1::new(&wrong_leaf)?);
            BatchProofBlobV1::decode(&batch.encode()?).map(|_| ())
        }
        "witness_hash_material_count_2" => {
            batch.witness_dag[0].hash_material.push(bytes(44));
            BatchProofBlobV1::decode(&batch.encode()?).map(|_| ())
        }
        other => panic!("unknown mutation {other}"),
    }
}

fn positive_source_bytes(source_id: &str) -> Vec<u8> {
    let sources = load_positive_manifest();
    let source = sources
        .cases
        .iter()
        .find(|source| source.id == source_id)
        .unwrap_or_else(|| panic!("unknown positive fixture {source_id}"));
    decode_hex(&source.canonical_bytes_hex)
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
    assert_eq!(
        value.len() % 2,
        0,
        "canonical source hex must have even length"
    );
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
        other => panic!("unknown expected error {other}"),
    }
}
