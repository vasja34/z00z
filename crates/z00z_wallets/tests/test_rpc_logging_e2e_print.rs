#![cfg(not(target_arch = "wasm32"))]
#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;
use std::sync::Arc;

use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcTransport};
use z00z_utils::codec::json;
use z00z_utils::logger::{Logger, RotatingFileLogger, RotationPolicy};
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

#[tokio::test]
async fn test_rpc_logging_end_prints() {
    let log_dir = tempfile::tempdir().unwrap();
    let log_path: PathBuf = log_dir.path().join("rpc_logger.json");
    let before = std::fs::read_to_string(&log_path).unwrap_or_default();

    let mut config =
        RpcLoggingConfig::from_default_wallet_yaml().expect("RPC logging config must load");
    config.output.path = log_path.to_string_lossy().to_string();
    let rotation = RotationPolicy {
        max_bytes: 1024 * 1024,
        keep_files: config.output.rotation.keep_files,
    };

    let file_logger = RotatingFileLogger::new(&log_path, rotation).unwrap();
    let logger: Arc<dyn Logger> = Arc::new(file_logger);

    let wallets_dir = tempfile::tempdir().unwrap().path().join("wallets");
    let service = Arc::new(WalletService::with_output_dir(wallets_dir));

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

    let time = Arc::new(SystemTimeProvider);
    let rng = SystemRngProvider;
    let transport = LoggedRpcTransport::new(base, config, logger, time, rng);

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

    let wallet_id_value = &created["wallet_id"];
    let wallet_id = wallet_id_value
        .as_str()
        .or_else(|| wallet_id_value.get(0).and_then(|v| v.as_str()))
        .or_else(|| wallet_id_value.get("0").and_then(|v| v.as_str()))
        .unwrap_or("stub-wallet-id");

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

    println!("\nRPC log path: {}", log_path.display());
    let content = std::fs::read_to_string(&log_path).unwrap_or_default();
    let appended = content.get(before.len()..).unwrap_or("");
    println!(
        "--- BEGIN appended rpc_logger.json ---\n{}\n--- END appended rpc_logger.json ---",
        appended
    );

    assert!(!content.contains("StrongPassw0rd!"));
    assert!(!content.contains("SECRET_MEMO"));
}
