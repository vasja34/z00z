use super::*;
use crate::rpc::types::tx::PortableWalletTxPackage;
use z00z_utils::codec::{json, Codec, JsonCodec};

fn truncation() -> RpcLoggingTruncationConfig {
    RpcLoggingTruncationConfig {
        non_secret_min_bytes: 24,
        head_chars: 6,
        tail_chars: 6,
    }
}

fn tx_package_fixture() -> String {
    include_str!("../../tests/fixtures/tx_package.json").to_string()
}

fn portable_package_fixture() -> String {
    let codec = JsonCodec;
    let tx_json = tx_package_fixture();
    let package: crate::tx::TxPackage = codec
        .deserialize(tx_json.as_bytes())
        .expect("tx package fixture");
    let portable = PortableWalletTxPackage {
        package_version: 1,
        chain_id: package.chain_id.to_string(),
        tx_hash_hex: package.tx_digest_hex,
        tx_bytes: tx_json.into_bytes(),
        metadata_hash_hex: "meta-hash-fixture".to_string(),
    };
    String::from_utf8(codec.serialize(&portable).expect("portable fixture")).expect("utf8")
}

#[test]
fn test_extract_wallet_session() {
    let params = json!({
        "session": {
            "wallet_id": "wallet-session"
        },
        "wallet_id": "wallet-123",
        "id": "wallet-should-not-win"
    });

    assert_eq!(
        extract_wallet_id(&params).as_deref(),
        Some("wallet-session")
    );
}

#[test]
fn test_extract_wallet_canonical() {
    let params = json!({
        "wallet_id": "wallet-abc"
    });

    assert_eq!(extract_wallet_id(&params).as_deref(), Some("wallet-abc"));
}

#[test]
fn test_extract_wallet_nested() {
    let params = json!({
        "session": {
            "wallet_id": "wallet-nested",
            "token": "secret"
        }
    });

    assert_eq!(extract_wallet_id(&params).as_deref(), Some("wallet-nested"));
}

#[test]
fn test_extract_wallet_top_level() {
    let params = json!({
        "wallet_id": "wallet-top-level",
        "id": "wallet-stale"
    });

    assert_eq!(
        extract_wallet_id(&params).as_deref(),
        Some("wallet-top-level")
    );
}

#[test]
fn test_non_secret_truncation() {
    let input = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let out = truncate_non_secret(input, &truncation());
    assert!(out.starts_with("<len="));
    assert!(out.contains(" ABCDEF...456789>"));
    assert!(!out.contains(input));
}

#[test]
fn test_secrets_redacted() {
    let params = json!({
        "name": "VeryLongWalletNameThatShouldBeTruncatedBecauseItIsNotSecret",
        "password": "P".repeat(200),
        "seed_phrase": "S".repeat(200)
    });

    let out = summarize_params("app.wallet.create_wallet", &params, &truncation()).unwrap();
    let obj = out.as_object().unwrap();

    assert_eq!(
        obj.get("password").and_then(|v| v.as_str()),
        Some("<redacted>")
    );
    assert_eq!(
        obj.get("seed_phrase").and_then(|v| v.as_str()),
        Some("<redacted>")
    );

    let name = obj.get("name").and_then(|v| v.as_str()).unwrap();
    assert!(
        name.starts_with("<len="),
        "non-secret strings should be truncated"
    );
}

#[test]
fn test_session_fingerprint() {
    let response = json!({
        "wallet_id": "stub-wallet-id",
        "token": "super-secret-session-token"
    });

    let out = summarize_response("wallet.session.unlock_wallet", &response, &truncation()).unwrap();
    let obj = out.as_object().unwrap();
    let token = obj.get("session_token").and_then(|v| v.as_str()).unwrap();
    assert!(token.starts_with("<fingerprint:"));
    assert!(!token.contains("super-secret-session-token"));
}

#[test]
fn test_asset_hex_truncation() {
    let asset_id: Vec<u8> = (0u8..32u8).collect();
    let params = json!({
        "session": {
            "wallet_id": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "token": "super-secret-session-token"
        },
        "amount": 123,
        "asset_id": asset_id,
    });

    let out = summarize_params("wallet.tx.send_transaction", &params, &truncation()).unwrap();
    let obj = out.as_object().unwrap();

    let summarized = obj.get("asset_id").and_then(|v| v.as_str()).unwrap();
    assert!(summarized.starts_with("<len=64 "));
    assert!(summarized.contains("..."));
}

#[test]
fn test_tx_data_redacted() {
    let params = json!({
        "session": {"wallet_id": "stub-wallet-id", "token": "super-secret-session-token"},
        "tx_data": "RAW_TX_BLOB_NOT_LOGGED",
    });

    let out = summarize_params("wallet.tx.send_transaction", &params, &truncation()).unwrap();
    let obj = out.as_object().unwrap();

    assert_eq!(
        obj.get("tx_data").and_then(|v| v.as_str()),
        Some("<redacted>")
    );
    assert_eq!(
        obj.get("session_token").and_then(|v| v.as_str()),
        Some("<redacted>")
    );
    assert!(!out.to_string().contains("RAW_TX_BLOB_NOT_LOGGED"));
}

