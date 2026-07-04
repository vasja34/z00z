use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};

use crate::CheckpointError;

use super::{
    archive_manifest::{
        decode_archive_manifest_bin_checked, decode_archive_manifest_json_checked,
        encode_archive_manifest_bin_checked, encode_archive_manifest_json_checked,
        CheckpointArchiveManifestV1,
    },
    archive_receipt::{
        decode_archive_receipt_bin_checked, decode_archive_receipt_json_checked,
        encode_archive_receipt_bin_checked, encode_archive_receipt_json_checked,
        ArchiveProviderReceiptV1,
    },
    artifact_final::{check_proof_sys, check_ver},
    audit::{check_audit_ver, CheckpointAudit},
    exec_input::{check_exec_ver, CheckpointExecInput},
    link::{
        decode_link_bin_checked, decode_link_json_checked, encode_link_bin_checked,
        encode_link_json_checked, CheckpointLink,
    },
    pruning::{
        decode_pruning_decision_bin_checked, decode_pruning_decision_json_checked,
        encode_pruning_decision_bin_checked, encode_pruning_decision_json_checked,
        PruningDecisionV1,
    },
    retrieval_audit::{
        decode_retrieval_audit_bin_checked, decode_retrieval_audit_json_checked,
        encode_retrieval_audit_bin_checked, encode_retrieval_audit_json_checked, RetrievalAuditV1,
    },
    state_snapshot::{
        decode_state_snapshot_bin_checked, decode_state_snapshot_json_checked,
        encode_state_snapshot_bin_checked, encode_state_snapshot_json_checked, StateSnapshotV1,
    },
    CheckpointArtifact, CheckpointDraft, CheckpointStatement,
};

pub(crate) fn check_artifact_contract(
    artifact: &CheckpointArtifact,
) -> Result<(), CheckpointError> {
    if crate::settlement::CheckRoot::from(artifact.prev_settlement_root()) != artifact.prev_root()
        || crate::settlement::CheckRoot::from(artifact.new_settlement_root()) != artifact.new_root()
    {
        return Err(CheckpointError::RootMix);
    }
    if artifact.cp_proof().is_empty() {
        return Err(CheckpointError::ProoflessFinal);
    }
    if artifact.has_partial_stmt_ids() {
        return Err(CheckpointError::ArtifactCompatMix);
    }

    match artifact.statement() {
        CheckpointStatement::Detached => Err(CheckpointError::ArtifactCompatMix),
        CheckpointStatement::CURRENT(_) => {
            if artifact.proof_sys().is_opaque_attest() {
                if let CheckpointStatement::CURRENT(stmt) = artifact.statement() {
                    if artifact.cp_proof() == stmt.backend_payload().as_slice() {
                        Ok(())
                    } else {
                        Err(CheckpointError::ProofMix)
                    }
                } else {
                    Err(CheckpointError::ArtifactCompatMix)
                }
            } else {
                Err(CheckpointError::ArtifactCompatMix)
            }
        }
    }
}

fn check_draft_contract(draft: &CheckpointDraft) -> Result<(), CheckpointError> {
    if crate::settlement::CheckRoot::from(draft.prev_settlement_root()) != draft.prev_root()
        || crate::settlement::CheckRoot::from(draft.new_settlement_root()) != draft.new_root()
    {
        return Err(CheckpointError::RootMix);
    }
    Ok(())
}

fn check_exec_contract(exec: &CheckpointExecInput) -> Result<(), CheckpointError> {
    if crate::settlement::CheckRoot::from(exec.prev_settlement_root()) != exec.prev_root() {
        return Err(CheckpointError::RootMix);
    }
    Ok(())
}

/// Decision 1 fixed the codec contract to dual JSON plus binary.
/// Binary bytes are the canonical identity source for all content-addressed ids.

pub fn encode_draft_bin(draft: &CheckpointDraft) -> Result<Vec<u8>, CheckpointError> {
    check_ver(draft.version())?;
    check_draft_contract(draft)?;
    Ok(BincodeCodec.serialize(draft)?)
}

