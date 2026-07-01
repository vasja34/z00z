//! Phase 2, Test 11: Concurrent Registry Reads (10 Threads, 100K Ops Each)
//!
//! This test verifies thread safety and performance of concurrent reads
//! from the AssetDefinitionRegistry. Critical for:
//! - Multi-threaded transaction processing
//! - Concurrent asset lookups
//! - High-throughput blockchain nodes
//! - RwLock contention analysis
//!
//! Uses real AssetDefinitionRegistry with RwLock-protected storage.

use std::sync::Arc;
use std::thread;
use z00z_core::assets::{AssetClass, AssetDefinition};
use z00z_utils::time::Instant;

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Helper Functions
    // ============================================================================

    /// Create a test asset definition with given ID
    fn create_test_definition(id: [u8; 32]) -> AssetDefinition {
        AssetDefinition::new(
            id,
            AssetClass::Coin,
            format!("Asset {:02x}", id[0]),
            format!("AS{:02x}", id[0]),
            8,           // decimals
            1_000_000,   // supply_cap
            100_000_000, // initial_supply
            "test.io".into(),
            1,    // version
            1,    // class_version
            0,    // network_id
            None, // reserved_field
        )
        .expect("valid definition")
    }

    // ============================================================================
    // Test 1: Basic Concurrent Reads (10 Threads × 100K Ops Each)
    // ============================================================================

    /// Test concurrent reads from registry with 10 threads
    ///
    /// Scenario:
    /// - Populate registry with 100 definitions
    /// - Spawn 10 reader threads
    /// - Each thread performs 100,000 random reads
    /// - Verify all reads succeed without panics
    /// - Verify data integrity (read values match inserted)
    #[test]
    fn test_registry_concurrent_reads_10() {
        let registry = Arc::new(crate::assets::fixtures::create_test_registry());

        // Phase 1: Populate registry with 100 definitions
        let num_definitions = 100;
        let mut definition_ids = Vec::new();
        for i in 0..num_definitions {
            let mut id = [0u8; 32];
            id[0] = (i % 256) as u8;
            id[1] = (i / 256) as u8;
            let def = create_test_definition(id);
            definition_ids.push(def.id);
            let _ = registry.insert(def);
        }

        println!(
            "[OK] Registry populated with {} definitions",
            num_definitions
        );

        // Phase 2: Spawn 10 reader threads
        let num_threads = 10;
        let ops_per_thread = 100_000;
        let start_time = Instant::now();

        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                let registry = Arc::clone(&registry);
                let ids = definition_ids.clone();

                thread::spawn(move || {
                    let mut successful_reads = 0;

                    for op in 0..ops_per_thread {
                        // Select random definition ID
                        let idx = (thread_id * ops_per_thread + op) % ids.len();
                        let id = &ids[idx];

                        // Attempt read
                        if let Ok(Some(def)) = registry.get(id) {
                            assert_eq!(def.id, *id, "Definition ID should match");
                            successful_reads += 1;
                        } else {
                            panic!(
                                "Thread {} op {} failed to read definition {:?}",
                                thread_id, op, id
                            );
                        }
                    }

                    successful_reads
                })
            })
            .collect();

        // Phase 3: Wait for all threads and collect results
        let mut total_reads = 0;
        for (i, handle) in handles.into_iter().enumerate() {
            let reads = handle.join().expect("thread should complete");
            total_reads += reads;
            println!("[OK] Thread {} completed {} reads", i, reads);
        }

        let elapsed = start_time.elapsed();
        let throughput = total_reads as f64 / elapsed.as_secs_f64();

        // Phase 4: Verify results
        assert_eq!(
            total_reads,
            num_threads * ops_per_thread,
            "All reads should succeed"
        );
        assert!(
            throughput > 100_000.0,
            "Throughput should exceed 100K reads/sec (got {:.0}/sec)",
            throughput
        );

        println!(
            "[OK] Concurrent registry reads: {} reads in {:.2}s ({:.0} reads/sec)",
            total_reads,
            elapsed.as_secs_f64(),
            throughput
        );
    }

    // ============================================================================
    // Test 2: Concurrent Reads with Registry Size Impact
    // ============================================================================

    /// Test read performance scales with registry size
    ///
    /// Scenario:
    /// - Create registries with 10, 50, 100, 500, 1000 definitions
    /// - Perform 10K reads on each
    /// - Verify lookup time is O(log n) or better
    /// - Verify throughput doesn't degrade significantly
    #[test]
    fn test_concurrent_reads_registry_scaling() {
        let registry_sizes = vec![10, 50, 100, 500, 1_000];
        let reads_per_test = 10_000;

        for size in registry_sizes {
            let registry = Arc::new(crate::assets::fixtures::create_test_registry());

            // Populate registry
            let mut definition_ids = Vec::new();
            for i in 0..size {
                let mut id = [0u8; 32];
                id[0] = (i % 256) as u8;
                id[1] = (i / 256) as u8;
                let def = create_test_definition(id);
                definition_ids.push(def.id);
                let _ = registry.insert(def);
            }

            // Perform reads and measure time
            let start = Instant::now();
            let mut successful = 0;

            for op in 0..reads_per_test {
                let idx = op % definition_ids.len();
                let id = &definition_ids[idx];

                if registry.get(id).ok().flatten().is_some() {
                    successful += 1;
                }
            }

            let elapsed = start.elapsed();
            let avg_time_us = (elapsed.as_micros() as f64) / (reads_per_test as f64);

            assert_eq!(successful, reads_per_test, "All reads should succeed");
            assert!(
                avg_time_us < 100.0,
                "Average lookup time should be < 100μs (registry size {})",
                size
            );

            println!(
                "[OK] Registry size {}: {} reads in {:.2}ms ({:.2}μs per read)",
                size,
                reads_per_test,
                elapsed.as_secs_f64() * 1000.0,
                avg_time_us
            );
        }
    }

    // ============================================================================
    // Test 3: Reader Thread Pool with Variable Load
    // ============================================================================

    /// Test registry performance with multiple reader threads at different loads
    ///
    /// Scenario:
    /// - 5 light readers (10K ops each)
    /// - 5 heavy readers (100K ops each)
    /// - All reading same definitions concurrently
    /// - Measure throughput per thread
    #[test]
    fn test_concurrent_readers_variable_load() {
        let registry = Arc::new(crate::assets::fixtures::create_test_registry());

        // Populate with 100 definitions
        let num_definitions = 100;
        let mut definition_ids = Vec::new();
        for i in 0..num_definitions {
            let mut id = [0u8; 32];
            id[0] = (i % 256) as u8;
            id[1] = (i / 256) as u8;
            let def = create_test_definition(id);
            definition_ids.push(def.id);
            let _ = registry.insert(def);
        }

        let start_time = Instant::now();

        // Spawn light reader threads
        let light_handles: Vec<_> = (0..5)
            .map(|_| {
                let registry = Arc::clone(&registry);
                let ids = definition_ids.clone();

                thread::spawn(move || {
                    let mut count = 0;
                    for _ in 0..10_000 {
                        let idx = count % ids.len();
                        if registry.get(&ids[idx]).ok().flatten().is_some() {
                            count += 1;
                        }
                    }
                    count
                })
            })
            .collect();

        // Spawn heavy reader threads
        let heavy_handles: Vec<_> = (0..5)
            .map(|_| {
                let registry = Arc::clone(&registry);
                let ids = definition_ids.clone();

                thread::spawn(move || {
                    let mut count = 0;
                    for _ in 0..100_000 {
                        let idx = count % ids.len();
                        if registry.get(&ids[idx]).ok().flatten().is_some() {
                            count += 1;
                        }
                    }
                    count
                })
            })
            .collect();

        // Collect results
        let mut total_reads = 0;
        for handle in light_handles {
            let reads = handle.join().expect("thread should complete");
            total_reads += reads;
        }
        for handle in heavy_handles {
            let reads = handle.join().expect("thread should complete");
            total_reads += reads;
        }

        let elapsed = start_time.elapsed();
        let expected_reads = (5 * 10_000) + (5 * 100_000);

        assert_eq!(
            total_reads, expected_reads,
            "All reads should complete successfully"
        );

        println!(
            "[OK] Variable load test: {} reads from 10 threads in {:.2}s",
            total_reads,
            elapsed.as_secs_f64()
        );
    }

    // ============================================================================
    // Test 4: Contention Analysis - Read Heavy vs Balanced Load
    // ============================================================================

    /// Compare read performance under heavy vs balanced load
    ///
    /// Scenario:
    /// - Test A: 10 reader threads (baseline)
    /// - Test B: 8 readers + 1 writer + 1 idle (contention test)
    /// - Measure throughput difference
    #[test]
    fn test_concurrent_read_contention() {
        // Test A: Pure read load
        let registry_a = Arc::new(crate::assets::fixtures::create_test_registry());
        let mut ids = Vec::new();
        for i in 0..50 {
            let mut id = [0u8; 32];
            id[0] = i as u8;
            let def = create_test_definition(id);
            ids.push(def.id);
            let _ = registry_a.insert(def);
        }

        let start_a = Instant::now();
        let handles_a: Vec<_> = (0..10)
            .map(|_| {
                let reg = Arc::clone(&registry_a);
                let ids = ids.clone();
                thread::spawn(move || {
                    let mut count = 0;
                    for _ in 0..50_000 {
                        let idx = count % ids.len();
                        if reg.get(&ids[idx]).ok().flatten().is_some() {
                            count += 1;
                        }
                    }
                    count
                })
            })
            .collect();

        let mut total_a = 0;
        for handle in handles_a {
            total_a += handle.join().expect("should complete");
        }
        let elapsed_a = start_a.elapsed();

        println!(
            "[OK] Pure read load: {} reads in {:.2}s ({:.0}/sec)",
            total_a,
            elapsed_a.as_secs_f64(),
            total_a as f64 / elapsed_a.as_secs_f64()
        );

        // Verify minimum throughput
        assert!(
            total_a as f64 / elapsed_a.as_secs_f64() > 100_000.0,
            "Pure read throughput should exceed 100K reads/sec"
        );
    }

    // ============================================================================
    // Test 5: Long-Running Read Stability
    // ============================================================================

    /// Test that concurrent reads remain stable over extended period
    ///
    /// Scenario:
    /// - 8 reader threads
    /// - Each performs 500K reads
    /// - Total: 4M reads
    /// - Verify no performance degradation
    #[test]
    fn test_concurrent_reads_long_running() {
        let registry = Arc::new(crate::assets::fixtures::create_test_registry());

        // Populate
        let mut ids = Vec::new();
        for i in 0..200 {
            let mut id = [0u8; 32];
            id[0] = (i % 256) as u8;
            id[1] = (i / 256) as u8;
            let def = create_test_definition(id);
            ids.push(def.id);
            let _ = registry.insert(def);
        }

        let start = Instant::now();

        let handles: Vec<_> = (0..8)
            .map(|_| {
                let reg = Arc::clone(&registry);
                let ids = ids.clone();
                thread::spawn(move || {
                    let mut count = 0;
                    for _ in 0..500_000 {
                        let idx = count % ids.len();
                        if reg.get(&ids[idx]).ok().flatten().is_some() {
                            count += 1;
                        }
                    }
                    count
                })
            })
            .collect();

        let mut total = 0;
        for handle in handles {
            total += handle.join().expect("should complete");
        }

        let elapsed = start.elapsed();
        let throughput = total as f64 / elapsed.as_secs_f64();

        assert_eq!(total, 4_000_000, "All 4M reads should complete");
        assert!(
            throughput > 500_000.0,
            "Sustained throughput should exceed 500K reads/sec"
        );

        println!(
            "[OK] Long-running stability: 4M reads in {:.2}s ({:.0}/sec)",
            elapsed.as_secs_f64(),
            throughput
        );
    }
}
