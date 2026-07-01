#![cfg(not(target_arch = "wasm32"))]
#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use z00z_utils::codec::json;

use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcError, RpcTransport};
use z00z_utils::rng::SystemRngProvider;
use z00z_utils::time::{SystemTimeProvider, TimeProvider};

use z00z_wallets::rpc::logging::{LoggedRpcTransport, RpcLoggingConfig};
use z00z_wallets::rpc::methods::{
    AppRpcImpl, AssetRpcImpl, BackupRpcImpl, ChainRpcImpl, ChainScanRpcImpl, KeyRpcImpl,
    NetworkRpcImpl, StorageRpcImpl, TxRpcImpl, WalletRpcImpl,
};
use z00z_wallets::rpc::register_all_wallet_rpc_methods;
use z00z_wallets::rpc::types::wallet::{PersistWalletInfo, RuntimeCreateWalletResponse};
use z00z_wallets::services::{AppService, WalletService};

#[path = "test_inc/test_rpc_logger.inc"]
mod test_common;

const PASSWORD: &str = "CorrectPassw0rd!";

fn test_time_provider() -> Arc<dyn TimeProvider> {
    Arc::new(SystemTimeProvider)
}

fn setup_base_transport(output_dir: std::path::PathBuf) -> impl RpcTransport {
    let wallet_service = Arc::new(WalletService::with_output_dir(output_dir));
    let app_service = Arc::new(AppService::with_wallet_service(Arc::clone(&wallet_service)));

    let wallet_rpc = Arc::new(WalletRpcImpl::new(Arc::clone(&wallet_service)));
    let app_rpc = Arc::new(AppRpcImpl::new(Arc::clone(&app_service)));

    let asset_rpc = Arc::new(AssetRpcImpl::new());
    let tx_rpc = Arc::new(TxRpcImpl::new(Arc::clone(&wallet_service)));
    let backup_rpc = Arc::new(BackupRpcImpl::new(Arc::clone(&wallet_service)));
    let key_rpc = Arc::new(KeyRpcImpl::new(Arc::clone(&wallet_service)));

    let chain_rpc = Arc::new(ChainRpcImpl::new(Arc::clone(&app_service)));
    let network_rpc = Arc::new(NetworkRpcImpl::with_app_service(Arc::clone(&app_service)));
    let scan_rpc = Arc::new(ChainScanRpcImpl::new(Arc::clone(&app_service)));
    let storage_rpc = Arc::new(StorageRpcImpl::new(Arc::clone(&wallet_service)));

    let dispatcher = Arc::new(RpcDispatcher::new());
    register_all_wallet_rpc_methods(
        &dispatcher,
        app_rpc,
        wallet_rpc,
        asset_rpc,
        tx_rpc,
        backup_rpc,
        key_rpc,
        chain_rpc,
        network_rpc,
        scan_rpc,
        storage_rpc,
    )
    .expect("wallet RPC registration should succeed");

    LocalRpcTransport::new(dispatcher)
}

async fn create_wallet(transport: &impl RpcTransport, name: &str) -> RuntimeCreateWalletResponse {
    let value = transport
        .call(
            "app.wallet.create_wallet",
            json!({"name": name, "password": PASSWORD, "seed_phrase": null}),
        )
        .await
        .expect("create_wallet must succeed");

    serde_json::from_value(value).expect("RuntimeCreateWalletResponse must deserialize")
}

async fn unlock_wallet(transport: &impl RpcTransport, wallet_id: &str) -> serde_json::Value {
    transport
        .call(
            "wallet.session.unlock_wallet",
            json!({"wallet_id": wallet_id, "password": PASSWORD}),
        )
        .await
        .expect("unlock_wallet must succeed")
}

#[tokio::test]
async fn test_scenario_4_lifecycle_lock() {
    let dir = tempfile::tempdir().expect("tempdir");
    let transport = setup_base_transport(dir.path().join("wallets"));

    let created = create_wallet(&transport, "Scenario Wallet").await;
    let session = unlock_wallet(&transport, &created.wallet_id.0).await;

    let _ = transport
        .call(
            "wallet.key.list_receivers",
            json!({
                "session": session,
                "limit": 1,
                "cursor": null,
                "filter": null
            }),
        )
        .await
        .expect("list_receivers must succeed");

    transport
        .call(
            "wallet.lifecycle.on_event",
            json!({"event": "backgrounded"}),
        )
        .await
        .expect("lifecycle event must succeed");

    let err = transport
        .call(
            "wallet.key.list_receivers",
            json!({
                "session": session,
                "limit": 1,
                "cursor": null,
                "filter": null
            }),
        )
        .await
        .expect_err("list_receivers must fail after lifecycle lock");

    assert!(matches!(
        err,
        RpcError::WalletLocked
            | RpcError::AuthFailed
            | RpcError::SessionExpired
            | RpcError::SessionInvalid
    ));
}

