#![cfg(not(target_arch = "wasm32"))]

use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use tokio::sync::Mutex;
use z00z_utils::codec::json;

use z00z_core::assets::{
    decode_asset_pkg_json, encode_asset_pkg_json, AssetClass, AssetPkgWire, AssetWire,
};
use z00z_crypto::expert::encoding::SafePassword;
use z00z_crypto::{create_range_proof, Z00ZScalar};
use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcError, RpcTransport};
use z00z_wallets::db::{
    discover_wallet_store, object_inventory_store, open_wallet_store, ObjectInventoryFilter,
    ObjectInventoryStore, OwnedObjectFamily, OwnedObjectPayload, WalletIdentity,
};
use z00z_wallets::key::ReceiverKeys;
use z00z_wallets::rpc::methods::{
    AppRpcImpl, AssetRpcImpl, BackupRpcImpl, ChainRpcImpl, ChainScanRpcImpl, KeyRpcImpl,
    NetworkRpcImpl, StorageRpcImpl, TxRpcImpl, WalletRpcImpl,
};
use z00z_wallets::rpc::register_all_wallet_rpc_methods;
use z00z_wallets::rpc::types::common::PersistWalletId;
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
    output_dir: PathBuf,
    transport: LocalRpcTransport,
    asset_rpc: Arc<AssetRpcImpl>,
}

fn setup_env(output_dir: std::path::PathBuf) -> TestEnv {
    let wallet_service = Arc::new(WalletService::with_output_dir(output_dir.clone()));
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
        output_dir,
        transport: LocalRpcTransport::new(dispatcher),
        asset_rpc,
    }
}

fn setup_env_no_store(output_dir: std::path::PathBuf) -> TestEnv {
    let wallet_service = Arc::new(WalletService::with_output_dir(output_dir.clone()));
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
        output_dir,
        transport: LocalRpcTransport::new(dispatcher),
        asset_rpc,
    }
}

fn find_wallet_file(output_dir: &Path) -> PathBuf {
    let mut matches = std::fs::read_dir(output_dir)
        .expect("read_dir must succeed")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("wlt"))
        .collect::<Vec<_>>();
    matches.sort();
    assert_eq!(
        matches.len(),
        1,
        "expected one canonical wallet file in output dir"
    );
    matches.pop().expect("wallet file")
}

fn open_wallet_session(
    output_dir: &Path,
    wallet_id: &PersistWalletId,
) -> z00z_wallets::db::WalletSession {
    let wallet_path = find_wallet_file(output_dir);
    let discovery = discover_wallet_store(&wallet_path).expect("discover wallet");
    let identity = WalletIdentity {
        network: discovery.network,
        chain: discovery.chain,
    };
    open_wallet_store(
        &wallet_path,
        wallet_id,
        &SafePassword::from(PASSWORD),
        &identity,
    )
    .expect("open wallet store")
}

async fn create_wallet(env: &TestEnv, name: &str) -> RuntimeCreateWalletResponse {
    let value = env
        .transport
        .call(
            "app.wallet.create_wallet",
            json!({"name": name, "password": PASSWORD, "seed_phrase": TEST_SEED_PHRASE_24}),
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
            json!({"wallet_id": wallet_id, "password": PASSWORD}),
        )
        .await
        .expect("unlock_wallet must succeed");

    serde_json::from_value(value).expect("SessionToken must deserialize")
}

fn assert_dto_json(asset_data: &str) {
    let value: serde_json::Value = serde_json::from_str(asset_data).expect("dto json");
    assert!(value.get("secret").is_none());
    assert!(value.get("is_frozen").is_none());
    assert!(value.get("is_slashed").is_none());
    assert!(decode_asset_pkg_json(asset_data.as_bytes()).is_ok());
}

fn tamper_hex(text: &mut String) {
    assert!(!text.is_empty(), "tamper target must not be empty");
    let mut bytes = text.as_bytes().to_vec();
    bytes[0] = if bytes[0] == b'a' { b'b' } else { b'a' };
    *text = String::from_utf8(bytes).expect("tampered hex utf8");
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
        &mut SenderWallet::new([41u8; 32]),
        &[serial_id as u8; 32],
        0,
        asset.amount,
        &asset.definition.id,
    )
    .expect("stealth output");

    let tag16 = output.tag16.expect("tag16");

    asset.commitment = z00z_crypto::Commitment::from_bytes(&output.c_amount)
        .expect("commitment")
        .0;
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

fn dto_json(wire: &AssetWire) -> String {
    let dto = AssetPkgWire::from_wire(wire);
    let bytes = encode_asset_pkg_json(&dto).expect("encode dto");
    let text = String::from_utf8(bytes).expect("dto utf8");
    assert_dto_json(&text);
    text
}

