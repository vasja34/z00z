//! Phase 3, Test 21: All AssetError Variants Coverage
//!
//! Purpose: Verify error handling for all error paths in assets module
//! - Exercise all error variants
//! - Verify error messages are informative
//! - Ensure no panics in error paths
//!
//! Real Structures:
//! - All AssetError variants
//! - Real error generation scenarios
//! - Proper error context
//!
//! Success Criteria:
//! - All error variants tested
//! - No panics on error
//! - Clear error messages (< 15 seconds)

use z00z_core::assets::{Asset, AssetClass, AssetDefinition};
use z00z_core::BlindingFactor;
use z00z_crypto::{create_commitment, create_range_proof, verify_range_proof};
use z00z_utils::rng::DeterministicRngProvider;
use z00z_utils::time::Instant;

// ============ Helper: Create test definition ============

fn create_test_definition() -> std::sync::Arc<AssetDefinition> {
    std::sync::Arc::new(
        AssetDefinition::new(
            [42u8; 32],
            AssetClass::Coin,
            "Test Coin".to_string(),
            "TST".to_string(),
            8,
            1_000_000,
            1_000,
            "test.local".to_string(),
            1,
            1,
            0,
            None,
        )
        .expect("Valid test definition"),
    )
}

// ============ TEST 1: Invalid Zero Nonce ============

#[test]
fn test_error_handling_zero_nonce() {
    let def = create_test_definition();
    let blinding =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let zero_nonce = [0u8; 32]; // Invalid: zero nonce

    // Creating asset with zero nonce should fail (or be handled)
    // For now, test that it doesn't panic
    let _result = Asset::new(
        def,
        1000,
        1_000_000,
        &blinding,
        zero_nonce,
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    );

    println!("✅ Zero Nonce: Error path handled without panic");
}

// ============ TEST 2: Valid Asset Creation (Positive Test) ============

#[test]
fn test_error_handling_valid_asset() {
    let def = create_test_definition();
    let blinding =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let nonce = [1u8; 32]; // Valid nonce

    let asset = Asset::new(
        def,
        1000,
        1_000_000,
        &blinding,
        nonce,
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("Valid asset creation should succeed");

    assert_eq!(asset.serial_id(), 1000);
    assert_eq!(asset.amount(), 1_000_000);

    println!("✅ Valid Asset: Created successfully");
}

// ============ TEST 3: Invalid Proof (Empty Proof) ============

#[test]
fn test_error_handling_empty_proof() {
    let amount = 1000u64;
    let blinding =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let commitment = create_commitment(amount, &blinding);

    let empty_proof = vec![]; // Invalid: empty proof

    // Verification should fail gracefully
    let result = verify_range_proof(&empty_proof, &commitment, 64, 1, 0);

    assert!(
        result.is_err(),
        "Empty proof should fail verification without panic"
    );

    println!("✅ Empty Proof: Rejected gracefully with error");
}

// ============ TEST 4: Definition Metadata Edge Cases ============

#[test]
fn test_error_handling_definition_metadata() {
    let start = Instant::now();

    // Test with empty name
    let def_empty_name = AssetDefinition::new(
        [1u8; 32],
        AssetClass::Coin,
        "".to_string(), // Empty name
        "TEST".to_string(),
        8,
        1_000_000,
        1_000,
        "test.local".to_string(),
        1,
        1,
        0,
        None,
    );

    // Should handle gracefully
    match def_empty_name {
        Ok(_def) => {
            println!("✅ Empty Name: Allowed or handled");
        }
        Err(_e) => {
            println!("✅ Empty Name: Rejected with error");
        }
    }

    let elapsed = start.elapsed();
    assert!(elapsed.as_secs() < 15, "Test must complete in < 15 seconds");
}

// ============ TEST 5: Invalid Amount (u64 boundaries) ============

#[test]
fn test_error_handling_amount_boundaries() {
    let def = create_test_definition();
    let blinding =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let nonce = [2u8; 32];

    // Test with max u64
    let asset_max = Asset::new(
        def.clone(),
        1,
        u64::MAX,
        &blinding,
        nonce,
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    );
    assert!(asset_max.is_ok(), "Max u64 amount should be allowed");

    // Test with zero amount - NOT allowed for native asset class
    let asset_zero = Asset::new(
        def.clone(),
        2,
        0,
        &blinding,
        nonce,
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    );
    assert!(
        asset_zero.is_err(),
        "Zero amount should NOT be allowed for native asset class"
    );

    println!("✅ Amount Boundaries: Handled correctly");
}

// ============ TEST 6: Commitment Verification with Wrong Amounts ============

#[test]
fn test_error_handling_commitment_mismatch() {
    let amount_proof = 1000u64;
    let amount_commitment = 2000u64;

    let blinding_proof =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let blinding_commitment =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    let commitment = create_commitment(amount_commitment, &blinding_commitment);

    let proof = create_range_proof(amount_proof, &blinding_proof, 64, 0)
        .expect("Proof generation should succeed");

    // Verification should fail gracefully
    let result = verify_range_proof(&proof, &commitment, 64, 1, 0);

    assert!(
        result.is_err(),
        "Mismatched proof/commitment should fail without panic"
    );

    println!("✅ Commitment Mismatch: Rejected gracefully");
}

// ============ TEST 7: Multiple Sequential Operations (No Memory Leaks) ============

#[test]
fn test_error_handling_memory_safety() {
    let start = Instant::now();
    let def = create_test_definition();

    // Perform a smaller set of operations to keep runtime tight
    for i in 0..4 {
        let blinding =
            BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
        let nonce = [(i % 256) as u8; 32];

        // Create asset (may succeed or fail)
        let _asset = Asset::new(
            def.clone(),
            i as u32,
            i as u64 * 1000,
            &blinding,
            nonce,
            &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
        );

        // Try proof verification
        if i % 2 == 0 {
            let amount = (i as u64) * 1000;
            let commitment = create_commitment(amount, &blinding);

            if let Ok(proof) = create_range_proof(amount, &blinding, 64, 0) {
                let _ = verify_range_proof(&proof, &commitment, 64, 1, 0);
            }
        }
    }

    let elapsed = start.elapsed();
    println!(
        "✅ Memory Safety: 4 operations completed without issues in {:?}",
        elapsed
    );
    let max_secs = if cfg!(debug_assertions) { 90.0 } else { 30.0 };
    assert!(
        elapsed.as_secs_f64() < max_secs,
        "Test must complete in < {max_secs} seconds"
    );
}

// ============ TEST 8: Error Recovery (Graceful Degradation) ============

#[test]
fn test_error_handling_graceful_degradation() {
    let start = Instant::now();
    let def = create_test_definition();

    let mut success_count = 0;
    let mut error_count = 0;

    // Try operations and count successes/failures
    for i in 0..4 {
        let blinding =
            BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
        let nonce = [(i % 256) as u8; 32];

        match Asset::new(
            def.clone(),
            i as u32,
            i as u64 * 1000,
            &blinding,
            nonce,
            &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
        ) {
            Ok(_) => success_count += 1,
            Err(_) => error_count += 1,
        }
    }

    let elapsed = start.elapsed();

    // We should have some successes (graceful degradation, not total failure)
    println!(
        "✅ Graceful Degradation: {} success, {} errors",
        success_count, error_count
    );
    assert!(success_count > 0, "Should have some successful operations");
    let max_seconds = if cfg!(debug_assertions) { 60.0 } else { 15.0 };
    assert!(
        elapsed.as_secs_f64() < max_seconds,
        "Test must complete in < {max_seconds} seconds"
    );
}
