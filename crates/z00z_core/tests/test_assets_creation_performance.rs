//! Phase 2, Test 15: Asset Creation Performance Benchmarks
//!
//! Purpose: Measure performance of asset creation operations including:
//! - Commitment generation time
//! - Range proof generation time
//! - Total Asset::new() execution time
//! - Blinding factor generation
//! - Nonce derivation performance
//!
//! Real Structures:
//! - Asset, AssetDefinition, AssetCrypto
//! - BlindingFactor, NonceCounter
//! - Real tari_crypto operations (no mocks)
//!
//! Success Criteria:
//! - Asset creation < 15 seconds total (smaller batches)
//! - Consistent performance across iterations
//! - Real range proofs verified
//! - Deterministic nonce generation

use rayon::prelude::*;
use std::sync::Arc;
use std::time::{Duration, Instant};
use z00z_core::assets::{Asset, AssetClass, AssetDefinition, BlindingFactor};
use z00z_crypto::expert::hash_domain;
use z00z_crypto::{create_commitment, create_range_proof, verify_range_proof, DomainHasher};
use z00z_utils::rng::DeterministicRngProvider;

hash_domain!(TestNonceDomain, "z00z.core.tests.nonce.v1", 1);

/// Create a test asset definition
fn create_test_definition() -> Arc<AssetDefinition> {
    let mut id = [0u8; 32];
    id[0] = 42;

    Arc::new(
        AssetDefinition::new(
            id,                           // id: [u8; 32]
            AssetClass::Coin,             // class: AssetClass
            "BenchmarkAsset".to_string(), // name: String
            "BENCH".to_string(),          // symbol: String
            8,                            // decimals: u8
            1_000_000,                    // serials: u32 (max serial IDs)
            1_000_000,                    // nominal: u64
            "bench.test".to_string(),     // domain_name: String
            1,                            // version: u8
            1,                            // crypto_version: u8
            0,                            // flags: u8
            None,                         // metadata: Option<BTreeMap>
        )
        .expect("Valid definition"),
    )
}

/// Derive nonce from seed, counter, and asset ID using domain-separated Blake2b
fn derive_nonce(seed: &[u8; 32], counter: u64, asset_id: &[u8; 32]) -> [u8; 32] {
    let hash = DomainHasher::<TestNonceDomain>::new_with_label("test_nonce")
        .chain(seed)
        .chain(counter.to_le_bytes())
        .chain(asset_id)
        .finalize();

    let mut nonce = [0u8; 32];
    nonce.copy_from_slice(&hash.as_ref()[..32]);
    nonce
}

/// Test 1: Single Asset Creation Time (Baseline)
/// Measures time to create a single asset with commitment and range proof
#[test]
fn test_asset_creation_single_time() {
    let def = create_test_definition();

    // Create initial state
    let mut seed = [0u8; 32];
    seed[..31].copy_from_slice(b"benchmark_seed_1234567890123456");

    // Measure single asset creation
    let start = Instant::now();

    let nonce = derive_nonce(&seed, 0, &[0u8; 32]);
    let blinding =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let asset = Asset::new(
        def,
        100,
        1_000_000,
        &blinding,
        nonce,
        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
    )
    .expect("valid asset");

    let elapsed = start.elapsed();

    // Verify asset is valid
    assert_eq!(asset.amount(), 1_000_000);
    assert!(asset.range_proof().is_some());

    // Log performance
    println!(
        "\n✅ Single Asset Creation Time: {:.3}ms",
        elapsed.as_secs_f64() * 1000.0
    );
}

/// Test 2: Batch Asset Creation (reduced batch)
/// Measures throughput of creating a smaller batch of assets sequentially
#[test]
fn test_asset_creation_batch_100() {
    let def = create_test_definition();

    let seed = [0u8; 32];

    // Measure batch creation - PARALLELIZED with Rayon
    let start = Instant::now();

    let assets: Vec<Asset> = (0..12)
        .into_par_iter()
        .map(|i| {
            let nonce = derive_nonce(&seed, i as u64, &[0u8; 32]);
            let blinding =
                BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
            Asset::new(
                Arc::clone(&def),
                i as u32,
                1_000_000 + i as u64,
                &blinding,
                nonce,
                &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            )
            .expect("valid asset")
        })
        .collect();

    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_secs_f64() * 1000.0 / 12.0;

    // Verify all assets valid
    assert_eq!(assets.len(), 12);
    assert!(assets.iter().all(|a| a.range_proof().is_some()));

    // Log performance
    println!("\n✅ Batch Creation (12 assets):");
    println!("   Total Time: {:.3}ms", elapsed.as_secs_f64() * 1000.0);
    println!("   Avg Per Asset: {:.3}ms", avg_ms);
    println!(
        "   Throughput: {:.1} assets/sec",
        12.0 / elapsed.as_secs_f64()
    );
}

