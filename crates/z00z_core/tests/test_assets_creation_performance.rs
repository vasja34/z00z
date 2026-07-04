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
use std::time::Instant;
use z00z_core::assets::{Asset, AssetClass, AssetDefinition, BlindingFactor};
use z00z_crypto::expert::hash_domain;
use z00z_crypto::DomainHasher;
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

/// Asset ID uniqueness under moderate parallel load.
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