pub fn decode_draft_bin(bytes: &[u8]) -> Result<CheckpointDraft, CheckpointError> {
    let draft: CheckpointDraft = BincodeCodec.deserialize(bytes)?;
    check_ver(draft.version())?;
    check_draft_contract(&draft)?;
    Ok(draft)
}

pub fn encode_draft_json(draft: &CheckpointDraft) -> Result<Vec<u8>, CheckpointError> {
    check_ver(draft.version())?;
    check_draft_contract(draft)?;
    Ok(JsonCodec.serialize_pretty(draft)?)
}

pub fn decode_draft_json(bytes: &[u8]) -> Result<CheckpointDraft, CheckpointError> {
    let draft: CheckpointDraft = JsonCodec.deserialize(bytes)?;
    check_ver(draft.version())?;
    check_draft_contract(&draft)?;
    Ok(draft)
}

pub fn encode_art_bin(artifact: &CheckpointArtifact) -> Result<Vec<u8>, CheckpointError> {
    check_ver(artifact.version())?;
    check_proof_sys(artifact.proof_sys())?;
    check_artifact_contract(artifact)?;
    Ok(BincodeCodec.serialize(artifact)?)
}

pub fn decode_art_bin(bytes: &[u8]) -> Result<CheckpointArtifact, CheckpointError> {
    let artifact: CheckpointArtifact = BincodeCodec.deserialize(bytes)?;
    check_ver(artifact.version())?;
    check_proof_sys(artifact.proof_sys())?;
    check_artifact_contract(&artifact)?;
    Ok(artifact)
}

pub fn encode_art_json(artifact: &CheckpointArtifact) -> Result<Vec<u8>, CheckpointError> {
    check_ver(artifact.version())?;
    check_proof_sys(artifact.proof_sys())?;
    check_artifact_contract(artifact)?;
    Ok(JsonCodec.serialize_pretty(artifact)?)
}

pub fn decode_art_json(bytes: &[u8]) -> Result<CheckpointArtifact, CheckpointError> {
    let artifact: CheckpointArtifact = JsonCodec.deserialize(bytes)?;
    check_ver(artifact.version())?;
    check_proof_sys(artifact.proof_sys())?;
    check_artifact_contract(&artifact)?;
    Ok(artifact)
}

pub fn encode_link_bin(link: &CheckpointLink) -> Result<Vec<u8>, CheckpointError> {
    encode_link_bin_checked(link)
}

pub fn decode_link_bin(bytes: &[u8]) -> Result<CheckpointLink, CheckpointError> {
    decode_link_bin_checked(bytes)
}

pub fn encode_link_json(link: &CheckpointLink) -> Result<Vec<u8>, CheckpointError> {
    encode_link_json_checked(link)
}

pub fn decode_link_json(bytes: &[u8]) -> Result<CheckpointLink, CheckpointError> {
    decode_link_json_checked(bytes)
}

pub fn encode_exec_bin(exec: &CheckpointExecInput) -> Result<Vec<u8>, CheckpointError> {
    check_exec_ver(exec.version())?;
    check_exec_contract(exec)?;
    Ok(BincodeCodec.serialize(exec)?)
}

pub fn decode_exec_bin(bytes: &[u8]) -> Result<CheckpointExecInput, CheckpointError> {
    let exec: CheckpointExecInput = BincodeCodec.deserialize(bytes)?;
    check_exec_ver(exec.version())?;
    check_exec_contract(&exec)?;
    Ok(exec)
}

pub fn encode_exec_json(exec: &CheckpointExecInput) -> Result<Vec<u8>, CheckpointError> {
    check_exec_ver(exec.version())?;
    check_exec_contract(exec)?;
    Ok(JsonCodec.serialize_pretty(exec)?)
}

pub fn decode_exec_json(bytes: &[u8]) -> Result<CheckpointExecInput, CheckpointError> {
    let exec: CheckpointExecInput = JsonCodec.deserialize(bytes)?;
    check_exec_ver(exec.version())?;
    check_exec_contract(&exec)?;
    Ok(exec)
}

