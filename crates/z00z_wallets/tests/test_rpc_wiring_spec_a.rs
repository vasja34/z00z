#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use async_trait::async_trait;
use z00z_utils::codec::json;

use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcError, RpcTransport};
use z00z_utils::logger::Logger;
use z00z_utils::rng::SystemRngProvider;
use z00z_utils::time::{SystemTimeProvider, TimeProvider};

#[path = "test_inc/test_rpc_logger.inc"]
mod test_common;

use z00z_core::assets::AssetClass;
use z00z_crypto::{create_range_proof, Commitment, Z00ZScalar};
use z00z_wallets::chain::ReceiverCardRecord;
use z00z_wallets::receiver::receiver_card::format_receiver_handle;
use z00z_wallets::receiver::ValidateReceiverCard;
use z00z_wallets::receiver::{ScanResult, StealthOutputScanner};
use z00z_wallets::rpc::types::chain::RuntimeStartScanResponse;
use z00z_wallets::rpc::types::common::PersistWalletId;
use z00z_wallets::rpc::types::key::RuntimeGetReceiverCardResponse;
use z00z_wallets::rpc::types::network::{RuntimeChainSettingsResponse, RuntimeSwitchChainResponse};
use z00z_wallets::rpc::types::wallet::{
    PersistWalletInfo, RuntimeCreateWalletResponse, RuntimeLockWalletResponse,
    RuntimeShowSeedPhraseResponse,
};
use z00z_wallets::rpc::{
    logging::{LoggedRpcTransport, RpcLoggingConfig},
    methods::{
        AppRpcImpl, AssetRpcImpl, BackupRpcImpl, ChainRpcImpl, ChainScanRpcImpl, KeyRpcImpl,
        NetworkRpcImpl, StorageRpcImpl, TxRpcImpl, WalletRpcImpl,
    },
    register_all_wallet_rpc_methods,
};
use z00z_wallets::services::{AppService, WalletService};
use z00z_wallets::stealth::{build_tx_output_serial_unchecked, SenderWallet};

struct TestTransport<T: RpcTransport> {
    _dir: tempfile::TempDir,
    service: Arc<WalletService>,
    inner: T,
}

type TestLoggedTransport = TestTransport<LoggedRpcTransport<LocalRpcTransport, SystemRngProvider>>;

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

fn test_time_provider() -> Arc<dyn TimeProvider> {
    Arc::new(SystemTimeProvider)
}

fn test_logging_config() -> RpcLoggingConfig {
    let mut cfg =
        RpcLoggingConfig::from_default_wallet_yaml().expect("RPC logging config must load");
    cfg.enabled = true;
    cfg
}

fn setup_logged_transport(logger: Arc<dyn Logger>) -> TestLoggedTransport {
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

    let time = test_time_provider();
    let rng = SystemRngProvider;

    let transport = LoggedRpcTransport::new(base, test_logging_config(), logger, time, rng);
    TestTransport {
        _dir: dir,
        service,
        inner: transport,
    }
}

async fn seed_spendable_coin(transport: &TestLoggedTransport, wallet_id: &PersistWalletId) {
    let mut asset =
        z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 7, 50_000)
            .expect("valid std asset");
    let recv_keys = transport
        .service
        .receiver_keys(wallet_id)
        .await
        .expect("receiver keys");
    let card = recv_keys.export_receiver_card().expect("receiver card");
    let mut sender_wallet = SenderWallet::new([41u8; 32]);
    let output = build_tx_output_serial_unchecked(
        &card,
        None,
        &mut sender_wallet,
        &[7u8; 32],
        asset.serial_id,
        asset.amount,
        &asset.definition.id,
        asset.serial_id,
    )
    .expect("stealth output");

    asset.commitment = Commitment::from_bytes(&output.c_amount)
        .expect("commitment")
        .0;
    asset.owner_pub = None;
    asset.owner_signature = None;
    asset.r_pub = Some(output.r_pub);
    asset.owner_tag = Some(output.owner_tag);
    asset.enc_pack = Some(output.enc_pack);
    asset.tag16 = output.tag16;
    asset.leaf_ad_id = Some(asset.definition.id);

    let scanner = StealthOutputScanner::from_keys(&recv_keys);
    if let ScanResult::Mine { wallet_output } = scanner.scan_leaf(&asset) {
        if let Some(blinding_bytes) = wallet_output.blinding.as_ref().copied() {
            let blinding = Z00ZScalar::try_from_bytes(blinding_bytes).expect("blinding scalar");
            asset.range_proof =
                Some(create_range_proof(asset.amount, &blinding, 64, 0).expect("proof"));
        }
    }

    transport
        .service
        .put_claimed_asset(wallet_id, asset)
        .await
        .expect("seed put_claimed_asset must succeed");
}