#[test]
fn test_verify_pkg_redaction() {
    let params = json!({
        "session": {"wallet_id": "stub-wallet-id", "token": "super-secret-session-token"},
        "tx_data": tx_package_fixture(),
    });

    let out = summarize_params(
        "wallet.tx.verify_transaction_package",
        &params,
        &truncation(),
    )
    .unwrap();
    let obj = out.as_object().unwrap();
    let summary = obj
        .get("tx_data_summary")
        .and_then(|v| v.as_object())
        .expect("tx summary");

    assert_eq!(
        obj.get("session_token").and_then(|v| v.as_str()),
        Some("<redacted>")
    );
    assert!(summary.get("tx_digest_hex").is_some());
    assert!(summary.get("amount").is_some());
    assert!(summary.get("inputs_count").is_some());
    assert!(!out.to_string().contains("\"tx\":"));
    assert!(!out.to_string().contains("super-secret-session-token"));
}

#[test]
fn test_import_pkg_redaction() {
    let params = json!({
        "session": {"wallet_id": "stub-wallet-id", "token": "super-secret-session-token"},
        "tx_data": portable_package_fixture(),
    });

    let out = summarize_params("wallet.tx.import_transaction", &params, &truncation()).unwrap();
    let obj = out.as_object().unwrap();
    let summary = obj
        .get("tx_data_summary")
        .and_then(|v| v.as_object())
        .expect("portable summary");
    let embedded = summary
        .get("tx_package")
        .and_then(|v| v.as_object())
        .expect("embedded tx summary");

    assert_eq!(
        obj.get("session_token").and_then(|v| v.as_str()),
        Some("<redacted>")
    );
    assert_eq!(
        summary.get("package_version").and_then(|v| v.as_u64()),
        Some(1)
    );
    assert!(embedded.get("tx_digest_hex").is_some());
    assert!(!out.to_string().contains("\"tx_bytes\""));
    assert!(!out.to_string().contains("super-secret-session-token"));
}

#[test]
fn test_verify_resp_redaction() {
    let response = json!({
        "tx_digest_hex": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "package_status": "submitted",
        "is_valid": true,
        "lifecycle": "submitted",
        "import_ready": true,
        "all_owned_spendable": true,
        "owned_outputs": [{"asset_data": "SECRET_ASSET_PAYLOAD"}],
        "errors": ["private detail"],
        "error_codes": ["NoOwnedOutputs"]
    });

    let out = summarize_response(
        "wallet.tx.verify_transaction_package",
        &response,
        &truncation(),
    )
    .unwrap();
    let obj = out.as_object().unwrap();

    assert_eq!(
        obj.get("owned_outputs_count").and_then(|v| v.as_u64()),
        Some(1)
    );
    assert_eq!(obj.get("errors_count").and_then(|v| v.as_u64()), Some(1));
    assert_eq!(
        obj.get("error_codes_count").and_then(|v| v.as_u64()),
        Some(1)
    );
    assert!(!out.to_string().contains("SECRET_ASSET_PAYLOAD"));
    assert!(!out.to_string().contains("private detail"));
}

#[test]
fn test_rotate_key_redaction() {
    let params = json!({
        "session": {"wallet_id": "stub-wallet-id", "token": "super-secret-session-token"},
        "password": "StrongPassw0rd!",
        "confirmation": "ROTATE"
    });

    let out = summarize_params("wallet.key.rotate_master_key", &params, &truncation()).unwrap();
    let obj = out.as_object().unwrap();

    assert_eq!(
        obj.get("password").and_then(|v| v.as_str()),
        Some("<redacted>")
    );
    assert_eq!(
        obj.get("confirmation").and_then(|v| v.as_str()),
        Some("<redacted>")
    );
    assert_eq!(
        obj.get("wallet_id").and_then(|v| v.as_str()),
        Some("stub-wallet-id")
    );
}

#[test]
fn test_rotate_key_top_level() {
    let params = json!({
        "wallet_id": "compat-wallet-id",
        "password": "StrongPassw0rd!",
        "confirmation": "ROTATE"
    });

    let out = summarize_params("wallet.key.rotate_master_key", &params, &truncation()).unwrap();
    let obj = out.as_object().unwrap();

    assert_eq!(
        obj.get("wallet_id").and_then(|v| v.as_str()),
        Some("compat-wallet-id")
    );
    assert_eq!(
        obj.get("password").and_then(|v| v.as_str()),
        Some("<redacted>")
    );
}

#[test]
fn test_rotate_key_confirmation() {
    let params = json!({
        "wallet_id": "compat-wallet-id",
        "password": "StrongPassw0rd!"
    });

    let out = summarize_params("wallet.key.rotate_master_key", &params, &truncation()).unwrap();
    let obj = out.as_object().unwrap();

    assert_eq!(obj.get("confirmation"), None);
    assert_eq!(
        obj.get("password").and_then(|v| v.as_str()),
        Some("<redacted>")
    );
}
