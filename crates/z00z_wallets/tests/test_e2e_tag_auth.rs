//! E2E Phase 2 tag authority flow.
//! Acceptance coverage: `wallet_asset_scanner` is adapter-only, `asset_leaf_scan` stays
//! authoritative for card-bound ownership, and tag equality alone never proves
//! ownership across card-bound or request-bound paths.

use std::sync::Arc;

use thiserror::Error;
use z00z_core::{assets::AssetLeaf, genesis::asset_std::asset_from_dev_cfg, Asset};
use z00z_wallets::{
    build_tx_output_unchecked,
    key::{ReceiverKeys, ReceiverSecret, StealthKeyError},
    receiver::{
        receiver_scan_leaf, receiver_scan_report, PaymentRequest, PaymentRequestError,
        ReceiveReject, ReceiveStatus, RequestMetadata, ScanResult, StealthOutputScanner,
        Tag16Context, ValidityStatus,
    },
    stealth::{
        ecdh::{compute_dh_receiver, decode_r_pub},
        kdf::{derive_k_dh, derive_k_dh_with_req},
    },
    SenderWallet, StealthError, TxStealthOutput,
};

#[derive(Debug, Error)]
enum TestErr {
    #[error(transparent)]
    Key(#[from] StealthKeyError),
    #[error(transparent)]
    Stealth(#[from] StealthError),
    #[error(transparent)]
    Req(#[from] PaymentRequestError),
    #[error("asset build failed")]
    Asset,
}

struct CardCase {
    keys: ReceiverKeys,
    asset: Asset,
    leaf: AssetLeaf,
}

struct ReqCase {
    keys: ReceiverKeys,
    req: PaymentRequest,
    bad: PaymentRequest,
    asset: Asset,
}

fn mk_keys(seed: u8) -> Result<ReceiverKeys, TestErr> {
    let mut bytes = [seed; 32];
    bytes[31] ^= 0x39;
    let secret = ReceiverSecret::from_bytes(bytes)?;
    ReceiverKeys::from_receiver_secret(secret).map_err(TestErr::from)
}

fn mk_req(keys: &ReceiverKeys, tail: u8) -> Result<PaymentRequest, TestErr> {
    let mut req_id = [0x70; 32];
    req_id[31] = tail;
    let mut req = PaymentRequest {
        version: 1,
        owner_handle: keys.owner_handle,
        view_pk: keys
            .view_pk
            .as_bytes()
            .try_into()
            .map_err(|_| TestErr::Asset)?,
        identity_pk: keys
            .identity_pk
            .as_bytes()
            .try_into()
            .map_err(|_| TestErr::Asset)?,
        req_id,
        chain_id: 77,
        amount: Some(800),
        expiry: u64::MAX,
        metadata: Some(RequestMetadata {
            memo: Some("phase2 tag".to_string()),
            payment_id: Some([tail; 16]),
            min_confirmations: None,
            return_receiver: None,
            created_at: 1,
        }),
        signature: [0u8; 64],
    };

    req.sign(keys.reveal_identity_sk())?;
    assert!(matches!(req.check_validity(), ValidityStatus::Valid(_)));
    req.verify()?;
    Ok(req)
}

fn mk_asset(amount: u64, out: &TxStealthOutput, aid: [u8; 32]) -> Result<Asset, TestErr> {
    let mut asset = asset_from_dev_cfg("z00z", 0, amount).map_err(|_| TestErr::Asset)?;
    let mut def = (*asset.definition).clone();
    def.id = aid;
    asset.definition = Arc::new(def);
    asset.commitment = z00z_crypto::Commitment::from_bytes(&out.c_amount)
        .map_err(|_| TestErr::Asset)?
        .0;
    asset.owner_pub = None;
    asset.owner_signature = None;
    asset.r_pub = Some(out.r_pub);
    asset.owner_tag = Some(out.owner_tag);
    asset.enc_pack = Some(out.enc_pack.clone());
    asset.tag16 = out.tag16;
    asset.leaf_ad_id = Some(asset.definition.id);
    Ok(asset)
}

fn mk_leaf(out: &TxStealthOutput, aid: [u8; 32]) -> AssetLeaf {
    AssetLeaf {
        asset_id: aid,
        serial_id: 0,
        r_pub: out.r_pub,
        owner_tag: out.owner_tag,
        c_amount: out.c_amount,
        enc_pack: out.enc_pack.clone(),
        range_proof: Vec::new(),
        tag16: out.tag16.expect("tag16"),
    }
}

fn card_ctx(keys: &ReceiverKeys, asset: &Asset) -> Result<Tag16Context, TestErr> {
    let r_pub = decode_r_pub(&asset.r_pub.expect("r_pub"))?;
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &r_pub)?;
    Ok(Tag16Context {
        k_dh: derive_k_dh(&dh),
        req_id: None,
    })
}

fn req_ctx(
    keys: &ReceiverKeys,
    asset: &Asset,
    req: &PaymentRequest,
) -> Result<Tag16Context, TestErr> {
    let r_pub = decode_r_pub(&asset.r_pub.expect("r_pub"))?;
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &r_pub)?;
    Ok(Tag16Context {
        k_dh: derive_k_dh_with_req(&dh, &req.req_id),
        req_id: Some(req.req_id),
    })
}

fn flip_tag(mut asset: Asset) -> Asset {
    asset.tag16 = asset.tag16.map(|tag| tag ^ 1);
    asset
}

fn flip_leaf(mut leaf: AssetLeaf) -> AssetLeaf {
    leaf.tag16 ^= 1;
    leaf
}

fn mk_card_case() -> Result<CardCase, TestErr> {
    let keys = mk_keys(0x21)?;
    let card = keys.export_receiver_card()?;
    let mut sender = SenderWallet::new([0x31; 32]);
    let aid = [0x41; 32];
    let out = build_tx_output_unchecked(&card, None, &mut sender, &[0x51; 32], 0, 700, &aid)?;
    Ok(CardCase {
        keys,
        asset: mk_asset(700, &out, aid)?,
        leaf: mk_leaf(&out, aid),
    })
}

fn mk_req_case() -> Result<ReqCase, TestErr> {
    let keys = mk_keys(0x61)?;
    let req = mk_req(&keys, 0xA1)?;
    let bad = mk_req(&keys, 0xB2)?;
    let card = keys.export_receiver_card()?;
    let mut sender = SenderWallet::new([0x71; 32]);
    let aid = [0x81; 32];
    let amount = req.amount.expect("amount");
    let out =
        build_tx_output_unchecked(&card, Some(&req), &mut sender, &[0x91; 32], 0, amount, &aid)?;
    Ok(ReqCase {
        keys,
        req,
        bad,
        asset: mk_asset(amount, &out, aid)?,
    })
}

fn is_mine(res: ScanResult) {
    assert!(matches!(res, ScanResult::Mine { .. }));
}

fn is_none(res: ScanResult) {
    assert!(matches!(res, ScanResult::NotMine));
}

fn is_maybe(res: ScanResult) {
    assert!(matches!(
        res,
        ScanResult::MaybeMine {
            tag16_match: true,
            m1_failed: true
        }
    ));
}

fn no_mine(scan: &StealthOutputScanner, asset: &Asset) {
    assert!(!matches!(scan.scan_leaf(asset), ScanResult::Mine { .. }));
}

fn own_leaf(keys: &ReceiverKeys, leaf: &AssetLeaf) {
    assert!(receiver_scan_leaf(keys, leaf).expect("leaf scan").is_some());
}

fn no_leaf(keys: &ReceiverKeys, leaf: &AssetLeaf) {
    assert!(receiver_scan_leaf(keys, leaf).expect("leaf scan").is_none());
}

fn own_rep(keys: &ReceiverKeys, leaf: &AssetLeaf) {
    let rep = receiver_scan_report(keys, leaf).expect("leaf report");
    assert_eq!(rep.status, ReceiveStatus::Detected);
}

fn bad_rep(keys: &ReceiverKeys, leaf: &AssetLeaf) {
    let rep = receiver_scan_report(keys, leaf).expect("leaf report");
    assert_eq!(rep.status, ReceiveStatus::InvalidProof);
    assert_eq!(rep.reject, Some(ReceiveReject::InvalidProof));
}

fn card_hit(scan: &StealthOutputScanner, asset: &Asset) {
    is_mine(scan.scan_leaf_tag_only(asset));
    is_mine(scan.scan_leaf(asset));
}

fn card_miss(scan: &StealthOutputScanner, asset: &Asset) {
    is_none(scan.scan_leaf_tag_only(asset));
    no_mine(scan, asset);
}

fn req_hit(scan: &StealthOutputScanner, asset: &Asset) {
    is_mine(scan.scan_leaf_tag_only(asset));
    is_mine(scan.scan_leaf(asset));
}

fn req_miss(scan: &StealthOutputScanner, asset: &Asset) {
    is_maybe(scan.scan_leaf_tag_only(asset));
    no_mine(scan, asset);
}

fn extract_tag16(asset: &Asset) -> u16 {
    asset.tag16.expect("tag16")
}

fn scan_card(case: &CardCase) -> StealthOutputScanner {
    let mut scan = StealthOutputScanner::from_keys(&case.keys);
    scan.add_tag_context(
        extract_tag16(&case.asset),
        card_ctx(&case.keys, &case.asset).expect("card ctx"),
    );
    scan.materialize_complete_tag_contexts(Vec::<(u16, Tag16Context)>::new());
    scan
}

fn scan_wrong(case: &CardCase) -> StealthOutputScanner {
    let mut scan = StealthOutputScanner::from_keys(&case.keys);
    scan.add_tag_context(
        extract_tag16(&case.asset),
        Tag16Context {
            k_dh: [0xAA; 32],
            req_id: None,
        },
    );
    scan.materialize_complete_tag_contexts(Vec::<(u16, Tag16Context)>::new());
    scan
}

fn scan_req(case: &ReqCase, req: &PaymentRequest) -> StealthOutputScanner {
    let mut scan = StealthOutputScanner::from_keys(&case.keys);
    scan.add_request(req);
    scan.add_tag_context(
        extract_tag16(&case.asset),
        req_ctx(&case.keys, &case.asset, req).expect("req ctx"),
    );
    scan.materialize_complete_tag_contexts(Vec::<(u16, Tag16Context)>::new());
    scan
}

fn req_none(case: &ReqCase) {
    let scan = StealthOutputScanner::from_keys(&case.keys);
    is_none(scan.scan_leaf_tag_only(&case.asset));
    let full = scan.scan_leaf(&case.asset);
    assert!(matches!(full, ScanResult::NotMine));
    assert_eq!(full.recv_report().status, ReceiveStatus::NotMine);
    assert_eq!(full.recv_report().reject, Some(ReceiveReject::NotMine));
}

fn req_card(case: &ReqCase) {
    let mut scan = StealthOutputScanner::from_keys(&case.keys);
    scan.add_tag_context(
        extract_tag16(&case.asset),
        card_ctx(&case.keys, &case.asset).expect("card ctx"),
    );
    scan.materialize_complete_tag_contexts(Vec::<(u16, Tag16Context)>::new());
    is_maybe(scan.scan_leaf_tag_only(&case.asset));
    no_mine(&scan, &case.asset);
}

fn card_good(case: &CardCase) {
    let good = scan_card(case);
    card_hit(&good, &case.asset);
    own_leaf(&case.keys, &case.leaf);
    own_rep(&case.keys, &case.leaf);
}

fn card_wrong(case: &CardCase) {
    let wrong = scan_wrong(case);
    is_maybe(wrong.scan_leaf_tag_only(&case.asset));
    is_mine(wrong.scan_leaf(&case.asset));
    own_leaf(&case.keys, &case.leaf);
}

fn card_flip(case: &CardCase) {
    let good = scan_card(case);
    let bad_asset = flip_tag(case.asset.clone());
    let bad_leaf = flip_leaf(case.leaf.clone());

    card_miss(&good, &bad_asset);
    no_leaf(&case.keys, &bad_leaf);
    bad_rep(&case.keys, &bad_leaf);
}

fn req_good(case: &ReqCase) {
    let good = scan_req(case, &case.req);
    req_hit(&good, &case.asset);
}

fn req_bad(case: &ReqCase) {
    let bad = scan_req(case, &case.bad);
    req_miss(&bad, &case.asset);
}

fn card_main(case: &CardCase) {
    card_good(case);
    card_wrong(case);
    card_flip(case);
}

fn req_main(case: &ReqCase) {
    req_none(case);
    req_card(case);
    req_bad(case);
    req_good(case);
}

#[test]
fn test_e2e_tag_auth_card() {
    let case = mk_card_case().expect("card case");
    card_main(&case);
}

#[test]
fn test_e2e_tag_auth_req() {
    let case = mk_req_case().expect("req case");
    req_main(&case);
}
