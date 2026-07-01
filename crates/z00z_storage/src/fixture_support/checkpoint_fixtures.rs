use crate::{
    checkpoint::audit::{CheckpointAudit, CheckpointAuditVersion},
    checkpoint::{
        CheckpointArtifact, CheckpointDraft, CheckpointExecInput, CheckpointExecInputId,
        CheckpointExecOut, CheckpointExecTx, CheckpointExecVersion, CheckpointId, CheckpointInRef,
        CheckpointLink, CheckpointLinkVersion, CheckpointProof, CheckpointVersion, CreatedEnt,
        SpentEnt,
    },
    settlement::{CheckRoot, DefinitionId, SerialId, SettlementStore, TerminalLeaf},
    snapshot::PrepSnapshotId,
};
use serde::Serialize;
use z00z_core::assets::AssetLeaf;
use z00z_utils::codec::{Codec, JsonCodec};

#[derive(Serialize)]
struct PriorMadeEnt {
    asset_id_hex: String,
    leaf_hash_hex: String,
}

#[derive(Serialize)]
struct PriorStage6 {
    prev_root_hex: String,
    new_root_hex: String,
    spent_delta: Vec<String>,
    created_delta: Vec<PriorMadeEnt>,
    fragment_ids: Vec<String>,
}

pub fn draft() -> CheckpointDraft {
    CheckpointDraft::new(
        CheckpointVersion::CURRENT,
        41,
        empty_check_root(),
        CheckRoot::new([2u8; 32]),
        vec![SpentEnt::new([3u8; 32])],
        vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
    )
}

fn empty_check_root() -> CheckRoot {
    CheckRoot::from(
        SettlementStore::new()
            .settlement_root()
            .expect("empty settlement root"),
    )
}

pub fn proof(draft: &CheckpointDraft, _byte: u8) -> CheckpointProof {
    draft
        .attest_proof(
            PrepSnapshotId::new([7u8; 32]),
            CheckpointExecInputId::new([8u8; 32]),
        )
        .expect("proof")
}

pub fn artifact() -> CheckpointArtifact {
    let draft = draft();
    draft.finalize(proof(&draft, 9)).expect("artifact")
}

pub fn exec() -> CheckpointExecInput {
    CheckpointExecInput::new(
        CheckpointExecVersion::CURRENT,
        PrepSnapshotId::new([9u8; 32]),
        empty_check_root(),
        vec![CheckpointExecTx::new(
            vec![CheckpointInRef::new([2u8; 32], SerialId::new(7))],
            vec![CheckpointExecOut::new(
                DefinitionId::new([3u8; 32]),
                TerminalLeaf::from(AssetLeaf::dummy_for_scan(7)),
            )
            .expect("exec out")],
            vec![3u8],
        )
        .expect("exec tx")],
    )
    .expect("exec")
}

pub fn link(checkpoint_id: CheckpointId, exec_id: CheckpointExecInputId) -> CheckpointLink {
    CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        checkpoint_id,
        PrepSnapshotId::new([7u8; 32]),
        exec_id,
    )
    .expect("link")
}

pub fn audit(checkpoint_id: CheckpointId) -> CheckpointAudit {
    CheckpointAudit::new(
        CheckpointAuditVersion::CURRENT,
        checkpoint_id,
        vec![String::from("frag-1")],
    )
    .expect("audit")
}

pub fn prior_stage6_json() -> Vec<u8> {
    JsonCodec
        .serialize(&PriorStage6 {
            prev_root_hex: "11".repeat(32),
            new_root_hex: "22".repeat(32),
            spent_delta: vec!["33".repeat(32)],
            created_delta: vec![PriorMadeEnt {
                asset_id_hex: "44".repeat(32),
                leaf_hash_hex: "55".repeat(32),
            }],
            fragment_ids: vec![String::from("frag_1"), String::from("frag_2")],
        })
        .expect("prior stage6 json")
}
