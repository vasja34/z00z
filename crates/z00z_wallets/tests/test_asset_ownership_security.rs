#![cfg(not(target_arch = "wasm32"))]

// Real theft-resistance boundary: these tests pin wallet-local import
// ownership rules and do not claim a validator-level trustless ownership
// theorem. Withholding before publication and validator-facing anti-theft
// closure remain separate open risks.

use std::sync::{Arc, OnceLock};

use tokio::sync::Mutex;
use z00z_utils::codec::json;

use z00z_core::assets::{encode_asset_pkg_json, AssetClass, AssetPkgWire, AssetWire};
use z00z_crypto::{create_range_proof, Z00ZScalar};
use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcError, RpcTransport};
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
const WITNESS_GATE_SRC: &str = include_str!("../src/tx/witness_gate.rs");
const SPEND_VERIFICATION_SRC: &str = include_str!("../src/tx/spend_verification.rs");
const SCENARIO1_SEMANTICS_SRC: &str = include_str!("test_scenario1_semantics.rs");

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

fn mk_wire(serial_id: u32) -> AssetWire {
    let mut wire = AssetWire::from_asset(
        &z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, serial_id, 10)
            .expect("valid std asset"),
    );
    wire.secret = None;
    wire
}

fn mk_receiver_card(keys: &ReceiverKeys) -> ReceiverCard {
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

#[test]
fn test_layered_theft_stays_narrow() {
    assert!(
        WITNESS_GATE_SRC.contains("does not close withholding risk before")
            && WITNESS_GATE_SRC.contains("prove public anti-theft closure"),
        "witness gate must keep withholding and public anti-theft closure separate"
    );
    assert!(
        SPEND_VERIFICATION_SRC.contains("Real theft-resistance boundary")
            && SPEND_VERIFICATION_SRC.contains("receiver-secret")
            && SPEND_VERIFICATION_SRC.contains("plus `s_out`")
            && SPEND_VERIFICATION_SRC.contains("wallet-local post-scan exclusivity gate")
            && SPEND_VERIFICATION_SRC.contains("validator-facing")
            && SPEND_VERIFICATION_SRC.contains("public trustless theorem"),
        "public spend verification must stay narrower than the wallet-local ownership theorem"
    );
    assert!(
        SCENARIO1_SEMANTICS_SRC.contains("Real theft-resistance boundary stays wallet-local here")
            && SCENARIO1_SEMANTICS_SRC.contains("public verifier")
            && SCENARIO1_SEMANTICS_SRC.contains("remains narrower than that theorem"),
        "scenario semantics must describe the anti-theft boundary as compositional"
    );
}

#[test]
fn test_sender_ignorance_stays_forbidden() {
    assert!(
        WITNESS_GATE_SRC.contains("receiver-secret-gated wallet-local seam")
            && SPEND_VERIFICATION_SRC.contains("receiver-secret")
            && SPEND_VERIFICATION_SRC.contains("plus `s_out`")
            && SPEND_VERIFICATION_SRC.contains("wallet-local post-scan exclusivity gate")
            && SCENARIO1_SEMANTICS_SRC
                .contains("receiver-secret-gated wallet-local ownership rule"),
        "wallet-local anti-theft wording must stay receiver-secret-gated"
    );
    assert!(
        !WITNESS_GATE_SRC.contains("sender ignorance")
            && !SPEND_VERIFICATION_SRC.contains("sender ignorance")
            && !SCENARIO1_SEMANTICS_SRC.contains("sender ignorance"),
        "sender-ignorance wording must stay banned from the honest anti-theft boundary"
    );
}

fn mk_stealth_wire(serial_id: u32, keys: &ReceiverKeys) -> AssetWire {
    let mut asset = z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 0, 10)
        .expect("valid std asset");

    let card = mk_receiver_card(keys);
    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut SenderWallet::new([31u8; 32]),
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

fn flip_sig(node: &mut serde_json::Value) -> bool {
    match node {
        serde_json::Value::Array(items) => {
            for item in items {
                if flip_sig(item) {
                    return true;
                }
            }
            false
        }
        serde_json::Value::Object(map) => {
            for value in map.values_mut() {
                if flip_sig(value) {
                    return true;
                }
            }
            false
        }
        serde_json::Value::Number(num) => {
            if let Some(val) = num.as_u64() {
                *node = serde_json::Value::from((val ^ 1) & 0xff);
                return true;
            }
            false
        }
        serde_json::Value::String(text) => {
            if text.is_empty() {
                return false;
            }
            let mut bytes = text.as_bytes().to_vec();
            bytes[0] = if bytes[0] == b'a' { b'b' } else { b'a' };
            *text = String::from_utf8_lossy(&bytes).to_string();
            true
        }
        _ => false,
    }
}

async fn import_wire(
    env: &TestEnv,
    session: SessionToken,
    wire: &AssetWire,
) -> Result<(), RpcError> {
    env.transport
        .call(
            "wallet.asset.import_asset",
            json!({
                "session": session,
                "asset_data": dto_json(wire)
            }),
        )
        .await
        .map(|_| ())
}

fn dto_json(wire: &AssetWire) -> String {
    let dto = AssetPkgWire::from_wire(wire);
    let bytes = encode_asset_pkg_json(&dto).expect("encode dto");
    String::from_utf8(bytes).expect("dto utf8")
}

#[tokio::test]
async fn test_import_own_asset() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let created = create_wallet(&env, "Own Wallet").await;
    let session = unlock_wallet(&env, &created.wallet_id.0).await;

    let recv_keys = env
        .asset_rpc
        .test_receiver_keys(&created.wallet_id)
        .await
        .expect("receiver keys");
    let wire = mk_stealth_wire(1, &recv_keys);

    let res = import_wire(&env, session, &wire).await;
    assert!(res.is_ok(), "own asset must import");
}

