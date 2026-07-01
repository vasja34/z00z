//! Genesis Performance Benchmarks
//!
//! Criterion benchmarks for genesis generation performance.
//!
//! Run with: cargo bench --bench genesis_bench
//!
//! # Note on Direct rand Usage
//!
//! This benchmark uses direct rand::SeedableRng and ChaCha20Rng instead of
//! z00z_utils::rng abstractions. This is intentional for benchmarks because:
//! 1. Benchmarks need direct access to RNG internals for accurate timing
//! 2. z00z_utils::rng::DeterministicRngProvider uses the same ChaCha20Rng internally
//! 3. The abstraction layer would add overhead that doesn't reflect production performance
//! 4. Benchmarks are not production code and don't need the same abstraction requirements

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::SeedableRng; // Direct usage for benchmarks - see module doc
use rand_chacha::ChaCha20Rng;
use rayon::prelude::*;
use std::sync::Arc;
use z00z_core::assets::nonce::derive_nonce;
use z00z_core::assets::{Asset, AssetClass, AssetDefinition};
use z00z_core::genesis::validator::validate_genesis_commitments_batch;
use z00z_core::genesis::ChainType;
use z00z_core::genesis::{derive_deterministic_rng_seed, derive_genesis_blinding};

/// Create a test AssetDefinition for benchmarking
fn create_bench_definition() -> AssetDefinition {
    AssetDefinition::new(
        [1u8; 32],
        AssetClass::Coin,
        "BenchCoin".to_string(),
        "BCH".to_string(),
        8,
        10,
        1_000_000,
        "bench.z00z".to_string(),
        1,
        1,
        0,
        None,
    )
    .unwrap()
}

/// Create test AssetDefinition with custom serial count
fn create_bench_definition_with_serials(serials: u32) -> AssetDefinition {
    AssetDefinition::new(
        [1u8; 32],
        AssetClass::Coin,
        "BenchCoin".to_string(),
        "BCH".to_string(),
        8,
        serials,
        1_000_000,
        "bench.z00z".to_string(),
        1,
        1,
        0,
        None,
    )
    .unwrap()
}

/// Generate a single test asset for benchmarking
fn generate_bench_asset_with_id(
    definition: Arc<AssetDefinition>,
    serial_id: u32,
    genesis_seed: &[u8; 32],
) -> Asset {
    let amount = definition.nominal;

    let blinding =
        derive_genesis_blinding(genesis_seed, &definition.id, serial_id, ChainType::Devnet)
            .unwrap();
    let nonce = derive_nonce(genesis_seed, serial_id as u64, 0, &[0u8; 32]);
    let rng_seed =
        derive_deterministic_rng_seed(genesis_seed, &definition.id, serial_id, ChainType::Devnet);
    let mut rng = ChaCha20Rng::from_seed(rng_seed);

    Asset::new(definition, serial_id, amount, &blinding, nonce, &mut rng).unwrap()
}

/// Benchmark: Single asset generation
///
/// Measures the time to generate a single genesis asset including:
/// - Blinding factor derivation
/// - Nonce generation
/// - RNG setup
/// - Asset creation with range proof
fn bench_single_asset_generation(c: &mut Criterion) {
    let genesis_seed = [0x42u8; 32];
    let definition = Arc::new(create_bench_definition());

    c.bench_function("generate_single_asset", |b| {
        b.iter(|| {
            let serial_id = 0u32;
            let amount = definition.nominal;

            let blinding = derive_genesis_blinding(
                black_box(&genesis_seed),
                black_box(&definition.id),
                black_box(serial_id),
                ChainType::Devnet,
            )
            .unwrap();

            let nonce = derive_nonce(
                black_box(&genesis_seed),
                black_box(serial_id as u64),
                black_box(0),
                black_box(&[0u8; 32]),
            );

            let rng_seed = derive_deterministic_rng_seed(
                black_box(&genesis_seed),
                black_box(&definition.id),
                black_box(serial_id),
                ChainType::Devnet,
            );
            let mut rng = ChaCha20Rng::from_seed(rng_seed);

            Asset::new(
                black_box(Arc::clone(&definition)),
                black_box(serial_id),
                black_box(amount),
                black_box(&blinding),
                black_box(nonce),
                &mut rng,
            )
            .unwrap()
        })
    });
}

/// Benchmark: Parallel generation of 1000 assets
///
/// Measures throughput of nested parallel generation:
/// - Level 2 parallelism across serial IDs
/// - Tests scalability with rayon thread pool
/// - Validates performance target: ~1000 assets/sec
fn bench_parallel_generation_1000_assets(c: &mut Criterion) {
    let genesis_seed = [0x42u8; 32];
    let definition = Arc::new(create_bench_definition_with_serials(1000));

    c.bench_function("generate_1000_assets_parallel", |b| {
        b.iter(|| {
            let assets: Vec<Asset> = (0..black_box(definition.serials))
                .into_par_iter()
                .map(|serial_id| {
                    let amount = definition.nominal;

                    let blinding = derive_genesis_blinding(
                        &genesis_seed,
                        &definition.id,
                        serial_id,
                        ChainType::Devnet,
                    )
                    .unwrap();

                    let nonce = derive_nonce(&genesis_seed, serial_id as u64, 0, &[0u8; 32]);

                    let rng_seed = derive_deterministic_rng_seed(
                        &genesis_seed,
                        &definition.id,
                        serial_id,
                        ChainType::Devnet,
                    );
                    let mut rng = ChaCha20Rng::from_seed(rng_seed);

                    Asset::new(
                        Arc::clone(&definition),
                        serial_id,
                        amount,
                        &blinding,
                        nonce,
                        &mut rng,
                    )
                    .unwrap()
                })
                .collect();

            black_box(assets)
        })
    });
}

/// Benchmark: Batch validation of 100 commitments
///
/// Measures range proof verification performance:
/// - Batch verification (O(log n) complexity)
/// - Target: <10ms for 100 proofs
/// - Critical for genesis state validation
fn bench_batch_commitment_validation_100(c: &mut Criterion) {
    let genesis_seed = [0x42u8; 32];
    let definition = Arc::new(create_bench_definition_with_serials(100));

    // Pre-generate assets for validation benchmark
    let assets: Vec<Asset> = (0..100)
        .map(|serial_id| {
            generate_bench_asset_with_id(Arc::clone(&definition), serial_id, &genesis_seed)
        })
        .collect();

    c.bench_function("validate_100_commitments_batch", |b| {
        b.iter(|| validate_genesis_commitments_batch(black_box(&assets)))
    });
}

criterion_group!(
    benches,
    bench_single_asset_generation,
    bench_parallel_generation_1000_assets,
    bench_batch_commitment_validation_100
);
criterion_main!(benches);
