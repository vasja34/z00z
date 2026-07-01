use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::card::receiver_card_trust::PinCheckResult,
    receiver::{
        PaymentRequest, PinEntry, PinnedReceiverCards, ReceiverCard, ReceiverCardError,
        RequestParams, TrustLevel, ValidatePaymentRequest, ValidateReceiverCard, ValidationOutcome,
        VerifyResult,
    },
    stealth::{
        build_tx_output_unchecked, build_tx_stealth_output_validated,
        output::build_card_stealth_output_validated, BuildCheck, SenderWallet, StealthError,
    },
};

fn make_keys() -> ReceiverKeys {
    let sec = ReceiverSecret::generate().expect("secret");
    ReceiverKeys::from_receiver_secret(sec).expect("keys")
}

fn make_card(keys: &ReceiverKeys) -> ReceiverCard {
    let card = keys.export_receiver_card().expect("card");
    card.verify().expect("verify");
    card
}

fn make_req(keys: &ReceiverKeys, chain: u32) -> PaymentRequest {
    PaymentRequest::generate(
        keys,
        RequestParams {
            amount: Some(777),
            expiry_seconds: 600,
            memo: Some("step6".to_string()),
            payment_id: Some([0x61u8; 16]),
        },
        chain,
    )
    .expect("request")
}

fn bad_sig(card: &ReceiverCard) -> ReceiverCard {
    let mut bytes = card.canonical_encoding();
    let sig_at = bytes.len().saturating_sub(64);
    bytes[sig_at..].fill(0);
    ReceiverCard::from_canonical_encoding(&bytes).expect("bad card")
}

fn bad_view_pk(card: &ReceiverCard) -> ReceiverCard {
    let mut bad = card.clone();
    bad.view_pk = [0u8; 32];
    bad
}

fn pinned_card_pins(card: &ReceiverCard) -> PinnedReceiverCards {
    let mut pins = PinnedReceiverCards::new();

    assert_eq!(
        pins.verify_or_pin(card, None).expect("first pin"),
        VerifyResult::NewPin
    );
    assert_eq!(
        pins.verify_or_pin(card, None).expect("verified pin"),
        VerifyResult::Verified
    );

    let entry = pins.get(&card.owner_handle).expect("pin entry").clone();
    PinnedReceiverCards::from_pairs(vec![(
        card.owner_handle,
        PinEntry {
            trust_level: TrustLevel::Pinned,
            ..entry
        },
    )])
}

#[test]
fn test_raw_needs_card_check() {
    let amount = 777u64;
    let aid = [0x51u8; 32];
    let tx = [0x52u8; 32];
    let keys = make_keys();
    let good = make_card(&keys);
    let bad = bad_sig(&good);

    assert!(matches!(
        bad.validate_signature(),
        Err(ReceiverCardError::InvalidSignature)
    ));
    good.validate_signature().expect("good card");

    let mut bad_sender = SenderWallet::new([0x53u8; 32]);
    let misuse = build_tx_output_unchecked(&bad, None, &mut bad_sender, &tx, 1, amount, &aid)
        .expect("caller misuse still builds");
    let mut good_sender = SenderWallet::new([0x54u8; 32]);
    let checked = build_tx_output_unchecked(&good, None, &mut good_sender, &tx, 1, amount, &aid)
        .expect("validated card builds");

    assert!(misuse.tag16.is_some());
    assert!(checked.tag16.is_some());
}

#[test]
fn test_strict_needs_req_ok() {
    let chain = 27u32;
    let amount = 777u64;
    let aid = [0x55u8; 32];
    let tx = [0x56u8; 32];
    let keys = make_keys();
    let card = make_card(&keys);
    let req = make_req(&keys, chain);
    let mut pins = z00z_wallets::receiver::PinnedReceiverCards::new();
    let mut first_sender = SenderWallet::new([0x57u8; 32]);
    let err = build_tx_stealth_output_validated(
        &card,
        Some(&req),
        BuildCheck {
            pins: &mut pins,
            chain_id: chain,
        },
        &mut first_sender,
        &tx,
        2,
        amount,
        &aid,
    )
    .expect_err("non-approved request");

    assert_eq!(err, StealthError::InvalidStealthInput);
    let first = req.validate_all(&mut pins, chain).expect("first");
    let second = req.validate_all(&mut pins, chain).expect("second");
    assert_eq!(first, ValidationOutcome::RequiresUserConfirmation);
    assert_eq!(second, ValidationOutcome::Approved);

    let mut ok_sender = SenderWallet::new([0x58u8; 32]);
    let strict = build_tx_stealth_output_validated(
        &card,
        Some(&req),
        BuildCheck {
            pins: &mut pins,
            chain_id: chain,
        },
        &mut ok_sender,
        &tx,
        2,
        amount,
        &aid,
    )
    .expect("approved request");

    assert!(strict.tag16.is_some());
}

