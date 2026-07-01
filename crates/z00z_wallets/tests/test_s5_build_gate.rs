use z00z_core::assets::AssetPackPlain;
use z00z_wallets::{
    build_tx_output_unchecked, build_tx_stealth_output_validated,
    key::{ReceiverKeys, ReceiverSecret},
    receiver::{
        PaymentRequest, PinnedReceiverCards, ReceiverCard, RequestParams, ValidatePaymentRequest,
        ValidationOutcome,
    },
    stealth::ecdh::{compute_dh_receiver, compute_r_pub, decode_r_pub, derive_r_hedged},
    stealth::kdf::{
        compute_leaf_ad, compute_owner_tag, compute_tag16, compute_tag16_with_req, derive_k_dh,
        derive_k_dh_with_req, derive_s_out,
    },
    stealth::zkpack::ZkPack,
    validate_output_self, BuildCheck, SenderValidationCtx, SenderWallet, StealthError, TagMode,
    TxStealthOutput,
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

fn make_req(keys: &ReceiverKeys, chain: u32, mark: u8) -> PaymentRequest {
    PaymentRequest::generate(
        keys,
        RequestParams {
            amount: Some(777),
            expiry_seconds: 600,
            memo: Some(format!("req-{mark}")),
            payment_id: Some([mark; 16]),
        },
        chain,
    )
    .expect("request")
}

fn parse_req(req: &PaymentRequest) -> PaymentRequest {
    PaymentRequest::from_untrusted_bytes(&req.canonical_encoding()).expect("parse")
}

fn ok_pins(req: &PaymentRequest, chain: u32) -> PinnedReceiverCards {
    let mut pins = PinnedReceiverCards::new();
    let first = req.validate_all(&mut pins, chain).expect("first");
    assert_eq!(first, ValidationOutcome::RequiresUserConfirmation);
    let second = req.validate_all(&mut pins, chain).expect("second");
    assert_eq!(second, ValidationOutcome::Approved);
    pins
}

fn card_kdh(keys: &ReceiverKeys, out: &TxStealthOutput) -> [u8; 32] {
    let r_pub = decode_r_pub(&out.r_pub).expect("r_pub");
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &r_pub).expect("dh");
    derive_k_dh(&dh)
}

fn req_kdh(keys: &ReceiverKeys, out: &TxStealthOutput, req: &PaymentRequest) -> [u8; 32] {
    let r_pub = decode_r_pub(&out.r_pub).expect("r_pub");
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &r_pub).expect("dh");
    derive_k_dh_with_req(&dh, &req.req_id)
}

fn open_light(out: &TxStealthOutput, k_dh: &[u8; 32], aid: &[u8; 32]) -> AssetPackPlain {
    let leaf_ad = compute_leaf_ad(aid, 0, &out.r_pub, &out.owner_tag, &out.c_amount);
    let bytes = ZkPack::decrypt(k_dh, &leaf_ad, &out.r_pub, aid, 0, &out.enc_pack).expect("open");
    AssetPackPlain::decode_checked(&bytes).expect("plain")
}

fn check_card_branch(
    keys: &ReceiverKeys,
    card: &ReceiverCard,
    tx: &[u8; 32],
    aid: &[u8; 32],
    amount: u64,
) {
    let mut sender = SenderWallet::new([0x61u8; 32]);
    let out = build_tx_output_unchecked(card, None, &mut sender, tx, 3, amount, aid).expect("raw");
    let k_dh = card_kdh(keys, &out);
    let leaf_ad = compute_leaf_ad(aid, 0, &out.r_pub, &out.owner_tag, &out.c_amount);
    let plain = open_light(&out, &k_dh, aid);
    let ctx = SenderValidationCtx {
        k_dh,
        owner_handle: card.owner_handle,
        asset_id: *aid,
        serial_id: 0,
        tag_mode: TagMode::CardBound,
    };

    assert_eq!(
        out.owner_tag,
        compute_owner_tag(&card.owner_handle, &ctx.k_dh)
    );
    assert_eq!(out.tag16, Some(compute_tag16(&ctx.k_dh, &leaf_ad)));
    assert_eq!(plain.value, amount);
    assert_eq!(plain.s_out, derive_s_out(&ctx.k_dh, &out.r_pub, 0));
    validate_output_self(&out, &ctx, amount).expect("raw self");
}

