use bech32::FromBase32;
use z00z_utils::time::{SystemTimeProvider, TimeProvider};

use super::*;
use crate::key::{
    derive_identity_public_key, derive_identity_secret_key, ReceiverKeys, ReceiverSecret,
};

fn test_card() -> ReceiverCard {
    ReceiverCard {
        version: CARD_VER_1,
        owner_handle: [7u8; 32],
        view_pk: [9u8; 32],
        identity_pk: [11u8; 32],
        card_id: Some([3u8; 16]),
        metadata: Some(CardMetadata {
            created_at: 123,
            display_name: Some("Alice".to_string()),
            valid_until: Some(
                SystemTimeProvider
                    .compat_unix_timestamp()
                    .saturating_add(3600),
            ),
            contact: Some("alice@z00z".to_string()),
        }),
        signature: [0u8; 64],
    }
}

fn signed_card() -> ReceiverCard {
    let secret = ReceiverSecret::generate().expect("secret");
    let identity_sk = derive_identity_secret_key(&secret, 0).expect("identity sk");
    let identity_pk = derive_identity_public_key(&identity_sk).expect("identity pk");
    let mut card = test_card();
    card.identity_pk = identity_pk.as_bytes().try_into().expect("identity pk size");
    card.view_pk = identity_pk.as_bytes().try_into().expect("view pk size");
    card.sign(&identity_sk).expect("sign");
    card
}

#[test]
fn test_canonical_encoding_deterministic() {
    let card = signed_card();
    assert_eq!(card.canonical_encoding(), card.canonical_encoding());
}

#[test]
fn test_canonical_encoding_field_order() {
    let card = test_card();
    let encoded = card.canonical_encoding_unsigned();
    assert_eq!(encoded[0], CARD_VER_1);
    assert_eq!(&encoded[1..33], &card.owner_handle);
    assert_eq!(&encoded[33..65], &card.view_pk);
    assert_eq!(&encoded[65..97], &card.identity_pk);
}

#[test]
fn test_canonical_encoding_roundtrip() {
    let card = signed_card();
    let encoded = card.canonical_encoding();
    let decoded = ReceiverCard::from_canonical_encoding(&encoded).expect("decode");
    assert_eq!(decoded, card);
}

#[test]
fn test_receiver_card_sign_verify() {
    let card = signed_card();
    card.verify().expect("verify");
}

#[test]
fn test_receiver_card_invalid_signature() {
    let mut card = signed_card();
    card.signature[13] ^= 0xFF;
    assert!(card.verify().is_err());
}

#[test]
fn test_receiver_card_tampered_data() {
    let mut card = signed_card();
    card.owner_handle[0] ^= 0x01;
    assert!(card.verify().is_err());
}

#[test]
fn test_sign_reject_key_mismatch() {
    let secret_a = ReceiverSecret::generate().expect("secret a");
    let secret_b = ReceiverSecret::generate().expect("secret b");
    let sk_a = derive_identity_secret_key(&secret_a, 0).expect("sk a");
    let sk_b = derive_identity_secret_key(&secret_b, 0).expect("sk b");
    let pk_b = derive_identity_public_key(&sk_b).expect("pk b");

    let mut card = test_card();
    card.identity_pk = pk_b.as_bytes().try_into().expect("pk bytes");
    card.view_pk = pk_b.as_bytes().try_into().expect("pk bytes");

    assert!(matches!(
        card.sign(&sk_a),
        Err(ReceiverCardError::KeyMismatch)
    ));
}

#[test]
fn test_card_encoding_roundtrip() {
    let card = signed_card();
    let compact = encode_card_compact(&card);
    let decoded = decode_card_compact(&compact).expect("decode");
    assert_eq!(decoded, card);
}

#[test]
fn test_card_base64_url_safe() {
    let card = signed_card();
    let compact = encode_card_compact(&card);
    assert!(!compact.contains('+'));
    assert!(!compact.contains('/'));
    assert!(!compact.contains('='));
}

#[test]
fn test_format_receiver_handle_roundtrip() {
    let owner = [0x5Au8; 32];
    let encoded = format_receiver_handle(&owner).expect("encode");
    let (hrp, data, variant) = bech32::decode(&encoded).expect("decode");
    let decoded = Vec::<u8>::from_base32(&data).expect("from base32");

    assert_eq!(hrp, "z00z");
    assert_eq!(variant, Variant::Bech32m);
    assert_eq!(decoded, owner);
}

#[test]
fn test_prove_ownership_valid() {
    let secret = ReceiverSecret::generate().expect("secret");
    let identity_sk = derive_identity_secret_key(&secret, 0).expect("identity sk");
    let identity_pk = derive_identity_public_key(&identity_sk).expect("identity pk");

    let card = ReceiverCard {
        version: CARD_VER_1,
        owner_handle: [9u8; 32],
        view_pk: identity_pk.as_bytes().try_into().expect("pk bytes"),
        identity_pk: identity_pk.as_bytes().try_into().expect("pk bytes"),
        card_id: None,
        metadata: None,
        signature: [0u8; 64],
    };

    let challenge = [3u8; 32];
    let proof = prove_ownership(&card, &identity_sk, &challenge).expect("proof");
    let proof_sig = sig_from_bytes(&proof).expect("proof sig");
    verify_identity(&identity_pk, &challenge, &card.owner_handle, &proof_sig).expect("verify");
}

