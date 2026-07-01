#![cfg(not(target_arch = "wasm32"))]
#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcTransport};
use z00z_utils::codec::json;
use z00z_utils::logger::VecLogger;
use z00z_utils::rng::SystemRngProvider;
use z00z_utils::time::SystemTimeProvider;

#[path = "test_inc/test_rpc_logger.inc"]
mod test_common;

use z00z_wallets::rpc::types::common::PersistWalletId;
use z00z_wallets::rpc::{
    logging::{LoggedRpcTransport, RpcLoggingConfig},
    methods::{
        AppRpcImpl, AssetRpcImpl, BackupRpcImpl, ChainRpcImpl, ChainScanRpcImpl, KeyRpcImpl,
        NetworkRpcImpl, StorageRpcImpl, TxRpcImpl, WalletRpcImpl,
    },
    register_all_wallet_rpc_methods,
};
use z00z_wallets::services::{AppService, WalletService};

async fn create_wallet_session<T: RpcTransport>(transport: &T) -> (String, serde_json::Value) {
    let created = transport
        .call(
            "app.wallet.create_wallet",
            json!({
                "name": "My Wallet",
                "password": "StrongPassw0rd!",
                "seed_phrase": null
            }),
        )
        .await
        .unwrap();

    assert_eq!(created["name"], "My Wallet");
    let wallet_id_value = &created["wallet_id"];
    assert!(!wallet_id_value.is_null());
    let wallet_id = wallet_id_value
        .as_str()
        .or_else(|| wallet_id_value.get(0).and_then(|v| v.as_str()))
        .or_else(|| wallet_id_value.get("0").and_then(|v| v.as_str()))
        .unwrap_or_else(|| panic!("unexpected wallet_id shape: {wallet_id_value:?}"))
        .to_string();
    assert!(!wallet_id.is_empty());
    assert!(created["seed_phrase"].as_str().unwrap_or("").len() > 10);

    let session = transport
        .call(
            "wallet.session.unlock_wallet",
            json!({
                "wallet_id": wallet_id,
                "password": "StrongPassw0rd!"
            }),
        )
        .await
        .unwrap();

    (wallet_id, session)
}

async fn check_wallet_list<T: RpcTransport>(transport: &T) {
    let list = transport
        .call("app.wallet.list_wallets", json!({}))
        .await
        .unwrap();
    assert!(!list.as_array().unwrap().is_empty());
}

async fn check_asset_list<T: RpcTransport>(transport: &T, wallet_id: &str) {
    let assets = transport
        .call(
            "wallet.asset.list_assets",
            json!({
                "wallet_id": wallet_id,
                "limit": 10,
                "cursor": null,
                "filter": null
            }),
        )
        .await
        .unwrap();

    assert!(assets["assets"].is_array());
}

async fn create_backup<T: RpcTransport>(transport: &T, session: &serde_json::Value) -> String {
    let create_backup = transport
        .call(
            "wallet.backup.create_backup",
            json!({
                "session": session,
                "password": "StrongPassw0rd!",
                "destination": null
            }),
        )
        .await
        .unwrap();

    assert!(
        create_backup["success"].as_bool().unwrap_or(false),
        "create_backup should report success: {create_backup:?}"
    );
    let backup_path = create_backup["backup_path"]
        .as_str()
        .unwrap_or_else(|| panic!("missing backup_path in response: {create_backup:?}"));
    assert!(!backup_path.is_empty());
    backup_path.to_string()
}

async fn check_backup_list<T: RpcTransport>(transport: &T, session: &serde_json::Value) {
    let listed = transport
        .call(
            "wallet.backup.list_backups",
            json!({
                "session": session,
                "cursor": null,
                "limit": 10
            }),
        )
        .await
        .unwrap();

    assert_eq!(listed["items"].as_array().map(|items| items.len()), Some(1));
}

async fn restore_backup<T: RpcTransport>(transport: &T, backup_path: &str, wallet_id: &str) {
    let restored = transport
        .call(
            "wallet.backup.restore_backup",
            json!({
                "backup_path": backup_path,
                "password": "StrongPassw0rd!",
                "wallet_name": null
            }),
        )
        .await
        .unwrap();

    assert!(
        restored["success"].as_bool().unwrap_or(false),
        "restore_backup should report success: {restored:?}"
    );
    assert_eq!(restored["wallet_id"], wallet_id);
}

async fn check_backup_roundtrip<T: RpcTransport>(
    transport: &T,
    service: &Arc<WalletService>,
    wallet_id: &str,
    session: &serde_json::Value,
) {
    let backup_path = create_backup(transport, session).await;
    check_backup_list(transport, session).await;

    service
        .unregister_wallet(&PersistWalletId(wallet_id.to_string()))
        .await
        .unwrap();

    restore_backup(transport, &backup_path, wallet_id).await;
}

