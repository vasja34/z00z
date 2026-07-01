#![cfg(not(target_arch = "wasm32"))]

use std::sync::{Arc, MutexGuard, OnceLock};

use async_trait::async_trait;
use z00z_crypto::expert::encoding::SafePassword;
use z00z_utils::codec::json;

use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcError, RpcTransport};
use z00z_wallets::rpc::methods::{AppRpcImpl, KeyRpcImpl, WalletRpcImpl};
use z00z_wallets::rpc::register_all_wallet_rpc_methods;
use z00z_wallets::rpc::types::key::{RuntimeDeriveReceiverResponse, RuntimeListReceiversResponse};
use z00z_wallets::rpc::types::security::SessionToken;
use z00z_wallets::{
    key::Bip44Path,
    services::{AppService, RateLimitPrecheck, WalletService},
    WalletError,
};

#[path = "test_inc/test_wallet_env_lock.inc"]
mod test_common;

struct WalletChainEnvGuard {
    _guard: MutexGuard<'static, ()>,
    prev_network: Option<String>,
    prev_chain: Option<String>,
}

impl WalletChainEnvGuard {
    fn new(network: &str, chain: &str) -> Self {
        let guard = test_common::wallet_env_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let prev_network = std::env::var("Z00Z_WALLET_NETWORK").ok();
        let prev_chain = std::env::var("Z00Z_WALLET_CHAIN").ok();

        std::env::set_var("Z00Z_WALLET_NETWORK", network);
        std::env::set_var("Z00Z_WALLET_CHAIN", chain);

        Self {
            _guard: guard,
            prev_network,
            prev_chain,
        }
    }
}

impl Drop for WalletChainEnvGuard {
    fn drop(&mut self) {
        match &self.prev_network {
            Some(value) => std::env::set_var("Z00Z_WALLET_NETWORK", value),
            None => std::env::remove_var("Z00Z_WALLET_NETWORK"),
        }

        match &self.prev_chain {
            Some(value) => std::env::set_var("Z00Z_WALLET_CHAIN", value),
            None => std::env::remove_var("Z00Z_WALLET_CHAIN"),
        }
    }
}

fn wallet_service_errors_serial_lock() -> &'static tokio::sync::Mutex<()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

struct TestTransport<T: RpcTransport> {
    _dir: tempfile::TempDir,
    inner: T,
}

#[async_trait(?Send)]
impl<T: RpcTransport> RpcTransport for TestTransport<T> {
    async fn call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, RpcError> {
        self.inner.call(method, params).await
    }
}

fn setup_key_transport(service: Arc<WalletService>) -> impl RpcTransport {
    let dir = tempfile::tempdir().unwrap();
    let wallet_rpc = Arc::new(WalletRpcImpl::new(Arc::clone(&service)));
    let app_service = Arc::new(AppService::with_wallet_service(Arc::clone(&service)));
    let app_rpc = Arc::new(AppRpcImpl::new(Arc::clone(&app_service)));
    let key_rpc = Arc::new(KeyRpcImpl::new(Arc::clone(&service)));

    let dispatcher = Arc::new(RpcDispatcher::new());
    register_all_wallet_rpc_methods(
        &dispatcher,
        app_rpc,
        wallet_rpc,
        Arc::new(z00z_wallets::rpc::methods::AssetRpcImpl::new()),
        Arc::new(z00z_wallets::rpc::methods::TxRpcImpl::new(Arc::clone(
            &service,
        ))),
        Arc::new(z00z_wallets::rpc::methods::BackupRpcImpl::new(Arc::clone(
            &service,
        ))),
        key_rpc,
        Arc::new(z00z_wallets::rpc::methods::ChainRpcImpl::new(Arc::clone(
            &app_service,
        ))),
        Arc::new(
            z00z_wallets::rpc::methods::NetworkRpcImpl::with_app_service(Arc::clone(&app_service)),
        ),
        Arc::new(z00z_wallets::rpc::methods::ChainScanRpcImpl::new(
            Arc::clone(&app_service),
        )),
        Arc::new(z00z_wallets::rpc::methods::StorageRpcImpl::new(service)),
    )
    .expect("wallet RPC registration should succeed");

    TestTransport {
        _dir: dir,
        inner: LocalRpcTransport::new(dispatcher),
    }
}

#[tokio::test]
async fn test_shallow_services_limit_contract() {
    let _serial = wallet_service_errors_serial_lock().lock().await;
    let allowed = RateLimitPrecheck::Allowed;
    assert!(matches!(allowed, RateLimitPrecheck::Allowed));
}