#[test]
fn test_s5_card_card_ok() {
    let amount = 777u64;
    let aid = [0x59u8; 32];
    let tx = [0x5Au8; 32];
    let keys = make_keys();
    let card = make_card(&keys);
    let mut pins = pinned_card_pins(&card);
    let mut sender = SenderWallet::new([0x5Bu8; 32]);

    let strict = build_card_stealth_output_validated(
        &card,
        BuildCheck {
            pins: &mut pins,
            chain_id: 27,
        },
        &mut sender,
        &tx,
        3,
        amount,
        &aid,
    )
    .expect("approved card");

    assert!(strict.tag16.is_some());
}

#[test]
fn test_s5_card_unapproved_card() {
    let amount = 777u64;
    let aid = [0x5Cu8; 32];
    let tx = [0x5Du8; 32];
    let keys = make_keys();
    let card = make_card(&keys);
    let mut pins = PinnedReceiverCards::new();
    let mut sender = SenderWallet::new([0x5Eu8; 32]);

    let err = build_card_stealth_output_validated(
        &card,
        BuildCheck {
            pins: &mut pins,
            chain_id: 27,
        },
        &mut sender,
        &tx,
        4,
        amount,
        &aid,
    )
    .expect_err("new card must reject");

    assert_eq!(err, StealthError::InvalidStealthInput);
    assert!(pins.is_empty());
}

