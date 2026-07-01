//! Benchmarks for the async batch threshold used by `AsyncReceiverManagerImpl::derive_batch`.
//!
//! This suite measures the tradeoff between:
//! - Inline derivation under an async lock (low overhead for small batches)
//! - `tokio::task::spawn_blocking` (avoids blocking the async runtime for larger batches)
//!
//! The goal is to keep `ASYNC_BATCH_THRESHOLD` aligned with real performance.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use z00z_wallets::{
    key::{Bip44Path, KeyManagerImpl},
    receiver::{
        AsyncReceiverManager, AsyncReceiverManagerImpl, ReceiverManagerConfig, ReceiverManagerImpl,
    },
    ChainType,
};

const BENCH_SEED: [u8; 64] = [
    0x7A, 0x31, 0xC9, 0x0F, 0x4B, 0x88, 0xE2, 0xD1, 0x9F, 0x3C, 0xA6, 0x57, 0x0B, 0xD4, 0x2A, 0xF8,
    0x61, 0x04, 0xBE, 0x93, 0xCD, 0x72, 0x18, 0xAA, 0x56, 0xE0, 0x33, 0x9D, 0xF1, 0x67, 0x2C, 0x80,
    0xD7, 0x4E, 0x1B, 0x99, 0xA4, 0x06, 0x5F, 0xE8, 0x23, 0xBA, 0xC1, 0x70, 0xD2, 0x3E, 0x14, 0x8B,
    0xFF, 0x35, 0x69, 0x10, 0xCE, 0xA9, 0x47, 0x5D, 0x82, 0x26, 0xF4, 0x0A, 0x9A, 0xB7, 0x58, 0xE5,
];

const THRESHOLDS: &[usize] = &[1, 5, 10, 20, 50, 100];
const BATCH_SIZES: &[u32] = &[1, 5, 10, 20, 50, 100];

fn make_paths(batch: u32) -> Vec<Bip44Path> {
    (0..batch).map(|i| Bip44Path::payment(i).unwrap()).collect()
}

fn build_async_mgr(
    threshold: usize,
) -> AsyncReceiverManagerImpl<KeyManagerImpl, z00z_utils::time::SystemTimeProvider> {
    let mut key_manager = KeyManagerImpl::new();
    key_manager
        .init_from_seed(&BENCH_SEED, ChainType::Devnet)
        .unwrap();

    let sync_mgr = ReceiverManagerImpl::new(key_manager).build().unwrap();
    AsyncReceiverManagerImpl::new_with_config(
        sync_mgr,
        ReceiverManagerConfig {
            async_batch_threshold: Some(threshold),
        },
    )
    .unwrap()
}

fn bench_async_batch_threshold(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();

    let mut group = c.benchmark_group("receiver_manager/async_batch_threshold");

    for &threshold in THRESHOLDS {
        for &batch in BATCH_SIZES {
            group.bench_function(format!("batch_{batch}_threshold_{threshold}"), |b| {
                b.iter_batched(
                    || {
                        let mgr = build_async_mgr(threshold);
                        let paths = make_paths(batch);
                        (mgr, paths)
                    },
                    |(mgr, paths)| {
                        let keys = rt
                            .block_on(async { mgr.derive_batch(&paths).await })
                            .unwrap();
                        black_box(keys);
                    },
                    BatchSize::SmallInput,
                )
            });
        }
    }

    group.finish();
}

criterion_group!(benches, bench_async_batch_threshold);
criterion_main!(benches);
