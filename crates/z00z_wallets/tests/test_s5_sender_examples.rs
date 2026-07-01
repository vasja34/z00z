use std::{fs, path::PathBuf, sync::Arc};

#[path = "test_inc/test_range_proof_env.inc"]
mod test_common;

#[path = "test_inc/test_s5_sender_examples_support.inc"]
mod test_s5_sender_examples_support;

use test_common::RangeProofEnvGuard;
use z00z_core::{
    genesis::asset_std::{asset_from_dev_class, serials_from_dev_class},
    AssetClass, AssetWire,
};
use z00z_crypto::{create_range_proof, Z00ZScalar};
use z00z_utils::codec::Codec;
use z00z_wallets::{
    build_tx_output_unchecked, build_tx_stealth_output_validated, BuildCheck, SenderWallet,
    StealthError, TxStealthOutput,
};
use z00z_wallets::{
    chain::{
        receiver_card_record::CardRecordError, verify_receiver_card_record, ReceiverCardRecord,
    },
    key::{ReceiverKeys, ReceiverSecret},
    receiver::request::{decode_request_compact, encode_request_compact},
    receiver::{
        decode_card_compact, encode_card_compact, PaymentRequest, PinnedReceiverCards, ReceiveNext,
        ReceiveStatus, ReceiverCard, ReceiverCardError, RequestParams, ScanResult,
        StealthOutputScanner, ValidatePaymentRequest, ValidateReceiverCard, ValidationOutcome,
        VerifyResult,
    },
    receiver::{receiver_scan_leaf, receiver_scan_report},
    stealth::build_card_stealth_leaf,
    stealth::ecdh::{compute_dh_receiver, decode_r_pub},
    stealth::kdf::{
        compute_leaf_ad, compute_tag16, compute_tag16_with_req, derive_k_dh, derive_k_dh_with_req,
    },
    tx::{derive_output_nonce, ClaimTxPackage, ClaimTxVerifier, ClaimTxVerifierImpl},
};

const CLAIM_TX: &str = include_str!("../src/tx/claim_tx.rs");
const CHAIN_ID: u32 = 3;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn load_optional_spec(path: &str) -> Option<String> {
    let spec = repo_root().join(path);
    fs::read_to_string(spec).ok()
}

#[derive(Debug, PartialEq, Eq)]
struct PubOut {
    r_pub: [u8; 32],
    owner_tag: [u8; 32],
    tag16: Option<u16>,
    c_amount: [u8; 32],
}

fn make_keys(seed: [u8; 32]) -> ReceiverKeys {
    let secret = ReceiverSecret::from_bytes(seed).expect("secret");
    ReceiverKeys::from_receiver_secret(secret).expect("keys")
}

fn make_card(keys: &ReceiverKeys) -> ReceiverCard {
    let card = keys.export_receiver_card().expect("card");
    card.verify().expect("verify");
    card
}

fn make_req(keys: &ReceiverKeys, chain_id: u32) -> PaymentRequest {
    PaymentRequest::generate(
        keys,
        RequestParams {
            amount: Some(777),
            expiry_seconds: 600,
            memo: Some("phase9".to_string()),
            payment_id: Some([0x42u8; 16]),
        },
        chain_id,
    )
    .expect("request")
}

fn pub_out(output: &TxStealthOutput) -> PubOut {
    PubOut {
        r_pub: output.r_pub,
        owner_tag: output.owner_tag,
        tag16: output.tag16,
        c_amount: output.c_amount,
    }
}

fn req_tag16(keys: &ReceiverKeys, req: &PaymentRequest, output: &PubOut) -> u16 {
    let r_pub = decode_r_pub(&output.r_pub).expect("r pub");
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &r_pub).expect("dh");
    let k_dh = derive_k_dh_with_req(&dh, &req.req_id);
    compute_tag16_with_req(&k_dh, &req.req_id)
}

fn card_tag16(keys: &ReceiverKeys, aid: &[u8; 32], serial_id: u32, output: &PubOut) -> u16 {
    let r_pub = decode_r_pub(&output.r_pub).expect("r pub");
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &r_pub).expect("dh");
    let k_dh = derive_k_dh(&dh);
    let leaf_ad = compute_leaf_ad(
        aid,
        serial_id,
        &output.r_pub,
        &output.owner_tag,
        &output.c_amount,
    );
    compute_tag16(&k_dh, &leaf_ad)
}

fn rotate_card(keys: &mut ReceiverKeys) -> ReceiverCard {
    let card = keys.rotate_view().expect("rotate");
    card.verify().expect("verify");
    card
}

