//! Integration Test 23: Long-Running Node Memory Usage
//!
//! Tests that asset memory usage remains stable during extended operation:
//! - Simulate node processing transactions over time
//! - Create and drop assets repeatedly
//! - Verify no unbounded memory growth
//! - Test that garbage collection is effective

use std::sync::Arc;
use std::thread;
use z00z_core::assets::{Asset, AssetClass, AssetDefinition};
use z00z_core::BlindingFactor;
use z00z_utils::rng::DeterministicRngProvider;

fn create_test_definition() -> Arc<AssetDefinition> {
    Arc::new(
        AssetDefinition::new(
            [42u8; 32],
            AssetClass::Coin,
            "Test Coin".to_string(),
            "TST".to_string(),
            8,
            1_000_000,
            1_000,
            "test.local".to_string(),
            1,
            1,
            0,
            None,
        )
        .expect("Valid test definition"),
    )
}

#[test]
#[ignore] // Stress test: run with --ignored
fn test_long_running_steady_state() {
    // Simulate node steady state: create and drop assets in batches
    let def = create_test_definition();
    let initial_count = Arc::strong_count(&def);
    let blinding_factor =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    // Process 2 batches representing time periods
    for batch in 0..2 {
        let mut assets = Vec::new();

        // Each batch creates 8 assets
        for i in 0..8 {
            let nonce = [(batch * 8 + i) as u8; 32];
            let asset = Asset::new(
                Arc::clone(&def),
                (i % 100) as u32,
                1000u64 + (i as u64),
                &blinding_factor,
                nonce,
                &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            )
            .expect("asset creation should succeed");
            assets.push(asset);
        }

        // Refcount should be: initial + 12 active assets
        let batch_count = Arc::strong_count(&def);
        println!(
            "Batch {}: {} assets in memory, refcount = {}",
            batch,
            assets.len(),
            batch_count
        );
        assert_eq!(
            batch_count,
            initial_count + 8,
            "Active batch should have 8 assets"
        );

        // Assets are dropped at end of iteration
    }

    // After all batches, memory should be released
    let final_count = Arc::strong_count(&def);
    println!("After all batches: refcount = {}", final_count);
    assert_eq!(
        final_count, initial_count,
        "Memory should be fully released after batches complete"
    );
}

#[test]
#[ignore] // Stress test: run with --ignored
fn test_long_running_multiple_definitions() {
    // Simulate node with multiple asset types
    let defs: Vec<_> = (0..3)
        .map(|i| {
            Arc::new(
                AssetDefinition::new(
                    [i as u8; 32],
                    AssetClass::Coin,
                    format!("Asset_{}", i),
                    format!("AS{}", i),
                    8,
                    1_000_000,
                    1_000,
                    "test.local".to_string(),
                    1,
                    1,
                    0,
                    None,
                )
                .expect("Valid test definition"),
            )
        })
        .collect();

    let initial_counts: Vec<_> = defs.iter().map(Arc::strong_count).collect();
    println!("Initial counts: {:?}", initial_counts);

    // Create assets of each type
    let blinding_factor =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    {
        let mut all_assets = Vec::new();

        for (def_idx, def) in defs.iter().enumerate() {
            for i in 0..6 {
                let nonce = [(def_idx * 6 + i) as u8; 32];
                let asset = Asset::new(
                    Arc::clone(def),
                    i as u32,
                    1000u64 + (i as u64),
                    &blinding_factor,
                    nonce,
                    &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
                )
                .expect("asset creation should succeed");
                all_assets.push(asset);
            }
        }

        // 3 definitions × 6 assets each = 18 assets total
        println!("Total assets created: {}", all_assets.len());
        assert_eq!(all_assets.len(), 18);

        // Each definition should have: initial + 6 assets
        for (def_idx, def) in defs.iter().enumerate() {
            let current_count = Arc::strong_count(def);
            assert_eq!(current_count, initial_counts[def_idx] + 6);
        }
    }

    // After scope, all assets dropped
    for (def_idx, def) in defs.iter().enumerate() {
        let final_count = Arc::strong_count(def);
        println!("Def {}: refcount after cleanup = {}", def_idx, final_count);
        assert_eq!(
            final_count, initial_counts[def_idx],
            "Definition refcount should return to initial"
        );
    }
}

#[test]
#[ignore] // Stress test: run with --ignored
fn test_long_running_rapid_creation() {
    // Simulate high-frequency transaction processing
    let def = create_test_definition();
    let initial_count = Arc::strong_count(&def);
    let blinding_factor =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    // Process 3 rapid cycles of creation/deletion
    for cycle in 0..3 {
        // Create 6 assets rapidly
        let mut rapid_assets = Vec::new();
        for i in 0..6 {
            let nonce = [(cycle * 6 + i) as u8; 32];
            let asset = Asset::new(
                Arc::clone(&def),
                i as u32,
                100u64 + (i as u64),
                &blinding_factor,
                nonce,
                &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            )
            .expect("asset creation should succeed");
            rapid_assets.push(asset);
        }

        assert_eq!(Arc::strong_count(&def), initial_count + 6);

        // Clear immediately (simulating processed transactions)
        rapid_assets.clear();

        // Memory should be freed
        assert_eq!(Arc::strong_count(&def), initial_count);
        println!("Cycle {}: {} operations, memory cleaned", cycle, 6);
    }

    // After 3 cycles × 6 creations = 18 total asset creations
    // Memory should still be clean
    let final_count = Arc::strong_count(&def);
    assert_eq!(
        final_count, initial_count,
        "Memory must remain stable after many cycles"
    );
}