#[test]
fn test_s5_card_tentative_pin() {
    let amount = 777u64;
    let aid = [0x77u8; 32];
    let tx = [0x78u8; 32];
    let keys = make_keys();
    let card = make_card(&keys);
    let mut pins = PinnedReceiverCards::new();
    assert_eq!(
        pins.verify_or_pin(&card, None).expect("tentative pin"),
        VerifyResult::NewPin
    );
    let mut sender = SenderWallet::new([0x79u8; 32]);

    let err = build_card_stealth_output_validated(
        &card,
        BuildCheck {
            pins: &mut pins,
            chain_id: 27,
        },
        &mut sender,
        &tx,
        5,
        amount,
        &aid,
    )
    .expect_err("tentative pin must reject");

    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_s5_card_bad_sig() {
    let amount = 777u64;
    let aid = [0x5Fu8; 32];
    let tx = [0x60u8; 32];
    let keys = make_keys();
    let card = make_card(&keys);
    let bad = bad_sig(&card);
    let mut pins = pinned_card_pins(&card);
    let mut sender = SenderWallet::new([0x61u8; 32]);

    let err = build_card_stealth_output_validated(
        &bad,
        BuildCheck {
            pins: &mut pins,
            chain_id: 27,
        },
        &mut sender,
        &tx,
        5,
        amount,
        &aid,
    )
    .expect_err("bad signature must reject");

    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_s5_card_view_pk() {
    let amount = 777u64;
    let aid = [0x62u8; 32];
    let tx = [0x63u8; 32];
    let keys = make_keys();
    let card = make_card(&keys);
    let bad = bad_view_pk(&card);
    let mut pins = pinned_card_pins(&card);
    let mut sender = SenderWallet::new([0x64u8; 32]);

    let err = build_card_stealth_output_validated(
        &bad,
        BuildCheck {
            pins: &mut pins,
            chain_id: 27,
        },
        &mut sender,
        &tx,
        6,
        amount,
        &aid,
    )
    .expect_err("bad view key must reject");

    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_s5_req_self_check() {
    let amount = 777u64;
    let aid = [0x65u8; 32];
    let tx = [0x66u8; 32];
    let keys = make_keys();
    let card = make_card(&keys);
    let mut pins = PinnedReceiverCards::new();
    let mut sender = SenderWallet::new([0x67u8; 32]);

    let strict = build_tx_stealth_output_validated(
        &card,
        None,
        BuildCheck {
            pins: &mut pins,
            chain_id: 27,
        },
        &mut sender,
        &tx,
        7,
        amount,
        &aid,
    )
    .expect("strict self-check path");

    assert!(strict.tag16.is_some());
    assert!(pins.is_empty());
}

#[test]
fn test_s5_card_revoked_pin() {
    let amount = 777u64;
    let aid = [0x68u8; 32];
    let tx = [0x69u8; 32];
    let keys = make_keys();
    let card = make_card(&keys);
    let mut pins = pinned_card_pins(&card);
    let mut sender = SenderWallet::new([0x6Au8; 32]);
    pins.revoke(&card.owner_handle);

    let err = build_card_stealth_output_validated(
        &card,
        BuildCheck {
            pins: &mut pins,
            chain_id: 27,
        },
        &mut sender,
        &tx,
        8,
        amount,
        &aid,
    )
    .expect_err("revoked card must reject");

    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_s5_card_key_change() {
    let amount = 777u64;
    let aid = [0x6Bu8; 32];
    let tx = [0x6Cu8; 32];
    let mut keys = make_keys();
    let card = make_card(&keys);
    let rotated = keys.rotate_view().expect("rotated card");
    let mut pins = pinned_card_pins(&card);
    let mut sender = SenderWallet::new([0x6Du8; 32]);

    let err = build_card_stealth_output_validated(
        &rotated,
        BuildCheck {
            pins: &mut pins,
            chain_id: 27,
        },
        &mut sender,
        &tx,
        9,
        amount,
        &aid,
    )
    .expect_err("view-key drift must reject");

    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_s5_card_expired_pin() {
    let amount = 777u64;
    let aid = [0x74u8; 32];
    let tx = [0x75u8; 32];
    let keys = make_keys();
    let card = make_card(&keys);
    let mut pins = pinned_card_pins(&card);
    let entry = pins.get(&card.owner_handle).expect("pin entry").clone();
    pins = PinnedReceiverCards::from_pairs(vec![(
        card.owner_handle,
        PinEntry {
            trust_level: TrustLevel::Expired,
            ..entry
        },
    )]);
    let mut sender = SenderWallet::new([0x76u8; 32]);

    let err = build_card_stealth_output_validated(
        &card,
        BuildCheck {
            pins: &mut pins,
            chain_id: 27,
        },
        &mut sender,
        &tx,
        12,
        amount,
        &aid,
    )
    .expect_err("expired pin must reject");

    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_s5_card_pk_mismatch() {
    let amount = 777u64;
    let aid = [0x6Eu8; 32];
    let tx = [0x6Fu8; 32];
    let keys = make_keys();
    let card = make_card(&keys);
    let mut pins = PinnedReceiverCards::from_pairs(vec![(
        card.owner_handle,
        PinEntry {
            view_pk: card.view_pk,
            identity_pk: [0x7Au8; 32],
            directory_id: None,
            first_seen: 1,
            trust_level: TrustLevel::Pinned,
        },
    )]);
    let mut sender = SenderWallet::new([0x70u8; 32]);

    let err = build_card_stealth_output_validated(
        &card,
        BuildCheck {
            pins: &mut pins,
            chain_id: 27,
        },
        &mut sender,
        &tx,
        10,
        amount,
        &aid,
    )
    .expect_err("identity drift must reject");

    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_s5_card_placeholder_pin() {
    let amount = 777u64;
    let aid = [0x71u8; 32];
    let tx = [0x72u8; 32];
    let keys = make_keys();
    let card = make_card(&keys);
    let mut pins = PinnedReceiverCards::new();
    let mut sender = SenderWallet::new([0x73u8; 32]);

    assert_eq!(
        pins.verify_request_identity(&card.owner_handle, &card.identity_pk),
        PinCheckResult::NewIdentity
    );

    let err = build_card_stealth_output_validated(
        &card,
        BuildCheck {
            pins: &mut pins,
            chain_id: 27,
        },
        &mut sender,
        &tx,
        11,
        amount,
        &aid,
    )
    .expect_err("request-only placeholder pin must reject");

    assert_eq!(err, StealthError::InvalidStealthInput);
}
