#![allow(clippy::useless_conversion, clippy::unit_arg)]
// crates/z00z_core/benches/registry_bench.rs
//
// Performance benchmarks for AssetRegistry with z00z_utils traits
//
// Benchmarks:
// - Asset registration with NoopLogger (overhead measurement)
// - Asset registration with TracingLogger (real-world overhead)
// - Batch insert performance
// - Snapshot save/load performance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;
use z00z_core::assets::{
    definition::AssetDefinition, registry::AssetDefinitionRegistry, AssetClass,
};
use z00z_utils::prelude::{NoopLogger, NoopMetrics, SystemTimeProvider, TracingLogger};

fn bench_registry_insert_noop_logger(c: &mut Criterion) {
    c.bench_function("registry_insert_noop_logger", |b| {
        let logger = Arc::new(NoopLogger);
        let metrics = Arc::new(NoopMetrics);
        let time = Arc::new(SystemTimeProvider);

        let registry = AssetDefinitionRegistry::new(logger, metrics, time);

        let mut counter = 0u8;
        b.iter(|| {
            let def = AssetDefinition::new(
                [counter; 32],
                AssetClass::Token,
                format!("TKN{}", counter).into(),
                format!("Token {}", counter).into(),
                6,
                25_000,
                1_000_000,
                "test.io".into(),
                1,
                1,
                0b0000_0001,
                None,
            )
            .unwrap();

            registry.insert(def).unwrap();
            counter = counter.wrapping_add(1);
        });
    });
}

fn bench_registry_insert_tracing_logger(c: &mut Criterion) {
    c.bench_function("registry_insert_tracing_logger", |b| {
        let logger = Arc::new(TracingLogger);
        let metrics = Arc::new(NoopMetrics);
        let time = Arc::new(SystemTimeProvider);

        let registry = AssetDefinitionRegistry::new(logger, metrics, time);

        let mut counter = 0u8;
        b.iter(|| {
            let def = AssetDefinition::new(
                [counter; 32],
                AssetClass::Token,
                format!("TKN{}", counter).into(),
                format!("Token {}", counter).into(),
                6,
                25_000,
                1_000_000,
                "test.io".into(),
                1,
                1,
                0b0000_0001,
                None,
            )
            .unwrap();

            registry.insert(def).unwrap();
            counter = counter.wrapping_add(1);
        });
    });
}

fn bench_registry_batch_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_batch_insert");

    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let logger = Arc::new(NoopLogger);
                let metrics = Arc::new(NoopMetrics);
                let time = Arc::new(SystemTimeProvider);

                let registry = AssetDefinitionRegistry::new(logger, metrics, time);

                let mut batch = Vec::with_capacity(size);
                for i in 0..size {
                    let def = AssetDefinition::new(
                        [(i % 256) as u8; 32],
                        AssetClass::Token,
                        format!("TKN{}", i).into(),
                        format!("Token {}", i).into(),
                        6,
                        25_000,
                        1_000_000,
                        "test.io".into(),
                        1,
                        1,
                        0b0000_0001,
                        None,
                    )
                    .unwrap();
                    batch.push(def);
                }

                black_box(registry.insert_batch(batch).unwrap());
            });
        });
    }

    group.finish();
}

fn bench_registry_snapshot_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_snapshot");

    // Create a registry with 100 assets
    let logger = Arc::new(NoopLogger);
    let metrics = Arc::new(NoopMetrics);
    let time = Arc::new(SystemTimeProvider);

    let registry = AssetDefinitionRegistry::new(logger, metrics, time);

    for i in 0..100 {
        let def = AssetDefinition::new(
            [(i % 256) as u8; 32],
            AssetClass::Token,
            format!("TKN{}", i).into(),
            format!("Token {}", i).into(),
            6,
            25_000,
            1_000_000,
            "test.io".into(),
            1,
            1,
            0b0000_0001,
            None,
        )
        .unwrap();
        registry.insert(def).unwrap();
    }

    // Benchmark snapshot creation
    group.bench_function("create_snapshot", |b| {
        b.iter(|| {
            black_box(registry.create_snapshot().unwrap());
        });
    });

    // Benchmark snapshot loading
    let snapshot = registry.create_snapshot().unwrap();
    group.bench_function("update_from_snapshot", |b| {
        b.iter(|| {
            let logger = Arc::new(NoopLogger);
            let metrics = Arc::new(NoopMetrics);
            let time = Arc::new(SystemTimeProvider);

            let new_registry = AssetDefinitionRegistry::new(logger, metrics, time);
            black_box(new_registry.update_from_snapshot(snapshot.clone()).unwrap());
        });
    });

    group.finish();
}

fn bench_registry_concurrent_access(c: &mut Criterion) {
    use std::thread;

    c.bench_function("registry_concurrent_10_threads", |b| {
        b.iter(|| {
            let logger: Arc<dyn z00z_utils::prelude::Logger> = Arc::new(NoopLogger);
            let metrics: Arc<dyn z00z_utils::prelude::MetricsSink> = Arc::new(NoopMetrics);
            let time: Arc<dyn z00z_utils::prelude::TimeProvider> = Arc::new(SystemTimeProvider);

            let registry = Arc::new(AssetDefinitionRegistry::new(logger, metrics, time));

            let handles: Vec<_> = (0..10)
                .map(|i| {
                    let registry = Arc::clone(&registry);
                    thread::spawn(move || {
                        for j in 0..10 {
                            let id = [i * 10 + j; 32];
                            let def = AssetDefinition::new(
                                id,
                                AssetClass::Token,
                                format!("T{}{}", i, j).into(),
                                format!("Token {}{}", i, j).into(),
                                6,
                                25_000,
                                1_000_000,
                                "test.io".into(),
                                1,
                                1,
                                0b0000_0001,
                                None,
                            )
                            .unwrap();
                            registry.insert(def).unwrap();
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }

            black_box(&registry);
        });
    });
}

criterion_group!(
    benches,
    bench_registry_insert_noop_logger,
    bench_registry_insert_tracing_logger,
    bench_registry_batch_insert,
    bench_registry_snapshot_operations,
    bench_registry_concurrent_access,
);
criterion_main!(benches);
