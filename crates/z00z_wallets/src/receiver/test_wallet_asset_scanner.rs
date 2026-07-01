use z00z_core::Asset;

use super::{
    ReceiveNext, ReceiveReject, ReceiveStatus, ScanChunk, ScanResult, ScanStrategy,
    StealthOutputScanner, Tag16Context,
};
use crate::db::ScanStatePayload;
use crate::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::ReceiverCard,
    stealth::{build_tx_output_unchecked, SenderWallet},
};

#[test]
fn test_scan_falls_back() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let mut sender_wallet = SenderWallet::new([44u8; 32]);
    let asset_id = [55u8; 32];
    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut sender_wallet,
        &[45u8; 32],
        0,
        77,
        &asset_id,
    )
    .expect("output");

    let mut asset = make_asset(77, asset_id);
    asset.r_pub = Some(output.r_pub);
    asset.owner_tag = Some(output.owner_tag);
    asset.commitment = z00z_crypto::Commitment::from_bytes(&output.c_amount)
        .expect("commitment")
        .0;
    asset.enc_pack = Some(output.enc_pack);
    asset.tag16 = output.tag16;

    let mut scanner = StealthOutputScanner::from_keys(&receiver_keys);
    scanner.add_tag_context(
        output.tag16.expect("tag16") ^ 1,
        Tag16Context {
            k_dh: [9u8; 32],
            req_id: None,
        },
    );

    assert!(matches!(scanner.scan_leaf(&asset), ScanResult::Mine { .. }));
}

#[test]
fn test_scan_rejects_partial() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let mut asset = make_asset(78, [56u8; 32]);
    asset.r_pub = Some([1u8; 32]);
    asset.owner_tag = Some([2u8; 32]);

    let scanner = StealthOutputScanner::from_keys(&receiver_keys);
    assert!(matches!(scanner.scan_leaf(&asset), ScanResult::NotMine));
}

#[test]
fn test_ignores_cache_false_hit() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let mut sender_wallet = SenderWallet::new([67u8; 32]);
    let asset_id = [68u8; 32];
    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut sender_wallet,
        &[69u8; 32],
        0,
        89,
        &asset_id,
    )
    .expect("output");

    let mut asset = make_asset(89, asset_id);
    asset.r_pub = Some(output.r_pub);
    asset.owner_tag = Some(output.owner_tag);
    asset.commitment = z00z_crypto::Commitment::from_bytes(&output.c_amount)
        .expect("commitment")
        .0;
    asset.enc_pack = Some(output.enc_pack);
    asset.tag16 = output.tag16;

    let mut scanner = StealthOutputScanner::from_keys(&receiver_keys);
    scanner.add_tag_context(
        output.tag16.expect("tag16"),
        Tag16Context {
            k_dh: [0xAB; 32],
            req_id: None,
        },
    );

    assert!(matches!(scanner.scan_leaf(&asset), ScanResult::Mine { .. }));
}

#[test]
fn test_recv_status_map() {
    let wallet_output = super::WalletStealthOutput {
        asset_id: [0u8; 32],
        serial_id: 0,
        pack_version: z00z_core::assets::AssetPackVersion::Basic,
        amount: 1,
        asset_secret: super::WalletReveal::unavailable(),
        blinding: super::WalletReveal::unavailable(),
        memo: None,
        r_pub: [0u8; 32],
        owner_tag: [0u8; 32],
        decrypted_at: 0,
    };

    assert_eq!(
        ScanResult::Mine {
            wallet_output: Box::new(wallet_output),
        }
        .recv_status(),
        super::ReceiveStatus::Detected
    );
    assert_eq!(
        ScanResult::MaybeMine {
            tag16_match: true,
            m1_failed: false
        }
        .recv_status(),
        super::ReceiveStatus::InvalidProof
    );
    assert_eq!(
        ScanResult::NotMine.recv_status(),
        super::ReceiveStatus::NotMine
    );
}

#[test]
fn test_recv_status_code_map() {
    assert_eq!(ReceiveStatus::Detected.rpc_code(), "RECEIVE_DETECTED");
    assert_eq!(
        ReceiveStatus::InvalidProof.rpc_code(),
        "RECEIVE_INVALID_PROOF"
    );
    assert_eq!(ReceiveStatus::NotMine.rpc_code(), "RECEIVE_NOT_MINE");
}

