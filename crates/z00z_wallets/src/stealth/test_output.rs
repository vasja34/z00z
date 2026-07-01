use super::{
    build_card_output_serial_checked, build_card_stealth_output_validated, build_output_core,
    build_tx_output_serial_unchecked, build_tx_output_unchecked, build_tx_stealth_output_validated,
    compute_owner_tag, constant_time_eq, create_owner_tag_sender, decode_public_key,
    derive_r_hedged, handle_tag_mismatch, m1_owner_tag_check, validate_stealth_leaf_fields,
    verify_owner_tag, verify_owner_tag_with_req, verify_owner_two_factor, BuildCheck, SenderWallet,
};
use crate::key::{ReceiverKeys, ReceiverSecret};
use crate::receiver::{PaymentRequest, PinEntry, PinnedReceiverCards, ReceiverCard, TrustLevel};
use crate::stealth::crypto::ephemeral::generate_r_hedged;
use crate::stealth::ecdh::{
    compute_dh_receiver, compute_dh_sender, compute_r_pub, decode_r_pub, encode_r_pub,
};
use crate::stealth::kdf::{derive_k_dh, derive_s_out};
use crate::stealth::{validate_output_self, SenderValidationCtx, StealthError, TagMode};
use std::sync::Arc;
use z00z_crypto::{create_range_proof, Hidden, ZkPackEncrypted};
use z00z_storage::settlement::TerminalLeaf;

fn verify_ownership(
    receiver_keys: &ReceiverKeys,
    r_pub: &[u8; 32],
    owner_tag: &[u8; 32],
) -> Result<bool, StealthError> {
    verify_owner_tag(receiver_keys, r_pub, owner_tag)
}

fn receiver_card(receiver_keys: &ReceiverKeys) -> ReceiverCard {
    receiver_keys.export_receiver_card().expect("signed card")
}

fn payment_request(card: &ReceiverCard, receiver_keys: &ReceiverKeys) -> PaymentRequest {
    let mut request = PaymentRequest {
        version: 1,
        owner_handle: card.owner_handle,
        view_pk: card.view_pk,
        identity_pk: card.identity_pk,
        req_id: [42u8; 32],
        chain_id: 1,
        amount: Some(777),
        expiry: u64::MAX,
        metadata: None,
        signature: [0u8; 64],
    };
    request
        .sign(receiver_keys.reveal_identity_sk())
        .expect("sign request");
    request
}

fn pinned_card_pins(card: &ReceiverCard) -> PinnedReceiverCards {
    PinnedReceiverCards::from_pairs(vec![(
        card.owner_handle,
        PinEntry {
            view_pk: card.view_pk,
            identity_pk: card.identity_pk,
            directory_id: None,
            first_seen: 0,
            trust_level: TrustLevel::Pinned,
        },
    )])
}

#[test]
fn test_unchecked_card_builds() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let signed = receiver_keys.export_receiver_card().expect("signed card");

    let unchecked = ReceiverCard {
        signature: [0u8; 64],
        ..signed.clone()
    };

    assert!(unchecked.verify().is_err());

    let mut sender_wallet = SenderWallet::new([0x44u8; 32]);
    let output = build_tx_output_unchecked(
        &unchecked,
        None,
        &mut sender_wallet,
        &[0x22u8; 32],
        0,
        77,
        &[0x11u8; 32],
    )
    .expect("unchecked receiver card still reaches builder");

    assert_ne!(output.owner_tag, [0u8; 32]);
}

#[test]
fn test_serial_aware_serial_id() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let mut sender_wallet = SenderWallet::new([0x55u8; 32]);

    let zero_serial = build_tx_output_serial_unchecked(
        &card,
        None,
        &mut sender_wallet,
        &[0x22u8; 32],
        0,
        77,
        &[0x11u8; 32],
        0,
    );
    assert!(
        zero_serial.is_ok(),
        "serial-aware raw sender seam must keep the zero serial lane available for live Stage 1 assets"
    );

    let too_large_serial = build_tx_output_serial_unchecked(
        &card,
        None,
        &mut sender_wallet,
        &[0x22u8; 32],
        0,
        77,
        &[0x11u8; 32],
        50_001,
    );
    assert!(matches!(
        too_large_serial,
        Err(StealthError::InvalidStealthInput)
    ));
}

