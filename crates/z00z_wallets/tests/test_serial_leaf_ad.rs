use serde::Deserialize;
use z00z_utils::{
    codec::{Codec, YamlCodec},
    io::read_to_string,
};
use z00z_wallets::stealth::kdf::compute_leaf_ad;

#[derive(Debug, Deserialize)]
struct LeafAdVecFile {
    spec_version: String,
    vectors: Vec<LeafAdVec>,
}

#[derive(Debug, Deserialize)]
struct LeafAdVec {
    asset_id_hex: String,
    serial_id: u32,
    r_pub_hex: String,
    owner_tag_hex: String,
    c_amount_hex: String,
    leaf_ad_hex: String,
}

fn hex_to_arr32(hex_str: &str) -> [u8; 32] {
    let bytes = hex::decode(hex_str).expect("hex decode failed");
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
fn test_serial_leaf_diff() {
    let asset_id = [0x11u8; 32];
    let r_pub = [0x22u8; 32];
    let owner_tag = [0x33u8; 32];
    let c_amount = [0x44u8; 32];

    let leaf_42 = compute_leaf_ad(&asset_id, 42, &r_pub, &owner_tag, &c_amount);
    let leaf_43 = compute_leaf_ad(&asset_id, 43, &r_pub, &owner_tag, &c_amount);

    assert_ne!(leaf_42, leaf_43);
}

#[test]
fn test_leaf_ad_vectors() {
    let asset_id = [0x11u8; 32];
    let r_pub = [0x22u8; 32];
    let owner_tag = [0x33u8; 32];
    let c_amount = [0x44u8; 32];

    let got_0 = compute_leaf_ad(&asset_id, 0, &r_pub, &owner_tag, &c_amount);
    let got_42 = compute_leaf_ad(&asset_id, 42, &r_pub, &owner_tag, &c_amount);
    let got_max = compute_leaf_ad(&asset_id, u32::MAX, &r_pub, &owner_tag, &c_amount);

    assert_eq!(
        got_0,
        [
            0x39, 0x50, 0x9a, 0x8b, 0x76, 0x24, 0x8d, 0xd1, 0xaa, 0xf2, 0x43, 0x29, 0x42, 0x67,
            0x8e, 0x29, 0x75, 0xe1, 0x52, 0x37, 0x51, 0xeb, 0xbd, 0x61, 0xa4, 0x25, 0xba, 0xb5,
            0xbf, 0x6e, 0x1b, 0x8c,
        ]
    );
    assert_eq!(
        got_42,
        [
            0x57, 0x41, 0x1d, 0x24, 0xab, 0x69, 0xc4, 0x82, 0x99, 0x87, 0x40, 0x17, 0xe6, 0x51,
            0x63, 0x6a, 0xf2, 0xcd, 0xba, 0xc3, 0x15, 0x5d, 0x2b, 0x1d, 0x95, 0x04, 0x07, 0x2b,
            0x70, 0x47, 0x8e, 0x9d,
        ]
    );
    assert_eq!(
        got_max,
        [
            0xc6, 0xeb, 0xea, 0xd3, 0x0e, 0x18, 0xad, 0x64, 0x04, 0x98, 0x2f, 0x38, 0x2e, 0xcb,
            0xae, 0x31, 0xdb, 0x7b, 0x98, 0x9b, 0x72, 0xba, 0xb0, 0x76, 0xce, 0x57, 0x8a, 0x8f,
            0x25, 0x3b, 0x76, 0xf2,
        ]
    );
}

#[test]
fn test_serial_le_encoding() {
    assert_eq!(u32::to_le_bytes(0x1234_5678), [0x78, 0x56, 0x34, 0x12]);
}

#[test]
fn test_serial_le_roundtrip() {
    let serial = 0x89ab_cdefu32;
    let bytes = serial.to_le_bytes();
    assert_eq!(u32::from_le_bytes(bytes), serial);
}

#[test]
fn test_golden_leaf_ad_vectors() {
    let file_path = format!(
        "{}/tests/fixtures/leaf_ad_vectors.yaml",
        env!("CARGO_MANIFEST_DIR")
    );
    let yaml_text = read_to_string(&file_path).expect("read yaml");
    let codec = YamlCodec;
    let vectors: LeafAdVecFile = codec.deserialize(yaml_text.as_bytes()).expect("parse yaml");

    assert_eq!(vectors.spec_version, "4.7.4.3-v1");
    assert!(vectors.vectors.len() >= 3, "need at least 3 golden vectors");

    for item in vectors.vectors {
        let asset_id = hex_to_arr32(&item.asset_id_hex);
        let r_pub = hex_to_arr32(&item.r_pub_hex);
        let owner_tag = hex_to_arr32(&item.owner_tag_hex);
        let c_amount = hex_to_arr32(&item.c_amount_hex);

        let got = compute_leaf_ad(&asset_id, item.serial_id, &r_pub, &owner_tag, &c_amount);
        assert_eq!(to_hex_lower(&got), item.leaf_ad_hex);
    }
}
