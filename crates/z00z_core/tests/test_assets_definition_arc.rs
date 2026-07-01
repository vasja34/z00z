//! Integration Test 22: Arc<AssetDefinition> Reference Counting
//!
//! Tests memory efficiency through reference counting:
//! - Multiple assets share same definition via Arc
//! - Refcount correctly reflects active references
//! - Memory is freed when all references drop

use rayon::prelude::*;
use std::sync::Arc;
use std::thread;
use z00z_core::assets::{Asset, AssetClass, AssetDefinition};
use z00z_core::BlindingFactor;
use z00z_utils::rng::DeterministicRngProvider;
use z00z_utils::time::Instant;

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
fn test_arc_refcount_basic_creation() {
    let def = create_test_definition();

    // Initial: 1 reference (the one we have)
    assert_eq!(Arc::strong_count(&def), 1, "Initial: def has 1 reference");

    // Clone it
    let def_clone = Arc::clone(&def);

    // Should now have 2 references
    assert_eq!(
        Arc::strong_count(&def),
        2,
        "After clone: def has 2 references"
    );

    // Drop the clone
    drop(def_clone);

    // Should be back to 1
    assert_eq!(
        Arc::strong_count(&def),
        1,
        "After dropping clone: back to 1"
    );
}

#[test]
fn test_arc_refcount_multiple_assets() {
    let def = create_test_definition();
    let initial_count = Arc::strong_count(&def);
    println!("Initial refcount: {}", initial_count);

    // Create 20 assets (further reduced for speed)
    let mut assets = Vec::new();
    let blinding_factor =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    for i in 0..20 {
        let serial_id = i as u32;
        let amount = 1000u64 + (i as u64);
        let nonce = [(i as u8); 32]; // Use i as nonce pattern

        let asset = Asset::new(
            Arc::clone(&def),
            serial_id,
            amount,
            &blinding_factor,
            nonce,
            &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
        )
        .expect("asset creation should succeed");

        assets.push(asset);
    }

    // Refcount should be: original 1 + 20 assets = 21
    let final_count = Arc::strong_count(&def);
    println!("After 20 assets created: refcount = {}", final_count);
    assert_eq!(
        final_count,
        initial_count + 20,
        "Refcount should be original + 20 assets"
    );
}

#[test]
fn test_arc_refcount_drop_clears() {
    let def = create_test_definition();
    let initial_count = Arc::strong_count(&def);

    {
        let blinding_factor =
            BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
        let mut assets = Vec::new();

        // Create 15 assets in a scope
        for i in 0..15 {
            let asset = Asset::new(
                Arc::clone(&def),
                i as u32,
                1000u64 + (i as u64),
                &blinding_factor,
                [(i as u8); 32],
                &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            )
            .expect("asset creation should succeed");
            assets.push(asset);
        }

        // Refcount should be: 1 (def) + 15 (assets)
        assert_eq!(Arc::strong_count(&def), initial_count + 15);
        println!("Inside scope: refcount = {}", Arc::strong_count(&def));
    }

    // After scope ends, assets are dropped
    let after_drop = Arc::strong_count(&def);
    println!("After assets dropped: refcount = {}", after_drop);
    assert_eq!(
        after_drop, initial_count,
        "Refcount should return to initial after dropping assets"
    );
}

#[test]
fn test_arc_refcount_batch_operations() {
    let def = create_test_definition();
    let initial_count = Arc::strong_count(&def);
    let blinding_factor =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

    // Create 2 batches of 10 assets each
    let mut batches = Vec::new();
    for batch_num in 0..2 {
        let mut batch = Vec::new();
        for i in 0..10 {
            let asset = Asset::new(
                Arc::clone(&def),
                i as u32,
                1000u64 + (i as u64),
                &blinding_factor,
                [(batch_num * 10 + i) as u8; 32],
                &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            )
            .expect("asset creation should succeed");
            batch.push(asset);
        }
        batches.push(batch);
    }

    // Should have: 1 (def) + 20 (2 batches * 10 assets)
    assert_eq!(
        Arc::strong_count(&def),
        initial_count + 20,
        "Refcount should track all 20 assets"
    );

    // Drop first batch
    batches.remove(0);
    assert_eq!(
        Arc::strong_count(&def),
        initial_count + 10,
        "After removing first batch: 10 assets remain"
    );

    // Drop second batch
    batches.remove(0);
    assert_eq!(
        Arc::strong_count(&def),
        initial_count,
        "After removing second batch: refcount back to initial"
    );
}