fn build_fixed_sender_output_core(card: &ReceiverCard, serial_id: u32) -> TerminalLeaf {
    let r = derive_r_hedged(&[9u8; 32], &[8u8; 32], &[7u8; 32], 0).expect("fixed r");
    let r_pub = compute_r_pub(&r).expect("r pub");
    let r_pub_bytes = encode_r_pub(&r_pub);
    let view_pk = decode_public_key(&card.view_pk).expect("view pk");
    let dh = compute_dh_sender(&r, &view_pk).expect("dh");
    let k_dh = derive_k_dh(&dh);
    let blinding = derive_r_hedged(&[6u8; 32], &[5u8; 32], &[4u8; 32], 1).expect("blinding");
    let hidden_blinding = Hidden::hide(blinding);
    let range_proof = create_range_proof(
        77,
        hidden_blinding.reveal(),
        z00z_crypto::RANGE_PROOF_BITS,
        z00z_crypto::MIN_VALUE_PROMISE,
    )
    .expect("range proof");
    let s_out = derive_s_out(&k_dh, &r_pub_bytes, serial_id);

    build_output_core(
        &k_dh,
        &r_pub_bytes,
        &card.owner_handle,
        77,
        serial_id,
        s_out,
        hidden_blinding.reveal(),
        range_proof,
    )
    .expect("fixed-input leaf")
}

#[test]
fn test_serial_id_are_fixed() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let serial_7 = build_fixed_sender_output_core(&card, 7);
    let serial_29 = build_fixed_sender_output_core(&card, 29);

    assert_ne!(
        serial_7.asset_id, serial_29.asset_id,
        "asset_id derivation must change when only serial_id changes under fixed sender inputs"
    );
    assert_ne!(
        serial_7.enc_pack, serial_29.enc_pack,
        "encrypted payload binding must change when only serial_id changes under fixed sender inputs"
    );
    assert_ne!(
        serial_7.tag16, serial_29.tag16,
        "tag16 binding must change when only serial_id changes under fixed sender inputs"
    );
}

#[test]
fn test_public_serial_validation_ctx() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let asset_id = [0x11u8; 32];
    let mut sender_wallet = SenderWallet::new([0x55u8; 32]);

    let output = build_tx_output_serial_unchecked(
        &card,
        None,
        &mut sender_wallet,
        &[0x22u8; 32],
        0,
        77,
        &asset_id,
        7,
    )
    .expect("serial-aware output");

    let r_pub = decode_r_pub(&output.r_pub).expect("decode r_pub");
    let dh = compute_dh_receiver(receiver_keys.reveal_view_sk(), &r_pub).expect("receiver dh");
    let ctx = SenderValidationCtx {
        k_dh: derive_k_dh(&dh),
        owner_handle: card.owner_handle,
        asset_id,
        serial_id: 7,
        tag_mode: TagMode::CardBound,
    };

    validate_output_self(&output, &ctx, 77).expect("matching serial validation context");

    let mut wrong_serial_ctx = ctx;
    wrong_serial_ctx.serial_id = 29;

    assert!(
        matches!(
            validate_output_self(&output, &wrong_serial_ctx, 77),
            Err(StealthError::InvalidStealthInput)
        ),
        "public serial-aware raw sender seam must bind the output to the requested serial_id"
    );
}

#[test]
fn test_owner_tag_deterministic() {
    let owner_handle = [1u8; 32];
    let k_dh = [2u8; 32];
    let first = compute_owner_tag(&owner_handle, &k_dh);
    let second = compute_owner_tag(&owner_handle, &k_dh);
    assert_eq!(first, second);
}

#[test]
fn test_eq_owner_tag() {
    let owner_handle = [14u8; 32];
    let k_dh = [15u8; 32];

    let tag = compute_owner_tag(&owner_handle, &k_dh);
    let explicit = z00z_crypto::kdf::compute_owner_tag(&owner_handle, &k_dh);
    assert_eq!(tag, explicit);
}

#[test]
fn test_owner_tag_computation() {
    let owner_handle = [41u8; 32];
    let k_dh = [42u8; 32];
    let first = compute_owner_tag(&owner_handle, &k_dh);
    let second = compute_owner_tag(&owner_handle, &k_dh);
    assert_eq!(first, second);
}

#[test]
fn test_create_owner_tag_sender() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");

    let sender_salt = [55u8; 32];
    let tx_digest = [56u8; 32];
    let r = generate_r_hedged(&sender_salt, &tx_digest, 1).expect("r");

    let owner_tag =
        create_owner_tag_sender(&receiver_keys.owner_handle, &receiver_keys.view_pk, &r)
            .expect("owner_tag");
    assert_ne!(owner_tag, [0u8; 32]);
}