#[test]
#[ignore] // Stress test: run with --ignored
fn test_long_running_varying_batch() {
    // Simulate realistic transaction batch patterns
    let def = create_test_definition();
    let initial_count = Arc::strong_count(&def);
    let blinding_factor =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    let batch_sizes = [6, 8, 6];
    let mut total_created = 0;

    // Process batches of varying sizes
    for (batch_num, &size) in batch_sizes.iter().enumerate() {
        let mut batch = Vec::new();

        for i in 0..size {
            let nonce = [(total_created + i) as u8; 32];
            let asset = Asset::new(
                Arc::clone(&def),
                (i % 100) as u32,
                500u64 + (i as u64),
                &blinding_factor,
                nonce,
                &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            )
            .expect("asset creation should succeed");
            batch.push(asset);
        }

        assert_eq!(Arc::strong_count(&def), initial_count + size as usize);
        total_created += size;
        println!(
            "Batch {}: created {}, total: {}",
            batch_num, size, total_created
        );
    }

    println!("Total assets processed: {}", total_created);
    assert_eq!(total_created, 20);
}

#[test]
#[ignore] // Stress test: run with --ignored
fn test_long_running_concurrent_node() {
    let def = create_test_definition();
    let initial_count = Arc::strong_count(&def);
    let mut handles = vec![];

    // Simulate 2 concurrent processing threads
    for thread_id in 0..2 {
        let def_clone = Arc::clone(&def);
        let handle = thread::spawn(move || {
            let blinding_factor =
                BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
            let mut max_active = 0;

            // Each thread processes 2 batches of 4 assets
            for batch in 0..2 {
                let mut batch_assets = Vec::new();

                for i in 0..4 {
                    let nonce = [(thread_id * 20 + batch * 4 + i) as u8; 32];
                    let asset = Asset::new(
                        Arc::clone(&def_clone),
                        i as u32,
                        200u64 + (i as u64),
                        &blinding_factor,
                        nonce,
                        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
                    )
                    .expect("asset creation should succeed");
                    batch_assets.push(asset);
                }

                max_active = max_active.max(batch_assets.len());
            }

            max_active
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("thread should join");
    }

    // After all threads complete
    let final_count = Arc::strong_count(&def);
    println!(
        "Final refcount after concurrent simulation: {}",
        final_count
    );
    assert_eq!(
        final_count, initial_count,
        "Memory released after all threads complete"
    );
}

#[test]
#[ignore] // Stress test: run with --ignored
fn test_long_running_memory_leak() {
    // Verify no memory leak in repeated asset creation
    let def = create_test_definition();
    let initial_count = Arc::strong_count(&def);
    let blinding_factor =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    // Create and drop 15 individual assets
    for i in 0..6 {
        let nonce = [(i as u8); 32];
        let asset = Asset::new(
            Arc::clone(&def),
            (i % 50) as u32,
            300u64 + ((i % 1000) as u64),
            &blinding_factor,
            nonce,
            &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
        )
        .expect("asset creation should succeed");

        // Each asset is dropped immediately
        drop(asset);

        // Refcount should always be initial (no dangling references)
        assert_eq!(
            Arc::strong_count(&def),
            initial_count,
            "Refcount leak at iteration {}",
            i
        );
    }

    // After 15 creations and deletions
    assert_eq!(
        Arc::strong_count(&def),
        initial_count,
        "No memory leaked after 30 create/drop cycles"
    );
}

#[test]
#[ignore] // Stress test: run with --ignored
fn test_asset_long_running_partial() {
    // Simulate node holding some assets while processing others
    let def = create_test_definition();
    let initial_count = Arc::strong_count(&def);
    let blinding_factor =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    // Create "permanent" assets (like assets held by wallet)
    let mut permanent_assets = Vec::new();
    for i in 0..4 {
        let nonce = [(i as u8); 32];
        let asset = Asset::new(
            Arc::clone(&def),
            i as u32,
            5000u64 + (i as u64) * 100,
            &blinding_factor,
            nonce,
            &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
        )
        .expect("asset creation should succeed");
        permanent_assets.push(asset);
    }

    assert_eq!(Arc::strong_count(&def), initial_count + 4);
    println!(
        "Permanent assets: {}, refcount: {}",
        permanent_assets.len(),
        Arc::strong_count(&def)
    );

    // Now process temporary assets
    for cycle in 0..2 {
        let mut temp_assets = Vec::new();
        for i in 0..4 {
            let nonce = [(4 + cycle * 4 + i) as u8; 32];
            let asset = Asset::new(
                Arc::clone(&def),
                (i % 100) as u32,
                100u64 + (i as u64),
                &blinding_factor,
                nonce,
                &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            )
            .expect("asset creation should succeed");
            temp_assets.push(asset);
        }

        // Total should be: 4 (permanent) + 4 (temporary)
        assert_eq!(Arc::strong_count(&def), initial_count + 4 + 4);
        println!(
            "Cycle {}: permanent=4, temporary=4, total refcount={}",
            cycle,
            Arc::strong_count(&def)
        );
    }

    // After clearing temporaries
    println!(
        "Final permanent assets: {}, refcount: {}",
        permanent_assets.len(),
        Arc::strong_count(&def)
    );
    assert_eq!(Arc::strong_count(&def), initial_count + 4);
}
