#![cfg(not(target_arch = "wasm32"))]

//! E2E Phase 2 runtime parity flow.
//! Acceptance coverage: integration runtime receive and canonical leaf scan
//! must keep parity on the same artifact for owned and foreign receiver paths.

use std::sync::Arc;

use tempfile::TempDir;
use z00z_core::{assets::AssetLeaf, genesis::asset_std::asset_from_dev_class, Asset, AssetClass};
use z00z_crypto::{create_range_proof, Z00ZScalar};
use z00z_wallets::{
    build_tx_output_unchecked,
    key::ReceiverKeys,
    receiver::{receiver_scan_leaf, receiver_scan_report},
    receiver::{
        DetectedAssetPack, ReceiveNext, ReceiveReject, ReceiveReport, ReceiveStatus, ScanResult,
        StealthOutputScanner,
    },
    rpc::{
        methods::{WalletRpcImpl, WalletRpcServer},
        types::common::PersistWalletId,
    },
    services::{AppService, WalletService},
    tx::wire_decrypt_leaf,
    SenderWallet,
};

const PASS: &str = "CorrectPassw0rd!";

struct RtEnv {
    _tmp: TempDir,
    app: AppService,
    rpc: WalletRpcImpl,
    svc: Arc<WalletService>,
}

struct Case {
    bob_id: PersistWalletId,
    carol_id: PersistWalletId,
    bob_keys: ReceiverKeys,
    carol_keys: ReceiverKeys,
    leaf: AssetLeaf,
    asset: Asset,
    amount: u64,
}

struct OwnChk {
    case: Case,
    pack: DetectedAssetPack,
    canon: ReceiveReport,
    run: ReceiveReport,
    scan: ScanResult,
}

fn mk_env() -> RtEnv {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("wallets");
    let svc = Arc::new(WalletService::with_output_dir(out));
    let app = AppService::with_wallet_service(Arc::clone(&svc));
    let rpc = WalletRpcImpl::new(Arc::clone(&svc));

    RtEnv {
        _tmp: tmp,
        app,
        rpc,
        svc,
    }
}

async fn mk_id(env: &RtEnv, name: &str) -> PersistWalletId {
    let made = env
        .app
        .create_wallet(name.to_string(), PASS.to_string(), None)
        .await
        .expect("create");
    let wallet_id = made.wallet_id.clone();

    let _ = env
        .rpc
        .unlock_wallet(wallet_id.clone(), PASS.to_string())
        .await
        .expect("unlock");

    wallet_id
}

fn mk_asset(leaf: &AssetLeaf, amount: u64) -> Asset {
    let mut asset = asset_from_dev_class(AssetClass::Coin, leaf.serial_id, amount).expect("asset");
    asset.leaf_ad_id = Some(leaf.asset_id);
    asset.commitment = z00z_crypto::Commitment::from_bytes(&leaf.c_amount)
        .expect("commitment")
        .as_commitment()
        .clone();
    asset.owner_pub = None;
    asset.owner_signature = None;
    asset.r_pub = Some(leaf.r_pub);
    asset.owner_tag = Some(leaf.owner_tag);
    asset.enc_pack = Some(leaf.enc_pack.clone());
    asset.tag16 = Some(leaf.tag16);
    asset.range_proof = Some(leaf.range_proof.clone());
    asset
}

async fn mk_case(env: &RtEnv) -> Case {
    let bob_id = mk_id(env, "phase2-bob").await;
    let carol_id = mk_id(env, "phase2-carol").await;
    let bob_keys = env.svc.receiver_keys(&bob_id).await.expect("bob keys");
    let carol_keys = env.svc.receiver_keys(&carol_id).await.expect("carol keys");
    let amount = 913;
    let aid = [0x17; 32];
    let sid = 0;
    let card = bob_keys.export_receiver_card().expect("card");
    let mut sender = SenderWallet::new([0x27; 32]);
    let out = build_tx_output_unchecked(&card, None, &mut sender, &[0x37; 32], sid, amount, &aid)
        .expect("output");
    let mut leaf = AssetLeaf {
        asset_id: aid,
        serial_id: sid,
        r_pub: out.r_pub,
        owner_tag: out.owner_tag,
        c_amount: out.c_amount,
        enc_pack: out.enc_pack,
        range_proof: Vec::new(),
        tag16: out.tag16.expect("tag16"),
    };
    let pack = receiver_scan_leaf(&bob_keys, &leaf)
        .expect("scan")
        .expect("owned pack");
    let blind = Z00ZScalar::try_from_bytes(pack.blinding).expect("blinding");
    leaf.range_proof = create_range_proof(amount, &blind, 64, 0).expect("proof");
    let asset = mk_asset(&leaf, amount);

    Case {
        bob_id,
        carol_id,
        bob_keys,
        carol_keys,
        leaf,
        asset,
        amount,
    }
}

