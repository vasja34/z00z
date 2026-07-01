//! NoopLogger implementation
//!
//! A no-op logger that discards all messages with zero overhead.
//! Used in tests and benchmarks to eliminate logging overhead.

use crate::logger::Logger;

/// No-op logger that discards all messages
///
/// This logger implementation does nothing, making it ideal for:
/// - Unit tests (eliminates logging spam)
/// - Benchmarks (accurate performance measurements)
/// - Any scenario where logging should be disabled
///
/// All methods are inline for zero overhead with `NoopMetrics`.
///
/// # Examples
///
/// ```
/// use z00z_utils::logger::{Logger, NoopLogger};
/// use std::sync::Arc;
///
/// let logger: Arc<dyn Logger> = Arc::new(NoopLogger);
/// logger.info("This message is discarded");
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopLogger;

impl Logger for NoopLogger {
    #[inline]
    fn error(&self, _msg: &str) {}

    #[inline]
    fn warn(&self, _msg: &str) {}

    #[inline]
    fn info(&self, _msg: &str) {}

    #[inline]
    fn debug(&self, _msg: &str) {}

    #[inline]
    fn trace(&self, _msg: &str) {}
}
