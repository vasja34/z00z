use std::path::PathBuf;

use z00z_core::{assets::AssetWire, genesis::asset_std::asset_from_dev_class, AssetClass};
use z00z_crypto::{create_commitment, Z00ZScalar};
use z00z_utils::io::{create_dir_all, write_file};
use z00z_wallets::{
    claim::{read_state, rehydrate_rows, verify_resume_wire, ClaimStateRow},
    tx::derive_balance_commitment,
};

fn scalar(seed: u64) -> Z00ZScalar {
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&seed.to_le_bytes());
    Z00ZScalar::try_from_bytes(bytes).expect("scalar")
}

fn temp_file(name: &str) -> PathBuf {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("target/claim_core_safety")
        .join(format!("{}_{}", std::process::id(), name));
    create_dir_all(base.parent().expect("parent")).expect("mkdir");
    base
}

#[test]
fn test_derive_commitment_rejects_empty() {
    let got = derive_balance_commitment(&[], &[]).expect_err("empty vectors must reject");
    assert!(got.contains("non-empty"));
}

#[test]
fn test_read_handles_malformed_json() {
    let path = temp_file("claim_state_bad.json");
    write_file(&path, b"{not-json").expect("write malformed");
    let got = read_state(&path).expect_err("malformed json must reject");
    assert!(got.contains("corrupt"));
}

#[test]
fn test_rehydrate_bad_no_panic() {
    let rows = vec![ClaimStateRow {
        wallet_id: "alice".to_string(),
        asset_id_hex: "xyz".to_string(),
    }];
    let got = rehydrate_rows(&rows).expect_err("invalid hex must reject");
    assert!(got.contains("invalid asset hex"));
}

#[test]
fn test_owner_mismatch_no_panic() {
    let asset = asset_from_dev_class(AssetClass::Coin, 1, 100).expect("asset");
    let wire: AssetWire = AssetWire::from_asset(&asset);

    let got = verify_resume_wire(&wire, Some([0xAA; 32])).expect_err("mismatch must reject");
    assert!(got.contains("owner binding mismatch"));
}

#[test]
fn test_derive_commitment_smoke_ok() {
    let in_a = create_commitment(11, &scalar(1)).expect("in a");
    let in_b = create_commitment(7, &scalar(2)).expect("in b");
    let out_a = create_commitment(3, &scalar(3)).expect("out a");

    let delta = derive_balance_commitment(&[in_a, in_b], &[out_a]).expect("derive delta");
    assert_ne!(delta.as_bytes(), [0u8; 32]);
}
