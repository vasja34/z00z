#![cfg(not(target_arch = "wasm32"))]

use std::{
    fs,
    path::{Path, PathBuf},
};

fn workspace_root() -> PathBuf {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or(crate_dir)
}

fn tracked_paths() -> Vec<PathBuf> {
    let root = workspace_root();
    [
        "crates/z00z_runtime/aggregators/src",
        "crates/z00z_runtime/aggregators/tests",
        "crates/z00z_runtime/aggregators/README.md",
        "crates/z00z_rollup_node/src",
        "crates/z00z_rollup_node/tests",
        "crates/z00z_runtime/validators/tests",
        "crates/z00z_runtime/watchers/src",
        "crates/z00z_runtime/watchers/tests",
        "crates/z00z_simulator/src",
        "crates/z00z_simulator/tests",
        "config/hjmt_runtime/sim_5a7s",
        "crates/z00z_storage/scripts/run_storage_settlement_bench.py",
    ]
    .into_iter()
    .map(|relative| root.join(relative))
    .collect()
}

fn is_scanned_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("rs" | "md" | "yaml" | "yml" | "json" | "py")
    )
}

fn collect_files(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_file() {
        if is_scanned_file(path)
            && path.file_name().and_then(|name| name.to_str())
                != Some("test_secondary_terminology_guard.rs")
        {
            files.push(path.to_path_buf());
        }
        return;
    }

    let entries = fs::read_dir(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| panic!("failed to read entry: {error}"));
        let child = entry.path();
        if child.is_dir() {
            collect_files(&child, files);
        } else if is_scanned_file(&child)
            && child.file_name().and_then(|name| name.to_str())
                != Some("test_secondary_terminology_guard.rs")
        {
            files.push(child);
        }
    }
}

fn is_ident(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn contains_token(source: &str, needle: &str) -> bool {
    for (offset, _) in source.match_indices(needle) {
        let before = offset
            .checked_sub(1)
            .and_then(|index| source.as_bytes().get(index))
            .copied();
        let after = source.as_bytes().get(offset + needle.len()).copied();
        let before_ok = before.is_none_or(|byte| !is_ident(byte));
        let after_ok = after.is_none_or(|byte| !is_ident(byte));
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

#[test]
fn test_secondary_terminology_guard() {
    let forbidden_word = ["stand", "by"].concat();
    let forbidden_exact = [
        concat!("Stand", "byState"),
        concat!("Takeover", "Standby"),
        concat!("stand", "by_ids"),
        concat!("stand", "by_aggregator_ids"),
        concat!("stand", "by_shard_ids"),
        concat!("removed_aggregator_absent_from_", "stand", "by_tables"),
        concat!("stand", "by_join_six_by_seven"),
        concat!("unknown_", "stand", "by_rejects_at_load"),
        concat!("stand", "by_set_updates_digest"),
    ];

    let mut files = Vec::new();
    for path in tracked_paths() {
        collect_files(&path, &mut files);
    }
    files.sort();
    files.dedup();

    let mut findings = Vec::new();
    for path in files {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        let relative = path
            .strip_prefix(workspace_root())
            .unwrap_or(&path)
            .display()
            .to_string();

        if contains_token(&source, &forbidden_word) {
            findings.push(format!("{relative}: forbidden token {}", forbidden_word));
        }
        for needle in forbidden_exact {
            if source.contains(needle) {
                findings.push(format!("{relative}: forbidden literal {needle}"));
            }
        }
    }

    assert!(
        findings.is_empty(),
        "Phase 067 terminology drift detected:\n{}",
        findings.join("\n")
    );
}