#[test]
fn test_prove_ownership_wrong_key() {
    let secret_a = ReceiverSecret::generate().expect("secret a");
    let secret_b = ReceiverSecret::generate().expect("secret b");
    let sk_a = derive_identity_secret_key(&secret_a, 0).expect("sk a");
    let sk_b = derive_identity_secret_key(&secret_b, 0).expect("sk b");
    let pk_a = derive_identity_public_key(&sk_a).expect("pk a");

    let card = ReceiverCard {
        version: CARD_VER_1,
        owner_handle: [7u8; 32],
        view_pk: pk_a.as_bytes().try_into().expect("pk bytes"),
        identity_pk: pk_a.as_bytes().try_into().expect("pk bytes"),
        card_id: None,
        metadata: None,
        signature: [0u8; 64],
    };

    let challenge = [8u8; 32];
    let proof = prove_ownership(&card, &sk_b, &challenge);
    assert!(matches!(proof, Err(ReceiverCardError::KeyMismatch)));
}

#[test]
fn test_sign_card() {
    let secret = ReceiverSecret::generate().expect("secret");
    let identity_sk = derive_identity_secret_key(&secret, 0).expect("identity sk");
    let identity_pk = derive_identity_public_key(&identity_sk).expect("identity pk");

    let mut card = test_card();
    card.identity_pk = identity_pk.as_bytes().try_into().expect("pk bytes");
    card.view_pk = identity_pk.as_bytes().try_into().expect("pk bytes");

    card.sign(&identity_sk).expect("signed");
    card.verify().expect("verify");
}

#[test]
fn test_receiver_keys_export_path() {
    let secret = ReceiverSecret::generate().expect("secret");
    let mut keys = ReceiverKeys::from_receiver_secret(secret).expect("keys");

    let mut card = keys.export_receiver_card().expect("create");
    card.verify().expect("verify");

    card.signature = [0u8; 64];
    card.sign(keys.reveal_identity_sk()).expect("sign");
    card.verify().expect("verify");

    let before = keys.export_receiver_card().expect("before");
    let rotated = keys.rotate_view().expect("rotate");
    assert_eq!(before.owner_handle, rotated.owner_handle);
    assert_ne!(before.view_pk, rotated.view_pk);
    rotated.verify().expect("verify rotated");
}

#[test]
fn test_decode_compact_too_long() {
    let encoded = URL_SAFE_NO_PAD.encode(vec![0u8; MAX_CARD_SIZE + 1]);
    assert!(matches!(
        decode_card_compact(&encoded),
        Err(ReceiverCardError::InvalidCardSize)
    ));
}

#[test]
fn test_validate_reject_identity_point() {
    let mut card = signed_card();
    card.view_pk = [0u8; 32];
    assert!(card.validate_ecc_points().is_err());
}

#[test]
fn test_validate_wrong_version() {
    let mut card = signed_card();
    card.version = 2;
    assert!(card.validate_structure().is_err());
}

#[test]
fn test_validate_reject_identity_pk() {
    let mut card = signed_card();
    card.identity_pk = [0u8; 32];
    assert!(card.validate_ecc_points().is_err());
}

#[test]
fn test_verify_reject_owner_handle() {
    let mut card = signed_card();
    card.owner_handle[0] ^= 0x01;

    assert!(matches!(
        card.verify(),
        Err(ReceiverCardError::VerifyFailed)
    ));
}

#[test]
fn test_verify_rejects_expired_card() {
    let secret = ReceiverSecret::generate().expect("secret");
    let identity_sk = derive_identity_secret_key(&secret, 0).expect("identity sk");
    let identity_pk = derive_identity_public_key(&identity_sk).expect("identity pk");

    let mut card = ReceiverCard {
        version: CARD_VER_1,
        owner_handle: [4u8; 32],
        view_pk: identity_pk.as_bytes().try_into().expect("pk bytes"),
        identity_pk: identity_pk.as_bytes().try_into().expect("pk bytes"),
        card_id: None,
        metadata: Some(CardMetadata {
            created_at: 1,
            display_name: None,
            valid_until: Some(1),
            contact: None,
        }),
        signature: [0u8; 64],
    };

    card.sign(&identity_sk).expect("sign");

    assert!(matches!(card.verify(), Err(ReceiverCardError::CardExpired)));
}

#[test]
fn test_untrusted_parse_too_short() {
    let bytes = vec![0u8; MIN_CARD_SIZE - 1];
    assert!(ReceiverCard::from_untrusted_bytes(&bytes).is_err());
}

#[test]
fn test_untrusted_parse_too_long() {
    let bytes = vec![0u8; MAX_CARD_SIZE + 1];
    assert!(ReceiverCard::from_untrusted_bytes(&bytes).is_err());
}

#[test]
fn test_untrusted_parse_invalid_point() {
    let mut card = signed_card();
    card.view_pk = [0u8; 32];
    let bytes = card.canonical_encoding();
    assert!(ReceiverCard::from_untrusted_bytes(&bytes).is_err());
}