fn check_req_branch(
    keys: &ReceiverKeys,
    card: &ReceiverCard,
    req: &PaymentRequest,
    tx: &[u8; 32],
    aid: &[u8; 32],
    amount: u64,
    chain: u32,
) {
    let mut pins = ok_pins(req, chain);
    let mut sender = SenderWallet::new([0x71u8; 32]);
    let out = build_tx_stealth_output_validated(
        card,
        Some(req),
        BuildCheck {
            pins: &mut pins,
            chain_id: chain,
        },
        &mut sender,
        tx,
        4,
        amount,
        aid,
    )
    .expect("strict");
    let k_dh = req_kdh(keys, &out, req);
    let plain = open_light(&out, &k_dh, aid);
    let ctx = SenderValidationCtx {
        k_dh,
        owner_handle: card.owner_handle,
        asset_id: *aid,
        serial_id: 0,
        tag_mode: TagMode::RequestBound { req_id: req.req_id },
    };

    assert_eq!(
        out.owner_tag,
        compute_owner_tag(&card.owner_handle, &ctx.k_dh)
    );
    assert_eq!(
        out.tag16,
        Some(compute_tag16_with_req(&ctx.k_dh, &req.req_id))
    );
    assert_eq!(plain.value, amount);
    validate_output_self(&out, &ctx, amount).expect("strict self");
}

fn check_new_id(
    card: &ReceiverCard,
    req: &PaymentRequest,
    tx: &[u8; 32],
    aid: &[u8; 32],
    amount: u64,
    chain: u32,
) {
    let mut pins = PinnedReceiverCards::new();
    let mut sender = SenderWallet::new([0x81u8; 32]);
    let err = build_tx_stealth_output_validated(
        card,
        Some(req),
        BuildCheck {
            pins: &mut pins,
            chain_id: chain,
        },
        &mut sender,
        tx,
        5,
        amount,
        aid,
    )
    .expect_err("new id");
    assert_eq!(err, StealthError::InvalidStealthInput);
    assert!(pins.is_empty());
}

fn check_bad_chain(
    card: &ReceiverCard,
    req: &PaymentRequest,
    tx: &[u8; 32],
    aid: &[u8; 32],
    amount: u64,
    chain: u32,
) {
    let mut pins = ok_pins(req, chain);
    let mut sender = SenderWallet::new([0x82u8; 32]);
    let err = build_tx_stealth_output_validated(
        card,
        Some(req),
        BuildCheck {
            pins: &mut pins,
            chain_id: chain.wrapping_add(1),
        },
        &mut sender,
        tx,
        5,
        amount,
        aid,
    )
    .expect_err("bad chain");
    assert_eq!(err, StealthError::InvalidStealthInput);
}

fn check_expired(
    card: &ReceiverCard,
    req: &PaymentRequest,
    tx: &[u8; 32],
    aid: &[u8; 32],
    amount: u64,
    chain: u32,
) {
    let mut exp_req = req.clone();
    exp_req.expiry = 0;
    let mut pins = ok_pins(req, chain);
    let mut sender = SenderWallet::new([0x83u8; 32]);
    let err = build_tx_stealth_output_validated(
        card,
        Some(&exp_req),
        BuildCheck {
            pins: &mut pins,
            chain_id: chain,
        },
        &mut sender,
        tx,
        5,
        amount,
        aid,
    )
    .expect_err("expired");
    assert_eq!(err, StealthError::InvalidStealthInput);
}

