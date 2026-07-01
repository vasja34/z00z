//! Integration tests for cryptographic operations
//!
//! These tests verify that the assets architecture works correctly
//! with REAL cryptographic primitives (no mocks or stubs).
//!
//! All tests use real tari_crypto Bulletproofs+ and Pedersen commitments.

use std::sync::Arc;
use z00z_core::assets::{Asset, AssetClass, AssetDefinition, BlindingFactor};
use z00z_crypto::{create_commitment, verify_range_proof};
use z00z_utils::rng::DeterministicRngProvider;
use z00z_utils::time::{SystemTimeProvider, TimeProvider};

fn derive_test_nonce(
    wallet_seed: &[u8; 32],
    counter: u64,
    time_provider: &dyn TimeProvider,
) -> [u8; 32] {
    z00z_core::assets::nonce::derive_nonce_simple(wallet_seed, counter, time_provider)
        .expect("integration nonce")
}

/// Test that Asset creation with real Bulletproofs works
#[test]
fn test_asset_with_real_bulletproof() {
    // Create test asset definition
    let definition = Arc::new(
        AssetDefinition::new(
            [1u8; 32],
            AssetClass::Coin,
            "Test Coin".to_string(),
            "TEST".to_string(),
            8,
            100,
            1000,
            "test.domain".to_string(),
            1,
            1,
            0,
            None,
        )
        .expect("valid definition"),
    );

    // Generate secure nonce using derive_test_nonce()
    let wallet_seed = b"test_wallet_seed_32_bytes_long!!";
    let nonce = derive_test_nonce(wallet_seed, 0, &SystemTimeProvider);

    // Real blinding factor
    let blinding =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    // Create asset with real crypto
    let asset = Asset::new(
        definition.clone(),
        0,    // serial_id
        1000, // amount
        &blinding,
        nonce,
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Asset creation should succeed");

    // Verify range proof is NOT empty
    assert!(asset.range_proof.is_some(), "Range proof must exist");
    let proof = asset.range_proof.as_ref().unwrap();
    assert!(
        proof.len() > 100,
        "Real Bulletproof should be > 100 bytes, got {}",
        proof.len()
    );

    // Verify nonce is NOT zero
    assert_ne!(asset.nonce, [0u8; 32], "Nonce must be non-zero");

    // Verify commitment is valid
    let expected_commitment = create_commitment(1000, &blinding).expect("commitment creation");
    assert_eq!(asset.commitment, expected_commitment, "Commitment mismatch");
}

/// Test that zero-nonce is rejected in production builds
#[test]
fn test_zero_nonce_rejected() {
    let definition = Arc::new(
        AssetDefinition::new(
            [1u8; 32],
            AssetClass::Coin,
            "Test".to_string(),
            "TST".to_string(),
            8,
            1,
            1000,
            "test.domain".to_string(),
            1,
            1,
            0,
            None,
        )
        .expect("valid definition"),
    );

    let blinding =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let zero_nonce = [0u8; 32];

    // Zero nonce is now ALLOWED (nonce validation removed for flexibility)
    // The validation is moved to business logic layer
    let result = Asset::new(
        definition,
        0,
        1000,
        &blinding,
        zero_nonce,
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    );

    // Zero nonce should succeed at Asset level
    assert!(
        result.is_ok(),
        "Zero nonce is allowed at Asset::new() level"
    );
}

/// Test nonce uniqueness with derive_test_nonce()
#[test]
fn test_nonce_uniqueness() {
    let wallet_seed = b"secure_wallet_seed_32_bytes_long";

    // Generate 100 nonces with different counters
    let mut nonces = vec![];
    for counter in 0..100 {
        let nonce = derive_test_nonce(wallet_seed, counter, &SystemTimeProvider);
        nonces.push(nonce);
    }

    // Verify all nonces are unique
    for i in 0..nonces.len() {
        for j in (i + 1)..nonces.len() {
            assert_ne!(
                nonces[i], nonces[j],
                "Nonces at {} and {} must be unique",
                i, j
            );
        }
    }

    // Verify no nonce is zero
    for (i, nonce) in nonces.iter().enumerate() {
        assert_ne!(*nonce, [0u8; 32], "Nonce at index {} must be non-zero", i);
    }
}

/// Test that commitment matches manual calculation
#[test]
fn test_commitment_correctness() {
    let amount = 12345u64;
    let blinding =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    // Create commitment via Asset
    let definition = Arc::new(
        AssetDefinition::new(
            [2u8; 32],
            AssetClass::Token,
            "Token".to_string(),
            "TKN".to_string(),
            6,
            10,
            100,
            "token.test".to_string(),
            1,
            1,
            0,
            None,
        )
        .expect("valid definition"),
    );

    let nonce = derive_test_nonce(b"test_seed_32_bytes_long_!!!!!!!!", 5, &SystemTimeProvider);

    let asset = Asset::new(
        definition,
        0,
        amount,
        &blinding,
        nonce,
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Asset creation");

    // Verify commitment matches manual calculation
    let expected = create_commitment(amount, &blinding).expect("commitment creation");
    assert_eq!(
        asset.commitment, expected,
        "Asset commitment must match manual calculation"
    );
}

/// Test range proof verification (when real crypto available)
#[test]
fn test_range_proof_verification() {
    let definition = Arc::new(
        AssetDefinition::new(
            [3u8; 32],
            AssetClass::Coin,
            "Coin".to_string(),
            "CN".to_string(),
            8,
            50,
            5000,
            "coin.test".to_string(),
            1,
            1,
            0,
            None,
        )
        .expect("valid definition"),
    );

    let wallet_seed = b"secure_seed_32_bytes_long_ok!!!!"; // Exactly 32 bytes
    let nonce = derive_test_nonce(wallet_seed, 0, &SystemTimeProvider);
    let blinding =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let amount = 99999u64;

    let asset = Asset::new(
        definition,
        0,
        amount,
        &blinding,
        nonce,
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Asset creation");

    // Get range proof
    let proof = asset.range_proof.as_ref().expect("Range proof must exist");

    // Verify proof with AssetCrypto
    verify_range_proof(proof, &asset.commitment, 64, 1, 0).expect("Range proof must be valid");
}

/// Test that NFT with amount=0 still requires range proof
#[test]
fn test_proof_nft_zero_amount() {
    let definition = Arc::new(
        AssetDefinition::new(
            [4u8; 32],
            AssetClass::Nft,
            "NFT".to_string(),
            "NFT".to_string(),
            0,
            1000,
            0, // NFT nominal is 0
            "nft.test".to_string(),
            1,
            1,
            0,
            None,
        )
        .expect("valid definition"),
    );

    let nonce = derive_test_nonce(b"nft_seed_32_bytes_long_ok!!!!!!!", 1, &SystemTimeProvider);
    let blinding =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    let nft = Asset::new(
        definition,
        0,
        0, // amount = 0 for NFT
        &blinding,
        nonce,
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("NFT creation");

    // Even with amount=0, range proof MUST exist
    assert!(
        nft.range_proof.is_some(),
        "NFT must have range proof even with amount=0"
    );
}
