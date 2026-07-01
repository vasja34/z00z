use z00z_core::AssetWire;
use z00z_crypto::expert::encoding::to_hex;
use z00z_storage::checkpoint::{
    CheckpointDraft, CheckpointExecInput, CheckpointExecInputId, CheckpointExecOut,
    CheckpointExecTx, CheckpointExecVersion, CheckpointId, CheckpointInRef, CheckpointLink,
    CheckpointLinkVersion, CheckpointProof,
};
use z00z_storage::settlement::{CheckRoot, DefinitionId, SerialId};
use z00z_storage::snapshot::PrepSnapshotId;
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_wallets::tx::{asset_wire_to_leaf, TxInputWire, TxOutputWire, TxPackage};

use super::bridge_output_router::build_made_rows;
use super::bundle_lane_impl::{Checkpoint, FragTx};
use super::fragment_construction::decode_hex32;

pub(crate) fn in_ref_from_input(input: &TxInputWire) -> Result<CheckpointInRef, String> {
    Ok(CheckpointInRef::new(
        decode_hex32(&input.asset_id_hex)?,
        SerialId::new(input.serial_id),
    ))
}

pub(crate) fn exec_out_from_wire(output: &TxOutputWire) -> Result<CheckpointExecOut, String> {
    let asset = output
        .asset_wire
        .clone()
        .to_asset()
        .map_err(|e| format!("stage6: output asset decode failed: {e}"))?;
    let leaf = asset_wire_to_leaf(&AssetWire::from_asset(&asset))
        .map_err(|e| format!("stage6: output leaf build failed: {e}"))?;
    let wire = output
        .asset_wire
        .clone()
        .to_wire()
        .map_err(|e| format!("stage6: output wire decode failed: {e}"))?;

    CheckpointExecOut::new(DefinitionId::new(wire.definition.id), leaf).map_err(|e| e.to_string())
}

pub(crate) fn build_exec_input(
    snap_id: PrepSnapshotId,
    prev_root: CheckRoot,
    pkg: &TxPackage,
    outputs: &[TxOutputWire],
) -> Result<CheckpointExecInput, String> {
    let tx_proof = JsonCodec
        .serialize(&pkg.tx.proof)
        .map_err(|e| format!("stage6: tx proof encode failed: {e}"))?;
    let input_refs = pkg
        .tx
        .inputs
        .iter()
        .map(in_ref_from_input)
        .collect::<Result<Vec<_>, String>>()?;
    let outputs = outputs
        .iter()
        .map(exec_out_from_wire)
        .collect::<Result<Vec<_>, String>>()?;
    let tx = CheckpointExecTx::new(input_refs, outputs, tx_proof).map_err(|e| e.to_string())?;
    CheckpointExecInput::new(CheckpointExecVersion::CURRENT, snap_id, prev_root, vec![tx])
        .map_err(|e| e.to_string())
}

pub(crate) fn draft_link(
    snap_id: PrepSnapshotId,
    exec_id: CheckpointExecInputId,
) -> Result<CheckpointLink, String> {
    CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        CheckpointId::new([0u8; 32]),
        snap_id,
        exec_id,
    )
    .map_err(|e| e.to_string())
}

pub(crate) fn build_attest_proof(
    draft: &CheckpointDraft,
    _pkg: &TxPackage,
    snap_id: PrepSnapshotId,
    exec_id: CheckpointExecInputId,
) -> Result<CheckpointProof, String> {
    // The accepted current-stack path now seals the backend-owned attested
    // checkpoint payload that is derived from the finalized statement itself.
    // This keeps non-authoritative proof/spent-state success closed on the
    // accepted path only.
    // Compatibility-looking proof bytes remain non-authoritative and do not
    // widen this path into recursive-proof backend or broader PH32-SPEND closure.
    draft
        .attest_proof(snap_id, exec_id)
        .map_err(|e| e.to_string())
}

pub(crate) fn checkpoint_from_draft(
    draft: &CheckpointDraft,
    outputs: &[TxOutputWire],
    frags: &[FragTx],
) -> Result<Checkpoint, String> {
    Ok(Checkpoint {
        prev_root_hex: to_hex(draft.prev_root().as_bytes()),
        new_root_hex: to_hex(draft.new_root().as_bytes()),
        spent_delta: draft
            .spent_delta()
            .iter()
            .map(|item| to_hex(item.terminal_id().as_bytes()))
            .collect(),
        created_delta: build_made_rows(outputs)?,
        fragment_ids: frags.iter().map(|frag| frag.id.clone()).collect(),
    })
}
