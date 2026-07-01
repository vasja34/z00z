#![cfg(not(target_arch = "wasm32"))]
#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcTransport};
use z00z_utils::codec::json;
use z00z_utils::logger::Logger;
use z00z_utils::rng::SystemRngProvider;
use z00z_utils::time::SystemTimeProvider;

use chrono::{TimeZone as _, Utc};

use z00z_wallets::rpc::{
    logging::{LoggedRpcTransport, RpcLoggingConfig, RpcLoggingInstaller},
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

fn parse_json_vec_logger_line(line: &str) -> Option<serde_json::Value> {
    // VecLogger format: [LEVEL] {json}
    let (_prefix, json) = line.split_once("] ")?;
    serde_json::from_str(json).ok()
}

fn test_assert_ts_valid_recent(v: &serde_json::Value) {
    let ts = v
        .get("ts")
        .and_then(|s| s.as_str())
        .expect("ts must be present and string");

    let naive = chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S%.3f")
        .unwrap_or_else(|e| panic!("ts must match YYYY-MM-DD HH:MM:SS.mmm (UTC): ts={ts} err={e}"));

    let logged = Utc.from_utc_datetime(&naive);
    let now = Utc::now();
    let drift_ms = (now - logged).num_milliseconds().abs();

    // Allow a generous window to avoid flakes on slow CI / debugging runs.
    assert!(
        drift_ms <= 5 * 60 * 1000,
        "ts too far from current time: ts={ts} now={} drift_ms={drift_ms}",
        now.format("%Y-%m-%d %H:%M:%S%.3f")
    );
}

fn test_config() -> RpcLoggingConfig {
    RpcLoggingConfig::from_default_wallet_yaml().expect("RPC logging config must load")
}

async fn setup_transport(logger: Arc<dyn Logger>) -> impl RpcTransport {
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
    let time = Arc::new(SystemTimeProvider);
    let rng = SystemRngProvider;

    LoggedRpcTransport::new(base, test_config(), logger, time, rng)
}

#[tokio::test]
async fn test_rpc_logging_prevents_double() {
    let (logger, vec_logger) = test_common::rpc_test_tee_logger();

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
    let time: Arc<dyn z00z_utils::time::TimeProvider> = Arc::new(SystemTimeProvider);
    let rng = SystemRngProvider;

    let installer = RpcLoggingInstaller::new();
    let wrapped = installer.maybe_wrap_transport(
        base,
        test_config(),
        Arc::clone(&logger),
        Arc::clone(&time),
        rng,
    );

    // Accidental second wrapping attempt (guarded).
    let rng2 = SystemRngProvider;
    let wrapped_again =
        installer.maybe_wrap_transport(wrapped, test_config(), Arc::clone(&logger), time, rng2);

    vec_logger.clear();
    let _ = wrapped_again
        .call("app.wallet.list_wallets", json!({}))
        .await
        .unwrap();

    let lines = vec_logger.lines();
    if should_dump_rpc_logs() {
        println!("\n--- BEGIN VecLogger RPC logs (acceptance/double-wrap) ---");
        for line in &lines {
            println!("{line}");
        }
        println!("--- END VecLogger RPC logs (acceptance/double-wrap) ---\n");
    }
    assert_eq!(
        lines.len(),
        2,
        "expected no duplicate logging from double-wrapping"
    );
}

mod emits_two_lines_on_success {
    use super::*;

    #[tokio::test]
    async fn test_rpc_logging_emits_two() {
        let (logger, vec_logger) = test_common::rpc_test_tee_logger();
        let transport = setup_transport(logger).await;

        vec_logger.clear();

        let _ = transport
            .call("app.wallet.list_wallets", json!({}))
            .await
            .unwrap();

        let lines = vec_logger.lines();
        if should_dump_rpc_logs() {
            println!("\n--- BEGIN VecLogger RPC logs (acceptance/success) ---");
            for line in &lines {
                println!("{line}");
            }
            println!("--- END VecLogger RPC logs (acceptance/success) ---\n");
        }
        assert_eq!(lines.len(), 2, "expected request + response only");

        let first = parse_json_vec_logger_line(&lines[0]).unwrap();
        let second = parse_json_vec_logger_line(&lines[1]).unwrap();

        test_assert_ts_valid_recent(&first);
        test_assert_ts_valid_recent(&second);

        assert_eq!(
            first.get("event").and_then(|v| v.as_str()),
            Some("rpc.request")
        );
        assert_eq!(
            second.get("event").and_then(|v| v.as_str()),
            Some("rpc.response")
        );

        let rid1 = first.get("request_id").and_then(|v| v.as_str()).unwrap();
        let rid2 = second.get("request_id").and_then(|v| v.as_str()).unwrap();
        assert_eq!(rid1, rid2);

        assert!(second.get("duration_ms").is_some());
    }
}

mod rpc_error_output_minimal {
    use super::*;

    #[tokio::test]
    async fn test_rpc_logging_emits_two() {
        let (logger, vec_logger) = test_common::rpc_test_tee_logger();
        let transport = setup_transport(logger).await;

        vec_logger.clear();

        let err = transport
            .call(
                "unknown.method",
                json!({"password": "StrongPassw0rd!", "seed_phrase": "secret"}),
            )
            .await;

        assert!(err.is_err());

        let lines = vec_logger.lines();
        if should_dump_rpc_logs() {
            println!("\n--- BEGIN VecLogger RPC logs (acceptance/error) ---");
            for line in &lines {
                println!("{line}");
            }
            println!("--- END VecLogger RPC logs (acceptance/error) ---\n");
        }
        assert_eq!(lines.len(), 2, "expected request + error only");

        assert!(!lines[0].contains("StrongPassw0rd!"));
        assert!(!lines[1].contains("StrongPassw0rd!"));

        let first = parse_json_vec_logger_line(&lines[0]).unwrap();
        let second = parse_json_vec_logger_line(&lines[1]).unwrap();

        test_assert_ts_valid_recent(&first);
        test_assert_ts_valid_recent(&second);

        assert_eq!(
            first.get("event").and_then(|v| v.as_str()),
            Some("rpc.request")
        );
        assert_eq!(
            second.get("event").and_then(|v| v.as_str()),
            Some("rpc.error")
        );

        // Unknown method deny-by-default: no params_summary.
        assert!(first.get("params_summary").is_none());

        // Error record must use a bounded template.
        assert!(second.get("error_code").is_some());
        let msg = second
            .get("error_message")
            .and_then(|v| v.as_str())
            .unwrap();
        assert!(
            matches!(
                msg,
                "Authentication failed"
                    | "Permission denied"
                    | "Rate limit exceeded"
                    | "Validation failed"
                    | "Internal error"
            ),
            "error_message must be templated"
        );
    }
}
