#![cfg(not(target_arch = "wasm32"))]

use std::sync::{Arc, OnceLock};

use tokio::sync::Mutex;
use z00z_utils::codec::json;

use z00z_core::assets::{encode_asset_pkg_json, AssetClass, AssetPkgWire, AssetWire};
use z00z_crypto::{create_range_proof, Z00ZScalar};
use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcTransport};
use z00z_wallets::key::ReceiverKeys;
use z00z_wallets::rpc::methods::{
    AppRpcImpl, AssetRpcImpl, BackupRpcImpl, ChainRpcImpl, ChainScanRpcImpl, KeyRpcImpl,
    NetworkRpcImpl, StorageRpcImpl, TxRpcImpl, WalletRpcImpl,
};
use z00z_wallets::rpc::register_all_wallet_rpc_methods;
use z00z_wallets::rpc::types::wallet::{RuntimeCreateWalletResponse, SessionToken};
use z00z_wallets::services::{AppService, WalletService};
use z00z_wallets::{build_tx_output_unchecked, SenderWallet};
use z00z_wallets::{
    receiver::ReceiverCard,
    receiver::{ScanResult, StealthOutputScanner},
};

const PASSWORD: &str = "CorrectPassw0rd!";
const SEED24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

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
            json!({"name": name, "password": PASSWORD, "seed_phrase": SEED24}),
        )
        .await
        .expect("create_wallet must succeed");

    serde_json::from_value(value).expect("RuntimeCreateWalletResponse")
}

async fn unlock_wallet(env: &TestEnv, wallet_id: &str) -> SessionToken {
    let value = env
        .transport
        .call(
            "wallet.session.unlock_wallet",
            json!({"wallet_id": wallet_id, "password": PASSWORD}),
        )
        .await
        .expect("unlock_wallet must succeed");

    serde_json::from_value(value).expect("SessionToken")
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

fn mk_stealth_wire(serial_id: u32, keys: &ReceiverKeys) -> AssetWire {
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

    let tag16 = output.tag16.expect("tag16");

    asset.commitment = z00z_crypto::Commitment::from_bytes(&output.c_amount)
        .expect("commitment")
        .as_commitment()
        .clone();
    asset.owner_pub = None;
    asset.owner_signature = None;
    asset.r_pub = Some(output.r_pub);
    asset.owner_tag = Some(output.owner_tag);
    asset.enc_pack = Some(output.enc_pack);
    asset.tag16 = Some(tag16);
    asset.leaf_ad_id = Some(asset.definition.id);

    let scanner = StealthOutputScanner::from_keys(keys);
    let ScanResult::Mine { wallet_output } = scanner.scan_leaf(&asset) else {
        panic!("owned leaf")
    };
    let blinding_bytes = wallet_output.blinding.expect("blinding bytes");
    let blinding = Z00ZScalar::try_from_bytes(blinding_bytes).expect("blinding scalar");
    asset.range_proof = Some(create_range_proof(asset.amount, &blinding, 64, 0).expect("proof"));

    let mut wire = AssetWire::from_asset(&asset);
    wire.secret = None;
    wire
}

async fn import_wire(env: &TestEnv, session: SessionToken, wire: &AssetWire) -> serde_json::Value {
    env.transport
        .call(
            "wallet.asset.import_asset",
            json!({
                "session": session,
                "asset_data": dto_json(wire)
            }),
        )
        .await
        .expect("import must succeed")
}

fn dto_json(wire: &AssetWire) -> String {
    let dto = AssetPkgWire::from_wire(wire);
    let bytes = encode_asset_pkg_json(&dto).expect("encode dto");
    let text = String::from_utf8(bytes).expect("dto utf8");
    let value: serde_json::Value = serde_json::from_str(&text).expect("dto json");
    assert!(value.get("secret").is_none());
    assert!(value.get("is_frozen").is_none());
    assert!(value.get("is_slashed").is_none());
    text
}

fn assert_dup(value: &serde_json::Value, message: &str, is_inserted: bool) {
    assert_eq!(
        value.get("message").and_then(serde_json::Value::as_str),
        Some(message)
    );
    assert_eq!(
        value
            .get("is_inserted")
            .and_then(serde_json::Value::as_bool),
        Some(is_inserted)
    );
    assert_eq!(
        value
            .get("asset_already_exists")
            .and_then(serde_json::Value::as_bool),
        Some(!is_inserted)
    );
}

async fn get_balance(env: &TestEnv, wallet_id: &str, asset_id: [u8; 32]) -> serde_json::Value {
    env.transport
        .call(
            "wallet.asset.get_asset_balance",
            json!({"wallet_id": wallet_id, "asset_id": asset_id}),
        )
        .await
        .expect("get_asset_balance must succeed")
}

async fn list_items(env: &TestEnv, wallet_id: &str) -> Vec<serde_json::Value> {
    let value = env
        .transport
        .call("wallet.asset.list_assets", json!({"wallet_id": wallet_id}))
        .await
        .expect("list_assets must succeed");

    if let Some(items) = value.get("assets").and_then(serde_json::Value::as_array) {
        return items.clone();
    }

    panic!("list_assets response must expose canonical assets array")
}

#[tokio::test]
async fn test_duplicate_import_idempotent() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let created = create_wallet(&env, "Replay Wallet").await;
    let session = unlock_wallet(&env, &created.wallet_id.0).await;

    let recv_keys = env
        .asset_rpc
        .test_receiver_keys(&created.wallet_id)
        .await
        .expect("receiver keys");
    let wire = mk_stealth_wire(1, &recv_keys);
    let asset_id = wire.clone().to_asset().expect("to_asset").asset_id();

    let first = import_wire(&env, session.clone(), &wire).await;
    assert_dup(&first, "asset_imported", true);

    let bal_before = get_balance(&env, &created.wallet_id.0, asset_id).await;

    let second = import_wire(&env, session, &wire).await;
    assert_dup(&second, "asset_already_exists", false);

    let bal_after = get_balance(&env, &created.wallet_id.0, asset_id).await;
    assert_eq!(bal_before, bal_after);
}

