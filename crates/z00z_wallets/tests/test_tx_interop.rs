use std::path::PathBuf;

#[path = "test_inc/test_mod.rs"]
mod test_common;

use test_common::managed_test_output_root;
use z00z_utils::io::{create_dir_all, write_file};
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::receiver_card::{decode_card_compact, encode_card_compact},
    receiver::request::{
        create_invoice_for_merchant, decode_request_compact, encode_request_compact,
    },
    receiver::{
        PaymentRequest, PaymentRequestError, PinnedReceiverCards, ReceiverCard, ReceiverCardError,
        ValidatePaymentRequest, ValidateReceiverCard, ValidationOutcome,
    },
};

fn out_dir() -> PathBuf {
    managed_test_output_root("e2e15")
}

fn mk_keys() -> ReceiverKeys {
    let mut sec = [0u8; 32];
    for (idx, byte) in sec.iter_mut().enumerate() {
        *byte = idx as u8 + 3;
    }
    let recv = ReceiverSecret::from_bytes(sec).expect("receiver secret");
    ReceiverKeys::from_receiver_secret(recv).expect("receiver keys")
}

#[test]
fn test_stage4_interop() {
    if cfg!(debug_assertions) {
        return;
    }

    let keys = mk_keys();
    let chain_id = 77u32;

    let card = keys.export_receiver_card().expect("create card");
    let card_raw = card.canonical_encoding();
    let card_dec = ReceiverCard::from_canonical_encoding(&card_raw).expect("decode card");
    card_dec.validate_structure().expect("card structure");
    card_dec.validate_signature().expect("card signature");
    let card_compact = encode_card_compact(&card_dec);
    let card_compact_dec = decode_card_compact(&card_compact).expect("decode card compact");
    assert_eq!(
        card_compact_dec, card_dec,
        "card compact roundtrip must match"
    );

    let req = create_invoice_for_merchant(
        &card_dec,
        keys.reveal_identity_sk(),
        chain_id,
        42,
        Some("interop".to_string()),
    )
    .expect("create request");
    req.verify().expect("req signature");
    let req_raw = req.canonical_encoding();
    let req_dec = PaymentRequest::from_canonical_encoding(&req_raw).expect("decode req");
    req_dec.verify().expect("verify req decoded");
    let req_compact = encode_request_compact(&req_dec);
    let req_compact_dec = decode_request_compact(&req_compact).expect("decode req compact");
    assert_eq!(
        req_compact_dec, req_dec,
        "request compact roundtrip must match"
    );

    let mut pins = PinnedReceiverCards::new();
    let chk1 = req_dec
        .validate_all(&mut pins, chain_id)
        .expect("validate req first");
    assert_eq!(
        chk1,
        ValidationOutcome::RequiresUserConfirmation,
        "first contact must require confirmation"
    );
    let chk2 = req_dec
        .validate_all(&mut pins, chain_id)
        .expect("validate req second");
    assert_eq!(
        chk2,
        ValidationOutcome::Approved,
        "second check must approve"
    );
    let bad_chain = req_dec.validate_all(&mut pins, chain_id + 1);
    assert!(matches!(bad_chain, Err(PaymentRequestError::WrongChainId)));

    let mut bad_card_dom = card_dec.clone();
    bad_card_dom.owner_handle[0] ^= 0x01;
    assert!(
        matches!(
            bad_card_dom.validate_signature(),
            Err(ReceiverCardError::VerifyFailed)
        ),
        "card domain mismatch must fail signature"
    );

    let mut bad_req_dom = req_dec.clone();
    bad_req_dom.req_id[0] ^= 0x01;
    assert!(
        matches!(bad_req_dom.verify(), Err(PaymentRequestError::VerifyFailed)),
        "request domain mismatch must fail signature"
    );

    let mut bad_card_ver = card_dec.clone();
    bad_card_ver.version = 9;
    assert!(matches!(
        bad_card_ver.validate_structure(),
        Err(ReceiverCardError::UnsupportedVersion)
    ));

    let mut bad_req_ver = req_dec.clone();
    bad_req_ver.version = 9;
    let mut pins_ver = PinnedReceiverCards::new();
    assert!(matches!(
        bad_req_ver.validate_all(&mut pins_ver, chain_id),
        Err(PaymentRequestError::UnsupportedVersion)
    ));

    let mut bad_card_len = card_raw.clone();
    bad_card_len.pop();
    let bad_card_len_err =
        ReceiverCard::from_canonical_encoding(&bad_card_len).expect_err("bad card len");
    assert!(matches!(
        bad_card_len_err,
        ReceiverCardError::InvalidCardBytes
    ));

    let mut bad_req_len = req_raw.clone();
    bad_req_len.pop();
    let bad_req_len_err =
        PaymentRequest::from_canonical_encoding(&bad_req_len).expect_err("bad req len");
    assert!(matches!(
        bad_req_len_err,
        PaymentRequestError::InvalidRequestBytes | PaymentRequestError::InvalidRequestSize
    ));

    create_dir_all(out_dir()).expect("mkdir outputs/tests/e2e15");
    let corpus = serde_json::json!({
        "test": "E2E-15",
        "card_hex": hex::encode(&card_raw),
        "request_hex": hex::encode(&req_raw),
        "malformed": {
            "card_bad_len_hex": hex::encode(&bad_card_len),
            "request_bad_len_hex": hex::encode(&bad_req_len),
            "card_bad_version": bad_card_ver.version,
            "request_bad_version": bad_req_ver.version,
            "card_domain_flip": true,
            "request_domain_flip": true
        }
    });
    let corpus_bytes = serde_json::to_vec_pretty(&corpus).expect("json corpus");
    write_file(out_dir().join("e2e15_corpus.json"), &corpus_bytes).expect("write corpus");

    let mut out = String::from("E2E-15 validators\n");
    out.push_str("card_decode=1 card_verify=1\n");
    out.push_str("card_compact=1\n");
    out.push_str("req_decode=1 req_verify=1\n");
    out.push_str("req_compact=1\n");
    out.push_str(&format!("chain_ok_first={:?}\n", chk1));
    out.push_str(&format!("chain_ok_second={:?}\n", chk2));
    out.push_str("chain_bad=WrongChainId\n");
    out.push_str("bad_card_ver=UnsupportedVersion\n");
    out.push_str("bad_req_ver=UnsupportedVersion\n");
    out.push_str("bad_card_len=reject\n");
    out.push_str("bad_req_len=reject\n");
    out.push_str("bad_card_domain=VerifyFailed\n");
    out.push_str("bad_req_domain=VerifyFailed\n");
    write_file(out_dir().join("e2e15_validate.txt"), out.as_bytes()).expect("write validate");
}
