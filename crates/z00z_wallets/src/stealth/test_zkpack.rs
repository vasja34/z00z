use super::super::{
    kdf::{derive_nonce, derive_pack_key, derive_pack_nonce},
    tag::compute_leaf_ad,
};
use super::ZkPack;
use serde::Deserialize;
use z00z_crypto::protocol::zkpack::{ZKPACK_CT_LEN, ZKPACK_TAG_LEN};
use z00z_utils::{
    codec::{Codec, YamlCodec},
    io::read_to_string,
};

#[derive(Debug, Deserialize)]
struct ZkPackVecFile {
    vectors: Vec<ZkPackVec>,
}

#[derive(Debug, Deserialize)]
struct ZkPackVec {
    k_dh_hex: String,
    leaf_ad_hex: String,
    r_pub_hex: String,
    asset_id_hex: String,
    serial_id: u32,
    plaintext_hex: String,
    pack_key_hex: String,
    nonce12_hex: String,
    ciphertext_hex: String,
    tag_hex: String,
}

fn plain72() -> [u8; ZKPACK_CT_LEN] {
    [0x5Au8; ZKPACK_CT_LEN]
}

fn hex_to_arr32(hex_str: &str) -> [u8; 32] {
    let bytes = hex::decode(hex_str).expect("hex");
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    out
}

fn to_hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[test]
fn test_aead_ctx_valid() {
    let k_dh = [1u8; 32];
    let leaf_ad = [2u8; 32];
    let r_pub = [3u8; 32];
    let asset_id = [4u8; 32];
    let serial = 9u32;
    let pt = plain72();

    let enc = ZkPack::encrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, serial, &pt);
    assert_eq!(k_dh.len(), 32);
    assert_eq!(leaf_ad.len(), 32);
    assert_eq!(enc.ciphertext.len(), ZKPACK_CT_LEN);
    assert_eq!(enc.tag.len(), ZKPACK_TAG_LEN);
}

#[test]
fn test_aead_out_size() {
    let k_dh = [10u8; 32];
    let leaf_ad = [11u8; 32];
    let r_pub = [12u8; 32];
    let asset_id = [13u8; 32];
    let pt = plain72();

    let enc = ZkPack::encrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 1, &pt);
    assert_eq!(enc.ciphertext.len(), 72);
    assert_eq!(enc.tag.len(), 16);
}

#[test]
fn test_tag_len_16() {
    let k_dh = [14u8; 32];
    let leaf_ad = [15u8; 32];
    let r_pub = [16u8; 32];
    let asset_id = [17u8; 32];
    let pt = plain72();

    let enc = ZkPack::encrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 2, &pt);
    assert_eq!(enc.tag.len(), 16);
}

#[test]
fn test_tag_flip_none() {
    let k_dh = [18u8; 32];
    let leaf_ad = [19u8; 32];
    let r_pub = [20u8; 32];
    let asset_id = [21u8; 32];
    let pt = plain72();

    let mut enc = ZkPack::encrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 3, &pt);
    enc.tag[0] ^= 0x01;
    let dec = ZkPack::decrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 3, &enc);
    assert!(dec.is_none());
}

#[test]
fn test_aad_wrong_none() {
    let k_dh = [22u8; 32];
    let leaf_ad_a = [23u8; 32];
    let leaf_ad_b = [24u8; 32];
    let r_pub = [25u8; 32];
    let asset_id = [26u8; 32];
    let pt = plain72();

    let enc = ZkPack::encrypt(&k_dh, &leaf_ad_a, &r_pub, &asset_id, 4, &pt);
    let dec = ZkPack::decrypt(&k_dh, &leaf_ad_b, &r_pub, &asset_id, 4, &enc);
    assert!(dec.is_none());
}

#[test]
fn test_diff_rpub_diff_ct() {
    let k_dh = [27u8; 32];
    let leaf_ad = [28u8; 32];
    let r_pub_a = [29u8; 32];
    let r_pub_b = [30u8; 32];
    let asset_id = [31u8; 32];
    let pt = plain72();

    let enc_a = ZkPack::encrypt(&k_dh, &leaf_ad, &r_pub_a, &asset_id, 5, &pt);
    let enc_b = ZkPack::encrypt(&k_dh, &leaf_ad, &r_pub_b, &asset_id, 5, &pt);
    assert_ne!(enc_a.ciphertext, enc_b.ciphertext);
}