#[tokio::test]
async fn test_derive_rejects_chain_config() {
    let _serial = wallet_service_errors_serial_lock().lock().await;
    let temp = tempfile::tempdir().expect("tempdir");
    let service = Arc::new(WalletService::with_output_dir(temp.path().join("wallets")));
    let app = AppService::with_wallet_service(Arc::clone(&service));

    let password = "Aa1!bB2@cC3#dD4$eE5%";
    let created = {
        let _env = WalletChainEnvGuard::new("p2p", "broken-chain");
        app.create_wallet("wallet-errors".to_string(), password.to_string(), None)
            .await
            .expect("create wallet")
    };

    let _session = {
        let _env = WalletChainEnvGuard::new("p2p", "broken-chain");
        service
            .unlock_wallet_in_memory(&created.wallet_id, &SafePassword::from(password))
            .await
            .expect("unlock wallet")
    };

    let err = service
        .derive_public_key_for_path(&created.wallet_id, Bip44Path::payment(0).unwrap())
        .await
        .unwrap_err();

    match err {
        WalletError::InvalidConfig(message) => {
            assert!(message.contains("invalid wallet chain 'broken-chain'"));
        }
        other => panic!("expected invalid config, got {other:?}"),
    }
}

#[tokio::test]
async fn test_list_receivers_persisted_chain() {
    let _serial = wallet_service_errors_serial_lock().lock().await;
    let temp = tempfile::tempdir().expect("tempdir");
    let service = Arc::new(WalletService::with_output_dir(temp.path().join("wallets")));
    let app = AppService::with_wallet_service(Arc::clone(&service));
    let transport = setup_key_transport(Arc::clone(&service));

    let source_dir = tempfile::tempdir().expect("source tempdir");
    let source_path = source_dir.path().join("wallet-selected.wlt");

    {
        let _env = WalletChainEnvGuard::new("p2p", "devnet");
        let app = AppService::with_wallet_service(Arc::clone(&service));
        let created = app
            .create_wallet(
                "wallet-errors-source".to_string(),
                "Aa1!bB2@cC3#dD4$eE5%".to_string(),
                None,
            )
            .await
            .expect("create source wallet");
        let managed_path = std::fs::read_dir(temp.path().join("wallets"))
            .expect("read wallet output dir")
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .find(|path| path.extension().is_some_and(|ext| ext == "wlt"))
            .unwrap_or_else(|| panic!("missing persisted .wlt for {}", created.wallet_id.0));
        std::fs::copy(&managed_path, &source_path).expect("copy source wallet");
    }

    let source_bytes = std::fs::read(&source_path).expect("read source wallet bytes");

    let discovery = {
        let _env = WalletChainEnvGuard::new("p2p", "broken-chain");
        app.open_wallet_source(z00z_wallets::rpc::types::wallet::WalletSource::Bytes {
            bytes: source_bytes,
        })
        .await
        .expect("open_wallet_source must succeed")
    };

    let unlocked = transport
        .call(
            "wallet.session.unlock_wallet",
            json!({
                "wallet_id": discovery.wallet_id.0,
                "password": "Aa1!bB2@cC3#dD4$eE5%"
            }),
        )
        .await
        .expect("unlock must succeed");

    let session: SessionToken = serde_json::from_value(unlocked).expect("session must deserialize");

    let derived = {
        let _env = WalletChainEnvGuard::new("p2p", "broken-chain");
        transport
            .call(
                "wallet.key.derive_receiver",
                json!({"session": session, "path": Bip44Path::payment(0).unwrap().to_string()}),
            )
            .await
            .expect("derive_receiver must succeed")
    };

    let derived: RuntimeDeriveReceiverResponse =
        serde_json::from_value(derived).expect("derive_receiver response must deserialize");

    let public_key_vec = hex::decode(&derived.public_key).expect("public key hex");
    let public_key: [u8; 32] = public_key_vec
        .try_into()
        .expect("public key must be 32 bytes");
    let expected_receiver_id = hex::encode(public_key);

    assert_eq!(derived.public_key, expected_receiver_id);

    let listed = {
        let _env = WalletChainEnvGuard::new("p2p", "broken-chain");
        transport
            .call(
                "wallet.key.list_receivers",
                json!({"session": session, "limit": 10, "cursor": null, "filter": null}),
            )
            .await
            .expect("list_receivers must succeed")
    };

    let listed: RuntimeListReceiversResponse =
        serde_json::from_value(listed).expect("list_receivers response must deserialize");
    assert_eq!(listed.items.len(), 1);
    assert_eq!(listed.items[0].receiver_id, expected_receiver_id);
    assert_eq!(listed.items[0].public_key, derived.public_key);
}
