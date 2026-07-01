use std::path::{Path, PathBuf};

use z00z_storage::snapshot::{
    PrepFsStore, PrepReplayEntry, PrepSnapshot, PrepSnapshotId, PrepSnapshotStore,
};
use z00z_utils::io::path_exists;

use crate::scenario_1::stage_6::prep_ref::read_prep_ref;

pub(crate) fn resolve_input_path(out: &Path, path: &str) -> Result<PathBuf, String> {
    let p = PathBuf::from(path);
    if let Some(found) = direct_path(out, &p)? {
        return Ok(found);
    }
    if let Some(found) = search_path(&p)? {
        return Ok(found);
    }
    Ok(out.join(p))
}

fn direct_path(out: &Path, path: &Path) -> Result<Option<PathBuf>, String> {
    let marker = Path::new("crates/z00z_simulator/outputs/scenario_1");
    if path.is_absolute() {
        return Ok(Some(path.to_path_buf()));
    }
    if let Ok(stripped) = path.strip_prefix(marker) {
        return Ok(Some(out.join(stripped)));
    }
    if path_exists(path).map_err(|e| e.to_string())? {
        return Ok(Some(path.to_path_buf()));
    }
    Ok(None)
}

fn search_path(path: &Path) -> Result<Option<PathBuf>, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    for base in cwd.ancestors() {
        let cand = base.join(path);
        if path_exists(&cand).map_err(|e| e.to_string())? {
            return Ok(Some(cand));
        }
    }
    Ok(None)
}

pub(crate) fn load_prep(
    path: &Path,
) -> Result<(PrepSnapshotId, PrepSnapshot, Vec<PrepReplayEntry>), String> {
    let snapshot_id = read_prep_ref(path)?;
    let store = PrepFsStore::new(path.parent().unwrap_or(Path::new(".")));
    let snapshot = store
        .load_snapshot(&snapshot_id)
        .map_err(|e| format!("failed loading checkpoint snapshot: {e}"))?;
    let replay = store
        .replay_entries(&snapshot)
        .map_err(|e| format!("failed building checkpoint replay entries: {e}"))?;
    Ok((snapshot_id, snapshot, replay))
}
