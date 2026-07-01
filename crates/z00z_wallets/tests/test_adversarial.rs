#![allow(deprecated)]

use std::time::Instant;

use z00z_core::assets::AssetLeaf;
use z00z_crypto::Z00ZRistrettoPoint;
use z00z_wallets::{
    build_card_stealth_leaf, build_tx_output_unchecked,
    key::{ReceiverKeys, ReceiverSecret},
    receiver::receiver_scan_leaf,
    receiver::EphemeralCache,
    receiver::ReceiverCard,
    stealth::ecdh::{derive_r_hedged, receiver_derive_dh},
    stealth::kdf::{compute_leaf_ad, derive_k_dh},
    stealth::zkpack::ZkPack,
    SenderWallet, WalletError,
};

fn make_leaf(amount: u64) -> (ReceiverKeys, AssetLeaf) {
    let recv_secret = ReceiverSecret::from_bytes([0x22u8; 32]).expect("receiver secret");
    let keys = ReceiverKeys::from_receiver_secret(recv_secret).expect("receiver keys");
    let card = ReceiverCard {
        version: 1,
        owner_handle: keys.owner_handle,
        view_pk: keys.view_pk.to_bytes(),
        identity_pk: [9u8; 32],
        card_id: None,
        metadata: None,
        signature: [0u8; 64],
    };
    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut SenderWallet::new([0x33u8; 32]),
        &[0x44u8; 32],
        1,
        amount,
        &[0x55u8; 32],
    )
    .expect("stealth output create failed");
    let leaf = AssetLeaf {
        asset_id: [0x55u8; 32],
        serial_id: 0,
        r_pub: output.r_pub,
        owner_tag: output.owner_tag,
        c_amount: output.c_amount,
        enc_pack: output.enc_pack,
        range_proof: Vec::new(),
        tag16: output.tag16.expect("tag16"),
    };
    (keys, leaf)
}

fn flip_bit(slice: &[u8], bit_pos: usize) -> Vec<u8> {
    let mut out = slice.to_vec();
    let idx = bit_pos / 8;
    let bit = bit_pos % 8;
    if idx < out.len() {
        out[idx] ^= 1u8 << bit;
    }
    out
}

#[test]
fn test_adv_identity_view_pk() {
    let bad_card = ReceiverCard {
        version: 1,
        owner_handle: [1u8; 32],
        view_pk: Z00ZRistrettoPoint::identity().to_bytes(),
        identity_pk: [2u8; 32],
        card_id: None,
        metadata: None,
        signature: [0u8; 64],
    };
    let result = build_card_stealth_leaf(&bad_card, 1000, 1);
    assert!(
        matches!(result, Err(WalletError::IdentityPointNotAllowed)),
        "identity view_pk must be rejected"
    );
}

#[test]
fn test_adv_dup_rng_failure() {
    let seed = [0xDEu8; 32];
    let tx_digest = [0xADu8; 32];
    let rng_bytes = [0xFEu8; 32];
    let r0 = derive_r_hedged(&rng_bytes, &seed, &tx_digest, 0).expect("r0");
    let r1 = derive_r_hedged(&rng_bytes, &seed, &tx_digest, 1).expect("r1");
    assert_ne!(
        r0.to_bytes(),
        r1.to_bytes(),
        "different index must produce different r"
    );

    let r0b = derive_r_hedged(&rng_bytes, &seed, &tx_digest, 0).expect("r0b");
    assert_eq!(
        r0.to_bytes(),
        r0b.to_bytes(),
        "same args must be deterministic"
    );

    let mut cache = EphemeralCache::new(2);
    let owner = [0x22u8; 32];
    let r_pub = Z00ZRistrettoPoint::from_secret_key(&r0).to_bytes();
    assert!(
        cache.check_and_insert(&owner, &r_pub).is_ok(),
        "first insert must succeed"
    );
    assert!(
        matches!(
            cache.check_and_insert(&owner, &r_pub),
            Err(WalletError::DuplicateEphemeralR)
        ),
        "duplicate must be rejected"
    );
}

#[test]
fn test_adv_bit_flip_tamper() {
    let (keys, leaf) = make_leaf(1000);
    let plain = receiver_scan_leaf(&keys, &leaf)
        .expect("scan failed")
        .expect("owner leaf must decrypt");

    let mut tamper_owner = leaf.clone();
    let owner_flip = flip_bit(&tamper_owner.owner_tag, 0);
    tamper_owner.owner_tag.copy_from_slice(&owner_flip);
    let owner_scan = receiver_scan_leaf(&keys, &tamper_owner).expect("owner tamper scan failed");
    assert_ne!(owner_scan, Some(plain.clone()));

    let mut tamper_commit = leaf.clone();
    let c_flip = flip_bit(&tamper_commit.c_amount, 7);
    tamper_commit.c_amount.copy_from_slice(&c_flip);
    let c_scan = receiver_scan_leaf(&keys, &tamper_commit).expect("commit tamper scan failed");
    assert_ne!(c_scan, Some(plain.clone()));

    let mut tamper_tag = leaf.clone();
    let tag_flip = flip_bit(&tamper_tag.enc_pack.tag, 3);
    tamper_tag.enc_pack.tag.copy_from_slice(&tag_flip);
    let tag_scan = receiver_scan_leaf(&keys, &tamper_tag).expect("tag tamper scan failed");
    assert_eq!(tag_scan, None);

    let mut tamper_ct = leaf.clone();
    let ct_flip = flip_bit(&tamper_ct.enc_pack.ciphertext, 0);
    tamper_ct.enc_pack.ciphertext = ct_flip;
    let ct_scan = receiver_scan_leaf(&keys, &tamper_ct).expect("ct tamper scan failed");
    assert_eq!(ct_scan, None);
}

