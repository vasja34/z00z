//! Logger trait tests

use crate::logger::{Logger, NoopLogger, StdoutLogger, TracingLogger};
use std::sync::Arc;
use std::thread;

#[test]
fn test_noop_logger_basic() {
    let logger = NoopLogger;
    // Should not panic
    logger.error("error");
    logger.warn("warn");
    logger.info("info");
    logger.debug("debug");
    logger.trace("trace");
}

#[test]
fn test_stdout_logger_basic() {
    let logger = StdoutLogger;
    // Should not panic
    logger.error("error");
    logger.warn("warn");
    logger.info("info");
    logger.debug("debug");
    logger.trace("trace");
}

#[test]
fn test_tracing_logger_basic() {
    let logger = TracingLogger;
    // Should not panic
    logger.error("error");
    logger.warn("warn");
    logger.info("info");
    logger.debug("debug");
    logger.trace("trace");
}

/// CRITICAL TEST: Verify Logger trait object safety
/// Without object safety, we can't use `Box<dyn Logger>` or `Arc<dyn Logger>`
#[test]
fn test_logger_object_safety() {
    let _: Box<dyn Logger> = Box::new(NoopLogger);
    let _: Box<dyn Logger> = Box::new(StdoutLogger);
    let _: Box<dyn Logger> = Box::new(TracingLogger);

    let _: Arc<dyn Logger> = Arc::new(NoopLogger);
    let _: Arc<dyn Logger> = Arc::new(StdoutLogger);
    let _: Arc<dyn Logger> = Arc::new(TracingLogger);
}

/// CRITICAL TEST: Verify Logger is Send + Sync (thread safe)
#[test]
fn test_logger_thread_safety() {
    let logger = Arc::new(StdoutLogger);
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let logger = Arc::clone(&logger);
            thread::spawn(move || {
                logger.info(&format!("Thread {}", i));
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}
/// Test that different logger implementations can be swapped
#[test]
fn test_logger_polymorphism() {
    fn test_process_with_logger(logger: &dyn Logger) {
        logger.info("Processing...");
        logger.debug("Debug info");
    }

    test_process_with_logger(&NoopLogger);
    test_process_with_logger(&StdoutLogger);
    test_process_with_logger(&TracingLogger);
}
