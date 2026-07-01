#![cfg(not(target_arch = "wasm32"))]
//! E2E tests for key derivation RPC methods.
//!
//! **IMPORTANT:** These tests modify global environment variables (Z00Z_WALLET_NETWORK, Z00Z_WALLET_CHAIN)
//! and MUST be run serially to avoid interference:
//!
//! ```bash
//! cargo test --test test_rpc_key_derive_e2e -- --test-threads=1
//! ```

use std::sync::Arc;

use z00z_core::genesis::ChainType;
use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcError, RpcTransport};
use z00z_utils::{codec::json, rng::SystemRngProvider, time::SystemTimeProvider};

#[path = "test_inc/test_rpc_logger.inc"]
mod test_rpc_logger;
#[path = "test_inc/test_wallet_env.inc"]
mod test_wallet_env;

use z00z_wallets::{
    key::receiver_keys::derive_rotated_view_secret_key,
    key::{derive_view_secret_key, ReceiverKeys, ReceiverSecret},
    rpc::{
        logging::{LoggedRpcTransport, RpcLoggingConfig},
        methods::{
            AppRpcImpl, AssetRpcImpl, BackupRpcImpl, ChainRpcImpl, ChainScanRpcImpl, KeyRpcImpl,
            NetworkRpcImpl, StorageRpcImpl, TxRpcImpl, WalletRpcImpl,
        },
        register_all_wallet_rpc_methods,
        types::{
            key::RuntimeDeriveReceiverResponse, security::SessionToken,
            wallet::RuntimeCreateWalletResponse,
        },
    },
    services::{AppService, WalletService},
};

struct AddrLimitEnvGuard {
    prev_network: Option<String>,
    prev_chain: Option<String>,
    prev_rate: Option<String>,
    prev_burst: Option<String>,
}

impl AddrLimitEnvGuard {
    fn new(network: &str, chain: &str, rate_per_sec: u32, burst: u32) -> Self {
        let _guard = test_wallet_env::wallet_env_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        let prev_network = std::env::var("Z00Z_WALLET_NETWORK").ok();
        let prev_chain = std::env::var("Z00Z_WALLET_CHAIN").ok();
        let prev_rate = std::env::var("Z00Z_WALLET_RECEIVER_DERIVE_RATE_PER_SEC").ok();
        let prev_burst = std::env::var("Z00Z_WALLET_RECEIVER_DERIVE_BURST").ok();

        std::env::set_var("Z00Z_WALLET_NETWORK", network);
        std::env::set_var("Z00Z_WALLET_CHAIN", chain);
        std::env::set_var(
            "Z00Z_WALLET_RECEIVER_DERIVE_RATE_PER_SEC",
            rate_per_sec.to_string(),
        );
        std::env::set_var("Z00Z_WALLET_RECEIVER_DERIVE_BURST", burst.to_string());

        Self {
            prev_network,
            prev_chain,
            prev_rate,
            prev_burst,
        }
    }
}

impl Drop for AddrLimitEnvGuard {
    fn drop(&mut self) {
        let _guard = test_wallet_env::wallet_env_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        match &self.prev_network {
            Some(v) => std::env::set_var("Z00Z_WALLET_NETWORK", v),
            None => std::env::remove_var("Z00Z_WALLET_NETWORK"),
        }
        match &self.prev_chain {
            Some(v) => std::env::set_var("Z00Z_WALLET_CHAIN", v),
            None => std::env::remove_var("Z00Z_WALLET_CHAIN"),
        }
        match &self.prev_rate {
            Some(v) => std::env::set_var("Z00Z_WALLET_RECEIVER_DERIVE_RATE_PER_SEC", v),
            None => std::env::remove_var("Z00Z_WALLET_RECEIVER_DERIVE_RATE_PER_SEC"),
        }
        match &self.prev_burst {
            Some(v) => std::env::set_var("Z00Z_WALLET_RECEIVER_DERIVE_BURST", v),
            None => std::env::remove_var("Z00Z_WALLET_RECEIVER_DERIVE_BURST"),
        }
    }
}
fn test_logging_config() -> RpcLoggingConfig {
    let mut cfg =
        RpcLoggingConfig::from_default_wallet_yaml().expect("RPC logging config must load");
    cfg.enabled = true;
    cfg
}

fn setup_transport() -> (tempfile::TempDir, impl RpcTransport) {
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

    let (logger, _vec_logger) = test_rpc_logger::rpc_test_tee_logger();
    let time = Arc::new(SystemTimeProvider);
    let rng = SystemRngProvider;

    let transport = LoggedRpcTransport::new(base, test_logging_config(), logger, time, rng);
    (dir, transport)
}

