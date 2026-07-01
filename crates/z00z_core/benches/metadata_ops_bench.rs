//! Metadata Operations Benchmarks
//!
//! Tests performance of asset metadata operations: hashing, field updates, validation.
//! Critical for NFT metadata, token descriptions, and off-chain data anchoring.
//!
//! Reference: assets_benches_review.md Section "Recommendation #6"
//!
//! Scenarios:
//! - Metadata hash computation (Blake2b over custom fields)
//! - Field insertion/update in small vs large metadata maps
//! - Hash verification (recompute + compare)
//! - Metadata size scaling (100 bytes to 10KB)
//!
//! Expected Performance:
//! - Hash computation: O(n) where n=total metadata size
//! - Field update: O(log n) for BTreeMap insertion
//! - Hash verification: Same as computation + constant comparison
//!
//! Real-world context:
//! - NFT minting: Anchor metadata hash on-chain
//! - Token updates: Modify description, links, etc.
//! - Metadata validation: Verify integrity against on-chain hash

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use std::collections::BTreeMap;
use z00z_core::assets::AssetMetadata;

/// Benchmark: Compute metadata hash for small metadata (100 bytes)
///
/// Small metadata - typical for simple tokens (name, symbol, decimals).
/// Blake2b hash over sorted BTreeMap entries + timestamp.
/// Expected: ~100-200 nanoseconds for Blake2b + BTreeMap iteration
fn bench_metadata_hash_100bytes(c: &mut Criterion) {
    c.bench_function("metadata_hash_100bytes", |b| {
        b.iter_batched(
            || {
                // Setup: Create small metadata (~100 bytes)
                let mut fields = BTreeMap::new();
                fields.insert("name".to_string(), "MyToken".to_string());
                fields.insert("symbol".to_string(), "MTK".to_string());
                fields.insert("decimals".to_string(), "18".to_string());
                fields.insert(
                    "description".to_string(),
                    "A simple token for testing".to_string(),
                );

                AssetMetadata {
                    custom_fields: fields,
                    metadata_hash: [0u8; 32], // Placeholder
                    timestamp: 1700000000,
                }
            },
            |metadata| {
                // Measure: Compute Blake2b hash
                metadata.compute_hash()
            },
            BatchSize::SmallInput,
        );
    });
}

/// Benchmark: Compute metadata hash for medium metadata (1KB)
///
/// Medium metadata - typical for NFTs with multiple attributes.
/// Example: NFT with 10-20 properties (rarity, stats, traits, etc.)
/// Expected: ~500-1000 nanoseconds (scales linearly with data size)
fn bench_metadata_hash_1kb(c: &mut Criterion) {
    c.bench_function("metadata_hash_1kb", |b| {
        b.iter_batched(
            || {
                // Setup: Create medium metadata (~1KB)
                let mut fields = BTreeMap::new();
                fields.insert("name".to_string(), "Epic Sword NFT".to_string());
                fields.insert(
                    "description".to_string(),
                    "A legendary sword with powerful enchantments".to_string(),
                );
                fields.insert("image".to_string(), "ipfs://Qm...hash...".to_string());
                fields.insert("rarity".to_string(), "legendary".to_string());
                fields.insert("attack".to_string(), "150".to_string());
                fields.insert("defense".to_string(), "75".to_string());
                fields.insert("durability".to_string(), "1000".to_string());
                fields.insert("enchantment".to_string(), "fire_damage_+25%".to_string());
                fields.insert("creator".to_string(), "0x1234567890abcdef".to_string());
                fields.insert(
                    "collection".to_string(),
                    "Legendary Weapons Series".to_string(),
                );

                // Add more fields to reach ~1KB
                for i in 0..20 {
                    fields.insert(format!("trait_{}", i), format!("value_{}", i));
                }

                AssetMetadata {
                    custom_fields: fields,
                    metadata_hash: [0u8; 32],
                    timestamp: 1700000000,
                }
            },
            |metadata| {
                // Measure: Hash computation for 1KB metadata
                metadata.compute_hash()
            },
            BatchSize::SmallInput,
        );
    });
}

