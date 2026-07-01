use z00z_core::{genesis::asset_std::asset_from_dev_cfg, Asset};
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::{
        PaymentRequest, PinnedReceiverCards, ReceiverCard, RequestParams, ValidatePaymentRequest,
        ValidationOutcome, VerifyResult,
    },
    receiver::{ScanResult, StealthOutputScanner},
    stealth::ecdh::{compute_dh_receiver, decode_r_pub},
    stealth::kdf::{
        compute_leaf_ad, compute_tag16, compute_tag16_with_req, derive_k_dh, derive_k_dh_with_req,
        derive_s_out,
    },
    stealth::{
        build_tx_output_unchecked, build_tx_stealth_output_validated, verify_owner_tag,
        verify_owner_tag_with_req, verify_owner_two_factor, BuildCheck, SenderWallet, StealthError,
        TxStealthOutput,
    },
};

const SPEND_VERIFICATION_SRC: &str = include_str!("../src/tx/spend_verification.rs");
const SPEND_RULES_SRC: &str = include_str!("../src/tx/spend_rules.rs");
const WITNESS_GATE_SRC: &str = include_str!("../src/tx/witness_gate.rs");
const TX_MOD_SRC: &str = include_str!("../src/tx/mod.rs");
const OUTPUT_FLOW_SRC: &str = include_str!("../src/tx/tx_output.rs");
const RECEIVE_ACTIONS_SRC: &str = include_str!("../src/services/wallet_actions_receive.rs");
const REQUIREMENTS_SRC: &str = include_str!("../../../.planning/REQUIREMENTS.md");

// Real theft-resistance boundary stays wallet-local here: the receiver-secret-gated wallet-local ownership rule combines receiver_secret with the re-derived output secret, while the public verifier remains narrower than that theorem.

const SERIAL_ID: u32 = 0;

fn make_keys(seed: [u8; 32]) -> ReceiverKeys {
    let secret = ReceiverSecret::from_bytes(seed).expect("secret");
    ReceiverKeys::from_receiver_secret(secret).expect("keys")
}

fn make_card(keys: &ReceiverKeys) -> ReceiverCard {
    let card = keys.export_receiver_card().expect("card");
    card.verify().expect("verify");
    card
}

fn make_req(keys: &ReceiverKeys, chain_id: u32, amount: u64) -> PaymentRequest {
    PaymentRequest::generate(
        keys,
        RequestParams {
            amount: Some(amount),
            expiry_seconds: 600,
            memo: Some("phase32".to_string()),
            payment_id: Some([0x52u8; 16]),
        },
        chain_id,
    )
    .expect("request")
}

fn approve_req(req: &PaymentRequest, pins: &mut PinnedReceiverCards, chain_id: u32) {
    assert_eq!(
        req.validate_all(pins, chain_id)
            .expect("first approval step"),
        ValidationOutcome::RequiresUserConfirmation
    );
    assert_eq!(
        req.validate_all(pins, chain_id)
            .expect("second approval step"),
        ValidationOutcome::Approved
    );
}

fn make_asset(amount: u64) -> Asset {
    asset_from_dev_cfg("z00z", 0, amount).expect("asset")
}

fn apply_output(asset: &mut Asset, output: &TxStealthOutput) {
    asset.commitment = z00z_crypto::Commitment::from_bytes(&output.c_amount)
        .expect("commitment")
        .0;
    asset.r_pub = Some(output.r_pub);
    asset.owner_tag = Some(output.owner_tag);
    asset.enc_pack = Some(output.enc_pack.clone());
    asset.tag16 = output.tag16;
    asset.leaf_ad_id = Some(asset.definition.id);
}

fn req_tag16(keys: &ReceiverKeys, req: &PaymentRequest, output: &TxStealthOutput) -> u16 {
    let r_pub = decode_r_pub(&output.r_pub).expect("r pub");
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &r_pub).expect("dh");
    let k_dh = derive_k_dh_with_req(&dh, &req.req_id);
    compute_tag16_with_req(&k_dh, &req.req_id)
}

fn card_tag16(keys: &ReceiverKeys, asset_id: &[u8; 32], output: &TxStealthOutput) -> u16 {
    let r_pub = decode_r_pub(&output.r_pub).expect("r pub");
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &r_pub).expect("dh");
    let k_dh = derive_k_dh(&dh);
    let leaf_ad = compute_leaf_ad(
        asset_id,
        SERIAL_ID,
        &output.r_pub,
        &output.owner_tag,
        &output.c_amount,
    );
    compute_tag16(&k_dh, &leaf_ad)
}

#[test]
fn test_s1_decrypt_boundary() {
    let keys = make_keys([0x11u8; 32]);
    let card = make_card(&keys);
    let mut sender = SenderWallet::new([0x12u8; 32]);
    let mut asset = make_asset(111);

    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut sender,
        &[0x13u8; 32],
        0,
        asset.amount,
        &asset.definition.id,
    )
    .expect("output");
    apply_output(&mut asset, &output);

    let scanner = StealthOutputScanner::from_keys(&keys);
    assert!(matches!(scanner.scan_leaf(&asset), ScanResult::Mine { .. }));

    asset.leaf_ad_id = Some([0x99u8; 32]);
    assert!(!matches!(
        scanner.scan_leaf(&asset),
        ScanResult::Mine { .. }
    ));
}