#[tokio::test]
async fn test_import_tampered_sig_rejected() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let created = create_wallet(&env, "Import Security Wallet").await;
    let session = unlock_wallet(&env, &created.wallet_id.0).await;

    let wire = AssetWire::from_asset(
        &z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 1, 10)
            .expect("valid std asset"),
    );
    let asset_data = dto_json(&wire);

    let mut root: serde_json::Value = serde_json::from_str(&asset_data).expect("json parse");
    let sig = root
        .get_mut("owner_signature")
        .expect("owner_signature present");
    let text = sig.as_str().expect("owner_signature must be hex string");
    let mut bad = text.to_string();
    tamper_hex(&mut bad);
    *sig = serde_json::Value::String(bad);

    let tampered = serde_json::to_string(&root).expect("json encode");
    let err = env
        .transport
        .call(
            "wallet.asset.import_asset",
            json!({"session": session, "asset_data": tampered}),
        )
        .await
        .expect_err("tampered signature import must fail");

    match err {
        RpcError::InvalidParams(message) => {
            assert_eq!(message, "IMPORT_CRYPTO_VERIFY_FAILED");
        }
        other => panic!("expected InvalidParams, got {other:?}"),
    }
}

#[tokio::test]
async fn test_invalid_range_proof_rejected() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let created = create_wallet(&env, "Import Security Wallet").await;
    let session = unlock_wallet(&env, &created.wallet_id.0).await;

    let mut wire = AssetWire::from_asset(
        &z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 1, 10)
            .expect("valid std asset"),
    );
    let proof = wire.range_proof.as_mut().expect("range_proof present");
    proof[0] ^= 1;

    let err = env
        .transport
        .call(
            "wallet.asset.import_asset",
            json!({"session": session, "asset_data": dto_json(&wire)}),
        )
        .await
        .expect_err("invalid range proof import must fail");

    match err {
        RpcError::InvalidParams(message) => {
            assert_eq!(message, "IMPORT_CRYPTO_VERIFY_FAILED");
        }
        other => panic!("expected InvalidParams, got {other:?}"),
    }
}

#[tokio::test]
async fn test_import_partial_stealth() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let created = create_wallet(&env, "Import Security Wallet").await;
    let session = unlock_wallet(&env, &created.wallet_id.0).await;

    let mut wire = AssetWire::from_asset(
        &z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 1, 10)
            .expect("valid std asset"),
    );
    wire.r_pub = Some([21u8; 32]);
    wire.owner_tag = None;
    wire.enc_pack = None;

    let err = env
        .transport
        .call(
            "wallet.asset.import_asset",
            json!({"session": session, "asset_data": dto_json(&wire)}),
        )
        .await
        .expect_err("partial stealth must fail");

    match err {
        RpcError::InvalidParams(message) => {
            assert_eq!(message, "IMPORT_STEALTH_INCONSISTENT");
        }
        other => panic!("expected InvalidParams, got {other:?}"),
    }
}

#[tokio::test]
async fn test_import_non_stealth_ok() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let created = create_wallet(&env, "Import Security Wallet").await;
    let session = unlock_wallet(&env, &created.wallet_id.0).await;

    let wire = AssetWire::from_asset(
        &z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 1, 10)
            .expect("valid std asset"),
    );

    let asset = wire
        .clone()
        .to_asset()
        .expect("non-stealth wire must be valid");
    assert!(asset.validate_stealth_consistency().is_ok());

    let err = env
        .transport
        .call(
            "wallet.asset.import_asset",
            json!({"session": session, "asset_data": dto_json(&wire)}),
        )
        .await
        .expect_err("non-owned transparent asset must fail ownership check");

    match err {
        RpcError::InvalidParams(message) => {
            assert_eq!(message, "IMPORT_OWNER_MISMATCH");
        }
        other => panic!("expected InvalidParams, got {other:?}"),
    }
}