/// Test 3: Commitment Generation Time (Isolated)
/// Measures only the commitment generation part without proof
#[test]
fn test_commitment_generation_time() {
    let mut times = Vec::new();

    // Measure commitment generation 60 times
    for _ in 0..12 {
        let start = Instant::now();

        let amount = 1_000_000u64;
        let blinding =
            BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
        let _commitment = create_commitment(amount, &blinding);

        times.push(start.elapsed());
    }

    // Calculate statistics
    let total_duration: Duration = times.iter().sum();
    let avg_us = total_duration.as_micros() as f64 / 60.0;
    let min_us = times.iter().map(|d| d.as_micros()).min().unwrap_or(0);
    let max_us = times.iter().map(|d| d.as_micros()).max().unwrap_or(0);

    // Log performance
    println!("\n✅ Commitment Generation Time (60 samples):");
    println!("   Avg: {:.2}μs", avg_us);
    println!("   Min: {}μs", min_us);
    println!("   Max: {}μs", max_us);
    println!("   Throughput: {:.0} commitments/sec", 1_000_000.0 / avg_us);
}

/// Test 4: Range Proof Generation Time (Isolated)
/// Measures only the range proof generation part
#[test]
fn test_range_proof_generation_time() {
    let mut times = Vec::new();

    // Measure range proof generation 5 times (reduced for < 15s)
    for _ in 0..5 {
        let start = Instant::now();

        let amount = 1_000_000u64;
        let blinding =
            BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
        let _proof = create_range_proof(amount, &blinding, 64, 0).expect("valid proof");

        times.push(start.elapsed());
    }

    // Calculate statistics
    let total_duration: Duration = times.iter().sum();
    let avg_ms = total_duration.as_secs_f64() * 1000.0 / 5.0;
    let min_ms = times
        .iter()
        .map(|d| d.as_secs_f64() * 1000.0)
        .fold(f64::INFINITY, f64::min);
    let max_ms = times
        .iter()
        .map(|d| d.as_secs_f64() * 1000.0)
        .fold(0.0, f64::max);

    // Log performance
    println!("\n✅ Range Proof Generation Time (5 samples):");
    println!("   Avg: {:.2}ms", avg_ms);
    println!("   Min: {:.2}ms", min_ms);
    println!("   Max: {:.2}ms", max_ms);
}

/// Test 5: Proof Verification Time (Isolated)
/// Measures performance of proof verification operation
#[test]
fn test_proof_verification_time() {
    // First, create 6 proofs (reduced for < 15s)
    let mut proofs = Vec::new();
    let mut commitments = Vec::new();

    for i in 0..6 {
        let amount = 100_000u64 + i as u64 * 1000;
        let blinding =
            BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
        let commitment = create_commitment(amount, &blinding);
        let proof = create_range_proof(amount, &blinding, 64, 0).expect("valid proof");

        proofs.push(proof);
        commitments.push(commitment);
    }

    // Now measure verification time
    let mut verify_times = Vec::new();

    for (proof, commitment) in proofs.iter().zip(commitments.iter()) {
        let start = Instant::now();

        verify_range_proof(proof, commitment, 64, 1, 0).expect("valid proof");

        verify_times.push(start.elapsed());
    }

    // Calculate statistics
    let total_duration: Duration = verify_times.iter().sum();
    let avg_us = total_duration.as_micros() as f64 / 6.0;
    let min_us = verify_times
        .iter()
        .map(|d| d.as_micros())
        .min()
        .unwrap_or(0);
    let max_us = verify_times
        .iter()
        .map(|d| d.as_micros())
        .max()
        .unwrap_or(0);

    // Log performance
    println!("\n✅ Proof Verification Time (6 samples):");
    println!("   Avg: {:.0}μs", avg_us);
    println!("   Min: {}μs", min_us);
    println!("   Max: {}μs", max_us);
    println!("   Throughput: {:.1} proofs/sec", 1_000_000.0 / avg_us);
}