#[test]
fn test_owner_tag_unlinkability() {
    let owner_handle = [3u8; 32];
    let left = compute_owner_tag(&owner_handle, &[4u8; 32]);
    let right = compute_owner_tag(&owner_handle, &[5u8; 32]);
    assert_ne!(left, right);
}

#[test]
fn test_m1_check_success() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");

    let sender_salt = [9u8; 32];
    let tx_digest = [8u8; 32];
    let r = generate_r_hedged(&sender_salt, &tx_digest, 0).expect("r");
    let r_pub = compute_r_pub(&r).expect("r pub");
    let r_pub_bytes = encode_r_pub(&r_pub);

    let dh = compute_dh_sender(&r, &receiver_keys.view_pk).expect("dh");
    let owner_tag = compute_owner_tag(&receiver_keys.owner_handle, &derive_k_dh(&dh));

    let ok = verify_owner_tag(&receiver_keys, &r_pub_bytes, &owner_tag).expect("verify");
    assert!(ok);

    let ownership = verify_ownership(&receiver_keys, &r_pub_bytes, &owner_tag).expect("ownership");
    assert_eq!(ok, ownership);
}

#[test]
fn test_owner_tag_verification_success() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");

    let sender_salt = [43u8; 32];
    let tx_digest = [44u8; 32];
    let r = generate_r_hedged(&sender_salt, &tx_digest, 0).expect("r");
    let r_pub = compute_r_pub(&r).expect("r pub");
    let r_pub_bytes = encode_r_pub(&r_pub);

    let dh = compute_dh_sender(&r, &receiver_keys.view_pk).expect("dh");
    let owner_tag = compute_owner_tag(&receiver_keys.owner_handle, &derive_k_dh(&dh));
    let ok = verify_owner_tag(&receiver_keys, &r_pub_bytes, &owner_tag).expect("verify");
    assert!(ok);
}

#[test]
fn test_m1_wrong_owner() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");

    let sender_salt = [7u8; 32];
    let tx_digest = [6u8; 32];
    let r = generate_r_hedged(&sender_salt, &tx_digest, 1).expect("r");
    let r_pub = compute_r_pub(&r).expect("r pub");
    let r_pub_bytes = encode_r_pub(&r_pub);

    let dh = compute_dh_sender(&r, &receiver_keys.view_pk).expect("dh");
    let mut wrong_handle = receiver_keys.owner_handle;
    wrong_handle[0] ^= 1;
    let owner_tag = compute_owner_tag(&wrong_handle, &derive_k_dh(&dh));

    let ok = verify_owner_tag(&receiver_keys, &r_pub_bytes, &owner_tag).expect("verify");
    assert!(!ok);
}

#[test]
fn test_verification_failure_wrong_handle() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");

    let sender_salt = [45u8; 32];
    let tx_digest = [46u8; 32];
    let r = generate_r_hedged(&sender_salt, &tx_digest, 1).expect("r");
    let r_pub = compute_r_pub(&r).expect("r pub");
    let r_pub_bytes = encode_r_pub(&r_pub);

    let dh = compute_dh_sender(&r, &receiver_keys.view_pk).expect("dh");
    let mut wrong_handle = receiver_keys.owner_handle;
    wrong_handle[1] ^= 1;
    let owner_tag = compute_owner_tag(&wrong_handle, &derive_k_dh(&dh));

    let ok = verify_owner_tag(&receiver_keys, &r_pub_bytes, &owner_tag).expect("verify");
    assert!(!ok);
}

#[test]
fn test_validate_stealth_leaf_fields() {
    let mut asset_ok =
        z00z_core::genesis::asset_std::asset_from_dev_cfg("z00z", 0, 100).expect("std asset");
    let mut definition = (*asset_ok.definition).clone();
    definition.id = [77u8; 32];
    asset_ok.definition = Arc::new(definition);
    asset_ok.r_pub = Some([1u8; 32]);
    asset_ok.owner_tag = Some([2u8; 32]);
    asset_ok.enc_pack = Some(ZkPackEncrypted {
        version: 1,
        ciphertext: vec![9u8; 8],
        tag: [0u8; 16],
    });

    assert!(validate_stealth_leaf_fields(&asset_ok).is_ok());

    let mut asset_bad =
        z00z_core::genesis::asset_std::asset_from_dev_cfg("z00z", 0, 100).expect("std asset");
    let mut definition_bad = (*asset_bad.definition).clone();
    definition_bad.id = [77u8; 32];
    asset_bad.definition = Arc::new(definition_bad);
    asset_bad.r_pub = Some([1u8; 32]);
    asset_bad.owner_tag = Some([2u8; 32]);
    asset_bad.enc_pack = None;

    let result = validate_stealth_leaf_fields(&asset_bad);
    assert!(matches!(result, Err(StealthError::InvalidStealthInput)));
}

