use std::{path::PathBuf, sync::OnceLock};

use z00z_utils::io::{hash_root_inputs, prepare_managed_root};

pub fn stage4_output_root() -> PathBuf {
    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("outputs/scenario_1/tests/e2e18");
    prepare_managed_root(&out, &output_fingerprint()).expect("prepare stage4 output root");
    out
}

fn output_fingerprint() -> String {
    static VALUE: OnceLock<String> = OnceLock::new();
    VALUE
        .get_or_init(|| {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
            hash_root_inputs(
                "scenario-test-output-v1",
                &[
                    root.join("Cargo.toml"),
                    root.join("Cargo.lock"),
                    root.join(".cargo/config.toml"),
                    root.join("crates/z00z_simulator/Cargo.toml"),
                    root.join("crates/z00z_storage/Cargo.toml"),
                    root.join("crates/z00z_wallets/Cargo.toml"),
                    root.join("crates/z00z_utils/Cargo.toml"),
                ],
                &[
                    root.join("crates/z00z_simulator/src"),
                    root.join("crates/z00z_simulator/tests"),
                    root.join("crates/z00z_storage/src"),
                    root.join("crates/z00z_wallets/src"),
                    root.join("crates/z00z_utils/src"),
                ],
            )
            .expect("hash stage4 output root")
        })
        .clone()
}
