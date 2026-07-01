//! MetricsSink trait definition

/// Metrics abstraction for telemetry and monitoring
///
/// This trait provides a way to track performance metrics without being
/// tightly coupled to a specific metrics library (Prometheus, etc.).
///
/// # Use Cases
///
/// - Track asset creation count
/// - Measure transaction validation latency
/// - Monitor commitment generation performance
/// - Record proof verification throughput
/// - Monitor queue depths and system state
///
/// # Examples
///
/// ```
/// use z00z_utils::metrics::{MetricsSink, NoopMetrics};
/// use z00z_utils::time::Instant;
///
/// let metrics = NoopMetrics;
/// let start = Instant::now();
/// // ... expensive operation ...
/// metrics.observe_histogram("operation_ms", start.elapsed().as_secs_f64() * 1000.0);
/// metrics.inc_counter("operations_completed", 1);
/// ```
pub trait MetricsSink: Send + Sync {
    /// Increment a counter by a value
    ///
    /// Used to count discrete events.
    fn inc_counter(&self, name: &str, value: u64);

    /// Record a histogram observation (e.g., latency in ms)
    ///
    /// Used to measure distributions (latency, sizes, etc.).
    fn observe_histogram(&self, name: &str, value: f64);

    /// Set a gauge value (e.g., current queue size)
    ///
    /// Used to track current state (not cumulative).
    fn set_gauge(&self, name: &str, value: f64);
}
