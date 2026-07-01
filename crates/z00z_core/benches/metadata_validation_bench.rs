/// Metadata Validation Benchmarks
///
/// **Reference**: See `assets_benches_review.md` Section "M-3: Metadata validation benchmarks"
///
/// Measures real metadata validation performance:
/// - Blake2b-256 hash computation for metadata commitment
/// - Hash verification (recompute + compare)
/// - Field operations on custom_fields BTreeMap
///
/// ## Why this matters:
/// - Metadata validation happens on every asset transaction
/// - Hash verification is critical for integrity checks
/// - BTreeMap operations must scale with custom field count
///
/// ## Scenarios tested:
/// - Minimal metadata (no custom fields)
/// - Small metadata (3 custom fields, ~100 bytes)
/// - Medium metadata (10 custom fields, ~1KB)
/// - Large metadata (50 custom fields, ~10KB)
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use std::collections::BTreeMap;
use z00z_core::AssetMetadata;

/// Benchmark: Compute metadata hash (Blake2b-256)
/// Hash covers: custom_fields (sorted) || timestamp
fn metadata_compute_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("metadata_hash_computation");

    // Minimal: no custom fields
    group.bench_function("minimal_no_fields", |b| {
        b.iter_batched(
            || AssetMetadata {
                custom_fields: BTreeMap::new(),
                metadata_hash: [0u8; 32],
                timestamp: 1234567890,
            },
            |metadata| black_box(metadata.compute_hash()),
            BatchSize::SmallInput,
        )
    });

    // Small: 3 fields, ~100 bytes
    group.bench_function("small_3_fields", |b| {
        b.iter_batched(
            || {
                let mut custom_fields = BTreeMap::new();
                custom_fields.insert("name".to_string(), "Test Token".to_string());
                custom_fields.insert("symbol".to_string(), "TST".to_string());
                custom_fields.insert("memo".to_string(), "x".repeat(50));
                AssetMetadata {
                    custom_fields,
                    metadata_hash: [0u8; 32],
                    timestamp: 1234567890,
                }
            },
            |metadata| black_box(metadata.compute_hash()),
            BatchSize::SmallInput,
        )
    });

    // Medium: 10 fields, ~1KB
    group.bench_function("medium_10_fields", |b| {
        b.iter_batched(
            || {
                let mut custom_fields = BTreeMap::new();
                for i in 0..10 {
                    custom_fields.insert(format!("field_{}", i), "x".repeat(100));
                }
                AssetMetadata {
                    custom_fields,
                    metadata_hash: [0u8; 32],
                    timestamp: 1234567890,
                }
            },
            |metadata| black_box(metadata.compute_hash()),
            BatchSize::SmallInput,
        )
    });

    // Large: 50 fields, ~10KB
    group.bench_function("large_50_fields", |b| {
        b.iter_batched(
            || {
                let mut custom_fields = BTreeMap::new();
                for i in 0..50 {
                    custom_fields.insert(format!("field_{}", i), "x".repeat(200));
                }
                AssetMetadata {
                    custom_fields,
                    metadata_hash: [0u8; 32],
                    timestamp: 1234567890,
                }
            },
            |metadata| black_box(metadata.compute_hash()),
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

/// Benchmark: Verify metadata hash (recompute + compare)
/// This is the actual validation operation used in production
fn metadata_verify_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("metadata_hash_verification");

    // Minimal: no custom fields
    group.bench_function("minimal_no_fields", |b| {
        b.iter_batched(
            || {
                let mut metadata = AssetMetadata {
                    custom_fields: BTreeMap::new(),
                    metadata_hash: [0u8; 32],
                    timestamp: 1234567890,
                };
                // Compute correct hash
                metadata.metadata_hash = metadata.compute_hash();
                metadata
            },
            |metadata| black_box(metadata.verify_hash()),
            BatchSize::SmallInput,
        )
    });

    // Small: 3 fields
    group.bench_function("small_3_fields", |b| {
        b.iter_batched(
            || {
                let mut custom_fields = BTreeMap::new();
                custom_fields.insert("name".to_string(), "Test Token".to_string());
                custom_fields.insert("symbol".to_string(), "TST".to_string());
                custom_fields.insert("memo".to_string(), "x".repeat(50));
                let mut metadata = AssetMetadata {
                    custom_fields,
                    metadata_hash: [0u8; 32],
                    timestamp: 1234567890,
                };
                metadata.metadata_hash = metadata.compute_hash();
                metadata
            },
            |metadata| black_box(metadata.verify_hash()),
            BatchSize::SmallInput,
        )
    });

    // Medium: 10 fields
    group.bench_function("medium_10_fields", |b| {
        b.iter_batched(
            || {
                let mut custom_fields = BTreeMap::new();
                for i in 0..10 {
                    custom_fields.insert(format!("field_{}", i), "x".repeat(100));
                }
                let mut metadata = AssetMetadata {
                    custom_fields,
                    metadata_hash: [0u8; 32],
                    timestamp: 1234567890,
                };
                metadata.metadata_hash = metadata.compute_hash();
                metadata
            },
            |metadata| black_box(metadata.verify_hash()),
            BatchSize::SmallInput,
        )
    });

    // Large: 50 fields
    group.bench_function("large_50_fields", |b| {
        b.iter_batched(
            || {
                let mut custom_fields = BTreeMap::new();
                for i in 0..50 {
                    custom_fields.insert(format!("field_{}", i), "x".repeat(200));
                }
                let mut metadata = AssetMetadata {
                    custom_fields,
                    metadata_hash: [0u8; 32],
                    timestamp: 1234567890,
                };
                metadata.metadata_hash = metadata.compute_hash();
                metadata
            },
            |metadata| black_box(metadata.verify_hash()),
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

/// Benchmark: BTreeMap field operations
/// Measures insertion and lookup performance with varying field counts
fn metadata_btreemap_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("metadata_btreemap_ops");

    // Insert 10 fields
    group.bench_function("insert_10_fields", |b| {
        b.iter(|| {
            let mut custom_fields = BTreeMap::new();
            for i in 0..10 {
                custom_fields.insert(
                    black_box(format!("field_{}", i)),
                    black_box("value".to_string()),
                );
            }
            black_box(custom_fields)
        })
    });

    // Lookup in 50 fields
    group.bench_function("lookup_in_50_fields", |b| {
        let mut custom_fields = BTreeMap::new();
        for i in 0..50 {
            custom_fields.insert(format!("field_{}", i), "value".to_string());
        }
        b.iter(|| {
            for i in 0..10 {
                black_box(custom_fields.get(&format!("field_{}", i)));
            }
        })
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_millis(500))
        .measurement_time(std::time::Duration::from_millis(1500))
        .sample_size(10)
        .without_plots();
    targets = metadata_compute_hash, metadata_verify_hash, metadata_btreemap_operations
}
criterion_main!(benches);