#[test]
fn test_s1_bridge_scope() {
    assert!(
        SPEND_VERIFICATION_SRC.contains("wallet, scan, report, and spend-witness bridge paths")
            && SPEND_VERIFICATION_SRC.contains("repository-wide total closure"),
        "public spend verification must keep leaf_ad_id scope on shipped bridge paths"
    );
    assert!(
        WITNESS_GATE_SRC.contains("wallet, scan, report, and spend-witness bridge")
            && WITNESS_GATE_SRC.contains("repository-wide artifact closure"),
        "witness gate must keep the decrypt/state split accepted-path scoped"
    );
}

#[test]
fn test_s1_scan_locality() {
    assert!(
        RECEIVE_ACTIONS_SRC.contains("receiver-secret plus `s_out`")
            && RECEIVE_ACTIONS_SRC.contains("wallet-local accepted-path")
            && RECEIVE_ACTIONS_SRC.contains("public trustless theorem"),
        "receive actions must keep post-scan exclusivity wallet-local instead of public"
    );
    assert!(
        SPEND_VERIFICATION_SRC.contains("receiver-secret")
            && SPEND_VERIFICATION_SRC.contains("plus `s_out`")
            && SPEND_VERIFICATION_SRC.contains("wallet-local post-scan exclusivity gate")
            && SPEND_VERIFICATION_SRC.contains("public trustless theorem"),
        "public spend verification must not upgrade wallet-local exclusivity into a public theorem"
    );
}

#[test]
fn test_s1_tag16_modes() {
    let chain_id = 17u32;
    let amount = 777u64;
    let tx_digest = [0x21u8; 32];
    let keys = make_keys([0x22u8; 32]);
    let card = make_card(&keys);
    let req = make_req(&keys, chain_id, amount);
    let asset_id = [0x23u8; 32];

    let mut raw_sender = SenderWallet::new([0x24u8; 32]);
    let raw = build_tx_output_unchecked(
        &card,
        None,
        &mut raw_sender,
        &tx_digest,
        1,
        amount,
        &asset_id,
    )
    .expect("raw output");

    let mut pins = PinnedReceiverCards::new();
    approve_req(&req, &mut pins, chain_id);

    let mut strict_sender = SenderWallet::new([0x25u8; 32]);
    let strict = build_tx_stealth_output_validated(
        &card,
        Some(&req),
        BuildCheck {
            pins: &mut pins,
            chain_id,
        },
        &mut strict_sender,
        &tx_digest,
        1,
        amount,
        &asset_id,
    )
    .expect("strict output");

    assert_eq!(raw.tag16, Some(card_tag16(&keys, &asset_id, &raw)));
    assert_eq!(strict.tag16, Some(req_tag16(&keys, &req, &strict)));
    assert_ne!(strict.tag16, Some(card_tag16(&keys, &asset_id, &strict)));

    assert!(verify_owner_tag(&keys, &raw.r_pub, &raw.owner_tag).expect("raw verify"));
    assert!(!verify_owner_tag(&keys, &strict.r_pub, &strict.owner_tag)
        .expect("strict card mode verify"));
    assert!(
        verify_owner_tag_with_req(&keys, &strict.r_pub, &strict.owner_tag, Some(&req.req_id))
            .expect("strict req mode verify")
    );
}

