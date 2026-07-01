use std::path::PathBuf;

#[path = "test_inc/test_mod.rs"]
mod test_common;

use test_common::managed_test_output_root;
use z00z_core::assets::{leaf::PackErr, AssetPackPlain};
use z00z_crypto::Z00ZScalar;
use z00z_utils::io::{create_dir_all, write_file};

fn scalar_seed(seed: u64) -> Z00ZScalar {
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&seed.to_le_bytes());
    Z00ZScalar::try_from_bytes(bytes).expect("scalar seed")
}

fn base_pack() -> AssetPackPlain {
    let mut s_out = [0u8; 32];
    for (idx, val) in s_out.iter_mut().enumerate() {
        *val = idx as u8;
    }

    AssetPackPlain {
        value: 42_000,
        blinding: scalar_seed(7).to_bytes(),
        s_out,
    }
}

fn rotate_tail_left(bytes: &[u8], at: usize) -> Vec<u8> {
    let mut out = bytes.to_vec();
    let keep = out[at];
    out[at..(AssetPackPlain::SIZE - 1)].copy_from_slice(&bytes[(at + 1)..AssetPackPlain::SIZE]);
    out[AssetPackPlain::SIZE - 1] = keep;
    out
}

fn endian_flip(bytes: &[u8]) -> Vec<u8> {
    let mut out = bytes.to_vec();
    out[..8].reverse();
    out
}

fn mk_noise(len: usize) -> Vec<u8> {
    let mut out = vec![0u8; len];
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    for byte in &mut out {
        state ^= state << 7;
        state ^= state >> 9;
        state ^= state << 8;
        *byte = (state & 0xFF) as u8;
    }
    out
}

fn out_dir() -> PathBuf {
    managed_test_output_root("e2e04")
}

fn write_bin(name: &str, bytes: &[u8]) {
    write_file(out_dir().join(name), bytes).expect("write fixture")
}

#[test]
fn test_stage4_assetpack() {
    let base = base_pack();
    let bytes = base.to_bytes();

    assert_eq!(
        bytes.len(),
        AssetPackPlain::SIZE,
        "baseline must be 72 bytes"
    );

    let decoded = AssetPackPlain::decode_checked(&bytes).expect("baseline decode");
    assert_eq!(decoded, base, "baseline decode must match expected fields");

    let mut log = String::new();
    log.push_str("E2E-04 decode log\n");
    log.push_str("case|result\n");

    for len in 0..=71usize {
        let bad = vec![0u8; len];
        let got = AssetPackPlain::decode_strict(&bad);
        assert_eq!(got, Err(PackErr::BadLen), "bad len {} must be BadLen", len);
    }

    for len in 73..=80usize {
        let bad = vec![0u8; len];
        let got = AssetPackPlain::decode_strict(&bad);
        assert_eq!(got, Err(PackErr::BadLen), "bad len {} must be BadLen", len);
    }

    let big = mk_noise(4096);
    assert_eq!(
        AssetPackPlain::decode_strict(&big),
        Err(PackErr::BadLen),
        "large random len must be BadLen"
    );
    log.push_str("invalid_len|BadLen\n");

    let flip = endian_flip(&bytes);
    let flip_dec = AssetPackPlain::decode_checked(&flip).expect("endian decode");
    assert_ne!(flip_dec.value, base.value, "endian-flip must change value");
    log.push_str("endian_flip|ValueMismatch\n");

    let sh8 = rotate_tail_left(&bytes, 8);
    let sh40 = rotate_tail_left(&bytes, 40);
    let got8 = AssetPackPlain::decode_checked(&sh8);
    let got40 = AssetPackPlain::decode_checked(&sh40);

    assert!(
        got8 != Ok(base.clone()),
        "offset shift at 8 must not decode as canonical payload"
    );
    assert!(
        got40 != Ok(base.clone()),
        "offset shift at 40 must not decode as canonical payload"
    );
    log.push_str("offset_8|NonCanonical\n");
    log.push_str("offset_40|NonCanonical\n");

    let mut blind_bad = bytes.clone();
    blind_bad[8..40].fill(0xFF);
    assert_eq!(
        AssetPackPlain::decode_checked(&blind_bad),
        Err(PackErr::BadBlind),
        "bad blinding must map to BadBlind"
    );
    log.push_str("bad_blinding|BadBlind\n");

    create_dir_all(out_dir()).expect("mkdir outputs/tests/e2e04");
    write_bin("baseline.bin", &bytes);
    write_bin("endian_flip.bin", &flip);
    write_bin("offset_8.bin", &sh8);
    write_bin("offset_40.bin", &sh40);
    write_bin("bad_blinding.bin", &blind_bad);
    write_file(out_dir().join("e2e04_decode.txt"), log.as_bytes()).expect("write decode log");
}
