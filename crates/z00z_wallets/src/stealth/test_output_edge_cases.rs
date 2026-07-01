use super::*;
use crate::stealth::{SenderValidationCtx, TxStealthOutput};
use crate::tx::TxOutRole;
use std::sync::Arc;
use z00z_core::assets::AssetClass;
use z00z_core::assets::AssetPackPlain;
use z00z_crypto::{domains::AssetIdDomain, hash_zk::hash_zk, Hidden, Z00ZScalar};
use z00z_utils::rng::SystemRngProvider;

fn pinned_card_pins(card: &ReceiverCard) -> PinnedReceiverCards {
    PinnedReceiverCards::from_pairs(vec![(
        card.owner_handle,
        crate::receiver::PinEntry {
            view_pk: card.view_pk,
            identity_pk: card.identity_pk,
            directory_id: None,
            first_seen: 0,
            trust_level: crate::receiver::TrustLevel::Pinned,
        },
    )])
}

#[test]
fn test_handle_tag_mismatch() {
    let mut leaf =
        z00z_core::genesis::asset_std::asset_from_dev_cfg("z00z", 0, 1).expect("std asset");
    let mut definition = (*leaf.definition).clone();
    definition.id = [99u8; 32];
    leaf.definition = Arc::new(definition);

    leaf.owner_tag = Some([10u8; 32]);
    let mismatch = handle_tag_mismatch(&leaf, &[11u8; 32]);
    assert!(matches!(mismatch, crate::receiver::ScanResult::NotMine));

    let match_result = handle_tag_mismatch(&leaf, &[10u8; 32]);
    assert!(matches!(
        match_result,
        crate::receiver::ScanResult::MaybeMine {
            tag16_match: false,
            m1_failed: false
        }
    ));
}

#[test]
fn test_constant_time_eq() {
    assert!(constant_time_eq(&[1u8; 32], &[1u8; 32]));
    assert!(!constant_time_eq(&[1u8; 32], &[2u8; 32]));
}

#[test]
fn test_m1_owner_tag_check() {
    assert!(m1_owner_tag_check(&[3u8; 32], &[3u8; 32]));
    assert!(!m1_owner_tag_check(&[3u8; 32], &[4u8; 32]));
}

#[test]
fn test_cache_hit_retry() {
    let owner_handle = [71u8; 32];
    let mut sender_wallet = SenderWallet::with_cap([72u8; 32], 4);
    let first = z00z_crypto::Z00ZScalar::try_from_bytes([1u8; 32]).expect("first scalar");
    let second = z00z_crypto::Z00ZScalar::try_from_bytes([2u8; 32]).expect("second scalar");

    let selected = super::super::select_r_with(&mut sender_wallet, &owner_handle, |retry_index| {
        if retry_index == 0 {
            return Ok(
                z00z_crypto::Z00ZScalar::try_from_bytes(first.to_bytes()).expect("first scalar")
            );
        }

        Ok(z00z_crypto::Z00ZScalar::try_from_bytes(second.to_bytes()).expect("second scalar"))
    })
    .expect("selected");

    let first_bytes =
        super::super::encode_r_pub(&super::super::compute_r_pub(&first).expect("first r_pub"));
    let second_bytes =
        super::super::encode_r_pub(&super::super::compute_r_pub(&second).expect("second r_pub"));

    assert_eq!(selected.1, first_bytes);
    let retried = super::super::select_r_with(&mut sender_wallet, &owner_handle, |retry_index| {
        if retry_index == 0 {
            return Ok(
                z00z_crypto::Z00ZScalar::try_from_bytes(first.to_bytes()).expect("first scalar")
            );
        }

        Ok(z00z_crypto::Z00ZScalar::try_from_bytes(second.to_bytes()).expect("second scalar"))
    })
    .expect("retried");
    assert_eq!(retried.1, second_bytes);
}

#[test]
fn test_retry_ceiling_failure() {
    let owner_handle = [81u8; 32];
    let mut sender_wallet = SenderWallet::with_cap([82u8; 32], 4);
    let scalar = z00z_crypto::Z00ZScalar::try_from_bytes([3u8; 32]).expect("scalar");

    let first = super::super::select_r_with(&mut sender_wallet, &owner_handle, |_| {
        Ok(z00z_crypto::Z00ZScalar::try_from_bytes(scalar.to_bytes()).expect("scalar"))
    })
    .expect("first");
    let again = super::super::select_r_with(&mut sender_wallet, &owner_handle, |_| {
        Ok(z00z_crypto::Z00ZScalar::try_from_bytes(scalar.to_bytes()).expect("scalar"))
    });

    assert_ne!(first.1, [0u8; 32]);
    assert!(matches!(again, Err(StealthError::RetryLimitReached)));
}

