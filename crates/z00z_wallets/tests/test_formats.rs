use std::{fs, path::PathBuf};

use serde::de::DeserializeOwned;
use z00z_core::assets::AssetLeaf;
use z00z_wallets::receiver::{
    PaymentRequest, PaymentRequestError, ReceiverCard, ReceiverCardError,
};
use z00z_wallets::tx::TxPackage;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn read_fixture<T: DeserializeOwned>(name: &str) -> T {
    let path = fixture_root().join(name);
    let raw = fs::read_to_string(path).expect("read fixture");
    serde_json::from_str(&raw).expect("decode fixture")
}

fn card_meta_flag_pos(bytes: &[u8]) -> usize {
    let card_id_flag = bytes[97];
    let mut pos = 98usize;
    if card_id_flag == 1 {
        pos += 16;
    }
    pos
}

fn card_meta_display_len_pos(bytes: &[u8]) -> usize {
    let mut pos = card_meta_flag_pos(bytes);
    assert_eq!(bytes[pos], 1);
    pos += 1;

    assert_eq!(bytes[pos], 1);
    pos + 1
}

fn req_amount_flag_pos() -> usize {
    1 + 32 + 32 + 32 + 32 + 4
}

fn req_meta_flag_pos(bytes: &[u8]) -> usize {
    let mut pos = req_amount_flag_pos();
    assert_eq!(bytes[pos], 1);
    pos += 1 + 8;
    pos + 8
}

fn req_meta_memo_len_pos(bytes: &[u8]) -> usize {
    let mut pos = req_meta_flag_pos(bytes);
    assert_eq!(bytes[pos], 1);
    pos += 1;

    assert_eq!(bytes[pos], 1);
    pos + 1
}

#[test]
fn test_card_fixture_roundtrip() {
    let card: ReceiverCard = read_fixture("receiver_card.json");
    let encoded = card.canonical_encoding();
    let decoded = ReceiverCard::from_canonical_encoding(&encoded).expect("decode card");
    let reencoded = decoded.canonical_encoding();

    assert_eq!(decoded, card);
    assert_eq!(reencoded, encoded);
}

#[test]
fn test_card_bad_flag() {
    let card: ReceiverCard = read_fixture("receiver_card.json");
    let mut encoded = card.canonical_encoding();
    let pos = card_meta_flag_pos(&encoded);
    encoded[pos] = 2;

    let result = ReceiverCard::from_canonical_encoding(&encoded);
    assert!(matches!(result, Err(ReceiverCardError::InvalidCardFlag)));
}

#[test]
fn test_card_bad_len() {
    let card: ReceiverCard = read_fixture("receiver_card.json");
    let mut encoded = card.canonical_encoding();
    encoded.pop();

    let result = ReceiverCard::from_canonical_encoding(&encoded);
    assert!(matches!(result, Err(ReceiverCardError::InvalidCardBytes)));
}

#[test]
fn test_card_bad_opt_len() {
    let card: ReceiverCard = read_fixture("receiver_card.json");
    let mut encoded = card.canonical_encoding();
    let len_pos = card_meta_display_len_pos(&encoded);
    encoded[len_pos..len_pos + 4].copy_from_slice(&(u32::MAX).to_le_bytes());

    let result = ReceiverCard::from_canonical_encoding(&encoded);
    assert!(matches!(
        result,
        Err(ReceiverCardError::InvalidCardBytes)
            | Err(ReceiverCardError::InvalidCardString)
            | Err(ReceiverCardError::InvalidCardSize)
            | Err(ReceiverCardError::InvalidCardFlag)
    ));
}

#[test]
fn test_req_fixture_roundtrip() {
    let req: PaymentRequest = read_fixture("payment_request.json");
    let encoded = req.canonical_encoding();
    let decoded = PaymentRequest::from_canonical_encoding(&encoded).expect("decode req");
    let reencoded = decoded.canonical_encoding();

    assert_eq!(decoded, req);
    assert_eq!(reencoded, encoded);
}

#[test]
fn test_req_bad_flag() {
    let req: PaymentRequest = read_fixture("payment_request.json");
    let mut encoded = req.canonical_encoding();
    let pos = req_meta_flag_pos(&encoded);
    encoded[pos] = 2;

    let result = PaymentRequest::from_canonical_encoding(&encoded);
    assert!(matches!(
        result,
        Err(PaymentRequestError::InvalidRequestFlag)
    ));
}

#[test]
fn test_req_bad_len() {
    let req: PaymentRequest = read_fixture("payment_request.json");
    let mut encoded = req.canonical_encoding();
    encoded.pop();

    let result = PaymentRequest::from_canonical_encoding(&encoded);
    assert!(matches!(
        result,
        Err(PaymentRequestError::InvalidRequestBytes)
            | Err(PaymentRequestError::InvalidRequestSize)
    ));
}

#[test]
fn test_req_bad_endian() {
    let req: PaymentRequest = read_fixture("payment_request.json");
    let mut encoded = req.canonical_encoding();
    let pos = 1 + 32 + 32 + 32 + 32;
    encoded[pos..pos + 4].copy_from_slice(&req.chain_id.to_be_bytes());

    let decoded = PaymentRequest::from_canonical_encoding(&encoded).expect("decode req");
    assert_eq!(decoded.chain_id, req.chain_id.swap_bytes());
}

#[test]
fn test_req_bad_opt_len() {
    let req: PaymentRequest = read_fixture("payment_request.json");
    let mut encoded = req.canonical_encoding();
    let len_pos = req_meta_memo_len_pos(&encoded);
    encoded[len_pos..len_pos + 4].copy_from_slice(&(u32::MAX).to_le_bytes());

    let result = PaymentRequest::from_canonical_encoding(&encoded);
    assert!(matches!(
        result,
        Err(PaymentRequestError::InvalidRequestBytes)
            | Err(PaymentRequestError::InvalidRequestString)
            | Err(PaymentRequestError::InvalidRequestSize)
    ));
}

#[test]
fn test_asset_leaf_fixture_roundtrip() {
    let leaf: AssetLeaf = read_fixture("asset_leaf.json");
    let encoded = serde_json::to_vec(&leaf).expect("encode leaf");
    let decoded: AssetLeaf = serde_json::from_slice(&encoded).expect("decode leaf");
    let reencoded = serde_json::to_vec(&decoded).expect("reencode leaf");

    assert_eq!(decoded, leaf);
    assert_eq!(reencoded, encoded);
}

#[test]
fn test_tx_package_fixture_roundtrip() {
    let pkg: TxPackage = read_fixture("tx_package.json");
    let encoded = serde_json::to_vec(&pkg).expect("encode tx package");
    let decoded: TxPackage = serde_json::from_slice(&encoded).expect("decode tx package");
    let reencoded = serde_json::to_vec(&decoded).expect("reencode tx package");

    assert_eq!(decoded, pkg);
    assert_eq!(reencoded, encoded);
}

#[test]
fn test_asset_bad_array_len() {
    let raw = r#"{
      "asset_id":[1,2,3],
      "serial_id":1,
      "r_pub":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      "owner_tag":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      "c_amount":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      "enc_pack":{"version":1,"ciphertext":[],"tag":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]},
      "range_proof":[],
      "tag16":0
    }"#;
    let result = serde_json::from_str::<AssetLeaf>(raw);
    assert!(result.is_err());
}
