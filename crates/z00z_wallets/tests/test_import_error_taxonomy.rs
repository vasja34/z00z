#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;
use std::sync::{Arc, LazyLock};

use tokio::sync::Mutex;
use z00z_utils::codec::json;

use z00z_core::assets::{encode_asset_pkg_json, AssetClass, AssetPkgWire, AssetWire};
use z00z_crypto::{create_range_proof, Z00ZScalar};
use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcError, RpcTransport};
use z00z_wallets::claim::registry::{self as claim_registry, GlobalClaimRegistry};
use z00z_wallets::claim::{claim_scope_hash, sign_claim_receipt, ClaimReceipt};
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

const PASS: &str = "CorrectPassw0rd!";
const SEED24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

static TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

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

fn err_code(err: RpcError) -> String {
    match err {
        RpcError::InvalidParams(message) => parse_import_code(&message),
        RpcError::RequestFailed(message)
        | RpcError::Internal(message)
        | RpcError::InvalidResponse(message)
        | RpcError::TransportError(message)
        | RpcError::MethodNotFound(message)
        | RpcError::WalletNotFound(message)
        | RpcError::RateLimited(message) => parse_import_code(&message),
        RpcError::AuthFailed | RpcError::SessionExpired | RpcError::SessionInvalid => {
            "AUTH_FAILED".to_string()
        }
        RpcError::WalletLocked => "WALLET_LOCKED".to_string(),
    }
}

fn parse_import_code(message: &str) -> String {
    let Some(start) = message.find("IMPORT_") else {
        return message.to_string();
    };

    let tail = &message[start..];
    let end = tail
        .find(|ch: char| !(ch.is_ascii_uppercase() || ch == '_'))
        .unwrap_or(tail.len());
    tail[..end].to_string()
}

async fn call_import(
    env: &TestEnv,
    session: &SessionToken,
    asset_data: String,
) -> Result<serde_json::Value, RpcError> {
    env.transport
        .call(
            "wallet.asset.import_asset",
            json!({
                "session": session,
                "asset_data": asset_data,
            }),
        )
        .await
}

async fn call_import_wire(
    env: &TestEnv,
    session: &SessionToken,
    wire: &AssetWire,
) -> Result<serde_json::Value, RpcError> {
    call_import(env, session, dto_json(wire)).await
}

fn dto_json(wire: &AssetWire) -> String {
    let dto = AssetPkgWire::from_wire(wire);
    let bytes = encode_asset_pkg_json(&dto).expect("encode dto");
    String::from_utf8(bytes).expect("dto utf8")
}

fn dto_json_with_secret(wire: &AssetWire, secret: [u8; 32]) -> String {
    let mut root: serde_json::Value = serde_json::from_str(&dto_json(wire)).expect("dto parse");
    let object = root.as_object_mut().expect("dto object");
    object.insert("secret".to_string(), serde_json::json!(hex::encode(secret)));
    serde_json::to_string(&root).expect("dto encode with secret")
}

fn dto_json_null_secret(wire: &AssetWire) -> String {
    let mut root: serde_json::Value = serde_json::from_str(&dto_json(wire)).expect("dto parse");
    let object = root.as_object_mut().expect("dto object");
    object.insert("secret".to_string(), serde_json::Value::Null);
    serde_json::to_string(&root).expect("dto encode with null secret")
}

async fn case_session_code(env: &TestEnv, session: &SessionToken) -> String {
    let mut bad_session = session.clone();
    bad_session.token = format!("{}-bad", bad_session.token);
    let err = call_import_wire(
        env,
        &bad_session,
        &AssetWire::from_asset(
            &z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 1, 10)
                .expect("std asset"),
        ),
    )
    .await
    .expect_err("must fail");

    err_code(err)
}

async fn case_crypto_code(env: &TestEnv, session: &SessionToken) -> String {
    let mut bad_proof = AssetWire::from_asset(
        &z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 2, 10)
            .expect("std asset"),
    );
    bad_proof.range_proof.as_mut().expect("proof")[0] ^= 1;
    let err = call_import_wire(env, session, &bad_proof)
        .await
        .expect_err("must fail");

    err_code(err)
}

