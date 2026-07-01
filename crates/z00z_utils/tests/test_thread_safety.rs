/// Integration tests for thread safety across all modules
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use z00z_utils::prelude::{
    Logger, MetricsSink, NoopLogger, NoopMetrics, StdoutLogger, SystemTimeProvider, TimeProvider,
};
#[test]
fn test_system_time_provider_concurrent() {
    let time_provider: Arc<dyn TimeProvider> = Arc::new(SystemTimeProvider);
    let mut handles = vec![];

    // Spawn threads that read time
    for _i in 0..10 {
        let time_clone = Arc::clone(&time_provider);
        let handle = thread::spawn(move || {
            for _j in 0..100 {
                let _ts = time_clone.compat_unix_timestamp();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("thread panicked");
    }
}

#[test]
fn test_multiple_logger_types_concurrent() {
    let loggers: Vec<Arc<dyn Logger>> = vec![Arc::new(NoopLogger), Arc::new(StdoutLogger)];

    let mut handles = vec![];

    for (idx, logger) in loggers.into_iter().enumerate() {
        for i in 0..10 {
            let logger_clone = Arc::clone(&logger);
            let handle = thread::spawn(move || {
                for j in 0..50 {
                    logger_clone.info(&format!("logger {} thread {} msg {}", idx, i, j));
                }
            });
            handles.push(handle);
        }
    }

    for handle in handles {
        handle.join().expect("thread panicked");
    }
}

#[test]
fn test_combined_trait_usage_concurrent() {
    let logger: Arc<dyn Logger> = Arc::new(NoopLogger);
    let metrics: Arc<dyn MetricsSink> = Arc::new(NoopMetrics);
    let time_provider: Arc<dyn TimeProvider> = Arc::new(SystemTimeProvider);

    let mut handles = vec![];

    for i in 0..10 {
        let logger_clone = Arc::clone(&logger);
        let metrics_clone = Arc::clone(&metrics);
        let time_clone = Arc::clone(&time_provider);

        let handle = thread::spawn(move || {
            for j in 0..100 {
                logger_clone.debug(&format!("Processing {} {}", i, j));
                metrics_clone.inc_counter("operations", 1);
                let _ts = time_clone.compat_unix_timestamp();
                metrics_clone.observe_histogram("timing", (i * j) as f64);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("thread panicked");
    }
}

#[test]
fn test_logger_thread_pool_simulation() {
    // Simulate a thread pool where multiple threads use same logger
    let logger: Arc<dyn Logger> = Arc::new(NoopLogger);
    let task_count = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    // 4 worker threads
    for worker_id in 0..4 {
        let logger_clone = Arc::clone(&logger);
        let task_counter = Arc::clone(&task_count);

        let handle = thread::spawn(move || {
            // Each worker processes 25 tasks
            for task_id in 0..25 {
                logger_clone.info(&format!("Worker {} processing task {}", worker_id, task_id));

                // Simulate work
                std::thread::sleep(Duration::from_millis(1));

                *task_counter.lock().unwrap() += 1;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("thread panicked");
    }

    // Verify all tasks were processed
    assert_eq!(*task_count.lock().unwrap(), 100);
}
