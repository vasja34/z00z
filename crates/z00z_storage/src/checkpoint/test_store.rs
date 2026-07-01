use z00z_utils::codec::{BincodeCodec, Codec};

use super::{check_art_key, check_draft_key, check_exec_key, load_artifact, load_draft};
use crate::{
    checkpoint::{
        derive_draft_id, CheckpointDraft, CheckpointDraftId, CheckpointExecInputId, CheckpointId,
        CheckpointVersion, CreatedEnt, SpentEnt,
    },
    settlement::CheckRoot,
    CheckpointError,
};

fn draft() -> CheckpointDraft {
    CheckpointDraft::new(
        CheckpointVersion::CURRENT,
        13,
        CheckRoot::new([1u8; 32]),
        CheckRoot::new([2u8; 32]),
        vec![SpentEnt::new([3u8; 32])],
        vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
    )
}

#[test]
fn test_load_rejects_draft_bytes() {
    let bytes = BincodeCodec.serialize(&draft()).expect("encode draft");
    let err = load_artifact(&bytes).expect_err("draft bytes must reject as wrong class");

    assert!(matches!(err, CheckpointError::WrongClass));
}

#[test]
fn test_load_rejects_artifact_bytes() {
    let draft = draft();
    let proof = draft
        .attest_proof(
            crate::snapshot::PrepSnapshotId::new([6u8; 32]),
            CheckpointExecInputId::new([7u8; 32]),
        )
        .expect("proof");
    let art = draft.finalize(proof).expect("artifact");
    let bytes = BincodeCodec.serialize(&art).expect("encode artifact");
    let err = load_draft(&bytes).expect_err("artifact bytes must reject as wrong class");

    assert!(matches!(err, CheckpointError::WrongClass));
}

#[test]
fn test_key_checks_are_precise() {
    let want_draft = derive_draft_id(&draft()).expect("draft id");
    let err_draft = check_draft_key(want_draft, CheckpointDraftId::new([0u8; 32]))
        .expect_err("draft key mismatch");
    let err_art = check_art_key(CheckpointId::new([1u8; 32]), CheckpointId::new([2u8; 32]))
        .expect_err("art key mismatch");
    let err_exec = check_exec_key(
        CheckpointExecInputId::new([1u8; 32]),
        CheckpointExecInputId::new([2u8; 32]),
    )
    .expect_err("exec key mismatch");

    assert!(matches!(err_draft, CheckpointError::KeyMix));
    assert!(matches!(err_art, CheckpointError::KeyMix));
    assert!(matches!(err_exec, CheckpointError::KeyMix));
}