async fn case_owner_code(env: &TestEnv, session: &SessionToken) -> String {
    let plain = AssetWire::from_asset(
        &z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 3, 10)
            .expect("std asset"),
    );
    let err = call_import_wire(env, session, &plain)
        .await
        .expect_err("must fail");

    err_code(err)
}

async fn case_stealth_code(env: &TestEnv, session: &SessionToken) -> String {
    let mut bad_stealth = AssetWire::from_asset(
        &z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 4, 10)
            .expect("std asset"),
    );
    bad_stealth.r_pub = Some([21u8; 32]);
    bad_stealth.owner_tag = None;
    bad_stealth.enc_pack = None;
    let err = call_import_wire(env, session, &bad_stealth)
        .await
        .expect_err("must fail");

    err_code(err)
}

async fn case_secret_code(env: &TestEnv, session: &SessionToken, keys: &ReceiverKeys) -> String {
    let bad_secret = mk_stealth_wire(10, keys);
    let err = call_import(env, session, dto_json_with_secret(&bad_secret, [9u8; 32]))
        .await
        .expect_err("must fail");

    err_code(err)
}

async fn case_conflict_code(env: &TestEnv, session: &SessionToken, keys: &ReceiverKeys) -> String {
    let conflict_wire = mk_stealth_wire(11, keys);
    let conflict_asset_id = conflict_wire
        .clone()
        .to_asset()
        .expect("conflict to_asset")
        .asset_id();
    let conflict_receipt = ClaimReceipt {
        schema_ver: 1,
        asset_id: conflict_asset_id,
        wallet_id: b"other_wallet".to_vec(),
        claim_scope: claim_scope_hash("dev-chain"),
        identity_pk: keys.identity_pk.to_bytes(),
    };
    let conflict_sig = sign_claim_receipt(keys, &conflict_receipt).expect("conflict sign");
    claim_registry::global_claim_registry()
        .reserve(
            conflict_asset_id,
            "other_wallet",
            &conflict_receipt,
            &conflict_sig,
        )
        .expect("reserve conflict row");

    let err = call_import_wire(env, session, &conflict_wire)
        .await
        .expect_err("reserved asset must fail with conflict");

    err_code(err)
}

async fn case_cons_code(env: &TestEnv, session: &SessionToken, keys: &ReceiverKeys) -> String {
    env.asset_rpc.set_finalize_fail(true);
    let cons_wire = mk_stealth_wire(12, keys);
    let err = call_import_wire(env, session, &cons_wire)
        .await
        .expect_err("finalize fail must reject");
    env.asset_rpc.set_finalize_fail(false);

    err_code(err)
}

async fn case_dup_code(env: &TestEnv, session: &SessionToken, keys: &ReceiverKeys) -> String {
    let dup_wire = mk_stealth_wire(13, keys);
    let first = call_import_wire(env, session, &dup_wire)
        .await
        .expect("first duplicate seed import must pass");
    let second = call_import_wire(env, session, &dup_wire)
        .await
        .expect("second import must be accepted as duplicate");
    let msg = second
        .get("message")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .to_string();

    assert!(first
        .get("success")
        .and_then(|value| value.as_bool())
        .unwrap_or(false));

    msg
}

async fn case_bad_json_code(env: &TestEnv, session: &SessionToken) -> String {
    let err = call_import(env, session, "not-json".to_string())
        .await
        .expect_err("must fail");

    err_code(err)
}

fn assert_import(code: &str) {
    assert!(code.starts_with("IMPORT_"));
}

fn add_code(classes: &mut BTreeSet<String>, code: String, want: &str) {
    assert_eq!(code, want);
    classes.insert(code);
}

