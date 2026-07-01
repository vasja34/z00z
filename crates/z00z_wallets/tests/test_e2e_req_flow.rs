//! E2E Phase 1 request-bound flow.
//! Acceptance coverage: only base and request-bound `k_dh` survive, only
//! card-bound and request-bound `tag16` survive, and request-aware receive
//! semantics do not collapse into plain-path or card-bound fallback
//! behavior. These request-bound tests lock in the promotion path toward the
//! normal privacy path while keeping any card-only fallback explicit.

use std::sync::Arc;

use thiserror::Error;
use z00z_core::{genesis::asset_std::asset_from_dev_cfg, Asset};
use z00z_wallets::{
    build_tx_output_unchecked,
    key::{ReceiverKeys, ReceiverSecret, StealthKeyError},
    receiver::{PaymentRequest, RequestMetadata, ValidityStatus},
    receiver::{ReceiveNext, ReceiveReject, ReceiveStatus, ScanResult, StealthOutputScanner},
    stealth::ecdh::{compute_dh_receiver, decode_r_pub},
    stealth::kdf::{
        compute_leaf_ad, compute_tag16, compute_tag16_with_req, derive_k_dh, derive_k_dh_with_req,
    },
    SenderWallet, StealthError, TxStealthOutput,
};

const WALLET_SERVICE_SRC: &str = include_str!("../src/services/wallet_service.rs");
const WALLET_SERVICE_ACTIONS_RECEIVE_SRC: &str =
    include_str!("../src/services/wallet_actions_receive.rs");

