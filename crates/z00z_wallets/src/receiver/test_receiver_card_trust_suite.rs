use std::sync::{Arc, Mutex};

use crate::key::{
    derive_identity_public_key, derive_identity_secret_key, derive_view_public_key,
    derive_view_secret_key, ReceiverSecret,
};

use super::*;

fn view_key(seed: u8) -> [u8; 32] {
    let recv = ReceiverSecret::from_encrypted(
        &ReceiverSecret::generate()
            .expect("secret")
            .to_encrypted(&[seed])
            .expect("enc"),
        &[seed],
    )
    .expect("dec");
    let view_sk = derive_view_secret_key(&recv).expect("view sk");
    let view_pk = derive_view_public_key(&view_sk).expect("view pk");
    view_pk.as_bytes().try_into().expect("view pk size")
}

fn make_card_with_metadata(
    seed: u8,
    owner: [u8; 32],
    view: [u8; 32],
    metadata: Option<crate::receiver::receiver_card::CardMetadata>,
) -> ReceiverCard {
    let recv = ReceiverSecret::from_encrypted(
        &ReceiverSecret::generate()
            .expect("secret")
            .to_encrypted(b"pw")
            .expect("enc"),
        b"pw",
    )
    .expect("dec");
    let sk = derive_identity_secret_key(&recv, u32::from(seed)).expect("sk");
    let identity_pk = derive_identity_public_key(&sk).expect("pk");

    let mut card = ReceiverCard {
        version: 1,
        owner_handle: owner,
        view_pk: view,
        identity_pk: identity_pk.as_bytes().try_into().expect("identity pk size"),
        card_id: None,
        metadata,
        signature: [0u8; 64],
    };
    card.sign(&sk).expect("sign");
    card
}

fn make_card(seed: u8, owner: [u8; 32], view: [u8; 32]) -> ReceiverCard {
    make_card_with_metadata(seed, owner, view, None)
}

#[test]
fn test_tofu_first_use() {
    let mut pins = PinnedReceiverCards::new();
    let card = make_card(1, [1u8; 32], view_key(1));

    let result = pins.verify_or_pin(&card, Some("dir")).expect("verify");
    assert_eq!(result, VerifyResult::NewPin);
    assert_eq!(pins.len(), 1);
}

#[test]
fn test_tofu_verified_second_use() {
    let mut pins = PinnedReceiverCards::new();
    let card = make_card(2, [3u8; 32], view_key(2));

    pins.verify_or_pin(&card, None).expect("first");
    let result = pins.verify_or_pin(&card, None).expect("second");
    assert_eq!(result, VerifyResult::Verified);
}

#[test]
fn test_tofu_view_key_rotation() {
    let mut pins = PinnedReceiverCards::new();
    let first_view = view_key(3);
    let second_view = view_key(4);
    let first = make_card(3, [5u8; 32], first_view);
    let second = make_card(3, [5u8; 32], second_view);

    pins.verify_or_pin(&first, None).expect("first");
    let result = pins.verify_or_pin(&second, None).expect("second");

    assert_eq!(
        result,
        VerifyResult::ViewKeyChanged {
            old_pk: first_view,
            new_pk: second_view,
            requires_confirmation: true,
        }
    );
}

#[test]
fn test_confirm_rotation_marks_pinned() {
    let mut pins = PinnedReceiverCards::new();
    let owner = [6u8; 32];
    let first = view_key(6);
    let second = view_key(7);
    pins.pins.insert(
        owner,
        PinEntry {
            view_pk: first,
            identity_pk: [7u8; 32],
            directory_id: None,
            first_seen: 1,
            trust_level: TrustLevel::Tentative,
        },
    );

    pins.confirm_rotation(&owner, &second);
    let pin = pins.get(&owner).unwrap();
    assert_eq!(pin.view_pk, second);
    assert_eq!(pin.trust_level, TrustLevel::Pinned);
}

#[test]
fn test_revoke_marks_revoked() {
    let mut pins = PinnedReceiverCards::new();
    let owner = [8u8; 32];
    pins.pins.insert(
        owner,
        PinEntry {
            view_pk: [1u8; 32],
            identity_pk: [2u8; 32],
            directory_id: None,
            first_seen: 1,
            trust_level: TrustLevel::Tentative,
        },
    );

    pins.revoke(&owner);
    assert_eq!(pins.get(&owner).unwrap().trust_level, TrustLevel::Revoked);
}

#[test]
fn test_verify_request_identity_paths() {
    let mut pins = PinnedReceiverCards::new();
    let owner = [9u8; 32];
    let identity = [10u8; 32];
    assert_eq!(
        pins.verify_request_identity(&owner, &identity),
        PinCheckResult::NewIdentity
    );
    assert_eq!(
        pins.verify_request_identity(&owner, &identity),
        PinCheckResult::Verified
    );
    assert_eq!(
        pins.verify_request_identity(&owner, &[11u8; 32]),
        PinCheckResult::IdentityChanged
    );
}

#[test]
fn test_to_from_pairs_roundtrip() {
    let mut pins = PinnedReceiverCards::new();
    let card = make_card(12, [12u8; 32], view_key(12));
    pins.verify_or_pin(&card, Some("dir")).unwrap();

    let pairs = pins.to_pairs();
    let restored = PinnedReceiverCards::from_pairs(pairs);
    assert_eq!(restored.len(), 1);
    assert_eq!(restored.get(&card.owner_handle), pins.get(&card.owner_handle));
}

#[test]
fn test_verify_rejects_expired_card() {
    let mut pins = PinnedReceiverCards::new();
    let card = make_card_with_metadata(
        13,
        [13u8; 32],
        view_key(13),
        Some(crate::receiver::receiver_card::CardMetadata {
            created_at: 0,
            display_name: None,
            valid_until: Some(0),
            contact: None,
        }),
    );

    let err = pins.verify_or_pin(&card, None).unwrap_err();
    assert!(matches!(err, ReceiverCardError::CardExpired));
}

#[test]
fn test_tofu_is_send_safe() {
    let pins = Arc::new(Mutex::new(PinnedReceiverCards::new()));
    let card = make_card(14, [14u8; 32], view_key(14));

    let handle = std::thread::spawn({
        let pins = Arc::clone(&pins);
        move || pins.lock().unwrap().verify_or_pin(&card, None).unwrap()
    });

    assert_eq!(handle.join().unwrap(), VerifyResult::NewPin);
}