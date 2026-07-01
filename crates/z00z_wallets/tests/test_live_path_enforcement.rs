#![cfg(not(target_arch = "wasm32"))]

use std::sync::{Arc, OnceLock};

use tokio::sync::Mutex;
use z00z_core::assets::{encode_asset_pkg_json, AssetClass, AssetPkgWire, AssetWire};
use z00z_crypto::{create_range_proof, Z00ZScalar};
use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcError, RpcTransport};
use z00z_wallets::key::ReceiverKeys;
use z00z_wallets::rpc::methods::{
    AppRpcImpl, AssetRpcImpl, BackupRpcImpl, ChainRpcImpl, ChainScanRpcImpl, KeyRpcImpl,
    NetworkRpcImpl, StorageRpcImpl, TxRpcImpl, WalletRpcImpl,
};
use z00z_wallets::rpc::register_all_wallet_rpc_methods;
use z00z_wallets::rpc::types::asset::RuntimeImportAssetResponse;
use z00z_wallets::rpc::types::wallet::{RuntimeCreateWalletResponse, SessionToken};
use z00z_wallets::services::{AppService, WalletService};
use z00z_wallets::{build_tx_output_unchecked, SenderWallet};
use z00z_wallets::{
    receiver::ReceiverCard,
    receiver::{ScanResult, StealthOutputScanner},
};

const PASSWORD: &str = "CorrectPassw0rd!";
const TEST_SEED_PHRASE_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct TestEnv {
    transport: LocalRpcTransport,
    asset_rpc: Arc<AssetRpcImpl>,
}

fn setup_env(output_dir: std::path::PathBuf) -> TestEnv {
    let wallet_service = Arc::new(WalletService::with_output_dir(output_dir));
    let app_service = Arc::new(AppService::with_wallet_service(Arc::clone(&wallet_service)));

    let wallet_rpc = Arc::new(WalletRpcImpl::new(Arc::clone(&wallet_service)));
    let app_rpc = Arc::new(AppRpcImpl::new(Arc::clone(&app_service)));
    let asset_rpc = Arc::new(AssetRpcImpl::with_wallet_service(Arc::clone(
        &wallet_service,
    )));
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
        Arc::clone(&asset_rpc),
        tx_rpc,
        backup_rpc,
        key_rpc,
        chain_rpc,
        network_rpc,
        scan_rpc,
        storage_rpc,
    )
    .expect("wallet RPC registration should succeed");

    TestEnv {
        transport: LocalRpcTransport::new(dispatcher),
        asset_rpc,
    }
}

async fn create_wallet(env: &TestEnv, name: &str) -> RuntimeCreateWalletResponse {
    let value = env
        .transport
        .call(
            "app.wallet.create_wallet",
            serde_json::json!({"name": name, "password": PASSWORD, "seed_phrase": TEST_SEED_PHRASE_24}),
        )
        .await
        .expect("create_wallet must succeed");

    serde_json::from_value(value).expect("RuntimeCreateWalletResponse must deserialize")
}

async fn unlock_wallet(env: &TestEnv, wallet_id: &str) -> SessionToken {
    let value = env
        .transport
        .call(
            "wallet.session.unlock_wallet",
            serde_json::json!({"wallet_id": wallet_id, "password": PASSWORD}),
        )
        .await
        .expect("unlock_wallet must succeed");

    serde_json::from_value(value).expect("SessionToken must deserialize")
}

fn mk_recv_card(keys: &ReceiverKeys) -> ReceiverCard {
    ReceiverCard {
        version: 1,
        owner_handle: keys.owner_handle,
        view_pk: keys.view_pk.as_bytes().try_into().expect("view pk"),
        identity_pk: keys.identity_pk.as_bytes().try_into().expect("identity pk"),
        card_id: None,
        metadata: None,
        signature: [0u8; 64],
    }
}

fn mk_owned_wire(serial_id: u32, keys: &ReceiverKeys) -> AssetWire {
    let mut asset = z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 0, 10)
        .expect("valid std asset");

    let card = mk_recv_card(keys);
    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut SenderWallet::new([51u8; 32]),
        &[serial_id as u8; 32],
        0,
        asset.amount,
        &asset.definition.id,
    )
    .expect("stealth output");

    asset.commitment = z00z_crypto::Commitment::from_bytes(&output.c_amount)
        .expect("commitment")
        .0;
    asset.owner_pub = None;
    asset.owner_signature = None;
    asset.r_pub = Some(output.r_pub);
    asset.owner_tag = Some(output.owner_tag);
    asset.enc_pack = Some(output.enc_pack);
    asset.tag16 = output.tag16;
    asset.leaf_ad_id = Some(asset.definition.id);

    let scanner = StealthOutputScanner::from_keys(keys);
    let ScanResult::Mine { wallet_output } = scanner.scan_leaf(&asset) else {
        panic!("owned leaf expected")
    };
    let blinding = Z00ZScalar::try_from_bytes(wallet_output.blinding.expect("blinding bytes"))
        .expect("blinding scalar");
    asset.range_proof = Some(create_range_proof(asset.amount, &blinding, 64, 0).expect("proof"));

    let mut wire = AssetWire::from_asset(&asset);
    wire.secret = None;
    wire
}

#[tokio::test]
async fn test_live_path_verify_complete() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();
    env.asset_rpc.reset_verify_complete_count();

    let created = create_wallet(&env, "Live Verify Wallet").await;
    let session = unlock_wallet(&env, &created.wallet_id.0).await;

    let wallet_id = created.wallet_id.clone();
    let keys = env
        .asset_rpc
        .test_receiver_keys(&wallet_id)
        .await
        .expect("receiver keys");
    let wire = mk_owned_wire(1, &keys);

    let value = env
        .transport
        .call(
            "wallet.asset.import_asset",
            serde_json::json!({"session": session, "asset_data": dto_json(&wire)}),
        )
        .await
        .expect("import should succeed");

    let response: RuntimeImportAssetResponse =
        serde_json::from_value(value).expect("import response decode");
    assert!(response.status.success);
    assert!(response.is_inserted);
    assert!(!response.asset_already_exists);
    assert!(env.asset_rpc.verify_complete_count() >= 1);
}

#[tokio::test]
async fn test_live_path_stub_removed() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();
    env.asset_rpc.reset_verify_complete_count();

    let created = create_wallet(&env, "Live Reject Wallet").await;
    let session = unlock_wallet(&env, &created.wallet_id.0).await;

    let mut wire = AssetWire::from_asset(
        &z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 2, 10)
            .expect("valid std asset"),
    );
    let proof = wire.range_proof.as_mut().expect("range proof present");
    proof[0] ^= 1;

    let err = env
        .transport
        .call(
            "wallet.asset.import_asset",
            serde_json::json!({"session": session, "asset_data": dto_json(&wire)}),
        )
        .await
        .expect_err("invalid wire must fail");

    match err {
        RpcError::InvalidParams(message) => assert_eq!(message, "IMPORT_CRYPTO_VERIFY_FAILED"),
        other => panic!("expected InvalidParams, got {other:?}"),
    }
    assert_eq!(env.asset_rpc.verify_complete_count(), 0);
}

fn dto_json(wire: &AssetWire) -> String {
    let dto = AssetPkgWire::from_wire(wire);
    let bytes = encode_asset_pkg_json(&dto).expect("encode dto");
    String::from_utf8(bytes).expect("dto utf8")
}
