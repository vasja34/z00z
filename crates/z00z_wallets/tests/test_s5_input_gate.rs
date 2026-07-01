use z00z_utils::time::{SystemTimeProvider, TimeProvider};
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::card::receiver_card_trust::PinCheckResult,
    receiver::{
        decode_card_compact, encode_card_compact, CardMetadata, PaymentRequest,
        PaymentRequestError, PinnedReceiverCards, ReceiverCard, ReceiverCardError, RequestParams,
        ValidatePaymentRequest, ValidateReceiverCard, ValidationOutcome, VerifyResult,
    },
    stealth::{
        build_tx_output_unchecked, build_tx_stealth_output_validated, BuildCheck, SenderWallet,
        StealthError,
    },
};

fn unix_now() -> u64 {
    SystemTimeProvider.compat_unix_timestamp()
}

fn make_keys() -> ReceiverKeys {
    let sec = ReceiverSecret::generate().expect("secret");
    ReceiverKeys::from_receiver_secret(sec).expect("keys")
}

fn make_card(keys: &ReceiverKeys) -> ReceiverCard {
    let card = keys.export_receiver_card().expect("card");
    card.verify().expect("verify card");
    card
}

fn make_req(keys: &ReceiverKeys, chain: u32) -> PaymentRequest {
    PaymentRequest::generate(
        keys,
        RequestParams {
            amount: Some(77),
            expiry_seconds: 600,
            memo: Some("phase1".to_string()),
            payment_id: Some([9u8; 16]),
        },
        chain,
    )
    .expect("request")
}

fn parse_req(req: &PaymentRequest) -> PaymentRequest {
    PaymentRequest::from_untrusted_bytes(&req.canonical_encoding()).expect("parse request")
}

fn check_req_base_errs(req: &PaymentRequest) {
    let bad_pk = [0xffu8; 32];

    let mut bad_ver = req.clone();
    bad_ver.version = 2;
    assert!(matches!(
        bad_ver.validate_all(&mut PinnedReceiverCards::new(), 7),
        Err(PaymentRequestError::UnsupportedVersion)
    ));

    let mut bad_chain = req.clone();
    bad_chain.chain_id = 8;
    assert!(matches!(
        bad_chain.validate_all(&mut PinnedReceiverCards::new(), 7),
        Err(PaymentRequestError::WrongChainId)
    ));

    let mut bad_exp = req.clone();
    bad_exp.expiry = unix_now().saturating_sub(1);
    assert!(matches!(
        bad_exp.validate_all(&mut PinnedReceiverCards::new(), 7),
        Err(PaymentRequestError::RequestExpired)
    ));

    let mut bad_view = req.clone();
    bad_view.view_pk = [0u8; 32];
    assert!(matches!(
        bad_view.validate_all(&mut PinnedReceiverCards::new(), 7),
        Err(PaymentRequestError::IdentityPoint)
    ));

    let mut bad_id = req.clone();
    bad_id.identity_pk = [0u8; 32];
    assert!(matches!(
        bad_id.validate_all(&mut PinnedReceiverCards::new(), 7),
        Err(PaymentRequestError::IdentityPoint)
    ));

    let mut bad_view_enc = req.clone();
    bad_view_enc.view_pk = bad_pk;
    assert!(matches!(
        bad_view_enc.validate_all(&mut PinnedReceiverCards::new(), 7),
        Err(PaymentRequestError::InvalidPublicKey)
    ));

    let mut bad_id_enc = req.clone();
    bad_id_enc.identity_pk = bad_pk;
    assert!(matches!(
        bad_id_enc.validate_all(&mut PinnedReceiverCards::new(), 7),
        Err(PaymentRequestError::InvalidPublicKey)
    ));
}

fn check_req_sig_errs(req: &PaymentRequest) {
    let mut bad_sig = req.clone();
    bad_sig.signature = [0u8; 64];
    assert!(matches!(
        bad_sig.validate_all(&mut PinnedReceiverCards::new(), 7),
        Err(PaymentRequestError::InvalidSignature)
    ));

    let mut bad_vfy = req.clone();
    bad_vfy.owner_handle[0] ^= 1;
    assert!(matches!(
        bad_vfy.validate_all(&mut PinnedReceiverCards::new(), 7),
        Err(PaymentRequestError::VerifyFailed)
    ));
}

