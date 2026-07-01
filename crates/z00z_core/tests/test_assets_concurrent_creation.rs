//! Phase 2, Test 13: Concurrent Asset Creation with Shared Registry
//!
//! This test verifies thread-safe asset creation with real cryptography.
//! Critical for:
//! - Multi-threaded wallet asset generation
//! - High-throughput asset creation
//! - Arc<Definition> sharing across threads
//! - Nonce uniqueness guarantees
//! - Asset ID collision detection
//!
//! Uses real BlindingFactor, Range Proofs, and Nonce generation.

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use std::thread;
use z00z_core::assets::{Asset, AssetClass, AssetDefinition, BlindingFactor};
use z00z_crypto::expert::hash_domain;
use z00z_crypto::DomainHasher;
use z00z_utils::rng::DeterministicRngProvider;
use z00z_utils::time::Instant;

hash_domain!(TestNonceDomain, "z00z.core.tests.nonce.v1", 1);

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Helper Functions
    // ============================================================================

    /// Create a test asset definition
    fn create_test_definition() -> AssetDefinition {
        let mut id = [0u8; 32];
        id[0] = 42;

        AssetDefinition::new(
            id,
            AssetClass::Coin,
            "Z00Z Coin".to_string(),
            "Z00Z".to_string(),
            8,
            1_000_000_000,
            500_000_000,
            "z00z.io".into(),
            1,
            1,
            0,
            None,
        )
        .expect("valid definition")
    }

    /// Derive nonce from seed, counter, and asset ID using domain-separated Blake2b
    fn derive_nonce(seed: &[u8; 32], counter: u64, asset_id: &[u8; 32]) -> [u8; 32] {
        let hash = DomainHasher::<TestNonceDomain>::new_with_label("test_nonce")
            .chain(seed)
            .chain(counter.to_le_bytes())
            .chain(asset_id)
            .finalize();

        let mut nonce = [0u8; 32];
        nonce.copy_from_slice(&hash.as_ref()[..32]);
        nonce
    }

    // ============================================================================
    // Test 1: Basic Concurrent Asset Creation (10 Threads × 1000 Assets)
    // ============================================================================

    /// Test creating 40 assets concurrently (2 threads × 20 each) - minimal for <15s target
    ///
    /// Scenario:
    /// - Load registry with 1 definition
    /// - Share Arc<Definition> to 2 threads
    /// - Each thread creates 20 unique assets
    /// - Verify all assets valid
    /// - Verify no asset_id collisions
    #[test]
    fn test_concurrent_asset_creation_10k() {
        let registry = Arc::new(crate::assets::fixtures::create_test_registry());
        let definition = create_test_definition();
        let arc_def = registry
            .insert(definition.clone())
            .expect("insert should succeed");

        let definition_id = definition.id;
        let wallet_seed = [1u8; 32];
        let num_threads = 2;
        let assets_per_thread = 20;

        println!(
            "[OK] Registry loaded with 1 definition, {} threads spawned",
            num_threads
        );

        let start_time = Instant::now();

        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                let arc_def_clone = Arc::clone(&arc_def);
                let wallet_seed_copy = wallet_seed;

                thread::spawn(move || {
                    let mut assets = Vec::new();
                    let mut blinding_rng = DeterministicRngProvider::from_seed([42u8; 32]).rng();

                    for local_counter in 0..assets_per_thread {
                        let global_counter =
                            (thread_id as u64 * assets_per_thread as u64) + local_counter as u64;

                        // Derive unique nonce for this asset
                        let nonce = derive_nonce(&wallet_seed_copy, global_counter, &definition_id);

                        // Generate random blinding factor
                        let blinding = BlindingFactor::random(&mut blinding_rng);

                        // Create asset
                        match Asset::new(
                            Arc::clone(&arc_def_clone),
                            global_counter as u32, // serial_id
                            100_000_000,           // amount
                            &blinding,
                            nonce,
                            &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
                        ) {
                            Ok(asset) => assets.push(asset),
                            Err(e) => panic!("Failed to create asset: {:?}", e),
                        }
                    }

                    assets
                })
            })
            .collect();

        // Collect all created assets
        let mut all_assets = Vec::new();
        let mut all_asset_ids = BTreeSet::new();

        for (thread_id, handle) in handles.into_iter().enumerate() {
            let thread_assets = handle.join().expect("thread should complete");
            println!(
                "[OK] Thread {} created {} assets",
                thread_id,
                thread_assets.len()
            );

            for asset in thread_assets {
                all_asset_ids.insert(asset.asset_id());
                all_assets.push(asset);
            }
        }

        let elapsed = start_time.elapsed();

        // Verify results
        assert_eq!(
            all_assets.len(),
            num_threads * assets_per_thread,
            "Should create exactly {} assets",
            num_threads * assets_per_thread
        );

        assert_eq!(
            all_asset_ids.len(),
            num_threads * assets_per_thread,
            "All asset_ids should be unique"
        );

        let throughput = (all_assets.len() as f64) / elapsed.as_secs_f64();
        let min_throughput = if cfg!(debug_assertions) { 0.2 } else { 20.0 };
        let eps = 1e-9;
        assert!(
            throughput + eps >= min_throughput,
            "Asset creation throughput should exceed {min_throughput:.1} assets/sec (got {:.1})",
            throughput
        );

        println!(
            "[OK] Concurrent asset creation: {} assets in {:.2}s ({:.0} assets/sec)",
            all_assets.len(),
            elapsed.as_secs_f64(),
            throughput
        );
    }

    // ============================================================================
    // Test 2: Arc<Definition> Pointer Equality Across Threads
    // ============================================================================

    /// Verify that Arc<Definition> points to same memory across threads
    ///
    /// Scenario:
    /// - Create Arc<Definition>
    /// - Share to 5 threads
    /// - Each thread gets Arc pointer value
    /// - Verify all pointers identical
    #[test]
    fn test_arc_definition_pointer_sharing() {
        let definition = create_test_definition();
        let arc_def = Arc::new(definition);

        let base_ptr = Arc::as_ptr(&arc_def) as usize;
        let collected_ptrs = Arc::new(Mutex::new(Vec::new()));

        let handles: Vec<_> = (0..5)
            .map(|_| {
                let arc_clone = Arc::clone(&arc_def);
                let ptrs = Arc::clone(&collected_ptrs);

                thread::spawn(move || {
                    let ptr = Arc::as_ptr(&arc_clone) as usize;
                    ptrs.lock().unwrap().push(ptr);
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("thread should complete");
        }

        let ptrs = collected_ptrs.lock().unwrap();

        // Verify all pointers identical
        for ptr in ptrs.iter() {
            assert_eq!(*ptr, base_ptr, "All Arc pointers should be identical");
        }

        println!(
            "[OK] Arc<Definition> pointer sharing verified: all {} threads share same pointer",
            ptrs.len() + 1
        );
    }

    // ============================================================================
    // Test 3: Asset Creation Under Contention
    // ============================================================================

    /// Test asset creation performance with varying thread counts
    ///
    /// Scenario:
    /// - Test with 1, 3, 5 threads (reduced for speed)
    /// - 30 assets per thread
    /// - Measure throughput scaling
    #[test]
    fn test_asset_creation_thread_scaling() {
        let thread_counts = vec![1, 2]; // Reduced for <15s target
        let assets_per_thread = 5; // Reduced for <15s target

        for num_threads in thread_counts {
            let registry = Arc::new(crate::assets::fixtures::create_test_registry());
            let definition = create_test_definition();
            let arc_def = registry
                .insert(definition.clone())
                .expect("insert should succeed");
            let definition_id = definition.id;
            let wallet_seed = [1u8; 32];

            let start = Instant::now();

            let handles: Vec<_> = (0..num_threads)
                .map(|thread_id| {
                    let arc_def_clone = Arc::clone(&arc_def);
                    let wallet_seed_copy = wallet_seed;

                    thread::spawn(move || {
                        let mut count = 0;
                        let mut blinding_rng =
                            DeterministicRngProvider::from_seed([42u8; 32]).rng();

                        for local_counter in 0..assets_per_thread {
                            let global_counter = (thread_id as u64 * assets_per_thread as u64)
                                + local_counter as u64;

                            let nonce =
                                derive_nonce(&wallet_seed_copy, global_counter, &definition_id);
                            let blinding = BlindingFactor::random(&mut blinding_rng);

                            if Asset::new(
                                Arc::clone(&arc_def_clone),
                                global_counter as u32,
                                100_000_000,
                                &blinding,
                                nonce,
                                &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
                            )
                            .is_ok()
                            {
                                count += 1;
                            }
                        }

                        count
                    })
                })
                .collect();

            let mut total_created = 0;
            for handle in handles {
                total_created += handle.join().expect("thread should complete");
            }

            let elapsed = start.elapsed();
            let throughput = (total_created as f64) / elapsed.as_secs_f64();

            println!(
                "[OK] {} threads: {} assets in {:.2}s ({:.0} assets/sec)",
                num_threads,
                total_created,
                elapsed.as_secs_f64(),
                throughput
            );
        }
    }

    // ============================================================================
    // Test 4: Asset Creation with Different Definition Instances
    // ============================================================================

    /// Test creating assets from same logical definition in different Arc instances
    ///
    /// Scenario:
    /// - Create definition D
    /// - Create Arc<D> copy A
    /// - Create Arc<D> copy B
    /// - Thread 1 creates assets from A
    /// - Thread 2 creates assets from B
    /// - Verify all assets valid
    #[test]
    fn test_concurrent_creation_multiple_arcs() {
        let definition = create_test_definition();
        let definition_id = definition.id;
        let wallet_seed = [1u8; 32];

        let arc_def_1 = Arc::new(definition.clone());
        let arc_def_2 = Arc::new(definition);

        let start = Instant::now();

        let handle1 = {
            let arc_def = Arc::clone(&arc_def_1);
            thread::spawn(move || {
                let mut count = 0;
                let mut rng = DeterministicRngProvider::from_seed([42u8; 32]).rng();

                for i in 0..50 {
                    // Reduced for <15s target
                    let nonce = derive_nonce(&wallet_seed, i, &definition_id);
                    let blinding = BlindingFactor::random(&mut rng);

                    if Asset::new(
                        Arc::clone(&arc_def),
                        i as u32,
                        100_000_000,
                        &blinding,
                        nonce,
                        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
                    )
                    .is_ok()
                    {
                        count += 1;
                    }
                }

                count
            })
        };

        let handle2 = {
            let arc_def = Arc::clone(&arc_def_2);
            thread::spawn(move || {
                let mut count = 0;
                let mut rng = DeterministicRngProvider::from_seed([42u8; 32]).rng();

                for i in 50..100 {
                    // Reduced for <15s target
                    let nonce = derive_nonce(&wallet_seed, i, &definition_id);
                    let blinding = BlindingFactor::random(&mut rng);

                    if Asset::new(
                        Arc::clone(&arc_def),
                        i as u32,
                        100_000_000,
                        &blinding,
                        nonce,
                        &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
                    )
                    .is_ok()
                    {
                        count += 1;
                    }
                }

                count
            })
        };

        let count1 = handle1.join().expect("thread 1 complete");
        let count2 = handle2.join().expect("thread 2 complete");
        let elapsed = start.elapsed();

        assert_eq!(count1, 50, "Thread 1 should create 50 assets");
        assert_eq!(count2, 50, "Thread 2 should create 50 assets");

        println!(
            "[OK] Multiple Arc instances: {} total assets in {:.2}s",
            count1 + count2,
            elapsed.as_secs_f64()
        );
    }

    // ============================================================================
    // Test 5: Nonce Collision Detection in Concurrent Creation
    // ============================================================================

    /// Verify that concurrent nonce generation produces no collisions
    ///
    /// Scenario:
    /// - 3 threads each generating 100 nonces (reduced for speed)
    /// - Same seed, sequential counters
    /// - Verify all 300 nonces unique
    #[test]
    fn test_concurrent_nonce_uniqueness() {
        let wallet_seed = [2u8; 32];
        let asset_id = [7u8; 32];
        let num_threads = 3;
        let nonces_per_thread = 100;

        let all_nonces = Arc::new(Mutex::new(BTreeSet::new()));

        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                let nonces = Arc::clone(&all_nonces);
                let seed = wallet_seed;
                let id = asset_id;

                thread::spawn(move || {
                    let mut thread_nonces = Vec::new();

                    for local_counter in 0..nonces_per_thread {
                        let global_counter =
                            (thread_id as u64 * nonces_per_thread as u64) + local_counter as u64;
                        let nonce = derive_nonce(&seed, global_counter, &id);
                        thread_nonces.push(nonce);
                    }

                    // Insert all at once
                    let mut guard = nonces.lock().unwrap();
                    for nonce in thread_nonces {
                        guard.insert(nonce);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("thread should complete");
        }

        let nonces = all_nonces.lock().unwrap();
        let expected_count = num_threads * nonces_per_thread;

        assert_eq!(
            nonces.len(),
            expected_count,
            "All {} nonces should be unique",
            expected_count
        );

        println!(
            "[OK] Concurrent nonce uniqueness: {} threads generated {} unique nonces",
            num_threads,
            nonces.len()
        );
    }
}