async fn create_wallet(
    transport: &impl RpcTransport,
    password: &str,
) -> RuntimeCreateWalletResponse {
    let value = transport
        .call(
            "app.wallet.create_wallet",
            json!({
                "name": "E2E Wallet",
                "password": password,
                "seed_phrase": null
            }),
        )
        .await
        .expect("create_wallet must succeed");

    serde_json::from_value(value).expect("RuntimeCreateWalletResponse must deserialize")
}

async fn unlock_wallet(
    transport: &impl RpcTransport,
    wallet_id: &str,
    password: &str,
) -> SessionToken {
    let value = transport
        .call(
            "wallet.session.unlock_wallet",
            json!({"wallet_id": wallet_id, "password": password}),
        )
        .await
        .expect("unlock_wallet must succeed");

    serde_json::from_value(value).expect("SessionToken must deserialize")
}

async fn test_e2e_key_derivation(transport: &impl RpcTransport, chain: ChainType) {
    let switch_method = match chain {
        ChainType::Mainnet => "app.chain.switch_to_mainnet",
        ChainType::Devnet => "app.chain.switch_to_devnet",
        ChainType::Testnet => "app.chain.switch_to_testnet",
    };

    transport
        .call(switch_method, json!({}))
        .await
        .expect("switch chain must succeed");

    let created = create_wallet(transport, "CorrectPassw0rd!").await;
    let session = unlock_wallet(transport, &created.wallet_id.0, "CorrectPassw0rd!").await;

    let path = "m/44'/1337'/0'/0/0";
    let params = json!({"session": session, "path": path});

    let first = transport
        .call("wallet.key.derive_receiver", params.clone())
        .await
        .expect("wallet.key.derive_receiver must succeed");

    let second = transport
        .call("wallet.key.derive_receiver", params)
        .await
        .expect("wallet.key.derive_receiver must succeed twice");

    let first: RuntimeDeriveReceiverResponse =
        serde_json::from_value(first).expect("derive_receiver response must deserialize");
    let second: RuntimeDeriveReceiverResponse =
        serde_json::from_value(second).expect("derive_receiver response must deserialize");

    assert_eq!(first.path, path);
    assert_eq!(first.path, second.path);
    assert_eq!(first.public_key, second.public_key);

    assert!(!first.public_key.trim().is_empty());

    let pk_bytes = hex::decode(&first.public_key).expect("public_key hex must decode");
    let pk_bytes: [u8; 32] = pk_bytes
        .as_slice()
        .try_into()
        .expect("public_key must be 32 bytes");
    assert_eq!(hex::encode(pk_bytes), first.public_key);
}

#[test]
fn test_live_view_anchor() {
    let bytes = [0x51; 32];
    let secret = ReceiverSecret::from_bytes(bytes).expect("receiver secret");
    let keys = ReceiverKeys::from_receiver_secret(
        ReceiverSecret::from_bytes(bytes).expect("receiver secret"),
    )
    .expect("receiver keys");
    let live = derive_view_secret_key(&secret).expect("live view key");
    let rotated = derive_rotated_view_secret_key(&secret, 1).expect("rotated view key");

    assert_eq!(keys.reveal_view_sk().as_bytes(), live.as_bytes());
    assert_ne!(live.as_bytes(), rotated.as_bytes());
}

#[tokio::test]
async fn test_e2e_key_derivation_mainnet() {
    // Phase 6 FIX: Set BOTH env vars (network is required for chain to be read!)
    // CRITICAL: Set env vars BEFORE setup_transport() so wallet identity resolves correctly.
    // IMPORTANT: Guard with a global lock to avoid cross-test env var races.
    let _env = test_wallet_env::WalletEnvGuard::new("p2p", "mainnet");

    let (_dir, transport) = setup_transport();

    test_e2e_key_derivation(&transport, ChainType::Mainnet).await;
}

#[tokio::test]
async fn test_e2e_key_derivation_devnet() {
    // Phase 6 FIX: Set BOTH env vars (network is required for chain to be read!).
    // IMPORTANT: Guard with a global lock to avoid cross-test env var races.
    let _env = test_wallet_env::WalletEnvGuard::new("p2p", "devnet");

    let (_dir, transport) = setup_transport();
    test_e2e_key_derivation(&transport, ChainType::Devnet).await;
}

#[tokio::test]
async fn test_e2e_key_derivation_testnet() {
    // Phase 6 FIX: Set BOTH env vars (network is required for chain to be read!).
    // IMPORTANT: Guard with a global lock to avoid cross-test env var races.
    let _env = test_wallet_env::WalletEnvGuard::new("p2p", "testnet");

    let (_dir, transport) = setup_transport();
    test_e2e_key_derivation(&transport, ChainType::Testnet).await;
}