#[test]
fn test_card_validated_build_ok() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let mut pins = pinned_card_pins(&card);
    let mut sender_wallet = SenderWallet::new([151u8; 32]);

    let output = build_card_stealth_output_validated(
        &card,
        BuildCheck {
            pins: &mut pins,
            chain_id: 1,
        },
        &mut sender_wallet,
        &[152u8; 32],
        21,
        777,
        &[153u8; 32],
    )
    .expect("validated card output");

    assert!(output.tag16.is_some());
    assert!(verify_owner_tag(&receiver_keys, &output.r_pub, &output.owner_tag).expect("verify"));
}

#[test]
fn test_validated_serial_build_ok() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let mut pins = pinned_card_pins(&card);
    let mut sender_wallet = SenderWallet::new([161u8; 32]);

    let output = build_card_output_serial_checked(
        &card,
        BuildCheck {
            pins: &mut pins,
            chain_id: 1,
        },
        &mut sender_wallet,
        &[162u8; 32],
        23,
        999,
        &[163u8; 32],
        7,
    )
    .expect("validated serial card output");

    assert!(output.tag16.is_some());
    assert!(verify_owner_tag(&receiver_keys, &output.r_pub, &output.owner_tag).expect("verify"));
}

#[test]
fn test_card_validated_bad_signature() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let mut bad_card = card.clone();
    bad_card.signature[0] ^= 1;
    let mut pins = pinned_card_pins(&card);
    let mut sender_wallet = SenderWallet::new([154u8; 32]);

    let err = build_card_stealth_output_validated(
        &bad_card,
        BuildCheck {
            pins: &mut pins,
            chain_id: 1,
        },
        &mut sender_wallet,
        &[155u8; 32],
        22,
        888,
        &[156u8; 32],
    )
    .expect_err("bad signature must reject");

    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_constant_time_comparison() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");

    let sender_salt = [5u8; 32];
    let tx_digest = [4u8; 32];
    let r = generate_r_hedged(&sender_salt, &tx_digest, 2).expect("r");
    let r_pub = compute_r_pub(&r).expect("r pub");
    let r_pub_bytes = encode_r_pub(&r_pub);

    let dh = compute_dh_sender(&r, &receiver_keys.view_pk).expect("dh");
    let mut owner_tag = compute_owner_tag(&receiver_keys.owner_handle, &derive_k_dh(&dh));

    let ok = verify_owner_tag(&receiver_keys, &r_pub_bytes, &owner_tag).expect("verify");
    assert!(ok);

    owner_tag[31] ^= 1;
    let bad = verify_owner_tag(&receiver_keys, &r_pub_bytes, &owner_tag).expect("verify");
    assert!(!bad);
}

#[test]
fn test_identity_point_rejection() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let err = verify_owner_tag(&receiver_keys, &[0u8; 32], &[1u8; 32]).expect_err("err");
    assert_eq!(err, StealthError::IdentityPointRejected);
}

#[test]
fn test_build_output_with_req() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let request = payment_request(&card, &receiver_keys);

    let mut sender_wallet = SenderWallet::new([1u8; 32]);
    let output = build_tx_output_unchecked(
        &card,
        Some(&request),
        &mut sender_wallet,
        &[2u8; 32],
        7,
        777,
        &[3u8; 32],
    )
    .expect("output");

    assert_eq!(output.r_pub.len(), 32);
    assert_eq!(output.owner_tag.len(), 32);
    assert!(output.tag16.is_some());
    assert!(!output.enc_pack.ciphertext.is_empty());
}

#[test]
fn test_build_output_req_sig() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let mut request = payment_request(&card, &receiver_keys);
    request.signature[0] ^= 0x01;

    let mut sender_wallet = SenderWallet::new([21u8; 32]);
    let err = build_tx_output_unchecked(
        &card,
        Some(&request),
        &mut sender_wallet,
        &[22u8; 32],
        7,
        777,
        &[23u8; 32],
    )
    .expect_err("invalid request signature must reject output build");

    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_build_output_no_req() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);

    let mut sender_wallet = SenderWallet::new([4u8; 32]);
    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut sender_wallet,
        &[5u8; 32],
        11,
        123,
        &[6u8; 32],
    )
    .expect("output");

    assert!(output.tag16.is_some());
    assert!(!output.enc_pack.ciphertext.is_empty());
}

