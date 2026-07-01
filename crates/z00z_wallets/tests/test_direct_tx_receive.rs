#![cfg(not(target_arch = "wasm32"))]

use std::sync::{Arc, OnceLock};

use tokio::sync::Mutex;
use z00z_utils::codec::json;

use z00z_core::assets::{AssetClass, AssetPkgWire, AssetWire};
use z00z_crypto::{create_range_proof, Z00ZScalar};
use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcTransport};
use z00z_wallets::key::ReceiverKeys;
use z00z_wallets::rpc::methods::{
    AppRpcImpl, AssetRpcImpl, BackupRpcImpl, ChainRpcImpl, ChainScanRpcImpl, KeyRpcImpl,
    NetworkRpcImpl, StorageRpcImpl, TxRpcImpl, WalletRpcImpl,
};
use z00z_wallets::rpc::register_all_wallet_rpc_methods;
use z00z_wallets::rpc::types::tx::{
    RuntimeTxErrorCode, RuntimeTxLifecycle, RuntimeVerifyTxPkgResponse,
};
use z00z_wallets::rpc::types::wallet::{RuntimeCreateWalletResponse, SessionToken};
use z00z_wallets::services::{AppService, WalletService};
use z00z_wallets::tx::fee_estimator::calculate_fee_for_wires;
use z00z_wallets::tx::{
    build_tx_package_digest, TxAuthWire, TxContextWire, TxInputWire, TxOutRole, TxOutputWire,
    TxPackage, TxProofWire, TxWire,
};
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
    let mut asset =
        z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 0, 100_000)
            .expect("valid std asset");
    asset.nonce = [serial_id as u8; 32];

    let card = mk_recv_card(keys);
    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut SenderWallet::new([87u8; 32]),
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

fn mk_tx_pkg(wire: &AssetWire, status: &str) -> String {
    let fee_seed = z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 9, 1)
        .expect("fee seed");
    let fee_seed = AssetWire::from_asset(&fee_seed);
    let fee = calculate_fee_for_wires(1, &[wire.clone(), fee_seed]).expect("fee");
    let fee_asset = z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 9, fee)
        .expect("fee asset");
    let fee_wire = AssetWire::from_asset(&fee_asset);
    let tx = TxWire {
        tx_type: "regular_tx".to_string(),
        inputs: vec![TxInputWire {
            asset_id_hex: hex::encode([3u8; 32]),
            serial_id: 1,
        }],
        outputs: vec![
            TxOutputWire {
                role: TxOutRole::Recipient,
                asset_wire: AssetPkgWire::from_wire(wire),
            },
            TxOutputWire {
                role: TxOutRole::Fee,
                asset_wire: AssetPkgWire::from_wire(&fee_wire),
            },
        ],
        fee,
        nonce: 1,
        context: TxContextWire::default(),
        proof: TxProofWire::default(),
        auth: TxAuthWire::default(),
    };
    let digest = build_tx_package_digest(
        "TxPackage",
        "regular_tx",
        1,
        3,
        "devnet",
        "z00z-devnet-1",
        &tx,
    )
    .expect("digest");
    let pkg = TxPackage {
        kind: "TxPackage".to_string(),
        package_type: "regular_tx".to_string(),
        version: 1,
        chain_id: 3,
        chain_type: "devnet".to_string(),
        chain_name: "z00z-devnet-1".to_string(),
        tx,
        tx_digest_hex: digest,
        status: status.to_string(),
    };
    serde_json::to_string(&pkg).expect("pkg json")
}

async fn verify_pkg(
    env: &TestEnv,
    session: SessionToken,
    tx_data: &str,
) -> RuntimeVerifyTxPkgResponse {
    let value = env
        .transport
        .call(
            "wallet.tx.verify_transaction_package",
            json!({"session": session, "tx_data": tx_data}),
        )
        .await
        .expect("verify_transaction_package must succeed");

    serde_json::from_value(value).expect("RuntimeVerifyTxPkgResponse")
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
async fn test_verify_package_pre_import() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let bob = create_wallet(&env, "Bob Verify Wallet").await;
    let bob_session = unlock_wallet(&env, &bob.wallet_id.0).await;
    let recv_keys = env
        .asset_rpc
        .test_receiver_keys(&bob.wallet_id)
        .await
        .expect("receiver keys");

    let wire = mk_stealth_wire(7, &recv_keys);
    let pkg_prepared = mk_tx_pkg(&wire, "prepared");
    let prepared = verify_pkg(&env, bob_session.clone(), &pkg_prepared).await;

    assert!(!prepared.is_valid);
    assert_eq!(prepared.lifecycle, RuntimeTxLifecycle::Failed);
    assert!(!prepared.import_ready);
    assert_eq!(
        prepared.error_codes,
        vec![RuntimeTxErrorCode::InvalidPublicSpendProof]
    );
    assert!(!prepared.all_owned_spendable);
    assert!(prepared.owned_outputs.is_empty());
    assert!(
        prepared
            .errors
            .iter()
            .any(|error| error.contains("public spend contract failed: missing spend proof")),
        "unexpected prepared errors: {:?}",
        prepared.errors
    );

    assert!(list_items(&env, &bob.wallet_id.0).await.is_empty());

    let pkg_confirmed = mk_tx_pkg(&wire, "confirmed");
    let confirmed = verify_pkg(&env, bob_session.clone(), &pkg_confirmed).await;

    assert!(!confirmed.is_valid);
    assert_eq!(confirmed.lifecycle, RuntimeTxLifecycle::Failed);
    assert!(!confirmed.import_ready);
    assert_eq!(
        confirmed.error_codes,
        vec![RuntimeTxErrorCode::InvalidPublicSpendProof]
    );
    assert!(!confirmed.all_owned_spendable);
    assert!(confirmed.owned_outputs.is_empty());
    assert!(
        confirmed
            .errors
            .iter()
            .any(|error| error.contains("public spend contract failed: missing spend proof")),
        "unexpected confirmed errors: {:?}",
        confirmed.errors
    );
    assert!(list_items(&env, &bob.wallet_id.0).await.is_empty());
}