#[test]
fn test_enc_ok() {
    let k_dh = [32u8; 32];
    let leaf_ad = [33u8; 32];
    let r_pub = [34u8; 32];
    let asset_id = [35u8; 32];
    let pt = plain72();

    let enc = ZkPack::encrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 6, &pt);
    assert!(!enc.ciphertext.is_empty());
}

#[test]
fn test_enc_roundtrip() {
    let k_dh = [44u8; 32];
    let leaf_ad = [45u8; 32];
    let r_pub = [46u8; 32];
    let asset_id = [47u8; 32];
    let pt = plain72();

    let enc = ZkPack::encrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 7, &pt);
    let dec = ZkPack::decrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 7, &enc);
    assert_eq!(dec, Some(pt.to_vec()));
}

#[test]
fn test_zero_kdh_enc() {
    let k_dh = [0u8; 32];
    let leaf_ad = [52u8; 32];
    let r_pub = [53u8; 32];
    let asset_id = [54u8; 32];
    let pt = plain72();

    let enc = ZkPack::encrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 8, &pt);
    let dec = ZkPack::decrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 8, &enc);
    assert_eq!(dec, Some(pt.to_vec()));
}

#[test]
fn test_wrong_ver_none() {
    let k_dh = [55u8; 32];
    let leaf_ad = [56u8; 32];
    let r_pub = [57u8; 32];
    let asset_id = [58u8; 32];
    let pt = plain72();

    let mut enc = ZkPack::encrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 9, &pt);
    enc.version = 2;
    let dec = ZkPack::decrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 9, &enc);
    assert!(dec.is_none());
}

#[test]
fn test_wrong_kdh_none() {
    let k_a = [59u8; 32];
    let k_b = [60u8; 32];
    let leaf_ad = [61u8; 32];
    let r_pub = [62u8; 32];
    let asset_id = [63u8; 32];
    let pt = plain72();

    let enc = ZkPack::encrypt(&k_a, &leaf_ad, &r_pub, &asset_id, 10, &pt);
    let dec = ZkPack::decrypt(&k_b, &leaf_ad, &r_pub, &asset_id, 10, &enc);
    assert!(dec.is_none());
}

#[test]
fn test_ct_flip_none() {
    let k_dh = [64u8; 32];
    let leaf_ad = [65u8; 32];
    let r_pub = [66u8; 32];
    let asset_id = [67u8; 32];
    let pt = plain72();

    let mut enc = ZkPack::encrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 11, &pt);
    enc.ciphertext[0] ^= 0x01;
    let dec = ZkPack::decrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 11, &enc);
    assert!(dec.is_none());
}

#[test]
fn test_rebind_leaf_none() {
    let k_dh = [68u8; 32];
    let r_pub = [69u8; 32];
    let asset_id = [70u8; 32];
    let owner_tag = [71u8; 32];
    let c_amount = [72u8; 32];
    let pt = plain72();

    let leaf_a = compute_leaf_ad(&asset_id, 42, &r_pub, &owner_tag, &c_amount);
    let mut leaf_b = leaf_a;
    leaf_b[0] ^= 0x01;
    let enc = ZkPack::encrypt(&k_dh, &leaf_a, &r_pub, &asset_id, 42, &pt);
    let dec = ZkPack::decrypt(&k_dh, &leaf_b, &r_pub, &asset_id, 42, &enc);
    assert!(dec.is_none());
}

#[test]
fn test_owner_swap_none() {
    let k_dh = [73u8; 32];
    let r_pub = [74u8; 32];
    let asset_id = [75u8; 32];
    let owner_a = [76u8; 32];
    let owner_b = [77u8; 32];
    let c_amount = [78u8; 32];
    let pt = plain72();

    let leaf_a = compute_leaf_ad(&asset_id, 43, &r_pub, &owner_a, &c_amount);
    let leaf_b = compute_leaf_ad(&asset_id, 43, &r_pub, &owner_b, &c_amount);
    let enc = ZkPack::encrypt(&k_dh, &leaf_a, &r_pub, &asset_id, 43, &pt);
    let dec = ZkPack::decrypt(&k_dh, &leaf_b, &r_pub, &asset_id, 43, &enc);
    assert!(dec.is_none());
}