async fn create_wallet(
    transport: &impl RpcTransport,
    password: &str,
) -> RuntimeCreateWalletResponse {
    let value = transport
        .call(
            "app.wallet.create_wallet",
            json!({
                "name": "Spec Wallet",
                "password": password,
                "seed_phrase": null
            }),
        )
        .await
        .expect("create_wallet must succeed");

    serde_json::from_value(value).expect("RuntimeCreateWalletResponse must deserialize")
}

#[tokio::test]
async fn test_app_wallet_wallets_accepts() {
    let (logger, _vec_logger) = test_common::rpc_test_tee_logger();
    let transport = setup_logged_transport(logger);

    let value = transport
        .call("app.wallet.list_wallets", json!({}))
        .await
        .expect("list_wallets must succeed");

    let wallets: Vec<PersistWalletInfo> = serde_json::from_value(value)
        .expect("list_wallets result must deserialize into Vec<PersistWalletInfo>");

    assert!(wallets.is_empty() || !wallets[0].name.is_empty());
}

#[tokio::test]
async fn test_app_wallet_wallet_rejects() {
    let (logger, _vec_logger) = test_common::rpc_test_tee_logger();
    let transport = setup_logged_transport(logger);

    let err = transport
        .call("app.wallet.create_wallet", json!({"name": "x"}))
        .await
        .expect_err("missing password must fail");

    assert!(matches!(err, RpcError::InvalidParams(_)));
}

#[tokio::test]
async fn test_app_wallet_wallet_wrong() {
    let (logger, vec_logger) = test_common::rpc_test_tee_logger();
    let transport = setup_logged_transport(logger);

    let created = create_wallet(&transport, "CorrectPassw0rd!").await;

    vec_logger.clear();

    let err = transport
        .call(
            "app.wallet.delete_wallet",
            json!({"wallet_id": created.wallet_id.0, "password": "WrongPassw0rd!"}),
        )
        .await
        .expect_err("wrong password must fail");

    assert!(matches!(err, RpcError::AuthFailed));

    let lines = vec_logger.lines();
    assert_eq!(lines.len(), 2, "expected request + error log");
    for line in &lines {
        assert!(
            !line.contains("WrongPassw0rd!"),
            "password must not appear in logs"
        );
    }
}

#[tokio::test]
async fn test_app_chain_methods_accept() {
    let (logger, _vec_logger) = test_common::rpc_test_tee_logger();
    let transport = setup_logged_transport(logger);

    for method in [
        "app.chain.switch_to_mainnet",
        "app.chain.switch_to_testnet",
        "app.chain.switch_to_devnet",
    ] {
        let value = transport
            .call(method, json!({}))
            .await
            .unwrap_or_else(|e| panic!("{method} must succeed, got {e:?}"));

        let _resp: RuntimeSwitchChainResponse = serde_json::from_value(value)
            .unwrap_or_else(|e| panic!("{method} response must deserialize: {e}"));
    }
}

#[tokio::test]
async fn test_app_network_tor_rejects() {
    let (logger, _vec_logger) = test_common::rpc_test_tee_logger();
    let transport = setup_logged_transport(logger);

    let err = transport
        .call("app.network.switch_to_tor", json!({}))
        .await
        .expect_err("missing enable must fail");

    assert!(matches!(err, RpcError::InvalidParams(_)));

    let ok = transport
        .call("app.network.switch_to_tor", json!({"enable": true}))
        .await
        .expect("switch_to_tor must succeed with enable");

    let _resp: RuntimeChainSettingsResponse = serde_json::from_value(ok)
        .expect("switch_to_tor response must deserialize into RuntimeChainSettingsResponse");
}

