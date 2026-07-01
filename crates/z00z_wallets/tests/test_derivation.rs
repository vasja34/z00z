use std::{fs, path::PathBuf};

use serde::Deserialize;
use z00z_crypto::{
    hash_to_scalar_domain, kdf as crypto_kdf, protocol::ecdh as crypto_ecdh, Z00ZRistrettoPoint,
};
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    stealth::ecdh::{receiver_derive_dh, sender_derive_dh_with_r},
    stealth::kdf::{compute_leaf_ad, compute_owner_tag, compute_tag16, derive_k_dh, derive_s_out},
};

fn make_keys(secret: [u8; 32]) -> ReceiverKeys {
    let recv = ReceiverSecret::from_bytes(secret).expect("receiver secret");
    ReceiverKeys::from_receiver_secret(recv).expect("receiver keys")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChainVec {
    r: [u8; 32],
    r_pub: [u8; 32],
    dh: [u8; 32],
    k_dh: [u8; 32],
    s_out: [u8; 32],
    asset_id: [u8; 32],
    owner_tag: [u8; 32],
    leaf_ad: [u8; 32],
    tag16: u16,
}

#[derive(Debug, Deserialize)]
struct ChainFix {
    recv_sec: String,
    r_seed: String,
    serial_id: u32,
    c_amount: String,
    expected: ChainExp,
}

#[derive(Debug, Deserialize)]
struct ChainExp {
    r: String,
    r_pub: String,
    dh: String,
    k_dh: String,
    s_out: String,
    asset_id: String,
    owner_tag: String,
    leaf_ad: String,
    tag16: u16,
}

#[derive(Debug, Deserialize)]
struct TranscriptFix {
    recv_sec: String,
    r_seed: String,
    expected: TranscriptExp,
}

#[derive(Debug, Deserialize)]
struct TranscriptExp {
    r: String,
    r_pub: String,
    dh: String,
    k_dh: String,
}

#[derive(Debug, Deserialize)]
struct DriftFix {
    recv_sec: String,
    r_seed: String,
    serial_id: u32,
    c_amount: String,
    expected_wallet_leaf: String,
    expected_crypto_leaf: String,
}

fn fix_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn read_fix<T: for<'de> Deserialize<'de>>(name: &str) -> T {
    let raw = fs::read_to_string(fix_root().join(name)).expect("read fixture");
    serde_json::from_str(&raw).expect("decode fixture")
}

fn hex32(bytes: [u8; 32]) -> String {
    hex::encode(bytes)
}

fn parse_hex<const N: usize>(hex_in: &str) -> [u8; N] {
    let raw = hex::decode(hex_in).expect("hex decode");
    let arr: [u8; N] = raw.try_into().expect("hex size");
    arr
}

fn build_vec(
    recv_sec: &[u8; 32],
    r_seed: &[u8; 32],
    serial_id: u32,
    c_amount: &[u8; 32],
) -> ChainVec {
    let keys = make_keys(*recv_sec);
    let r = hash_to_scalar_domain(b"z00z.consensus.ephemeral_scalar.v1", &[r_seed]);
    let sender = sender_derive_dh_with_r(&keys.view_pk, &r).expect("sender");
    let recv_dh = receiver_derive_dh(keys.reveal_view_sk(), &sender.r_pub).expect("receiver");

    let r_pub = sender.r_pub.to_bytes();
    let k_dh = derive_k_dh(&sender.dh.to_bytes());
    let s_out = derive_s_out(&k_dh, &r_pub, serial_id);
    let asset_id = crypto_kdf::derive_asset_id(&s_out);
    let owner_tag = compute_owner_tag(&keys.owner_handle, &k_dh);
    let leaf_ad = compute_leaf_ad(&asset_id, serial_id, &r_pub, &owner_tag, c_amount);
    let tag16 = compute_tag16(&k_dh, &leaf_ad);

    assert_eq!(sender.dh.to_bytes(), recv_dh.to_bytes(), "dh parity");

    ChainVec {
        r: r.to_bytes(),
        r_pub,
        dh: sender.dh.to_bytes(),
        k_dh,
        s_out,
        asset_id,
        owner_tag,
        leaf_ad,
        tag16,
    }
}

#[test]
fn test_chain_vector() {
    let fix: ChainFix = read_fix("phase11_chain.json");
    let recv_sec = parse_hex::<32>(&fix.recv_sec);
    let r_seed = parse_hex::<32>(&fix.r_seed);
    let c_amount = parse_hex::<32>(&fix.c_amount);

    let got = build_vec(&recv_sec, &r_seed, fix.serial_id, &c_amount);

    assert_eq!(hex32(got.r), fix.expected.r);
    assert_eq!(hex32(got.r_pub), fix.expected.r_pub);
    assert_eq!(hex32(got.dh), fix.expected.dh);
    assert_eq!(hex32(got.k_dh), fix.expected.k_dh);
    assert_eq!(hex32(got.s_out), fix.expected.s_out);
    assert_eq!(hex32(got.asset_id), fix.expected.asset_id);
    assert_eq!(hex32(got.owner_tag), fix.expected.owner_tag);
    assert_eq!(hex32(got.leaf_ad), fix.expected.leaf_ad);
    assert_eq!(got.tag16, fix.expected.tag16);
}

#[test]
fn test_sender_fix() {
    let fix: TranscriptFix = read_fix("phase11_sender.json");
    let recv_sec = parse_hex::<32>(&fix.recv_sec);
    let r_seed = parse_hex::<32>(&fix.r_seed);

    let keys = make_keys(recv_sec);
    let r = hash_to_scalar_domain(b"z00z.consensus.ephemeral_scalar.v1", &[&r_seed]);
    let sender = sender_derive_dh_with_r(&keys.view_pk, &r).expect("sender");
    let k_dh = derive_k_dh(&sender.dh.to_bytes());

    assert_eq!(hex32(r.to_bytes()), fix.expected.r);
    assert_eq!(hex32(sender.r_pub.to_bytes()), fix.expected.r_pub);
    assert_eq!(hex32(sender.dh.to_bytes()), fix.expected.dh);
    assert_eq!(hex32(k_dh), fix.expected.k_dh);
}

#[test]
fn test_receiver_fix() {
    let fix: TranscriptFix = read_fix("phase11_sender.json");
    let recv_sec = parse_hex::<32>(&fix.recv_sec);
    let r_pub_bytes = parse_hex::<32>(&fix.expected.r_pub);

    let keys = make_keys(recv_sec);
    let r_pub = Z00ZRistrettoPoint::try_from_bytes(r_pub_bytes).expect("r_pub");
    let dh = receiver_derive_dh(keys.reveal_view_sk(), &r_pub).expect("receiver");
    let k_dh = derive_k_dh(&dh.to_bytes());

    assert_eq!(hex32(dh.to_bytes()), fix.expected.dh);
    assert_eq!(hex32(k_dh), fix.expected.k_dh);
}

#[test]
fn test_api_parity() {
    let fix: ChainFix = read_fix("phase11_chain.json");
    let recv_sec = parse_hex::<32>(&fix.recv_sec);
    let r_seed = parse_hex::<32>(&fix.r_seed);
    let c_amount = parse_hex::<32>(&fix.c_amount);
    let got = build_vec(&recv_sec, &r_seed, fix.serial_id, &c_amount);
    let keys = make_keys(recv_sec);

    let dh = Z00ZRistrettoPoint::try_from_bytes(got.dh).expect("dh");
    let wallet_k = derive_k_dh(&dh.to_bytes());
    let crypto_k = crypto_ecdh::derive_dh_key(&dh);
    let crypto_asset = crypto_kdf::derive_asset_id(&got.s_out);
    let crypto_owner = crypto_kdf::compute_owner_tag(&keys.owner_handle, &wallet_k);

    assert_eq!(wallet_k, crypto_k);
    assert_eq!(got.asset_id, crypto_asset);
    assert_eq!(got.owner_tag, crypto_owner);
    assert_eq!(hex32(wallet_k), fix.expected.k_dh);
}

#[test]
fn test_domain_mismatch() {
    let fix: ChainFix = read_fix("phase11_chain.json");
    let recv_sec = parse_hex::<32>(&fix.recv_sec);
    let r_seed = parse_hex::<32>(&fix.r_seed);
    let c_amount = parse_hex::<32>(&fix.c_amount);
    let got = build_vec(&recv_sec, &r_seed, fix.serial_id, &c_amount);

    let crypto_leaf = crypto_kdf::derive_leaf_ad(
        &got.asset_id,
        fix.serial_id,
        &got.r_pub,
        &got.owner_tag,
        &c_amount,
    );
    let keys = make_keys(recv_sec);
    let crypto_owner = crypto_kdf::compute_owner_tag(&keys.owner_handle, &got.k_dh);

    assert_ne!(got.leaf_ad, crypto_leaf);
    assert_eq!(got.owner_tag, crypto_owner);
}

#[test]
fn test_migration_vec() {
    let fix: DriftFix = read_fix("phase11_migration.json");
    let recv_sec = parse_hex::<32>(&fix.recv_sec);
    let r_seed = parse_hex::<32>(&fix.r_seed);
    let c_amount = parse_hex::<32>(&fix.c_amount);
    let got = build_vec(&recv_sec, &r_seed, fix.serial_id, &c_amount);
    let crypto_leaf = crypto_kdf::derive_leaf_ad(
        &got.asset_id,
        fix.serial_id,
        &got.r_pub,
        &got.owner_tag,
        &c_amount,
    );

    assert_eq!(hex32(got.leaf_ad), fix.expected_wallet_leaf);
    assert_eq!(hex32(crypto_leaf), fix.expected_crypto_leaf);
    assert_ne!(got.leaf_ad, crypto_leaf);
}
