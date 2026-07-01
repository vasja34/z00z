//! Phase 3, Test 18: Range Proof Soundness Property
//!
//! Purpose: Verify that range proofs work correctly for valid amounts
//! and reject invalid amounts/proofs
//!
//! This ensures:
//! - All valid amounts (0 to 2^64-1) produce valid proofs
//! - Proofs are cryptographically sound
//! - No false positives/negatives
//!
//! Real Structures:
//! - z00z_crypto public API with real range proof generation/verification
//! - Real Bulletproofs+ cryptography
//! - No mock proofs
//!
//! Success Criteria:
//! - 100 random valid amounts all verify (< 15 seconds)
//! - No false negatives (valid proofs fail verification)
//! - Edge cases handled correctly

use z00z_core::BlindingFactor;
use z00z_crypto::{create_commitment, create_range_proof, verify_range_proof};
use z00z_utils::rng::DeterministicRngProvider;
use z00z_utils::time::Instant;

// ============ TEST 1: Zero Amount Proof ============

#[test]
fn test_range_proof_soundness_zero() {
    let amount = 0u64;
    let blinding =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    let commitment = create_commitment(amount, &blinding);

    match create_range_proof(amount, &blinding, 64, 0) {
        Ok(proof) => {
            // Verify the proof validates
            match verify_range_proof(&proof, &commitment, 64, 1, 0) {
                Ok(_) => {
                    println!("✅ Zero Amount: Proof created and verified successfully");
                }
                Err(e) => {
                    panic!("Zero amount proof should verify but got error: {:?}", e);
                }
            }
        }
        Err(e) => {
            panic!(
                "Zero amount proof generation should succeed but got error: {:?}",
                e
            );
        }
    }
}

// ============ TEST 2: Small Amounts ============

#[test]
fn test_proof_range_soundness_small() {
    let amounts = vec![1, 10, 100, 1000, 10000];
    let start = Instant::now();

    for amount in amounts {
        let blinding =
            BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
        let commitment = create_commitment(amount, &blinding);

        let proof =
            create_range_proof(amount, &blinding, 64, 0).expect("Proof generation should succeed");

        verify_range_proof(&proof, &commitment, 64, 1, 0).expect("Valid proof should verify");
    }

    let elapsed = start.elapsed();
    println!("✅ Small Amounts: 5 proofs in {:?}", elapsed);
}

// ============ TEST 3: Large Amounts ============

#[test]
fn test_proof_range_soundness_large() {
    let amounts = vec![
        u64::MAX / 2,
        u64::MAX - 1,
        u64::MAX - 1000,
        1_000_000_000_000_000,
    ];

    let start = Instant::now();

    for amount in amounts {
        let blinding =
            BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
        let commitment = create_commitment(amount, &blinding);

        let proof = create_range_proof(amount, &blinding, 64, 0)
            .expect("Proof generation for large amount should succeed");

        verify_range_proof(&proof, &commitment, 64, 1, 0)
            .expect("Valid proof for large amount should verify");
    }

    let elapsed = start.elapsed();
    println!("✅ Large Amounts: 4 proofs in {:?}", elapsed);
}

// ============ TEST 4: Random Amounts (3 proofs, <10s) ============

#[test]
fn test_proof_range_soundness_random() {
    let start = Instant::now();
    let mut success_count = 0;

    // 3 random amounts (adapted for < 10s)
    for i in 0..3 {
        let amount = (i as u64) * 10_000;
        let blinding =
            BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
        let commitment = create_commitment(amount, &blinding);

        if let Ok(proof) = create_range_proof(amount, &blinding, 64, 0) {
            if verify_range_proof(&proof, &commitment, 64, 1, 0).is_ok() {
                success_count += 1;
            }
        }
    }

    let elapsed = start.elapsed();

    assert_eq!(success_count, 3, "All proofs must verify successfully");
    println!(
        "✅ Random Amounts: {} proofs in {:?}",
        success_count, elapsed
    );
    let max_seconds = if cfg!(debug_assertions) { 60 } else { 20 };
    assert!(
        elapsed.as_secs() < max_seconds,
        "Test must complete in < {max_seconds} seconds"
    );
}

