//! Logger trait definition

/// Logging abstraction for dependency injection
///
/// This trait eliminates the need for hardcoded `tracing::` calls throughout
/// the codebase. Instead, loggers can be injected at construction time,
/// making code testable and flexible.
///
/// # Design Notes
///
/// - Takes `&str` parameters (not `fmt::Arguments`) for object safety
/// - Trait objects are fully supported: `Box<dyn Logger>`, `Arc<dyn Logger>`
/// - All methods are non-blocking and synchronous
///
/// # Examples
///
/// ```
/// use z00z_utils::logger::{Logger, NoopLogger};
///
/// let logger: Box<dyn Logger> = Box::new(NoopLogger);
/// logger.error("An error occurred");
/// logger.warn("Warning message");
/// logger.info("Info message");
/// logger.debug("Debug message");
/// logger.trace("Trace message");
/// ```
pub trait Logger: Send + Sync {
    /// Log an error message
    fn error(&self, msg: &str);

    /// Log a warning message
    fn warn(&self, msg: &str);

    /// Log an info message
    fn info(&self, msg: &str);

    /// Log a debug message
    fn debug(&self, msg: &str);

    /// Log a trace message
    fn trace(&self, msg: &str);
}
