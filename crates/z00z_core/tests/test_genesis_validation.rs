//! Asset Validation Tests for Genesis Module
//!
//! Tests to verify genesis assets have all required fields correctly initialized.

use crate::genesis::helpers::*;
use z00z_core::assets::AssetClass;
use z00z_core::genesis::GenesisAssetAccumulator;
use z00z_crypto::expert::encoding::ByteArray;

#[test]
fn test_asset_genesis_has_fields() {
    let asset = generate_test_asset();

    // Verify all fields initialized correctly
    assert_eq!(asset.serial_id, 0);
    assert_eq!(asset.amount, 1_000_000);
    assert!(
        asset.range_proof.is_some(),
        "Genesis asset must have range proof"
    );

    // Verify commitment is valid (32 bytes)
    assert_eq!(asset.commitment.as_bytes().len(), 32);

    // Verify nonce is non-zero
    assert_ne!(asset.nonce, [0u8; 32]);

    // Verify definition reference is valid
    assert_eq!(asset.definition.symbol, "TST");
    assert_eq!(asset.definition.class, z00z_core::assets::AssetClass::Coin);
}

#[test]
fn test_genesis_asset_proof_verifies() {
    let asset = generate_test_asset();

    // Verify range proof is present
    assert!(
        asset.range_proof.is_some(),
        "Genesis asset must have range proof"
    );

    // Note: Full proof verification is tested in validator tests
    // Here we just verify the proof exists and has correct structure
    if let Some(proof) = &asset.range_proof {
        // Proof should serialize to non-empty bytes
        let proof_bytes = proof.as_bytes();
        assert!(
            !proof_bytes.is_empty(),
            "Range proof should have non-zero size"
        );
    }
}

#[test]
fn test_asset_genesis_commitment() {
    let asset = generate_test_asset();

    // Commitment should never be all zeros (cryptographic security)
    let commitment_bytes = asset.commitment.as_bytes();
    let all_zeros = commitment_bytes.iter().all(|&b| b == 0);

    assert!(!all_zeros, "Commitment must not be all zeros");
}

#[test]
fn test_asset_genesis_different_serials() {
    let asset1 = generate_test_asset_with_id(0);
    let asset2 = generate_test_asset_with_id(1);

    // Different serial IDs should produce different commitments
    assert_ne!(
        asset1.commitment.as_bytes(),
        asset2.commitment.as_bytes(),
        "Different serials must have different commitments"
    );

    // Different nonces
    assert_ne!(
        asset1.nonce, asset2.nonce,
        "Different serials must have different nonces"
    );
}

#[test]
fn test_asset_genesis_accumulator_class() {
    let mut accumulator = GenesisAssetAccumulator::new();

    let coin = generate_test_asset_with_id(0);
    accumulator.push(coin, AssetClass::Coin);

    assert_eq!(accumulator.coins.len(), 1);
    assert_eq!(accumulator.tokens.len(), 0);
    assert_eq!(accumulator.nfts.len(), 0);
    assert_eq!(accumulator.voids.len(), 0);
    assert_eq!(accumulator.total_count(), 1);
}

#[test]
fn test_asset_genesis_accumulator_get() {
    let accumulator = generate_test_accumulator();

    let coins = accumulator.get_by_class(AssetClass::Coin);
    assert_eq!(coins.len(), 5);

    let tokens = accumulator.get_by_class(AssetClass::Token);
    assert_eq!(tokens.len(), 0);
}

#[test]
fn test_genesis_asset_accumulator_flatten() {
    let accumulator = generate_test_accumulator();

    let flat = accumulator.flatten();
    assert_eq!(flat.len(), 5);
    assert_eq!(flat.len(), accumulator.total_count());
}