#[tokio::test]
async fn test_app_chain_stop_local_scan() {
    let (logger, _vec_logger) = test_common::rpc_test_tee_logger();
    let transport = setup_logged_transport(logger);

    let created = create_wallet(&transport, "StrongPassw0rd!").await;

    let value = transport
        .call(
            "app.chain.start_local_scan",
            json!({"wallet_id": created.wallet_id.0, "from_height": null}),
        )
        .await
        .expect("start_local_scan must succeed");

    let started: RuntimeStartScanResponse = serde_json::from_value(value)
        .expect("start_local_scan response must deserialize into RuntimeStartScanResponse");

    let job_id = started
        .job
        .job_id
        .expect("start_local_scan must return job_id");
    assert!(!job_id.trim().is_empty());

    let stopped = transport
        .call(
            "app.chain.stop_local_scan",
            json!({"wallet_id": created.wallet_id.0}),
        )
        .await
        .expect("stop_local_scan must succeed");

    assert!(stopped.is_null(), "stop_local_scan result must be null");
}

#[tokio::test]
async fn test_wallet_session_wrong_password() {
    let (logger, _vec_logger) = test_common::rpc_test_tee_logger();
    let transport = setup_logged_transport(logger);

    let created = create_wallet(&transport, "CorrectPassw0rd!").await;

    let err = transport
        .call(
            "wallet.session.unlock_wallet",
            json!({"wallet_id": created.wallet_id.0, "password": "WrongPassw0rd!"}),
        )
        .await
        .expect_err("wrong password must fail");

    assert!(matches!(err, RpcError::AuthFailed));
}

#[tokio::test]
async fn test_lock_transport_auth_required() {
    let (logger, _vec_logger) = test_common::rpc_test_tee_logger();
    let transport = setup_logged_transport(logger);

    let created = create_wallet(&transport, "CorrectPassw0rd!").await;
    let unlocked = transport
        .call(
            "wallet.session.unlock_wallet",
            json!({"wallet_id": created.wallet_id.0, "password": "CorrectPassw0rd!"}),
        )
        .await
        .expect("unlock_wallet must succeed");

    let session: z00z_wallets::rpc::types::security::SessionToken =
        serde_json::from_value(unlocked).expect("unlock_wallet result must deserialize");

    let err = transport
        .call(
            "wallet.session.lock_wallet",
            json!({"wallet_id": created.wallet_id.0}),
        )
        .await
        .expect_err("unauthenticated lock_wallet must fail");

    assert!(matches!(err, RpcError::InvalidParams(_)));

    let locked = transport
        .call(
            "wallet.session.lock_wallet",
            json!({"session": session.clone()}),
        )
        .await
        .expect("authenticated lock_wallet must succeed");

    let response: RuntimeLockWalletResponse =
        serde_json::from_value(locked).expect("lock_wallet response must deserialize");
    assert_eq!(response.wallet_id, created.wallet_id);

    let err = transport
        .call(
            "wallet.session.show_seed_phrase",
            json!({
                "session": session,
                "password": "CorrectPassw0rd!",
                "confirmation": "I understand"
            }),
        )
        .await
        .expect_err("locked wallet session must be unusable");

    assert!(matches!(
        err,
        RpcError::SessionInvalid | RpcError::SessionExpired
    ));
}

#[tokio::test]
async fn test_wallet_session_seed_phrase() {
    let (logger, vec_logger) = test_common::rpc_test_tee_logger();
    let transport = setup_logged_transport(logger);

    let created = create_wallet(&transport, "CorrectPassw0rd!").await;

    let unlocked = transport
        .call(
            "wallet.session.unlock_wallet",
            json!({"wallet_id": created.wallet_id.0, "password": "CorrectPassw0rd!"}),
        )
        .await
        .expect("unlock_wallet must succeed");

    let session: z00z_wallets::rpc::types::security::SessionToken =
        serde_json::from_value(unlocked).expect("unlock_wallet result must deserialize");

    vec_logger.clear();

    let value = transport
        .call(
            "wallet.session.show_seed_phrase",
            json!({"session": session, "password": "CorrectPassw0rd!", "confirmation": "I understand"}),
        )
        .await
        .expect("show_seed_phrase must succeed");

    let response: RuntimeShowSeedPhraseResponse =
        serde_json::from_value(value).expect("show_seed_phrase result must deserialize");

    let encrypted_payload = &response.encrypted_payload;
    assert!(
        !encrypted_payload.ciphertext.trim().is_empty(),
        "ciphertext must be present"
    );

    let lines = vec_logger.lines();
    assert_eq!(lines.len(), 2, "expected request + response log");

    for line in &lines {
        assert!(
            !line.contains(&encrypted_payload.ciphertext),
            "encrypted payload ciphertext must not appear in logs"
        );
        assert!(
            !line.contains("stub-seed-phrase"),
            "plaintext marker must not appear in logs"
        );
    }
}