#[tokio::test]
async fn test_replay_no_balance() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let created = create_wallet(&env, "Replay Wallet").await;
    let session = unlock_wallet(&env, &created.wallet_id.0).await;

    let recv_keys = env
        .asset_rpc
        .test_receiver_keys(&created.wallet_id)
        .await
        .expect("receiver keys");
    let wire = mk_stealth_wire(2, &recv_keys);
    let asset_id = wire.clone().to_asset().expect("to_asset").asset_id();

    let _ = import_wire(&env, session.clone(), &wire).await;
    let one_balance = get_balance(&env, &created.wallet_id.0, asset_id).await;

    for _ in 0..5 {
        let replay = import_wire(&env, session.clone(), &wire).await;
        assert_dup(&replay, "asset_already_exists", false);
    }

    let final_balance = get_balance(&env, &created.wallet_id.0, asset_id).await;
    assert_eq!(one_balance, final_balance);
}

#[tokio::test]
async fn test_replay_no_list_drift() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let created = create_wallet(&env, "Replay Wallet").await;
    let session = unlock_wallet(&env, &created.wallet_id.0).await;

    let recv_keys = env
        .asset_rpc
        .test_receiver_keys(&created.wallet_id)
        .await
        .expect("receiver keys");
    let wire = mk_stealth_wire(3, &recv_keys);

    let _ = import_wire(&env, session.clone(), &wire).await;
    let one_items = list_items(&env, &created.wallet_id.0).await;

    for _ in 0..4 {
        let replay = import_wire(&env, session.clone(), &wire).await;
        assert_dup(&replay, "asset_already_exists", false);
    }

    let final_items = list_items(&env, &created.wallet_id.0).await;
    assert_eq!(one_items, final_items);
}