#[test]
fn test_multi_output_unlinkability() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let mut sender_wallet = SenderWallet::new([47u8; 32]);

    let first = build_tx_output_unchecked(
        &card,
        None,
        &mut sender_wallet,
        &[48u8; 32],
        1,
        10,
        &[49u8; 32],
    )
    .expect("first");

    let second = build_tx_output_unchecked(
        &card,
        None,
        &mut sender_wallet,
        &[48u8; 32],
        2,
        10,
        &[49u8; 32],
    )
    .expect("second");

    assert_ne!(first.r_pub, second.r_pub);
    assert_ne!(first.owner_tag, second.owner_tag);
}

#[test]
fn test_output_verify_ok() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);

    let mut sender_wallet = SenderWallet::new([7u8; 32]);
    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut sender_wallet,
        &[8u8; 32],
        19,
        50,
        &[9u8; 32],
    )
    .expect("output");

    let ok = verify_owner_tag(&receiver_keys, &output.r_pub, &output.owner_tag).expect("verify");
    assert!(ok);
}

#[test]
fn test_output_verify_fail() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);

    let wrong_secret = ReceiverSecret::generate().expect("receiver secret");
    let wrong_keys = ReceiverKeys::from_receiver_secret(wrong_secret).expect("keys");

    let mut sender_wallet = SenderWallet::new([10u8; 32]);
    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut sender_wallet,
        &[11u8; 32],
        20,
        51,
        &[12u8; 32],
    )
    .expect("output");

    let ok = verify_owner_tag(&wrong_keys, &output.r_pub, &output.owner_tag).expect("verify");
    assert!(!ok);
}

#[test]
fn test_sender_cannot_spend() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);

    let mut sender_wallet = SenderWallet::new([61u8; 32]);
    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut sender_wallet,
        &[62u8; 32],
        33,
        500,
        &[63u8; 32],
    )
    .expect("output");

    let r_pub_decoded = decode_r_pub(&output.r_pub).expect("r pub");
    let dh = compute_dh_receiver(receiver_keys.reveal_view_sk(), &r_pub_decoded).expect("dh");
    let k_dh = derive_k_dh(&dh);
    let s_out = super::derive_s_out(&k_dh, &output.r_pub, 0);

    let owner_ok = verify_owner_two_factor(
        receiver_keys.reveal_receiver_secret(),
        &output.r_pub,
        &output.owner_tag,
        &s_out,
        0,
    )
    .expect("receiver verify");
    assert!(owner_ok);

    let sender_material = ReceiverSecret::from_bytes(sender_wallet.secret_salt)
        .expect("sender material as receiver secret");
    let can_spend = verify_owner_two_factor(
        &sender_material,
        &output.r_pub,
        &output.owner_tag,
        &s_out,
        0,
    )
    .expect("verify");
    assert!(!can_spend);
}

#[test]
fn test_m1_req_path_ok() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let request = payment_request(&card, &receiver_keys);
    let mut sender_wallet = SenderWallet::new([23u8; 32]);

    let output = build_tx_output_unchecked(
        &card,
        Some(&request),
        &mut sender_wallet,
        &[24u8; 32],
        3,
        777,
        &[25u8; 32],
    )
    .expect("output");

    let plain = verify_owner_tag(&receiver_keys, &output.r_pub, &output.owner_tag).expect("verify");
    assert!(!plain);

    let bound = verify_owner_tag_with_req(
        &receiver_keys,
        &output.r_pub,
        &output.owner_tag,
        Some(&request.req_id),
    )
    .expect("verify");
    assert!(bound);
}

#[test]
fn test_owner_tag_flow_e2e() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let request = payment_request(&card, &receiver_keys);
    let mut sender_wallet = SenderWallet::new([50u8; 32]);

    let output = build_tx_output_unchecked(
        &card,
        Some(&request),
        &mut sender_wallet,
        &[51u8; 32],
        5,
        777,
        &[52u8; 32],
    )
    .expect("output");

    let verified = verify_owner_tag_with_req(
        &receiver_keys,
        &output.r_pub,
        &output.owner_tag,
        Some(&request.req_id),
    )
    .expect("verify");
    assert!(verified);
}

#[test]
fn test_req_amount_mismatch() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let card = receiver_card(&receiver_keys);
    let request = payment_request(&card, &receiver_keys);
    let mut sender_wallet = SenderWallet::new([26u8; 32]);

    let err = build_tx_output_unchecked(
        &card,
        Some(&request),
        &mut sender_wallet,
        &[27u8; 32],
        4,
        778,
        &[28u8; 32],
    )
    .expect_err("mismatch");

    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[path = "test_output_edge_cases.rs"]
mod extra;
