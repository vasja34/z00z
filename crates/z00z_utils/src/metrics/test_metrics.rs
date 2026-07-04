//! MetricsSink tests

use crate::metrics::{MetricsSink, NoopMetrics};
use std::sync::Arc;
use std::thread;

/// Test trait object safety
#[test]
fn test_metrics_object_safety() {
    let _: Box<dyn MetricsSink> = Box::new(NoopMetrics);
    let _: Arc<dyn MetricsSink> = Arc::new(NoopMetrics);
}

/// Test thread safety
#[test]
fn test_metrics_thread_safety() {
    let metrics = Arc::new(NoopMetrics);
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let metrics = Arc::clone(&metrics);
            thread::spawn(move || {
                metrics.inc_counter("test", 1);
                metrics.observe_histogram("test", i as f64);
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

/// Test that implementations can be swapped
#[test]
fn test_metrics_polymorphism() {
    fn test_process_with_metrics(metrics: &dyn MetricsSink) {
        metrics.inc_counter("events", 1);
        metrics.observe_histogram("latency", 42.5);
        metrics.set_gauge("count", 100.0);
    }

    test_process_with_metrics(&NoopMetrics);
}