fn has_no_overclaim(text: &str) {
    for item in [
        "is a final public verifier",
        "is the final public verifier",
        "is protocol-complete",
        "final authority closure is complete",
    ] {
        assert!(!text.contains(item), "forbidden text present: {item}");
    }
}

fn rebuild_def(
    definition: &z00z_core::AssetDefinition,
    serial_id: u32,
) -> z00z_core::AssetDefinition {
    z00z_core::AssetDefinition::new(
        [0u8; 32],
        definition.class,
        format!("{}-{serial_id}", definition.name),
        definition.symbol.clone(),
        definition.decimals,
        definition.serials,
        definition.nominal,
        definition.domain_name.clone(),
        definition.version,
        definition.crypto_version,
        definition.policy_flags,
        definition.metadata.clone(),
    )
    .expect("canonical test definition")
}

fn make_wire(serial_id: u32, keys: &ReceiverKeys) -> AssetWire {
    let mut asset = asset_from_dev_class(AssetClass::Coin, 0, 100).expect("asset");
    let def = rebuild_def(asset.definition.as_ref(), serial_id);
    asset.definition = Arc::new(def);

    let card = keys.export_receiver_card().expect("card");
    let tx_seed = derive_output_nonce(&asset.definition.id, asset.serial_id);
    let mut sender = SenderWallet::new([0x71u8; 32]);
    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut sender,
        &tx_seed,
        0,
        asset.amount,
        &asset.definition.id,
    )
    .expect("output");

    asset.commitment = z00z_crypto::Commitment::from_bytes(&output.c_amount)
        .expect("commit")
        .0;
    asset.owner_pub = None;
    asset.owner_signature = None;
    asset.r_pub = Some(output.r_pub);
    asset.owner_tag = Some(output.owner_tag);
    asset.enc_pack = Some(output.enc_pack);
    asset.tag16 = output.tag16;
    asset.leaf_ad_id = Some(asset.definition.id);

    let scanner = StealthOutputScanner::from_keys(keys);
    let ScanResult::Mine { wallet_output } = scanner.scan_leaf(&asset) else {
        panic!("owned leaf expected");
    };
    let blinding = Z00ZScalar::try_from_bytes(wallet_output.blinding.expect("blinding"))
        .expect("blinding bytes");
    asset.range_proof = Some(create_range_proof(asset.amount, &blinding, 64, 0).expect("proof"));

    let mut wire = AssetWire::from_asset(&asset);
    wire.secret = None;
    wire
}

fn pkg_bytes(pkg: &ClaimTxPackage) -> Vec<u8> {
    z00z_utils::codec::JsonCodec
        .serialize(pkg)
        .expect("pkg bytes")
}

fn req_roundtrip(req: &PaymentRequest, pins: &mut PinnedReceiverCards, chain_id: u32) {
    req.verify().expect("verify");
    assert_eq!(
        req.validate_all(pins, chain_id).expect("first"),
        ValidationOutcome::RequiresUserConfirmation
    );
    assert_eq!(
        req.validate_all(pins, chain_id).expect("second"),
        ValidationOutcome::Approved
    );
}

#[test]
fn test_ex_s5_req_build() {
    let chain_id = 17u32;
    let tx_digest = [0x31u8; 32];
    let aid = [0x32u8; 32];
    let keys = make_keys([0x11u8; 32]);
    let req = make_req(&keys, chain_id);
    let wire = encode_request_compact(&req);
    let parsed = decode_request_compact(&wire).expect("decode");
    let mut pins = PinnedReceiverCards::new();
    req_roundtrip(&parsed, &mut pins, chain_id);

    let card = make_card(&keys);
    let mut sender = SenderWallet::new([0x33u8; 32]);
    let output = build_tx_stealth_output_validated(
        &card,
        Some(&parsed),
        BuildCheck {
            pins: &mut pins,
            chain_id,
        },
        &mut sender,
        &tx_digest,
        3,
        777,
        &aid,
    )
    .expect("validated output");
    let view = pub_out(&output);

    assert_eq!(view.tag16, Some(req_tag16(&keys, &parsed, &view)));
    assert_ne!(view.r_pub, [0u8; 32]);
    assert_ne!(view.owner_tag, [0u8; 32]);
    assert_ne!(view.c_amount, [0u8; 32]);

    let mut fresh_pins = PinnedReceiverCards::new();
    let mut bad_sender = SenderWallet::new([0x34u8; 32]);
    let err = build_tx_stealth_output_validated(
        &card,
        Some(&parsed),
        BuildCheck {
            pins: &mut fresh_pins,
            chain_id,
        },
        &mut bad_sender,
        &tx_digest,
        3,
        777,
        &aid,
    )
    .expect_err("validation must precede build");
    assert_eq!(err, StealthError::InvalidStealthInput);
}

