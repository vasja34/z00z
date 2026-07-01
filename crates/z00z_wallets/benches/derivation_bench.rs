//! Benchmarks for BIP-32/BIP-44 derivation hot paths
//!
//! This benchmark suite measures the performance of:
//! 1. Master key initialization from BIP-39 seed
//! 2. Account-level derivation
//! 3. Leaf derivation (address keys)
//! 4. Ristretto mapping (BIP-32 to Ristretto conversion)
//!
//! Note: All benchmarks use a deterministic seed for consistent results.
//! This is appropriate for measuring deterministic cryptographic operations.

use bip32::ChildNumber;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use std::str::FromStr;
use std::sync::Arc;
use z00z_wallets::key::{
    Bip32KeyDeriver, Bip39Seed64, Bip44KeyManager, Bip44Path, KeyManagerImpl, MasterKeyGenerator,
    RistrettoBridge,
};
use z00z_wallets::receiver::{ReceiverManager, ReceiverManagerImpl};
use z00z_wallets::ChainType;

const BENCH_SEED: [u8; 64] = [
    0x7A, 0x31, 0xC9, 0x0F, 0x4B, 0x88, 0xE2, 0xD1, 0x9F, 0x3C, 0xA6, 0x57, 0x0B, 0xD4, 0x2A, 0xF8,
    0x61, 0x04, 0xBE, 0x93, 0xCD, 0x72, 0x18, 0xAA, 0x56, 0xE0, 0x33, 0x9D, 0xF1, 0x67, 0x2C, 0x80,
    0xD7, 0x4E, 0x1B, 0x99, 0xA4, 0x06, 0x5F, 0xE8, 0x23, 0xBA, 0xC1, 0x70, 0xD2, 0x3E, 0x14, 0x8B,
    0xFF, 0x35, 0x69, 0x10, 0xCE, 0xA9, 0x47, 0x5D, 0x82, 0x26, 0xF4, 0x0A, 0x9A, 0xB7, 0x58, 0xE5,
];

/// Benchmark master key initialization from BIP-39 seed
fn bench_master_key_initialization(c: &mut Criterion) {
    let seed = BENCH_SEED;

    c.bench_function("bip32/master_key_initialization", |b| {
        b.iter(|| {
            let master = MasterKeyGenerator::from_seed(&seed).unwrap();
            criterion::black_box(master);
        })
    });
}

/// Benchmark account-level derivation
fn bench_account_derivation(c: &mut Criterion) {
    let seed_bytes = BENCH_SEED;
    let manager =
        Bip44KeyManager::new(Bip39Seed64::new(seed_bytes), 1337, ChainType::Devnet).unwrap();

    c.bench_function("bip32/account_derivation", |b| {
        b.iter(|| {
            let account_key = manager.derive_account_key(0).unwrap();
            criterion::black_box(account_key);
        })
    });
}

/// Benchmark leaf derivation (address keys)
fn bench_leaf_derivation(c: &mut Criterion) {
    let seed_bytes = BENCH_SEED;
    let manager =
        Bip44KeyManager::new(Bip39Seed64::new(seed_bytes), 1337, ChainType::Devnet).unwrap();

    c.bench_function("bip32/leaf_derivation", |b| {
        b.iter(|| {
            let address_key = manager.derive_address_key(0, 0, 0).unwrap();
            criterion::black_box(address_key);
        })
    });
}

/// Benchmark Ristretto mapping (BIP-32 to Ristretto conversion)
fn bench_ristretto_mapping(c: &mut Criterion) {
    let seed_bytes = BENCH_SEED;
    let manager =
        Bip44KeyManager::new(Bip39Seed64::new(seed_bytes), 1337, ChainType::Devnet).unwrap();
    let xprv = manager.derive_address_key(0, 0, 0).unwrap();
    let path = Bip44Path::new_z00z(0, 0, 0).unwrap();

    c.bench_function("bip32/ristretto_mapping", |b| {
        b.iter(|| {
            let ristretto_key =
                RistrettoBridge::to_ristretto_key(&xprv, ChainType::Devnet, &path).unwrap();
            criterion::black_box(ristretto_key);
        })
    });
}

/// Benchmark full derivation path (master -> account -> leaf -> Ristretto)
fn bench_full_derivation_path(c: &mut Criterion) {
    let seed = BENCH_SEED;

    c.bench_function("bip32/full_derivation_path", |b| {
        b.iter(|| {
            // Master key initialization
            let master = MasterKeyGenerator::from_seed(&seed).unwrap();

            // Account-level derivation (use partial path)
            let account_partial = vec![
                ChildNumber::new(44, true).unwrap(),
                ChildNumber::new(1337, true).unwrap(),
                ChildNumber::new(0, true).unwrap(),
            ];
            let account_key =
                Bip32KeyDeriver::derive_from_intermediate(&master, &account_partial).unwrap();

            // Leaf derivation
            let leaf_path = Bip44Path::from_str("m/44'/1337'/0'/0/0").unwrap();
            let leaf_key = Bip32KeyDeriver::derive_child(&master, &leaf_path).unwrap();

            // Ristretto mapping
            let ristretto_key =
                RistrettoBridge::to_ristretto_key(&leaf_key, ChainType::Devnet, &leaf_path)
                    .unwrap();

            criterion::black_box((account_key, ristretto_key));
        })
    });
}

