use z00z_core::genesis::asset_std::{asset_from_dev_cfg, def_from_dev_cfg};
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::ReceiverCard,
    receiver::{ScanResult, StealthOutputScanner},
    stealth::{build_tx_output_unchecked, OwnerTag, SenderWallet},
};

const LIB_RS: &str = include_str!("../src/lib.rs");
const STEALTH_MOD: &str = include_str!("../src/stealth/mod.rs");

#[test]
fn unchecked_builder_noncanonical() {
    assert!(LIB_RS.contains("build_tx_output_unchecked"));
    assert!(STEALTH_MOD.contains("build_tx_output_unchecked"));
    assert!(!LIB_RS.contains(" build_tx_stealth_output,"));
    assert!(!STEALTH_MOD.contains(" build_tx_stealth_output,"));
}

#[test]
fn test_owner_tag_compute() {
    let owner = [1u8; 32];
    let k_dh = [2u8; 32];
    let tag = OwnerTag::compute(&owner, &k_dh);
    assert_eq!(tag.as_bytes().len(), 32);
}

#[test]
fn test_owner_tag_verify_ok() {
    let owner = [3u8; 32];
    let k_dh = [4u8; 32];
    let tag = OwnerTag::compute(&owner, &k_dh);
    assert!(tag.verify(tag.as_bytes()));
}

#[test]
fn test_owner_tag_verify_fail() {
    let k_dh = [5u8; 32];
    let tag_a = OwnerTag::compute(&[6u8; 32], &k_dh);
    let tag_b = OwnerTag::compute(&[7u8; 32], &k_dh);
    assert!(!tag_a.verify(tag_b.as_bytes()));
}

#[test]
fn test_owner_tag_flow() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);

    let asset_id = def_from_dev_cfg("z00z").expect("std def").id;
    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut SenderWallet::new([8u8; 32]),
        &[9u8; 32],
        0,
        1000,
        &asset_id,
    )
    .expect("output");

    let mut asset = make_test_asset(1000);
    asset.commitment = z00z_crypto::Commitment::from_bytes(&output.c_amount)
        .expect("commitment")
        .0;
    asset.r_pub = Some(output.r_pub);
    asset.owner_tag = Some(output.owner_tag);
    asset.enc_pack = Some(output.enc_pack);
    asset.tag16 = None;
    asset.leaf_ad_id = Some(asset.definition.id);

    let scanner = StealthOutputScanner::from_keys(&receiver_keys);
    let result = scanner.scan_leaf(&asset);
    assert!(matches!(result, ScanResult::Mine { .. }));
}

#[test]
fn test_multi_output_unlink() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);

    let out_a = build_tx_output_unchecked(
        &card,
        None,
        &mut SenderWallet::new([11u8; 32]),
        &[12u8; 32],
        0,
        1000,
        &[13u8; 32],
    )
    .expect("out_a");

    let out_b = build_tx_output_unchecked(
        &card,
        None,
        &mut SenderWallet::new([11u8; 32]),
        &[12u8; 32],
        1,
        2000,
        &[13u8; 32],
    )
    .expect("out_b");

    assert_ne!(out_a.owner_tag, out_b.owner_tag);
    assert_ne!(out_a.r_pub, out_b.r_pub);
}

fn receiver_card(keys: &ReceiverKeys) -> ReceiverCard {
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

fn make_test_asset(amount: u64) -> z00z_core::Asset {
    asset_from_dev_cfg("z00z", 0, amount).expect("std asset")
}
