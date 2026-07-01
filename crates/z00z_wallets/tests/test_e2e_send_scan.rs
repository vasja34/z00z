//! E2E Phase 1 canonical send-and-scan flow.
//! Acceptance coverage: one public path for sender/scan work, canonical owned
//! detection flow, and foreign-output rejection through the scan facade.

use thiserror::Error;
use z00z_core::{genesis::asset_std::asset_from_dev_cfg, Asset};
use z00z_wallets::{
    build_tx_output_unchecked,
    key::{derive_view_secret_key, ReceiverKeys, ReceiverSecret, StealthKeyError},
    receiver::{ReceiveNext, ReceiveReject, ReceiveStatus, ScanResult, StealthOutputScanner},
    SenderWallet, StealthError, TxStealthOutput,
};

#[derive(Debug, Error)]
enum TestErr {
    #[error(transparent)]
    Key(#[from] StealthKeyError),
    #[error(transparent)]
    Stealth(#[from] StealthError),
    #[error("asset build failed")]
    Asset,
    #[error("owned scan expected")]
    Owned,
    #[error("foreign scan expected")]
    Foreign,
}

struct SendCase {
    bob: ReceiverKeys,
    carol: ReceiverKeys,
    asset: Asset,
    aid: [u8; 32],
    amount: u64,
}

fn mk_keys(seed: u8) -> Result<ReceiverKeys, TestErr> {
    let mut bytes = [seed; 32];
    bytes[31] ^= 0x5A;
    let secret = ReceiverSecret::from_bytes(bytes)?;
    ReceiverKeys::from_receiver_secret(secret).map_err(TestErr::from)
}

fn mk_secret(seed: u8) -> Result<ReceiverSecret, TestErr> {
    let mut bytes = [seed; 32];
    bytes[31] ^= 0x5A;
    ReceiverSecret::from_bytes(bytes).map_err(TestErr::from)
}

fn mk_asset(amount: u64, out: &TxStealthOutput) -> Result<Asset, TestErr> {
    let mut asset = asset_from_dev_cfg("z00z", 0, amount).map_err(|_| TestErr::Asset)?;
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

fn must_mine(
    scan: &StealthOutputScanner,
    asset: &Asset,
    _aid: [u8; 32],
    amount: u64,
) -> Result<(), TestErr> {
    let result = scan.scan_leaf(asset);
    let report = result.recv_report();
    let ScanResult::Mine { ref wallet_output } = result else {
        return Err(TestErr::Owned);
    };

    assert_eq!(report.status, ReceiveStatus::Detected);
    assert_eq!(report.reject, None);
    assert_eq!(report.next, ReceiveNext::ReportOnly);
    assert_eq!(wallet_output.asset_id, asset.asset_id());
    assert_eq!(wallet_output.serial_id, 0);
    assert_eq!(wallet_output.amount, amount);
    assert_eq!(wallet_output.r_pub, asset.r_pub.expect("r_pub"));
    assert_eq!(wallet_output.owner_tag, asset.owner_tag.expect("owner_tag"));
    Ok(())
}

fn must_not_mine(scan: &StealthOutputScanner, asset: &Asset) -> Result<(), TestErr> {
    let result = scan.scan_leaf(asset);
    if !matches!(result, ScanResult::NotMine) {
        return Err(TestErr::Foreign);
    }
    assert_eq!(result.recv_report().status, ReceiveStatus::NotMine);
    assert_eq!(result.recv_report().reject, Some(ReceiveReject::NotMine));
    assert_eq!(result.recv_report().next, ReceiveNext::ReportOnly);
    Ok(())
}

fn mk_case() -> Result<SendCase, TestErr> {
    let bob = mk_keys(0x11)?;
    let carol = mk_keys(0x22)?;
    let live = derive_view_secret_key(&mk_secret(0x11)?)?;
    assert_eq!(bob.reveal_view_sk().as_bytes(), live.as_bytes());
    let card = bob.export_receiver_card()?;
    let mut sender = SenderWallet::new([0x33; 32]);
    let amount = 5_000;
    let aid = asset_from_dev_cfg("z00z", 0, amount)
        .map_err(|_| TestErr::Asset)?
        .definition
        .id;
    let out = build_tx_output_unchecked(&card, None, &mut sender, &[0x44; 32], 0, amount, &aid)?;
    let asset = mk_asset(amount, &out)?;

    Ok(SendCase {
        bob,
        carol,
        asset,
        aid,
        amount,
    })
}

fn run_case(case: &SendCase) {
    must_mine(
        &StealthOutputScanner::from_keys(&case.bob),
        &case.asset,
        case.aid,
        case.amount,
    )
    .expect("owned scan");
    must_not_mine(&StealthOutputScanner::from_keys(&case.carol), &case.asset)
        .expect("foreign scan");
}

#[test]
fn test_e2e_send_scan() {
    let case = mk_case().expect("phase 1 fixture");
    run_case(&case);
}
