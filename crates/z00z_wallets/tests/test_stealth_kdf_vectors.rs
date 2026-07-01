use std::path::PathBuf;

use serde::Deserialize;
use z00z_crypto::kdf;
use z00z_utils::{
    codec::{Codec, YamlCodec},
    io::read_to_string,
};
use z00z_wallets::stealth::kdf::{
    compute_leaf_ad, compute_owner_tag, compute_tag16, compute_tag16_with_req, derive_k_dh,
    derive_k_dh_with_req, derive_s_out,
};

#[derive(Debug, Deserialize)]
struct StealthKdfVecFile {
    spec_version: String,
    vectors: Vec<StealthKdfVec>,
}

#[derive(Debug, Deserialize)]
struct StealthKdfVec {
    label: String,
    dh_hex: String,
    req_id_hex: Option<String>,
    k_dh_hex: String,
    r_pub_hex: String,
    serial_id: u32,
    c_amount_hex: String,
    s_out_hex: String,
    asset_id_hex: String,
    owner_handle_hex: String,
    owner_tag_hex: String,
    leaf_ad_hex: String,
    tag16: Option<u16>,
    tag16_with_req: Option<u16>,
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/stealth_kdf_vectors.yaml")
}

fn load_vectors() -> Vec<StealthKdfVec> {
    let yaml_text = read_to_string(fixture_path()).expect("read fixture");
    let codec = YamlCodec;
    let file: StealthKdfVecFile = codec.deserialize(yaml_text.as_bytes()).expect("parse yaml");
    assert_eq!(file.spec_version, "stealth-kdf-v1");
    file.vectors
}

fn parse_hex<const N: usize>(hex_in: &str) -> [u8; N] {
    let raw = hex::decode(hex_in).expect("hex decode");
    raw.try_into().expect("hex size")
}

fn find_vec<'a>(vectors: &'a [StealthKdfVec], label: &str) -> &'a StealthKdfVec {
    vectors
        .iter()
        .find(|vector| vector.label == label)
        .expect("vector")
}

#[test]
fn test_stealth_kdf_vectors() {
    for vector in load_vectors() {
        let dh = parse_hex::<32>(&vector.dh_hex);
        let r_pub = parse_hex::<32>(&vector.r_pub_hex);
        let c_amount = parse_hex::<32>(&vector.c_amount_hex);
        let owner_handle = parse_hex::<32>(&vector.owner_handle_hex);
        let k_dh = match &vector.req_id_hex {
            Some(req_id_hex) => {
                let req_id = parse_hex::<32>(req_id_hex);
                derive_k_dh_with_req(&dh, &req_id)
            }
            None => derive_k_dh(&dh),
        };
        let s_out = derive_s_out(&k_dh, &r_pub, vector.serial_id);
        let asset_id = kdf::derive_asset_id(&s_out);
        let owner_tag = compute_owner_tag(&owner_handle, &k_dh);
        let leaf_ad = compute_leaf_ad(&asset_id, vector.serial_id, &r_pub, &owner_tag, &c_amount);

        assert_eq!(hex::encode(k_dh), vector.k_dh_hex);
        assert_eq!(hex::encode(s_out), vector.s_out_hex);
        assert_eq!(hex::encode(asset_id), vector.asset_id_hex);
        assert_eq!(hex::encode(owner_tag), vector.owner_tag_hex);
        assert_eq!(hex::encode(leaf_ad), vector.leaf_ad_hex);

        match (&vector.req_id_hex, vector.tag16, vector.tag16_with_req) {
            (Some(req_id_hex), None, Some(expected)) => {
                let req_id = parse_hex::<32>(req_id_hex);
                assert_eq!(compute_tag16_with_req(&k_dh, &req_id), expected);
            }
            (None, Some(expected), None) => {
                assert_eq!(compute_tag16(&k_dh, &leaf_ad), expected);
            }
            _ => panic!("invalid fixture tag layout"),
        }
    }
}

#[test]
fn test_tag_arg_order_gap() {
    let vectors = load_vectors();
    let base = find_vec(&vectors, "base-card");
    let k_dh = parse_hex::<32>(&base.k_dh_hex);
    let leaf_ad = parse_hex::<32>(&base.leaf_ad_hex);

    assert_ne!(
        compute_tag16(&k_dh, &leaf_ad),
        compute_tag16(&leaf_ad, &k_dh)
    );
}

#[test]
fn test_leaf_arg_order_gap() {
    let vectors = load_vectors();
    let base = find_vec(&vectors, "base-card");
    let asset_id = parse_hex::<32>(&base.asset_id_hex);
    let r_pub = parse_hex::<32>(&base.r_pub_hex);
    let owner_tag = parse_hex::<32>(&base.owner_tag_hex);
    let c_amount = parse_hex::<32>(&base.c_amount_hex);

    let canonical = compute_leaf_ad(&asset_id, base.serial_id, &r_pub, &owner_tag, &c_amount);
    let swapped = compute_leaf_ad(&asset_id, base.serial_id, &owner_tag, &r_pub, &c_amount);

    assert_ne!(canonical, swapped);
}

#[test]
fn test_request_bound_diverges() {
    let vectors = load_vectors();
    let base = find_vec(&vectors, "base-card");
    let request = find_vec(&vectors, "request-bound");

    assert_eq!(base.dh_hex, request.dh_hex);
    assert_eq!(base.r_pub_hex, request.r_pub_hex);
    assert_eq!(base.owner_handle_hex, request.owner_handle_hex);
    assert_eq!(base.serial_id, request.serial_id);
    assert_eq!(base.c_amount_hex, request.c_amount_hex);
    assert_ne!(base.k_dh_hex, request.k_dh_hex);
    assert_ne!(base.s_out_hex, request.s_out_hex);
    assert_ne!(base.asset_id_hex, request.asset_id_hex);
    assert_ne!(base.owner_tag_hex, request.owner_tag_hex);
    assert_ne!(base.leaf_ad_hex, request.leaf_ad_hex);
    assert_ne!(base.tag16, request.tag16_with_req);
}