#[tokio::test]
async fn test_error_codes_distinct() {
    let _guard = TEST_LOCK.lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let wa = create_wallet(&env, "tax_a").await;
    let sa = unlock_wallet(&env, &wa.wallet_id.0).await;

    let keys_a = env
        .asset_rpc
        .test_receiver_keys(&sa.wallet_id)
        .await
        .expect("receiver keys a");

    let mut classes = BTreeSet::<String>::new();

    add_code(
        &mut classes,
        case_session_code(&env, &sa).await,
        "IMPORT_SESSION_INVALID",
    );
    add_code(
        &mut classes,
        case_bad_json_code(&env, &sa).await,
        "IMPORT_MALFORMED_JSON",
    );
    add_code(
        &mut classes,
        case_crypto_code(&env, &sa).await,
        "IMPORT_CRYPTO_VERIFY_FAILED",
    );
    add_code(
        &mut classes,
        case_owner_code(&env, &sa).await,
        "IMPORT_OWNER_MISMATCH",
    );
    add_code(
        &mut classes,
        case_stealth_code(&env, &sa).await,
        "IMPORT_STEALTH_INCONSISTENT",
    );
    add_code(
        &mut classes,
        case_secret_code(&env, &sa, &keys_a).await,
        "IMPORT_SECRET_FIELD_FORBIDDEN",
    );
    add_code(
        &mut classes,
        case_conflict_code(&env, &sa, &keys_a).await,
        "IMPORT_CLAIM_CONFLICT",
    );
    add_code(
        &mut classes,
        case_cons_code(&env, &sa, &keys_a).await,
        "IMPORT_CONSERVATION_VIOLATION",
    );

    classes.insert(case_dup_code(&env, &sa, &keys_a).await);

    assert!(classes.iter().all(|code| !code.is_empty()));
    assert_eq!(classes.len(), 9, "must keep distinct classes: {classes:?}");
}

#[tokio::test]
async fn test_error_no_generic_collapse() {
    let _guard = TEST_LOCK.lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let wa = create_wallet(&env, "nogeneric_a").await;
    let sa = unlock_wallet(&env, &wa.wallet_id.0).await;

    let mut bad_proof = AssetWire::from_asset(
        &z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 11, 10)
            .expect("std asset"),
    );
    bad_proof.range_proof.as_mut().expect("proof")[0] ^= 1;

    let mut bad_stealth = AssetWire::from_asset(
        &z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 12, 10)
            .expect("std asset"),
    );
    bad_stealth.r_pub = Some([11u8; 32]);

    let bad_secret = AssetWire::from_asset(
        &z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 13, 10)
            .expect("std asset"),
    );

    let err_bad_json = case_bad_json_code(&env, &sa).await;

    let err_crypto = err_code(
        call_import_wire(&env, &sa, &bad_proof)
            .await
            .expect_err("must fail"),
    );
    let err_stealth = err_code(
        call_import_wire(&env, &sa, &bad_stealth)
            .await
            .expect_err("must fail"),
    );
    let err_secret = err_code(
        call_import(&env, &sa, dto_json_with_secret(&bad_secret, [1u8; 32]))
            .await
            .expect_err("must fail"),
    );

    assert_ne!(err_bad_json, err_crypto);
    assert_ne!(err_bad_json, err_stealth);
    assert_ne!(err_bad_json, err_secret);
    assert_eq!(err_bad_json, "IMPORT_MALFORMED_JSON");
    assert_ne!(err_crypto, err_stealth);
    assert_ne!(err_crypto, err_secret);
    assert_ne!(err_stealth, err_secret);
    assert_import(&err_bad_json);
    assert_import(&err_crypto);
    assert_import(&err_stealth);
    assert_import(&err_secret);
}

#[tokio::test]
async fn test_bad_json_code() {
    let _guard = TEST_LOCK.lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let wa = create_wallet(&env, "badjson_a").await;
    let sa = unlock_wallet(&env, &wa.wallet_id.0).await;

    let err = call_import(&env, &sa, "not-json".to_string())
        .await
        .expect_err("must fail");

    assert_eq!(err_code(err), "IMPORT_MALFORMED_JSON");
}

#[tokio::test]
async fn test_secret_null_code() {
    let _guard = TEST_LOCK.lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let env = setup_env(dir.path().join("wallets"));
    env.asset_rpc.clear_claim_rows();

    let wa = create_wallet(&env, "nullsecret_a").await;
    let sa = unlock_wallet(&env, &wa.wallet_id.0).await;
    let keys = env
        .asset_rpc
        .test_receiver_keys(&sa.wallet_id)
        .await
        .expect("receiver keys");

    let wire = mk_stealth_wire(24, &keys);
    let err = call_import(&env, &sa, dto_json_null_secret(&wire))
        .await
        .expect_err("must fail");

    assert_eq!(err_code(err), "IMPORT_SECRET_FIELD_FORBIDDEN");
}
