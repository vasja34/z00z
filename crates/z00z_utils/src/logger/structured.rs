//! Structured event logging abstractions.

use crate::logger::Logger;

/// Log level used for structured events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Error level.
    Error,
    /// Warning level.
    Warn,
    /// Info level.
    Info,
    /// Debug level.
    Debug,
    /// Trace level.
    Trace,
}

/// Log level provider for structured events.
pub trait EventLevel {
    /// Returns log level for this event.
    fn log_level(&self) -> LogLevel;
}

/// Object-safe structured event type.
pub trait LogEvent: erased_serde::Serialize + EventLevel {}

impl<T> LogEvent for T where T: erased_serde::Serialize + EventLevel {}

erased_serde::serialize_trait_object!(LogEvent);

/// Structured logger extension for all `Logger` implementations.
pub trait StructuredLogger: Logger {
    /// Logs typed event as JSON and routes it by event level.
    fn log_event(&self, event: &dyn LogEvent) {
        let payload = encode_event(event).unwrap_or_else(serialize_error_payload);

        match event.log_level() {
            LogLevel::Error => self.error(&payload),
            LogLevel::Warn => self.warn(&payload),
            LogLevel::Info => self.info(&payload),
            LogLevel::Debug => self.debug(&payload),
            LogLevel::Trace => self.trace(&payload),
        }
    }
}

impl<T> StructuredLogger for T where T: Logger + ?Sized {}

fn serialize_error_payload() -> String {
    "{\"event\":\"logger.serialize_error\",\"message\":\"structured log serialization failed\"}"
        .to_string()
}

fn encode_event(event: &dyn LogEvent) -> Option<String> {
    let mut bytes = Vec::new();
    let mut serializer = serde_json::Serializer::new(&mut bytes);
    if erased_serde::serialize(event, &mut serializer).is_err() {
        return None;
    }

    String::from_utf8(bytes).ok()
}