#[test]
fn test_adv_noncanon_r_pub() {
    let recv_secret = ReceiverSecret::from_bytes([0x22u8; 32]).expect("receiver secret");
    let keys = ReceiverKeys::from_receiver_secret(recv_secret).expect("receiver keys");
    let bad_leaf = AssetLeaf {
        r_pub: [0xFFu8; 32],
        ..AssetLeaf::default()
    };

    let result = receiver_scan_leaf(&keys, &bad_leaf);
    assert!(
        matches!(result, Ok(None)),
        "non-canonical R_pub must gracefully skip"
    );
}

#[test]
fn test_adv_dos_identity_flood() {
    let recv_secret = ReceiverSecret::from_bytes([0x22u8; 32]).expect("receiver secret");
    let keys = ReceiverKeys::from_receiver_secret(recv_secret).expect("receiver keys");
    let identity_leaf = AssetLeaf {
        r_pub: [0x00u8; 32],
        ..AssetLeaf::default()
    };

    for _ in 0..10_000 {
        let result = receiver_scan_leaf(&keys, &identity_leaf);
        assert!(
            matches!(result, Ok(None)),
            "identity R_pub must never crash or Err"
        );
    }

    let start = Instant::now();
    for _ in 0..10_000 {
        let _ = receiver_scan_leaf(&keys, &identity_leaf);
    }
    assert!(
        start.elapsed().as_secs() < 1,
        "identity flood must not cause DoS"
    );
}

#[test]
fn test_adv_serial_relabel_rejected() {
    let (keys, mut leaf) = make_leaf(1000);
    leaf.serial_id = 1_000_000;

    let result = receiver_scan_leaf(&keys, &leaf).expect("scan must not fail");
    assert_eq!(
        result, None,
        "serial relabel must not pass receiver ownership checks"
    );
}

#[test]
fn test_skip_unknown_boundary() {
    let (keys, mut leaf) = make_leaf(1000);
    leaf.serial_id = 2_000_000;

    let result = receiver_scan_leaf(&keys, &leaf).expect("scan must not fail on unknown");
    assert_eq!(result, None, "unknown leaf must be skipped");
}

#[test]
fn test_adv_reject_bad_len() {
    let (keys, mut leaf) = make_leaf(1000);

    let r_pub = Z00ZRistrettoPoint::try_from_bytes(leaf.r_pub).expect("invalid R_pub");
    let dh = receiver_derive_dh(keys.reveal_view_sk(), &r_pub).expect("receiver derive failed");
    let k_dh = derive_k_dh(&dh.to_bytes());
    let leaf_ad = compute_leaf_ad(
        &leaf.asset_id,
        leaf.serial_id,
        &leaf.r_pub,
        &leaf.owner_tag,
        &leaf.c_amount,
    );

    let malformed_plain = [0xAAu8; 71];
    leaf.enc_pack = ZkPack::encrypt(
        &k_dh,
        &leaf_ad,
        &leaf.r_pub,
        &leaf.asset_id,
        leaf.serial_id,
        &malformed_plain,
    );

    let result = receiver_scan_leaf(&keys, &leaf);
    assert!(
        matches!(result, Err(WalletError::InvalidAssetPack("wrong length"))),
        "wrong-length plaintext must return InvalidAssetPack"
    );
}

#[test]
fn test_rejects_decryptable_not_spendable() {
    let recv_secret = ReceiverSecret::from_bytes([0x22u8; 32]).expect("receiver secret");
    let keys = ReceiverKeys::from_receiver_secret(recv_secret).expect("receiver keys");
    let mut wrong_owner = keys.owner_handle;
    wrong_owner[0] ^= 0x01;

    let bad_card = ReceiverCard {
        version: 1,
        owner_handle: wrong_owner,
        view_pk: keys.view_pk.to_bytes(),
        identity_pk: [9u8; 32],
        card_id: None,
        metadata: None,
        signature: [0u8; 64],
    };

    let output = build_tx_output_unchecked(
        &bad_card,
        None,
        &mut SenderWallet::new([0x33u8; 32]),
        &[0x44u8; 32],
        1,
        1000,
        &[0x55u8; 32],
    )
    .expect("stealth output create failed");
    let leaf = AssetLeaf {
        asset_id: [0x55u8; 32],
        serial_id: 0,
        r_pub: output.r_pub,
        owner_tag: output.owner_tag,
        c_amount: output.c_amount,
        enc_pack: output.enc_pack,
        range_proof: Vec::new(),
        tag16: output.tag16.expect("tag16"),
    };
    let result = receiver_scan_leaf(&keys, &leaf).expect("scan must not fail");
    assert_eq!(
        result, None,
        "leaf with a mismatched owner_handle must not be accepted as owned"
    );
}