#[tokio::test]
async fn test_finalize_fail_quarantine() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let created = create_wallet(&env, "Import Security Wallet").await;
    let session = unlock_wallet(&env, &created.wallet_id.0).await;

    let recv_keys = env
        .asset_rpc
        .test_receiver_keys(&created.wallet_id)
        .await
        .expect("receiver keys");
    let wire = mk_stealth_wire(1, &recv_keys);
    let asset_id = wire.clone().to_asset().expect("asset wire").asset_id();

    env.asset_rpc.set_finalize_fail(true);
    let err = env
        .transport
        .call(
            "wallet.asset.import_asset",
            json!({"session": session, "asset_data": dto_json(&wire)}),
        )
        .await
        .expect_err("finalize failure must reject import");

    match err {
        RpcError::RequestFailed(message) => {
            assert!(message.contains("IMPORT_CONSERVATION_VIOLATION"));
        }
        other => panic!("expected RequestFailed, got {other:?}"),
    }

    assert!(
        env.asset_rpc.has_claim_row(asset_id),
        "claim reservation row must not be released"
    );
    assert!(
        env.asset_rpc.has_stored_asset(asset_id).await,
        "asset must be written to storage before finalize fail"
    );
    assert!(
        env.asset_rpc
            .is_asset_quarantined(&created.wallet_id, asset_id)
            .await,
        "asset must be quarantined on finalize failure"
    );

    let bal_err = env
        .transport
        .call(
            "wallet.asset.get_asset_balance",
            json!({"wallet_id": created.wallet_id.0, "asset_id": asset_id}),
        )
        .await
        .expect_err("quarantined asset must be unavailable for balance");

    assert!(matches!(bal_err, RpcError::InvalidParams(_)));

    let spend_err = env
        .transport
        .call(
            "wallet.asset.send_asset",
            json!({
                "session": session,
                "asset_id": asset_id,
                "recipient": "bob",
                "amount": 1
            }),
        )
        .await
        .expect_err("quarantined asset must not be spendable");

    assert!(matches!(spend_err, RpcError::InvalidParams(_)));
}

#[tokio::test]
async fn test_no_store_reject() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env_no_store(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let created = create_wallet(&env, "No Store Wallet").await;
    let session = unlock_wallet(&env, &created.wallet_id.0).await;

    let recv_keys = env
        .asset_rpc
        .test_receiver_keys(&created.wallet_id)
        .await
        .expect("receiver keys");
    let wire = mk_stealth_wire(2, &recv_keys);

    let resp = env
        .transport
        .call(
            "wallet.asset.import_asset",
            json!({"session": session, "asset_data": dto_json(&wire)}),
        )
        .await
        .expect("wallet-native durable import should succeed without sidecar store");

    let inserted = resp
        .get("is_inserted")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    assert!(inserted, "expected first import to be inserted");
}

#[tokio::test]
async fn test_put_fail_release() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let created = create_wallet(&env, "Put Fail Wallet").await;
    let session = unlock_wallet(&env, &created.wallet_id.0).await;

    let recv_keys = env
        .asset_rpc
        .test_receiver_keys(&created.wallet_id)
        .await
        .expect("receiver keys");
    let wire = mk_stealth_wire(4, &recv_keys);
    let asset_id = wire.clone().to_asset().expect("asset wire").asset_id();

    std::env::set_var("Z00Z_FAIL_ASSET_SAVE", "1");
    let err = env
        .transport
        .call(
            "wallet.asset.import_asset",
            json!({"session": session, "asset_data": dto_json(&wire)}),
        )
        .await
        .expect_err("put failure must reject import");
    std::env::remove_var("Z00Z_FAIL_ASSET_SAVE");

    match err {
        RpcError::RequestFailed(message) => {
            assert!(message.contains("IMPORT_CONSERVATION_VIOLATION"));
        }
        other => panic!("expected RequestFailed, got {other:?}"),
    }

    assert!(
        !env.asset_rpc.is_claim_pending(&created.wallet_id, asset_id),
        "pending row must be released on put failure"
    );
    assert!(
        !env.asset_rpc.has_claim_row(asset_id),
        "final claim row must not exist on put failure"
    );
    assert!(
        !env.asset_rpc.has_stored_asset(asset_id).await,
        "asset must not remain stored on put failure"
    );
}

#[tokio::test]
async fn test_finalize_retry_ok() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let created = create_wallet(&env, "Retry Wallet").await;
    let session = unlock_wallet(&env, &created.wallet_id.0).await;

    let recv_keys = env
        .asset_rpc
        .test_receiver_keys(&created.wallet_id)
        .await
        .expect("receiver keys");
    let wire = mk_stealth_wire(3, &recv_keys);
    let asset_id = wire.clone().to_asset().expect("asset wire").asset_id();

    env.asset_rpc.set_finalize_fail(true);
    let _ = env
        .transport
        .call(
            "wallet.asset.import_asset",
            json!({"session": session.clone(), "asset_data": dto_json(&wire)}),
        )
        .await
        .expect_err("first import must fail on finalize");

    assert!(
        env.asset_rpc.is_claim_pending(&created.wallet_id, asset_id),
        "pending row expected after finalize fail"
    );

    env.asset_rpc.set_finalize_fail(false);
    let second = env
        .transport
        .call(
            "wallet.asset.import_asset",
            json!({"session": session, "asset_data": dto_json(&wire)}),
        )
        .await
        .expect("retry import must succeed");

    assert_eq!(
        second
            .get("asset_already_exists")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert!(
        !env.asset_rpc.is_claim_pending(&created.wallet_id, asset_id),
        "pending row must be finalized on retry"
    );
}

