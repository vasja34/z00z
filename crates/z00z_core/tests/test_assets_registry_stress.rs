// crates/z00z_core/tests/test_assets_registry_stress.rs
//
//! Concurrent stress test for AssetDefinitionRegistry
//!
//! Tests lock ordering and deadlock prevention under heavy concurrent load

use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use z00z_core::assets::definition::AssetDefinition;
use z00z_core::assets::registry::AssetDefinitionRegistry;
use z00z_core::assets::AssetClass;
use z00z_utils::prelude::{NoopLogger, NoopMetrics, SystemTimeProvider};
use z00z_utils::time::Instant;

/// Create test observability instances (logger, metrics, time)
fn create_test_observability() -> (
    Arc<dyn z00z_utils::logger::Logger>,
    Arc<dyn z00z_utils::metrics::MetricsSink>,
    Arc<dyn z00z_utils::time::TimeProvider>,
) {
    (
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    )
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

#[test]
fn test_registry_concurrent_stress_no() {
    let (logger, metrics, time_provider) = create_test_observability();
    let registry = Arc::new(AssetDefinitionRegistry::new(logger, metrics, time_provider));
    let mut initial_ids = Vec::new();

    // Pre-populate with some definitions
    for i in 0..10 {
        let mut asset_id = [0u8; 32];
        asset_id[0] = i;

        let def = AssetDefinition::new(
            asset_id,
            AssetClass::Token,
            format!("Token {}", i),
            format!("TK{}", i),
            8,
            1000 + i as u32,
            1_000_000,
            format!("token{}.example", i),
            1,
            1,
            0,
            None,
        )
        .unwrap();

        initial_ids.push(def.id);
        registry.insert(def).unwrap();
    }

    const NUM_THREADS: usize = 20;
    const ITERATIONS: usize = 500;

    println!("🔄 Starting concurrent stress test:");
    println!("   Threads: {}", NUM_THREADS);
    println!("   Iterations per thread: {}", ITERATIONS);
    println!("   Total operations: {}", NUM_THREADS * ITERATIONS);

    let start = Instant::now();
    let completion_timeout = Duration::from_secs(env_u64("Z00Z_REGISTRY_STRESS_TIMEOUT_SECS", 180));
    let (done_tx, done_rx) = mpsc::channel();

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let reg = Arc::clone(&registry);
            let initial_ids = initial_ids.clone();
            let done_tx = done_tx.clone();
            thread::spawn(move || {
                for iteration in 0..ITERATIONS {
                    let op = (thread_id + iteration) % 5;

                    match op {
                        // Read operations (50% of load)
                        0 => {
                            let asset_id = initial_ids[(thread_id + iteration) % initial_ids.len()];
                            let _ = reg.get(&asset_id);
                        }
                        1 => {
                            let _ = reg.get_version();
                        }
                        2 => {
                            let _ = reg.len();
                        }

                        // Write operations (40% of load)
                        3 => {
                            let mut asset_id = [0u8; 32];
                            asset_id[0] = ((thread_id + iteration) % 10) as u8;
                            asset_id[1] = thread_id as u8;

                            let def = AssetDefinition::new(
                                asset_id,
                                AssetClass::Token,
                                format!("Thread {} Asset {}", thread_id, iteration),
                                format!("T{}A{}", thread_id, iteration),
                                8,
                                2000 + thread_id as u32,
                                2_000_000,
                                format!("thread{}_asset{}.test", thread_id, iteration),
                                1,
                                1,
                                0,
                                None,
                            )
                            .unwrap();

                            let _ = reg.insert(def);
                        }

                        // Snapshot operations (10% of load)
                        4 => {
                            if let Ok(snapshot) = reg.create_snapshot() {
                                // Simulate network transmission delay
                                thread::sleep(Duration::from_micros(10));

                                // Restore snapshot (tests lock ordering)
                                let _ = reg.update_from_snapshot(snapshot);
                            }
                        }

                        _ => unreachable!(),
                    }

                    // Add tiny random delay to increase contention
                    if iteration % 100 == 0 {
                        thread::sleep(Duration::from_micros(1));
                    }
                }

                let _ = done_tx.send(());
            })
        })
        .collect();
    drop(done_tx);

    let joiner = thread::spawn(move || -> Result<(), String> {
        for (i, handle) in handles.into_iter().enumerate() {
            handle
                .join()
                .map_err(|e| format!("Thread {} panicked: {:?}", i, e))?;
        }
        Ok(())
    });

    for completed in 1..=NUM_THREADS {
        match done_rx.recv_timeout(completion_timeout) {
            Ok(()) => {
                if completed == NUM_THREADS {
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                panic!(
                    "Concurrent registry stress did not complete within {:?} (completed {}/{})",
                    completion_timeout,
                    completed - 1,
                    NUM_THREADS
                );
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }

    match joiner.join() {
        Ok(Ok(())) => {}
        Ok(Err(err)) => panic!("{err}"),
        Err(err) => panic!("Join coordinator panicked: {:?}", err),
    }

    let duration = start.elapsed();

    println!("✅ Stress test completed successfully:");
    println!("   Threads completed: {}/{}", NUM_THREADS, NUM_THREADS);
    println!("   Duration: {:?}", duration);
    println!(
        "   Operations/sec: {:.0}",
        (NUM_THREADS * ITERATIONS) as f64 / duration.as_secs_f64()
    );

    // Verify registry is still functional
    let version = registry.get_version().unwrap();
    let len = registry.len().unwrap();
    println!("   Final registry state: version={}, size={}", version, len);

    assert!(
        len >= 10,
        "Registry should still contain at least initial 10 definitions"
    );
}

#[test]
fn test_concurrent_batch_operations() {
    let (logger, metrics, time_provider) = create_test_observability();
    let registry = Arc::new(AssetDefinitionRegistry::new(logger, metrics, time_provider));

    const NUM_THREADS: usize = 10;
    const BATCH_SIZE: usize = 20;

    println!("🔄 Starting concurrent batch operations test:");
    println!("   Threads: {}", NUM_THREADS);
    println!("   Batch size: {}", BATCH_SIZE);

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let reg = Arc::clone(&registry);
            thread::spawn(move || {
                let mut batch = Vec::new();
                let mut batch_ids = Vec::new();

                for i in 0..BATCH_SIZE {
                    let mut asset_id = [0u8; 32];
                    asset_id[0] = thread_id as u8;
                    asset_id[1] = i as u8;

                    let def = AssetDefinition::new(
                        asset_id,
                        AssetClass::Token,
                        format!("Batch Asset T{} I{}", thread_id, i),
                        format!("B{}A{}", thread_id, i),
                        8,
                        3000 + (thread_id * 100) as u32,
                        3_000_000,
                        format!("batch{}_asset{}.test", thread_id, i),
                        1,
                        1,
                        0,
                        None,
                    )
                    .unwrap();

                    batch_ids.push(def.id);
                    batch.push(def);
                }

                // Insert batch (tests write lock)
                reg.insert_batch(batch).unwrap();

                // Immediately read (tests read lock after write)
                for asset_id in batch_ids {
                    assert!(
                        reg.get(&asset_id).unwrap().is_some(),
                        "Batch asset should be in registry"
                    );
                }
            })
        })
        .collect();

    for (i, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(_) => {}
            Err(e) => {
                panic!("Batch thread {} panicked: {:?}", i, e);
            }
        }
    }

    let len = registry.len().unwrap();
    println!("✅ Batch operations test passed: {} assets inserted", len);

    assert_eq!(
        len,
        NUM_THREADS * BATCH_SIZE,
        "All batch assets should be in registry"
    );
}