#[test]
fn test_cache_evicts_oldest() {
    let owner_handle = [91u8; 32];
    let mut sender_wallet = SenderWallet::with_cap([92u8; 32], 1);
    let first = z00z_crypto::Z00ZScalar::try_from_bytes([4u8; 32]).expect("first scalar");
    let second = z00z_crypto::Z00ZScalar::try_from_bytes([5u8; 32]).expect("second scalar");

    let first_bytes = super::super::select_r_with(&mut sender_wallet, &owner_handle, |_| {
        Ok(z00z_crypto::Z00ZScalar::try_from_bytes(first.to_bytes()).expect("first scalar"))
    })
    .expect("first")
    .1;
    let second_bytes = super::super::select_r_with(&mut sender_wallet, &owner_handle, |_| {
        Ok(z00z_crypto::Z00ZScalar::try_from_bytes(second.to_bytes()).expect("second scalar"))
    })
    .expect("second")
    .1;
    let reused = super::super::select_r_with(&mut sender_wallet, &owner_handle, |_| {
        Ok(z00z_crypto::Z00ZScalar::try_from_bytes(first.to_bytes()).expect("first scalar"))
    })
    .expect("reused")
    .1;

    assert_ne!(first_bytes, second_bytes);
    assert_eq!(reused, first_bytes);
}

#[test]
fn test_zero_scalar_stays_error() {
    let owner_handle = [101u8; 32];
    let mut sender_wallet = SenderWallet::with_cap([102u8; 32], 2);

    let err = super::super::select_r_with(&mut sender_wallet, &owner_handle, |_| {
        Err(StealthError::ZeroScalarRejected)
    });

    assert!(matches!(err, Err(StealthError::ZeroScalarRejected)));
}

#[test]
fn test_same_tuple_no_reuse() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let mut sender_wallet = SenderWallet::new([111u8; 32]);
    let tx_digest = [112u8; 32];
    let asset_id = [113u8; 32];

    let first = build_tx_output_unchecked(
        &card,
        None,
        &mut sender_wallet,
        &tx_digest,
        7,
        10,
        &asset_id,
    )
    .expect("first");
    let second = build_tx_output_unchecked(
        &card,
        None,
        &mut sender_wallet,
        &tx_digest,
        7,
        10,
        &asset_id,
    )
    .expect("second");

    assert_ne!(first.r_pub, second.r_pub);
}

#[test]
fn test_bundle_serial_range_enforced() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let mut rng = SystemRngProvider.rng();

    let err = crate::stealth::build_output_bundle(
        "bob".to_string(),
        TxOutRole::Recipient,
        AssetClass::Coin,
        &card,
        10,
        0,
    )
    .expect_err("bundle must reject serials below range");
    assert!(err.contains("serial_id 0 out of range [1, 50000]"));

    let err = crate::stealth::build_output_bundle_with_rng(
        "bob".to_string(),
        TxOutRole::Recipient,
        AssetClass::Coin,
        &card,
        10,
        50_001,
        &mut rng,
    )
    .expect_err("bundle rng path must reject serials above range");
    assert!(err.contains("serial_id 50001 out of range [1, 50000]"));
}

fn make_light(amount: u64, seed: u8) -> (TxStealthOutput, SenderValidationCtx) {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let mut sender_wallet = SenderWallet::new([seed; 32]);

    super::super::build_output_ctx(
        &card,
        None,
        &mut sender_wallet,
        &[seed.wrapping_add(1); 32],
        u32::from(seed),
        amount,
        &[seed.wrapping_add(2); 32],
    )
    .expect("output")
}

#[test]
fn test_validate_output_self_ok() {
    let (output, ctx) = make_light(321, 121);
    validate_output_self(&output, &ctx, 321).expect("validated");
}