/// Benchmark: Compute metadata hash for large metadata (10KB)
///
/// Large metadata - complex NFTs with extensive attributes or embedded data.
/// Example: Generative art NFT with all generation parameters.
/// Expected: ~3-5 microseconds (linear scaling with size)
fn bench_metadata_hash_10kb(c: &mut Criterion) {
    c.bench_function("metadata_hash_10kb", |b| {
        b.iter_batched(
            || {
                // Setup: Create large metadata (~10KB)
                let mut fields = BTreeMap::new();

                // Large description field
                fields.insert("description".to_string(), "x".repeat(5000));

                // Many attributes
                for i in 0..100 {
                    fields.insert(
                        format!("property_{}", i),
                        format!("detailed_value_with_more_text_{}", i),
                    );
                }

                AssetMetadata {
                    custom_fields: fields,
                    metadata_hash: [0u8; 32],
                    timestamp: 1700000000,
                }
            },
            |metadata| {
                // Measure: Hash computation for 10KB metadata
                metadata.compute_hash()
            },
            BatchSize::SmallInput,
        );
    });
}

/// Benchmark: Add field to small metadata map
///
/// Insert new field into metadata with 5 existing fields.
/// BTreeMap insertion is O(log n) - should be very fast for small maps.
/// Expected: ~20-50 nanoseconds (BTreeMap insert + string allocation)
fn bench_metadata_add_field_small(c: &mut Criterion) {
    c.bench_function("metadata_add_field_small_map", |b| {
        b.iter_batched(
            || {
                // Setup: Small metadata map (5 fields)
                let mut fields = BTreeMap::new();
                fields.insert("name".to_string(), "Token".to_string());
                fields.insert("symbol".to_string(), "TKN".to_string());
                fields.insert("decimals".to_string(), "18".to_string());
                fields.insert("total_supply".to_string(), "1000000".to_string());
                fields.insert("website".to_string(), "https://example.com".to_string());
                fields
            },
            |mut fields| {
                // Measure: Insert new field
                fields.insert("contact".to_string(), "team@example.com".to_string());
                fields
            },
            BatchSize::SmallInput,
        );
    });
}

/// Benchmark: Add field to large metadata map
///
/// Insert new field into metadata with 1000 existing fields.
/// BTreeMap is O(log n) - should scale logarithmically.
/// Expected: ~50-100 nanoseconds (log(1000) ≈ 10 comparisons)
fn bench_metadata_add_field_large(c: &mut Criterion) {
    c.bench_function("metadata_add_field_large_map", |b| {
        b.iter_batched(
            || {
                // Setup: Large metadata map (1000 fields)
                let mut fields = BTreeMap::new();
                for i in 0..1000 {
                    fields.insert(format!("field_{}", i), format!("value_{}", i));
                }
                fields
            },
            |mut fields| {
                // Measure: Insert new field into large map
                fields.insert("new_field".to_string(), "new_value".to_string());
                fields
            },
            BatchSize::SmallInput,
        );
    });
}

/// Benchmark: Update existing field in metadata
///
/// Modify value of existing field - same as insertion in BTreeMap.
/// Expected: ~20-50 nanoseconds for small map
fn bench_metadata_update_field(c: &mut Criterion) {
    c.bench_function("metadata_update_existing_field", |b| {
        b.iter_batched(
            || {
                // Setup: Metadata with field to update
                let mut fields = BTreeMap::new();
                fields.insert("name".to_string(), "OldName".to_string());
                fields.insert("version".to_string(), "1.0".to_string());
                fields
            },
            |mut fields| {
                // Measure: Update existing field
                fields.insert("version".to_string(), "2.0".to_string());
                fields
            },
            BatchSize::SmallInput,
        );
    });
}

