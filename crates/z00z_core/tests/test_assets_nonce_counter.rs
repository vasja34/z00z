//! Phase 2, Test 14: NonceCounter Thread Safety
//!
//! This test verifies thread-safe counter operations critical for:
//! - Multi-threaded wallet nonce generation
//! - Preventing counter race conditions
//! - Detecting lost increments
//! - Overflow handling
//!
//! Uses Arc<Mutex<NonceCounter>> for thread-safe access.

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use std::thread;
use z00z_core::assets::nonce::NonceCounter;
use z00z_utils::time::{Instant, SystemTimeProvider};

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Test 1: Basic Thread Safety - 10 Threads × 1000 Increments
    // ============================================================================

    /// Test that counter increments are thread-safe with no lost updates
    ///
    /// Scenario:
    /// - 10 threads share Arc<Mutex<NonceCounter>>
    /// - Each thread increments 1000 times
    /// - Verify final value = 10,000 (no lost increments)
    #[test]
    fn test_nonce_counter_thread_safety() {
        let counter = Arc::new(Mutex::new(NonceCounter::new()));
        let num_threads = 10;
        let increments_per_thread = 1000;

        assert_eq!(
            counter.lock().unwrap().value(),
            0,
            "Counter should start at 0"
        );

        let start_time = Instant::now();

        let handles: Vec<_> = (0..num_threads)
            .map(|_| {
                let counter_clone = Arc::clone(&counter);

                thread::spawn(move || {
                    let mut local_increments = 0;

                    for _ in 0..increments_per_thread {
                        match counter_clone
                            .lock()
                            .unwrap()
                            .increment_unsafe(&SystemTimeProvider)
                        {
                            Ok(_value) => local_increments += 1,
                            Err(e) => panic!("Increment failed: {:?}", e),
                        }
                    }

                    local_increments
                })
            })
            .collect();

        // Wait for all threads
        let mut total_increments = 0;
        for (i, handle) in handles.into_iter().enumerate() {
            let thread_increments = handle.join().expect("thread should complete");
            total_increments += thread_increments;
            println!(
                "[OK] Thread {} completed {} increments",
                i, thread_increments
            );
        }

        let elapsed = start_time.elapsed();
        let final_value = counter.lock().unwrap().value();

        // Verify results
        assert_eq!(
            final_value,
            (num_threads * increments_per_thread) as u64,
            "Final counter value should equal total increments"
        );

        assert_eq!(
            total_increments,
            num_threads * increments_per_thread,
            "All increments should complete successfully"
        );

        let throughput = total_increments as f64 / elapsed.as_secs_f64();

        println!(
            "[OK] NonceCounter thread safety: {} total increments in {:.2}s ({:.0} inc/sec)",
            final_value,
            elapsed.as_secs_f64(),
            throughput
        );
    }

    // ============================================================================
    // Test 2: Counter Values Uniqueness
    // ============================================================================

    /// Verify that each increment produces a unique counter value
    ///
    /// Scenario:
    /// - Collect counter values from all threads
    /// - Verify all values are unique
    /// - Verify no gaps in sequence
    #[test]
    fn test_nonce_counter_uniqueness() {
        let counter = Arc::new(Mutex::new(NonceCounter::new()));
        let num_threads = 5;
        let increments_per_thread = 200;

        let all_values = Arc::new(Mutex::new(Vec::new()));

        let handles: Vec<_> = (0..num_threads)
            .map(|_| {
                let counter_clone = Arc::clone(&counter);
                let values_clone = Arc::clone(&all_values);

                thread::spawn(move || {
                    let mut thread_values = Vec::new();

                    for _ in 0..increments_per_thread {
                        match counter_clone
                            .lock()
                            .unwrap()
                            .increment_unsafe(&SystemTimeProvider)
                        {
                            Ok(value) => thread_values.push(value),
                            Err(e) => panic!("Increment failed: {:?}", e),
                        }
                    }

                    values_clone.lock().unwrap().extend(thread_values);
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("thread should complete");
        }

        let values = all_values.lock().unwrap();
        let value_set: BTreeSet<u64> = values.iter().cloned().collect();

        assert_eq!(
            value_set.len(),
            num_threads * increments_per_thread,
            "All counter values should be unique"
        );

        // Verify no gaps: should have all values from 1 to 1000
        let expected_count = num_threads * increments_per_thread;
        let min_value = *values.iter().min().expect("should have values");
        let max_value = *values.iter().max().expect("should have values");

        assert_eq!(min_value, 1, "First value should be 1");
        assert_eq!(
            max_value, expected_count as u64,
            "Last value should be {}",
            expected_count
        );

        println!(
            "[OK] NonceCounter uniqueness: {} unique values, no gaps (1-{})",
            value_set.len(),
            expected_count
        );
    }

    // ============================================================================
    // Test 3: High-Concurrency Load Test
    // ============================================================================

    /// Test counter under high concurrency (20 threads)
    ///
    /// Scenario:
    /// - 20 concurrent reader/writer threads
    /// - Each performs 500 increments
    /// - Measure lock contention and throughput
    #[test]
    fn test_nonce_counter_high_concurrency() {
        let counter = Arc::new(Mutex::new(NonceCounter::new()));
        let num_threads = 20;
        let increments_per_thread = 500;

        let start = Instant::now();

        let handles: Vec<_> = (0..num_threads)
            .map(|_| {
                let counter_clone = Arc::clone(&counter);

                thread::spawn(move || {
                    for _ in 0..increments_per_thread {
                        let _ = counter_clone
                            .lock()
                            .unwrap()
                            .increment_unsafe(&SystemTimeProvider);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("thread should complete");
        }

        let elapsed = start.elapsed();
        let final_value = counter.lock().unwrap().value();
        let expected = (num_threads * increments_per_thread) as u64;

        assert_eq!(final_value, expected, "Counter should reach expected value");

        let throughput = expected as f64 / elapsed.as_secs_f64();

        println!(
            "[OK] High concurrency (20 threads): {} increments in {:.2}s ({:.0} inc/sec)",
            final_value,
            elapsed.as_secs_f64(),
            throughput
        );
    }

    // ============================================================================
    // Test 4: Counter Value Persistence Through Reads
    // ============================================================================

    /// Verify that counter value() method returns correct value under concurrency
    ///
    /// Scenario:
    /// - Thread 1: continuously increments
    /// - Thread 2: continuously reads value
    /// - Verify read values are monotonically increasing
    #[test]
    fn test_nonce_counter_read_consistency() {
        let counter = Arc::new(Mutex::new(NonceCounter::new()));
        let mut_counter = Arc::clone(&counter);
        let read_counter = Arc::clone(&counter);

        let writer_handle = thread::spawn(move || {
            let mut count = 0;
            for _ in 0..1000 {
                if mut_counter
                    .lock()
                    .unwrap()
                    .increment_unsafe(&SystemTimeProvider)
                    .is_ok()
                {
                    count += 1;
                }
            }
            count
        });

        let reader_handle = thread::spawn(move || {
            let mut prev_value = 0u64;
            let mut monotonic_violations = 0;

            for _ in 0..5000 {
                let current_value = read_counter.lock().unwrap().value();
                if current_value < prev_value {
                    monotonic_violations += 1;
                }
                prev_value = current_value;
            }

            monotonic_violations
        });

        let written = writer_handle.join().expect("writer should complete");
        let violations = reader_handle.join().expect("reader should complete");

        assert_eq!(
            violations, 0,
            "Counter value should never decrease (monotonic)"
        );

        println!(
            "[OK] Read consistency: {} increments, 0 monotonicity violations",
            written
        );
    }

    // ============================================================================
    // Test 5: Overflow Handling
    // ============================================================================

    /// Test counter behavior at near-overflow values
    ///
    /// Scenario:
    /// - Set counter to u64::MAX - 10
    /// - Try to increment 20 times
    /// - Verify overflow error occurs at correct point
    #[test]
    fn test_nonce_counter_overflow_handling() {
        let mut counter = NonceCounter::new();
        let near_max = u64::MAX - 10;

        // Use recovery method to set high value
        let _ = counter.set_value_recovery(near_max, &SystemTimeProvider);
        assert_eq!(
            counter.value(),
            near_max,
            "Counter should be set to near_max"
        );

        // Try to increment past the limit
        let mut successful_increments = 0;
        let mut overflow_count = 0;

        for _ in 0..20 {
            match counter.increment_unsafe(&SystemTimeProvider) {
                Ok(_) => successful_increments += 1,
                Err(_) => overflow_count += 1,
            }
        }

        assert_eq!(
            successful_increments, 10,
            "Should allow 10 increments before overflow"
        );
        assert_eq!(overflow_count, 10, "Should get 10 overflow errors");

        println!(
            "[OK] Overflow handling: {} successful increments, {} overflow errors",
            successful_increments, overflow_count
        );
    }

    // ============================================================================
    // Test 6: Recovery Method Safety
    // ============================================================================

    /// Test that set_value_recovery prevents regression
    ///
    /// Scenario:
    /// - Set counter to 1000
    /// - Increment to 1010
    /// - Try to set back to 500 (should panic)
    #[test]
    fn test_prevents_nonce_counter_recovery() {
        let mut counter = NonceCounter::new();
        let _ = counter.set_value_recovery(1000, &SystemTimeProvider);

        for _ in 0..10 {
            let _ = counter.increment_unsafe(&SystemTimeProvider);
        }

        assert!(counter.value() >= 1010, "Counter should be at 1010+");

        // This should return Err because 500 < 1010
        let result = counter.set_value_recovery(500, &SystemTimeProvider);
        assert!(
            result.is_err(),
            "set_value_recovery should reject lower value"
        );
    }

    // ============================================================================
    // Test 7: Multiple Shared References
    // ============================================================================

    /// Test counter behavior with multiple Arc references
    ///
    /// Scenario:
    /// - Create counter
    /// - Clone Arc 10 times
    /// - Distribute to 10 threads
    /// - All increment same counter
    /// - Verify final value correct
    #[test]
    fn test_nonce_counter_arc_cloning() {
        let counter = Arc::new(Mutex::new(NonceCounter::new()));
        let mut handles = Vec::new();

        for _ in 0..10 {
            let counter_clone = Arc::clone(&counter);

            let handle = thread::spawn(move || {
                let mut count = 0;
                for _ in 0..100 {
                    if counter_clone
                        .lock()
                        .unwrap()
                        .increment_unsafe(&SystemTimeProvider)
                        .is_ok()
                    {
                        count += 1;
                    }
                }
                count
            });

            handles.push(handle);
        }

        let mut total = 0;
        for handle in handles {
            total += handle.join().expect("thread should complete");
        }

        let final_value = counter.lock().unwrap().value();

        assert_eq!(final_value, 1000, "Final value should be 1000");
        assert_eq!(total, 1000, "Total increments should be 1000");

        println!(
            "[OK] Arc cloning: {} increments across 10 threads",
            final_value
        );
    }
}
