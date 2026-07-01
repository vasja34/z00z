//! Logger implementation example demonstrating three different loggers.
//!
//! This example shows how to:
//! - Use TracingLogger (production logger)
//! - Use StdoutLogger (dev/debugging)
//! - Use NoopLogger (zero overhead for tests)
//! - Implement the dependency injection pattern with `Arc<dyn Logger>`
//! - Switch between loggers at runtime
//!
//! Run with: `cargo run --package z00z_utils --example logger_demo`

use std::sync::Arc;
use z00z_utils::prelude::*;

/// Example function that accepts a logger trait object
/// This demonstrates dependency injection pattern
fn process_data(logger: Arc<dyn Logger>, items: usize) {
    logger.info("=== Starting data processing ===");
    logger.debug(&format!("Processing {} items", items));

    for i in 0..items {
        logger.trace(&format!("Item {}/{}", i + 1, items));
        if i % 10 == 0 {
            logger.info(&format!("Progress: {}% complete", (i * 100) / items));
        }
    }

    logger.info("=== Processing complete ===");
}

/// Function that uses the logger internally
fn calculate_metrics(logger: Arc<dyn Logger>) -> u64 {
    logger.debug("Starting metric calculation");
    let result = (1..=1000).sum();
    logger.info(&format!("Calculated sum: {}", result));
    result
}

fn main() {
    println!("=== Z00Z Utils: Logger Demo ===\n");

    // Example 1: Using NoopLogger (zero overhead, no output)
    println!("--- NoopLogger (no output) ---");
    let noop_logger: Arc<dyn Logger> = Arc::new(NoopLogger);
    process_data(noop_logger, 100);
    println!("✓ Processing complete (no log output)\n");

    // Example 2: Using StdoutLogger (visible output)
    println!("--- StdoutLogger (visible output) ---");
    let stdout_logger: Arc<dyn Logger> = Arc::new(StdoutLogger);
    process_data(stdout_logger.clone(), 5);
    println!("✓ Processing complete\n");

    // Example 3: Using TracingLogger (production-grade)
    println!("--- TracingLogger (production) ---");
    let tracing_logger: Arc<dyn Logger> = Arc::new(TracingLogger);
    process_data(tracing_logger.clone(), 5);
    println!("✓ Processing complete\n");

    // Example 4: Function using logger internally
    println!("--- Logger in Functions ---");
    let _result = calculate_metrics(stdout_logger.clone());
    println!();

    // Example 5: Demonstrate logger independence
    println!("--- Logger Independence ---");
    println!("Each logger instance is independent:");
    let logger1 = Arc::new(StdoutLogger);
    let logger2 = Arc::new(NoopLogger);

    logger1.info("Logger 1: visible message");
    logger2.info("Logger 2: hidden message (NoopLogger)");
    println!("✓ Loggers work independently\n");

    // Example 6: Demonstrate trait object versatility
    println!("--- Working with Trait Objects ---");
    let loggers: Vec<Arc<dyn Logger>> = vec![
        Arc::new(NoopLogger),
        Arc::new(StdoutLogger),
        Arc::new(TracingLogger),
    ];

    for (i, logger) in loggers.iter().enumerate() {
        logger.debug(&format!("Logger {}: debug message", i + 1));
    }
    println!("✓ Trait objects allow flexible logger usage\n");

    // Example 7: Demonstrate all log levels
    println!("--- Log Levels Example ---");
    let logger = Arc::new(StdoutLogger);
    logger.error("This is an ERROR level message");
    logger.warn("This is a WARN level message");
    logger.info("This is an INFO level message");
    logger.debug("This is a DEBUG level message");
    logger.trace("This is a TRACE level message");

    println!("\n=== Example Completed Successfully ===");
}
