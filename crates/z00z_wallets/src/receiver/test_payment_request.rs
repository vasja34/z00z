use super::{
    generate_req_id, PaymentRequest, PaymentRequestError, RequestMetadata, RequestParams,
    ValidatePaymentRequest, ValidationOutcome, ValidityStatus, REQ_VER_1,
};
use crate::key::{ReceiverKeys, ReceiverSecret};
use crate::receiver::card::receiver_card_trust::PinnedReceiverCards;

fn make_keys() -> ReceiverKeys {
    let secret = ReceiverSecret::generate().expect("secret");
    ReceiverKeys::from_receiver_secret(secret).expect("keys")
}

fn signed_req(keys: &ReceiverKeys) -> PaymentRequest {
    let mut req = PaymentRequest {
        version: REQ_VER_1,
        owner_handle: keys.owner_handle,
        view_pk: keys.view_pk.as_bytes().try_into().expect("view pk"),
        identity_pk: keys.identity_pk.as_bytes().try_into().expect("identity pk"),
        req_id: generate_req_id().expect("req id"),
        chain_id: 7,
        amount: Some(11),
        expiry: u64::MAX,
        metadata: Some(RequestMetadata {
            memo: Some("memo".to_string()),
            payment_id: None,
            min_confirmations: None,
            return_receiver: None,
            created_at: 1,
        }),
        signature: [0u8; 64],
    };
    req.sign(keys.reveal_identity_sk()).expect("sign");
    req
}

#[test]
fn test_request_validate_wrong_version() {
    let keys = make_keys();
    let mut req = signed_req(&keys);
    req.version = 9;
    let mut pins = PinnedReceiverCards::new();

    assert!(matches!(
        req.validate_all(&mut pins, req.chain_id),
        Err(PaymentRequestError::UnsupportedVersion)
    ));
}

#[test]
fn test_request_validate_view_pk() {
    let keys = make_keys();
    let mut req = signed_req(&keys);
    req.view_pk = [0u8; 32];
    let mut pins = PinnedReceiverCards::new();

    assert!(matches!(
        req.validate_all(&mut pins, req.chain_id),
        Err(PaymentRequestError::IdentityPoint)
    ));
}

#[test]
fn test_request_validate_identity_pk() {
    let keys = make_keys();
    let mut req = signed_req(&keys);
    req.identity_pk = [0u8; 32];
    let mut pins = PinnedReceiverCards::new();

    assert!(matches!(
        req.validate_all(&mut pins, req.chain_id),
        Err(PaymentRequestError::IdentityPoint)
    ));
}

#[test]
fn test_request_verify_bad_signature() {
    let keys = make_keys();
    let mut req = signed_req(&keys);
    req.signature[5] ^= 0x55;

    assert!(matches!(
        req.verify(),
        Err(PaymentRequestError::VerifyFailed | PaymentRequestError::InvalidSignature)
    ));
}

#[test]
fn test_request_verify_owner_handle() {
    let keys = make_keys();
    let mut req = signed_req(&keys);
    req.owner_handle[0] ^= 0x01;

    assert!(matches!(
        req.verify(),
        Err(PaymentRequestError::VerifyFailed)
    ));
}

#[test]
fn test_request_validate_expired() {
    let keys = make_keys();
    let mut req = signed_req(&keys);
    req.expiry = 0;
    let mut pins = PinnedReceiverCards::new();

    assert!(matches!(
        req.validate_all(&mut pins, req.chain_id),
        Err(PaymentRequestError::RequestExpired)
    ));
    assert_eq!(req.check_validity(), ValidityStatus::Expired);
}

#[test]
fn test_validate_chain_pre_signature() {
    let keys = make_keys();
    let mut req = signed_req(&keys);
    req.signature[0] ^= 0xAA;
    let mut pins = PinnedReceiverCards::new();

    assert!(matches!(
        req.validate_all(&mut pins, req.chain_id.wrapping_add(1)),
        Err(PaymentRequestError::WrongChainId)
    ));
}

#[test]
fn test_request_validate_new_identity() {
    let keys = make_keys();
    let req = signed_req(&keys);
    let mut pins = PinnedReceiverCards::new();

    assert_eq!(
        req.validate_all(&mut pins, req.chain_id)
            .expect("validated"),
        ValidationOutcome::RequiresUserConfirmation
    );
}

#[test]
fn test_request_params_default() {
    let params = RequestParams::default();
    assert_eq!(params.amount, None);
    assert_eq!(params.expiry_seconds, 0);
}
