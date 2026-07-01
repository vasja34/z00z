#![allow(deprecated)]

use z00z_crypto::{hash::poseidon2_hash, hash_to_scalar_domain, Z00ZRistrettoPoint};
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    stealth::ecdh::sender_derive_dh_with_r,
    stealth::kdf::{
        compute_leaf_ad, compute_owner_tag, compute_tag16, derive_k_dh, derive_pack_key,
        derive_pack_nonce,
    },
};

fn make_keys(secret: [u8; 32]) -> ReceiverKeys {
    let recv = ReceiverSecret::from_bytes(secret).expect("receiver secret");
    ReceiverKeys::from_receiver_secret(recv).expect("receiver keys")
}

#[test]
fn test_golden_owner_handle() {
    let secret_22 = [0x22u8; 32];
    let secret_00 = [0x00u8; 32];
    let secret_ff = [0xFFu8; 32];

    let handle_22 = poseidon2_hash(b"z00z.consensus.receiver_id.v1", &[&secret_22]);
    assert_eq!(
        hex::encode(handle_22),
        "a2b19c2023207dfb834dfec204ccf9b4bb3acc17ddcbfd542e5af2cfcb409371",
        "handle[0x22;32]"
    );

    let handle_00 = poseidon2_hash(b"z00z.consensus.receiver_id.v1", &[&secret_00]);
    let handle_ff = poseidon2_hash(b"z00z.consensus.receiver_id.v1", &[&secret_ff]);
    assert_eq!(
        hex::encode(handle_00),
        "6d0a2f546c5cffb437d9d84dab039aa79db189d0fac6b565e8e72c37927dbdfb",
        "handle[0x00;32]"
    );
    assert_eq!(
        hex::encode(handle_ff),
        "9c70b1311e522b2f46a204537f29326780a70d9ca9926d7009f4436ba282154a",
        "handle[0xff;32]"
    );
}

#[test]
fn test_golden_recv_keys() {
    let keys = make_keys([0x11u8; 32]);

    assert_eq!(
        hex::encode(keys.owner_handle),
        "a17fb65d63b430d16be307cd600652cd4fac41450b257d504b6ccf96244d551d"
    );
    assert_eq!(
        hex::encode(keys.reveal_view_sk().to_bytes()),
        "14ef70529859ec3146a2d101d2f1b511b586717a5f835348e72260e8e31d9b06"
    );
    assert_eq!(
        hex::encode(keys.view_pk.to_bytes()),
        "68011c2b81d2adab05b4c778e3460435438537982d92d0871542bee02d497d4f"
    );
    assert_eq!(
        hex::encode(keys.reveal_identity_sk().to_bytes()),
        "c863d6f3433188986a537bc46dfba8707b9ea8f1fe52880b4fc61b87ad83a604"
    );
    assert_eq!(
        hex::encode(keys.identity_pk.to_bytes()),
        "6e290fa18514363bac2600a7d1c6cae7dd4e9b727780188191ca900a81d90522"
    );
}

#[test]
fn test_golden_tag16() {
    const GOLDEN: u16 = 0x5093;

    assert_eq!(
        compute_tag16(&[0x44u8; 32], &[0x55u8; 32]),
        GOLDEN,
        "tag16 golden vector"
    );
    assert_ne!(
        compute_tag16(&[0x44u8; 32], &[0x56u8; 32]),
        GOLDEN,
        "changed input must change tag16"
    );
    assert_ne!(
        compute_tag16(&[0x45u8; 32], &[0x55u8; 32]),
        GOLDEN,
        "changed input must change tag16"
    );
}

#[test]
fn test_golden_ecdh_cycle() {
    let recv_secret = [0x22u8; 32];
    let view_sk = hash_to_scalar_domain(b"z00z.consensus.view_key.v1", &[&recv_secret]);
    let view_pk = Z00ZRistrettoPoint::from_secret_key(&view_sk);

    let r = hash_to_scalar_domain(b"z00z.consensus.ephemeral_scalar.v1", &[&[0x17u8; 32]]);
    let sender = sender_derive_dh_with_r(&view_pk, &r).expect("sender derive failed");
    assert_eq!(
        sender.r.to_bytes(),
        r.to_bytes(),
        "deterministic scalar mismatch"
    );

    let k_dh = derive_k_dh(&sender.dh.to_bytes());
    let handle = poseidon2_hash(b"z00z.consensus.receiver_id.v1", &[&recv_secret]);
    let owner_tag = compute_owner_tag(&handle, &k_dh);

    let asset_id = [0x02u8; 32];
    let serial_id = 1u32;
    let c_amount = [0u8; 32];
    let leaf_ad = compute_leaf_ad(
        &asset_id,
        serial_id,
        &sender.r_pub.to_bytes(),
        &owner_tag,
        &c_amount,
    );
    let pack_key = derive_pack_key(&k_dh, &asset_id, serial_id);
    let nonce = derive_pack_nonce(&leaf_ad, &sender.r_pub.to_bytes(), &asset_id, serial_id);

    assert_eq!(
        hex::encode(view_sk.to_bytes()),
        "9bc85d850243efb27afe8f8ed78380eeaffe0ea519b283f0b84b6ba361005b03",
        "view_sk"
    );
    assert_eq!(
        hex::encode(sender.r_pub.to_bytes()),
        "2a090060ba284e9e0885e30621640332249769cc8b4e4b023e61d2cda6878d30",
        "r_pub"
    );
    assert_eq!(
        hex::encode(k_dh),
        "08ab9cb17aba0d640b86e7d1de8979f0a18c284a56e236532f12d9c343d5313e",
        "k_dh"
    );
    assert_eq!(
        hex::encode(owner_tag),
        "aa1431af1c561aa81fe53838271d141ea7e6a202f072f93514d7bf7a990c8e5f",
        "owner_tag"
    );
    assert_eq!(
        hex::encode(pack_key),
        "122501b6542d49e30d1ae14fd2f41cb24f77e011bdcd4732db4b9657143110dc",
        "pack_key"
    );
    assert_eq!(
        hex::encode(nonce),
        "28791789eb4433bde5658198f3d426c47c85fadd9740085d2f47a0ff861b1321",
        "nonce"
    );
}