async fn test_check_send_roundtrip<T: RpcTransport>(transport: &T, session: &serde_json::Value) {
    let send = transport
        .call(
            "wallet.key.get_receiver_card",
            json!({
                "session": session
            }),
        )
        .await
        .unwrap();

    let recipient = send["card_compact"]
        .as_str()
        .unwrap_or_else(|| panic!("missing card_compact in response: {send:?}"));

    let send = transport
        .call(
            "wallet.asset.send_asset",
            json!({
                "session": session,
                "asset_id": vec![0u8; 32],
                "recipient": recipient,
                "amount": 1
            }),
        )
        .await
        .unwrap();

    assert_eq!(send["status"], "stealth_submitted");
    assert!(send["asset_id"].as_array().is_some());
    assert_eq!(send["amount"], 1);
    assert!(send["owner_handle"].as_str().unwrap_or("").len() == 64);
}

fn make_transport(
    service: &Arc<WalletService>,
) -> (
    LoggedRpcTransport<LocalRpcTransport, SystemRngProvider>,
    Arc<VecLogger>,
) {
    let wallet_rpc = Arc::new(WalletRpcImpl::new(Arc::clone(service)));
    let app_service = Arc::new(AppService::with_wallet_service(Arc::clone(service)));
    let app_rpc = Arc::new(AppRpcImpl::new(Arc::clone(&app_service)));
    let asset_rpc = Arc::new(AssetRpcImpl::with_wallet_service(Arc::clone(service)));
    let tx_rpc = Arc::new(TxRpcImpl::new(Arc::clone(service)));
    let backup_rpc = Arc::new(BackupRpcImpl::new(Arc::clone(service)));
    let key_rpc = Arc::new(KeyRpcImpl::new(Arc::clone(service)));
    let chain_rpc = Arc::new(ChainRpcImpl::new(Arc::clone(&app_service)));
    let network_rpc = Arc::new(NetworkRpcImpl::with_app_service(Arc::clone(&app_service)));
    let scan_rpc = Arc::new(ChainScanRpcImpl::new(Arc::clone(&app_service)));
    let storage_rpc = Arc::new(StorageRpcImpl::new(Arc::clone(service)));
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

    let transport = LocalRpcTransport::new(dispatcher);
    let (logger, vec_logger) = test_common::rpc_test_tee_logger();
    let time = Arc::new(SystemTimeProvider);
    let rng = SystemRngProvider;
    let config =
        RpcLoggingConfig::from_default_wallet_yaml().expect("RPC logging config must load");
    (
        LoggedRpcTransport::new(transport, config, logger, time, rng),
        vec_logger,
    )
}

fn assert_log_pairs(lines: &[String]) {
    for line in lines {
        assert!(
            !line.contains("StrongPassw0rd!"),
            "password must not appear in logs"
        );
    }

    let json_lines: Vec<serde_json::Value> = lines
        .iter()
        .filter_map(|line| line.split_once("] ").map(|(_prefix, json)| json))
        .filter_map(|json| serde_json::from_str::<serde_json::Value>(json).ok())
        .collect();

    let first_request = json_lines
        .iter()
        .find(|v| v.get("event") == Some(&serde_json::Value::String("rpc.request".to_string())))
        .expect("rpc.request must exist");
    let request_id = first_request
        .get("request_id")
        .and_then(|v| v.as_str())
        .unwrap();

    let matching_response = json_lines.iter().find(|v| {
        let id = v.get("request_id").and_then(|v| v.as_str());
        let event = v.get("event").and_then(|v| v.as_str());
        id == Some(request_id) && matches!(event, Some("rpc.response") | Some("rpc.error"))
    });
    assert!(matching_response.is_some(), "request must have a pair");
}

#[tokio::test]
async fn test_local_dispatcher_can_call() {
    let dir = tempfile::tempdir().unwrap();
    let service = Arc::new(WalletService::with_output_dir(dir.path().join("wallets")));
    let (transport, vec_logger) = make_transport(&service);

    let (wallet_id, session) = create_wallet_session(&transport).await;
    check_wallet_list(&transport).await;
    check_asset_list(&transport, &wallet_id).await;
    test_check_send_roundtrip(&transport, &session).await;
    check_backup_roundtrip(&transport, &service, &wallet_id, &session).await;

    // Assert: middleware emitted logs and secrets are not present.
    let lines = vec_logger.lines();
    assert!(lines.len() >= 14, "expected at least 2 lines per call");
    assert_log_pairs(&lines);
}
