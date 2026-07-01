#![cfg(not(target_arch = "wasm32"))]

use z00z_core::{
    assets::{AssetLeaf, AssetPackPlain, AssetPackPlainMemo, AssetPackVersion},
    genesis::asset_std::asset_from_dev_class,
    Asset, AssetClass,
};
use z00z_wallets::{
    build_tx_output_unchecked,
    key::{ReceiverKeys, ReceiverSecret},
    receiver::{receiver_scan_leaf, receiver_scan_report, DetectedAssetPack},
    receiver::{ReceiveStatus, ScanResult, StealthOutputScanner},
    stealth::ecdh::{compute_dh_receiver, decode_r_pub},
    stealth::kdf::{compute_leaf_ad, compute_tag16, derive_k_dh},
    stealth::zkpack::ZkPack,
    SenderWallet, WalletError,
};

const MEMO_SERIAL_ID: u32 = 1_000_000;

struct MemoCase {
    keys: ReceiverKeys,
    leaf: AssetLeaf,
    asset: Asset,
    memo: Vec<u8>,
    amount: u64,
}

fn make_keys(seed: u8) -> ReceiverKeys {
    let secret = ReceiverSecret::from_bytes([seed; 32]).expect("secret");
    ReceiverKeys::from_receiver_secret(secret).expect("keys")
}

fn build_memo_case(seed: u8, amount: u64, memo: &[u8]) -> MemoCase {
    let keys = make_keys(seed);
    let card = keys.export_receiver_card().expect("card");
    let asset_id = [seed; 32];
    let mut sender = SenderWallet::new([seed ^ 0x5A; 32]);
    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut sender,
        &[seed ^ 0x11; 32],
        0,
        amount,
        &asset_id,
    )
    .expect("output");

    let r_pub = decode_r_pub(&output.r_pub).expect("r_pub");
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &r_pub).expect("dh");
    let k_dh = derive_k_dh(&dh);
    let base_leaf_ad = compute_leaf_ad(
        &asset_id,
        0,
        &output.r_pub,
        &output.owner_tag,
        &output.c_amount,
    );
    let memo_leaf_ad = compute_leaf_ad(
        &asset_id,
        MEMO_SERIAL_ID,
        &output.r_pub,
        &output.owner_tag,
        &output.c_amount,
    );
    let base_bytes = ZkPack::decrypt(
        &k_dh,
        &base_leaf_ad,
        &output.r_pub,
        &asset_id,
        0,
        &output.enc_pack,
    )
    .expect("decrypt original pack");
    let base_pack = AssetPackPlain::decode_checked(&base_bytes).expect("base pack");
    let memo_pack = AssetPackPlainMemo {
        value: base_pack.value,
        blinding: base_pack.blinding,
        s_out: base_pack.s_out,
        memo: memo.to_vec(),
    };
    let enc_pack = ZkPack::encrypt(
        &k_dh,
        &memo_leaf_ad,
        &output.r_pub,
        &asset_id,
        MEMO_SERIAL_ID,
        &memo_pack.encode_checked().expect("memo bytes"),
    );

    let leaf = AssetLeaf {
        asset_id,
        serial_id: MEMO_SERIAL_ID,
        r_pub: output.r_pub,
        owner_tag: output.owner_tag,
        c_amount: output.c_amount,
        enc_pack,
        range_proof: Vec::new(),
        tag16: compute_tag16(&k_dh, &memo_leaf_ad),
    };
    let mut asset = asset_from_dev_class(AssetClass::Coin, 0, amount).expect("asset");
    asset.serial_id = leaf.serial_id;
    asset.leaf_ad_id = Some(leaf.asset_id);
    asset.commitment = z00z_crypto::Commitment::from_bytes(&leaf.c_amount)
        .expect("commitment")
        .0;
    asset.owner_pub = None;
    asset.owner_signature = None;
    asset.r_pub = Some(leaf.r_pub);
    asset.owner_tag = Some(leaf.owner_tag);
    asset.enc_pack = Some(leaf.enc_pack.clone());
    asset.tag16 = Some(leaf.tag16);
    asset.range_proof = Some(leaf.range_proof.clone());

    MemoCase {
        keys,
        leaf,
        asset,
        memo: memo.to_vec(),
        amount,
    }
}

#[test]
fn test_memo_leaf_owned_pack() {
    let case = build_memo_case(0x91, 777, b"phase35-memo");

    let pack = receiver_scan_leaf(&case.keys, &case.leaf)
        .expect("scan")
        .expect("owned pack");
    let report = receiver_scan_report(&case.keys, &case.leaf).expect("report");

    assert_eq!(pack.pack_version, AssetPackVersion::Memo);
    assert_eq!(pack.value, case.amount);
    assert_eq!(pack.memo, Some(case.memo.clone()));
    assert_eq!(report.status, ReceiveStatus::Detected);
}

#[test]
fn test_memo_runtime_memo_private() {
    let case = build_memo_case(0x92, 888, b"private-memo-note");

    let scan = StealthOutputScanner::from_keys(&case.keys).scan_leaf(&case.asset);
    let ScanResult::Mine { wallet_output } = scan else {
        panic!("expected Mine runtime result");
    };

    assert_eq!(wallet_output.pack_version, AssetPackVersion::Memo);
    assert_eq!(wallet_output.amount, case.amount);
    assert_eq!(wallet_output.asset_id, case.asset.asset_id());
    assert_eq!(wallet_output.memo, Some(case.memo.clone()));
    assert_eq!(wallet_output.r_pub, case.leaf.r_pub);
    assert_eq!(wallet_output.owner_tag, case.leaf.owner_tag);
}

#[test]
fn test_memo_leaf_memo_len() {
    let mut case = build_memo_case(0x93, 999, b"ok");

    let r_pub = decode_r_pub(&case.leaf.r_pub).expect("r_pub");
    let dh = compute_dh_receiver(case.keys.reveal_view_sk(), &r_pub).expect("dh");
    let k_dh = derive_k_dh(&dh);
    let leaf_ad = compute_leaf_ad(
        &case.leaf.asset_id,
        case.leaf.serial_id,
        &case.leaf.r_pub,
        &case.leaf.owner_tag,
        &case.leaf.c_amount,
    );

    let base = receiver_scan_leaf(&case.keys, &case.leaf)
        .expect("scan")
        .expect("owned pack");
    let mut malformed = base.opening_pack().to_bytes();
    malformed.extend_from_slice(&u16::MAX.to_le_bytes());

    case.leaf.enc_pack = ZkPack::encrypt(
        &k_dh,
        &leaf_ad,
        &case.leaf.r_pub,
        &case.leaf.asset_id,
        case.leaf.serial_id,
        &malformed,
    );

    let result = receiver_scan_leaf(&case.keys, &case.leaf);
    assert!(
        matches!(result, Err(WalletError::InvalidAssetPack("bad memo"))),
        "oversized memo payload must fail closed"
    );
}

#[test]
fn test_unknown_lane_fails() {
    let pack = DetectedAssetPack {
        pack_version: AssetPackVersion::Unknown,
        value: 17,
        blinding: [0u8; 32],
        s_out: [1u8; 32],
        memo: None,
    };

    assert_eq!(pack.to_bytes(), Err(z00z_core::assets::PackErr::BadVer));
}
