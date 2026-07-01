//! Phase 2, Test 16: Registry Lookup Performance Benchmarks
//!
//! Purpose: Measure performance of registry lookup operations:
//! - Registry.get() performance vs registry size (10, 100, 200 definitions)
//! - Single-threaded lookup throughput
//! - Multi-threaded lookup contention and RwLock contention
//! - Hit vs miss performance
//!
//! Real Structures:
//! - AssetDefinitionRegistry with RwLock
//! - AssetDefinition with real IDs
//! - Real lookups with Option<Arc<>> returns
//!
//! Success Criteria:
//! - All tests complete in < 15 seconds total
//! - Lookup time < 10μs per operation (any registry size)
//! - No panic on concurrent access

use std::sync::Arc;
use std::thread;
use z00z_core::assets::{AssetClass, AssetDefinition, AssetDefinitionRegistry};
use z00z_utils::time::Instant;

/// Create test asset definition with specific ID
fn create_definition_with_id(id_byte: u8, index: u32) -> Arc<AssetDefinition> {
    let mut id = [0u8; 32];
    id[0] = id_byte;
    id[1] = ((index >> 8) & 0xFF) as u8;
    id[2] = (index & 0xFF) as u8;

    Arc::new(
        AssetDefinition::new(
            id,
            AssetClass::Coin,
            format!("TestAsset{}", index),
            format!("TST{}", index),
            8,
            1_000_000,
            1_000_000,
            "test.local".to_string(),
            1,
            1,
            0,
            None,
        )
        .expect("Valid definition"),
    )
}

/// Populate registry with N definitions, return their IDs
fn populate_registry(registry: &AssetDefinitionRegistry, count: usize) -> Vec<[u8; 32]> {
    let mut ids = Vec::new();

    for i in 0..count {
        let def = create_definition_with_id(42, i as u32);
        let id = def.id;
        let _ = registry.insert((*def).clone());
        ids.push(id);
    }

    ids
}

// ============ TEST 1: Small Registry (10 definitions) ============

#[test]
fn test_registry_lookup_small_10() {
    let registry = crate::assets::fixtures::create_test_registry();
    let ids = populate_registry(&registry, 10);

    let start = Instant::now();
    for _ in 0..500 {
        for &id in &ids {
            let _ = registry.get(&id); // 500 × 10 = 5K lookups
        }
    }
    let elapsed = start.elapsed();

    println!(
        "Small registry (10 defs): {} lookups in {:?} ({:.2}μs/lookup)",
        5000,
        elapsed,
        elapsed.as_micros() as f64 / 5000.0
    );

    assert!(elapsed.as_millis() < 100, "5K lookups should be < 100ms");
}

// ============ TEST 2: Medium Registry (100 definitions) ============

#[test]
fn test_registry_lookup_medium_100() {
    let registry = crate::assets::fixtures::create_test_registry();
    let ids = populate_registry(&registry, 100);

    let start = Instant::now();
    for _ in 0..500 {
        for &id in &ids {
            let _ = registry.get(&id); // 500 × 100 = 50K lookups
        }
    }
    let elapsed = start.elapsed();

    println!(
        "Medium registry (100 defs): {} lookups in {:?} ({:.2}μs/lookup)",
        50000,
        elapsed,
        elapsed.as_micros() as f64 / 50000.0
    );

    assert!(elapsed.as_millis() < 500, "50K lookups should be < 500ms");
}

// ============ TEST 3: Large Registry (200 definitions) ============

#[test]
fn test_registry_lookup_large_200() {
    let registry = crate::assets::fixtures::create_test_registry();
    let ids = populate_registry(&registry, 200);

    let start = Instant::now();
    for _ in 0..250 {
        for &id in &ids {
            let _ = registry.get(&id); // 250 × 200 = 50K lookups
        }
    }
    let elapsed = start.elapsed();

    println!(
        "Large registry (200 defs): {} lookups in {:?} ({:.2}μs/lookup)",
        50000,
        elapsed,
        elapsed.as_micros() as f64 / 50000.0
    );

    assert!(elapsed.as_millis() < 500, "50K lookups should be < 500ms");
}

// ============ TEST 4: Multi-threaded Contention ============