pub fn encode_audit_bin(audit: &CheckpointAudit) -> Result<Vec<u8>, CheckpointError> {
    check_audit_ver(audit.version())?;
    Ok(BincodeCodec.serialize(audit)?)
}

pub fn decode_audit_bin(bytes: &[u8]) -> Result<CheckpointAudit, CheckpointError> {
    let audit: CheckpointAudit = BincodeCodec.deserialize(bytes)?;
    check_audit_ver(audit.version())?;
    Ok(audit)
}

pub fn encode_audit_json(audit: &CheckpointAudit) -> Result<Vec<u8>, CheckpointError> {
    check_audit_ver(audit.version())?;
    Ok(JsonCodec.serialize_pretty(audit)?)
}

pub fn decode_audit_json(bytes: &[u8]) -> Result<CheckpointAudit, CheckpointError> {
    let audit: CheckpointAudit = JsonCodec.deserialize(bytes)?;
    check_audit_ver(audit.version())?;
    Ok(audit)
}

pub fn encode_archive_manifest_bin(
    manifest: &CheckpointArchiveManifestV1,
) -> Result<Vec<u8>, CheckpointError> {
    encode_archive_manifest_bin_checked(manifest)
}

pub fn decode_archive_manifest_bin(
    bytes: &[u8],
) -> Result<CheckpointArchiveManifestV1, CheckpointError> {
    decode_archive_manifest_bin_checked(bytes)
}

pub fn encode_archive_manifest_json(
    manifest: &CheckpointArchiveManifestV1,
) -> Result<Vec<u8>, CheckpointError> {
    encode_archive_manifest_json_checked(manifest)
}

pub fn decode_archive_manifest_json(
    bytes: &[u8],
) -> Result<CheckpointArchiveManifestV1, CheckpointError> {
    decode_archive_manifest_json_checked(bytes)
}

pub fn encode_archive_receipt_bin(
    receipt: &ArchiveProviderReceiptV1,
) -> Result<Vec<u8>, CheckpointError> {
    encode_archive_receipt_bin_checked(receipt)
}

pub fn decode_archive_receipt_bin(
    bytes: &[u8],
) -> Result<ArchiveProviderReceiptV1, CheckpointError> {
    decode_archive_receipt_bin_checked(bytes)
}

pub fn encode_archive_receipt_json(
    receipt: &ArchiveProviderReceiptV1,
) -> Result<Vec<u8>, CheckpointError> {
    encode_archive_receipt_json_checked(receipt)
}

pub fn decode_archive_receipt_json(
    bytes: &[u8],
) -> Result<ArchiveProviderReceiptV1, CheckpointError> {
    decode_archive_receipt_json_checked(bytes)
}

pub fn encode_retrieval_audit_bin(audit: &RetrievalAuditV1) -> Result<Vec<u8>, CheckpointError> {
    encode_retrieval_audit_bin_checked(audit)
}

pub fn decode_retrieval_audit_bin(bytes: &[u8]) -> Result<RetrievalAuditV1, CheckpointError> {
    decode_retrieval_audit_bin_checked(bytes)
}

pub fn encode_retrieval_audit_json(audit: &RetrievalAuditV1) -> Result<Vec<u8>, CheckpointError> {
    encode_retrieval_audit_json_checked(audit)
}

pub fn decode_retrieval_audit_json(bytes: &[u8]) -> Result<RetrievalAuditV1, CheckpointError> {
    decode_retrieval_audit_json_checked(bytes)
}

pub fn encode_state_snapshot_bin(snapshot: &StateSnapshotV1) -> Result<Vec<u8>, CheckpointError> {
    encode_state_snapshot_bin_checked(snapshot)
}

pub fn decode_state_snapshot_bin(bytes: &[u8]) -> Result<StateSnapshotV1, CheckpointError> {
    decode_state_snapshot_bin_checked(bytes)
}