#[derive(Debug, Error)]
enum TestErr {
    #[error(transparent)]
    Key(#[from] StealthKeyError),
    #[error(transparent)]
    Stealth(#[from] StealthError),
    #[error(transparent)]
    Req(#[from] z00z_wallets::receiver::PaymentRequestError),
    #[error("asset build failed")]
    Asset,
    #[error("expected owned payment")]
    Owned,
    #[error("expected not-mine payment")]
    Foreign,
    #[error("request-bound tag mismatch")]
    Tag,
}

struct ReqCase {
    keys: ReceiverKeys,
    req: PaymentRequest,
    bad: PaymentRequest,
    asset: Asset,
    out: TxStealthOutput,
    aid: [u8; 32],
    amount: u64,
}

fn mk_keys(seed: u8) -> Result<ReceiverKeys, TestErr> {
    let mut bytes = [seed; 32];
    bytes[31] ^= 0x3C;
    let secret = ReceiverSecret::from_bytes(bytes)?;
    ReceiverKeys::from_receiver_secret(secret).map_err(TestErr::from)
}

fn mk_req(keys: &ReceiverKeys, tail: u8) -> Result<PaymentRequest, TestErr> {
    let mut req_id = [0x90; 32];
    req_id[31] = tail;
    let mut req = PaymentRequest {
        version: 1,
        owner_handle: keys.owner_handle,
        view_pk: keys
            .view_pk
            .as_bytes()
            .try_into()
            .map_err(|_| TestErr::Tag)?,
        identity_pk: keys
            .identity_pk
            .as_bytes()
            .try_into()
            .map_err(|_| TestErr::Tag)?,
        req_id,
        chain_id: 77,
        amount: Some(7_000),
        expiry: u64::MAX,
        metadata: Some(RequestMetadata {
            memo: Some("phase1 invoice".to_string()),
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

fn mk_asset(amount: u64, aid: [u8; 32], out: &TxStealthOutput) -> Result<Asset, TestErr> {
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

fn req_tag(
    keys: &ReceiverKeys,
    req: &PaymentRequest,
    out: &TxStealthOutput,
) -> Result<u16, TestErr> {
    let r_pub = decode_r_pub(&out.r_pub)?;
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &r_pub)?;
    let k_dh = derive_k_dh_with_req(&dh, &req.req_id);
    Ok(compute_tag16_with_req(&k_dh, &req.req_id))
}

fn card_tag(keys: &ReceiverKeys, aid: &[u8; 32], out: &TxStealthOutput) -> Result<u16, TestErr> {
    let r_pub = decode_r_pub(&out.r_pub)?;
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &r_pub)?;
    let k_dh = derive_k_dh(&dh);
    let leaf_ad = compute_leaf_ad(aid, 0, &out.r_pub, &out.owner_tag, &out.c_amount);
    Ok(compute_tag16(&k_dh, &leaf_ad))
}

fn mk_case() -> Result<ReqCase, TestErr> {
    let keys = mk_keys(0x61)?;
    let req = mk_req(&keys, 0xA1)?;
    let bad = mk_req(&keys, 0xB2)?;
    let card = keys.export_receiver_card()?;
    let mut sender = SenderWallet::new([0x71; 32]);
    let amount = req.amount.expect("req amount");
    let aid = asset_from_dev_cfg("z00z", 0, amount)
        .map_err(|_| TestErr::Asset)?
        .definition
        .id;
    let out =
        build_tx_output_unchecked(&card, Some(&req), &mut sender, &[0x81; 32], 0, amount, &aid)?;
    let asset = mk_asset(amount, aid, &out)?;

    Ok(ReqCase {
        keys,
        req,
        bad,
        asset,
        out,
        aid,
        amount,
    })
}

fn must_mine(scan: &StealthOutputScanner, asset: &Asset, amount: u64) -> Result<(), TestErr> {
    let result = scan.scan_leaf(asset);
    let report = result.recv_report();
    let ScanResult::Mine { ref wallet_output } = result else {
        return Err(TestErr::Owned);
    };

    assert_eq!(report.status, ReceiveStatus::Detected);
    assert_eq!(report.reject, None);
    assert_eq!(report.next, ReceiveNext::ReportOnly);
    assert_eq!(wallet_output.asset_id, asset.asset_id());
    assert_eq!(wallet_output.serial_id, asset.serial_id);
    assert_eq!(wallet_output.amount, amount);
    assert_eq!(wallet_output.r_pub, asset.r_pub.expect("r_pub"));
    assert_eq!(wallet_output.owner_tag, asset.owner_tag.expect("owner_tag"));
    Ok(())
}

fn must_not_mine(scan: &StealthOutputScanner, asset: &Asset) -> Result<(), TestErr> {
    let res = scan.scan_leaf(asset);
    if !matches!(res, ScanResult::NotMine) {
        return Err(TestErr::Foreign);
    }
    assert_eq!(res.recv_report().status, ReceiveStatus::NotMine);
    assert_eq!(res.recv_report().reject, Some(ReceiveReject::NotMine));
    assert_eq!(res.recv_report().next, ReceiveNext::ReportOnly);
    Ok(())
}

fn check_good(case: &ReqCase) -> Result<(), TestErr> {
    let mut good = StealthOutputScanner::from_keys(&case.keys);
    good.add_request(&case.req);
    must_mine(&good, &case.asset, case.amount)
}

fn check_plain(case: &ReqCase) -> Result<(), TestErr> {
    let plain = StealthOutputScanner::from_keys(&case.keys);
    must_not_mine(&plain, &case.asset)
}

fn check_wrong(case: &ReqCase) -> Result<(), TestErr> {
    let mut wrong = StealthOutputScanner::from_keys(&case.keys);
    wrong.add_request(&case.bad);
    must_not_mine(&wrong, &case.asset)
}

fn tag_is_req(case: &ReqCase) {
    let good_tag = req_tag(&case.keys, &case.req, &case.out).expect("request tag");
    assert_eq!(case.out.tag16, Some(good_tag), "{}", TestErr::Tag);
}

fn tag_not_card(case: &ReqCase) {
    let base_tag = card_tag(&case.keys, &case.aid, &case.out).expect("card tag");
    assert_ne!(case.out.tag16, Some(base_tag), "{}", TestErr::Tag);
}

fn run_req(case: &ReqCase) {
    tag_is_req(case);
    tag_not_card(case);
    check_good(case).expect("request-owned scan");
    check_plain(case).expect("plain-path reject");
    check_wrong(case).expect("wrong-request reject");
}

#[test]
fn test_e2e_req_flow() {
    let case = mk_case().expect("phase 1 request fixture");
    assert_ne!(case.req.req_id, case.bad.req_id);
    run_req(&case);
}

#[test]
fn test_request_lane_stays_distinct() {
    let case = mk_case().expect("phase 1 request fixture");

    let mut req_scan = StealthOutputScanner::from_keys(&case.keys);
    req_scan.add_request(&case.req);
    must_mine(&req_scan, &case.asset, case.amount).expect("request-aware scan");

    let plain_scan = StealthOutputScanner::from_keys(&case.keys);
    must_not_mine(&plain_scan, &case.asset).expect("plain scan stays noncanonical");
}

#[test]
fn test_request_card_routes_distinct() {
    assert!(
        WALLET_SERVICE_SRC.contains("preferred privacy lane")
            && WALLET_SERVICE_SRC.contains("plain or card-bound noncanonical"),
        "wallet service facade must keep request-aware receive distinct from card-bound receive"
    );
    assert!(
        WALLET_SERVICE_ACTIONS_RECEIVE_SRC.contains("preferred request-aware")
            && WALLET_SERVICE_ACTIONS_RECEIVE_SRC.contains("receive lane")
            && WALLET_SERVICE_ACTIONS_RECEIVE_SRC.contains("request-bound inbox")
            && WALLET_SERVICE_ACTIONS_RECEIVE_SRC.contains("off-consensus")
            && WALLET_SERVICE_ACTIONS_RECEIVE_SRC.contains("card-path")
            && WALLET_SERVICE_ACTIONS_RECEIVE_SRC.contains("equivalent privacy theorem"),
        "receive actions must reject card-path equivalence claims"
    );
}
