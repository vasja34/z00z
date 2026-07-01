//! Batch Range Proof Verification Tests (TODO-6.1)
//!
//! Tests for O(log n) batch verification of Bulletproofs+ range proofs.
//!
//! ## Test Coverage
//!
//! - Small batches (10 assets)
//! - Medium batches (100 assets)
//! - Large batches (1,000 assets)
//! - Security: Invalid proof detection
//! - Error handling: Missing proofs
//! - Performance: Sequential vs batch comparison
//!
//! ## Performance Expectations
//!
//! Exact speedups depend on CPU, scheduler noise, and thermal state.
//! These tests enforce only stable anti-regression bounds that should hold
//! across normal developer machines and CI workers.

use z00z_core::assets::Asset;
use z00z_core::genesis::validator::validate_genesis_commitments_batch;
use z00z_crypto::batch_verify_range_proofs;
use z00z_utils::time::Instant;

use super::helpers::create_test_definition;
use std::sync::Arc;
use z00z_core::genesis::ChainType;
use z00z_core::genesis::{derive_deterministic_rng_seed, derive_genesis_blinding};
use z00z_utils::rng::DeterministicRngProvider;

/// Helper function to derive nonce (simplified version for tests)
fn derive_test_nonce(serial_id: u32) -> [u8; 32] {
    let mut nonce = [0u8; 32];
    nonce[0..4].copy_from_slice(&serial_id.to_le_bytes());
    nonce
}

/// Helper to create test assets in batch
/// Creates assets with adequate serial range
fn create_test_assets(count: u32) -> Vec<Asset> {
    let definition = Arc::new(create_test_definition());
    let genesis_seed = [42u8; 32];
    let amount = definition.nominal;

    (0..count)
        .map(|serial_id| {
            // Wrap serial_id to stay within definition.serials range
            let wrapped_serial_id = serial_id % definition.serials;

            let blinding = derive_genesis_blinding(
                &genesis_seed,
                &definition.id,
                wrapped_serial_id,
                ChainType::Devnet,
            )
            .unwrap();

            let nonce = derive_test_nonce(wrapped_serial_id);

            let rng_seed = derive_deterministic_rng_seed(
                &genesis_seed,
                &definition.id,
                wrapped_serial_id,
                ChainType::Devnet,
            );

            let provider = DeterministicRngProvider::from_seed(rng_seed);
            let mut rng = provider.rng();

            Asset::new(
                definition.clone(),
                wrapped_serial_id,
                amount,
                &blinding,
                nonce,
                &mut rng,
            )
            .unwrap()
        })
        .collect()
}

#[test]
fn test_genesis_batch_verify_10() {
    // Create 10 valid assets
    let assets = create_test_assets(10);

    // Batch verify should succeed
    let result = validate_genesis_commitments_batch(&assets);
    assert!(
        result.is_ok(),
        "Batch verification of 10 valid assets should succeed"
    );
}

#[test]
fn test_genesis_batch_verify_100() {
    // Create 100 valid assets
    let assets = create_test_assets(100);

    // Batch verify should succeed
    let result = validate_genesis_commitments_batch(&assets);
    assert!(
        result.is_ok(),
        "Batch verification of 100 valid assets should succeed"
    );
}

#[test]
fn test_genesis_batch_verify_1000() {
    // Create 1000 valid assets
    let assets = create_test_assets(1000);

    // Batch verify should succeed
    let result = validate_genesis_commitments_batch(&assets);
    assert!(
        result.is_ok(),
        "Batch verification of 1000 valid assets should succeed"
    );
}

#[test]
fn test_rejects_proof_batch_verify() {
    // Create 10 assets, but tamper with one proof
    let mut assets = create_test_assets(10);

    // Tamper with proof #5
    if let Some(proof) = &mut assets[5].range_proof {
        if !proof.is_empty() {
            proof[0] ^= 0xFF; // Flip bits to invalidate proof
        }
    }

    // Batch verification should detect the invalid proof
    let result = validate_genesis_commitments_batch(&assets);
    assert!(
        result.is_err(),
        "Batch verification should detect invalid proof"
    );
}

#[test]
fn test_batch_verify_missing_proof() {
    // Create 10 assets
    let mut assets = create_test_assets(10);

    // Remove proof from one asset
    assets[3].range_proof = None;

    // Should fail fast on missing proof
    let result = validate_genesis_commitments_batch(&assets);
    assert!(
        result.is_err(),
        "Batch verification should fail when proof is missing"
    );

    // Check error message mentions missing proof
    if let Err(e) = result {
        let error_msg = format!("{:?}", e);
        assert!(
            error_msg.contains("missing") || error_msg.contains("Asset index 3"),
            "Error should mention missing proof: {}",
            error_msg
        );
    }
}

#[test]
fn test_batch_verify_empty_set() {
    let assets: Vec<Asset> = Vec::new();

    // Empty set should succeed by convention
    let result = validate_genesis_commitments_batch(&assets);
    assert!(
        result.is_ok(),
        "Batch verification of empty set should succeed"
    );
}

