#![cfg(not(target_arch = "wasm32"))]

use std::sync::{Arc, OnceLock};

use tokio::sync::Mutex;
use z00z_simulator::{
    scenario_1::claim_pkg_consumer::{load_claim_leaves, wrap_claim_packages},
    scenario_1::stage_3::{
        build_claim_package, patch_claim_bundle_membership, write_claim_bundle_store,
    },
};
use z00z_utils::{
    codec::{json, Codec},
    io::save_json,
};

use crate::claim_pkg_crypto::patch_claim_crypto;
use z00z_core::{
    assets::{decode_asset_pkg_json, encode_asset_pkg_json, AssetClass, AssetPkgWire},
    AssetWire,
};
use z00z_crypto::{create_range_proof, Z00ZScalar};
use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcTransport};
use z00z_wallets::key::ReceiverKeys;
use z00z_wallets::receiver::{ReceiverCard, ScanResult, StealthOutputScanner};
use z00z_wallets::rpc::methods::{
    AppRpcImpl, AssetRpcImpl, BackupRpcImpl, ChainRpcImpl, ChainScanRpcImpl, KeyRpcImpl,
    NetworkRpcImpl, StorageRpcImpl, TxRpcImpl, WalletRpcImpl,
};
use z00z_wallets::rpc::register_all_wallet_rpc_methods;
use z00z_wallets::rpc::types::wallet::{RuntimeCreateWalletResponse, SessionToken};
use z00z_wallets::services::{AppService, WalletService};
use z00z_wallets::stealth::{build_tx_output_unchecked, SenderWallet};
use z00z_wallets::tx::derive_output_nonce;

const PASS: &str = "CorrectPassw0rd!";
const SEED24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const CHAIN_ID: u32 = 3;
const CHAIN_TYPE: &str = "devnet";
const CHAIN_NAME: &str = "z00z-devnet-1";

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
            json!({"name": name, "password": PASS, "seed_phrase": SEED24}),
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
            json!({"wallet_id": wallet_id, "password": PASS}),
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

fn rebuild_def(
    definition: &z00z_core::AssetDefinition,
    serial_id: u32,
) -> z00z_core::AssetDefinition {
    z00z_core::AssetDefinition::new(
        [0u8; 32],
        definition.class,
        format!("{}-{serial_id}", definition.name),
        definition.symbol.clone(),
        definition.decimals,
        definition.serials,
        definition.nominal,
        definition.domain_name.clone(),
        definition.version,
        definition.crypto_version,
        definition.policy_flags,
        definition.metadata.clone(),
    )
    .expect("canonical test definition")
}

fn mk_stealth_wire(serial_id: u32, keys: &ReceiverKeys) -> AssetWire {
    let mut asset = z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 0, 10)
        .expect("valid std asset");
    let def = rebuild_def(asset.definition.as_ref(), serial_id);
    asset.definition = Arc::new(def);

    let card = mk_recv_card(keys);
    let tx_seed = derive_output_nonce(&asset.definition.id, asset.serial_id);
    let mut sender_wallet = SenderWallet::new([41u8; 32]);
    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut sender_wallet,
        &tx_seed,
        0,
        asset.amount,
        &asset.definition.id,
    )
    .expect("stealth output");

    let commitment = z00z_crypto::Commitment::from_bytes(&output.c_amount).expect("commitment");
    asset.commitment = commitment.as_commitment().clone();
    asset.owner_pub = None;
    asset.owner_signature = None;
    asset.r_pub = Some(output.r_pub);
    asset.owner_tag = Some(output.owner_tag);
    asset.enc_pack = Some(output.enc_pack);
    asset.tag16 = output.tag16;
    asset.leaf_ad_id = Some(asset.definition.id);

    let scanner = StealthOutputScanner::from_keys(keys);
    let ScanResult::Mine { wallet_output } = scanner.scan_leaf(&asset) else {
        panic!("owned leaf")
    };
    let blinding = Z00ZScalar::try_from_bytes(wallet_output.blinding.expect("blinding bytes"))
        .expect("blinding scalar");
    asset.range_proof = Some(create_range_proof(asset.amount, &blinding, 64, 0).expect("proof"));

    let mut wire = AssetWire::from_asset(&asset);
    wire.secret = None;
    wire
}

fn make_pkg(serial_id: u32, keys: &ReceiverKeys) -> z00z_wallets::tx::ClaimTxPackage {
    let wire = mk_stealth_wire(serial_id, keys);
    let claim_id = derive_output_nonce(&wire.definition.id, wire.serial_id);
    let leaf = z00z_wallets::tx::asset_wire_to_leaf(&wire).expect("claim leaf");
    let pkg_bytes = build_claim_package(
        CHAIN_ID,
        CHAIN_TYPE,
        CHAIN_NAME,
        "wallet-under-test",
        &hex::encode(leaf.asset_id),
        wire.amount,
        &claim_id,
        &keys.owner_handle,
        serial_id as u64,
        Some(wire),
        Some(keys),
    )
    .expect("build claim package");
    let mut pkg = z00z_utils::codec::JsonCodec
        .deserialize(&pkg_bytes)
        .expect("decode claim package");
    patch_claim_crypto(&mut pkg);
    pkg
}

fn assert_dto_json(asset_data: &str) {
    let value: serde_json::Value = serde_json::from_str(asset_data).expect("dto json");
    assert!(value.get("secret").is_none());
    assert!(value.get("is_frozen").is_none());
    assert!(value.get("is_slashed").is_none());
    assert!(decode_asset_pkg_json(asset_data.as_bytes()).is_ok());
}

fn dto_json(wire: &AssetWire) -> String {
    let dto = AssetPkgWire::from_wire(wire);
    let bytes = encode_asset_pkg_json(&dto).expect("dto json bytes");
    let text = String::from_utf8(bytes).expect("dto utf8");
    assert_dto_json(&text);
    text
}

#[tokio::test]
async fn test_claim_pkg_import() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let created = create_wallet(&env, "Claim Import Wallet").await;
    let session = unlock_wallet(&env, &created.wallet_id.0).await;
    let recv_keys = env
        .asset_rpc
        .test_receiver_keys(&created.wallet_id)
        .await
        .expect("receiver keys");

    let pkg_path = dir.path().join("tx_claim_pkg.json");
    let mut packages = vec![make_pkg(11, &recv_keys), make_pkg(12, &recv_keys)];
    patch_claim_bundle_membership(&mut packages).expect("patch bundle membership");
    write_claim_bundle_store(dir.path(), &packages).expect("write claim store");
    save_json(&pkg_path, &wrap_claim_packages(packages)).expect("write claim package file");

    let leafs = load_claim_leaves(&pkg_path).expect("load claim leaves");
    assert_eq!(leafs.len(), 2);

    for leaf in &leafs {
        let resp = env
            .transport
            .call(
                "wallet.asset.import_asset",
                json!({
                    "session": session,
                    "asset_data": dto_json(leaf),
                }),
            )
            .await
            .expect("claim leaf import must succeed");

        let inserted = resp
            .get("is_inserted")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        assert!(inserted, "expected first import to insert asset");

        let asset_id = leaf.clone().to_asset().expect("leaf to asset").asset_id();
        assert!(env.asset_rpc.has_claim_row(asset_id));
        assert!(env.asset_rpc.has_stored_asset(asset_id).await);
    }

    assert_eq!(env.asset_rpc.verify_complete_count(), 2);
}
