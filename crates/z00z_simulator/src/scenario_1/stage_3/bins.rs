use super::{load_bincode, save_bincode, Asset, HashMap, Path, SimContext};

use std::path::PathBuf;
use z00z_utils::io;

pub(crate) fn resolve_actor_idxs(ctx: &SimContext) -> Result<Vec<usize>, String> {
    let mut by_name: HashMap<String, usize> = HashMap::new();
    for (index, actor) in ctx.actors.iter().enumerate() {
        by_name.insert(actor.name.to_lowercase(), index);
    }

    let mut out = Vec::with_capacity(3);
    for expected in ["alice", "bob", "charlie"] {
        let idx = by_name
            .get(expected)
            .copied()
            .ok_or_else(|| format!("stage3 requires actor '{expected}'"))?;
        out.push(idx);
    }

    Ok(out)
}

pub(crate) fn load_stage1_bins(outputs_dir: &Path) -> Result<Vec<Asset>, String> {
    let mut out = Vec::new();

    for path in list_stage1_bins(outputs_dir)? {
        let mut part: Vec<Asset> = load_bincode(&path).map_err(|e| e.to_string())?;
        out.append(&mut part);
    }

    Ok(out)
}

pub(crate) fn empty_stage1_bins(outputs_dir: &Path) -> Result<(), String> {
    let empty: Vec<Asset> = Vec::new();

    for path in list_stage1_bins(outputs_dir)? {
        save_bincode(&path, &empty).map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub(crate) fn has_restored_bins(outputs_dir: &Path) -> bool {
    let Ok(paths) = list_stage1_bins(outputs_dir) else {
        return false;
    };

    for path in paths {
        if let Ok(items) = load_bincode::<Vec<Asset>>(&path) {
            if !items.is_empty() {
                return true;
            }
        }
    }
    false
}

fn list_stage1_bins(outputs_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let base = outputs_dir.join("genesis");
    let entries = io::read_dir(&base).map_err(|e| e.to_string())?;

    let mut files = Vec::new();
    for entry in entries {
        let path = entry;
        if !path.is_file() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name.starts_with("genesis_") && name.ends_with(".bin") {
            files.push(path);
        }
    }

    files.sort();
    if files.is_empty() {
        return Err("no genesis_*.bin files found".to_string());
    }
    Ok(files)
}