/// Benchmark BIP-44 key manager full workflow
fn bench_bip44_key_manager_workflow(c: &mut Criterion) {
    let seed_bytes = BENCH_SEED;

    c.bench_function("bip32/bip44_key_manager_workflow", |b| {
        b.iter(|| {
            // Create manager
            let manager =
                Bip44KeyManager::new(Bip39Seed64::new(seed_bytes), 1337, ChainType::Devnet)
                    .unwrap();

            // Derive account key
            let account_key = manager.derive_account_key(0).unwrap();

            // Derive address key
            let address_key = manager.derive_address_key(0, 0, 0).unwrap();

            // Derive Ristretto key
            let ristretto_key = manager.derive_ristretto_key(0, 0, 0).unwrap();

            criterion::black_box((account_key, address_key, ristretto_key));
        })
    });
}

/// Benchmark cold cache performance (first derivation)
fn bench_cold_cache_derivation(c: &mut Criterion) {
    let seed_bytes = BENCH_SEED;

    c.bench_function("bip32/cold_cache_derivation", |b| {
        b.iter(|| {
            // Create new manager each iteration (cold cache)
            let manager =
                Bip44KeyManager::new(Bip39Seed64::new(seed_bytes), 1337, ChainType::Devnet)
                    .unwrap();

            // Derive address key (first derivation, no cache)
            let address_key = manager.derive_address_key(0, 0, 0).unwrap();

            criterion::black_box(address_key);
        })
    });
}

/// Benchmark concurrent derivation performance
fn bench_concurrent_derivation(c: &mut Criterion) {
    let seed_bytes = BENCH_SEED;

    c.bench_function("bip32/concurrent_derivation", |b| {
        b.iter(|| {
            let manager = Arc::new(
                Bip44KeyManager::new(Bip39Seed64::new(seed_bytes), 1337, ChainType::Devnet)
                    .unwrap(),
            );

            // Derive multiple keys concurrently using rayon
            use rayon::prelude::*;
            let results: Vec<_> = (0..10)
                .into_par_iter()
                .map(|i| manager.derive_address_key(0, 0, i).unwrap())
                .collect();

            criterion::black_box(results);
        })
    });
}

/// Benchmark error path (invalid coin type)
fn bench_error_path(c: &mut Criterion) {
    let seed_bytes = BENCH_SEED;

    c.bench_function("bip32/error_path_invalid_asset", |b| {
        b.iter(|| {
            let result = Bip44KeyManager::new(Bip39Seed64::new(seed_bytes), 0, ChainType::Devnet);
            let _ = criterion::black_box(result);
        })
    });
}

/// Benchmark ReceiverManager derive_spend_key cold path.
fn bench_receiver_manager_derive_cold(c: &mut Criterion) {
    let seed_bytes = BENCH_SEED;
    let path = Bip44Path::payment(0).unwrap();

    c.bench_function("receiver_manager/derive_spend_key_cold", |b| {
        b.iter_batched(
            || {
                let mut key_manager = KeyManagerImpl::new();
                key_manager
                    .init_from_seed(&seed_bytes, ChainType::Devnet)
                    .unwrap();
                ReceiverManagerImpl::new(key_manager).build().unwrap()
            },
            |mut mgr| {
                let pk = mgr.derive_spend_key(path).unwrap();
                criterion::black_box(pk);
            },
            BatchSize::SmallInput,
        )
    });
}

/// Benchmark ReceiverManager derive_spend_key after warming cache for the same path.
fn bench_receiver_manager_derive_warm(c: &mut Criterion) {
    let seed_bytes = BENCH_SEED;
    let path = Bip44Path::payment(0).unwrap();

    c.bench_function("receiver_manager/derive_spend_key_warm", |b| {
        b.iter_batched(
            || {
                let mut key_manager = KeyManagerImpl::new();
                key_manager
                    .init_from_seed(&seed_bytes, ChainType::Devnet)
                    .unwrap();
                let mut mgr = ReceiverManagerImpl::new(key_manager).build().unwrap();

                let _ = mgr.derive_spend_key(path).unwrap();
                mgr
            },
            |mut mgr| {
                let pk = mgr.derive_spend_key(path).unwrap();
                criterion::black_box(pk);
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    bench_master_key_initialization,
    bench_account_derivation,
    bench_leaf_derivation,
    bench_ristretto_mapping,
    bench_full_derivation_path,
    bench_bip44_key_manager_workflow,
    bench_cold_cache_derivation,
    bench_concurrent_derivation,
    bench_error_path,
    bench_receiver_manager_derive_cold,
    bench_receiver_manager_derive_warm,
);
criterion_main!(benches);
