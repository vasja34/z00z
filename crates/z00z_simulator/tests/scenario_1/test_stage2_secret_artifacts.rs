use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

#[cfg(feature = "wallet_debug_tools")]
use std::os::unix::fs::PermissionsExt;

#[cfg(feature = "wallet_debug_tools")]
use z00z_utils::io::{load_json_bounded, read_to_string};

use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::scenario_support;
use z00z_simulator::scenario_1::support::stage_runner_support;

const STAGE2_SRC: &str = include_str!("../../src/scenario_1/stage_2/mod.rs");
const STAGE3_FINALIZE_SRC: &str = include_str!("../../src/scenario_1/stage_3/finalize.rs");
const CONFIG_ACCESSORS_SRC: &str = include_str!("../../src/config/config_accessors.rs");
const SIM_README_SRC: &str = include_str!("../../README.md");
const WALLET_INTEGRATION_TESTS_SRC: &str = include_str!("test_wallet_integration.rs");

static STAGE23_OUT: OnceLock<PathBuf> = OnceLock::new();

fn stage23_out() -> &'static PathBuf {
    STAGE23_OUT.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage2_secret_artifacts_v1", |base| {
            let (cfg_path, design_path, out) = scenario_support::make_cfg_in(base, |_| {});
            let _ctx =
                stage_runner_support::run_stage_setup(&cfg_path, &design_path, &[1_u32, 2, 3]);
            assert!(out.join("wallets").exists());
        });
        root.join("outputs/scenario_1")
    })
}

fn public_secret_path(out: &Path) -> PathBuf {
    out.join("wallets").join("wlt_secrets_debug.md")
}

fn private_secret_path(out: &Path) -> PathBuf {
    out.join("wallets")
        .join("private")
        .join("wlt_secrets_debug.md")
}

#[test]
fn test_stage2_secrets_public_path() {
    let out = stage23_out();

    assert!(
        !public_secret_path(out).exists(),
        "stage2 must never publish wallet secrets on the public wallet lane"
    );
}

#[cfg(feature = "wallet_debug_tools")]
#[test]
fn test_stage2_debug_stay_private() {
    let out = stage23_out();
    let path = private_secret_path(out);

    assert!(
        path.exists(),
        "wallet_debug_tools must keep the debug secret artifact on the private lane"
    );
    let mode = std::fs::metadata(&path)
        .expect("metadata for private debug secret artifact")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(
        mode, 0o600,
        "private debug secret artifact must stay mode 0600"
    );

    let text = read_to_string(&path).expect("read private debug secret artifact");
    assert!(
        text.contains("# Wallet Secrets (Stage 2) [DEBUG]"),
        "private debug secret artifact must keep the explicit debug banner"
    );
    assert!(
        text.contains("Alice_Pass_Z00Z_42!"),
        "wallet_debug_tools artifact must preserve the intentional debug-only password lane"
    );
}

#[cfg(not(feature = "wallet_debug_tools"))]
#[test]
fn test_stage2_secrets_debug_dump() {
    let out = stage23_out();

    assert!(
        !private_secret_path(out).exists(),
        "stage2 must not emit the private debug secret artifact without wallet_debug_tools"
    );
}

#[test]
fn test_default_secret_stays_narrow() {
    assert!(
        STAGE2_SRC.contains("default lane emitted no plaintext wallet secret artifact")
            && STAGE2_SRC.contains("run_export_roundtrip"),
        "stage 2 must keep the claim narrow to the default plaintext artifact lane"
    );
    assert!(
        WALLET_INTEGRATION_TESTS_SRC
            .contains("default simulator lane must not emit a plaintext wallet secret artifact")
            && WALLET_INTEGRATION_TESTS_SRC.contains("wallet_debug_tools feature must keep the private debug secret artifact on the private lane")
            && WALLET_INTEGRATION_TESTS_SRC.contains("wallet_debug_tools is disabled, so the private debug secret artifact must stay absent"),
        "wallet integration tests must keep default-lane silence separate from the debug lane"
    );
}

#[test]
fn test_debug_lane_stays_private() {
    assert!(
        CONFIG_ACCESSORS_SRC.contains("private stage 2 debug secret artifact")
            && CONFIG_ACCESSORS_SRC.contains("wallet_debug_tools")
            && CONFIG_ACCESSORS_SRC
                .contains("wallets_dir.join(\"private\").join(\"wlt_secrets_debug.md\")"),
        "config accessors must keep the debug secret artifact feature-gated and private-path only"
    );
    assert!(
        STAGE3_FINALIZE_SRC.contains("#[cfg(feature = \"wallet_debug_tools\")]")
            && STAGE3_FINALIZE_SRC.contains("export_wallet_debug_toolss"),
        "stage 3 finalize must keep debug secret exports behind wallet_debug_tools"
    );
    assert!(
        SIM_README_SRC.contains("Plaintext wallet-secret artifacts are not part of the default public scenario contract")
            && SIM_README_SRC.contains("behind the `wallet_debug_tools` feature gate")
            && SIM_README_SRC.contains("written to a private-permission path")
            && SIM_README_SRC.contains("absent from the default release-style stage contract")
            && SIM_README_SRC.contains(
                "Encrypted operational export and backup surfaces remain outside this narrower plaintext-debug-artifact claim."
            ),
        "simulator README must keep the debug lane explicit, private, and absent by default"
    );
}

#[cfg(feature = "wallet_debug_tools")]
#[test]
fn test_debug_dumps_redact_secret() {
    let out = stage23_out();

    for name in ["alice", "bob", "charlie"] {
        let path = out
            .join("claim")
            .join(format!("export_wallet_debug_{name}.json"));
        assert!(path.exists(), "missing stage3 debug dump for {name}");

        let text = read_to_string(&path).expect("read stage3 debug dump");
        assert!(
            !text.contains("\"seed_phrase\"") && !text.contains("\"plaintext_b64\""),
            "stage3 claim debug dump must not persist wallet secrets for {name}"
        );

        let root: serde_json::Value =
            load_json_bounded(&path, 64 * 1024 * 1024).expect("load stage3 debug dump");
        let secrets = root
            .get("secrets")
            .and_then(|value| value.as_array())
            .expect("stage3 debug dump secrets[]");
        assert!(
            secrets.is_empty(),
            "stage3 claim debug dump must redact secrets[] for {name}"
        );
        assert_eq!(
            root.get("secrets_redacted")
                .and_then(|value| value.as_bool()),
            Some(true),
            "stage3 claim debug dump must mark secrets_redacted=true for {name}"
        );
    }
}
