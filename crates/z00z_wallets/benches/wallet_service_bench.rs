use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;

use std::sync::Arc;

use z00z_utils::config::{ConfigSource, EnvConfig};
use z00z_wallets::rpc::types::common::PersistWalletId;
use z00z_wallets::services::{AppService, WalletService};

const TEST_SEED_PHRASE_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

fn env_u32(key: &str, default_value: u32) -> u32 {
    EnvConfig
        .get_typed::<u32>(key)
        .ok()
        .flatten()
        .unwrap_or(default_value)
}

fn setup_service_with_wallets(
    rt: &Runtime,
    wallet_count: u32,
) -> (Arc<WalletService>, Vec<PersistWalletId>) {
    let temp_dir = tempfile::tempdir().expect("tempdir must be creatable");
    let output_dir = temp_dir.path().to_path_buf();

    // Keep the tempdir alive for the lifetime of the service by leaking it.
    // This is acceptable for a benchmark process.
    std::mem::forget(temp_dir);

    let service = Arc::new(WalletService::with_output_dir(output_dir));
    let app = AppService::with_wallet_service(Arc::clone(&service));

    let mut wallet_ids = Vec::with_capacity(wallet_count as usize);
    rt.block_on(async {
        for i in 0..wallet_count {
            let resp = app
                .create_wallet(
                    format!("Bench Wallet {i}"),
                    "f8$u1-krZpQ3!mT7Xv".to_string(),
                    Some(TEST_SEED_PHRASE_24.to_string()),
                )
                .await
                .expect("app.create_wallet must succeed");
            let wallet_id = resp.wallet_id;
            wallet_ids.push(wallet_id);
        }
    });

    (service, wallet_ids)
}

fn wallet_service_benchmarks(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime");

    // IMPORTANT:
    // These benches are included in `cargo test --all-targets`.
    // Keep defaults bounded to avoid long-running test suites.
    // Use env vars to opt into larger workloads when running `cargo bench`.
    let wallets_small = env_u32("Z00Z_BENCH_WALLETS_SMALL", 10);
    let wallets_large = env_u32("Z00Z_BENCH_WALLETS_LARGE", 100);

    let (service_small, _wallet_ids_small) = setup_service_with_wallets(&rt, wallets_small);
    c.bench_function(
        &format!("wallet_service/list_wallets_in_memory/{wallets_small}"),
        |b| {
            b.iter(|| {
                rt.block_on(async {
                    let wallets = service_small
                        .list_wallets_in_memory()
                        .await
                        .expect("list_wallets_in_memory must succeed");
                    criterion::black_box(wallets);
                })
            })
        },
    );

    let (service_large, wallet_ids_large) = setup_service_with_wallets(&rt, wallets_large);
    let wallet_id = wallet_ids_large
        .last()
        .expect("at least one wallet id")
        .clone();

    c.bench_function(
        &format!("wallet_service/get_wallet_state/{wallets_large}"),
        |b| {
            b.iter(|| {
                rt.block_on(async {
                    let state = service_large
                        .get_wallet_state(&wallet_id)
                        .await
                        .expect("get_wallet_state must succeed");
                    criterion::black_box(state);
                })
            })
        },
    );

    // A tiny sanity pause so this file is not optimized into nothing in some builds.
    std::thread::sleep(Duration::from_millis(1));
}

criterion_group!(benches, wallet_service_benchmarks);
criterion_main!(benches);
