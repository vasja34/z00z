#![cfg(not(target_arch = "wasm32"))]
#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use serde_json::Value;

use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcTransport};
use z00z_utils::codec::json;
use z00z_utils::logger::Logger;
use z00z_utils::rng::SecureRngProvider;
use z00z_utils::time::{SystemTimeProvider, TimeProvider};

use z00z_wallets::rpc::logging::{
    LoggedRpcTransport, RpcLoggingConfig, RpcLoggingTruncationConfig,
};
use z00z_wallets::rpc::methods::{
    AppRpcImpl, AssetRpcImpl, BackupRpcImpl, ChainRpcImpl, ChainScanRpcImpl, KeyRpcImpl,
    NetworkRpcImpl, StorageRpcImpl, TxRpcImpl, WalletRpcImpl,
};
use z00z_wallets::rpc::register_all_wallet_rpc_methods;
use z00z_wallets::services::{AppService, WalletService};

#[path = "test_inc/test_rpc_logger.inc"]
mod test_common;

fn test_time_provider() -> Arc<dyn TimeProvider> {
    Arc::new(SystemTimeProvider)
}

fn enabled_logging_config() -> RpcLoggingConfig {
    let mut cfg =
        RpcLoggingConfig::from_default_wallet_yaml().expect("RPC logging config must load");
    cfg.enabled = true;
    cfg
}

fn truncate_non_secret(value: &str, trunc: &RpcLoggingTruncationConfig) -> String {
    if value.len() <= trunc.non_secret_min_bytes {
        return value.to_string();
    }

    let head = value.chars().take(trunc.head_chars).collect::<String>();
    let tail = value
        .chars()
        .rev()
        .take(trunc.tail_chars)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();

    format!("<len={} {}...{}>", value.chars().count(), head, tail)
}

fn expected_first_request_id(seed: u64, trunc: &RpcLoggingTruncationConfig) -> String {
    let mut inner = StdRng::seed_from_u64(seed);

    let mut bytes = [0u8; 16];
    inner.fill_bytes(&mut bytes);

    let full = hex::encode(bytes);
    truncate_non_secret(&full, trunc)
}

#[derive(Clone, Copy)]
struct TestSecureRngProvider {
    seed: u64,
}

impl SecureRngProvider for TestSecureRngProvider {
    type Rng = StdRng;

    fn rng(&self) -> Self::Rng {
        StdRng::seed_from_u64(self.seed)
    }
}

fn setup_logged_transport(
    logger: Arc<dyn Logger>,
    rng_seed: u64,
) -> (impl RpcTransport, RpcLoggingConfig) {
    let dir = tempfile::tempdir().expect("tempdir must create");
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
    let time = test_time_provider();
    let rng = TestSecureRngProvider { seed: rng_seed };

    let cfg = enabled_logging_config();
    (
        LoggedRpcTransport::new(base, cfg.clone(), logger, time, rng),
        cfg,
    )
}

fn parse_json(line: &str) -> Value {
    let json_start = line
        .find('{')
        .or_else(|| line.find('['))
        .expect("log line must contain JSON object/array");
    let json = &line[json_start..];
    match serde_json::from_str(json) {
        Ok(value) => value,
        Err(err) => panic!("log line must be valid JSON: {err}; raw={line:?}; extracted={json:?}"),
    }
}

#[tokio::test]
async fn test_request_id_mock_rng() {
    let (logger_1, vec_logger_1) = test_common::rpc_test_tee_logger();
    let (transport_1, cfg) = setup_logged_transport(logger_1, 123);

    let _ = transport_1
        .call("app.wallet.list_wallets", json!({}))
        .await
        .expect("list_wallets must succeed");

    let lines_1 = vec_logger_1.lines();
    assert_eq!(lines_1.len(), 2, "expected request + response log");

    let req_1 = parse_json(&lines_1[0]);
    assert_eq!(
        req_1.get("event").and_then(Value::as_str),
        Some("rpc.request")
    );

    let resp_1 = parse_json(&lines_1[1]);
    assert_eq!(
        resp_1.get("event").and_then(Value::as_str),
        Some("rpc.response"),
        "second log line must be rpc.response on success"
    );

    let expected = expected_first_request_id(123, &cfg.truncation);
    assert_eq!(
        req_1.get("request_id").and_then(Value::as_str),
        Some(expected.as_str()),
        "request_id must be deterministic for a fixed RNG seed"
    );

    assert_eq!(
        req_1.get("request_id"),
        resp_1.get("request_id"),
        "request_id must match between request and response"
    );

    let (logger_2, vec_logger_2) = test_common::rpc_test_tee_logger();
    let (transport_2, _cfg2) = setup_logged_transport(logger_2, 123);

    let _ = transport_2
        .call("app.wallet.list_wallets", json!({}))
        .await
        .expect("list_wallets must succeed");

    let lines_2 = vec_logger_2.lines();
    let req_2 = parse_json(&lines_2[0]);

    assert_eq!(
        req_1.get("request_id"),
        req_2.get("request_id"),
        "first request_id should match across transports with the same seed"
    );
}