fn check_dup_tuple(card: &ReceiverCard, tx: &[u8; 32], aid: &[u8; 32], amount: u64) {
    let mut sender = SenderWallet::new([0x91u8; 32]);
    let first = build_tx_output_unchecked(card, None, &mut sender, tx, 6, amount, aid).expect("a");
    let second = build_tx_output_unchecked(card, None, &mut sender, tx, 6, amount, aid).expect("b");

    assert_ne!(first.r_pub, second.r_pub);
    assert_ne!(first.owner_tag, second.owner_tag);
}

fn check_hedged_det(tx: &[u8; 32]) {
    let rng = [0x19u8; 32];
    let salt = [0x91u8; 32];
    let first = derive_r_hedged(&rng, &salt, tx, 6).expect("r a");
    let second = derive_r_hedged(&rng, &salt, tx, 6).expect("r b");

    assert_eq!(first.as_bytes(), second.as_bytes());
    assert_eq!(
        compute_r_pub(&first).expect("r_pub a").as_bytes(),
        compute_r_pub(&second).expect("r_pub b").as_bytes()
    );
}

fn check_req_bind(
    card: &ReceiverCard,
    req_a: &PaymentRequest,
    req_b: &PaymentRequest,
    tx: &[u8; 32],
    aid: &[u8; 32],
    amount: u64,
    chain: u32,
) {
    let _ = ok_pins(req_a, chain);
    let _ = ok_pins(req_b, chain);

    let mut sender_a = SenderWallet::new([0x92u8; 32]);
    let mut sender_b = SenderWallet::new([0x92u8; 32]);
    let out_a = build_tx_output_unchecked(card, Some(req_a), &mut sender_a, tx, 7, amount, aid)
        .expect("req a");
    let out_b = build_tx_output_unchecked(card, Some(req_b), &mut sender_b, tx, 7, amount, aid)
        .expect("req b");

    assert_ne!(out_a.owner_tag, out_b.owner_tag);
    assert_ne!(out_a.tag16, out_b.tag16);
}

fn check_out_idx(card: &ReceiverCard, tx: &[u8; 32], aid: &[u8; 32], amount: u64) {
    let mut sender = SenderWallet::new([0x93u8; 32]);
    let first =
        build_tx_output_unchecked(card, None, &mut sender, tx, 8, amount, aid).expect("idx a");
    let second =
        build_tx_output_unchecked(card, None, &mut sender, tx, 9, amount, aid).expect("idx b");

    assert_ne!(first.r_pub, second.r_pub);
}

#[test]
fn test_s5_out_build() {
    let chain = 21;
    let amount = 777u64;
    let aid = [0x41u8; 32];
    let tx = [0x51u8; 32];
    let keys = make_keys();
    let card = make_card(&keys);
    let req = parse_req(&make_req(&keys, chain, 1));

    check_card_branch(&keys, &card, &tx, &aid, amount);
    check_req_branch(&keys, &card, &req, &tx, &aid, amount, chain);
}

#[test]
fn test_s5_out_strict() {
    let chain = 22;
    let amount = 777u64;
    let aid = [0x42u8; 32];
    let tx = [0x52u8; 32];
    let keys = make_keys();
    let card = make_card(&keys);
    let req = parse_req(&make_req(&keys, chain, 2));

    let _ = keys;
    check_new_id(&card, &req, &tx, &aid, amount, chain);
    check_bad_chain(&card, &req, &tx, &aid, amount, chain);
    check_expired(&card, &req, &tx, &aid, amount, chain);
}

#[test]
fn test_s5_out_diff() {
    let chain = 23;
    let amount = 777u64;
    let aid = [0x43u8; 32];
    let tx = [0x53u8; 32];
    let keys = make_keys();
    let card = make_card(&keys);
    let req_a = parse_req(&make_req(&keys, chain, 3));
    let req_b = parse_req(&make_req(&keys, chain, 4));

    check_hedged_det(&tx);
    check_dup_tuple(&card, &tx, &aid, amount);
    check_req_bind(&card, &req_a, &req_b, &tx, &aid, amount, chain);
    check_out_idx(&card, &tx, &aid, amount);
}
