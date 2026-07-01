//! Metrics and telemetry abstraction
//!
//! This module provides a metrics sink trait to enable dependency injection
//! of metrics implementations. Metrics are essential for monitoring performance,
//! tracking events, and understanding system behavior in production.
//!
//! # Examples
//!
//! ```
//! use z00z_utils::metrics::{MetricsSink, NoopMetrics};
//! use z00z_utils::time::Instant;
//!
//! let metrics = NoopMetrics;
//! let start = Instant::now();
//! // ... expensive operation ...
//! metrics.observe_histogram("operation_ms", start.elapsed().as_secs_f64() * 1000.0);
//! metrics.inc_counter("operations_completed", 1);
//! ```

mod noop;
mod traits;

pub use noop::NoopMetrics;
pub use traits::MetricsSink;

#[cfg(test)]
mod test_metrics;