#[test]
fn test_camt_swap_none() {
    let k_dh = [79u8; 32];
    let r_pub = [80u8; 32];
    let asset_id = [81u8; 32];
    let owner = [82u8; 32];
    let c_a = [83u8; 32];
    let c_b = [84u8; 32];
    let pt = plain72();

    let leaf_a = compute_leaf_ad(&asset_id, 44, &r_pub, &owner, &c_a);
    let leaf_b = compute_leaf_ad(&asset_id, 44, &r_pub, &owner, &c_b);
    let enc = ZkPack::encrypt(&k_dh, &leaf_a, &r_pub, &asset_id, 44, &pt);
    let dec = ZkPack::decrypt(&k_dh, &leaf_b, &r_pub, &asset_id, 44, &enc);
    assert!(dec.is_none());
}

#[test]
fn test_bind_asset_id() {
    let k_dh = [91u8; 32];
    let r_pub = [92u8; 32];
    let asset_a = [93u8; 32];
    let asset_b = [94u8; 32];
    let owner = [95u8; 32];
    let c_amt = [96u8; 32];
    let serial = 55u32;
    let pt = plain72();

    let leaf_a = compute_leaf_ad(&asset_a, serial, &r_pub, &owner, &c_amt);
    let leaf_b = compute_leaf_ad(&asset_b, serial, &r_pub, &owner, &c_amt);
    let enc = ZkPack::encrypt(&k_dh, &leaf_a, &r_pub, &asset_a, serial, &pt);
    let dec = ZkPack::decrypt(&k_dh, &leaf_b, &r_pub, &asset_a, serial, &enc);
    assert!(dec.is_none());
}

#[test]
fn test_bind_serial_id() {
    let k_dh = [97u8; 32];
    let r_pub = [98u8; 32];
    let asset_id = [99u8; 32];
    let owner = [100u8; 32];
    let c_amt = [101u8; 32];
    let pt = plain72();

    let leaf_42 = compute_leaf_ad(&asset_id, 42, &r_pub, &owner, &c_amt);
    let leaf_43 = compute_leaf_ad(&asset_id, 43, &r_pub, &owner, &c_amt);
    let enc = ZkPack::encrypt(&k_dh, &leaf_42, &r_pub, &asset_id, 42, &pt);
    let dec = ZkPack::decrypt(&k_dh, &leaf_43, &r_pub, &asset_id, 42, &enc);
    assert!(dec.is_none());
}

#[test]
fn test_bind_r_pub() {
    let k_dh = [102u8; 32];
    let r_a = [103u8; 32];
    let r_b = [104u8; 32];
    let asset_id = [105u8; 32];
    let owner = [106u8; 32];
    let c_amt = [107u8; 32];
    let serial = 77u32;
    let pt = plain72();

    let leaf_a = compute_leaf_ad(&asset_id, serial, &r_a, &owner, &c_amt);
    let leaf_b = compute_leaf_ad(&asset_id, serial, &r_b, &owner, &c_amt);
    let enc = ZkPack::encrypt(&k_dh, &leaf_a, &r_a, &asset_id, serial, &pt);
    let dec = ZkPack::decrypt(&k_dh, &leaf_b, &r_a, &asset_id, serial, &enc);
    assert!(dec.is_none());
}

#[test]
fn test_bind_owner_tag() {
    let k_dh = [108u8; 32];
    let r_pub = [109u8; 32];
    let asset_id = [110u8; 32];
    let owner_a = [111u8; 32];
    let owner_b = [112u8; 32];
    let c_amt = [113u8; 32];
    let serial = 88u32;
    let pt = plain72();

    let leaf_a = compute_leaf_ad(&asset_id, serial, &r_pub, &owner_a, &c_amt);
    let leaf_b = compute_leaf_ad(&asset_id, serial, &r_pub, &owner_b, &c_amt);
    let enc = ZkPack::encrypt(&k_dh, &leaf_a, &r_pub, &asset_id, serial, &pt);
    let dec = ZkPack::decrypt(&k_dh, &leaf_b, &r_pub, &asset_id, serial, &enc);
    assert!(dec.is_none());
}

#[test]
fn test_bind_c_amount() {
    let k_dh = [114u8; 32];
    let r_pub = [115u8; 32];
    let asset_id = [116u8; 32];
    let owner = [117u8; 32];
    let c_a = [118u8; 32];
    let c_b = [119u8; 32];
    let serial = 89u32;
    let pt = plain72();

    let leaf_a = compute_leaf_ad(&asset_id, serial, &r_pub, &owner, &c_a);
    let leaf_b = compute_leaf_ad(&asset_id, serial, &r_pub, &owner, &c_b);
    let enc = ZkPack::encrypt(&k_dh, &leaf_a, &r_pub, &asset_id, serial, &pt);
    let dec = ZkPack::decrypt(&k_dh, &leaf_b, &r_pub, &asset_id, serial, &enc);
    assert!(dec.is_none());
}