fn extract_json_vec_logger_line(line: &str) -> &str {
    let start = line
        .find('{')
        .unwrap_or_else(|| panic!("expected JSON object in VecLogger line: {line}"));
    &line[start..]
}

#[tokio::test]
async fn test_scenario_1_unlock_list() {
    let dir = tempfile::tempdir().expect("tempdir");
    let base = setup_base_transport(dir.path().join("wallets"));

    let (logger, _vec_logger) = test_common::rpc_test_tee_logger();
    let cfg = RpcLoggingConfig::from_default_wallet_yaml().expect("RPC logging config must load");
    let time = test_time_provider();
    let rng = SystemRngProvider;

    let transport = LoggedRpcTransport::new(base, cfg, logger, time, rng);

    let created = create_wallet(&transport, "Scenario Wallet").await;
    let _session = unlock_wallet(&transport, &created.wallet_id.0).await;

    let value = transport
        .call("app.wallet.list_wallets", json!({}))
        .await
        .expect("list_wallets must succeed");

    let wallets: Vec<PersistWalletInfo> =
        serde_json::from_value(value).expect("Vec<PersistWalletInfo>");
    assert!(wallets.iter().any(|w| w.id == created.wallet_id));
}

#[tokio::test]
async fn test_scenario_2_list_emits() {
    let dir = tempfile::tempdir().expect("tempdir");
    let base = setup_base_transport(dir.path().join("wallets"));

    let (logger, vec_logger) = test_common::rpc_test_tee_logger();

    let cfg = RpcLoggingConfig::from_default_wallet_yaml().expect("RPC logging config must load");
    let time = test_time_provider();
    let rng = SystemRngProvider;

    let transport = LoggedRpcTransport::new(base, cfg, logger, time, rng);

    let created = create_wallet(&transport, "Scenario Wallet").await;
    let _session = unlock_wallet(&transport, &created.wallet_id.0).await;

    vec_logger.clear();

    let _ = transport
        .call(
            "wallet.asset.list_assets",
            json!({"wallet_id": created.wallet_id.0}),
        )
        .await
        .expect("list_assets must succeed");

    let lines = vec_logger.lines();
    assert_eq!(lines.len(), 2);

    for line in lines {
        let json = extract_json_vec_logger_line(&line);
        let v: serde_json::Value = serde_json::from_str(json).expect("valid json");
        assert!(v.get("ts_ms").is_none(), "ts_ms must be skipped");
        assert!(
            v.get("wallet_id_full").is_none(),
            "wallet_id_full must be skipped"
        );
        assert!(!line.contains(PASSWORD), "password must not appear in logs");
    }
}

#[tokio::test]
async fn test_scenario_3_tx_invalid() {
    let dir = tempfile::tempdir().expect("tempdir");
    let base = setup_base_transport(dir.path().join("wallets"));

    let (logger, vec_logger) = test_common::rpc_test_tee_logger();

    let cfg = RpcLoggingConfig::from_default_wallet_yaml().expect("RPC logging config must load");
    let time = test_time_provider();
    let rng = SystemRngProvider;

    let transport = LoggedRpcTransport::new(base, cfg, logger, time, rng);

    let created = create_wallet(&transport, "Scenario Wallet").await;
    let session = unlock_wallet(&transport, &created.wallet_id.0).await;

    vec_logger.clear();

    let err = transport
        .call(
            "wallet.tx.send_transaction",
            json!({
                "session": session,
                "tx_data": "RAW_TX_SHOULD_NOT_APPEAR",
            }),
        )
        .await
        .expect_err("invalid params must fail");

    assert!(matches!(err, RpcError::InvalidParams(_)));

    let lines = vec_logger.lines();
    assert_eq!(lines.len(), 2);

    let error_line = &lines[1];
    let json = extract_json_vec_logger_line(error_line);
    let v: serde_json::Value = serde_json::from_str(json).expect("valid json");

    let error_code = v.get("error_code").and_then(|x| x.as_str()).unwrap_or("");
    let error_message = v
        .get("error_message")
        .and_then(|x| x.as_str())
        .unwrap_or("");

    assert_eq!(error_code, "invalid_params");
    assert_eq!(error_message, "Validation failed");
    assert!(!error_line.contains("RAW_TX_SHOULD_NOT_APPEAR"));
}