/// Benchmark: Verify metadata hash (recompute + compare)
///
/// Full verification: recompute hash and compare with stored hash.
/// Used to validate metadata integrity against on-chain commitment.
/// Expected: Same as hash computation + ~1ns for comparison
fn bench_metadata_verify_hash(c: &mut Criterion) {
    c.bench_function("metadata_verify_hash", |b| {
        b.iter_batched(
            || {
                // Setup: Create metadata with correct hash
                let mut fields = BTreeMap::new();
                fields.insert("name".to_string(), "VerifiedToken".to_string());
                fields.insert("symbol".to_string(), "VTK".to_string());

                let mut metadata = AssetMetadata {
                    custom_fields: fields,
                    metadata_hash: [0u8; 32],
                    timestamp: 1700000000,
                };

                // Compute correct hash
                metadata.metadata_hash = metadata.compute_hash();
                metadata
            },
            |metadata| {
                // Measure: Verify hash (recompute + compare)
                metadata.verify_hash()
            },
            BatchSize::SmallInput,
        );
    });
}

/// Benchmark: Verify metadata hash for large metadata (10KB)
///
/// Verification cost scales with metadata size (must rehash all data).
/// Expected: Same as 10KB hash computation + comparison
fn bench_metadata_verify_hash_large(c: &mut Criterion) {
    c.bench_function("metadata_verify_hash_10kb", |b| {
        b.iter_batched(
            || {
                // Setup: Large metadata with correct hash
                let mut fields = BTreeMap::new();
                fields.insert("description".to_string(), "x".repeat(5000));
                for i in 0..100 {
                    fields.insert(format!("attr_{}", i), format!("val_{}", i));
                }

                let mut metadata = AssetMetadata {
                    custom_fields: fields,
                    metadata_hash: [0u8; 32],
                    timestamp: 1700000000,
                };

                metadata.metadata_hash = metadata.compute_hash();
                metadata
            },
            |metadata| {
                // Measure: Verify large metadata hash
                metadata.verify_hash()
            },
            BatchSize::SmallInput,
        );
    });
}

/// Benchmark: Create metadata from scratch (full construction)
///
/// Real-world scenario: NFT minting with metadata.
/// Includes BTreeMap creation, field insertion, and hash computation.
/// Expected: ~200-500 nanoseconds total
fn bench_metadata_create_full(c: &mut Criterion) {
    c.bench_function("metadata_create_full_nft", |b| {
        b.iter(|| {
            // Measure: Full metadata creation
            let mut fields = BTreeMap::new();
            fields.insert("name".to_string(), "Cool NFT #1234".to_string());
            fields.insert(
                "description".to_string(),
                "A unique digital artwork".to_string(),
            );
            fields.insert("image".to_string(), "ipfs://QmHash...".to_string());
            fields.insert(
                "attributes".to_string(),
                r#"{"rarity":"rare","type":"art"}"#.to_string(),
            );

            let mut metadata = AssetMetadata {
                custom_fields: fields,
                metadata_hash: [0u8; 32],
                timestamp: 1700000000,
            };

            // Compute hash
            metadata.metadata_hash = metadata.compute_hash();
            metadata
        });
    });
}

/// Benchmark: Clone metadata (deep copy)
///
/// Metadata cloning needed when creating derived assets or updating.
/// BTreeMap clone requires copying all entries.
/// Expected: Scales with number of fields and total size
fn bench_metadata_clone(c: &mut Criterion) {
    c.bench_function("metadata_clone_medium", |b| {
        b.iter_batched(
            || {
                // Setup: Medium metadata to clone
                let mut fields = BTreeMap::new();
                for i in 0..50 {
                    fields.insert(format!("field_{}", i), format!("value_{}", i));
                }

                AssetMetadata {
                    custom_fields: fields,
                    metadata_hash: [1u8; 32],
                    timestamp: 1700000000,
                }
            },
            |metadata| {
                // Measure: Clone metadata
                metadata.clone()
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group! {
    name = metadata_ops_benches;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_millis(500))
        .measurement_time(std::time::Duration::from_millis(1500))
        .sample_size(10)
        .without_plots();
    targets = bench_metadata_hash_100bytes, bench_metadata_hash_1kb, bench_metadata_hash_10kb,
              bench_metadata_add_field_small, bench_metadata_add_field_large, bench_metadata_update_field,
              bench_metadata_verify_hash, bench_metadata_verify_hash_large, bench_metadata_create_full,
              bench_metadata_clone
}
criterion_main!(metadata_ops_benches);