pub fn encode_state_snapshot_json(snapshot: &StateSnapshotV1) -> Result<Vec<u8>, CheckpointError> {
    encode_state_snapshot_json_checked(snapshot)
}

pub fn decode_state_snapshot_json(bytes: &[u8]) -> Result<StateSnapshotV1, CheckpointError> {
    decode_state_snapshot_json_checked(bytes)
}

pub fn encode_pruning_decision_bin(
    decision: &PruningDecisionV1,
) -> Result<Vec<u8>, CheckpointError> {
    encode_pruning_decision_bin_checked(decision)
}

pub fn decode_pruning_decision_bin(bytes: &[u8]) -> Result<PruningDecisionV1, CheckpointError> {
    decode_pruning_decision_bin_checked(bytes)
}

pub fn encode_pruning_decision_json(
    decision: &PruningDecisionV1,
) -> Result<Vec<u8>, CheckpointError> {
    encode_pruning_decision_json_checked(decision)
}

pub fn decode_pruning_decision_json(bytes: &[u8]) -> Result<PruningDecisionV1, CheckpointError> {
    decode_pruning_decision_json_checked(bytes)
}

#[cfg(test)]
mod tests {
    use super::{
        decode_archive_manifest_bin, decode_archive_manifest_json, decode_archive_receipt_bin,
        decode_archive_receipt_json, decode_art_bin, decode_audit_bin, decode_draft_bin,
        decode_draft_json, decode_exec_bin, decode_exec_json, decode_link_bin, decode_link_json,
        decode_pruning_decision_bin, decode_pruning_decision_json, decode_retrieval_audit_bin,
        decode_retrieval_audit_json, decode_state_snapshot_bin, decode_state_snapshot_json,
        encode_archive_manifest_bin, encode_archive_manifest_json, encode_archive_receipt_bin,
        encode_archive_receipt_json, encode_art_bin, encode_audit_bin, encode_draft_bin,
        encode_exec_bin, encode_link_bin, encode_pruning_decision_bin,
        encode_pruning_decision_json, encode_retrieval_audit_bin, encode_retrieval_audit_json,
        encode_state_snapshot_bin, encode_state_snapshot_json,
    };
    use crate::{
        checkpoint::audit::{CheckpointAudit, CheckpointAuditVersion},
        checkpoint::{
            ArchiveBackend, ArchiveManifestVersion, ArchiveProviderReceiptV1,
            ArchiveProviderReceiptVersion, CheckpointArchiveManifestV1, CheckpointArtifact,
            CheckpointDraft, CheckpointExecInput, CheckpointExecInputId, CheckpointExecOut,
            CheckpointExecTx, CheckpointExecVersion, CheckpointId, CheckpointInRef, CheckpointLink,
            CheckpointLinkVersion, CheckpointVersion, CreatedEnt, PruningDecisionV1,
            PruningDecisionVersion, PruningNodeClass, RetrievalAuditV1, RetrievalAuditVersion,
            SpentEnt, StateSnapshotV1, StateSnapshotVersion,
        },
        settlement::CheckRoot,
        snapshot::PrepSnapshotId,
        CheckpointError,
    };
    use z00z_core::assets::AssetLeaf;

