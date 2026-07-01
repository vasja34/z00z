use std::path::Path;

use z00z_utils::{
    codec::{json, Codec, JsonCodec, Value},
    io::{create_dir_all, path_exists, read_file, save_json},
};

use super::storage_view::{storage_root, LEDGER_PATH_FILE};

pub(super) fn save_summary(view_root: &Path, patch: Value) -> Result<(), String> {
    save_json_patch(&view_root.join("summary.json"), &[], patch)
}

pub(super) fn save_ledger_path(
    out: &Path,
    clear_keys: &[&str],
    patch: Value,
) -> Result<(), String> {
    save_json_patch(
        &storage_root(out, "").join(LEDGER_PATH_FILE),
        clear_keys,
        patch,
    )
}

fn save_json_patch(path: &Path, clear_keys: &[&str], patch: Value) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("storage view path parent missing: {}", path.display()))?;
    create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut current = if path_exists(path).map_err(|e| e.to_string())? {
        JsonCodec
            .deserialize(
                read_file(path)
                    .map_err(|e| format!("storage view summary read failed: {e}"))?
                    .as_slice(),
            )
            .map_err(|e| format!("storage view summary decode failed: {e}"))?
    } else {
        json!({})
    };
    clear_json_keys(&mut current, clear_keys);
    merge_json(&mut current, &patch);
    save_json(path, &current).map_err(|e| e.to_string())?;
    Ok(())
}

fn clear_json_keys(value: &mut Value, keys: &[&str]) {
    let Some(map) = value.as_object_mut() else {
        return;
    };
    for key in keys {
        map.remove(*key);
    }
}

fn merge_json(dst: &mut Value, src: &Value) {
    let (Some(dst_map), Some(src_map)) = (dst.as_object_mut(), src.as_object()) else {
        *dst = src.clone();
        return;
    };
    for (key, value) in src_map {
        dst_map.insert(key.clone(), value.clone());
    }
}