fn check_req_pins(req: &PaymentRequest) {
    let mut pins = PinnedReceiverCards::new();

    let first = pins.verify_request_identity(&req.owner_handle, &req.identity_pk);
    assert_eq!(first, PinCheckResult::NewIdentity);

    let second = pins.verify_request_identity(&req.owner_handle, &req.identity_pk);
    assert_eq!(second, PinCheckResult::Verified);

    let mut other_id = req.identity_pk;
    other_id[0] ^= 1;
    let drift = pins.verify_request_identity(&req.owner_handle, &other_id);
    assert_eq!(drift, PinCheckResult::IdentityChanged);
}

fn check_req_trust(req: &PaymentRequest, pins: &mut PinnedReceiverCards) {
    let other = make_keys();
    let mut drift = parse_req(&make_req(&other, 7));
    drift.owner_handle = req.owner_handle;
    drift.sign(other.reveal_identity_sk()).expect("resign");
    let drift_out = drift.validate_all(pins, 7).expect("drift");
    assert_eq!(drift_out, ValidationOutcome::IdentityMismatch);

    let mut rev_pins = PinnedReceiverCards::new();
    let _ = req.validate_all(&mut rev_pins, 7).expect("pin once");
    let _ = req.validate_all(&mut rev_pins, 7).expect("pin twice");
    rev_pins.revoke(&req.owner_handle);
    assert!(matches!(
        req.validate_all(&mut rev_pins, 7),
        Err(PaymentRequestError::PinRevoked)
    ));
}

fn check_req_prebuild(keys: &ReceiverKeys, req: &PaymentRequest, chain: u32) {
    let card = make_card(keys);
    let mut pins = PinnedReceiverCards::new();
    let mut sender = SenderWallet::new([0x41u8; 32]);
    let err = build_tx_stealth_output_validated(
        &card,
        Some(req),
        BuildCheck {
            pins: &mut pins,
            chain_id: chain,
        },
        &mut sender,
        &[0x42u8; 32],
        1,
        77,
        &[0x43u8; 32],
    )
    .expect_err("pre-approval request must reject");

    assert_eq!(err, StealthError::InvalidStealthInput);
}

fn build_req_ok(
    keys: &ReceiverKeys,
    req: &PaymentRequest,
    pins: &mut PinnedReceiverCards,
    chain: u32,
) {
    let card = make_card(keys);
    let mut sender = SenderWallet::new([0x44u8; 32]);
    let out = build_tx_stealth_output_validated(
        &card,
        Some(req),
        BuildCheck {
            pins,
            chain_id: chain,
        },
        &mut sender,
        &[0x45u8; 32],
        2,
        77,
        &[0x46u8; 32],
    )
    .expect("approved request builds");

    assert_ne!(out.owner_tag, [0u8; 32]);
}

fn check_card_errs(card: &ReceiverCard) {
    let bad_pt = [0xffu8; 32];

    let mut bad_ver = card.canonical_encoding();
    bad_ver[0] = 2;
    assert!(matches!(
        ReceiverCard::from_untrusted_bytes(&bad_ver),
        Err(ReceiverCardError::UnsupportedVersion)
    ));

    let mut bad_pk = card.canonical_encoding();
    bad_pk[33..65].copy_from_slice(&bad_pt);
    assert!(matches!(
        ReceiverCard::from_untrusted_bytes(&bad_pk),
        Err(ReceiverCardError::InvalidPublicKey)
    ));

    let mut bad_sig = card.canonical_encoding();
    let sig_at = bad_sig.len().saturating_sub(64);
    bad_sig[sig_at..].fill(0);
    let bad_card = ReceiverCard::from_canonical_encoding(&bad_sig).expect("bad card");
    assert!(matches!(
        bad_card.validate_signature(),
        Err(ReceiverCardError::InvalidSignature)
    ));
}

fn check_card_exp(card: &ReceiverCard, keys: &ReceiverKeys) {
    let mut exp = card.clone();
    exp.metadata = Some(CardMetadata {
        created_at: unix_now().saturating_sub(10),
        display_name: None,
        valid_until: Some(unix_now().saturating_sub(1)),
        contact: None,
    });
    exp.sign(keys.reveal_identity_sk()).expect("sign expired");

    let mut pins = PinnedReceiverCards::new();
    assert!(matches!(
        pins.verify_or_pin(&exp, None),
        Err(ReceiverCardError::CardExpired)
    ));
}