#[test]
fn test_output_self_bad_pack() {
    let (mut output, ctx) = make_light(654, 124);
    output.enc_pack.tag[0] ^= 1;

    let err = validate_output_self(&output, &ctx, 654).expect_err("tampered pack");
    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_output_self_bad_asset() {
    let (output, mut ctx) = make_light(100, 127);
    ctx.asset_id[0] ^= 1;

    let err = validate_output_self(&output, &ctx, 100).expect_err("wrong asset");
    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_output_self_bad_serial() {
    let (output, mut ctx) = make_light(101, 130);
    ctx.serial_id = 1;

    let err = validate_output_self(&output, &ctx, 101).expect_err("wrong serial");
    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_output_self_bad_owner() {
    let (output, mut ctx) = make_light(102, 133);
    ctx.owner_handle[0] ^= 1;

    let err = validate_output_self(&output, &ctx, 102).expect_err("wrong owner");
    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_output_self_bad_commit() {
    let (mut output, ctx) = make_light(103, 136);
    output.c_amount[0] ^= 1;

    let err = validate_output_self(&output, &ctx, 103).expect_err("wrong commitment");
    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_output_self_bad_tag16() {
    let (mut output, ctx) = make_light(104, 139);
    output.tag16 = output.tag16.map(|tag| tag ^ 1);

    let err = validate_output_self(&output, &ctx, 104).expect_err("wrong tag16");
    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_output_self_bad_mode() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let request = payment_request(&card, &receiver_keys);
    let mut sender_wallet = SenderWallet::new([142u8; 32]);

    let (output, mut ctx) = super::super::build_output_ctx(
        &card,
        Some(&request),
        &mut sender_wallet,
        &[143u8; 32],
        16,
        777,
        &[144u8; 32],
    )
    .expect("output");
    ctx.tag_mode = TagMode::CardBound;

    let err = validate_output_self(&output, &ctx, 777).expect_err("wrong mode");
    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_validated_build_bad_chain() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let request = payment_request(&card, &receiver_keys);
    let mut pins = PinnedReceiverCards::new();
    let mut sender_wallet = SenderWallet::new([145u8; 32]);

    let err = build_tx_stealth_output_validated(
        &card,
        Some(&request),
        BuildCheck {
            pins: &mut pins,
            chain_id: request.chain_id.wrapping_add(1),
        },
        &mut sender_wallet,
        &[146u8; 32],
        17,
        777,
        &[147u8; 32],
    )
    .expect_err("wrong chain must reject");

    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_validated_build_req_ok() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let request = payment_request(&card, &receiver_keys);
    let mut pins = PinnedReceiverCards::new();
    let _ = pins.verify_request_identity(&request.owner_handle, &request.identity_pk);

    let mut sender_wallet = SenderWallet::new([148u8; 32]);
    let output = build_tx_stealth_output_validated(
        &card,
        Some(&request),
        BuildCheck {
            pins: &mut pins,
            chain_id: request.chain_id,
        },
        &mut sender_wallet,
        &[149u8; 32],
        18,
        777,
        &[150u8; 32],
    )
    .expect("validated output");

    assert!(output.tag16.is_some());
}

#[test]
fn test_card_validated_stays_open() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let mut empty_pins = PinnedReceiverCards::new();
    let mut compat_wallet = SenderWallet::new([157u8; 32]);

    let compat = build_tx_stealth_output_validated(
        &card,
        None,
        BuildCheck {
            pins: &mut empty_pins,
            chain_id: 1,
        },
        &mut compat_wallet,
        &[158u8; 32],
        23,
        901,
        &[159u8; 32],
    )
    .expect("request-none compatibility path");
    assert!(compat.tag16.is_some());

    let mut missing_pins = PinnedReceiverCards::new();
    let mut card_wallet = SenderWallet::new([160u8; 32]);
    let err = build_card_stealth_output_validated(
        &card,
        BuildCheck {
            pins: &mut missing_pins,
            chain_id: 1,
        },
        &mut card_wallet,
        &[161u8; 32],
        24,
        901,
        &[162u8; 32],
    )
    .expect_err("card-only path must require pinned card approval");
    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_card_validated_view_point() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let mut bad_card = card.clone();
    bad_card.view_pk = [0u8; 32];
    let mut pins = pinned_card_pins(&card);
    let mut sender_wallet = SenderWallet::new([163u8; 32]);

    let err = build_card_stealth_output_validated(
        &bad_card,
        BuildCheck {
            pins: &mut pins,
            chain_id: 1,
        },
        &mut sender_wallet,
        &[164u8; 32],
        25,
        902,
        &[165u8; 32],
    )
    .expect_err("invalid view point must reject");

    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_card_validated_view_mismatch() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let mut wrong_view = card.view_pk;
    wrong_view[0] ^= 1;
    let mut pins = PinnedReceiverCards::from_pairs(vec![(
        card.owner_handle,
        crate::receiver::PinEntry {
            view_pk: wrong_view,
            identity_pk: card.identity_pk,
            directory_id: None,
            first_seen: 0,
            trust_level: crate::receiver::TrustLevel::Pinned,
        },
    )]);
    let mut sender_wallet = SenderWallet::new([166u8; 32]);

    let err = build_card_stealth_output_validated(
        &card,
        BuildCheck {
            pins: &mut pins,
            chain_id: 1,
        },
        &mut sender_wallet,
        &[167u8; 32],
        26,
        903,
        &[168u8; 32],
    )
    .expect_err("mismatched pinned view key must reject");

    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_card_validated_pinned_trust() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let mut pins = PinnedReceiverCards::from_pairs(vec![(
        card.owner_handle,
        crate::receiver::PinEntry {
            view_pk: card.view_pk,
            identity_pk: card.identity_pk,
            directory_id: None,
            first_seen: 0,
            trust_level: crate::receiver::TrustLevel::Tentative,
        },
    )]);
    let mut sender_wallet = SenderWallet::new([169u8; 32]);

    let err = build_card_stealth_output_validated(
        &card,
        BuildCheck {
            pins: &mut pins,
            chain_id: 1,
        },
        &mut sender_wallet,
        &[170u8; 32],
        27,
        904,
        &[171u8; 32],
    )
    .expect_err("tentative trust must reject");

    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_card_validated_identity_mismatch() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let mut wrong_identity = card.identity_pk;
    wrong_identity[0] ^= 1;
    let mut pins = PinnedReceiverCards::from_pairs(vec![(
        card.owner_handle,
        crate::receiver::PinEntry {
            view_pk: card.view_pk,
            identity_pk: wrong_identity,
            directory_id: None,
            first_seen: 0,
            trust_level: crate::receiver::TrustLevel::Pinned,
        },
    )]);
    let mut sender_wallet = SenderWallet::new([172u8; 32]);

    let err = build_card_stealth_output_validated(
        &card,
        BuildCheck {
            pins: &mut pins,
            chain_id: 1,
        },
        &mut sender_wallet,
        &[173u8; 32],
        28,
        905,
        &[174u8; 32],
    )
    .expect_err("mismatched pinned identity key must reject");

    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_pin_state_unchanged() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let request = payment_request(&card, &receiver_keys);
    let mut pins = PinnedReceiverCards::new();
    let mut sender_wallet = SenderWallet::new([154u8; 32]);

    let err = build_tx_stealth_output_validated(
        &card,
        Some(&request),
        BuildCheck {
            pins: &mut pins,
            chain_id: request.chain_id,
        },
        &mut sender_wallet,
        &[155u8; 32],
        20,
        777,
        &[156u8; 32],
    )
    .expect_err("new identity must still require confirmation");

    assert_eq!(err, StealthError::InvalidStealthInput);
    assert!(pins.is_empty());
}

#[test]
fn test_cross_path_matrix() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let mut sender_wallet = SenderWallet::new([151u8; 32]);

    let (light, ctx) = super::super::build_output_ctx(
        &card,
        None,
        &mut sender_wallet,
        &[152u8; 32],
        19,
        555,
        &[153u8; 32],
    )
    .expect("light");
    validate_output_self(&light, &ctx, 555).expect("light validated");

    let light_ad = crate::stealth::kdf::compute_leaf_ad(
        &ctx.asset_id,
        ctx.serial_id,
        &light.r_pub,
        &light.owner_tag,
        &light.c_amount,
    );
    let plain_bytes = crate::stealth::zkpack::ZkPack::decrypt(
        &ctx.k_dh,
        &light_ad,
        &light.r_pub,
        &ctx.asset_id,
        ctx.serial_id,
        &light.enc_pack,
    )
    .expect("plain bytes");
    let plain = AssetPackPlain::decode_checked(&plain_bytes).expect("plain");
    let full_serial_id = 1;
    let full_s_out = crate::stealth::kdf::derive_s_out(&ctx.k_dh, &light.r_pub, full_serial_id);
    let full = crate::stealth::build_stealth_leaf_with_blind(
        &ctx.k_dh,
        &light.r_pub,
        &ctx.owner_handle,
        plain.value,
        full_serial_id,
        full_s_out,
        &Hidden::hide(Z00ZScalar::try_from_bytes(plain.blinding).expect("blinding")),
    )
    .expect("full");
    let full_asset_id = hash_zk::<AssetIdDomain>("", &[&full_s_out]);

    assert_eq!(full.r_pub, light.r_pub);
    assert_eq!(full.owner_tag, light.owner_tag);
    assert_eq!(full.c_amount, light.c_amount);
    assert_eq!(full.serial_id, full_serial_id);
    assert_eq!(full.asset_id, full_asset_id);
    assert_ne!(full.asset_id, ctx.asset_id);
    assert_ne!(full.enc_pack, light.enc_pack);
    assert_ne!(full.tag16, light.tag16.expect("tag16"));
}