    fn root(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    fn draft() -> CheckpointDraft {
        CheckpointDraft::new(
            CheckpointVersion::CURRENT,
            31,
            CheckRoot::new([1u8; 32]),
            CheckRoot::new([2u8; 32]),
            vec![SpentEnt::new([3u8; 32])],
            vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
        )
    }

    fn artifact() -> CheckpointArtifact {
        let draft = draft();
        let proof = draft
            .attest_proof(
                PrepSnapshotId::new([7u8; 32]),
                CheckpointExecInputId::new([8u8; 32]),
            )
            .expect("proof");
        draft.finalize(proof).expect("artifact")
    }

    fn link() -> CheckpointLink {
        CheckpointLink::new(
            CheckpointLinkVersion::CURRENT,
            CheckpointId::new([6u8; 32]),
            PrepSnapshotId::new([7u8; 32]),
            CheckpointExecInputId::new([8u8; 32]),
        )
        .expect("link")
    }

    fn exec() -> CheckpointExecInput {
        CheckpointExecInput::new(
            CheckpointExecVersion::CURRENT,
            PrepSnapshotId::new([9u8; 32]),
            CheckRoot::new([1u8; 32]),
            vec![CheckpointExecTx::new(
                vec![CheckpointInRef::new(
                    [2u8; 32],
                    crate::settlement::SerialId::new(7),
                )],
                vec![CheckpointExecOut::new(
                    crate::settlement::DefinitionId::new([3u8; 32]),
                    crate::settlement::TerminalLeaf::from(AssetLeaf::dummy_for_scan(7)),
                )
                .expect("exec out")],
                vec![3u8],
            )
            .expect("exec tx")],
        )
        .expect("exec")
    }

    fn audit() -> CheckpointAudit {
        CheckpointAudit::new(
            CheckpointAuditVersion::CURRENT,
            CheckpointId::new([1u8; 32]),
            vec![String::from("frag-1")],
        )
        .expect("audit")
    }

    fn archive_manifest() -> CheckpointArchiveManifestV1 {
        CheckpointArchiveManifestV1::new(
            ArchiveManifestVersion::CURRENT,
            root(11),
            root(12),
            root(13),
            root(14),
            root(15),
            root(16),
            root(17),
            root(18),
            root(19),
            root(20),
            3,
        )
        .expect("archive manifest")
    }

    fn archive_receipt() -> ArchiveProviderReceiptV1 {
        ArchiveProviderReceiptV1::new(
            ArchiveProviderReceiptVersion::CURRENT,
            ArchiveBackend::IpfsPinned,
            root(21),
            4096,
            root(22),
            root(23),
            true,
            true,
        )
        .expect("archive receipt")
    }

    fn retrieval_audit() -> RetrievalAuditV1 {
        RetrievalAuditV1::new(
            RetrievalAuditVersion::CURRENT,
            1000,
            1000,
            root(24),
            root(25),
            root(26),
            [0u8; 32],
            3,
            true,
        )
        .expect("retrieval audit")
    }

    fn state_snapshot() -> StateSnapshotV1 {
        StateSnapshotV1::new(
            StateSnapshotVersion::CURRENT,
            10_000,
            10,
            10_000,
            root(27),
            root(28),
            root(29),
            root(30),
            root(31),
            root(32),
            root(33),
            root(34),
        )
        .expect("state snapshot")
    }

    fn pruning_decision() -> PruningDecisionV1 {
        PruningDecisionV1::new(
            PruningDecisionVersion::CURRENT,
            PruningNodeClass::FullNode,
            "local_full_node_only",
            10,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        )
        .expect("pruning decision")
    }

    #[test]
    fn test_codec_roundtrip_bin() {
        assert_eq!(
            decode_draft_bin(&encode_draft_bin(&draft()).expect("draft bin")).expect("draft"),
            draft()
        );
        assert_eq!(
            decode_art_bin(&encode_art_bin(&artifact()).expect("art bin")).expect("artifact"),
            artifact()
        );
        assert_eq!(
            decode_link_bin(&encode_link_bin(&link()).expect("link bin")).expect("link"),
            link()
        );
        assert_eq!(
            decode_exec_bin(&encode_exec_bin(&exec()).expect("exec bin")).expect("exec"),
            exec()
        );
        assert_eq!(
            decode_audit_bin(&encode_audit_bin(&audit()).expect("audit bin")).expect("audit"),
            audit()
        );
        assert_eq!(
            decode_archive_manifest_bin(
                &encode_archive_manifest_bin(&archive_manifest()).expect("archive manifest bin")
            )
            .expect("archive manifest"),
            archive_manifest()
        );
        assert_eq!(
            decode_archive_receipt_bin(
                &encode_archive_receipt_bin(&archive_receipt()).expect("archive receipt bin")
            )
            .expect("archive receipt"),
            archive_receipt()
        );
        assert_eq!(
            decode_retrieval_audit_bin(
                &encode_retrieval_audit_bin(&retrieval_audit()).expect("retrieval audit bin")
            )
            .expect("retrieval audit"),
            retrieval_audit()
        );
        assert_eq!(
            decode_state_snapshot_bin(
                &encode_state_snapshot_bin(&state_snapshot()).expect("state snapshot bin")
            )
            .expect("state snapshot"),
            state_snapshot()
        );
        assert_eq!(
            decode_pruning_decision_bin(
                &encode_pruning_decision_bin(&pruning_decision()).expect("pruning decision bin")
            )
            .expect("pruning decision"),
            pruning_decision()
        );
    }

    #[test]
    fn test_archive_codec_roundtrip_json() {
        assert_eq!(
            decode_archive_manifest_json(
                &encode_archive_manifest_json(&archive_manifest()).expect("archive manifest json")
            )
            .expect("archive manifest"),
            archive_manifest()
        );
        assert_eq!(
            decode_archive_receipt_json(
                &encode_archive_receipt_json(&archive_receipt()).expect("archive receipt json")
            )
            .expect("archive receipt"),
            archive_receipt()
        );
        assert_eq!(
            decode_retrieval_audit_json(
                &encode_retrieval_audit_json(&retrieval_audit()).expect("retrieval audit json")
            )
            .expect("retrieval audit"),
            retrieval_audit()
        );
        assert_eq!(
            decode_state_snapshot_json(
                &encode_state_snapshot_json(&state_snapshot()).expect("state snapshot json")
            )
            .expect("state snapshot"),
            state_snapshot()
        );
        assert_eq!(
            decode_pruning_decision_json(
                &encode_pruning_decision_json(&pruning_decision()).expect("pruning decision json")
            )
            .expect("pruning decision"),
            pruning_decision()
        );
    }

    #[test]
    fn test_bad_transport_rejects() {
        let err = decode_draft_bin(&[1u8, 2, 3]).expect_err("bad draft transport");

        assert!(matches!(err, CheckpointError::Codec(_)));
    }

    #[test]
    fn test_malformed_root_rejects() {
        let err = decode_exec_json(br#"{"version":1,"prev_root":[1,2],"tx_items":[]}"#)
            .expect_err("bad root must reject");

        assert!(matches!(err, CheckpointError::Codec(_)));
    }

    #[test]
    fn test_malformed_link_id_rejects() {
        let err = decode_link_json(
            br#"{
  "version": 1,
  "checkpoint_id": [1, 2],
  "prep_snapshot_id": [7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7],
  "exec_input_id": [8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8]
}"#,
        )
        .expect_err("bad id must reject");

        assert!(matches!(err, CheckpointError::Codec(_)));
    }

    #[test]
    fn test_malformed_version_tag_rejects() {
        let err = decode_draft_json(
            br#"{
  "version": "bad",
  "height": 31,
  "prev_root": [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
  "new_root": [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
  "spent_delta": [],
  "created_delta": []
}"#,
        )
        .expect_err("bad version tag must reject");

        assert!(matches!(err, CheckpointError::Codec(_)));
    }

    #[test]
    fn test_unsupported_version_rejects() {
        let err = CheckpointExecInput::new(
            CheckpointExecVersion::new(9),
            PrepSnapshotId::new([9u8; 32]),
            CheckRoot::new([1u8; 32]),
            vec![CheckpointExecTx::new(
                vec![CheckpointInRef::new(
                    [2u8; 32],
                    crate::settlement::SerialId::new(7),
                )],
                vec![CheckpointExecOut::new(
                    crate::settlement::DefinitionId::new([3u8; 32]),
                    crate::settlement::TerminalLeaf::from(AssetLeaf::dummy_for_scan(7)),
                )
                .expect("exec out")],
                vec![3u8],
            )
            .expect("exec tx")],
        )
        .expect_err("bad exec version");

        assert!(matches!(err, CheckpointError::VersionMix));
    }
}