#[test]
fn test_recv_report_map() {
    let wallet_output = super::WalletStealthOutput {
        asset_id: [0u8; 32],
        serial_id: 0,
        pack_version: z00z_core::assets::AssetPackVersion::Basic,
        amount: 1,
        asset_secret: super::WalletReveal::unavailable(),
        blinding: super::WalletReveal::unavailable(),
        memo: None,
        r_pub: [0u8; 32],
        owner_tag: [0u8; 32],
        decrypted_at: 0,
    };

    assert_eq!(
        ScanResult::Mine {
            wallet_output: Box::new(wallet_output),
        }
        .recv_report(),
        super::ReceiveReport {
            status: ReceiveStatus::Detected,
            reject: None,
            next: ReceiveNext::ReportOnly,
        }
    );
    assert_eq!(
        ScanResult::MaybeMine {
            tag16_match: true,
            m1_failed: true
        }
        .recv_report(),
        super::ReceiveReport {
            status: ReceiveStatus::InvalidProof,
            reject: Some(ReceiveReject::InvalidProof),
            next: ReceiveNext::ReportOnly,
        }
    );
    assert_eq!(
        ScanResult::NotMine.recv_report(),
        super::ReceiveReport {
            status: ReceiveStatus::NotMine,
            reject: Some(ReceiveReject::NotMine),
            next: ReceiveNext::ReportOnly,
        }
    );
}

#[test]
fn test_recv_reject_map() {
    assert_eq!(ReceiveReject::NotMine.recv_status(), ReceiveStatus::NotMine);
    assert_eq!(
        ReceiveReject::InvalidProof.recv_status(),
        ReceiveStatus::InvalidProof
    );
    assert_eq!(
        ReceiveReject::InvalidInput.recv_status(),
        ReceiveStatus::InvalidProof
    );
    assert_eq!(
        ReceiveReject::RuntimeFail.recv_status(),
        ReceiveStatus::InvalidProof
    );
    assert_eq!(ReceiveReject::NotMine.rpc_code(), "RECEIVE_NOT_MINE");
    assert_eq!(
        ReceiveReject::InvalidProof.rpc_code(),
        "RECEIVE_INVALID_PROOF"
    );
    assert_eq!(
        ReceiveReject::InvalidInput.rpc_code(),
        "RECEIVE_INVALID_INPUT"
    );
    assert_eq!(
        ReceiveReject::RuntimeFail.rpc_code(),
        "RECEIVE_INVALID_PROOF"
    );
    assert!(!ReceiveReject::NotMine.is_alerting());
    assert!(ReceiveReject::InvalidProof.is_alerting());
    assert!(ReceiveReject::InvalidInput.is_alerting());
    assert!(ReceiveReject::RuntimeFail.is_alerting());
}

#[test]
fn test_scan_skips_bad_ctx() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let mut sender_wallet = SenderWallet::new([70u8; 32]);
    let asset_id = [71u8; 32];
    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut sender_wallet,
        &[72u8; 32],
        0,
        90,
        &asset_id,
    )
    .expect("output");

    let mut asset = make_asset(90, asset_id);
    asset.r_pub = Some(output.r_pub);
    asset.owner_tag = Some(output.owner_tag);
    asset.commitment = z00z_crypto::Commitment::from_bytes(&output.c_amount)
        .expect("commitment")
        .0;
    asset.enc_pack = Some(output.enc_pack);
    asset.tag16 = output.tag16;

    let mut scanner = StealthOutputScanner::from_keys(&receiver_keys);
    scanner.materialize_complete_tag_contexts(vec![(
        output.tag16.expect("tag16"),
        Tag16Context {
            k_dh: [0x44; 32],
            req_id: None,
        },
    )]);

    let next = scanner.scan_leaf_tag_only(&asset);
    assert!(matches!(
        next,
        ScanResult::MaybeMine {
            tag16_match: true,
            m1_failed: true
        }
    ));
}

#[test]
fn test_background_strategy_complete_contexts() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let mut scanner = StealthOutputScanner::from_keys(&receiver_keys);

    for tag16 in 0..=10_000u16 {
        scanner.add_tag_context(
            tag16,
            Tag16Context {
                k_dh: [0x55; 32],
                req_id: None,
            },
        );
    }

    assert_ne!(
        scanner.background_scan_strategy(),
        ScanStrategy::TagFilterOnly
    );
}

#[test]
fn test_scan_range_resumes() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let scanner = StealthOutputScanner::from_keys(&receiver_keys);
    let chunk = ScanChunk {
        height: 12,
        hash: vec![1, 2, 3],
        leaves: Vec::new(),
    };
    let cursor = ScanStatePayload::new(0, Vec::new());

    let result = scanner
        .scan_range(&[chunk], Some(&cursor), Some(1))
        .expect("range");
    assert_eq!(result.stat.total_ckpt, 1);
}

fn make_asset(serial_id: u64, asset_id: [u8; 32]) -> Asset {
    let mut asset =
        z00z_core::genesis::asset_std::asset_from_dev_cfg("z00z", 0, serial_id).expect("std asset");
    asset.leaf_ad_id = Some(asset_id);
    asset
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
