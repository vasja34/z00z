use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use z00z_utils::time::{SystemTimeProvider, TimeProvider};
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::request::{
        create_invoice_for_merchant, decode_request_compact, encode_request_compact,
        generate_req_id, handle_payment_request_expiry, to_nfc_ndef,
    },
    receiver::{
        PaymentRequest, PaymentRequestError, PinnedReceiverCards, RequestParams,
        ValidatePaymentRequest, ValidationOutcome, ValidityStatus, VerifyResult,
    },
    stealth::kdf::compute_tag16_with_req,
};

const REQ_ID_OFFSET: usize = 1 + 32 + 32 + 32;
const CHAIN_ID_OFFSET: usize = REQ_ID_OFFSET + 32;

fn unix_timestamp() -> u64 {
    SystemTimeProvider.compat_unix_timestamp()
}

fn make_keys() -> ReceiverKeys {
    let sec = ReceiverSecret::generate().expect("secret");
    ReceiverKeys::from_receiver_secret(sec).expect("keys")
}

fn make_req(chain_id: u32) -> PaymentRequest {
    let keys = make_keys();
    PaymentRequest::generate(
        &keys,
        RequestParams {
            amount: Some(42),
            expiry_seconds: 600,
            memo: Some("invoice".to_string()),
            payment_id: Some([7u8; 16]),
        },
        chain_id,
    )
    .expect("request")
}

#[test]
fn test_payment_request_create() {
    let request = make_req(1);
    assert_eq!(request.version, 1);
    assert_ne!(request.req_id, [0u8; 32]);
}

#[test]
fn test_payment_request_serialize() {
    let request = make_req(7);
    let encoded = request.canonical_encoding();
    let decoded = PaymentRequest::from_canonical_encoding(&encoded).expect("decode");
    assert_eq!(decoded.owner_handle, request.owner_handle);
    assert_eq!(decoded.amount, request.amount);
}

#[test]
fn test_payment_request_amount_optional() {
    let keys = make_keys();
    let request = PaymentRequest::generate(
        &keys,
        RequestParams {
            amount: None,
            expiry_seconds: 60,
            memo: None,
            payment_id: None,
        },
        11,
    )
    .expect("request");
    assert_eq!(request.amount, None);
}

#[test]
fn test_req_id_uniqueness() {
    let first = generate_req_id().expect("id1");
    let second = generate_req_id().expect("id2");
    assert_ne!(first, second);
}

#[test]
fn test_req_id_not_zero() {
    let req_id = generate_req_id().expect("id");
    assert_ne!(req_id, [0u8; 32]);
}

#[test]
fn test_request_canonical_encoding_deterministic() {
    let request = make_req(42);
    let first = request.canonical_encoding_unsigned();
    let second = request.canonical_encoding_unsigned();
    assert_eq!(first, second);
}

#[test]
fn test_request_sign_verify() {
    let request = make_req(44);
    request.verify().expect("verify");
}

#[test]
fn test_request_invalid_signature() {
    let mut request = make_req(45);
    request.signature[10] ^= 0xFF;
    let result = request.verify();
    assert!(result.is_err());
}

#[test]
fn test_validate_wrong_version() {
    let mut request = make_req(2);
    request.version = 9;
    let mut pins = PinnedReceiverCards::new();
    let result = request.validate_all(&mut pins, 2);
    assert!(matches!(
        result,
        Err(PaymentRequestError::UnsupportedVersion)
    ));
}

#[test]
fn test_validate_wrong_chain_id() {
    let request = make_req(3);
    let mut pins = PinnedReceiverCards::new();
    let result = request.validate_all(&mut pins, 4);
    assert!(matches!(result, Err(PaymentRequestError::WrongChainId)));
}

#[test]
fn test_validate_expired() {
    let mut request = make_req(5);
    request.expiry = unix_timestamp().saturating_sub(1);
    let mut pins = PinnedReceiverCards::new();
    let result = request.validate_all(&mut pins, 5);
    assert!(matches!(result, Err(PaymentRequestError::RequestExpired)));
}

