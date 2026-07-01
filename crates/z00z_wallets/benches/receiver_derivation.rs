//! Benchmarks for receiver derivation throughput.
//!
//! This suite compares:
//! - Synchronous derivation using `ReceiverManagerImpl::derive_batch`
//! - Asynchronous derivation using `AsyncReceiverManagerImpl::derive_batch`
//!
//! Use this as a reference when deciding whether to override auto-tuning with a fixed
//! `async_batch_threshold` value.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use std::sync::Arc;
use z00z_wallets::{
    key::{Bip39Seed64, Bip44KeyManager, Bip44Path, KeyManagerImpl, Z00Z_BIP44_ASSET},
    receiver::{
        AsyncReceiverManager, AsyncReceiverManagerImpl, ReceiverManager, ReceiverManagerConfig,
        ReceiverManagerImpl,
    },
    ChainType,
};

const BENCH_SEED: [u8; 64] = [
    0x7A, 0x31, 0xC9, 0x0F, 0x4B, 0x88, 0xE2, 0xD1, 0x9F, 0x3C, 0xA6, 0x57, 0x0B, 0xD4, 0x2A, 0xF8,
    0x61, 0x04, 0xBE, 0x93, 0xCD, 0x72, 0x18, 0xAA, 0x56, 0xE0, 0x33, 0x9D, 0xF1, 0x67, 0x2C, 0x80,
    0xD7, 0x4E, 0x1B, 0x99, 0xA4, 0x06, 0x5F, 0xE8, 0x23, 0xBA, 0xC1, 0x70, 0xD2, 0x3E, 0x14, 0x8B,
    0xFF, 0x35, 0x69, 0x10, 0xCE, 0xA9, 0x47, 0x5D, 0x82, 0x26, 0xF4, 0x0A, 0x9A, 0xB7, 0x58, 0xE5,
];

const BATCH_SIZES: &[u32] = &[1, 2, 5, 10, 20, 50, 100];
const MAX_PATH_SCAN: u32 = 100_000;

fn make_paths(batch: u32) -> Arc<[Bip44Path]> {
    let manager = Bip44KeyManager::new(
        Bip39Seed64::new(BENCH_SEED),
        Z00Z_BIP44_ASSET,
        ChainType::Devnet,
    )
    .expect("bench key manager init must succeed");

    let mut paths = Vec::with_capacity(batch as usize);
    let mut idx = 0u32;
    while paths.len() < batch as usize {
        let path = Bip44Path::payment(idx).expect("bench path build must succeed");

        // BIP-32 derivation can fail for some indices (rare but possible) if the derived
        // key is invalid. Skip such indices to keep this benchmark panic-free.
        if manager.derive_address_key(0, 0, idx).is_ok() {
            paths.push(path);
        }

        idx = idx.saturating_add(1);
        if idx >= MAX_PATH_SCAN {
            panic!("failed to find {batch} derivable paths within scan limit {MAX_PATH_SCAN}");
        }
    }

    Arc::from(paths)
}

fn build_sync_mgr() -> ReceiverManagerImpl<KeyManagerImpl, z00z_utils::time::SystemTimeProvider> {
    let mut key_manager = KeyManagerImpl::new();
    key_manager
        .init_from_seed(&BENCH_SEED, ChainType::Devnet)
        .unwrap();

    ReceiverManagerImpl::new(key_manager).build().unwrap()
}

fn build_async_mgr(
) -> AsyncReceiverManagerImpl<KeyManagerImpl, z00z_utils::time::SystemTimeProvider> {
    let sync_mgr = build_sync_mgr();

    // Use a fixed threshold to reduce variance in this baseline benchmark.
    let cfg = ReceiverManagerConfig {
        async_batch_threshold: Some(10),
    };

    AsyncReceiverManagerImpl::new_with_config(sync_mgr, cfg).unwrap()
}

fn bench_receiver_derivation(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();

    let mut group = c.benchmark_group("receiver_manager/receiver_derivation");

    let path_sets: Vec<(u32, Arc<[Bip44Path]>)> = BATCH_SIZES
        .iter()
        .copied()
        .map(|batch| (batch, make_paths(batch)))
        .collect();

    for (batch, paths_base) in &path_sets {
        group.bench_function(format!("sync_batch_{batch}"), |b| {
            b.iter_batched(
                || {
                    let mgr = build_sync_mgr();
                    let paths = Arc::clone(paths_base);
                    (mgr, paths)
                },
                |(mut mgr, paths)| {
                    let keys = mgr
                        .derive_batch(paths.as_ref())
                        .expect("bench derive_batch sync must succeed for selected paths");
                    black_box(keys);
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(format!("async_batch_{batch}"), |b| {
            b.iter_batched(
                || {
                    let mgr = build_async_mgr();
                    let paths = Arc::clone(paths_base);
                    (mgr, paths)
                },
                |(mgr, paths)| {
                    let keys = rt
                        .block_on(async { mgr.derive_batch(paths.as_ref()).await })
                        .expect("bench derive_batch async must succeed for selected paths");
                    black_box(keys);
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

criterion_group!(benches, bench_receiver_derivation);
criterion_main!(benches);
