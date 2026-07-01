//! Phase 2, Test 12: Concurrent Registry Snapshot Updates
//!
//! This test verifies thread safety of registry updates during concurrent reads.
//! Critical for:
//! - Live registry updates from validators
//! - Multi-threaded wallet synchronization
//! - Non-blocking reads during snapshot updates
//! - Arc<Definition> validity preservation after updates
//!
//! Tests RwLock semantics and version monotonicity.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Barrier,
};
use std::thread;
use std::time::{Duration, Instant};
use z00z_core::assets::{
    snapshot::RegistryVersion, wire::DefinitionWire, AssetClass, AssetDefinition,
};

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Helper Functions
    // ============================================================================

    /// Create a test asset definition with given ID and name
    fn create_test_definition_with_name(id: [u8; 32], name: &str) -> AssetDefinition {
        AssetDefinition::new(
            id,
            AssetClass::Coin,
            name.to_string(),
            format!("AS{:02x}", id[0]),
            8,
            1_000_000,
            100_000_000,
            "test.io".into(),
            1,
            1,
            0,
            None,
        )
        .expect("valid definition")
    }

    /// Create a RegistrySnapshot from a vector of definitions
    fn create_test_snapshot(
        definitions: Vec<AssetDefinition>,
        version: u64,
    ) -> z00z_core::assets::snapshot::RegistrySnapshot {
        // Convert definitions to DefinitionWire format
        let wire_defs: Vec<DefinitionWire> = definitions.iter().map(DefinitionWire::from).collect();

        // Compute hash from the canonical ordered wire payloads.
        let mut ordered_defs = wire_defs.clone();
        ordered_defs.sort_by_key(|wire| wire.id);
        let hash = RegistryVersion::compute_hash(version, &ordered_defs);

        z00z_core::assets::snapshot::RegistrySnapshot {
            version: RegistryVersion {
                version,
                hash,
                timestamp: RegistryVersion::now(),
            },
            definitions: wire_defs,
        }
    }

    // ============================================================================
    // Test 1: Basic Concurrent Reads During Updates
    // ============================================================================

    /// Test that reads succeed during snapshot updates
    ///
    /// Scenario:
    /// - Thread 1: continuously reads definitions
    /// - Thread 2: updates registry with new snapshot every 100ms
    /// - Run for 5 seconds
    /// - Verify no read failures
    #[test]
    fn test_reads_during_snapshot_updates() {
        let registry = Arc::new(crate::assets::fixtures::create_test_registry());
        let mut initial_defs = Vec::new();
        let mut seed_ids = Vec::new();

        for i in 0..50 {
            let mut id = [0u8; 32];
            id[0] = (i % 256) as u8;
            id[1] = (i / 256) as u8;
            seed_ids.push(id);

            let def = create_test_definition_with_name(id, &format!("Asset v1_{}", i));
            initial_defs.push(def);
            let _ = registry.insert(create_test_definition_with_name(
                id,
                &format!("Asset v1_{}", i),
            ));
        }

        let initial_version = registry.get_version().expect("version");
        let num_definitions = 50;

        println!(
            "[OK] Initial registry populated with {} definitions, version {}",
            num_definitions, initial_version
        );

        // Get canonical IDs for reading
        let definition_ids: Vec<[u8; 32]> = initial_defs.iter().map(|def| def.id).collect();
        let definition_ids_reader = definition_ids.clone();
        let seed_ids_updater = seed_ids.clone();

        // Thread 1: Reader
        let registry_reader = Arc::clone(&registry);
        let reader_handle = thread::spawn(move || {
            let mut total_reads = 0;
            let mut failures = 0;
            let start = Instant::now();

            while start.elapsed() < Duration::from_secs(5) {
                for (i, id) in definition_ids_reader.iter().enumerate() {
                    if registry_reader.get(id).ok().flatten().is_some() {
                        total_reads += 1;
                    } else {
                        failures += 1;
                    }

                    if i % 5 == 0 {
                        std::thread::yield_now();
                    }
                }
            }

            (total_reads, failures, start.elapsed())
        });

        // Thread 2: Updater
        let registry_updater = Arc::clone(&registry);
        let updater_handle = thread::spawn(move || {
            let mut updates = 0;
            let mut last_version = registry_updater.get_version().expect("version");
            let start = Instant::now();

            while start.elapsed() < Duration::from_secs(5) {
                // Create new snapshot with updated definitions (version + 1)
                let new_version = last_version + 1;
                let mut new_defs = Vec::new();

                for (i, id) in seed_ids_updater.iter().enumerate() {
                    let def = create_test_definition_with_name(*id, &format!("Asset v1_{}", i));
                    new_defs.push(def);
                }

                let snapshot = create_test_snapshot(new_defs, new_version);

                match registry_updater.update_from_snapshot(snapshot) {
                    Ok(()) => {
                        updates += 1;
                        last_version = new_version;
                    }
                    Err(e) => {
                        panic!("Update failed: {:?}", e);
                    }
                }

                thread::sleep(Duration::from_millis(100));
            }

            (updates, last_version, start.elapsed())
        });

        // Wait for both threads
        let (total_reads, failures, read_duration) = reader_handle.join().expect("reader thread");
        let (updates, final_version, _update_duration) =
            updater_handle.join().expect("updater thread");

        // Verify results
        assert_eq!(failures, 0, "No read failures should occur during updates");
        assert!(
            total_reads > 1000,
            "Reader should complete significant reads"
        );
        assert!(
            updates > 0,
            "Updater should complete at least one snapshot update"
        );
        assert!(
            final_version > initial_version,
            "Registry version should increase"
        );

        println!(
            "[OK] Concurrent read-update: {} reads, {} updates in {:.2}s",
            total_reads,
            updates,
            read_duration.as_secs_f64()
        );
    }

    // ============================================================================
    // Test 2: Version Monotonicity
    // ============================================================================

    /// Test that registry version never decreases (prevents downgrades)
    ///
    /// Scenario:
    /// - Multiple threads performing updates with increasing versions
    /// - Attempt downgrade (should fail)
    /// - Verify final version is maximum
    #[test]
    fn test_version_monotonicity() {
        let registry = Arc::new(crate::assets::fixtures::create_test_registry());

        // Create base definitions
        let mut base_defs = Vec::new();
        for i in 0..10 {
            let mut id = [0u8; 32];
            id[0] = i as u8;
            base_defs.push(create_test_definition_with_name(
                id,
                &format!("Asset {}", i),
            ));
        }

        // Initial update to version 1
        let initial_snapshot = create_test_snapshot(base_defs.clone(), 1);
        registry
            .update_from_snapshot(initial_snapshot)
            .expect("should update to v1");

        assert_eq!(
            registry.get_version().expect("version"),
            1,
            "Should be at version 1"
        );

        // Valid update to version 2
        let second_snapshot = create_test_snapshot(base_defs.clone(), 2);
        registry
            .update_from_snapshot(second_snapshot)
            .expect("should update to v2");

        assert_eq!(
            registry.get_version().expect("version"),
            2,
            "Should be at version 2"
        );

        // Valid update to version 5
        let latest_snapshot = create_test_snapshot(base_defs.clone(), 5);
        registry
            .update_from_snapshot(latest_snapshot)
            .expect("should update to v5");

        assert_eq!(
            registry.get_version().expect("version"),
            5,
            "Should be at version 5"
        );

        // Attempt downgrade to version 3 (should fail)
        let third_snapshot = create_test_snapshot(base_defs.clone(), 3);
        let result = registry.update_from_snapshot(third_snapshot);

        assert!(result.is_err(), "Downgrade attempt should fail");

        // Verify version unchanged after failed downgrade
        assert_eq!(
            registry.get_version().expect("version"),
            5,
            "Version should remain 5 after failed downgrade"
        );

        println!("[OK] Version monotonicity verified: 1 → 2 → 5, downgrade to 3 rejected");
    }

    #[test]
    fn test_updaters_keep_latest_version() {
        let registry = Arc::new(crate::assets::fixtures::create_test_registry());
        let mut ids = Vec::new();

        for index in 0..10 {
            let mut id = [0u8; 32];
            id[0] = index as u8;
            ids.push(id);
            let def = create_test_definition_with_name(id, &format!("Asset {}", index));
            let _ = registry.insert(def);
        }

        let base_version = registry.get_version().expect("base version");
        let gate = Arc::new(Barrier::new(3));

        let make_defs = |ids: &[[u8; 32]], tag: u64| {
            ids.iter()
                .enumerate()
                .map(|(index, id)| {
                    create_test_definition_with_name(*id, &format!("Asset {} v{}", index, tag))
                })
                .collect::<Vec<_>>()
        };

        let ids_low = ids.clone();
        let gate_low = Arc::clone(&gate);
        let reg_low = Arc::clone(&registry);
        let low_handle = thread::spawn(move || {
            gate_low.wait();
            let mut accepted = 0;
            let mut rejected = 0;

            for version in (base_version + 1)..=(base_version + 10) {
                let snapshot = create_test_snapshot(make_defs(&ids_low, version), version);
                if reg_low.update_from_snapshot(snapshot).is_ok() {
                    accepted += 1;
                } else {
                    rejected += 1;
                }
                thread::yield_now();
            }

            (accepted, rejected)
        });

        let ids_high = ids.clone();
        let gate_high = Arc::clone(&gate);
        let reg_high = Arc::clone(&registry);
        let high_handle = thread::spawn(move || {
            gate_high.wait();
            let mut accepted = 0;
            let mut rejected = 0;

            for version in (base_version + 11)..=(base_version + 20) {
                let snapshot = create_test_snapshot(make_defs(&ids_high, version), version);
                if reg_high.update_from_snapshot(snapshot).is_ok() {
                    accepted += 1;
                } else {
                    rejected += 1;
                }
                thread::yield_now();
            }

            (accepted, rejected)
        });

        gate.wait();
        let (low_ok, low_fail) = low_handle.join().expect("low updater");
        let (high_ok, high_fail) = high_handle.join().expect("high updater");
        let final_version = registry.get_version().expect("final version");
        let snapshot = registry.create_snapshot().expect("export latest snapshot");
        let mut ordered_defs = snapshot.definitions.clone();
        ordered_defs.sort_by_key(|wire| wire.id);

        assert_eq!(final_version, base_version + 20);
        assert_eq!(snapshot.version.version, final_version);
        assert!(high_ok > 0, "higher-version updater should succeed");
        assert_eq!(ordered_defs[0].name, format!("Asset 0 v{}", final_version));

        println!(
            "[OK] Concurrent updaters kept latest version: final={}, low={}/{}, high={}/{}",
            final_version, low_ok, low_fail, high_ok, high_fail
        );
    }

    #[test]
    fn test_exported_snapshots_apply_cleanly() {
        let registry = Arc::new(crate::assets::fixtures::create_test_registry());
        let mut ids = Vec::new();

        for index in 0..12 {
            let mut id = [0u8; 32];
            id[0] = index as u8;
            ids.push(id);
            let def = create_test_definition_with_name(id, &format!("Asset {}", index));
            let _ = registry.insert(def);
        }

        let done = Arc::new(AtomicBool::new(false));
        let reg_update = Arc::clone(&registry);
        let ids_update = ids.clone();
        let done_update = Arc::clone(&done);
        let updater = thread::spawn(move || {
            let base_version = reg_update.get_version().expect("base version");
            for version in (base_version + 1)..=(base_version + 15) {
                let defs = ids_update
                    .iter()
                    .enumerate()
                    .map(|(index, id)| {
                        create_test_definition_with_name(
                            *id,
                            &format!("Asset {} v{}", index, version),
                        )
                    })
                    .collect::<Vec<_>>();

                let snapshot = create_test_snapshot(defs, version);
                reg_update
                    .update_from_snapshot(snapshot)
                    .expect("concurrent update should succeed");
                thread::sleep(Duration::from_millis(5));
            }
            done_update.store(true, Ordering::Release);
        });

        let reg_export = Arc::clone(&registry);
        let exporter = thread::spawn(move || {
            let mut exports = 0;
            while !done.load(Ordering::Acquire) || exports < 25 {
                let snapshot = reg_export.create_snapshot().expect("snapshot export");
                let mut ordered_defs = snapshot.definitions.clone();
                ordered_defs.sort_by_key(|wire| wire.id);
                let expected_hash =
                    RegistryVersion::compute_hash(snapshot.version.version, &ordered_defs);

                assert_eq!(snapshot.version.hash, expected_hash);

                let restored = crate::assets::fixtures::create_test_registry();
                restored
                    .update_from_snapshot(snapshot)
                    .expect("exported snapshot should apply cleanly");
                exports += 1;
                thread::yield_now();
            }

            exports
        });

        updater.join().expect("updater thread");
        let exports = exporter.join().expect("exporter thread");

        assert!(
            exports >= 25,
            "export thread should complete repeated exports"
        );
    }

    // ============================================================================
    // Test 3: Arc<Definition> Validity After Update
    // ============================================================================

    /// Test that old Arc references remain valid after registry update
    ///
    /// Scenario:
    /// - Get Arc<Definition> reference
    /// - Update registry with new definitions
    /// - Verify old Arc still points to valid data
    /// - Verify Arc pointer differs from new definitions
    #[test]
    fn test_arc_validity_after_update() {
        let registry = Arc::new(crate::assets::fixtures::create_test_registry());

        // Create initial definitions
        let mut id_0 = [0u8; 32];
        id_0[0] = 1;
        let first_definition = create_test_definition_with_name(id_0, "Asset");
        let def_id = first_definition.id;

        let _ = registry.insert(first_definition);

        // Get Arc reference before update
        let first_arc = registry
            .get(&def_id)
            .expect("definition should exist")
            .expect("should be Some");
        assert_eq!(first_arc.name, "Asset", "Arc should point to v1 definition");

        // Update registry with a fresh snapshot carrying the same canonical definition
        let replacement_definition = create_test_definition_with_name(id_0, "Asset");
        let second_snapshot = create_test_snapshot(vec![replacement_definition], 2);
        registry
            .update_from_snapshot(second_snapshot)
            .expect("should update to v2");

        // Verify old Arc still points to original data
        assert_eq!(
            first_arc.name, "Asset",
            "Old Arc should still point to v1 data (immutable)"
        );

        // Get new Arc after update
        let replacement_arc = registry
            .get(&def_id)
            .expect("definition should exist after update")
            .expect("should be Some");
        assert_eq!(
            replacement_arc.name, "Asset",
            "New Arc should point to replacement definition"
        );

        // Verify different Arc pointers
        assert!(
            !Arc::ptr_eq(&first_arc, &replacement_arc),
            "Arc pointers should differ (different definitions)"
        );

        println!("[OK] Arc validity preserved: v1 Arc still valid after update to v2");
    }

    // ============================================================================
    // Test 4: Multiple Concurrent Readers During Update
    // ============================================================================

    /// Test multiple reader threads with high concurrency
    ///
    /// Scenario:
    /// - 5 reader threads (10K reads each)
    /// - 1 updater thread (updates every 100ms)
    /// - Run for 3 seconds
    /// - Verify all readers succeed
    #[test]
    fn test_multiple_readers_during_update() {
        let registry = Arc::new(crate::assets::fixtures::create_test_registry());
        let mut seed_ids = Vec::new();

        // Populate registry
        let mut ids = Vec::new();

        for i in 0..50 {
            let mut id = [0u8; 32];
            id[0] = (i % 256) as u8;
            id[1] = (i / 256) as u8;
            seed_ids.push(id);

            let def = create_test_definition_with_name(id, &format!("Asset {}", i));
            ids.push(def.id);
            let _ = registry.insert(def);
        }

        let start_time = Instant::now();

        // Spawn 5 reader threads
        let mut reader_handles = Vec::new();

        for _ in 0..5 {
            let reg = Arc::clone(&registry);
            let ids_copy = ids.clone();

            let handle = thread::spawn(move || {
                let mut total_reads = 0;
                let mut failures = 0;

                while start_time.elapsed() < Duration::from_secs(3) {
                    for id in &ids_copy {
                        if reg.get(id).ok().flatten().is_some() {
                            total_reads += 1;
                        } else {
                            failures += 1;
                        }
                    }
                }

                (total_reads, failures)
            });

            reader_handles.push(handle);
        }

        // Spawn 1 updater thread
        let reg = Arc::clone(&registry);
        let seed_ids_copy = seed_ids.clone();
        let updater_handle = thread::spawn(move || {
            let mut updates = 0;
            let mut version = reg.get_version().expect("version before updates");

            while start_time.elapsed() < Duration::from_secs(3) {
                version += 1;

                let defs: Vec<_> = seed_ids_copy
                    .iter()
                    .enumerate()
                    .map(|(i, id)| create_test_definition_with_name(*id, &format!("Asset {}", i)))
                    .collect();

                let snapshot = create_test_snapshot(defs, version);
                if reg.update_from_snapshot(snapshot).is_ok() {
                    updates += 1;
                }

                thread::sleep(Duration::from_millis(100));
            }

            updates
        });

        // Collect results
        let mut total_reads = 0;
        let mut total_failures = 0;

        for handle in reader_handles {
            let (reads, failures) = handle.join().expect("reader should complete");
            total_reads += reads;
            total_failures += failures;
        }

        let updates = updater_handle.join().expect("updater should complete");

        assert_eq!(
            total_failures, 0,
            "No reader failures should occur during updates"
        );
        assert!(
            total_reads > 5000,
            "Readers should complete significant work"
        );
        assert!(updates > 0, "Updater should complete at least one update");

        println!(
            "[OK] Multiple readers: {} total reads, {} updates, 0 failures",
            total_reads, updates
        );
    }

    // ============================================================================
    // Test 5: Update Speed Under Read Pressure
    // ============================================================================

    /// Test that updates complete reasonably fast even with heavy read load
    ///
    /// Scenario:
    /// - Heavy reader load (8 threads, 50K reads each)
    /// - 1 updater completing 10 updates
    /// - Measure update latency
    #[test]
    fn test_update_latency_under_load() {
        let registry = Arc::new(crate::assets::fixtures::create_test_registry());

        // Populate
        let mut ids = Vec::new();
        for i in 0..100 {
            let mut id = [0u8; 32];
            id[0] = (i % 256) as u8;
            id[1] = (i / 256) as u8;
            let def = create_test_definition_with_name(id, &format!("Asset {}", i));
            ids.push(def.id);
            let _ = registry.insert(def);
        }

        // Spawn heavy readers
        let mut reader_handles = Vec::new();
        for _ in 0..8 {
            let reg = Arc::clone(&registry);
            let ids_copy = ids.clone();

            let handle = thread::spawn(move || {
                let mut reads = 0;
                for _ in 0..50_000 {
                    for id in &ids_copy {
                        if reg.get(id).ok().flatten().is_some() {
                            reads += 1;
                        }
                    }
                }
                reads
            });

            reader_handles.push(handle);
        }

        // Updater measures latency
        let reg = Arc::clone(&registry);
        let ids_copy = ids.clone();
        let updater_handle = thread::spawn(move || {
            let mut update_times = Vec::new();
            let base_version = reg.get_version().expect("version before latency test");

            for version in (base_version + 1)..=(base_version + 10) {
                let defs: Vec<_> = ids_copy
                    .iter()
                    .enumerate()
                    .map(|(i, id)| create_test_definition_with_name(*id, &format!("Asset {}", i)))
                    .collect();

                let snapshot = create_test_snapshot(defs, version);
                let start = Instant::now();
                reg.update_from_snapshot(snapshot)
                    .expect("update should succeed");
                let elapsed = start.elapsed();
                update_times.push(elapsed);
            }

            update_times
        });

        // Wait for completion
        let mut total_reads = 0;
        for handle in reader_handles {
            total_reads += handle.join().expect("reader complete");
        }

        let update_times = updater_handle.join().expect("updater complete");

        // Calculate stats
        let max_latency = update_times.iter().max().expect("should have updates");

        assert!(
            *max_latency < Duration::from_millis(100),
            "Update latency should be < 100ms even under heavy load"
        );

        println!(
            "[OK] Update latency: max {:.2}ms, avg {:.2}ms under {} reads",
            max_latency.as_secs_f64() * 1000.0,
            update_times
                .iter()
                .map(|d| d.as_secs_f64() * 1000.0)
                .sum::<f64>()
                / update_times.len() as f64,
            total_reads
        );
    }
}