#[tokio::test]
async fn test_import_wrong_wallet() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let _a = create_wallet(&env, "Wallet A").await;
    let b = create_wallet(&env, "Wallet B").await;
    let b_sess = unlock_wallet(&env, &b.wallet_id.0).await;

    let wire = mk_wire(2);

    let err = import_wire(&env, b_sess, &wire)
        .await
        .expect_err("wrong wallet import must fail");

    match err {
        RpcError::InvalidParams(message) => {
            assert_eq!(message, "IMPORT_OWNER_MISMATCH");
        }
        other => panic!("expected InvalidParams, got {other:?}"),
    }
}

#[tokio::test]
async fn test_import_no_owner_field() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let created = create_wallet(&env, "Wallet").await;
    let sess = unlock_wallet(&env, &created.wallet_id.0).await;

    let mut wire = mk_wire(3);
    wire.owner_pub = None;
    wire.owner_signature = None;
    wire.r_pub = None;
    wire.owner_tag = None;
    wire.enc_pack = None;
    wire.tag16 = None;

    let err = import_wire(&env, sess, &wire)
        .await
        .expect_err("missing owner fields must fail");

    match err {
        RpcError::InvalidParams(message) => {
            assert_eq!(message, "IMPORT_OWNER_MISMATCH");
        }
        other => panic!("expected InvalidParams, got {other:?}"),
    }
}

#[tokio::test]
async fn test_import_transparent_unsigned_rejected() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let created = create_wallet(&env, "Wallet").await;
    let sess = unlock_wallet(&env, &created.wallet_id.0).await;

    let mut wire = mk_wire(4);
    wire.owner_signature = None;

    let err = import_wire(&env, sess, &wire)
        .await
        .expect_err("unsigned transparent must fail");

    match err {
        RpcError::InvalidParams(message) => {
            assert_eq!(message, "IMPORT_OWNER_MISMATCH");
        }
        other => panic!("expected InvalidParams, got {other:?}"),
    }
}