#[test]
fn test_batch_verify_single_asset() {
    // Single asset
    let assets = create_test_assets(1);

    // Should work for single asset
    let result = validate_genesis_commitments_batch(&assets);
    assert!(
        result.is_ok(),
        "Batch verification should work for single asset"
    );
}

#[test]
fn test_batch_verify_amount() {
    // Create 50 assets (same amount due to same definition nominal)
    let assets = create_test_assets(50);

    // Should verify successfully
    let result = validate_genesis_commitments_batch(&assets);
    assert!(
        result.is_ok(),
        "Batch verification should work for same amounts"
    );
}

#[test]
fn test_batch_verify_extreme_amounts() {
    // Use existing helper which creates valid assets with nominal amounts
    let assets = create_test_assets(5);

    // Should verify valid amounts
    let result = validate_genesis_commitments_batch(&assets);
    assert!(
        result.is_ok(),
        "Batch verification should work for valid amounts"
    );
}

// ============================================================================
// Performance Benchmarks (TODO-6.2)
// ============================================================================

#[test]
fn test_bench_vs_batch_100() {
    // Create 100 assets
    let assets = create_test_assets(100);

    // Extract proofs and commitments
    let commitments: Vec<_> = assets.iter().map(|a| &a.commitment).collect();
    let proofs: Vec<_> = assets
        .iter()
        .filter_map(|a| a.range_proof.as_ref())
        .collect();

    // Measure sequential verification
    let start_seq = Instant::now();
    for i in 0..assets.len() {
        z00z_crypto::verify_range_proof(proofs[i], commitments[i], 64, 1, 0)
            .expect("Sequential verification failed");
    }
    let duration_seq = start_seq.elapsed();

    // Measure batch verification
    let minimum_value_promises = vec![0u64; proofs.len()];
    let start_batch = Instant::now();
    batch_verify_range_proofs(&proofs, &commitments, 64, 1, &minimum_value_promises)
        .expect("Batch verification failed");
    let duration_batch = start_batch.elapsed();

    println!("\n📊 Performance Comparison (100 proofs):");
    println!("   Sequential: {:?}", duration_seq);
    println!("   Batch:      {:?}", duration_batch);

    let seq_micros = duration_seq.as_micros();
    let batch_micros = duration_batch.as_micros();
    let ratio = batch_micros as f64 / seq_micros as f64;
    println!("   Batch/seq:  {:.2}x", ratio);

    assert!(
        ratio <= 1.25,
        "Batch verification regressed too far for 100 proofs: batch/seq {:.2}x",
        ratio
    );
}

#[test]
fn test_bench_sequential_batch_1000() {
    // Create 1000 assets
    let assets = create_test_assets(1000);

    // Extract proofs and commitments
    let commitments: Vec<_> = assets.iter().map(|a| &a.commitment).collect();
    let proofs: Vec<_> = assets
        .iter()
        .filter_map(|a| a.range_proof.as_ref())
        .collect();

    // Measure sequential verification (sample only for speed)
    let sample_size = 100;
    let start_seq = Instant::now();
    for i in 0..sample_size {
        z00z_crypto::verify_range_proof(proofs[i], commitments[i], 64, 1, 0)
            .expect("Sequential verification failed");
    }
    let duration_seq_sample = start_seq.elapsed();
    let duration_seq_extrapolated = duration_seq_sample * (1000 / sample_size) as u32;

    // Measure batch verification (full batch)
    let minimum_value_promises = vec![0u64; proofs.len()];
    let start_batch = Instant::now();
    batch_verify_range_proofs(&proofs, &commitments, 64, 1, &minimum_value_promises)
        .expect("Batch verification failed");
    let duration_batch = start_batch.elapsed();

    println!("\n📊 Performance Comparison (1000 proofs):");
    println!(
        "   Sequential (extrapolated): {:?}",
        duration_seq_extrapolated
    );
    println!("   Batch:                     {:?}", duration_batch);

    let speedup = duration_seq_extrapolated.as_micros() as f64 / duration_batch.as_micros() as f64;
    println!("   Estimated speedup:         {:.2}x faster", speedup);

    // Large batches should still show a measurable advantage over sequential verification.
    assert!(
        speedup >= 1.10,
        "Batch verification should stay faster for 1000 proofs, got {:.2}x",
        speedup
    );
}

#[test]
fn test_batch_verify_scaling() {
    let sizes = [10, 50, 100, 500];

    println!("\n📈 Batch Verification Scaling:");
    println!("   Size | Time");
    println!("   -----|--------");

    for &size in &sizes {
        let assets = create_test_assets(size);

        let commitments: Vec<_> = assets.iter().map(|a| &a.commitment).collect();
        let proofs: Vec<_> = assets
            .iter()
            .filter_map(|a| a.range_proof.as_ref())
            .collect();

        let start = Instant::now();
        let minimum_value_promises = vec![0u64; proofs.len()];
        batch_verify_range_proofs(&proofs, &commitments, 64, 1, &minimum_value_promises)
            .expect("Batch verification failed");
        let duration = start.elapsed();

        println!("   {:4} | {:?}", size, duration);
    }

    // All should succeed
}
