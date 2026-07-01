//! Metrics sink example demonstrating metrics collection and reporting.
//!
//! This example shows how to:
//! - Track counters (increment operations)
//! - Use gauges (set values)
//! - Measure histograms (latency tracking)
//! - Collect metrics in your application
//! - Zero-overhead NoopMetrics for testing
//!
//! Run with: `cargo run --package z00z_utils --example metrics_demo`

use std::sync::Arc;
use z00z_utils::prelude::*;
use z00z_utils::time::Instant;

/// Simulates an operation that we want to measure
fn process_item(item_id: usize, metrics: Arc<dyn MetricsSink>) -> u64 {
    let start = Instant::now();

    // Simulate work
    let result: u64 = (0..=item_id as u64).sum();

    // Record how long it took
    let elapsed_ms = start.elapsed().as_millis() as f64;
    metrics.observe_histogram("process_item_ms", elapsed_ms);
    metrics.inc_counter("items_processed", 1);

    result
}

/// Simulates a batch operation
fn batch_process(count: usize, metrics: Arc<dyn MetricsSink>) {
    let batch_start = Instant::now();

    for i in 0..count {
        let _result = process_item(i, metrics.clone());
    }

    let batch_time_ms = batch_start.elapsed().as_millis() as f64;
    metrics.observe_histogram("batch_process_ms", batch_time_ms);
    metrics.set_gauge("last_batch_size", count as f64);
}

fn main() {
    println!("=== Z00Z Utils: Metrics Demo ===\n");

    // Example 1: Using NoopMetrics (zero overhead)
    println!("--- NoopMetrics (production use) ---");
    let noop_metrics: Arc<dyn MetricsSink> = Arc::new(NoopMetrics);

    println!("Processing items with NoopMetrics:");
    for i in 0..5 {
        process_item(i * 100, noop_metrics.clone());
        println!("  ✓ Processed item {}", i);
    }
    println!("(No metrics collected - zero overhead)\n");

    // Example 2: Tracking counters
    println!("--- Counter Tracking ---");
    let metrics = Arc::new(NoopMetrics);

    println!("Simulating API requests:");
    for request_id in 0..10 {
        metrics.inc_counter("api_requests", 1);
        if request_id % 3 == 0 {
            metrics.inc_counter("api_errors", 1);
        }
        println!("  ✓ Request {} processed", request_id);
    }
    println!("(10 total requests, 4 errors tracked)\n");

    // Example 3: Tracking gauges (current values)
    println!("--- Gauge Tracking (Current Values) ---");
    let metrics = Arc::new(NoopMetrics);

    println!("Monitoring queue depth:");
    for depth in &[5, 12, 8, 15, 3] {
        metrics.set_gauge("queue_depth", *depth as f64);
        println!("  Queue depth set to: {}", depth);
    }
    println!("(Final queue depth: 3)\n");

    // Example 4: Tracking latency with histograms
    println!("--- Histogram Tracking (Latency) ---");
    let metrics = Arc::new(NoopMetrics);

    println!("Recording operation latencies:");
    let latencies = [5, 12, 8, 23, 15, 7, 19, 11];
    for (idx, latency) in latencies.iter().enumerate() {
        metrics.observe_histogram("operation_ms", *latency as f64);
        println!("  Operation {}: {}ms", idx + 1, latency);
    }
    println!("(Recorded 8 latency measurements)\n");

    // Example 5: Combined metrics in realistic scenario
    println!("--- Realistic Scenario: Batch Processing ---");
    let metrics = Arc::new(NoopMetrics);

    println!("Processing batches:");
    for batch_num in 0..3 {
        println!("  Batch {}:", batch_num + 1);
        batch_process(10, metrics.clone());
        println!("    ✓ 10 items processed");
    }
    println!("(Total: 30 items, 3 batches tracked)\n");

    // Example 6: Metrics types summary
    println!("--- Metrics Types Summary ---");
    println!("1. Counters: Total count of events (use inc_counter)");
    println!("   Example: Total API requests, Total errors");
    println!();
    println!("2. Gauges: Current value (use set_gauge)");
    println!("   Example: Queue depth, Active connections, Memory usage");
    println!();
    println!("3. Histograms: Distribution of values (use observe_histogram)");
    println!("   Example: Request latency, Operation duration, Data size");
    println!();

    // Example 7: Best practices
    println!("--- Best Practices ---");
    let metrics = Arc::new(NoopMetrics);

    // ✓ Good: Use Arc<dyn MetricsSink> for dependency injection
    fn report_metrics(metrics: Arc<dyn MetricsSink>) {
        metrics.inc_counter("operations", 1);
    }
    report_metrics(metrics.clone());
    println!("✓ Use trait objects for dependency injection");

    // ✓ Good: NoopMetrics for zero-cost testing
    let _test_metrics: Arc<dyn MetricsSink> = Arc::new(NoopMetrics);
    println!("✓ Use NoopMetrics in tests for zero overhead");

    // ✓ Good: Descriptive names
    println!("✓ Use descriptive metric names (e.g., 'request_duration_ms')");

    // ✓ Good: Record metrics near operations
    println!("✓ Record metrics close to operations for accuracy");

    println!("\n=== Example Completed Successfully ===");
}