#[test]
fn test_validate_tofu_new_identity() {
    let request = make_req(6);
    let mut pins = PinnedReceiverCards::new();
    let result = request.validate_all(&mut pins, 6).expect("validated");
    assert_eq!(result, ValidationOutcome::RequiresUserConfirmation);
}

#[test]
fn test_validate_tofu_verified() {
    let request = make_req(8);
    let mut pins = PinnedReceiverCards::new();

    let first = request.validate_all(&mut pins, 8).expect("first");
    assert_eq!(first, ValidationOutcome::RequiresUserConfirmation);

    let second = request.validate_all(&mut pins, 8).expect("second");
    assert_eq!(second, ValidationOutcome::Approved);
}

#[test]
fn test_req_validate_trait() {
    let request = make_req(14);
    let mut pins = PinnedReceiverCards::new();
    let first = <PaymentRequest as ValidatePaymentRequest>::validate_all(&request, &mut pins, 14)
        .expect("first");
    assert_eq!(first, ValidationOutcome::RequiresUserConfirmation);

    let second = <PaymentRequest as ValidatePaymentRequest>::validate_all(&request, &mut pins, 14)
        .expect("second");
    assert_eq!(second, ValidationOutcome::Approved);
}

#[test]
fn test_validate_tofu_identity_mismatch() {
    let keys_b = make_keys();
    let request_a = make_req(9);
    let mut request_b = PaymentRequest::generate(
        &keys_b,
        RequestParams {
            amount: Some(9),
            expiry_seconds: 300,
            memo: Some("b".to_string()),
            payment_id: None,
        },
        9,
    )
    .expect("request b");
    request_b.owner_handle = request_a.owner_handle;
    request_b.sign(keys_b.reveal_identity_sk()).expect("resign");

    let mut pins = PinnedReceiverCards::new();
    let _ = request_a.validate_all(&mut pins, 9).expect("first");
    let second = request_b.validate_all(&mut pins, 9).expect("second");
    assert_eq!(second, ValidationOutcome::IdentityMismatch);
}

#[test]
fn test_request_encoding_roundtrip() {
    let request = make_req(10);
    let encoded = encode_request_compact(&request);
    let decoded = decode_request_compact(&encoded).expect("decode");
    assert_eq!(decoded.req_id, request.req_id);
    assert_eq!(decoded.chain_id, request.chain_id);
}

#[test]
fn test_chain_rebinding_breaks_sig() {
    let request = make_req(24);
    let mut raw = URL_SAFE_NO_PAD
        .decode(encode_request_compact(&request))
        .expect("decode compact");
    raw[CHAIN_ID_OFFSET] ^= 0x01;
    let tampered = URL_SAFE_NO_PAD.encode(raw);

    let decoded = decode_request_compact(&tampered).expect("structural decode");
    assert_ne!(decoded.chain_id, request.chain_id);
    assert!(matches!(
        decoded.verify(),
        Err(PaymentRequestError::VerifyFailed)
    ));
}

#[test]
fn test_request_id_breaks_sig() {
    let request = make_req(25);
    let mut raw = URL_SAFE_NO_PAD
        .decode(encode_request_compact(&request))
        .expect("decode compact");
    raw[REQ_ID_OFFSET] ^= 0x01;
    let tampered = URL_SAFE_NO_PAD.encode(raw);

    let decoded = decode_request_compact(&tampered).expect("structural decode");
    assert_ne!(decoded.req_id, request.req_id);
    assert!(matches!(
        decoded.verify(),
        Err(PaymentRequestError::VerifyFailed)
    ));
}

#[test]
fn test_generate_payment_request() {
    let keys = make_keys();
    let request = PaymentRequest::generate(
        &keys,
        RequestParams {
            amount: Some(5),
            expiry_seconds: 90,
            memo: Some("a".to_string()),
            payment_id: None,
        },
        12,
    )
    .expect("request");

    assert_eq!(request.chain_id, 12);
    assert_eq!(request.amount, Some(5));
    request.verify().expect("verify");
}

#[test]
fn test_request_expiry_handling() {
    let mut request = make_req(13);
    request.expiry = unix_timestamp().saturating_add(30);
    let status = request.check_validity();
    assert!(matches!(status, ValidityStatus::ExpiringSoon(_)));

    request.expiry = unix_timestamp().saturating_sub(1);
    let status = request.check_validity();
    assert_eq!(status, ValidityStatus::Expired);
}