#[test]
fn test_registry_lookup_multithreaded_contention() {
    let registry = Arc::new(crate::assets::fixtures::create_test_registry());
    let ids = Arc::new(populate_registry(&registry, 100));

    let start = Instant::now();
    let mut handles = vec![];

    for _ in 0..4 {
        let registry = registry.clone();
        let ids = ids.clone();

        let handle = thread::spawn(move || {
            for _ in 0..300 {
                for &id in ids.iter() {
                    let _ = registry.get(&id); // 4 threads × 300 × 100 = 120K lookups
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    let elapsed = start.elapsed();

    println!(
        "Multi-threaded (4 threads × 100 defs): {} lookups in {:?} ({:.2}μs/lookup)",
        120000,
        elapsed,
        elapsed.as_micros() as f64 / 120000.0
    );

    assert!(elapsed.as_millis() < 1200, "120K lookups should be < 1.2s");
}

// ============ TEST 5: Hit vs Miss Ratio ============

#[test]
fn test_registry_lookup_hit_miss() {
    let registry = crate::assets::fixtures::create_test_registry();
    let ids = populate_registry(&registry, 100);

    // Hits
    let start = Instant::now();
    for _ in 0..500 {
        for &id in &ids {
            let _ = registry.get(&id).ok().flatten().is_some(); // Should all be Some
        }
    }
    let hit_time = start.elapsed();

    // Misses (IDs that don't exist)
    let mut miss_ids = vec![];
    for i in 0..100 {
        let mut id = [0u8; 32];
        id[0] = 99;
        id[1] = ((i >> 8) & 0xFF) as u8;
        id[2] = (i & 0xFF) as u8;
        miss_ids.push(id);
    }

    let start = Instant::now();
    for _ in 0..500 {
        for &id in &miss_ids {
            let _ = registry.get(&id).ok().flatten().is_none(); // Should all be None
        }
    }
    let miss_time = start.elapsed();

    println!(
        "Hit vs Miss (100 defs, 50K each):\n  Hits: {:?} ({:.2}μs/lookup)\n  Misses: {:?} ({:.2}μs/lookup)",
        hit_time,
        hit_time.as_micros() as f64 / 50000.0,
        miss_time,
        miss_time.as_micros() as f64 / 50000.0
    );

    assert!(hit_time.as_millis() < 500, "50K hits should be < 500ms");
    assert!(miss_time.as_millis() < 500, "50K misses should be < 500ms");
}

// ============ TEST 6: Scaling Analysis ============

#[test]
fn test_registry_scaling() {
    let sizes = [10, 50, 100, 200];

    for size in sizes {
        let registry = crate::assets::fixtures::create_test_registry();
        let ids = populate_registry(&registry, size);

        let lookups_per_test = 500;
        let start = Instant::now();

        for _ in 0..lookups_per_test {
            for &id in &ids {
                let _ = registry.get(&id);
            }
        }

        let elapsed = start.elapsed();
        let total_lookups = lookups_per_test * size;

        println!(
            "Scaling test: {} definitions, {} lookups in {:?} ({:.2}μs/lookup)",
            size,
            total_lookups,
            elapsed,
            elapsed.as_micros() as f64 / total_lookups as f64
        );
    }
}

// ============ TEST 7: Sequential vs Random Access ============

#[test]
fn test_registry_access_patterns() {
    let registry = crate::assets::fixtures::create_test_registry();
    let ids = populate_registry(&registry, 100);

    // Sequential access
    let start = Instant::now();
    for _ in 0..500 {
        for &id in &ids {
            let _ = registry.get(&id);
        }
    }
    let sequential = start.elapsed();

    // Reverse (pseudo-random) access
    let start = Instant::now();
    for _ in 0..500 {
        for i in (0..ids.len()).rev() {
            let _ = registry.get(&ids[i]);
        }
    }
    let reverse = start.elapsed();

    println!(
        "Access patterns (100 defs, 500 iterations each):\n  Sequential: {:?} ({:.2}μs/lookup)\n  Reverse: {:?} ({:.2}μs/lookup)",
        sequential,
        sequential.as_micros() as f64 / 50000.0,
        reverse,
        reverse.as_micros() as f64 / 50000.0
    );

    assert!(sequential.as_millis() < 500, "Sequential should be < 500ms");
    assert!(reverse.as_millis() < 500, "Reverse should be < 500ms");
}

// ============ TEST 8: Concurrent Registry State ============

#[test]
fn test_registry_concurrent_state() {
    let registry = Arc::new(crate::assets::fixtures::create_test_registry());

    // Populate in thread 1
    let reg1 = registry.clone();
    let h1 = thread::spawn(move || {
        populate_registry(&reg1, 50);
    });

    // Lookup in thread 2 (while populating)
    let reg2 = registry.clone();
    let h2 = thread::spawn(move || {
        thread::sleep(std::time::Duration::from_millis(1));

        let start = Instant::now();
        let mut found = 0;
        for i in 0..10000 {
            let mut id = [0u8; 32];
            id[0] = 42;
            id[1] = ((i >> 8) & 0xFF) as u8;
            id[2] = (i & 0xFF) as u8;

            if reg2.get(&id).ok().flatten().is_some() {
                found += 1;
            }
        }
        let elapsed = start.elapsed();

        (found, elapsed)
    });

    h1.join().expect("Thread 1 panicked");
    let (found, elapsed) = h2.join().expect("Thread 2 panicked");

    println!(
        "Concurrent state: {} found in 10K lookups, {:?} ({:.2}μs/lookup)",
        found,
        elapsed,
        elapsed.as_micros() as f64 / 10000.0
    );

    assert!(elapsed.as_millis() < 100, "10K lookups should be < 100ms");
}
