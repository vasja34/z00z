//! TracingLogger implementation
//!
//! Production logger that integrates with the `tracing` crate.

use crate::logger::Logger;

/// Production logger using the tracing crate
///
/// This logger implementation delegates all logging calls to the `tracing` crate,
/// which provides structured logging, filtering, and multiple output backends.
///
/// # Examples
///
/// ```
/// use z00z_utils::logger::{Logger, TracingLogger};
///
/// let logger = TracingLogger;
/// logger.info("Application started");
/// logger.error("Critical error occurred");
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct TracingLogger;

impl Logger for TracingLogger {
    fn error(&self, msg: &str) {
        tracing::error!("{}", msg);
    }

    fn warn(&self, msg: &str) {
        tracing::warn!("{}", msg);
    }

    fn info(&self, msg: &str) {
        tracing::info!("{}", msg);
    }

    fn debug(&self, msg: &str) {
        tracing::debug!("{}", msg);
    }

    fn trace(&self, msg: &str) {
        tracing::trace!("{}", msg);
    }
}