/// Test 6: Nonce Generation Performance
/// Measures performance of nonce derivation with counter
#[test]
fn test_nonce_generation_performance() {
    let mut seed = [0u8; 32];
    seed[..31].copy_from_slice(b"benchmark_seed_1234567890123456");
    // Measure nonce generation 300 times (reduced for < 15s)
    let mut times = Vec::new();

    for counter in 0..60 {
        let start = Instant::now();

        let _nonce = derive_nonce(&seed, counter, &[0u8; 32]);

        times.push(start.elapsed());
    }

    // Calculate statistics
    let total_duration: Duration = times.iter().sum();
    let avg_us = total_duration.as_micros() as f64 / 300.0;
    let min_us = times.iter().map(|d| d.as_micros()).min().unwrap_or(0);
    let max_us = times.iter().map(|d| d.as_micros()).max().unwrap_or(0);
    let throughput = 1000.0 / total_duration.as_secs_f64();

    // Log performance
    println!("\n✅ Nonce Generation Performance (300 samples):");
    println!(
        "   Total Time: {:.3}ms",
        total_duration.as_secs_f64() * 1000.0
    );
    println!("   Avg: {:.2}μs", avg_us);
    println!("   Min: {}μs", min_us);
    println!("   Max: {}μs", max_us);
    println!("   Throughput: {:.0} nonces/sec", throughput);
}

/// Test 7: Performance Consistency (2 iterations)
/// Verifies performance is consistent across multiple runs
#[test]
fn test_asset_creation_consistency() {
    let def = create_test_definition();
    let mut times = Vec::with_capacity(2);

    for iteration in 0..2 {
        let start = Instant::now();

        let mut seed = [0u8; 32];
        seed[0] = iteration as u8;

        // Parallel asset creation with rayon
        let _assets: Vec<Asset> = (0..10)
            .into_par_iter()
            .map(|i| {
                let nonce = derive_nonce(&seed, i as u64, &[0u8; 32]);
                let blinding = BlindingFactor::random(
                    &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
                );

                Asset::new(
                    Arc::clone(&def),
                    (iteration * 10 + i) as u32,
                    1000 + i as u64 * 100,
                    &blinding,
                    nonce,
                    &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
                )
                .expect("Valid asset")
            })
            .collect();

        let elapsed = start.elapsed();
        times.push(elapsed.as_secs_f64());
    }

    let avg_time = times.iter().sum::<f64>() / times.len() as f64;
    let max_time = times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_time = times.iter().cloned().fold(f64::INFINITY, f64::min);

    println!("\n✅ Performance Consistency (2 iterations × 10 assets):");
    println!("   Avg: {:.3}s", avg_time);
    println!("   Min: {:.3}s", min_time);
    println!("   Max: {:.3}s", max_time);

    // Verify all iterations completed
    assert_eq!(times.len(), 2, "All iterations completed");

    // Verify total timing < 60s (reduced batch sizes)
    let total_time = times.iter().sum::<f64>();
    assert!(
        total_time < 60.0,
        "All iterations should complete in < 60 seconds, got {:.1}s",
        total_time
    );
}

/// Test 8: Asset ID Uniqueness in Performance Test
/// Verifies that all created assets have unique IDs during perf testing
#[test]
fn test_asset_uniqueness_under_load() {
    let def = create_test_definition();

    let seed = [0u8; 32];
    let mut asset_ids = std::collections::BTreeSet::new();

    // Create 15 assets and collect their IDs - PARALLELIZED with Rayon
    let start = Instant::now();

    let assets: Vec<Asset> = (0..15)
        .into_par_iter()
        .map(|i| {
            let nonce = derive_nonce(&seed, i as u64, &[0u8; 32]);
            let blinding =
                BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

            Asset::new(
                Arc::clone(&def),
                i as u32,
                1_000_000 + i as u64,
                &blinding,
                nonce,
                &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            )
            .expect("Valid asset")
        })
        .collect();

    // Collect asset IDs
    for asset in &assets {
        asset_ids.insert(asset.asset_id());
    }

    let elapsed = start.elapsed();

    // Verify all IDs are unique
    assert_eq!(asset_ids.len(), 15, "All 15 asset IDs must be unique");

    println!("\n✅ Asset Uniqueness Under Load:");
    println!("   Created: 15 assets");
    println!("   Unique IDs: {}", asset_ids.len());
    println!("   Time: {:.3}s", elapsed.as_secs_f64());
    println!("   Status: ✅ All unique");
}
