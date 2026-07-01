use std::{
    fs,
    path::{Path, PathBuf},
};

fn collect_rust_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read simulator src directory") {
        let entry = entry.expect("directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

#[test]
fn test_simulator_avoids_deep_imports() {
    let src_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_rust_files(&src_root, &mut files);

    let forbidden = [
        "z00z_wallets::services::",
        "z00z_wallets::db::",
        "z00z_storage::settlement::store_internal",
    ];

    let mut hits = Vec::new();
    for path in files {
        let text = fs::read_to_string(&path).expect("read simulator source file");
        for pattern in forbidden {
            if text.contains(pattern) {
                hits.push(format!("{} -> {pattern}", path.display()));
            }
        }
    }

    assert!(
        hits.is_empty(),
        "simulator must use stable facades only; forbidden deep imports found:\n{}",
        hits.join("\n")
    );
}

#[test]
fn test_simulator_readme_harness_contract() {
    let readme = fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("README.md"))
        .expect("read simulator README");
    for needle in ["integration harness", "stable facades", "scenario contract"] {
        assert!(
            readme.contains(needle),
            "README.md must document the simulator boundary using '{needle}'"
        );
    }
}