#[test]
fn test_s5_req_gate() {
    let keys = make_keys();
    let req_ok = parse_req(&make_req(&keys, 7));
    let mut pins = PinnedReceiverCards::new();

    check_req_prebuild(&keys, &req_ok, 7);

    let first = req_ok.validate_all(&mut pins, 7).expect("first");
    assert_eq!(first, ValidationOutcome::RequiresUserConfirmation);

    let second = req_ok.validate_all(&mut pins, 7).expect("second");
    assert_eq!(second, ValidationOutcome::Approved);
    build_req_ok(&keys, &req_ok, &mut pins, 7);

    check_req_base_errs(&req_ok);
    check_req_sig_errs(&req_ok);
    check_req_trust(&req_ok, &mut pins);
}

#[test]
fn test_s5_card_path() {
    let keys = make_keys();
    let card = make_card(&keys);

    card.validate_structure().expect("structure");
    card.validate_ecc_points().expect("points");
    card.validate_signature().expect("signature");
    card.verify().expect("verify");

    let wire = encode_card_compact(&card);
    let decoded = decode_card_compact(&wire).expect("decode");
    assert_eq!(decoded.canonical_encoding(), card.canonical_encoding());

    let mut sender = SenderWallet::new([0x11u8; 32]);
    let output = build_tx_output_unchecked(
        &decoded,
        None,
        &mut sender,
        &[0x22u8; 32],
        1,
        77,
        &[0x33u8; 32],
    )
    .expect("output");
    assert_ne!(output.owner_tag, [0u8; 32]);

    check_card_errs(&card);
    check_card_exp(&card, &keys);
}

#[test]
fn test_s5_tofu_flow() {
    let mut pins = PinnedReceiverCards::new();
    let mut keys = make_keys();
    let card_a = make_card(&keys);

    let first = pins.verify_or_pin(&card_a, None).expect("first pin");
    assert_eq!(first, VerifyResult::NewPin);

    let second = pins.verify_or_pin(&card_a, None).expect("second pin");
    assert_eq!(second, VerifyResult::Verified);

    let req_a = parse_req(&make_req(&keys, 11));
    check_req_pins(&req_a);
    let req_ok = req_a.validate_all(&mut pins, 11).expect("request ok");
    assert_eq!(req_ok, ValidationOutcome::Approved);
    build_req_ok(&keys, &req_a, &mut pins, 11);

    let card_b = keys.rotate_view().expect("rotate");
    let drift = pins.verify_or_pin(&card_b, None).expect("view drift");
    assert!(matches!(
        drift,
        VerifyResult::ViewKeyChanged {
            requires_confirmation: true,
            ..
        }
    ));

    pins.confirm_rotation(&card_b.owner_handle, &card_b.view_pk);
    let card_ok = pins.verify_or_pin(&card_b, None).expect("card ok");
    assert_eq!(card_ok, VerifyResult::Verified);

    let other = make_keys();
    let mut card_c = make_card(&other);
    card_c.owner_handle = card_a.owner_handle;
    card_c.view_pk = card_b.view_pk;
    card_c
        .sign(other.reveal_identity_sk())
        .expect("resign card");
    let id_card = pins.verify_or_pin(&card_c, None).expect("id card");
    assert_eq!(id_card, VerifyResult::IdentityKeyChanged);

    let mut req_b = parse_req(&make_req(&other, 11));
    req_b.owner_handle = card_a.owner_handle;
    req_b.sign(other.reveal_identity_sk()).expect("resign req");
    let id_req = req_b.validate_all(&mut pins, 11).expect("id req");
    assert_eq!(id_req, ValidationOutcome::IdentityMismatch);

    pins.revoke(&card_a.owner_handle);
    assert!(matches!(
        pins.verify_or_pin(&card_a, None),
        Err(ReceiverCardError::PinRevoked)
    ));

    let req_c = parse_req(&make_req(&keys, 11));
    assert!(matches!(
        req_c.validate_all(&mut pins, 11),
        Err(PaymentRequestError::PinRevoked)
    ));
}