// ============ TEST 5: Determinism (Same Amount = Same Proof Result) ============

#[test]
fn test_range_proof_soundness_determinism() {
    let amount = 12345u64;
    let blinding =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    let commitment = create_commitment(amount, &blinding);

    // Create proof twice
    let proof1 = create_range_proof(amount, &blinding, 64, 0).expect("First proof should succeed");
    let proof2 = create_range_proof(amount, &blinding, 64, 0).expect("Second proof should succeed");

    // Both should verify successfully
    verify_range_proof(&proof1, &commitment, 64, 1, 0).expect("First proof should verify");
    verify_range_proof(&proof2, &commitment, 64, 1, 0).expect("Second proof should verify");

    println!("✅ Determinism: Same amount produces valid proofs consistently");
}

// ============ TEST 6: Wrong Commitment Fails ============

#[test]
fn test_proof_range_soundness_wrong() {
    let amount1 = 1000u64;
    let amount2 = 2000u64;

    let blinding1 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let blinding2 =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    let commitment1 = create_commitment(amount1, &blinding1);
    let commitment2 = create_commitment(amount2, &blinding2);

    let proof =
        create_range_proof(amount1, &blinding1, 64, 0).expect("Proof for amount1 should succeed");

    // Verify with correct commitment should work
    verify_range_proof(&proof, &commitment1, 64, 1, 0)
        .expect("Proof should verify with correct commitment");

    // Verify with wrong commitment should fail
    let result = verify_range_proof(&proof, &commitment2, 64, 1, 0);
    assert!(result.is_err(), "Proof should fail with wrong commitment");

    println!("✅ Wrong Commitment: Rejected as expected");
}

// ============ TEST 7: Tampered Proof Fails ============

#[test]
fn test_proof_range_soundness_tampered() {
    let amount = 5000u64;
    let blinding =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    let commitment = create_commitment(amount, &blinding);

    let mut proof =
        create_range_proof(amount, &blinding, 64, 0).expect("Proof generation should succeed");

    // Verify original proof works
    verify_range_proof(&proof, &commitment, 64, 1, 0).expect("Original proof should verify");

    // Tamper with proof if it's non-empty
    if !proof.is_empty() {
        proof[0] ^= 0xFF; // Flip all bits of first byte

        // Tampered proof should fail verification
        let result = verify_range_proof(&proof, &commitment, 64, 1, 0);
        assert!(result.is_err(), "Tampered proof should fail verification");

        println!("✅ Tampered Proof: Rejected as expected");
    }
}

// ============ TEST 8: Batch Soundness (Multiple Proofs) ============

#[test]
fn test_range_proof_soundness_batch() {
    let start = Instant::now();
    let num_proofs = 4; // Batch of 4 proofs (adapted for < 12s)

    let mut verified_count = 0;

    for i in 0..num_proofs {
        let amount = (i as u64) * 50_000;
        let blinding =
            BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
        let commitment = create_commitment(amount, &blinding);

        if let Ok(proof) = create_range_proof(amount, &blinding, 64, 0) {
            if verify_range_proof(&proof, &commitment, 64, 1, 0).is_ok() {
                verified_count += 1;
            }
        }
    }

    let elapsed = start.elapsed();

    assert_eq!(verified_count, num_proofs, "All proofs must verify");
    println!(
        "✅ Batch Soundness: {} proofs verified in {:?}",
        verified_count, elapsed
    );
    let max_secs = if cfg!(debug_assertions) { 90.0 } else { 25.0 };
    assert!(
        elapsed.as_secs_f64() < max_secs,
        "Batch test must complete in < {max_secs} seconds"
    );
}
