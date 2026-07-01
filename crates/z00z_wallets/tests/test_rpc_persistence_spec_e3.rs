#![cfg(not(target_arch = "wasm32"))]
#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use z00z_utils::codec::json;

use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcError, RpcTransport};

use z00z_wallets::rpc::methods::{
    AppRpcImpl, AssetRpcImpl, BackupRpcImpl, ChainRpcImpl, ChainScanRpcImpl, KeyRpcImpl,
    NetworkRpcImpl, StorageRpcImpl, TxRpcImpl, WalletRpcImpl,
};
use z00z_wallets::rpc::register_all_wallet_rpc_methods;
use z00z_wallets::rpc::types::security::RuntimeEncryptedResponse;
use z00z_wallets::rpc::types::wallet::{RuntimeCreateWalletResponse, RuntimeExportWalletResponse};
use z00z_wallets::services::{AppService, WalletService};

const PASSWORD: &str = "CorrectPassw0rd!";

fn setup_transport_with_services() -> (
    impl RpcTransport,
    Arc<WalletService>,
    Arc<AppService>,
    tempfile::TempDir,
) {
    let dir = tempfile::tempdir().expect("tempdir");
    let wallet_service = Arc::new(WalletService::with_output_dir(dir.path().join("wallets")));
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

    let transport = LocalRpcTransport::new(dispatcher);
    (transport, wallet_service, app_service, dir)
}

async fn rpc_create_wallet(transport: &impl RpcTransport) -> RuntimeCreateWalletResponse {
    let value = transport
        .call(
            "app.wallet.create_wallet",
            json!({
                "name": "Spec Wallet",
                "password": PASSWORD,
                "seed_phrase": null
            }),
        )
        .await
        .expect("create_wallet must succeed");

    serde_json::from_value(value).expect("RuntimeCreateWalletResponse must deserialize")
}

async fn rpc_unlock_wallet(transport: &impl RpcTransport, wallet_id: &str) {
    let _ = transport
        .call(
            "wallet.session.unlock_wallet",
            json!({"wallet_id": wallet_id, "password": PASSWORD}),
        )
        .await
        .expect("unlock_wallet must succeed");
}

#[tokio::test]
async fn test_e3_rpc_wallet_produces() {
    let (transport, _wallets, _app, _dir) = setup_transport_with_services();

    let created = rpc_create_wallet(&transport).await;
    rpc_unlock_wallet(&transport, &created.wallet_id.0).await;

    let value = transport
        .call(
            "app.wallet.export_wallet",
            json!({"wallet_id": created.wallet_id.0, "password": PASSWORD}),
        )
        .await
        .expect("export_wallet must succeed");

    let response: RuntimeExportWalletResponse =
        serde_json::from_value(value).expect("RuntimeExportWalletResponse must deserialize");

    let payload = response
        .encrypted_payload
        .expect("encrypted_payload must be present");

    assert!(!payload.ciphertext.trim().is_empty());
    assert_ne!(payload.metadata.algorithm, "none");

    // Basic secret-safety assertions.
    assert!(!payload.ciphertext.contains(PASSWORD));
}

#[tokio::test]
async fn test_e3_rpc_wallet_rejects() {
    let (transport_a, _wallets_a, _app_a, _dir_a) = setup_transport_with_services();

    let created = rpc_create_wallet(&transport_a).await;
    rpc_unlock_wallet(&transport_a, &created.wallet_id.0).await;

    let export_value = transport_a
        .call(
            "app.wallet.export_wallet",
            json!({"wallet_id": created.wallet_id.0, "password": PASSWORD}),
        )
        .await
        .expect("export_wallet must succeed");

    let export: RuntimeExportWalletResponse =
        serde_json::from_value(export_value).expect("RuntimeExportWalletResponse must deserialize");
    let payload = export.encrypted_payload.expect("encrypted payload");

    let data = serde_json::to_string(&payload).expect("payload json");

    let (transport_b, _wallets_b, _app_b, _dir_b) = setup_transport_with_services();

    let err = transport_b
        .call(
            "app.wallet.import_wallet",
            json!({
                "data": data,
                "password": "WrongPassw0rd!",
                "name": "Imported"
            }),
        )
        .await
        .expect_err("wrong password must fail");

    assert!(matches!(err, RpcError::AuthFailed));
}

#[tokio::test]
async fn test_e3_rpc_export_import() {
    let (transport_a, wallets_a, _app_a, _dir_a) = setup_transport_with_services();

    let created = rpc_create_wallet(&transport_a).await;

    // Set a non-default settings value before export (service-level write, RPC-level transfer).
    let mut settings = wallets_a
        .get_wallet_settings(&created.wallet_id)
        .await
        .expect("get settings");
    settings.currency_display = "EUR".to_string();
    settings.auto_lock_timeout = 999;
    wallets_a
        .set_wallet_settings(created.wallet_id.clone(), settings.clone())
        .await
        .expect("set settings");

    rpc_unlock_wallet(&transport_a, &created.wallet_id.0).await;

    let export_value = transport_a
        .call(
            "app.wallet.export_wallet",
            json!({"wallet_id": created.wallet_id.0, "password": PASSWORD}),
        )
        .await
        .expect("export_wallet must succeed");

    let export: RuntimeExportWalletResponse =
        serde_json::from_value(export_value).expect("RuntimeExportWalletResponse must deserialize");
    let payload: RuntimeEncryptedResponse = export.encrypted_payload.expect("encrypted payload");
    let data = serde_json::to_string(&payload).expect("payload json");

    let (transport_b, wallets_b, _app_b, _dir_b) = setup_transport_with_services();

    let import_value = transport_b
        .call(
            "app.wallet.import_wallet",
            json!({
                "data": data,
                "password": PASSWORD,
                "name": "Imported"
            }),
        )
        .await
        .expect("import_wallet must succeed");

    let imported: serde_json::Value = import_value;
    let imported_id = imported
        .get("wallet_id")
        .and_then(|v| v.as_str())
        .expect("wallet_id must exist");

    assert_eq!(imported_id, created.wallet_id.0);

    let loaded_settings = wallets_b
        .get_wallet_settings(&created.wallet_id)
        .await
        .expect("get imported settings");

    assert_eq!(loaded_settings.currency_display, settings.currency_display);
    assert_eq!(
        loaded_settings.auto_lock_timeout,
        settings.auto_lock_timeout
    );
}