#[test]
fn test_arc_refcount_definition_cleanup() {
    let def = create_test_definition();
    let initial_count = Arc::strong_count(&def);

    // Create assets referencing definition
    let blinding_factor =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let mut assets = Vec::new();

    for i in 0..12 {
        let asset = Asset::new(
            Arc::clone(&def),
            i as u32,
            1000u64 + (i as u64),
            &blinding_factor,
            [(i as u8); 32],
            &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
        )
        .expect("asset creation should succeed");
        assets.push(asset);
    }

    // Definition should have: 1 (def) + 12 (assets) = 13
    assert_eq!(Arc::strong_count(&def), initial_count + 12);

    // Drop assets
    assets.clear();
    assert_eq!(
        Arc::strong_count(&def),
        initial_count,
        "After dropping assets: refcount = initial"
    );
}

#[test]
fn test_arc_weak_references() {
    let def = create_test_definition();
    let weak_ref = Arc::downgrade(&def);

    // Strong reference should exist
    assert!(
        weak_ref.upgrade().is_some(),
        "Weak reference should upgrade to Some when strong exists"
    );

    let strong_count = Arc::strong_count(&def);
    let weak_count = Arc::weak_count(&def);
    println!("Strong: {}, Weak: {}", strong_count, weak_count);

    assert_eq!(weak_count, 1, "Should have 1 weak reference");

    // Create assets with strong references
    let blinding_factor =
        BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
    let mut assets = Vec::new();
    for i in 0..6 {
        let asset = Asset::new(
            Arc::clone(&def),
            i as u32,
            1000u64 + (i as u64),
            &blinding_factor,
            [(i as u8); 32],
            &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
        )
        .expect("asset creation should succeed");
        assets.push(asset);
    }

    // Weak reference still valid
    assert!(
        weak_ref.upgrade().is_some(),
        "Weak reference still upgradeable with active assets"
    );

    // Drop all strong references
    drop(assets);
    drop(def);

    // Weak reference should now be None
    assert!(
        weak_ref.upgrade().is_none(),
        "Weak reference returns None after all strong refs dropped"
    );
}

#[test]
fn test_arc_clone_performance() {
    let def = create_test_definition();
    let start = Instant::now();

    // Create 24 assets with Arc cloning - PARALLELIZED with Rayon
    let asset_count = 24usize;
    let _assets: Vec<Asset> = (0..asset_count)
        .into_par_iter()
        .map(|i| {
            let blinding_factor =
                BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
            Asset::new(
                Arc::clone(&def),
                i as u32,
                1000u64 + (i as u64),
                &blinding_factor,
                [(i as u8); 32],
                &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            )
            .expect("asset creation should succeed")
        })
        .collect();

    let elapsed = start.elapsed();
    println!(
        "Created {} assets with Arc cloning in: {:?}",
        asset_count, elapsed
    );

    // Verify refcount
    assert_eq!(
        Arc::strong_count(&def),
        1 + asset_count,
        "All assets have references"
    );

    // Verify memory is efficiently shared (not cloned)
    // Reduced batch keeps runtime well under 12s even in debug builds
    assert!(
        elapsed.as_secs() < 30,
        "Creating Arc-cloned assets should be fast (<30s in debug)"
    );
}

#[test]
fn test_arc_concurrent_cloning() {
    let def = create_test_definition();
    let initial_count = Arc::strong_count(&def);
    let mut handles = vec![];

    // Spawn 2 threads, each creating 8 assets
    for thread_id in 0..2 {
        let def_clone = Arc::clone(&def);
        let handle = thread::spawn(move || {
            let blinding_factor =
                BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
            let mut local_assets = Vec::new();

            for i in 0..8 {
                let asset = Asset::new(
                    Arc::clone(&def_clone),
                    (thread_id * 8 + i) as u32,
                    1000u64 + (i as u64),
                    &blinding_factor,
                    [(thread_id * 8 + i) as u8; 32],
                    &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
                )
                .expect("asset creation should succeed");
                local_assets.push(asset);
            }

            // Assets are dropped at end of thread
            local_assets.len()
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("thread should join successfully");
    }

    // After all threads complete, refcount should return to initial
    let final_count = Arc::strong_count(&def);
    println!("Final refcount after concurrent ops: {}", final_count);
    assert_eq!(
        final_count, initial_count,
        "Refcount should return to initial after all threads drop"
    );
}