#[tokio::test]
async fn test_wallet_key_master_key() {
    let (logger, _vec_logger) = test_common::rpc_test_tee_logger();
    let transport = setup_logged_transport(logger);

    let created = create_wallet(&transport, "CorrectPassw0rd!").await;

    let unlocked = transport
        .call(
            "wallet.session.unlock_wallet",
            json!({"wallet_id": created.wallet_id.0, "password": "CorrectPassw0rd!"}),
        )
        .await
        .expect("unlock_wallet must succeed");

    let session: z00z_wallets::rpc::types::security::SessionToken =
        serde_json::from_value(unlocked).expect("unlock_wallet result must deserialize");

    let err = transport
        .call(
            "wallet.key.rotate_master_key",
            json!({
                "session": session,
                "confirmation": "NOPE"
            }),
        )
        .await
        .expect_err("confirmation mismatch must fail");

    assert!(matches!(err, RpcError::InvalidParams(_)));
}

#[tokio::test]
async fn test_receiver_card_ok() {
    let (logger, _vec_logger) = test_common::rpc_test_tee_logger();
    let transport = setup_logged_transport(logger);

    let created = create_wallet(&transport, "CorrectPassw0rd!").await;
    let unlocked = transport
        .call(
            "wallet.session.unlock_wallet",
            json!({"wallet_id": created.wallet_id.0, "password": "CorrectPassw0rd!"}),
        )
        .await
        .expect("unlock_wallet must succeed");

    let session: z00z_wallets::rpc::types::security::SessionToken =
        serde_json::from_value(unlocked).expect("unlock_wallet result must deserialize");

    let value = transport
        .call("wallet.key.get_receiver_card", json!({"session": session}))
        .await
        .expect("wallet.key.get_receiver_card must succeed");

    let response: RuntimeGetReceiverCardResponse =
        serde_json::from_value(value).expect("response must deserialize");

    assert_eq!(response.owner_handle.len(), 64);
    assert_eq!(response.view_key.len(), 64);
    assert_eq!(response.identity_key.len(), 64);
    assert_eq!(response.signature.len(), 128);
    assert!(!response.card_compact.is_empty());
    assert!(response.owner_handle_display.starts_with("z00z1"));

    let decoded = ReceiverCardRecord::from_compact(&response.card_compact, None)
        .expect("card_compact must decode")
        .decode_card()
        .expect("record card must decode");
    decoded
        .validate_signature()
        .expect("receiver card signature must be valid");

    assert_eq!(response.owner_handle, hex::encode(decoded.owner_handle));
    assert_eq!(response.view_key, hex::encode(decoded.view_pk));
    assert_eq!(response.identity_key, hex::encode(decoded.identity_pk));
    assert_eq!(response.signature, hex::encode(decoded.signature));

    let expected_receiver_display = format_receiver_handle(&decoded.owner_handle)
        .expect("owner_handle must format to receiver display");
    assert_eq!(response.owner_handle_display, expected_receiver_display);
}

#[tokio::test]
async fn test_wallet_material_export_contract() {
    let (logger, _vec_logger) = test_common::rpc_test_tee_logger();
    let transport = setup_logged_transport(logger);

    let created = create_wallet(&transport, "CorrectPassw0rd!").await;
    let unlocked = transport
        .call(
            "wallet.session.unlock_wallet",
            json!({"wallet_id": created.wallet_id.0, "password": "CorrectPassw0rd!"}),
        )
        .await
        .expect("unlock_wallet must succeed");

    let session: z00z_wallets::rpc::types::security::SessionToken =
        serde_json::from_value(unlocked).expect("unlock_wallet result must deserialize");

    let exported = transport
        .call(
            "wallet.key.export_public_material",
            json!({"session": session, "account": 0, "password": "CorrectPassw0rd!"}),
        )
        .await
        .expect("wallet.key.export_public_material must succeed");

    let response: z00z_wallets::rpc::types::key::RuntimePubMaterialExportResponse =
        serde_json::from_value(exported).expect("export response must deserialize");
    assert_eq!(response.algorithm, "xchacha20poly1305");

    let err = transport
        .call("wallet.key.export_public_material", json!({"account": 0}))
        .await
        .expect_err("missing session must fail");

    assert!(matches!(err, RpcError::InvalidParams(_)));
}

