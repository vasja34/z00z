use serde::Serialize;
use std::fs;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use z00z_utils::codec::{Codec, JsonCodec, Value};
/// Integration tests for logger module with various implementations
use z00z_utils::logger::{
    EventLevel, FileLogger, LogLevel, Logger, NoopLogger, RotatingFileLogger, RotationPolicy,
    StdoutLogger, StructuredLogger, VecLogger,
};
#[test]
fn test_logger_with_counters() {
    // Simulate a logger that tracks calls (useful for testing)
    #[derive(Clone)]
    struct CountingLogger {
        counter: Arc<Mutex<usize>>,
    }

    impl Logger for CountingLogger {
        fn error(&self, _message: &str) {
            *self.counter.lock().unwrap() += 1;
        }
        fn warn(&self, _message: &str) {
            *self.counter.lock().unwrap() += 1;
        }
        fn info(&self, _message: &str) {
            *self.counter.lock().unwrap() += 1;
        }
        fn debug(&self, _message: &str) {
            *self.counter.lock().unwrap() += 1;
        }
        fn trace(&self, _message: &str) {
            *self.counter.lock().unwrap() += 1;
        }
    }

    let logger = CountingLogger {
        counter: Arc::new(Mutex::new(0)),
    };

    logger.info("msg1");
    logger.warn("msg2");
    logger.error("msg3");
    logger.debug("msg4");
    logger.trace("msg5");

    assert_eq!(*logger.counter.lock().unwrap(), 5);
}

#[test]
fn test_logger_message_formatting() {
    let logger = StdoutLogger;

    // Test various message types
    logger.info("simple message");
    logger.info(&format!("formatted {}", "message"));
    logger.info("message with special chars: !@#$%^&*()");
    logger.info("unicode: 你好世界 🌍");
    logger.info("very long message that is much longer than typical log messages and should still work correctly without any issues");
}

#[test]
fn test_logger_empty_messages() {
    let logger = NoopLogger;

    // Should handle empty strings gracefully
    logger.info("");
    logger.error("");
    logger.warn("");
    logger.debug("");
    logger.trace("");
}
#[test]
fn test_sanitizes_escape_control_bytes() {
    let dir = tempdir().expect("create temp dir");
    let log_path = dir.path().join("sanitize.log");
    let logger = FileLogger::new(&log_path).expect("create logger");

    logger.info("hello\n\u{1b}[31mred\u{1b}[0m\tworld\u{7}\0END \u{1b}]broken-tail");

    let content = fs::read_to_string(&log_path).expect("read log");
    assert!(content.contains("[INFO]"));
    assert!(content.contains("hello\\nred\\tworld\\x07END \\x1B]broken-tail"));
    assert!(!content.contains('\u{1b}'));
    assert!(!content.contains('\u{7}'));
}

#[test]
fn test_rotating_preserves_severity_prefix() {
    let dir = tempdir().expect("create temp dir");
    let log_path = dir.path().join("rotate.log");
    let logger = RotatingFileLogger::new(
        &log_path,
        RotationPolicy {
            max_bytes: 1024,
            keep_files: 1,
        },
    )
    .expect("create rotating logger");

    logger.error("rotating\n\u{1b}[31mmessage\u{1b}[0m\t\u{7}END \u{1b}]broken-tail");

    let content = fs::read_to_string(&log_path).expect("read log");
    assert!(content.contains("[ERROR] rotating\\nmessage\\t\\x07END \\x1B]broken-tail"));
    assert!(!content.contains('\u{1b}'));
    assert!(!content.contains('\u{7}'));
}

#[test]
fn test_macros_codec_compat_json() {
    let logger = VecLogger::new();

    z00z_utils::log_info!(&logger, event = "compat", count = 2, ok = true);

    let lines = logger.lines();
    assert_eq!(lines.len(), 1);

    let payload = lines[0]
        .split_once("] ")
        .map(|(_, payload)| payload)
        .expect("vec logger payload");
    let parsed: Value = JsonCodec
        .deserialize(payload.as_bytes())
        .expect("decode macro payload");

    assert_eq!(
        parsed.get("event").and_then(|value| value.as_str()),
        Some("compat")
    );
    assert_eq!(
        parsed.get("count").and_then(|value| value.as_u64()),
        Some(2)
    );
    assert_eq!(
        parsed.get("ok").and_then(|value| value.as_bool()),
        Some(true)
    );
}

#[derive(Debug)]
struct FailingEvent;

impl EventLevel for FailingEvent {
    fn log_level(&self) -> LogLevel {
        LogLevel::Warn
    }
}

impl Serialize for FailingEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(serde::ser::Error::custom(format!(
            "forced failure via {}",
            serializer.is_human_readable()
        )))
    }
}

#[test]
fn test_serialization_keeps_explicit_sentinel() {
    let logger = VecLogger::new();

    logger.log_event(&FailingEvent);

    let lines = logger.lines();
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("[WARN]"));
    assert!(lines[0].contains("logger.serialize_error"));
    assert!(lines[0].contains("structured log serialization failed"));
}
