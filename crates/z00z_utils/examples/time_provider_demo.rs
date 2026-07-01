//! Time provider example demonstrating time operations and testing patterns.
//!
//! This example shows how to:
//! - Use SystemTimeProvider for production code
//! - Use MockTimeProvider for deterministic testing
//! - Get Unix timestamps in seconds and milliseconds
//! - Implement dependency injection with TimeProvider trait
//! - Control time in tests for reproducibility
//!
//! Run with: `cargo run --package z00z_utils --example time_provider_demo`

use std::sync::Arc;
use std::time::Duration;
use z00z_utils::prelude::*;

/// Function that depends on time provider
/// This allows injecting different implementations for testing
fn measure_elapsed(time_provider: Arc<dyn TimeProvider>, label: &str) -> u64 {
    let start = time_provider.compat_unix_timestamp();
    println!("{} started at: {} seconds since epoch", label, start);

    // Simulate work
    std::thread::sleep(Duration::from_millis(100));

    let end = time_provider.compat_unix_timestamp();
    let elapsed = end.saturating_sub(start);
    println!("{} ended at: {} seconds since epoch", label, end);
    println!("  Elapsed: {} seconds\n", elapsed);

    elapsed
}

fn main() {
    println!("=== Z00Z Utils: Time Provider Demo ===\n");

    // Example 1: SystemTimeProvider for production
    println!("--- SystemTimeProvider (Production) ---");
    let system_time = Arc::new(SystemTimeProvider);

    let now_seconds = system_time
        .try_unix_timestamp()
        .expect("system clock after epoch");
    println!("Current time: {} seconds since epoch", now_seconds);

    let now_millis = system_time
        .try_unix_timestamp_ms()
        .expect("system clock after epoch");
    println!("Current time: {} milliseconds since epoch", now_millis);
    println!();

    // Example 2: Unix timestamp precision
    println!("--- Timestamp Precision ---");
    let time1 = system_time
        .try_unix_timestamp_ms()
        .expect("system clock after epoch");
    std::thread::sleep(Duration::from_millis(50));
    let time2 = system_time
        .try_unix_timestamp_ms()
        .expect("system clock after epoch");

    println!("Time 1: {}", time1);
    println!("Time 2: {}", time2);
    println!("Delta: {} milliseconds", time2 - time1);
    println!("(precision in milliseconds)\n");

    // Example 3: MockTimeProvider for testing
    println!("--- MockTimeProvider (Testing) ---");
    let mock_time = Arc::new(MockTimeProvider::new(
        std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(1000000),
    ));

    println!("Initial time: {}", mock_time.compat_unix_timestamp());
    println!("(Fixed at 1000000 seconds)\n");

    // Example 4: Advancing time in tests
    println!("--- Advancing Time (Deterministic Testing) ---");
    let test_time = Arc::new(MockTimeProvider::new(
        std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(1000),
    ));

    println!("Initial time: {}", test_time.compat_unix_timestamp());

    test_time.advance_by(Duration::from_secs(100));
    println!("After +100s: {}", test_time.compat_unix_timestamp());

    test_time.advance_by(Duration::from_secs(50));
    println!("After +50s more: {}", test_time.compat_unix_timestamp());
    println!("(Total advancement: 150 seconds)\n");

    // Example 5: Setting specific times
    println!("--- Setting Specific Times ---");
    let mock = Arc::new(MockTimeProvider::new(
        std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(1000),
    ));

    println!("Before set_time: {}", mock.compat_unix_timestamp());

    mock.set_time(std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(5000));
    println!("After set_time to 5000: {}", mock.compat_unix_timestamp());

    mock.set_time(std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(2500));
    println!("After set_time to 2500: {}", mock.compat_unix_timestamp());
    println!("(Can jump forward or backward)\n");

    // Example 6: Dependency injection for testing
    println!("--- Dependency Injection Pattern ---");
    println!("Using SystemTimeProvider:");
    measure_elapsed(system_time.clone(), "System operation");

    println!("Using MockTimeProvider (deterministic):");
    let mock_for_testing = Arc::new(MockTimeProvider::new(
        std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(1000),
    ));
    measure_elapsed(mock_for_testing.clone(), "Test operation");

    // Example 7: Use cases
    println!("--- Common Use Cases ---");
    println!("1. Rate limiting: Track requests per time window");
    println!("   let now = time_provider.try_unix_timestamp()?;");
    println!("   if now - last_request > 60 {{ /* ok */ }}");
    println!();
    println!("2. Session expiry: Check if session expired");
    println!("   let expired = now > session.created_at + session.duration;");
    println!();
    println!("3. Deterministic testing: Control time to test edge cases");
    println!("   mock_time.set_time(expiry_time);");
    println!("   assert!(is_expired(&session));");
    println!();

    // Example 8: Millisecond precision
    println!("--- Millisecond Precision ---");
    let mock_ms = Arc::new(MockTimeProvider::new(
        std::time::SystemTime::UNIX_EPOCH + Duration::from_millis(1234567890000),
    ));

    println!(
        "Mock time (millis): {}",
        mock_ms.compat_unix_timestamp_millis()
    );
    println!("Mock time (seconds): {}", mock_ms.compat_unix_timestamp());

    mock_ms.advance_by(Duration::from_millis(500));
    println!("After +500ms:");
    println!("  Millis: {}", mock_ms.compat_unix_timestamp_millis());
    println!(
        "  Seconds: {} (unchanged if < 1s)",
        mock_ms.compat_unix_timestamp()
    );
    println!();

    // Example 9: Thread-safe sharing
    println!("--- Thread-Safe Sharing ---");
    let shared_time = Arc::new(MockTimeProvider::new(
        std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(1000),
    ));

    let shared_time_clone = shared_time.clone();
    let handle = std::thread::spawn(move || {
        let t = shared_time_clone.compat_unix_timestamp();
        println!("Thread sees time: {}", t);
        t
    });

    let main_time = shared_time.compat_unix_timestamp();
    println!("Main thread time: {}", main_time);

    let thread_time = handle.join().unwrap();
    assert_eq!(main_time, thread_time);
    println!("✓ Same time visible across threads\n");

    println!("=== Example Completed Successfully ===");
}
