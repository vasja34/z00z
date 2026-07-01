use std::path::PathBuf;

#[path = "test_inc/test_mod.rs"]
mod test_common;

use serde::Deserialize;
use test_common::managed_test_output_root;
use z00z_core::assets::AssetPackPlain;
use z00z_crypto::{domains::EphemeralScalarDomain, hash::hash_to_scalar_zk, Hidden, Z00ZScalar};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{create_dir_all, read_to_string, write_file},
};
use z00z_wallets::{
    build_stealth_leaf_with_blind,
    key::{ReceiverKeys, ReceiverSecret},
    stealth::ecdh::sender_derive_dh_with_r,
    stealth::kdf::{compute_leaf_ad, derive_k_dh, derive_s_out},
    stealth::zkpack::ZkPack,
};

fn make_keys(secret: [u8; 32]) -> ReceiverKeys {
    let recv = ReceiverSecret::from_bytes(secret).expect("receiver secret");
    ReceiverKeys::from_receiver_secret(recv).expect("receiver keys")
}

#[derive(Debug, Deserialize)]
struct FixIn {
    recv_sec: String,
    r_seed: String,
    amount: u64,
    serial_id: u32,
    blind_seed: u64,
}

fn fix_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/e2e01_fix.json")
}

fn read_fix() -> FixIn {
    let raw = read_to_string(fix_path()).expect("read e2e05 fixture");
    JsonCodec
        .deserialize(raw.as_bytes())
        .expect("decode e2e05 fixture")
}

fn hex_arr<const N: usize>(raw: &str) -> [u8; N] {
    let bytes = hex::decode(raw).expect("hex decode");
    bytes.try_into().expect("hex size")
}

fn scalar_seed(seed: u64) -> Z00ZScalar {
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&seed.to_le_bytes());
    Z00ZScalar::try_from_bytes(bytes).expect("scalar seed")
}

fn hex32(raw: [u8; 32]) -> String {
    hex::encode(raw)
}

fn diff_idx(left: &[u8], right: &[u8]) -> Vec<usize> {
    left.iter()
        .zip(right.iter())
        .enumerate()
        .filter_map(|(idx, (l, r))| if l != r { Some(idx) } else { None })
        .collect()
}

fn out_dir() -> PathBuf {
    managed_test_output_root("e2e05")
}