#[tokio::test]
async fn test_rpc_derive_key_auth() {
    let (_dir, transport) = setup_transport();

    let err = transport
        .call(
            "wallet.key.derive_receiver",
            json!({"session": {"token": "nope"}, "path": "m/44'/1337'/0'/0/0"}),
        )
        .await
        .expect_err("missing/invalid session must fail");

    assert!(matches!(
        err,
        RpcError::AuthFailed | RpcError::InvalidParams(_)
    ));
}

#[tokio::test]
async fn test_derive_key_bad_path() {
    let (_dir, transport) = setup_transport();

    transport
        .call("app.chain.switch_to_mainnet", json!({}))
        .await
        .expect("switch chain must succeed");

    let created = create_wallet(&transport, "CorrectPassw0rd!").await;
    let session = unlock_wallet(&transport, &created.wallet_id.0, "CorrectPassw0rd!").await;

    let err = transport
        .call(
            "wallet.key.derive_receiver",
            json!({"session": session, "path": "m/44'/1337'/0'/0'/0"}),
        )
        .await
        .expect_err("invalid path must fail");

    assert!(matches!(err, RpcError::InvalidParams(_)));
}

#[tokio::test]
async fn test_label_receiver_rejects_field() {
    let (_dir, transport) = setup_transport();

    transport
        .call("app.chain.switch_to_mainnet", json!({}))
        .await
        .expect("switch chain must succeed");

    let created = create_wallet(&transport, "CorrectPassw0rd!").await;
    let session = unlock_wallet(&transport, &created.wallet_id.0, "CorrectPassw0rd!").await;

    let derived: RuntimeDeriveReceiverResponse = serde_json::from_value(
        transport
            .call(
                "wallet.key.derive_receiver",
                json!({"session": session.clone(), "path": "m/44'/1337'/0'/0/0"}),
            )
            .await
            .expect("derive_receiver must succeed"),
    )
    .expect("derive_receiver response must deserialize");

    let err = transport
        .call(
            "wallet.key.label_receiver",
            json!({
                "session": session,
                "address": derived.public_key,
                "label": "Noncanonical field must fail"
            }),
        )
        .await
        .expect_err("noncanonical address field must be rejected");

    assert!(matches!(err, RpcError::InvalidParams(_)));
}

#[tokio::test]
async fn test_derive_key_rate_limit() {
    let (_dir, transport) = setup_transport();

    transport
        .call("app.chain.switch_to_mainnet", json!({}))
        .await
        .expect("switch chain must succeed");

    let created = create_wallet(&transport, "CorrectPassw0rd!").await;
    let session = unlock_wallet(&transport, &created.wallet_id.0, "CorrectPassw0rd!").await;

    let params = json!({"session": session, "path": "m/44'/1337'/0'/0/0"});

    for _ in 0..20 {
        transport
            .call("wallet.key.derive_receiver", params.clone())
            .await
            .expect("derive_key must succeed before rate limit");
    }

    let err = transport
        .call("wallet.key.derive_receiver", params)
        .await
        .expect_err("derive_key must be rate limited after 20 calls");

    let err_dbg = format!("{err:?}");
    assert!(
        err_dbg.contains("Rate limited:"),
        "unexpected error: {err_dbg}"
    );
}

#[tokio::test]
async fn test_addr_limit_env() {
    // IMPORTANT: Guard with a global lock to avoid cross-test env var races.
    // Set env vars BEFORE setup_transport() so wallet identity + rate limit resolve correctly.
    let _env = AddrLimitEnvGuard::new("p2p", "mainnet", 1, 2);

    let (_dir, transport) = setup_transport();

    transport
        .call("app.chain.switch_to_mainnet", json!({}))
        .await
        .expect("switch chain must succeed");

    let created = create_wallet(&transport, "CorrectPassw0rd!").await;
    let session = unlock_wallet(&transport, &created.wallet_id.0, "CorrectPassw0rd!").await;

    for i in 0..2u32 {
        let params = json!({"session": session, "path": format!("m/44'/1337'/0'/0/{i}")});
        transport
            .call("wallet.key.derive_receiver", params)
            .await
            .expect("derive_key must succeed within Phase 14 burst");
    }

    let err = transport
        .call(
            "wallet.key.derive_receiver",
            json!({"session": session, "path": "m/44'/1337'/0'/0/2"}),
        )
        .await
        .expect_err("derive_key must be rate limited by Phase 14 after burst");

    let err_dbg = format!("{err:?}");
    assert!(
        err_dbg.to_lowercase().contains("rate limit exceeded"),
        "unexpected error: {err_dbg}"
    );
}
