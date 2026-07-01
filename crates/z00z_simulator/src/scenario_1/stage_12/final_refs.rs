use std::path::Path;

use z00z_storage::checkpoint::{CheckpointDraftId, CheckpointExecInputId};
use z00z_storage::snapshot::PrepSnapshotId;
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::read_file,
};

use crate::scenario_1::stage_11::Stage11Checkpoint;

pub(super) struct FinalRefs {
    pub(super) draft_id: CheckpointDraftId,
    pub(super) exec_id: CheckpointExecInputId,
    pub(super) snap_id: PrepSnapshotId,
}

pub(super) fn load_stage11_checkpoint(path: &Path) -> Result<Stage11Checkpoint, String> {
    JsonCodec
        .deserialize(read_file(path).map_err(|e| e.to_string())?.as_slice())
        .map_err(|e| format!("invalid stage7 checkpoint decode: {e}"))
}

pub(super) fn parse_refs(checkpoint: &Stage11Checkpoint) -> Result<FinalRefs, String> {
    Ok(FinalRefs {
        draft_id: parse_draft_id(&checkpoint.draft_id_hex)?,
        exec_id: parse_exec_id(&checkpoint.exec_input_id_hex)?,
        snap_id: parse_snap_id(&checkpoint.snapshot_id_hex)?,
    })
}

fn parse_hex32(value: &str) -> Result<[u8; 32], String> {
    let raw = hex::decode(value).map_err(|_| format!("invalid 32-byte hex {value}"))?;
    raw.try_into()
        .map_err(|_| format!("invalid 32-byte hex length {value}"))
}

fn parse_draft_id(value: &str) -> Result<CheckpointDraftId, String> {
    Ok(CheckpointDraftId::new(parse_hex32(value)?))
}

fn parse_exec_id(value: &str) -> Result<CheckpointExecInputId, String> {
    Ok(CheckpointExecInputId::new(parse_hex32(value)?))
}

fn parse_snap_id(value: &str) -> Result<PrepSnapshotId, String> {
    Ok(PrepSnapshotId::new(parse_hex32(value)?))
}