#[test]
fn test_s1_foreign_card_rejected() {
    let chain_id = 19u32;
    let amount = 777u64;
    let card_keys = make_keys([0x31u8; 32]);
    let req_keys = make_keys([0x32u8; 32]);
    let card = make_card(&card_keys);
    let req = make_req(&req_keys, chain_id, amount);

    let mut pins = PinnedReceiverCards::new();
    approve_req(&req, &mut pins, chain_id);

    let mut sender = SenderWallet::new([0x33u8; 32]);
    let err = build_tx_stealth_output_validated(
        &card,
        Some(&req),
        BuildCheck {
            pins: &mut pins,
            chain_id,
        },
        &mut sender,
        &[0x34u8; 32],
        2,
        amount,
        &[0x35u8; 32],
    )
    .expect_err("foreign request/card route must fail");

    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_s1_rotation_flow() {
    let mut keys = make_keys([0x41u8; 32]);
    let first = make_card(&keys);
    let mut pins = PinnedReceiverCards::new();

    assert_eq!(
        pins.verify_or_pin(&first, None).expect("new pin"),
        VerifyResult::NewPin
    );
    assert_eq!(
        pins.verify_or_pin(&first, None).expect("verified pin"),
        VerifyResult::Verified
    );

    let rotated = keys.rotate_view().expect("rotate view");
    match pins
        .verify_or_pin(&rotated, None)
        .expect("rotation verdict")
    {
        VerifyResult::ViewKeyChanged {
            requires_confirmation,
            ..
        } => assert!(requires_confirmation),
        other => panic!("unexpected rotation result: {other:?}"),
    }

    pins.confirm_rotation(&rotated.owner_handle, &rotated.view_pk);
    assert_eq!(
        pins.verify_or_pin(&rotated, None)
            .expect("post-confirm verify"),
        VerifyResult::Verified
    );
}

#[test]
fn test_s1_two_factor_gate() {
    let keys = make_keys([0x51u8; 32]);
    let card = make_card(&keys);
    let mut sender = SenderWallet::new([0x52u8; 32]);
    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut sender,
        &[0x53u8; 32],
        3,
        500,
        &[0x54u8; 32],
    )
    .expect("output");

    let r_pub = decode_r_pub(&output.r_pub).expect("r pub");
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &r_pub).expect("dh");
    let k_dh = derive_k_dh(&dh);
    let s_out = derive_s_out(&k_dh, &output.r_pub, SERIAL_ID);
    let receiver_secret = ReceiverSecret::from_bytes([0x51u8; 32]).expect("receiver secret");

    assert!(verify_owner_two_factor(
        &receiver_secret,
        &output.r_pub,
        &output.owner_tag,
        &s_out,
        SERIAL_ID,
    )
    .expect("receiver ownership"));

    let sender_material = ReceiverSecret::from_bytes(sender.secret_salt).expect("sender material");
    assert!(!verify_owner_two_factor(
        &sender_material,
        &output.r_pub,
        &output.owner_tag,
        &s_out,
        SERIAL_ID,
    )
    .expect("sender cannot satisfy receiver gate"));

    assert!(!verify_owner_two_factor(
        &receiver_secret,
        &output.r_pub,
        &output.owner_tag,
        &[0u8; 32],
        SERIAL_ID,
    )
    .expect("tampered s_out must fail"));
}

#[test]
fn test_s1_spend_boundary() {
    assert!(
        SPEND_VERIFICATION_SRC.contains("delivered")
            && SPEND_VERIFICATION_SRC.contains("persisted")
            && SPEND_VERIFICATION_SRC.contains("public spend contract")
            && SPEND_VERIFICATION_SRC.contains("deterministic nullifier semantics surface")
            && SPEND_VERIFICATION_SRC.contains("current proof/auth seam")
            && SPEND_VERIFICATION_SRC.contains("already live"),
        "public spend verifier must describe only the delivered persisted boundary"
    );
    assert!(
        SPEND_RULES_SRC.contains("delivered persisted public spend")
            && SPEND_RULES_SRC.contains("contract's owner")
            && SPEND_RULES_SRC.contains("deterministic nullifier semantics surface")
            && SPEND_RULES_SRC.contains("structural rule layer"),
        "spend rules must keep broader spend claims outside the delivered boundary"
    );
    assert!(
        REQUIREMENTS_SRC.contains("delivered persisted public spend contract")
            && REQUIREMENTS_SRC.contains("authenticates one signed nullifier field on the public seam")
            && REQUIREMENTS_SRC.contains(
                "witness bridge and structural spend rules enforce deterministic `chain_id || s_in` derivation"
            ),
        "PH32-SPEND must record the shipped deterministic nullifier closure honestly"
    );
}

#[test]
fn test_s1_nullifier_closure() {
    assert!(
        SPEND_VERIFICATION_SRC.contains("deterministic nullifier semantics surface")
            && !SPEND_VERIFICATION_SRC.contains("still carries no nullifier semantics"),
        "public spend verifier must advertise the shipped deterministic nullifier closure honestly"
    );
    assert!(
        SPEND_RULES_SRC.contains("deterministic nullifier semantics surface")
            && !SPEND_RULES_SRC.contains("still carries no nullifier semantics"),
        "spend rules documentation must advertise the shipped deterministic nullifier closure honestly"
    );
    assert!(
        REQUIREMENTS_SRC.contains("authenticates one signed nullifier field on the public seam")
            && REQUIREMENTS_SRC.contains(
                "witness bridge and structural spend rules enforce deterministic `chain_id || s_in` derivation"
            ),
        "PH32-SPEND must record the deterministic nullifier closure in active requirements"
    );
}

#[test]
fn test_s1_sender_authority_docs() {
    let tx_mod = TX_MOD_SRC.to_lowercase();
    assert!(
        tx_mod.contains("use `crate::tx` for tx-specific assembly and verification flows")
            && tx_mod.contains("use `crate::stealth` for public sender-output construction")
            && tx_mod.contains("no longer part of the public caller surface"),
        "tx docs must keep public sender construction under the canonical stealth module"
    );
    assert!(
        OUTPUT_FLOW_SRC.contains("These helpers stay internal to the tx facade")
            && OUTPUT_FLOW_SRC.contains("public sender-output construction surface"),
        "output flow docs must keep public sender-output construction under `stealth`"
    );
    assert!(
        REQUIREMENTS_SRC.contains("- [x] **PH34-SENDER-AUTHORITY**")
            && REQUIREMENTS_SRC
                .contains("`stealth` as the only public sender-construction authority")
            && REQUIREMENTS_SRC
                .contains("narrow test-only helpers as noncanonical internal surfaces"),
        "active requirements must record the sender-authority reclassification"
    );
}
