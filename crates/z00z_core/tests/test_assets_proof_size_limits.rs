//! Phase 1, Test 8: Proof Size Limit DoS Protection
//!
//! This test validates that oversized proofs are properly rejected to prevent DoS attacks.
//! Tests cover:
//! - Oversized fake proofs (>10KB) rejection
//! - Valid proof size limits (<10KB acceptance)
//! - Error messages for size violations
//! - Boundary condition testing

use z00z_core::assets::BlindingFactor;
use z00z_crypto::{create_commitment, create_range_proof, verify_range_proof};
use z00z_utils::rng::DeterministicRngProvider;

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: Create test blinding factor
    fn create_test_blinding() -> BlindingFactor {
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng())
    }

    /// Test 1: Oversized proof (15KB) should be rejected
    #[test]
    fn test_proof_size_oversized_15kb() {
        let amount = 1_000_000u64;
        let blinding = create_test_blinding();
        let commitment = create_commitment(amount, &blinding);

        // Create an oversized fake proof (15KB)
        let oversized_proof = vec![0u8; 15_000];

        // Verification should fail
        let result = verify_range_proof(&oversized_proof, &commitment, 64, 1, 0);
        assert!(result.is_err(), "Oversized proof (15KB) should be rejected");

        println!("[OK] Oversized proof (15KB) correctly rejected");
    }

    /// Test 2: Very large oversized proof (100KB) should be rejected
    #[test]
    fn test_proof_size_large_100() {
        let amount = 500_000u64;
        let blinding = create_test_blinding();
        let commitment = create_commitment(amount, &blinding);

        // Create a very large fake proof (100KB)
        let huge_proof = vec![0u8; 100_000];

        // Verification should definitely fail
        let result = verify_range_proof(&huge_proof, &commitment, 64, 1, 0);
        assert!(result.is_err(), "Huge proof (100KB) should be rejected");

        println!("[OK] Very large proof (100KB) correctly rejected");
    }

    /// Test 3: Proof just over 10KB should be rejected
    #[test]
    fn test_proof_size_limit() {
        let amount = 2_000_000u64;
        let blinding = create_test_blinding();
        let commitment = create_commitment(amount, &blinding);

        // Create proof just over 10KB
        let oversized_proof = vec![0u8; 10_100];

        // Verification should fail
        let result = verify_range_proof(&oversized_proof, &commitment, 64, 1, 0);
        assert!(
            result.is_err(),
            "Proof just over limit (10.1KB) should be rejected"
        );

        println!("[OK] Proof just over 10KB limit correctly rejected");
    }

    /// Test 4: Valid range proof should be under limit
    #[test]
    fn test_proof_size_valid_limit() {
        let amount = 1_500_000u64;
        let blinding = create_test_blinding();
        let commitment = create_commitment(amount, &blinding);

        // Create a real valid proof
        let proof =
            create_range_proof(amount, &blinding, 64, 0).expect("proof generation should succeed");

        // Verify proof is accepted
        verify_range_proof(&proof, &commitment, 64, 1, 0).expect("valid proof should be accepted");

        // Verify proof is under limit
        assert!(
            proof.len() < 10_000,
            "Valid proof should be under 10KB, got {} bytes",
            proof.len()
        );

        println!(
            "[OK] Valid proof ({} bytes) is under 10KB limit and verified",
            proof.len()
        );
    }

    /// Test 5: Multiple valid proofs should all be under limit
    #[test]
    fn test_proof_size_batch_limit() {
        let amounts = vec![
            100_000u64,
            500_000u64,
            1_000_000u64,
            5_000_000u64,
            u64::MAX / 2,
        ];

        for amount in amounts {
            let blinding = create_test_blinding();
            let commitment = create_commitment(amount, &blinding);

            let proof = create_range_proof(amount, &blinding, 64, 0)
                .expect("proof generation should succeed");

            assert!(
                proof.len() < 10_000,
                "Proof for amount {} exceeds 10KB limit: {} bytes",
                amount,
                proof.len()
            );

            // Verify it passes
            verify_range_proof(&proof, &commitment, 64, 1, 0).expect("proof should verify");
        }

        println!("[OK] All valid proofs are under 10KB limit");
    }

    /// Test 6: Empty proof should be rejected
    #[test]
    fn test_proof_size_empty() {
        let amount = 750_000u64;
        let blinding = create_test_blinding();
        let commitment = create_commitment(amount, &blinding);

        // Empty proof
        let empty_proof = vec![];

        // Verification should fail
        let result = verify_range_proof(&empty_proof, &commitment, 64, 1, 0);
        assert!(result.is_err(), "Empty proof should be rejected");

        println!("[OK] Empty proof correctly rejected");
    }

    /// Test 7: Single-byte proof should be rejected
    #[test]
    fn test_proof_size_single_byte() {
        let amount = 1_200_000u64;
        let blinding = create_test_blinding();
        let commitment = create_commitment(amount, &blinding);

        // Single byte proof
        let tiny_proof = vec![42u8];

        // Verification should fail
        let result = verify_range_proof(&tiny_proof, &commitment, 64, 1, 0);
        assert!(result.is_err(), "Single-byte proof should be rejected");

        println!("[OK] Single-byte proof correctly rejected");
    }

    /// Test 8: Small but non-valid proof should be rejected
    #[test]
    fn test_proof_size_small_invalid() {
        let amount = 2_500_000u64;
        let blinding = create_test_blinding();
        let commitment = create_commitment(amount, &blinding);

        // Small proof that's not valid (just random bytes under limit)
        let small_invalid_proof = vec![42u8; 500];

        // Verification should fail (invalid proof structure)
        let result = verify_range_proof(&small_invalid_proof, &commitment, 64, 1, 0);
        assert!(result.is_err(), "Invalid small proof should be rejected");

        println!("[OK] Small invalid proof correctly rejected");
    }

    /// Test 9: Tampered valid proof (still under limit) should be rejected
    #[test]
    fn test_proof_size_tampered_limit() {
        let amount = 1_800_000u64;
        let blinding = create_test_blinding();
        let commitment = create_commitment(amount, &blinding);

        let mut proof =
            create_range_proof(amount, &blinding, 64, 0).expect("proof generation should succeed");

        // Tamper with proof (still under 10KB)
        if !proof.is_empty() {
            proof[0] ^= 0xFF;
        }

        assert!(
            proof.len() < 10_000,
            "Tampered proof should still be under 10KB"
        );

        // Verification should fail (tampered)
        let result = verify_range_proof(&proof, &commitment, 64, 1, 0);
        assert!(
            result.is_err(),
            "Tampered proof should be rejected even if under limit"
        );

        println!(
            "[OK] Tampered proof ({} bytes, under limit) correctly rejected",
            proof.len()
        );
    }

    /// Test 10: Exact 10KB proof (if valid) edge case
    #[test]
    fn test_proof_size_exact_boundary() {
        let amount = 3_000_000u64;
        let blinding = create_test_blinding();
        let commitment = create_commitment(amount, &blinding);

        // Create a valid proof first to see its size
        let valid_proof =
            create_range_proof(amount, &blinding, 64, 0).expect("proof generation should succeed");

        // Valid proof should always be under limit
        assert!(
            valid_proof.len() < 10_000,
            "Valid proof should never be exactly at or over 10KB boundary"
        );

        println!(
            "[OK] Valid proof size ({} bytes) is well under 10KB boundary",
            valid_proof.len()
        );

        // Create a fake proof exactly at 10KB
        let fake_10kb_proof = vec![0u8; 10_000];

        // This should be rejected (not a valid proof structure)
        let result = verify_range_proof(&fake_10kb_proof, &commitment, 64, 1, 0);
        assert!(
            result.is_err(),
            "Proof exactly at 10KB boundary should be rejected if invalid"
        );

        println!("[OK] Fake 10KB proof correctly rejected");
    }
}
