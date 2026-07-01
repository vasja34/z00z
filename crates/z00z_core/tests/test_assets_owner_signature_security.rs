//! Integration tests for owner signature security
//!
//! This test suite validates critical security properties of Asset owner signatures:
//! 1. Range proof tampering detection (proof substitution attack prevention)
//! 2. Asset field modification detection (tamper resistance)
//! 3. Wrong secret key rejection (prevents unauthorized signing)
//!
//! These tests address security issues identified in devil's advocate audit.

use std::sync::Arc;
use z00z_core::assets::definition::AssetDefinition;
use z00z_core::assets::{Asset, AssetClass};
use z00z_core::assets::{BlindingFactor, PublicKey};
use z00z_utils::rng::DeterministicRngProvider;
use z00z_utils::time::{SystemTimeProvider, TimeProvider};

fn derive_test_nonce(
    rng: &mut (impl rand::RngCore + rand::CryptoRng),
    time_provider: &dyn TimeProvider,
) -> [u8; 32] {
    z00z_core::assets::nonce::derive_nonce_minimal(rng, time_provider).expect("integration nonce")
}

/// Helper function to create test asset definition
fn create_test_definition() -> Arc<AssetDefinition> {
    Arc::new(
        AssetDefinition::new(
            [42u8; 32],
            AssetClass::Coin,
            "Test Coin".to_string(),
            "TST".to_string(),
            8,
            1000,
            100_000_000,
            "test.io".to_string(),
            1,
            1,
            0b0001_0000, // burnable flag
            None,
        )
        .expect("valid definition"),
    )
}
#[test]
fn test_rejects_proof_range_tampering() {
    // CRITICAL TEST: Verifies that range_proof is included in signature
    // Without this, attacker could replace valid proof with fake one

    let def = create_test_definition();
    let secret = BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let nonce = derive_test_nonce(
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
        &SystemTimeProvider,
    );

    // Create asset with valid signature
    let mut asset = Asset::new(
        def,
        100,
        1_000_000,
        &secret,
        nonce,
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("valid asset");

    // Original signature should be valid
    assert!(
        asset.verify_owner_signature().is_ok(),
        "Original signature must be valid"
    );

    // Tamper with range_proof (simulate proof substitution attack)
    let fake_proof = vec![0xFF; 100]; // Fake proof data
    asset.range_proof = Some(fake_proof);

    // Signature MUST become invalid (proof is part of signed message)
    assert!(
        asset.verify_owner_signature().is_err(),
        "Signature MUST be invalid after range_proof tampering - CRITICAL SECURITY BUG if this fails!"
    );
}
#[test]
fn test_asset_tampering_invalidates_signature() {
    // Test that signature covers all critical Asset fields
    // Modification of ANY field should invalidate signature

    let def = create_test_definition();
    let secret = BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    let mut asset = Asset::new(
        def,
        100,
        1_000_000,
        &secret,
        derive_test_nonce(
            &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            &SystemTimeProvider,
        ),
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("valid asset");

    // Original signature valid
    assert!(asset.verify_owner_signature().is_ok());

    // Test 1: Modify amount
    let original_amount = asset.amount;
    asset.amount = 2_000_000;
    assert!(
        asset.verify_owner_signature().is_err(),
        "Modifying amount MUST invalidate signature"
    );
    asset.amount = original_amount; // Restore

    // Test 2: Modify serial_id
    let original_serial = asset.serial_id;
    asset.serial_id = 200;
    assert!(
        asset.verify_owner_signature().is_err(),
        "Modifying serial_id MUST invalidate signature"
    );
    asset.serial_id = original_serial; // Restore

    // Test 3: Modify is_burned flag
    asset.is_burned = true;
    assert!(
        asset.verify_owner_signature().is_err(),
        "Modifying is_burned MUST invalidate signature"
    );
    asset.is_burned = false; // Restore

    // Test 4: Modify is_frozen flag
    asset.is_frozen = true;
    assert!(
        asset.verify_owner_signature().is_err(),
        "Modifying is_frozen MUST invalidate signature"
    );
    asset.is_frozen = false; // Restore

    // Test 5: Modify is_slashed flag
    asset.is_slashed = true;
    assert!(
        asset.verify_owner_signature().is_err(),
        "Modifying is_slashed MUST invalidate signature"
    );
    asset.is_slashed = false; // Restore

    // Test 6: Modify lock_height
    asset.lock_height = Some(1000);
    assert!(
        asset.verify_owner_signature().is_err(),
        "Modifying lock_height MUST invalidate signature"
    );
    asset.lock_height = None; // Restore

    // Final check: restored asset should have valid signature again
    assert!(
        asset.verify_owner_signature().is_ok(),
        "Restored asset signature must be valid"
    );
}

#[test]
fn test_signature_wrong_secret_key() {
    // Test that sign_owner() rejects wrong secret key when owner_pub is already set

    let def = create_test_definition();
    let correct_secret =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let wrong_secret =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([99u8; 32]).rng());

    let asset = Asset::new(
        def,
        100,
        1_000_000,
        &correct_secret,
        derive_test_nonce(
            &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            &SystemTimeProvider,
        ),
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("valid asset");

    // Original signature is valid
    assert!(asset.verify_owner_signature().is_ok());

    // Try to re-sign with WRONG secret key
    let result = asset.sign_owner(
        &wrong_secret,
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    );

    // MUST reject wrong secret key
    assert!(
        result.is_err(),
        "sign_owner() MUST reject wrong secret key when owner_pub is already set"
    );

    // Error message should indicate secret/owner_pub mismatch
    if let Err(e) = result {
        let error_msg = format!("{}", e);
        assert!(
            error_msg.contains("doesn't match") || error_msg.contains("mismatch"),
            "Error message should indicate key mismatch: {}",
            error_msg
        );
    }
}

#[test]
fn test_signature_validates_asset_integrity() {
    // Test that sign_owner() calls validate() to ensure Asset is valid before signing

    let def = Arc::new(
        AssetDefinition::new(
            [0u8; 32],
            AssetClass::Coin,
            "Test".to_string(),
            "TST".to_string(),
            8,
            100, // Only 100 serials
            100_000_000,
            "test.io".to_string(),
            1,
            1,
            0, // Not burnable
            None,
        )
        .unwrap(),
    );

    let secret = BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    // Create valid asset first
    let mut asset = Asset::new(
        def,
        50, // Valid serial_id < 100
        1_000_000,
        &secret,
        derive_test_nonce(
            &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            &SystemTimeProvider,
        ),
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("valid asset");

    // Manually corrupt Asset to invalid state
    asset.serial_id = 150; // Invalid: >= serials (100)
    asset.owner_signature = None; // Remove signature so we can try to re-sign

    // Try to sign invalid Asset - should fail validation
    let result = asset.sign_owner(
        &secret,
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    );

    assert!(
        result.is_err(),
        "sign_owner() MUST reject invalid Asset (serial_id out of range)"
    );
}

#[test]
fn test_owner_pub_correctly_derived() {
    // Verify that owner_pub is correctly derived from blinding factor

    let def = create_test_definition();
    let secret = BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    let asset = Asset::new(
        def,
        100,
        1_000_000,
        &secret,
        derive_test_nonce(
            &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            &SystemTimeProvider,
        ),
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("valid asset");

    // owner_pub must be set
    assert!(
        asset.owner_pub.is_some(),
        "owner_pub must be set automatically"
    );

    // Derive expected public key from secret
    let expected_pub = PublicKey::from_secret_key(&secret);

    // Verify owner_pub matches
    assert_eq!(
        asset.owner_pub.unwrap(),
        expected_pub,
        "owner_pub must be correctly derived from secret key"
    );
}

#[test]
fn test_asset_signature_deterministic() {
    // Test that to_owner_message() is deterministic
    // Same Asset state → same message → same signature (with same nonce)

    let def = create_test_definition();
    let secret = BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let nonce = derive_test_nonce(
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
        &SystemTimeProvider,
    );

    // Create two identical assets
    let asset1 = Asset::new(
        Arc::clone(&def),
        100,
        1_000_000,
        &secret,
        nonce,
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("valid asset");

    let asset2 = Asset::new(
        def,
        100,
        1_000_000,
        &secret,
        nonce,
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("valid asset");

    // Messages should be DIFFERENT because range proofs are non-deterministic
    // Range proof generation uses random nonce internally
    let msg1 = asset1.to_owner_message();
    let msg2 = asset2.to_owner_message();

    // The messages will differ because range_proof bytes are included and are random
    assert_ne!(
        msg1, msg2,
        "to_owner_message() includes non-deterministic range_proof, so messages differ"
    );

    // But the commitment should be the same (deterministic)
    assert_eq!(
        asset1.commitment, asset2.commitment,
        "Commitments should be identical for same inputs"
    );
}

#[test]
fn test_proof_range_signature_changes() {
    // Test that changing range_proof changes the signature message
    // This is the core of the range proof tampering protection

    let def = create_test_definition();
    let secret = BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let nonce = derive_test_nonce(
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
        &SystemTimeProvider,
    );

    let asset = Asset::new(
        def,
        100,
        1_000_000,
        &secret,
        nonce,
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("valid asset");

    let original_message = asset.to_owner_message();

    // Create modified asset with different range proof
    let mut asset_modified = asset.clone();
    asset_modified.range_proof = Some(vec![0xFF; 100]); // Different proof

    let modified_message = asset_modified.to_owner_message();

    // Messages MUST be different
    assert_ne!(
        original_message, modified_message,
        "to_owner_message() MUST change when range_proof changes - CRITICAL for security!"
    );
}

#[test]
fn test_new_enforces_secret_key() {
    // CRITICAL TEST: Verify that Asset::new() enforces secret key validation
    // This validates the fix for race condition where owner_pub must be set BEFORE sign_owner()

    let def = create_test_definition();
    let correct_secret =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    // Create asset - it should set owner_pub BEFORE calling sign_owner()
    let asset = Asset::new(
        def,
        100,
        1_000_000,
        &correct_secret,
        derive_test_nonce(
            &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            &SystemTimeProvider,
        ),
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("valid asset");

    // owner_pub must be set and match the secret
    assert!(asset.owner_pub.is_some());
    let expected_pub = PublicKey::from_secret_key(&correct_secret);
    assert_eq!(asset.owner_pub.as_ref().unwrap(), &expected_pub);

    // Signature must be valid
    assert!(asset.verify_owner_signature().is_ok());

    // Now try to re-sign with DIFFERENT key - should fail
    let wrong_secret =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([99u8; 32]).rng());
    let result = asset.sign_owner(
        &wrong_secret,
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    );

    assert!(
        result.is_err(),
        "After fix, sign_owner() MUST detect secret/owner_pub mismatch"
    );
}