#[tokio::test]
async fn test_rejects_cash_asset_payload() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let created = create_wallet(&env, "Inventory Authority Wallet").await;
    let session = unlock_wallet(&env, &created.wallet_id.0).await;

    let recv_keys = env
        .asset_rpc
        .test_receiver_keys(&created.wallet_id)
        .await
        .expect("receiver keys");
    let wire = mk_stealth_wire(5, &recv_keys);
    let asset_id = wire.clone().to_asset().expect("asset wire").asset_id();

    env.transport
        .call(
            "wallet.asset.import_asset",
            json!({"session": session, "asset_data": dto_json(&wire)}),
        )
        .await
        .expect("owned import must succeed");
    env.transport
        .call("wallet.session.lock_wallet", json!({"session": session}))
        .await
        .expect("lock wallet");

    let store = object_inventory_store();
    let wallet_session = open_wallet_session(&env.output_dir, &created.wallet_id);

    assert!(
        store
            .lookup_non_asset_object(&wallet_session, &asset_id)
            .expect("lookup must succeed")
            .is_none(),
        "cash asset id must not resolve through voucher/right object storage",
    );

    let assets = store
        .list_wallet_inventory(
            &wallet_session,
            ObjectInventoryFilter {
                family: Some(OwnedObjectFamily::Asset),
                ..ObjectInventoryFilter::default()
            },
            None,
            usize::MAX,
        )
        .expect("asset projection");
    assert_eq!(assets.items.len(), 1);
    assert!(matches!(
        assets.items[0].payload,
        OwnedObjectPayload::Asset(_)
    ));

    let vouchers = store
        .list_voucher_claims(&wallet_session, None, None, usize::MAX)
        .expect("voucher list");
    assert!(
        vouchers.is_empty(),
        "cash import must not create voucher inventory rows",
    );

    let rights = store
        .list_right_inventory(&wallet_session, None, None, usize::MAX)
        .expect("right list");
    assert!(
        rights.is_empty(),
        "cash import must not create right inventory rows",
    );
}

#[tokio::test]
async fn test_rejects_cash_inventory_write() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let created = create_wallet(&env, "Object Rpc Boundary Wallet").await;
    let session = unlock_wallet(&env, &created.wallet_id.0).await;

    let recv_keys = env
        .asset_rpc
        .test_receiver_keys(&created.wallet_id)
        .await
        .expect("receiver keys");
    let wire = mk_stealth_wire(6, &recv_keys);
    let asset_id = wire.clone().to_asset().expect("asset wire").asset_id();

    let imported = env
        .transport
        .call(
            "wallet.asset.import_asset",
            json!({"session": session, "asset_data": dto_json(&wire)}),
        )
        .await
        .expect("owned import must succeed");
    assert_eq!(
        imported
            .get("is_inserted")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let objects = env
        .transport
        .call(
            "wallet.object.list_objects",
            json!({"wallet_id": created.wallet_id.0, "limit": 10, "cursor": null, "filter": null}),
        )
        .await
        .expect("object inventory");
    let object_items = objects
        .get("items")
        .and_then(serde_json::Value::as_array)
        .expect("object items");
    let asset_id_hex = hex::encode(asset_id);
    assert_eq!(object_items.len(), 1);
    assert_eq!(
        object_items[0]
            .get("family")
            .and_then(serde_json::Value::as_str),
        Some("asset")
    );
    assert_eq!(
        object_items[0]
            .get("stable_id_hex")
            .and_then(serde_json::Value::as_str),
        Some(asset_id_hex.as_str())
    );
    assert!(object_items[0].get("asset").is_some());
    assert!(object_items[0]
        .get("voucher")
        .unwrap_or(&serde_json::Value::Null)
        .is_null());
    assert!(object_items[0]
        .get("right")
        .unwrap_or(&serde_json::Value::Null)
        .is_null());

    let vouchers = env
        .transport
        .call(
            "wallet.object.list_vouchers",
            json!({"wallet_id": created.wallet_id.0, "limit": 10, "cursor": null, "status": null}),
        )
        .await
        .expect("voucher inventory");
    assert!(
        vouchers
            .get("items")
            .and_then(serde_json::Value::as_array)
            .expect("voucher items")
            .is_empty(),
        "cash import must not appear in wallet.object.list_vouchers",
    );

    let rights = env
        .transport
        .call(
            "wallet.object.list_rights",
            json!({"wallet_id": created.wallet_id.0, "limit": 10, "cursor": null, "status": null}),
        )
        .await
        .expect("right inventory");
    assert!(
        rights
            .get("items")
            .and_then(serde_json::Value::as_array)
            .expect("right items")
            .is_empty(),
        "cash import must not appear in wallet.object.list_rights",
    );

    env.transport
        .call(
            "wallet.asset.get_asset_balance",
            json!({"wallet_id": created.wallet_id.0, "asset_id": asset_id}),
        )
        .await
        .expect("cash asset must stay available on wallet.asset.*");
}
