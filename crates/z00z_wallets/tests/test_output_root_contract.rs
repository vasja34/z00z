#[path = "test_inc/test_mod.rs"]
mod test_common;

use std::sync::{Mutex, MutexGuard};

use test_common::managed_test_output_root;
use z00z_utils::io::{
    create_dir_all, read_to_string, remove_dir_all, stable_current_exe_scope, write_file,
};

const TEST_COMMON_SRC: &str = include_str!("test_inc/test_mod.rs");

fn contract_lock() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().expect("wallet output contract lock")
}

#[test]
fn test_output_scope_contract() {
    let _guard = contract_lock();
    for needle in [
        "wallet-test-output-v1",
        "\"Cargo.toml\"",
        "\"Cargo.lock\"",
        "\".cargo/config.toml\"",
        "\"crates/z00z_core/src\"",
        "\"crates/z00z_crypto/src\"",
        "\"crates/z00z_storage/src\"",
        "\"crates/z00z_utils/src\"",
        "\"crates/z00z_wallets/src\"",
        "\"crates/z00z_wallets/tests\"",
        "prune_scope_alias_dirs",
        "stable_current_exe_scope(\"unknown_test_binary\")",
    ] {
        assert!(
            TEST_COMMON_SRC.contains(needle),
            "wallet test output fingerprint contract must include {needle}"
        );
    }
}

#[test]
fn test_root_clears_stale_scope() {
    let _guard = contract_lock();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("outputs/tests")
        .join(stable_current_exe_scope("unknown_test_binary"));
    if root.exists() {
        remove_dir_all(&root).expect("clear prior wallet test outputs");
    }
    let stale_case = root.join("stale-case");
    create_dir_all(&stale_case).expect("create stale wallet case");
    write_file(root.join(".managed-root-fingerprint"), b"stale-root").expect("write stale mark");
    write_file(stale_case.join("stale.txt"), b"old").expect("write stale payload");

    let keep_case = managed_test_output_root("e2e-contract");
    write_file(keep_case.join("fresh.txt"), b"fresh").expect("write fresh payload");

    assert!(
        !stale_case.exists(),
        "fingerprint drift must clear stale wallet output siblings"
    );
    assert_eq!(
        read_to_string(keep_case.join("fresh.txt")).expect("read fresh wallet payload"),
        "fresh"
    );

    remove_dir_all(&root).expect("cleanup wallet test outputs");
}

#[test]
fn test_prunes_hash_scope_aliases() {
    let _guard = contract_lock();
    let keep_case = "wallet-output-contract-scope-alias";
    let keep_dir = managed_test_output_root(keep_case);
    let scope_dir = keep_dir.parent().expect("scope dir").to_path_buf();
    let scope_root = scope_dir.parent().expect("scope root").to_path_buf();
    let scope_name = scope_dir
        .file_name()
        .and_then(|name| name.to_str())
        .expect("scope name");
    let alias_dir = scope_root.join(format!("{scope_name}-9d4bd19d594c355f"));

    if scope_dir.exists() {
        remove_dir_all(&scope_dir).expect("clear canonical wallet test scope");
    }
    if alias_dir.exists() {
        remove_dir_all(&alias_dir).expect("clear stale wallet test alias scope");
    }

    create_dir_all(alias_dir.join(keep_case)).expect("create stale wallet test alias");
    write_file(alias_dir.join(".managed-root-fingerprint"), b"stale-root")
        .expect("write alias mark");
    write_file(alias_dir.join(keep_case).join("marker.txt"), b"old").expect("write alias marker");

    let out = managed_test_output_root(keep_case);
    write_file(out.join("fresh.txt"), b"fresh").expect("write fresh wallet payload");

    assert!(
        !alias_dir.exists(),
        "wallet test output scope rebuild must drop stale hash-suffixed aliases"
    );
    assert_eq!(
        read_to_string(out.join("fresh.txt")).expect("read fresh wallet payload"),
        "fresh"
    );

    if scope_dir.exists() {
        remove_dir_all(&scope_dir).expect("cleanup wallet test outputs");
    }
}

#[test]
fn test_output_root_reuses_match() {
    let _guard = contract_lock();
    let first = managed_test_output_root("e2e-contract-keep");
    write_file(first.join("keep.txt"), b"keep").expect("write keep file");

    let second = managed_test_output_root("e2e-contract-keep");

    assert_eq!(first, second);
    assert_eq!(
        read_to_string(second.join("keep.txt")).expect("read keep file"),
        "keep"
    );
}