#[test]
fn test_request_expiry_equal_now() {
    let mut request = make_req(16);
    request.expiry = unix_timestamp();
    assert!(request.is_expired());
    assert_eq!(request.check_validity(), ValidityStatus::Expired);
}

#[test]
fn test_remaining_seconds_positive() {
    let mut request = make_req(18);
    request.expiry = unix_timestamp().saturating_add(10);
    assert!(request.remaining_seconds() > 0);
}

#[test]
fn test_remaining_seconds_negative() {
    let mut request = make_req(19);
    request.expiry = unix_timestamp().saturating_sub(2);
    assert!(request.remaining_seconds() < 0);
}

#[test]
fn test_remaining_seconds_zero() {
    let mut request = make_req(20);
    request.expiry = unix_timestamp();
    assert_eq!(request.remaining_seconds(), 0);
}

#[test]
fn test_validate_revoked_identity_pin() {
    let request = make_req(17);
    let mut pins = PinnedReceiverCards::new();

    let _ = pins.verify_request_identity(&request.owner_handle, &request.identity_pk);
    pins.revoke(&request.owner_handle);

    let result = request.validate_all(&mut pins, 17);
    assert!(matches!(result, Err(PaymentRequestError::PinRevoked)));
}

#[test]
fn test_request_with_amount() {
    let request = make_req(14);
    assert_eq!(request.amount, Some(42));
}

#[test]
fn test_request_without_amount() {
    let keys = make_keys();
    let request = PaymentRequest::generate(
        &keys,
        RequestParams {
            amount: None,
            expiry_seconds: 120,
            memo: None,
            payment_id: None,
        },
        15,
    )
    .expect("request");

    assert_eq!(request.amount, None);
}

#[test]
fn test_tag16_req_unique() {
    let k_dh = [19u8; 32];
    let req_a = [1u8; 32];
    let req_b = [2u8; 32];

    let tag_a = compute_tag16_with_req(&k_dh, &req_a);
    let tag_b = compute_tag16_with_req(&k_dh, &req_b);
    assert_ne!(tag_a, tag_b);
}

#[test]
fn test_nfc_ndef_encoding() {
    let request = make_req(21);
    let ndef = to_nfc_ndef(&request);

    assert!(ndef.uri().starts_with("z00z:pay?"));
}

#[test]
fn test_nfc_ndef_roundtrip() {
    let request = make_req(22);
    let ndef = to_nfc_ndef(&request);
    let payload = ndef
        .uri()
        .strip_prefix("z00z:pay?data=")
        .expect("data query");
    let decoded = decode_request_compact(payload).expect("decode");

    assert_eq!(decoded.req_id, request.req_id);
}

#[test]
fn test_merchant_invoice_generation() {
    let keys = make_keys();
    let card = keys.export_receiver_card().expect("card");
    let request = create_invoice_for_merchant(&card, keys.reveal_identity_sk(), 77, 500, None)
        .expect("request");

    assert_eq!(request.amount, Some(500));
    assert_eq!(request.chain_id, 77);
    assert_eq!(request.owner_handle, card.owner_handle);
    request.verify().expect("verify");
}

#[test]
fn test_request_verify_fields() {
    let keys = make_keys();
    let card = keys.export_receiver_card().expect("card");
    let request = create_invoice_for_merchant(
        &card,
        keys.reveal_identity_sk(),
        88,
        750,
        Some("memo".to_string()),
    )
    .expect("request");

    request.verify().expect("verified");
    assert_ne!(request.req_id, [0u8; 32]);
    assert_eq!(request.amount, Some(750));
    assert_eq!(request.owner_handle, card.owner_handle);
}

#[test]
fn test_validate_expired_request() {
    let keys = make_keys();
    let card = keys.export_receiver_card().expect("card");
    let mut request = create_invoice_for_merchant(&card, keys.reveal_identity_sk(), 99, 100, None)
        .expect("request");
    request.expiry = unix_timestamp().saturating_sub(1);
    request.sign(keys.reveal_identity_sk()).expect("resign");

    assert!(request.verify().is_ok());
    assert!(matches!(request.check_validity(), ValidityStatus::Expired));
}

