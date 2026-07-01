#![cfg(not(target_arch = "wasm32"))]
#![cfg(not(target_arch = "wasm32"))]

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

use chrono::{TimeZone as _, Utc};

fn parse_logged_json_line(line: &str) -> serde_json::Value {
    let json = line
        .find('{')
        .and_then(|index| line.get(index..))
        .unwrap_or_else(|| panic!("expected JSON payload in log line, got: {line}"));

    serde_json::from_str(json).expect("must be valid json")
}

#[tokio::test]
async fn test_rpc_logging_sink_jsonl() {
    let dir = tempfile::tempdir().unwrap();

    let mut cfg =
        RpcLoggingConfig::from_default_wallet_yaml().expect("RPC logging config must load");
    let log_path = dir.path().join("rpc_wallet_jsonl.log");
    cfg.output.path = log_path.to_string_lossy().to_string();

    let rotation = RotationPolicy {
        // This test only needs to validate that JSONL is written.
        max_bytes: 1024 * 1024,
        keep_files: cfg.output.rotation.keep_files,
    };
    let file_logger = RotatingFileLogger::new(&log_path, rotation).unwrap();
    let logger: Arc<dyn Logger> = Arc::new(file_logger);

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

    let time: Arc<dyn z00z_utils::time::TimeProvider> = Arc::new(SystemTimeProvider);
    let rng = SystemRngProvider;

    let transport = LoggedRpcTransport::new(base, cfg, logger, time, rng);

    let before = std::fs::read_to_string(&log_path).unwrap_or_default();

    let _ = transport
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

    assert!(log_path.exists());

    let after = std::fs::read_to_string(&log_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", log_path.display()));
    let appended = after.get(before.len()..).unwrap_or("");

    let lines: Vec<&str> = appended.lines().collect();
    assert!(lines.len() >= 2, "expected at least request+response lines");

    for line in lines {
        let parsed = parse_logged_json_line(line);
        let ts = parsed
            .get("ts")
            .and_then(|v| v.as_str())
            .expect("ts must be a string");

        let naive = chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S%.3f")
            .unwrap_or_else(|e| panic!("ts must match YYYY-MM-DD HH:MM:SS.mmm: ts={ts} err={e}"));
        let logged = Utc.from_utc_datetime(&naive);
        let now = Utc::now();
        let drift_ms = (now - logged).num_milliseconds().abs();
        assert!(
            drift_ms <= 5 * 60 * 1000,
            "ts too far from current time: ts={ts} now={} drift_ms={drift_ms}",
            now.format("%Y-%m-%d %H:%M:%S%.3f")
        );

        assert_eq!(parsed.get("level").and_then(|v| v.as_str()), Some("info"));
        assert!(parsed.get("event").is_some());
    }
}

#[tokio::test]
async fn test_rpc_logging_rotation_keeps() {
    let dir = tempfile::tempdir().unwrap();

    let mut cfg =
        RpcLoggingConfig::from_default_wallet_yaml().expect("RPC logging config must load");
    let log_path = dir.path().join("rpc_logger.json");
    cfg.output.path = log_path.to_string_lossy().to_string();

    let rotation = RotationPolicy {
        max_bytes: 256,
        keep_files: cfg.output.rotation.keep_files,
    };
    let file_logger = RotatingFileLogger::new(&log_path, rotation).unwrap();
    let logger: Arc<dyn Logger> = Arc::new(file_logger);

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

    let time: Arc<dyn z00z_utils::time::TimeProvider> = Arc::new(SystemTimeProvider);
    let rng = SystemRngProvider;

    let keep_files = cfg.output.rotation.keep_files;

    let transport = LoggedRpcTransport::new(base, cfg, logger, time, rng);

    for _ in 0..200 {
        let _ = transport.call("app.wallet.list_wallets", json!({})).await;
    }

    assert!(log_path.exists());

    let rotated: Vec<std::path::PathBuf> = (1..=keep_files)
        .map(|i| std::path::PathBuf::from(format!("{}.{}", log_path.to_string_lossy(), i)))
        .filter(|p| p.exists())
        .collect();

    assert!(
        !rotated.is_empty(),
        "expected rotation to create at least one rotated file"
    );

    assert!(
        rotated.len() <= keep_files,
        "expected at most {keep_files} rotated files in probe window"
    );
}