#[test]
fn test_ex_s5_card_pay() {
    let tx_digest = [0x41u8; 32];
    let aid = [0x42u8; 32];
    let serial_id = 0u32;
    let keys = make_keys([0x21u8; 32]);
    let card = make_card(&keys);
    let card_compact = encode_card_compact(&card);
    let decoded = decode_card_compact(&card_compact).expect("decode");
    decoded.validate_structure().expect("shape");
    decoded.validate_ecc_points().expect("ecc");
    decoded.validate_signature().expect("sig");

    let mut pins = PinnedReceiverCards::new();
    let first = pins
        .verify_or_pin(&decoded, Some("offline-dir"))
        .expect("pin");
    assert_eq!(first, VerifyResult::NewPin);

    let mut sender = SenderWallet::new([0x43u8; 32]);
    let output = build_tx_output_unchecked(
        &decoded,
        None,
        &mut sender,
        &tx_digest,
        serial_id,
        555,
        &aid,
    )
    .expect("card output");
    let view = pub_out(&output);
    assert_eq!(view.tag16, Some(card_tag16(&keys, &aid, serial_id, &view)));

    let mut bad_bytes = decoded.canonical_encoding();
    let sig_at = bad_bytes.len().saturating_sub(64);
    bad_bytes[sig_at] ^= 0x01;
    let bad = ReceiverCard::from_canonical_encoding(&bad_bytes).expect("bad card");
    assert!(matches!(
        bad.validate_signature(),
        Err(ReceiverCardError::VerifyFailed | ReceiverCardError::InvalidSignature)
    ));

    let mut raw_sender = SenderWallet::new([0x44u8; 32]);
    let misuse = build_tx_output_unchecked(
        &bad,
        None,
        &mut raw_sender,
        &tx_digest,
        serial_id,
        555,
        &aid,
    )
    .expect("raw builder still builds invalid input");
    assert!(misuse.tag16.is_some());
}

#[test]
fn test_ex_s5_tofu_drift() {
    let mut first_keys = make_keys([0x51u8; 32]);
    let first = make_card(&first_keys);
    let mut pins = PinnedReceiverCards::new();

    assert_eq!(
        pins.verify_or_pin(&first, Some("merchant-a"))
            .expect("first"),
        VerifyResult::NewPin
    );
    assert_eq!(
        pins.verify_or_pin(&first, Some("merchant-a"))
            .expect("second"),
        VerifyResult::Verified
    );

    let drift = rotate_card(&mut first_keys);
    assert_eq!(drift.owner_handle, first.owner_handle);
    assert_eq!(drift.identity_pk, first.identity_pk);
    assert_ne!(drift.view_pk, first.view_pk);

    let drift_res = pins
        .verify_or_pin(&drift, Some("merchant-a"))
        .expect("drift");

    assert!(matches!(
        drift_res,
        VerifyResult::ViewKeyChanged {
            requires_confirmation: true,
            ..
        }
    ));
    assert_eq!(
        pins.get(&first.owner_handle).expect("pin").view_pk,
        first.view_pk,
        "rotation must stay pending until explicit confirmation"
    );
}

#[test]
fn test_ex_s5_record_pub() {
    let card = make_card(&make_keys([0x61u8; 32]));
    let record = ReceiverCardRecord::new(&card, card.canonical_encoding(), 7).expect("record");
    let decoded = verify_receiver_card_record(&record, None).expect("verify");
    assert_eq!(decoded.canonical_encoding(), record.receiver_card_bytes);

    let compact = record.to_compact().expect("compact");
    let roundtrip = ReceiverCardRecord::from_compact(&compact, None).expect("roundtrip");
    assert_eq!(roundtrip, record);

    let revoked = record.clone().revoked();
    assert!(matches!(
        verify_receiver_card_record(&revoked, None),
        Err(CardRecordError::Revoked)
    ));
    assert!(matches!(
        verify_receiver_card_record(&record, Some(8)),
        Err(CardRecordError::StaleEpoch)
    ));
}