#[tokio::test]
async fn test_receiver_card_params_invalid() {
    let (logger, _vec_logger) = test_common::rpc_test_tee_logger();
    let transport = setup_logged_transport(logger);

    let err = transport
        .call("wallet.key.get_receiver_card", json!({}))
        .await
        .expect_err("missing session must fail");

    assert!(matches!(err, RpcError::InvalidParams(_)));
}

#[tokio::test]
async fn test_wallet_asset_assets_missing() {
    let (logger, _vec_logger) = test_common::rpc_test_tee_logger();
    let transport = setup_logged_transport(logger);

    let err = transport
        .call("wallet.asset.list_assets", json!({}))
        .await
        .expect_err("missing wallet_id must fail");

    assert!(matches!(err, RpcError::InvalidParams(_)));
}

#[tokio::test]
async fn test_wallet_storage_storage_missing() {
    let (logger, _vec_logger) = test_common::rpc_test_tee_logger();
    let transport = setup_logged_transport(logger);

    let err = transport
        .call(
            "wallet.storage.export_storage",
            json!({
                "format": "json",
                "include_deleted": false
            }),
        )
        .await
        .expect_err("missing path must fail");

    assert!(matches!(err, RpcError::InvalidParams(_)));
}

#[tokio::test]
async fn test_wallet_asset_asset_balance() {
    let (logger, _vec_logger) = test_common::rpc_test_tee_logger();
    let transport = setup_logged_transport(logger);

    let err = transport
        .call(
            "wallet.asset.get_asset_balance",
            json!({
                "wallet_id": "not-a-real-wallet-id"
            }),
        )
        .await
        .expect_err("missing asset_id must fail");

    assert!(matches!(err, RpcError::InvalidParams(_)));
}

#[tokio::test]
async fn test_wallet_asset_asset_missing() {
    let (logger, _vec_logger) = test_common::rpc_test_tee_logger();
    let transport = setup_logged_transport(logger);

    let err = transport
        .call(
            "wallet.asset.send_asset",
            json!({
                "session": {
                    "token": "not-a-real-session-token",
                    "wallet_id": "not-a-real-wallet-id",
                    "created_at": 0,
                    "expires_at": 0,
                    "last_activity_at": 0,
                    "permissions": []
                },
                "asset_id": vec![0u8; 32],
                "amount": 1
            }),
        )
        .await
        .expect_err("missing recipient must fail");

    assert!(matches!(err, RpcError::InvalidParams(_)));
}

#[tokio::test]
async fn test_wallet_tx_transaction_idempotent() {
    let (logger, _vec_logger) = test_common::rpc_test_tee_logger();
    let transport = setup_logged_transport(logger);

    let created = create_wallet(&transport, "CorrectPassw0rd!").await;

    // Unlock first to avoid permission-related errors.
    let session = transport
        .call(
            "wallet.session.unlock_wallet",
            json!({"wallet_id": created.wallet_id.0, "password": "CorrectPassw0rd!"}),
        )
        .await
        .expect("unlock_wallet must succeed");
    seed_spendable_coin(&transport, &created.wallet_id).await;

    let receiver = transport
        .call(
            "wallet.key.get_receiver_card",
            json!({"session": session.clone()}),
        )
        .await
        .expect("get_receiver_card must succeed");
    let receiver: RuntimeGetReceiverCardResponse =
        serde_json::from_value(receiver).expect("receiver card response must deserialize");

    let now_ms = SystemTimeProvider.compat_unix_timestamp_millis();

    let params = json!({
        "session": session,
        "asset_id": null,
        "recipient": receiver.card_compact,
        "amount": 1,
        "memo": "memo",
        "idempotency_key": "11111111-1111-1111-1111-111111111111",
        "timestamp": now_ms
    });

    let first = transport
        .call("wallet.tx.send_transaction", params.clone())
        .await
        .expect("first send_transaction must succeed");

    let second = transport
        .call("wallet.tx.send_transaction", params)
        .await
        .expect("second send_transaction must succeed");

    assert_eq!(first, second, "same idempotency key must not duplicate");
}