#[tokio::test]
async fn test_transparent_bad_sig_rejected() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let created = create_wallet(&env, "Wallet").await;
    let sess = unlock_wallet(&env, &created.wallet_id.0).await;

    let wire = mk_wire(5);
    let asset_data = dto_json(&wire);
    let mut root: serde_json::Value = serde_json::from_str(&asset_data).expect("asset dto json");
    let sig = root
        .get_mut("owner_signature")
        .expect("owner signature present");
    assert!(flip_sig(sig), "must tamper signature bytes");
    let tampered = serde_json::to_string(&root).expect("json encode");

    let err = env
        .transport
        .call(
            "wallet.asset.import_asset",
            json!({
                "session": sess,
                "asset_data": tampered,
            }),
        )
        .await
        .map(|_| ())
        .expect_err("bad signature must fail");

    match err {
        RpcError::InvalidParams(message) => {
            assert_eq!(message, "IMPORT_CRYPTO_VERIFY_FAILED");
        }
        other => panic!("expected InvalidParams, got {other:?}"),
    }
}

#[tokio::test]
async fn test_stealth_import_not_mine() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let source_wallet = create_wallet(&env, "Wallet A").await;
    let target_wallet = create_wallet(&env, "Wallet B").await;
    let _source_sess = unlock_wallet(&env, &source_wallet.wallet_id.0).await;
    let target_sess = unlock_wallet(&env, &target_wallet.wallet_id.0).await;

    let source_keys = env
        .asset_rpc
        .test_receiver_keys(&source_wallet.wallet_id)
        .await
        .expect("source receiver keys");
    let wire = mk_stealth_wire(6, &source_keys);

    let err = import_wire(&env, target_sess, &wire)
        .await
        .expect_err("stealth not mine must fail");

    match err {
        RpcError::InvalidParams(message) => {
            assert_eq!(message, "IMPORT_STEALTH_INCONSISTENT");
        }
        other => panic!("expected InvalidParams, got {other:?}"),
    }
}

#[tokio::test]
async fn test_stealth_import_is_mine() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let wallet = create_wallet(&env, "Wallet A").await;
    let wallet_sess = unlock_wallet(&env, &wallet.wallet_id.0).await;

    let recv_keys = env
        .asset_rpc
        .test_receiver_keys(&wallet.wallet_id)
        .await
        .expect("receiver keys");
    let wire = mk_stealth_wire(7, &recv_keys);

    let res = import_wire(&env, wallet_sess, &wire).await;
    assert!(res.is_ok(), "stealth mine must import");
}

#[tokio::test]
async fn test_wallet_ownership_per_asset() {
    let _guard = test_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let source_wallet = create_wallet(&env, "Wallet A").await;
    let target_wallet = create_wallet(&env, "Wallet B").await;
    let _source_sess = unlock_wallet(&env, &source_wallet.wallet_id.0).await;
    let target_sess = unlock_wallet(&env, &target_wallet.wallet_id.0).await;

    let target_keys = env
        .asset_rpc
        .test_receiver_keys(&target_wallet.wallet_id)
        .await
        .expect("target receiver keys");
    let mine = mk_stealth_wire(8, &target_keys);
    let first = import_wire(&env, target_sess.clone(), &mine).await;
    assert!(first.is_ok(), "first owned import must pass");

    let source_keys = env
        .asset_rpc
        .test_receiver_keys(&source_wallet.wallet_id)
        .await
        .expect("source receiver keys");
    let not_mine = mk_stealth_wire(9, &source_keys);
    let second = import_wire(&env, target_sess, &not_mine)
        .await
        .expect_err("second not mine must fail");

    match second {
        RpcError::InvalidParams(message) => {
            assert_eq!(message, "IMPORT_STEALTH_INCONSISTENT");
        }
        other => panic!("expected InvalidParams, got {other:?}"),
    }
}
