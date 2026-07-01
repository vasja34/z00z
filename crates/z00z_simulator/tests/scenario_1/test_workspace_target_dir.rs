use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;
use z00z_utils::io::read_to_string;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn cargo_bin() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string())
}

fn cargo_metadata(dir: &Path) -> Value {
    let output = Command::new(cargo_bin())
        .arg("metadata")
        .arg("--format-version=1")
        .arg("--no-deps")
        .current_dir(dir)
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .unwrap_or_else(|err| panic!("run cargo metadata in {}: {err}", dir.display()));

    assert!(
        output.status.success(),
        "cargo metadata failed in {}: {}",
        dir.display(),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|err| panic!("parse cargo metadata in {}: {err}", dir.display()))
}

fn workspace_member_dirs() -> BTreeSet<PathBuf> {
    let root = repo_root();
    let metadata = cargo_metadata(&root);
    let mut dirs = BTreeSet::new();
    dirs.insert(root.clone());

    for package in metadata["packages"]
        .as_array()
        .expect("workspace metadata packages array")
    {
        let manifest = PathBuf::from(
            package["manifest_path"]
                .as_str()
                .expect("workspace package manifest path"),
        );
        let Some(dir) = manifest.parent() else {
            continue;
        };
        if dir.starts_with(root.join("crates")) {
            dirs.insert(dir.to_path_buf());
        }
    }

    dirs
}

fn fuzz_dirs() -> [PathBuf; 3] {
    let root = repo_root();
    [
        root.join("crates/z00z_core/fuzz"),
        root.join("crates/z00z_crypto/fuzz"),
        root.join("crates/z00z_storage/fuzz"),
    ]
}

#[test]
fn test_workspace_target_dir_contract() {
    let root = repo_root();
    let expected_target = normalize_path(&root.join("target/workspace"));
    let cargo_config = read_to_string(root.join(".cargo/config.toml")).expect("read cargo config");

    assert!(
        cargo_config.contains("target-dir = \"target/workspace\""),
        "root cargo config must pin the canonical workspace target dir"
    );

    let mut dirs = workspace_member_dirs();
    dirs.extend(fuzz_dirs());

    for dir in dirs {
        let metadata = cargo_metadata(&dir);
        let target_dir = PathBuf::from(
            metadata["target_directory"]
                .as_str()
                .expect("metadata target_directory"),
        );
        assert_eq!(
            normalize_path(&target_dir),
            expected_target,
            "cargo target_directory drifted for {}",
            dir.display()
        );
    }
}
