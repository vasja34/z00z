//! Benchmarks for receiver-cache purge overhead.
//!
//! This suite measures the cost of scanning the in-memory LRU cache and removing
//! expired entries. The setup (key derivation + cache population) is performed
//! outside the timed region.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use std::time::{Duration, SystemTime};
use z00z_utils::time::MockTimeProvider;
use z00z_wallets::key::{Bip44Path, KeyManagerImpl};
use z00z_wallets::receiver::{ReceiverManager, ReceiverManagerImpl};
use z00z_wallets::ChainType;

const BENCH_SEED: [u8; 64] = [
    0x7A, 0x31, 0xC9, 0x0F, 0x4B, 0x88, 0xE2, 0xD1, 0x9F, 0x3C, 0xA6, 0x57, 0x0B, 0xD4, 0x2A, 0xF8,
    0x61, 0x04, 0xBE, 0x93, 0xCD, 0x72, 0x18, 0xAA, 0x56, 0xE0, 0x33, 0x9D, 0xF1, 0x67, 0x2C, 0x80,
    0xD7, 0x4E, 0x1B, 0x99, 0xA4, 0x06, 0x5F, 0xE8, 0x23, 0xBA, 0xC1, 0x70, 0xD2, 0x3E, 0x14, 0x8B,
    0xFF, 0x35, 0x69, 0x10, 0xCE, 0xA9, 0x47, 0x5D, 0x82, 0x26, 0xF4, 0x0A, 0x9A, 0xB7, 0x58, 0xE5,
];

fn build_mgr(
    cache_size: usize,
    ttl: Duration,
    time: MockTimeProvider,
) -> ReceiverManagerImpl<KeyManagerImpl, MockTimeProvider> {
    let mut key_manager = KeyManagerImpl::new();
    key_manager
        .init_from_seed(&BENCH_SEED, ChainType::Devnet)
        .unwrap();

    ReceiverManagerImpl::new_with_config(key_manager, cache_size, ttl, time).unwrap()
}

fn fill_cache(mgr: &mut ReceiverManagerImpl<KeyManagerImpl, MockTimeProvider>, count: u32) {
    for i in 0..count {
        let path = Bip44Path::payment(i).unwrap();
        let _ = mgr.derive_spend_key(path).unwrap();
    }
}

fn bench_purge_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("receiver_manager/cache_purge");

    for &entries in &[128u32, 512u32, 1024u32] {
        group.bench_function(format!("purge_scan_{}", entries), |b| {
            b.iter_batched(
                || {
                    let time = MockTimeProvider::new(SystemTime::now());
                    let mut mgr = build_mgr(
                        (entries as usize) + 16,
                        Duration::from_secs(1),
                        time.clone(),
                    );
                    fill_cache(&mut mgr, entries);

                    // Expire all entries.
                    time.advance_by(Duration::from_secs(2));

                    mgr
                },
                |mgr| {
                    mgr.purge_expired().unwrap();
                    black_box(mgr.get_metrics());
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

criterion_group!(benches, bench_purge_scan);
criterion_main!(benches);