#[test]
fn test_stage4_serial() {
    if cfg!(debug_assertions) {
        return;
    }

    let fix = read_fix();
    let recv = hex_arr::<32>(&fix.recv_sec);
    let seed = hex_arr::<32>(&fix.r_seed);
    let keys = make_keys(recv);

    let r = hash_to_scalar_zk::<EphemeralScalarDomain>("", &[&seed]).expect("derive r");
    let sender = sender_derive_dh_with_r(&keys.view_pk, &r).expect("sender derive");
    let r_pub = sender.r_pub.to_bytes();
    let k_dh = derive_k_dh(&sender.dh.to_bytes());

    let serial_a = fix.serial_id;
    let serial_b = serial_a.wrapping_add(1);
    let s_out_a = derive_s_out(&k_dh, &r_pub, serial_a);
    let s_out_b = derive_s_out(&k_dh, &r_pub, serial_b);
    assert_ne!(s_out_a, s_out_b, "s_out must differ when serial differs");

    let blind = Hidden::hide(scalar_seed(fix.blind_seed));

    let leaf_a = build_stealth_leaf_with_blind(
        &k_dh,
        &r_pub,
        &keys.owner_handle,
        fix.amount,
        serial_a,
        s_out_a,
        &blind,
    )
    .expect("build leaf a");

    let leaf_b = build_stealth_leaf_with_blind(
        &k_dh,
        &r_pub,
        &keys.owner_handle,
        fix.amount,
        serial_b,
        s_out_b,
        &blind,
    )
    .expect("build leaf b");

    assert_ne!(
        leaf_a.asset_id, leaf_b.asset_id,
        "asset_id must differ by serial-derived s_out"
    );

    let leaf_ad_a = compute_leaf_ad(
        &leaf_a.asset_id,
        leaf_a.serial_id,
        &leaf_a.r_pub,
        &leaf_a.owner_tag,
        &leaf_a.c_amount,
    );
    let leaf_ad_b = compute_leaf_ad(
        &leaf_b.asset_id,
        leaf_b.serial_id,
        &leaf_b.r_pub,
        &leaf_b.owner_tag,
        &leaf_b.c_amount,
    );
    assert_ne!(
        leaf_ad_a, leaf_ad_b,
        "leaf_ad must differ when serial differs"
    );

    let payload_a = ZkPack::decrypt(
        &k_dh,
        &leaf_ad_a,
        &leaf_a.r_pub,
        &leaf_a.asset_id,
        leaf_a.serial_id,
        &leaf_a.enc_pack,
    )
    .expect("decrypt a");
    let payload_b = ZkPack::decrypt(
        &k_dh,
        &leaf_ad_b,
        &leaf_b.r_pub,
        &leaf_b.asset_id,
        leaf_b.serial_id,
        &leaf_b.enc_pack,
    )
    .expect("decrypt b");

    assert_eq!(payload_a.len(), AssetPackPlain::SIZE);
    assert_eq!(payload_b.len(), AssetPackPlain::SIZE);

    let pack_a = AssetPackPlain::decode_checked(&payload_a).expect("decode a");
    let pack_b = AssetPackPlain::decode_checked(&payload_b).expect("decode b");
    assert_eq!(pack_a.to_bytes(), payload_a, "schema must be stable for a");
    assert_eq!(pack_b.to_bytes(), payload_b, "schema must be stable for b");

    assert_eq!(pack_a.value, pack_b.value, "value must be same");
    assert_eq!(pack_a.blinding, pack_b.blinding, "blinding must be same");
    assert_ne!(pack_a.s_out, pack_b.s_out, "s_out must differ by serial");

    assert_eq!(
        &payload_a[0..40],
        &payload_b[0..40],
        "value+blinding must be equal"
    );
    let diff = diff_idx(&payload_a, &payload_b);
    assert!(!diff.is_empty(), "payloads must differ");
    assert!(
        diff.iter().all(|idx| *idx >= 40),
        "payload differences must be only in s_out segment"
    );

    let wrong_ad_a = compute_leaf_ad(
        &leaf_a.asset_id,
        serial_b,
        &leaf_a.r_pub,
        &leaf_a.owner_tag,
        &leaf_a.c_amount,
    );
    let wrong_ad_b = compute_leaf_ad(
        &leaf_b.asset_id,
        serial_a,
        &leaf_b.r_pub,
        &leaf_b.owner_tag,
        &leaf_b.c_amount,
    );

    assert_eq!(
        ZkPack::decrypt(
            &k_dh,
            &leaf_ad_a,
            &leaf_a.r_pub,
            &leaf_a.asset_id,
            serial_b,
            &leaf_a.enc_pack,
        ),
        None,
        "leaf a must fail decrypt with wrong serial only"
    );
    assert_eq!(
        ZkPack::decrypt(
            &k_dh,
            &wrong_ad_a,
            &leaf_a.r_pub,
            &leaf_a.asset_id,
            serial_a,
            &leaf_a.enc_pack,
        ),
        None,
        "leaf a must fail decrypt with wrong leaf_ad only"
    );
    assert_eq!(
        ZkPack::decrypt(
            &k_dh,
            &wrong_ad_a,
            &leaf_a.r_pub,
            &leaf_a.asset_id,
            serial_b,
            &leaf_a.enc_pack,
        ),
        None,
        "leaf a must fail decrypt with serial b context"
    );
    assert_eq!(
        ZkPack::decrypt(
            &k_dh,
            &leaf_ad_b,
            &leaf_b.r_pub,
            &leaf_b.asset_id,
            serial_a,
            &leaf_b.enc_pack,
        ),
        None,
        "leaf b must fail decrypt with wrong serial only"
    );
    assert_eq!(
        ZkPack::decrypt(
            &k_dh,
            &wrong_ad_b,
            &leaf_b.r_pub,
            &leaf_b.asset_id,
            serial_b,
            &leaf_b.enc_pack,
        ),
        None,
        "leaf b must fail decrypt with wrong leaf_ad only"
    );
    assert_eq!(
        ZkPack::decrypt(
            &k_dh,
            &wrong_ad_b,
            &leaf_b.r_pub,
            &leaf_b.asset_id,
            serial_a,
            &leaf_b.enc_pack,
        ),
        None,
        "leaf b must fail decrypt with serial a context"
    );

    create_dir_all(out_dir()).expect("mkdir outputs/tests/e2e05");

    let mut comp = String::new();
    comp.push_str("E2E-05 compare\n");
    comp.push_str(&format!("serial_a={}\n", serial_a));
    comp.push_str(&format!("serial_b={}\n", serial_b));
    comp.push_str(&format!("payload_diff_idx={:?}\n", diff));
    comp.push_str(&format!("payload_a_hex={}\n", hex::encode(&payload_a)));
    comp.push_str(&format!("payload_b_hex={}\n", hex::encode(&payload_b)));
    write_file(out_dir().join("e2e05_compare.txt"), comp.as_bytes()).expect("write compare");

    let mut sch = String::new();
    sch.push_str("E2E-05 schema\n");
    sch.push_str("fields=(value,blinding,s_out)\n");
    sch.push_str(&format!("size={}\n", AssetPackPlain::SIZE));
    sch.push_str(&format!("s_out_a={}\n", hex32(s_out_a)));
    sch.push_str(&format!("s_out_b={}\n", hex32(s_out_b)));
    sch.push_str(&format!("leaf_ad_a={}\n", hex32(leaf_ad_a)));
    sch.push_str(&format!("leaf_ad_b={}\n", hex32(leaf_ad_b)));
    write_file(out_dir().join("e2e05_schema.txt"), sch.as_bytes()).expect("write schema");
}
