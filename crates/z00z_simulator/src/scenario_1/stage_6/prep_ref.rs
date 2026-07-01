use std::path::Path;

use serde::{Deserialize, Serialize};
use z00z_storage::snapshot::PrepSnapshotId;
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{read_to_string, save_json},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PrepRefFile {
    snapshot_id_hex: String,
}

pub(crate) fn write_prep_ref(path: &Path, snapshot_id: PrepSnapshotId) -> Result<(), String> {
    let prep_ref = PrepRefFile {
        snapshot_id_hex: hex::encode(snapshot_id.as_bytes()),
    };
    save_json(path, &prep_ref).map_err(|err| {
        format!(
            "failed saving checkpoint prep ref {}: {err}",
            path.display()
        )
    })
}

pub(crate) fn read_prep_ref(path: &Path) -> Result<PrepSnapshotId, String> {
    let json = read_to_string(path).map_err(|err| {
        format!(
            "failed reading checkpoint prep ref {}: {err}",
            path.display()
        )
    })?;
    let prep_ref: PrepRefFile = JsonCodec
        .deserialize(json.as_bytes())
        .map_err(|err| format!("invalid checkpoint prep ref json: {err}"))?;
    let raw = hex::decode(&prep_ref.snapshot_id_hex)
        .map_err(|_| "invalid snapshot_id_hex".to_string())?;
    let bytes: [u8; 32] = raw
        .try_into()
        .map_err(|_| "invalid snapshot_id_hex length".to_string())?;
    Ok(PrepSnapshotId::new(bytes))
}
