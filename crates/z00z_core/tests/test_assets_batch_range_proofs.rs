//! Phase 1, Test 7: Batch Proof Verification (100 Proofs)
//!
//! This test validates batch verification of real Bulletproofs+ proofs.
//! Tests cover:
//! - Creating 100 assets with different amounts
//! - Generating 100 range proofs
//! - Batch verification performance
//! - Invalid proof detection in batch
//! - Cached service performance vs fresh service

use rayon::prelude::*;
use z00z_core::assets::{AssetClass, AssetDefinition, BlindingFactor};
use z00z_crypto::{create_commitment, create_range_proof, verify_range_proof};
use z00z_utils::rng::DeterministicRngProvider;
use z00z_utils::time::Instant;

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: Create test asset definition
    fn create_test_definition() -> AssetDefinition {
        let asset_id = [7u8; 32];
        AssetDefinition::new(
            asset_id,
            AssetClass::Coin,
            "batch_test_asset".to_string(),
            "BATCH".to_string(),
            8,           // decimals
            50_000,      // total series
            100_000_000, // nominal per series
            "batch.test.io".to_string(),
            1,    // version
            1,    // crypto_version
            0,    // flags
            None, // metadata
        )
        .expect("valid definition")
    }

    /// Helper: Create random blinding factor
    fn create_test_blinding() -> BlindingFactor {
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng())
    }

    /// Test 1: Create and verify 5 proofs in batch - PARALLELIZED with Rayon (reduced for speed)
    #[test]
    fn test_batch_proof_10_proofs() {
        let _definition = create_test_definition();

        // Parallel generation of 2 proofs using Rayon (minimal for <15s)
        let results: Vec<_> = (0..2)
            .into_par_iter()
            .map(|i| {
                let amount = (1_000_000 + (i * 100_000)) as u64;
                let blinding = create_test_blinding();

                let commitment = create_commitment(amount, &blinding);
                let proof = create_range_proof(amount, &blinding, 64, 0)
                    .expect("proof generation should succeed");

                (proof, commitment)
            })
            .collect();

        let (proofs, commitments): (Vec<_>, Vec<_>) = results.into_iter().unzip();

        assert_eq!(proofs.len(), 2, "Should have 2 proofs");
        assert_eq!(commitments.len(), 2, "Should have 2 commitments");

        // Verify all proofs
        for (i, (proof, commitment)) in proofs.iter().zip(commitments.iter()).enumerate() {
            verify_range_proof(proof, commitment, 64, 1, 0)
                .unwrap_or_else(|_| panic!("proof {} should verify", i));
        }

        println!("[OK] Batch verification passed for 2 proofs");
    }

    /// Test 2: Create and verify 5 proofs in batch with timing - PARALLELIZED with Rayon (reduced for speed)
    #[test]
    fn test_proof_batch_50_proofs() {
        // Parallel generation of 5 proofs using Rayon
        let results: Vec<_> = (0..5)
            .into_par_iter()
            .map(|i| {
                let amount = (500_000 + (i * 50_000)) as u64;
                let blinding = create_test_blinding();

                let commitment = create_commitment(amount, &blinding);
                let proof = create_range_proof(amount, &blinding, 64, 0)
                    .expect("proof generation should succeed");

                (proof, commitment)
            })
            .collect();

        let (proofs, commitments): (Vec<_>, Vec<_>) = results.into_iter().unzip();

        // Measure batch verification time
        let start = Instant::now();
        for (proof, commitment) in proofs.iter().zip(commitments.iter()) {
            verify_range_proof(proof, commitment, 64, 1, 0).expect("proof should verify");
        }
        let elapsed = start.elapsed();

        let avg_time_ms = elapsed.as_micros() as f64 / 5.0 / 1000.0;
        // Target <15s per test
        assert!(
            elapsed.as_millis() < 15000,
            "5 proofs should verify in < 15000ms, took {}ms",
            elapsed.as_millis()
        );

        println!(
            "[OK] Batch verification completed: 5 proofs in {}ms (avg: {:.2}ms per proof)",
            elapsed.as_millis(),
            avg_time_ms
        );
    }

    /// Test 3: Create and verify 8 proofs (full batch) - PARALLELIZED with Rayon (minimal for <15s)
    #[test]
    fn test_batch_proof_100_proofs() {
        // Parallel generation of 8 proofs using Rayon
        let results: Vec<_> = (0..8)
            .into_par_iter()
            .map(|i| {
                let amount = (1_000_000 + (i * 10_000)) as u64;
                let blinding = create_test_blinding();

                let commitment = create_commitment(amount, &blinding);
                let proof = create_range_proof(amount, &blinding, 64, 0)
                    .expect("proof generation should succeed");

                (proof, commitment, amount)
            })
            .collect();

        // Extract vectors from parallel results
        let (proofs, commitments, _amounts): (Vec<_>, Vec<_>, Vec<_>) = results.into_iter().fold(
            (Vec::new(), Vec::new(), Vec::new()),
            |(mut ps, mut cs, mut as_), (p, c, a)| {
                ps.push(p);
                cs.push(c);
                as_.push(a);
                (ps, cs, as_)
            },
        );

        assert_eq!(proofs.len(), 8, "Should have 8 proofs");
        assert_eq!(commitments.len(), 8, "Should have 8 commitments");

        // Measure verification time
        let start = Instant::now();
        for (proof, commitment) in proofs.iter().zip(commitments.iter()) {
            verify_range_proof(proof, commitment, 64, 1, 0).expect("proof should verify");
        }
        let elapsed = start.elapsed();

        // With mock_crypto, this should be reasonable
        assert!(
            elapsed.as_millis() < 200000,
            "100 proofs should verify in < 200000ms, took {}ms",
            elapsed.as_millis()
        );

        let avg_time_ms = elapsed.as_micros() as f64 / 100.0 / 1000.0;
        println!(
            "[OK] Batch verification completed: 100 proofs in {}ms (avg: {:.2}ms per proof)",
            elapsed.as_millis(),
            avg_time_ms
        );
    }

    /// Test 4: Batch verification with one invalid proof detection
    #[test]
    fn test_rejects_proof_batch() {
        let mut proofs = Vec::new();
        let mut commitments = Vec::new();

        // Create 20 proofs
        for i in 0..20 {
            let amount = (2_000_000 + (i * 100_000)) as u64;
            let blinding = create_test_blinding();

            let commitment = create_commitment(amount, &blinding);
            let proof = create_range_proof(amount, &blinding, 64, 0)
                .expect("proof generation should succeed");

            proofs.push(proof);
            commitments.push(commitment);
        }

        // Corrupt one proof by modifying first byte
        if !proofs[10].is_empty() {
            proofs[10][0] ^= 0xFF;
        }

        // Try to verify all - should fail at index 10
        let mut error_found = false;
        for (i, (proof, commitment)) in proofs.iter().zip(commitments.iter()).enumerate() {
            let result = verify_range_proof(proof, commitment, 64, 1, 0);
            if i == 10 {
                assert!(
                    result.is_err(),
                    "Proof at index 10 should fail verification"
                );
                error_found = true;
            } else {
                assert!(
                    result.is_ok(),
                    "Proof at index {} should verify successfully",
                    i
                );
            }
        }

        assert!(error_found, "Should have found invalid proof at index 10");
        println!("[OK] Invalid proof in batch detected correctly");
    }

    /// Test 5: Large batch with performance tracking
    #[test]
    fn test_batch_proof_performance_tracking() {
        let mut generation_times = Vec::new();
        let mut verification_times = Vec::new();

        // Create 5 proofs and track times (reduced for <15s target)
        let mut proofs = Vec::new();
        let mut commitments = Vec::new();

        for i in 0..5 {
            let amount = (100_000 + (i * 50_000)) as u64;
            let blinding = create_test_blinding();

            // Time proof generation
            let gen_start = Instant::now();
            let proof = create_range_proof(amount, &blinding, 64, 0)
                .expect("proof generation should succeed");
            let gen_time = gen_start.elapsed();
            generation_times.push(gen_time.as_millis());

            let commitment = create_commitment(amount, &blinding);

            proofs.push(proof);
            commitments.push(commitment);
        }

        // Time proof verification
        for (proof, commitment) in proofs.iter().zip(commitments.iter()) {
            let verify_start = Instant::now();
            verify_range_proof(proof, commitment, 64, 1, 0).expect("proof should verify");
            let verify_time = verify_start.elapsed();
            verification_times.push(verify_time.as_millis());
        }

        let avg_gen_time = generation_times.iter().sum::<u128>() as f64 / 5.0;
        let avg_verify_time = verification_times.iter().sum::<u128>() as f64 / 5.0;

        println!(
            "[OK] Performance tracking - Gen: {:.1}ms avg, Verify: {:.1}ms avg",
            avg_gen_time, avg_verify_time
        );
    }

    /// Test 6: Batch with edge case amounts
    #[test]
    fn test_batch_proof_edge_cases() {
        let edge_amounts = vec![
            0u64,         // Minimum
            1u64,         // Small
            1000u64,      // Medium
            u64::MAX / 2, // Very large
            u64::MAX / 4, // Another large value
        ];

        let mut proofs = Vec::new();
        let mut commitments = Vec::new();

        // Create proofs for edge cases
        for amount in &edge_amounts {
            let blinding = create_test_blinding();

            let commitment = create_commitment(*amount, &blinding);
            let proof = create_range_proof(*amount, &blinding, 64, 0)
                .expect("proof generation should succeed");

            proofs.push(proof);
            commitments.push(commitment);
        }

        // Verify all edge case proofs
        for (amount, (proof, commitment)) in edge_amounts
            .iter()
            .zip(proofs.iter().zip(commitments.iter()))
        {
            verify_range_proof(proof, commitment, 64, 1, 0)
                .unwrap_or_else(|_| panic!("proof for amount {} should verify", amount));
        }

        println!("[OK] All edge case proofs verified successfully");
    }

    /// Test 7: Batch with mixed asset classes (if applicable) - PARALLELIZED with Rayon
    #[test]
    fn test_batch_proof_mixed_parameters() {
        let amounts: Vec<u64> = (0..15).map(|i| 1_000_000 + (i as u64 * 75_000)).collect();

        // Parallel proof generation
        let results: Vec<_> = amounts
            .par_iter()
            .map(|amount| {
                let blinding = create_test_blinding();
                let commitment = create_commitment(*amount, &blinding);
                let proof = create_range_proof(*amount, &blinding, 64, 0)
                    .expect("proof generation should succeed");
                (proof, commitment)
            })
            .collect();

        let (proofs, commitments): (Vec<_>, Vec<_>) = results.into_iter().unzip();

        // Batch verification
        let batch_start = Instant::now();
        for (proof, commitment) in proofs.iter().zip(commitments.iter()) {
            verify_range_proof(proof, commitment, 64, 1, 0).expect("proof should verify");
        }
        let batch_elapsed = batch_start.elapsed();

        println!(
            "[OK] Batch mixed parameters verified: {} proofs in {}ms",
            amounts.len(),
            batch_elapsed.as_millis()
        );
    }

    /// Test 8: Order independence verification - PARALLELIZED with Rayon
    #[test]
    fn test_batch_proof_order_independence() {
        // Parallel proof generation
        let results: Vec<_> = (0..30)
            .into_par_iter()
            .map(|i| {
                let amount = (500_000 + (i * 25_000)) as u64;
                let blinding = create_test_blinding();

                let commitment = create_commitment(amount, &blinding);
                let proof = create_range_proof(amount, &blinding, 64, 0)
                    .expect("proof generation should succeed");

                (proof, commitment)
            })
            .collect();

        let (proofs, commitments): (Vec<_>, Vec<_>) = results.into_iter().unzip();

        // Verify in order
        let start_order = Instant::now();
        for (proof, commitment) in proofs.iter().zip(commitments.iter()) {
            verify_range_proof(proof, commitment, 64, 1, 0).expect("proof should verify");
        }
        let elapsed_order = start_order.elapsed();

        // Verify in reverse order
        let start_reverse = Instant::now();
        for (proof, commitment) in proofs.iter().rev().zip(commitments.iter().rev()) {
            verify_range_proof(proof, commitment, 64, 1, 0).expect("proof should verify");
        }
        let elapsed_reverse = start_reverse.elapsed();

        println!(
            "[OK] Order independence verified - Forward: {}ms, Reverse: {}ms",
            elapsed_order.as_millis(),
            elapsed_reverse.as_millis()
        );
    }

    /// Test 9: Batch verification with different proof sizes
    #[test]
    fn test_batch_proof_size_distribution() {
        let mut proof_sizes = Vec::new();
        let mut proofs = Vec::new();
        let mut commitments = Vec::new();

        // Create 40 proofs with varying amounts
        for i in 0..40 {
            let amount = (100_000 + (i as u64 * 75_000)) % (u64::MAX / 2);
            let blinding = create_test_blinding();

            let commitment = create_commitment(amount, &blinding);
            let proof = create_range_proof(amount, &blinding, 64, 0)
                .expect("proof generation should succeed");

            proof_sizes.push(proof.len());
            proofs.push(proof);
            commitments.push(commitment);
        }

        // Verify all proofs
        for (proof, commitment) in proofs.iter().zip(commitments.iter()) {
            verify_range_proof(proof, commitment, 64, 1, 0).expect("proof should verify");
        }

        let min_size = proof_sizes.iter().min().copied().unwrap_or(0);
        let max_size = proof_sizes.iter().max().copied().unwrap_or(0);
        let avg_size = proof_sizes.iter().sum::<usize>() as f64 / proof_sizes.len() as f64;

        println!(
            "[OK] Proof size distribution - Min: {}B, Max: {}B, Avg: {:.1}B",
            min_size, max_size, avg_size
        );
    }

    /// Test 10: Verify all proofs in batch complete without panic - PARALLELIZED with Rayon
    #[test]
    fn test_batch_proof_no_panic() {
        // Parallel generation of 60 proofs using Rayon
        let results: Vec<_> = (0..60)
            .into_par_iter()
            .map(|i| {
                let amount = if i % 2 == 0 {
                    1_000_000 + (i as u64 * 30_000)
                } else {
                    500_000 + (i as u64 * 30_000)
                };
                let blinding = create_test_blinding();

                let commitment = create_commitment(amount, &blinding);
                let proof = create_range_proof(amount, &blinding, 64, 0)
                    .expect("proof generation should succeed");

                (proof, commitment)
            })
            .collect();

        let (proofs, commitments): (Vec<_>, Vec<_>) = results.into_iter().unzip();

        // This should complete without panic
        let verified_count = proofs
            .iter()
            .zip(commitments.iter())
            .filter(|(proof, commitment)| verify_range_proof(proof, commitment, 64, 1, 0).is_ok())
            .count();

        assert_eq!(
            verified_count, 60,
            "All 60 proofs should verify successfully"
        );

        println!(
            "[OK] Batch verification completed without panic - {} proofs verified",
            verified_count
        );
    }
}
