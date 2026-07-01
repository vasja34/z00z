#![cfg(not(target_arch = "wasm32"))]
#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashMap;
use std::sync::Arc;

use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcTransport};
use z00z_utils::codec::json;
use z00z_utils::rng::SystemRngProvider;
use z00z_utils::time::SystemTimeProvider;

use z00z_wallets::rpc::{
    logging::{LoggedRpcTransport, RpcLoggingConfig},
    methods::{
        AppRpcImpl, AssetRpcImpl, BackupRpcImpl, ChainRpcImpl, ChainScanRpcImpl, KeyRpcImpl,
        NetworkRpcImpl, StorageRpcImpl, TxRpcImpl, WalletRpcImpl,
    },
    register_all_wallet_rpc_methods,
};
use z00z_wallets::services::{AppService, WalletService};

#[path = "test_inc/test_rpc_logger.inc"]
mod test_common;

fn should_dump_rpc_logs() -> bool {
    std::env::var("Z00Z_TEST_RPC_LOGS").is_ok()
}

fn extract_json_vec_logger_line(line: &str) -> Option<String> {
    // VecLogger format: [LEVEL] {json}
    let (_prefix, json) = line.split_once("] ")?;
    Some(json.to_string())
}

fn parse_json_vec_logger_line(line: &str) -> Option<serde_json::Value> {
    let (_, json) = line.split_once("] ")?;
    serde_json::from_str(json).ok()
}

