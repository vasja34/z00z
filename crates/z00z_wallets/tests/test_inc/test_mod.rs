#![allow(clippy::duplicate_mod)]

use std::path::PathBuf;
use std::sync::OnceLock;

use z00z_utils::io::{
    create_dir_all, hash_root_inputs, prune_scope_alias_dirs, reset_managed_root_once,
    stable_current_exe_scope,
};

pub fn managed_test_output_root(case_name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("outputs/tests")
        .join(exe_scope());
    prune_output_scope_aliases(&root);
    reset_managed_root_once(&root, &wallet_test_output_fingerprint(), &[], None)
        .expect("reset wallet test output root");
    let case_root = root.join(case_name);
    create_dir_all(&case_root).expect("create wallet test case root");
    case_root
}

fn prune_output_scope_aliases(scope_root: &std::path::Path) {
    let parent = scope_root
        .parent()
        .expect("wallet test output scope parent");
    let scope_name = scope_root
        .file_name()
        .and_then(|name| name.to_str())
        .expect("wallet test output scope name");
    prune_scope_alias_dirs(parent, scope_name).expect("prune wallet test output scope aliases");
}

fn exe_scope() -> String {
    stable_current_exe_scope("unknown_test_binary")
}

fn wallet_test_output_fingerprint() -> String {
    static VALUE: OnceLock<String> = OnceLock::new();
    VALUE
        .get_or_init(|| {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
            hash_root_inputs(
                "wallet-test-output-v1",
                &[
                    root.join("Cargo.toml"),
                    root.join("Cargo.lock"),
                    root.join(".cargo/config.toml"),
                    root.join("crates/z00z_core/Cargo.toml"),
                    root.join("crates/z00z_crypto/Cargo.toml"),
                    root.join("crates/z00z_storage/Cargo.toml"),
                    root.join("crates/z00z_utils/Cargo.toml"),
                    root.join("crates/z00z_wallets/Cargo.toml"),
                ],
                &[
                    root.join("crates/z00z_core/src"),
                    root.join("crates/z00z_crypto/src"),
                    root.join("crates/z00z_storage/src"),
                    root.join("crates/z00z_utils/src"),
                    root.join("crates/z00z_wallets/src"),
                    root.join("crates/z00z_wallets/tests"),
                ],
            )
            .expect("hash wallet test output root")
        })
        .clone()
}