#[test]
fn test_golden_yaml() {
    let file_path = format!(
        "{}/tests/fixtures/zkpack_golden.yaml",
        env!("CARGO_MANIFEST_DIR")
    );
    let yaml_text = read_to_string(&file_path).expect("read yaml");
    let codec = YamlCodec;
    let vectors: ZkPackVecFile = codec.deserialize(yaml_text.as_bytes()).expect("parse yaml");

    for item in vectors.vectors {
        let k_dh = hex_to_arr32(&item.k_dh_hex);
        let leaf_ad = hex_to_arr32(&item.leaf_ad_hex);
        let r_pub = hex_to_arr32(&item.r_pub_hex);
        let asset_id = hex_to_arr32(&item.asset_id_hex);
        let pt = hex::decode(&item.plaintext_hex).expect("pt");

        let pack_key = derive_pack_key(&k_dh, &asset_id, item.serial_id);
        let nonce12 = derive_nonce(&leaf_ad, &r_pub, &asset_id, item.serial_id);
        let enc = ZkPack::encrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, item.serial_id, &pt);

        assert_eq!(to_hex_lower(&pack_key), item.pack_key_hex);
        assert_eq!(to_hex_lower(&nonce12), item.nonce12_hex);
        assert_eq!(to_hex_lower(&enc.ciphertext), item.ciphertext_hex);
        assert_eq!(to_hex_lower(&enc.tag), item.tag_hex);

        let dec = ZkPack::decrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, item.serial_id, &enc);
        assert_eq!(dec, Some(pt));
    }
}

#[test]
fn test_replay_block_serial() {
    let k_dh = [1u8; 32];
    let r_pub = [2u8; 32];
    let asset_id = [3u8; 32];
    let owner_tag = [4u8; 32];
    let c_amount = [5u8; 32];
    let plain = b"z00z-pack-v1";

    let leaf42 = compute_leaf_ad(&asset_id, 42, &r_pub, &owner_tag, &c_amount);
    let enc = ZkPack::encrypt(&k_dh, &leaf42, &r_pub, &asset_id, 42, plain);

    let leaf43 = compute_leaf_ad(&asset_id, 43, &r_pub, &owner_tag, &c_amount);
    assert_ne!(leaf42, leaf43);

    let bad = ZkPack::decrypt(&k_dh, &leaf43, &r_pub, &asset_id, 43, &enc);
    assert!(bad.is_none());
}

#[test]
fn test_replay_leaf_mismatch() {
    let k_dh = [21u8; 32];
    let r_pub = [22u8; 32];
    let asset_id = [23u8; 32];
    let owner_tag = [24u8; 32];
    let c_amount = [25u8; 32];
    let plain = b"z00z-pack-v1";

    let leaf_ok = compute_leaf_ad(&asset_id, 42, &r_pub, &owner_tag, &c_amount);
    let enc = ZkPack::encrypt(&k_dh, &leaf_ok, &r_pub, &asset_id, 42, plain);

    let leaf_bad = compute_leaf_ad(&asset_id, 43, &r_pub, &owner_tag, &c_amount);
    assert_ne!(leaf_ok, leaf_bad);

    let bad = ZkPack::decrypt(&k_dh, &leaf_bad, &r_pub, &asset_id, 42, &enc);
    assert!(bad.is_none());
}

#[test]
fn test_key_diff_serial() {
    let k_dh = [7u8; 32];
    let asset_id = [8u8; 32];
    let key42 = derive_pack_key(&k_dh, &asset_id, 42);
    let key43 = derive_pack_key(&k_dh, &asset_id, 43);
    assert_ne!(key42, key43);
}

#[test]
fn test_nonce_diff_serial() {
    let leaf_ad = [9u8; 32];
    let r_pub = [10u8; 32];
    let asset_id = [11u8; 32];
    let nonce42 = derive_pack_nonce(&leaf_ad, &r_pub, &asset_id, 42);
    let nonce43 = derive_pack_nonce(&leaf_ad, &r_pub, &asset_id, 43);
    assert_ne!(nonce42, nonce43);
}