#[tokio::test]
async fn test_rpc_logging_risk_levels() {
    let dir = tempfile::tempdir().unwrap();
    let service = Arc::new(WalletService::with_output_dir(dir.path().join("wallets")));

    let wallet_rpc = Arc::new(WalletRpcImpl::new(Arc::clone(&service)));
    let app_service = Arc::new(AppService::with_wallet_service(Arc::clone(&service)));
    let app_rpc = Arc::new(AppRpcImpl::new(Arc::clone(&app_service)));
    let asset_rpc = Arc::new(AssetRpcImpl::new());
    let tx_rpc = Arc::new(TxRpcImpl::new(Arc::clone(&service)));
    let backup_rpc = Arc::new(BackupRpcImpl::new(Arc::clone(&service)));
    let key_rpc = Arc::new(KeyRpcImpl::new(Arc::clone(&service)));
    let chain_rpc = Arc::new(ChainRpcImpl::new(Arc::clone(&app_service)));
    let network_rpc = Arc::new(NetworkRpcImpl::with_app_service(Arc::clone(&app_service)));
    let scan_rpc = Arc::new(ChainScanRpcImpl::new(Arc::clone(&app_service)));
    let storage_rpc = Arc::new(StorageRpcImpl::new(Arc::clone(&service)));

    let dispatcher = Arc::new(RpcDispatcher::new());
    register_all_wallet_rpc_methods(
        &dispatcher,
        Arc::clone(&app_rpc),
        Arc::clone(&wallet_rpc),
        Arc::clone(&asset_rpc),
        Arc::clone(&tx_rpc),
        Arc::clone(&backup_rpc),
        Arc::clone(&key_rpc),
        Arc::clone(&chain_rpc),
        Arc::clone(&network_rpc),
        Arc::clone(&scan_rpc),
        Arc::clone(&storage_rpc),
    )
    .expect("wallet RPC registration should succeed");

    let base = LocalRpcTransport::new(dispatcher);

    let (logger, vec_logger) = test_common::rpc_test_tee_logger();
    let time = Arc::new(SystemTimeProvider);
    let rng = SystemRngProvider;

    let config =
        RpcLoggingConfig::from_default_wallet_yaml().expect("RPC logging config must load");

    let transport = LoggedRpcTransport::new(base, config, logger, time, rng);

    let password = "StrongPassw0rd!";
    let wrong_unlock_password = "WrongUnlockPassw0rd!";
    let wrong_show_password = "WrongShowPassw0rd!";
    let bad_session_token = "raw-session-token-must-not-leak";
    let verify_secret = r#"{"memo":"VERIFY_SECRET_MEMO","seed_phrase":"verify seed phrase"}"#;
    let import_secret =
        r#"{"memo":"IMPORT_SECRET_MEMO","receiver_secret":"import receiver secret"}"#;

    let created = transport
        .call(
            "app.wallet.create_wallet",
            json!({
                "name": "Risk Policy Wallet",
                "password": password,
                "seed_phrase": null
            }),
        )
        .await
        .unwrap();

    let wallet_id_value = &created["wallet_id"];
    let wallet_id = wallet_id_value
        .as_str()
        .or_else(|| wallet_id_value.get(0).and_then(|v| v.as_str()))
        .or_else(|| wallet_id_value.get("0").and_then(|v| v.as_str()))
        .unwrap_or_else(|| panic!("unexpected wallet_id shape: {wallet_id_value:?}"));

    // High
    let session = transport
        .call(
            "wallet.session.unlock_wallet",
            json!({"wallet_id": wallet_id, "password": password}),
        )
        .await
        .expect("unlock_wallet must succeed");

    let _ = transport
        .call(
            "wallet.session.unlock_wallet",
            json!({"wallet_id": wallet_id, "password": wrong_unlock_password}),
        )
        .await;

    // Critical
    let _ = transport
        .call(
            "wallet.session.show_seed_phrase",
            json!({
                "session": session.clone(),
                "password": password,
                "confirmation": "I understand"
            }),
        )
        .await;

    let _ = transport
        .call(
            "wallet.session.show_seed_phrase",
            json!({
                "session": session.clone(),
                "password": wrong_show_password,
                "confirmation": "I understand"
            }),
        )
        .await;

    // Critical (may succeed or fail depending on implementation; logging must still be correct)
    let _ = transport
        .call(
            "wallet.key.rotate_master_key",
            json!({
                "session": session.clone(),
                "password": password,
                "confirmation": "ROTATE"
            }),
        )
        .await;

    let bad_session = json!({
        "wallet_id": wallet_id,
        "token": bad_session_token,
        "created_at": 0,
        "expires_at": 0,
        "last_activity_at": 0,
        "permissions": []
    });
    let _ = transport
        .call(
            "wallet.key.list_receivers",
            json!({
                "session": bad_session,
                "limit": 1,
                "cursor": null,
                "filter": null
            }),
        )
        .await;

    let _ = transport
        .call(
            "wallet.lifecycle.on_event",
            json!({"event": "backgrounded"}),
        )
        .await;

    let _ = transport
        .call(
            "wallet.key.list_receivers",
            json!({
                "session": session.clone(),
                "limit": 1,
                "cursor": null,
                "filter": null
            }),
        )
        .await;

    // Medium
    let _ = transport
        .call(
            "wallet.key.validate_receiver_card",
            json!({"card_compact": "stub-card"}),
        )
        .await;

    // Low
    let _ = transport
        .call(
            "wallet.asset.list_assets",
            json!({
                "wallet_id": wallet_id,
                "limit": 10,
                "cursor": null,
                "filter": null
            }),
        )
        .await;

    // High
    let _ = transport
        .call(
            "wallet.tx.send_transaction",
            json!({
                "wallet_id": wallet_id,
                "asset_id": vec![0u8; 32],
                "recipient": "invalid-recipient-log-fixture",
                "amount": 1,
                "memo": "SECRET_MEMO",
                "idempotency_key": "idempotency-key-123"
            }),
        )
        .await;

    let _ = transport
        .call(
            "wallet.tx.verify_transaction_package",
            json!({
                "session": session.clone(),
                "tx_data": verify_secret
            }),
        )
        .await;

    let _ = transport
        .call(
            "wallet.tx.import_transaction",
            json!({
                "session": session.clone(),
                "tx_data": import_secret
            }),
        )
        .await;

    // Secret-leak regression: ensure literal secrets are absent.
    let lines = vec_logger.lines();
    assert!(!lines.is_empty());

    let json_lines: Vec<String> = lines
        .iter()
        .filter_map(|line| extract_json_vec_logger_line(line))
        .collect();
    let _ = json_lines;

    if should_dump_rpc_logs() {
        println!("\n--- BEGIN VecLogger RPC logs (risk policy) ---");
        for line in &lines {
            println!("{line}");
        }
        println!("--- END VecLogger RPC logs (risk policy) ---\n");
    }
    for line in &lines {
        assert!(!line.contains(password), "password must not appear in logs");
        assert!(
            !line.contains(wrong_unlock_password),
            "wrong unlock password must not appear in logs"
        );
        assert!(
            !line.contains(wrong_show_password),
            "wrong show-seed password must not appear in logs"
        );
        assert!(
            !line.contains(bad_session_token),
            "raw session token must not appear in logs"
        );
        assert!(
            !line.contains("SECRET_MEMO"),
            "memo must not appear in logs"
        );
        assert!(
            !line.contains("VERIFY_SECRET_MEMO"),
            "verify payload memo must not appear in logs"
        );
        assert!(
            !line.contains("verify seed phrase"),
            "verify payload seed phrase must not appear in logs"
        );
        assert!(
            !line.contains("IMPORT_SECRET_MEMO"),
            "import payload memo must not appear in logs"
        );
        assert!(
            !line.contains("import receiver secret"),
            "import payload receiver secret must not appear in logs"
        );
    }

    // Wallet ID redaction: top-level wallet_id must be truncated (never full-length).
    for line in &lines {
        let v = match parse_json_vec_logger_line(line) {
            Some(v) => v,
            None => continue,
        };

        let method = v.get("method").and_then(|v| v.as_str()).unwrap_or("");
        if method != "wallet.tx.send_transaction" {
            continue;
        }

        let logged_wallet_id = v.get("wallet_id").and_then(|v| v.as_str()).unwrap_or("");
        if logged_wallet_id.is_empty() {
            continue;
        }

        assert_ne!(
            logged_wallet_id, wallet_id,
            "wallet_id must not be logged in full"
        );
        assert!(
            logged_wallet_id.starts_with("<len=")
                && logged_wallet_id.contains("...")
                && logged_wallet_id.ends_with('>'),
            "wallet_id must be truncated; got: {logged_wallet_id}"
        );
    }

    // Risk-level classification: ensure each method has the expected risk.
    let mut seen: HashMap<String, String> = HashMap::new();

    for line in &lines {
        let v = match parse_json_vec_logger_line(line) {
            Some(v) => v,
            None => continue,
        };

        let method = match v.get("method").and_then(|v| v.as_str()) {
            Some(m) => m.to_string(),
            None => continue,
        };
        let risk = match v.get("risk").and_then(|v| v.as_str()) {
            Some(r) => r.to_string(),
            None => continue,
        };

        // Store the first observed risk per method.
        seen.entry(method).or_insert(risk);
    }

    assert_eq!(
        seen.get("app.wallet.create_wallet").map(String::as_str),
        Some("high")
    );
    assert_eq!(
        seen.get("wallet.session.unlock_wallet").map(String::as_str),
        Some("high")
    );
    assert_eq!(
        seen.get("wallet.session.show_seed_phrase")
            .map(String::as_str),
        Some("critical")
    );
    assert_eq!(
        seen.get("wallet.key.rotate_master_key").map(String::as_str),
        Some("critical")
    );
    assert_eq!(
        seen.get("wallet.key.validate_receiver_card")
            .map(String::as_str),
        Some("medium")
    );
    assert_eq!(
        seen.get("wallet.asset.list_assets").map(String::as_str),
        Some("low")
    );
    assert_eq!(
        seen.get("wallet.tx.send_transaction").map(String::as_str),
        Some("high")
    );
}
