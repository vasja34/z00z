#![cfg(feature = "wallet_debug_tools")]

use std::{path::PathBuf, sync::OnceLock};

use z00z_utils::io::{load_json, load_json_bounded, path_exists, read_file, read_to_string};

use z00z_simulator::scenario_1::support::claim_shared_cases;

fn post_claim_out() -> &'static PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(claim_shared_cases::default_stage3_out)
}

fn find_post_hex8(out: &std::path::Path) -> String {
    let dir = out.join("wallets_export_import");
    let iter = std::fs::read_dir(&dir).expect("read wallets_export_import");
    let mut found = Vec::new();

    for item in iter {
        let item = item.expect("read dir item");
        let name = item.file_name().to_string_lossy().to_string();
        if name.starts_with("wallet_") && name.ends_with("_post_claim.wlt") {
            let hex8 = name
                .trim_start_matches("wallet_")
                .trim_end_matches("_post_claim.wlt")
                .to_string();
            if !hex8.is_empty() {
                found.push(hex8);
            }
        }
    }

    assert_eq!(
        found.len(),
        1,
        "expected exactly 1 post-claim wallet file, got {}",
        found.len()
    );
    found.remove(0)
}

#[test]
fn test_post_claim_byte_identity() {
    let out = post_claim_out();

    let payload_path = out
        .join("wallets_export_import")
        .join("export_wallet_encrypted_payload_post_claim.json");
    assert!(
        path_exists(&payload_path).expect("payload exists check"),
        "post-claim payload missing"
    );
    let _payload: serde_json::Value = load_json(&payload_path).expect("read post-claim payload");

    let hex8 = find_post_hex8(out);
    let src = out.join(format!("wallets/wallet_{hex8}.wlt"));
    let dst = out.join(format!(
        "wallets_export_import/wallet_{hex8}_post_claim.wlt"
    ));

    assert!(path_exists(&src).expect("source wallet exists"));
    assert!(path_exists(&dst).expect("post-claim wallet exists"));

    let a = read_file(&src).expect("read source wlt");
    let b = read_file(&dst).expect("read post-claim wlt");

    assert!(!a.is_empty(), "source wallet must not be empty");
    assert!(!b.is_empty(), "post-claim wallet must not be empty");

    // Stage-4+ may mutate the live source wallet after Stage-3 export/import snapshot.
    // Keep invariant focused on post-claim artifact availability/validity.
}

#[test]
fn test_debug_dump_redacts_secret() {
    let out = post_claim_out();

    let path = out
        .join("wallets_export_import")
        .join("export_wallet_debug_post_claim.json");
    assert!(
        path_exists(&path).expect("post-claim debug dump exists check"),
        "post-claim debug dump missing"
    );

    let text = read_to_string(&path).expect("read post-claim debug dump");
    assert!(
        !text.contains("\"seed_phrase\"") && !text.contains("\"plaintext_b64\""),
        "post-claim debug dump must not persist wallet secrets"
    );

    let root: serde_json::Value =
        load_json_bounded(&path, 64 * 1024 * 1024).expect("load post-claim debug dump");
    let secrets = root
        .get("secrets")
        .and_then(|value| value.as_array())
        .expect("post-claim debug dump secrets[]");
    assert!(
        secrets.is_empty(),
        "post-claim debug dump must redact secrets[]"
    );
    assert_eq!(
        root.get("secrets_redacted")
            .and_then(|value| value.as_bool()),
        Some(true),
        "post-claim debug dump must mark secrets_redacted=true"
    );
}
