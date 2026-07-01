#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use z00z_core::{genesis::asset_std::asset_from_dev_cfg, AssetClass, AssetWire};
use z00z_crypto::expert::encoding::SafePassword;
use z00z_utils::io::read_file;
use z00z_wallets::domains::hashing::compute_wallet_file_id;
use z00z_wallets::key::ReceiverKeys;
use z00z_wallets::receiver::{
    PaymentRequest, PinEntry, PinnedReceiverCards, ReceiverCard, RequestInbox,
    RequestInboxValidation, RequestMetadata, RequestParams, RequestRangeHint, ScanChunk,
    TrustLevel,
};
use z00z_wallets::rpc::types::common::PersistWalletId;
use z00z_wallets::services::{AppService, WalletService};
use z00z_wallets::stealth::build_request_output_bundle;
use z00z_wallets::{bind_stealth_output_wire, WalletError};

const PASSWORD: &str = "Aa1!bB2@cC3#dD4$eE5%";
const SEED24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const DEVNET_CHAIN_ID: u32 = 3;

#[derive(Clone)]
struct WalletEnv {
    wallets: Arc<WalletService>,
    output_dir: std::path::PathBuf,
    wallet_id: PersistWalletId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WalletSnapshot {
    wlt_bytes: Vec<u8>,
    history_bytes: Option<Vec<u8>>,
}

async fn setup_wallet(name: &str) -> (WalletEnv, tempfile::TempDir) {
    let temp = tempfile::tempdir().expect("tempdir");
    let output_dir = temp.path().join("wallets");
    let wallets = Arc::new(WalletService::with_output_dir(output_dir.clone()));
    let app = AppService::with_wallet_service(Arc::clone(&wallets));

    let created = app
        .create_wallet(
            name.to_string(),
            PASSWORD.to_string(),
            Some(SEED24.to_string()),
        )
        .await
        .expect("create wallet");
    wallets
        .unlock_wallet_in_memory(&created.wallet_id, &SafePassword::from(PASSWORD))
        .await
        .expect("unlock wallet");

    (
        WalletEnv {
            wallets,
            output_dir,
            wallet_id: created.wallet_id,
        },
        temp,
    )
}

fn wallet_stem(wallet_id: &PersistWalletId) -> String {
    let hash = compute_wallet_file_id(&wallet_id.0);
    hex::encode(&hash[..8])
}

fn wallet_wlt_path(
    output_dir: &std::path::Path,
    wallet_id: &PersistWalletId,
) -> std::path::PathBuf {
    output_dir.join(format!("wallet_{}.wlt", wallet_stem(wallet_id)))
}

fn wallet_history_path(
    output_dir: &std::path::Path,
    wallet_id: &PersistWalletId,
) -> std::path::PathBuf {
    output_dir.join(format!(
        "wallet_{}_tx_history.jsonl",
        wallet_stem(wallet_id)
    ))
}

fn snapshot_wallet(env: &WalletEnv) -> WalletSnapshot {
    let wlt_path = wallet_wlt_path(&env.output_dir, &env.wallet_id);
    let history_path = wallet_history_path(&env.output_dir, &env.wallet_id);

    WalletSnapshot {
        wlt_bytes: read_file(&wlt_path).expect("read wallet file"),
        history_bytes: if history_path.exists() {
            Some(read_file(&history_path).expect("read tx history"))
        } else {
            None
        },
    }
}

fn make_request(
    keys: &ReceiverKeys,
    chain_id: u32,
    expiry_seconds: u64,
    amount: u64,
    mark: u8,
) -> PaymentRequest {
    PaymentRequest::generate(
        keys,
        RequestParams {
            amount: Some(amount),
            expiry_seconds,
            memo: Some(format!("phase062-{mark}")),
            payment_id: Some([mark; 16]),
        },
        chain_id,
    )
    .expect("generate request")
}

fn make_request_chunk(
    keys: &ReceiverKeys,
    request: &PaymentRequest,
    height: u64,
    amount: u64,
    mark: u8,
) -> ScanChunk {
    let card = keys.export_receiver_card().expect("receiver card");
    let bundle = build_request_output_bundle(
        format!("recv-{height}-{mark}"),
        z00z_wallets::tx::TxOutRole::Recipient,
        AssetClass::Coin,
        &card,
        request,
        amount,
        1,
    )
    .expect("build request bundle");
    let base_asset = asset_from_dev_cfg("z00z", 0, amount).expect("base asset");
    let asset = bind_stealth_output_wire(AssetWire::from_asset(&base_asset), &bundle.leaf)
        .expect("bind request bundle")
        .to_asset()
        .expect("request asset");

    ScanChunk {
        height,
        hash: vec![height as u8; 32],
        leaves: vec![asset],
    }
}

fn make_plain_chunk(keys: &ReceiverKeys, height: u64, amount: u64, mark: u8) -> ScanChunk {
    let card = ReceiverCard {
        version: 1,
        owner_handle: keys.owner_handle,
        view_pk: keys.view_pk.as_bytes().try_into().expect("view pk"),
        identity_pk: keys.identity_pk.as_bytes().try_into().expect("identity pk"),
        card_id: None,
        metadata: None,
        signature: [0u8; 64],
    };
    let base_asset = asset_from_dev_cfg("z00z", 0, amount).expect("asset");
    let output = z00z_wallets::build_output_bundle(
        format!("recv-{height}-{mark}"),
        z00z_wallets::tx::TxOutRole::Recipient,
        AssetClass::Coin,
        &card,
        amount,
        1,
    )
    .expect("output");
    let asset = bind_stealth_output_wire(AssetWire::from_asset(&base_asset), &output.leaf)
        .expect("bind output wire")
        .to_asset()
        .expect("scanned asset");

    ScanChunk {
        height,
        hash: vec![height as u8; 32],
        leaves: vec![asset],
    }
}

async fn prepin_request(env: &WalletEnv, keys: &ReceiverKeys) {
    let card = keys.export_receiver_card().expect("receiver card");
    let first = env
        .wallets
        .tofu_verify_pin(&env.wallet_id, &card, None)
        .await
        .expect("pin request card");
    assert!(
        matches!(
            first,
            z00z_wallets::receiver::VerifyResult::NewPin
                | z00z_wallets::receiver::VerifyResult::Verified
        ),
        "card pre-pin must succeed"
    );
}

async fn force_identity_mismatch(env: &WalletEnv, request: &PaymentRequest) {
    let pins = PinnedReceiverCards::from_pairs(vec![(
        request.owner_handle,
        PinEntry {
            view_pk: request.view_pk,
            identity_pk: [0xA5; 32],
            directory_id: None,
            first_seen: 7,
            trust_level: TrustLevel::Pinned,
        },
    )]);
    env.wallets
        .save_tofu(&env.wallet_id, &pins)
        .await
        .expect("save mismatched pin");
}

async fn assert_no_wallet_mutation(
    env: &WalletEnv,
    before: &WalletSnapshot,
    expected_error: &str,
    result: Result<z00z_wallets::receiver::ScanRangeOut, WalletError>,
) {
    let err = result.expect_err("request-assisted receive must fail closed");
    assert!(matches!(err, WalletError::InvalidParams(_)));
    assert!(
        err.to_string().contains(expected_error),
        "expected error to mention {expected_error}, got {err}"
    );
    let after = snapshot_wallet(env);
    assert_eq!(&after, before, "wallet file/history must stay unchanged");
    assert!(
        env.wallets
            .list_claimed_assets(&env.wallet_id)
            .await
            .expect("claims")
            .is_empty(),
        "invalid request path must not claim assets"
    );
}

#[test]
fn test_inbox_orders_metadata_only() {
    let mut inbox = RequestInbox::new();
    let req_a = PaymentRequest {
        version: 1,
        owner_handle: [0x11; 32],
        view_pk: [0x12; 32],
        identity_pk: [0x13; 32],
        req_id: [0x14; 32],
        chain_id: DEVNET_CHAIN_ID,
        amount: Some(7),
        expiry: u64::MAX,
        metadata: Some(RequestMetadata {
            memo: Some("a".to_string()),
            payment_id: Some([0x11; 16]),
            min_confirmations: None,
            return_receiver: None,
            created_at: 1,
        }),
        signature: [0u8; 64],
    };
    let mut req_b = req_a.clone();
    req_b.req_id = [0x21; 32];
    req_b.owner_handle = [0x22; 32];
    let mut req_c = req_a.clone();
    req_c.req_id = [0x31; 32];
    req_c.owner_handle = [0x32; 32];

    inbox.upsert(&req_a, RequestInboxValidation::Approved, None, 30);
    inbox.upsert(
        &req_b,
        RequestInboxValidation::Approved,
        Some(RequestRangeHint {
            start_height: 9,
            end_height: Some(9),
        }),
        20,
    );
    inbox.upsert(
        &req_c,
        RequestInboxValidation::Approved,
        Some(RequestRangeHint {
            start_height: 7,
            end_height: Some(7),
        }),
        10,
    );

    let ordered = inbox.list();
    assert_eq!(
        ordered
            .iter()
            .map(|record| record.request_id)
            .collect::<Vec<_>>(),
        vec![req_c.req_id, req_b.req_id, req_a.req_id]
    );
}

#[tokio::test]
async fn test_inbox_list_delete_metadata() {
    let (env, _temp) = setup_wallet("request-inbox-metadata").await;
    let keys = env
        .wallets
        .receiver_keys(&env.wallet_id)
        .await
        .expect("receiver keys");
    let request = make_request(&keys, DEVNET_CHAIN_ID, 600, 310, 0x41);
    let before = snapshot_wallet(&env);

    let mut inbox = RequestInbox::new();
    let record = inbox.upsert(
        &request,
        RequestInboxValidation::Approved,
        Some(RequestRangeHint {
            start_height: 7,
            end_height: Some(7),
        }),
        99,
    );
    assert_eq!(record.request_id, request.req_id);
    assert_eq!(inbox.len(), 1);
    assert_eq!(inbox.list()[0].request_id, request.req_id);
    assert!(inbox.remove(&request.req_id).is_some());
    assert!(inbox.is_empty());

    let after = snapshot_wallet(&env);
    assert_eq!(before, after, "metadata-only inbox must not mutate wallet");
    assert!(env
        .wallets
        .list_claimed_assets(&env.wallet_id)
        .await
        .expect("claims")
        .is_empty());
}

#[tokio::test]
async fn test_invalid_cases_keep_wallet() {
    let (wrong_chain_env, _temp) = setup_wallet("request-wrong-chain").await;
    let keys = wrong_chain_env
        .wallets
        .receiver_keys(&wrong_chain_env.wallet_id)
        .await
        .expect("receiver keys");
    let wrong_chain = make_request(&keys, DEVNET_CHAIN_ID + 1, 600, 310, 0x51);
    let wrong_chain_chunk = make_plain_chunk(&keys, 7, 310, 0x52);
    let wrong_chain_before = snapshot_wallet(&wrong_chain_env);
    let mut wrong_chain_inbox = RequestInbox::new();
    assert_no_wallet_mutation(
        &wrong_chain_env,
        &wrong_chain_before,
        "wrong chain id",
        wrong_chain_env
            .wallets
            .recv_range_with_inbox(
                &wrong_chain_env.wallet_id,
                &[wrong_chain_chunk],
                &[wrong_chain],
                &mut wrong_chain_inbox,
                None,
            )
            .await,
    )
    .await;

    let (expired_env, _temp) = setup_wallet("request-expired").await;
    let expired_keys = expired_env
        .wallets
        .receiver_keys(&expired_env.wallet_id)
        .await
        .expect("receiver keys");
    let expired = make_request(&expired_keys, DEVNET_CHAIN_ID, 0, 310, 0x61);
    let expired_chunk = make_plain_chunk(&expired_keys, 7, 310, 0x62);
    let expired_before = snapshot_wallet(&expired_env);
    let mut expired_inbox = RequestInbox::new();
    assert_no_wallet_mutation(
        &expired_env,
        &expired_before,
        "request expired",
        expired_env
            .wallets
            .recv_range_with_inbox(
                &expired_env.wallet_id,
                &[expired_chunk],
                &[expired],
                &mut expired_inbox,
                None,
            )
            .await,
    )
    .await;

    let (sig_env, _temp) = setup_wallet("request-bad-signature").await;
    let sig_keys = sig_env
        .wallets
        .receiver_keys(&sig_env.wallet_id)
        .await
        .expect("receiver keys");
    let mut bad_sig = make_request(&sig_keys, DEVNET_CHAIN_ID, 600, 310, 0x71);
    bad_sig.signature[0] ^= 0x55;
    let sig_chunk = make_plain_chunk(&sig_keys, 7, 310, 0x72);
    let sig_before = snapshot_wallet(&sig_env);
    let mut sig_inbox = RequestInbox::new();
    assert_no_wallet_mutation(
        &sig_env,
        &sig_before,
        "invalid signature",
        sig_env
            .wallets
            .recv_range_with_inbox(
                &sig_env.wallet_id,
                &[sig_chunk],
                &[bad_sig],
                &mut sig_inbox,
                None,
            )
            .await,
    )
    .await;

    let (pin_env, _temp) = setup_wallet("request-pin-mismatch").await;
    let pin_keys = pin_env
        .wallets
        .receiver_keys(&pin_env.wallet_id)
        .await
        .expect("receiver keys");
    let pin_request = make_request(&pin_keys, DEVNET_CHAIN_ID, 600, 310, 0x81);
    force_identity_mismatch(&pin_env, &pin_request).await;
    let pin_chunk = make_plain_chunk(&pin_keys, 7, 310, 0x82);
    let pin_before = snapshot_wallet(&pin_env);
    let mut pin_inbox = RequestInbox::new();
    assert_no_wallet_mutation(
        &pin_env,
        &pin_before,
        "request identity mismatch",
        pin_env
            .wallets
            .recv_range_with_inbox(
                &pin_env.wallet_id,
                &[pin_chunk],
                &[pin_request],
                &mut pin_inbox,
                None,
            )
            .await,
    )
    .await;

    let (version_env, _temp) = setup_wallet("request-unsupported-version").await;
    let version_keys = version_env
        .wallets
        .receiver_keys(&version_env.wallet_id)
        .await
        .expect("receiver keys");
    let mut bad_version = make_request(&version_keys, DEVNET_CHAIN_ID, 600, 310, 0x91);
    bad_version.version = 2;
    let version_chunk = make_plain_chunk(&version_keys, 7, 310, 0x92);
    let version_before = snapshot_wallet(&version_env);
    let mut version_inbox = RequestInbox::new();
    assert_no_wallet_mutation(
        &version_env,
        &version_before,
        "unsupported request version",
        version_env
            .wallets
            .recv_range_with_inbox(
                &version_env.wallet_id,
                &[version_chunk],
                &[bad_version],
                &mut version_inbox,
                None,
            )
            .await,
    )
    .await;
}

#[tokio::test]
async fn test_valid_case_reenters_lane() {
    let (env, _temp) = setup_wallet("request-valid").await;
    let keys = env
        .wallets
        .receiver_keys(&env.wallet_id)
        .await
        .expect("receiver keys");
    let request = make_request(&keys, DEVNET_CHAIN_ID, 600, 310, 0xA1);
    prepin_request(&env, &keys).await;
    let before = snapshot_wallet(&env);
    let chunk = make_request_chunk(&keys, &request, 7, 310, 0xA2);
    let mut inbox = RequestInbox::new();

    let out = env
        .wallets
        .recv_range_with_inbox(
            &env.wallet_id,
            &[chunk],
            &[request.clone()],
            &mut inbox,
            None,
        )
        .await
        .expect("request-assisted receive");

    assert_eq!(out.outputs.len(), 1);
    assert_eq!(out.stat.cursor.height(), 7);
    let claimed = env
        .wallets
        .list_claimed_assets(&env.wallet_id)
        .await
        .expect("claims");
    assert_eq!(claimed.len(), 1);
    assert_eq!(inbox.len(), 1);
    assert_eq!(inbox.list()[0].validation, RequestInboxValidation::Approved);

    let after = snapshot_wallet(&env);
    assert_ne!(before.wlt_bytes, after.wlt_bytes);
    assert_eq!(
        before.history_bytes, after.history_bytes,
        "request receive must not create tx history rows"
    );
}
