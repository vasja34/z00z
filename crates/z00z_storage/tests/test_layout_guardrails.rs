use std::fs;
use std::path::PathBuf;

fn tests_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")
}

#[test]
fn test_store_layout_flat() {
    let root = tests_root();
    let mut dirs: Vec<String> = fs::read_dir(&root)
        .expect("read tests root")
        .filter_map(|entry| {
            let entry = entry.expect("dir entry");
            entry
                .file_type()
                .expect("file type")
                .is_dir()
                .then(|| entry.file_name().to_string_lossy().to_string())
        })
        .collect();
    dirs.sort();

    assert_eq!(
        dirs,
        vec!["fixtures".to_string()],
        "storage tests must stay flat except for the canonical fixture directory"
    );

    for legacy_dir in [
        "assets",
        "checkpoint",
        "snapshot",
        "snapshot_suite",
        "support",
    ] {
        assert!(
            !root.join(legacy_dir).exists(),
            "legacy nested test dir must stay absent: {legacy_dir}"
        );
    }
}

#[test]
fn test_keep_flat_support_files() {
    let root = tests_root();
    for file in [
        "test_checkpoint_fixtures.inc",
        "test_guardrail_support.inc",
        "test_settlement_corpus_support.inc",
        "test_snapshot_mod.inc",
        "test_snapshot_fix.inc",
    ] {
        assert!(
            !root.join(file).exists(),
            "legacy flat support file must stay absent: {file}"
        );
    }

    for file in [
        "test_snapshot_ids.rs",
        "test_snapshot_leaf_hash.rs",
        "test_snapshot_ordering.rs",
        "test_snapshot_path_bind.rs",
        "test_snapshot_persist.rs",
        "test_snapshot_replay_bound.rs",
        "test_snapshot_root_bind.rs",
        "test_snapshot_versions.rs",
        "test_snapshot_wit_decode.rs",
    ] {
        assert!(
            root.join(file).is_file(),
            "missing canonical flat snapshot integration test: {file}"
        );
    }
}