fn pair_ids(case: &Case) {
    let runtime_leaf = wire_decrypt_leaf(&z00z_core::AssetWire::from_asset(&case.asset))
        .expect("runtime decrypt leaf");
    assert_eq!(runtime_leaf.asset_id, case.leaf.asset_id);
    assert_eq!(case.asset.serial_id, case.leaf.serial_id);
}

fn pair_meta(case: &Case) {
    assert_eq!(case.asset.r_pub, Some(case.leaf.r_pub));
    assert_eq!(case.asset.owner_tag, Some(case.leaf.owner_tag));
}

fn pair_pack(case: &Case) {
    assert_eq!(case.asset.enc_pack, Some(case.leaf.enc_pack.clone()));
    assert_eq!(case.asset.tag16, Some(case.leaf.tag16));
    assert_eq!(case.asset.commitment.as_bytes(), &case.leaf.c_amount);
}

fn own_can(case: &Case) -> (DetectedAssetPack, ReceiveReport) {
    let pack = receiver_scan_leaf(&case.bob_keys, &case.leaf)
        .expect("scan")
        .expect("owned pack");
    let rep = receiver_scan_report(&case.bob_keys, &case.leaf).expect("report");
    (pack, rep)
}

async fn own_run(env: &RtEnv, case: &Case) -> ReceiveReport {
    env.svc
        .scan_asset_report(&case.bob_id, &case.asset)
        .await
        .expect("recv")
}

async fn own_data() -> OwnChk {
    let env = mk_env();
    let case = mk_case(&env).await;
    let (pack, canon) = own_can(&case);
    let run = own_run(&env, &case).await;
    let scan = StealthOutputScanner::from_keys(&case.bob_keys).scan_leaf(&case.asset);

    OwnChk {
        case,
        pack,
        canon,
        run,
        scan,
    }
}

fn own_rep(case: &Case, pack: &DetectedAssetPack, canon: ReceiveReport, run: &ReceiveReport) {
    assert_eq!(canon, *run);
    assert_eq!(run.status, ReceiveStatus::Detected);
    assert_eq!(run.reject, None);
    assert_eq!(run.next, ReceiveNext::ReportOnly);
    assert_eq!(pack.value, case.amount);
}

fn own_out(case: &Case, pack: &DetectedAssetPack, scan: ScanResult) {
    let ScanResult::Mine { wallet_output } = scan else {
        panic!("expected Mine, got {scan:?}");
    };

    assert_eq!(wallet_output.amount, pack.value);
    assert_eq!(wallet_output.asset_id, case.asset.asset_id());
    assert_eq!(wallet_output.serial_id, case.leaf.serial_id);
    assert_eq!(wallet_output.asset_secret, Some(pack.s_out));
    assert_eq!(wallet_output.blinding, Some(pack.blinding));
    assert_eq!(wallet_output.r_pub, case.leaf.r_pub);
    assert_eq!(wallet_output.owner_tag, case.leaf.owner_tag);
}

fn own_pair(case: &Case) {
    pair_ids(case);
    pair_meta(case);
    pair_pack(case);
}

fn own_cmp(got: OwnChk) {
    own_pair(&got.case);
    own_rep(&got.case, &got.pack, got.canon, &got.run);
    own_out(&got.case, &got.pack, got.scan);
}

fn bad_can(case: &Case) -> (Option<DetectedAssetPack>, ReceiveReport) {
    let pack = receiver_scan_leaf(&case.carol_keys, &case.leaf).expect("scan");
    let rep = receiver_scan_report(&case.carol_keys, &case.leaf).expect("report");
    (pack, rep)
}

async fn bad_run(env: &RtEnv, case: &Case) -> ReceiveReport {
    env.svc
        .scan_asset_report(&case.carol_id, &case.asset)
        .await
        .expect("recv")
}

fn bad_cmp(pack: Option<DetectedAssetPack>, canon: ReceiveReport, run: ReceiveReport) {
    assert!(pack.is_none());
    assert_eq!(canon, run);
    assert_eq!(canon.status, ReceiveStatus::NotMine);
    assert_eq!(canon.reject, Some(ReceiveReject::NotMine));
    assert_eq!(canon.next, ReceiveNext::ReportOnly);
}

#[tokio::test]
async fn test_e2e_runtime_own() {
    own_cmp(own_data().await);
}

#[tokio::test]
async fn test_e2e_runtime_foreign() {
    let env = mk_env();
    let case = mk_case(&env).await;
    let (pack, canon) = bad_can(&case);
    let run = bad_run(&env, &case).await;

    bad_cmp(pack, canon, run);
}