#[test]
fn test_ex_s5_closure_demo() {
    let pkg = test_s5_sender_examples_support::make_pkg(
        CHAIN_ID,
        make_wire(7, &make_keys([0x72u8; 32])),
        &make_keys([0x72u8; 32]),
    );
    let report = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(
        report.valid,
        "live transitional helper must accept the canonical package"
    );
    assert!(report.reject_class.is_empty());

    let steps = report.report.expect("report");
    assert!(steps.nullifier_checked);
    assert!(steps.card_checked);
    assert!(steps.leaf_checked);
    assert!(steps.proof_checked);
    assert!(steps.authority_checked);
    assert!(steps.owner_attest_checked);
    assert!(steps.digest_checked);

    let mut bad_pkg = pkg.clone();
    bad_pkg.tx.auth.claim_authority_sig_hex = "ab".repeat(3);
    test_s5_sender_examples_support::sync_pkg(&mut bad_pkg);

    let bad = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&bad_pkg));
    assert!(!bad.valid);
    assert_eq!(bad.reject_class, "claim_authority_invalid");

    let bad_steps = bad.report.expect("bad report");
    assert!(bad_steps.proof_checked);
    assert!(!bad_steps.authority_checked);
    assert!(!bad_steps.owner_attest_checked);
    assert!(!bad_steps.digest_checked);

    assert!(CLAIM_TX.contains("const ZERO_ROOT: [u8; 32] = [0u8; 32];"));
    assert!(CLAIM_TX.contains("impl ClaimTxVerifier for ClaimTxVerifierImpl"));
    assert!(CLAIM_TX.contains("fn verify_claim_proof("));
    assert!(CLAIM_TX.contains("fn verify_claim_authority("));
    assert_eq!(test_s5_sender_examples_support::ZERO_ROOT, [0u8; 32]);

    let Some(post_spec) = load_optional_spec("specs/011-z00z-ecc-spec-5/post-spec-5.md") else {
        eprintln!("skip post-spec assertions: missing specs/011-z00z-ecc-spec-5/post-spec-5.md");
        return;
    };

    assert!(post_spec.contains(
        "does not yet expose a final verified Stage-1 authority bundle as a live code path"
    ));
    assert!(post_spec.contains("typed authority-signature path plus `ZERO_ROOT`"));
    assert!(post_spec.contains("verify_spend_witness_gate"));
    assert!(post_spec.contains("verify_fee_matches_formula"));
    assert!(post_spec.contains("verify_plaintext_balance_with_fee"));
    has_no_overclaim(&post_spec);
}

#[test]
fn test_ex_s5_leaf_bridge() {
    let _guard = RangeProofEnvGuard::new();
    let keys = make_keys([0x81u8; 32]);
    let card = make_card(&keys);
    let amount = 888u64;
    let serial_id = serials_from_dev_class(AssetClass::Coin).expect("dev coin serials") - 1;
    let leaf = build_card_stealth_leaf(&card, amount, serial_id).expect("leaf");
    let (canon, runtime) = test_s5_sender_examples_support::pair_leaf(&leaf, amount);

    let pack = receiver_scan_leaf(&keys, &canon)
        .expect("scan")
        .expect("owned pack");
    let report = receiver_scan_report(&keys, &canon).expect("report");
    let scan = StealthOutputScanner::from_keys(&keys).scan_leaf(&runtime);
    let run_report = scan.recv_report();

    assert_eq!(pack.value, amount);
    assert_eq!(canon.asset_id, leaf.asset_id);
    assert_eq!(canon.serial_id, serial_id);
    assert_eq!(canon.owner_tag, leaf.owner_tag);
    assert_eq!(runtime.r_pub, Some(leaf.r_pub));
    assert_eq!(runtime.owner_tag, Some(leaf.owner_tag));
    assert_eq!(runtime.tag16, Some(leaf.tag16));
    assert_eq!(report.status, ReceiveStatus::Detected);
    assert_eq!(report.reject, None);
    assert_eq!(report.next, ReceiveNext::ReportOnly);
    assert_eq!(report, run_report);
    assert!(
        !report.next.should_persist(),
        "successful receive means detect-and-decrypt, not automatic claim"
    );

    let ScanResult::Mine { wallet_output } = scan else {
        panic!("owned runtime asset expected");
    };

    assert_eq!(wallet_output.amount, amount);
    assert_eq!(wallet_output.asset_id, runtime.asset_id());
    assert_eq!(wallet_output.serial_id, serial_id);
    assert_eq!(wallet_output.asset_secret, Some(pack.s_out));
    assert_eq!(wallet_output.blinding, Some(pack.blinding));
    assert_eq!(wallet_output.r_pub, leaf.r_pub);
    assert_eq!(wallet_output.owner_tag, leaf.owner_tag);
}