#[tokio::test]
async fn test_payment_request_expiry_handler() {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    let keys = make_keys();
    let card = keys.export_receiver_card().expect("card");
    let mut request = create_invoice_for_merchant(&card, keys.reveal_identity_sk(), 66, 12, None)
        .expect("request");
    request.expiry = unix_timestamp().saturating_sub(1);
    request.sign(keys.reveal_identity_sk()).expect("resign");

    let called = Arc::new(AtomicBool::new(false));
    let marker = Arc::clone(&called);
    handle_payment_request_expiry(request, move |_| {
        marker.store(true, Ordering::SeqCst);
    })
    .await;

    assert!(called.load(Ordering::SeqCst));
}

#[test]
fn test_tag16_req_id_unique() {
    let dh = [11u8; 32];
    let req1 = [1u8; 32];
    let req2 = [2u8; 32];

    let tag1 = compute_tag16_with_req(&dh, &req1);
    let tag2 = compute_tag16_with_req(&dh, &req2);
    assert_ne!(tag1, tag2);
}

#[test]
fn test_pin_tofu() {
    let keys = make_keys();
    let card = keys.export_receiver_card().expect("card");
    let mut pins = PinnedReceiverCards::new();

    let result = pins.verify_or_pin(&card, None).expect("pin");
    assert!(matches!(
        result,
        z00z_wallets::receiver::VerifyResult::NewPin
    ));
    assert_eq!(pins.len(), 1);
    assert!(pins.get(&card.owner_handle).is_some());
}

#[test]
fn test_tofu_change_flow() {
    // Test Flow:
    // 1) Pin initial receiver card (TOFU first-use).
    // 2) Rotate view key and detect change.
    // 3) Simulate user decline (pin must not update).
    // 4) Confirm rotation and verify updated pin is accepted.
    let mut keys = make_keys();
    let card_0 = keys.export_receiver_card().expect("card v0");

    let mut pins = PinnedReceiverCards::new();
    let first = pins.verify_or_pin(&card_0, None).expect("first pin");
    assert!(matches!(first, VerifyResult::NewPin));

    let card_1 = keys.rotate_view().expect("rotate view");

    let changed = pins.verify_or_pin(&card_1, None).expect("changed");
    match changed {
        VerifyResult::ViewKeyChanged {
            old_pk,
            new_pk,
            requires_confirmation,
        } => {
            assert_eq!(old_pk, card_0.view_pk);
            assert_eq!(new_pk, card_1.view_pk);
            assert!(requires_confirmation);
        }
        _ => panic!("expected ViewKeyChanged"),
    }

    let declined = pins.verify_or_pin(&card_1, None).expect("declined path");
    assert!(matches!(declined, VerifyResult::ViewKeyChanged { .. }));

    pins.confirm_rotation(&card_1.owner_handle, &card_1.view_pk);
    let accepted = pins.verify_or_pin(&card_1, None).expect("accepted");
    assert!(matches!(accepted, VerifyResult::Verified));
    let pin = pins.get(&card_1.owner_handle).expect("pin entry");
    assert_eq!(pin.view_pk, card_1.view_pk);
}

#[test]
fn test_expiry_reject() {
    // Test Flow:
    // 1) Merchant creates signed invoice-style PaymentRequest.
    // 2) Sender validates it successfully before expiry.
    // 3) Request becomes expired and is re-signed.
    // 4) Sender-side validation must reject with RequestExpired.
    let keys = make_keys();
    let card = keys.export_receiver_card().expect("card");
    let mut request = create_invoice_for_merchant(
        &card,
        keys.reveal_identity_sk(),
        6,
        2_000,
        Some("invoice-6-2".to_string()),
    )
    .expect("request");

    request.verify().expect("prepare before expiry");
    assert_eq!(request.amount, Some(2_000));

    request.expiry = unix_timestamp().saturating_sub(1);
    request.sign(keys.reveal_identity_sk()).expect("resign");

    assert!(request.verify().is_ok());
    assert!(matches!(request.check_validity(), ValidityStatus::Expired));
}
