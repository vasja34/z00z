//! StdoutLogger implementation
//!
//! A simple logger that writes to stdout and stderr.
//! Used for development and debugging.

use crate::logger::Logger;

/// Simple stdout/stderr logger for development and debugging
///
/// Writes log messages to stdout (info, debug, trace) or stderr (error, warn).
/// Useful for quick debugging and development where full tracing infrastructure
/// is not available.
///
/// # Examples
///
/// ```
/// use z00z_utils::logger::{Logger, StdoutLogger};
///
/// let logger = StdoutLogger;
/// logger.info("Application started");
/// logger.error("An error occurred");
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct StdoutLogger;

impl Logger for StdoutLogger {
    fn error(&self, msg: &str) {
        eprintln!("[ERROR] {}", msg);
    }

    fn warn(&self, msg: &str) {
        eprintln!("[WARN]  {}", msg);
    }

    fn info(&self, msg: &str) {
        println!("[INFO]  {}", msg);
    }

    fn debug(&self, msg: &str) {
        println!("[DEBUG] {}", msg);
    }

    fn trace(&self, msg: &str) {
        println!("[TRACE] {}", msg);
    }
}
