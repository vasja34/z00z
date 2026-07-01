//! NoopMetrics implementation
//!
//! A no-op metrics sink that discards all metrics with zero overhead.
//! Used in tests, benchmarks, and as a default when metrics are not needed.

use crate::metrics::MetricsSink;

/// No-op metrics sink that discards all observations
///
/// This metrics sink implementation does nothing, making it ideal for:
/// - Unit tests (eliminates noise)
/// - Benchmarks (accurate performance measurements without metrics overhead)
/// - Default implementation when metrics are not needed
///
/// All methods are inline for zero overhead.
///
/// # Examples
///
/// ```
/// use z00z_utils::metrics::{MetricsSink, NoopMetrics};
/// use std::sync::Arc;
///
/// let metrics: Arc<dyn MetricsSink> = Arc::new(NoopMetrics);
/// metrics.inc_counter("events", 1);
/// metrics.observe_histogram("latency_ms", 42.5);
/// metrics.set_gauge("queue_size", 100.0);
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopMetrics;

impl MetricsSink for NoopMetrics {
    #[inline]
    fn inc_counter(&self, _name: &str, _value: u64) {}

    #[inline]
    fn observe_histogram(&self, _name: &str, _value: f64) {}

    #[inline]
    fn set_gauge(&self, _name: &str, _value: f64) {}
}
