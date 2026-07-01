//! App RPC types
//!
//! Request/response types for app.* JSON-RPC methods.
//!
//! Timestamp naming convention:
//! - `*_at`: point in time (milliseconds since Unix epoch)
//! - `*_ms` / `*_secs`: durations

use serde::{Deserialize, Serialize};
use z00z_utils::time::{format_unix_timestamp_millis_utc, TimeProvider};

/// View logs response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeViewLogsResponse {
    pub logs: Vec<RuntimeLogEntry>,
}

/// Log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeLogEntry {
    /// Timestamp (milliseconds since Unix epoch).
    pub timestamp: u64,
    pub level: String,
    pub message: String,
}

impl RuntimeLogEntry {
    /// Create a log entry with an explicit timestamp.
    pub fn new(timestamp: u64, level: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            timestamp,
            level: level.into(),
            message: message.into(),
        }
    }

    /// Create a log entry using the provided time source.
    pub fn new_now(
        time: &impl TimeProvider,
        level: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::new(time.compat_unix_timestamp_millis(), level, message)
    }

    /// Format the timestamp in UTC for human-readable output.
    pub fn format_time(&self) -> String {
        format_unix_timestamp_millis_utc(self.timestamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z00z_utils::codec::{Codec, JsonCodec};
    use z00z_utils::time::MockTimeProvider;

    #[test]
    fn test_runtime_log_entry_serialization() {
        let entry = RuntimeLogEntry {
            timestamp: 1_700_000_000_000,
            level: "INFO".to_string(),
            message: "Test log message".to_string(),
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&entry).unwrap();
        let deserialized: RuntimeLogEntry = codec.deserialize(&bytes).unwrap();

        assert_eq!(deserialized.timestamp, 1_700_000_000_000);
        assert_eq!(deserialized.level, "INFO");
        assert_eq!(deserialized.message, "Test log message");
    }

    #[test]
    fn test_runtime_log_entry_new() {
        let entry = RuntimeLogEntry::new(123456, "WARN", "Warning message");
        assert_eq!(entry.timestamp, 123456);
        assert_eq!(entry.level, "WARN");
        assert_eq!(entry.message, "Warning message");
    }

    #[test]
    fn test_runtime_entry_new_now() {
        let time = MockTimeProvider::from_unix_millis(1_700_000_000_000);
        let entry = RuntimeLogEntry::new_now(&time, "ERROR", "Error occurred");
        assert_eq!(entry.timestamp, 1_700_000_000_000);
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.message, "Error occurred");
    }

    #[test]
    fn test_empty_log_message() {
        let entry = RuntimeLogEntry::new(0, "DEBUG", "");
        assert!(entry.message.is_empty());
    }

    #[test]
    fn test_different_log_levels() {
        let levels = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR"];
        for level in levels {
            let entry = RuntimeLogEntry::new(1000, level, "test");
            assert_eq!(entry.level, level);
        }
    }

    #[test]
    fn test_response_with_empty_logs() {
        let response = RuntimeViewLogsResponse { logs: vec![] };

        let codec = JsonCodec;
        let bytes = codec.serialize(&response).unwrap();
        let deserialized: RuntimeViewLogsResponse = codec.deserialize(&bytes).unwrap();

        assert!(deserialized.logs.is_empty());
    }

    #[test]
    fn test_runtime_entry_format_time() {
        let entry = RuntimeLogEntry {
            timestamp: 1_700_000_000_000,
            level: "INFO".to_string(),
            message: "Test".to_string(),
        };

        let formatted = entry.format_time();
        assert!(!formatted.is_empty());
        assert!(formatted.contains("2023"));
    }

    #[test]
    fn test_timestamp_milliseconds_format() {
        // Test that format_time produces expected format
        let entry = RuntimeLogEntry {
            timestamp: 1_700_000_000_000,
            level: "INFO".to_string(),
            message: "Test".to_string(),
        };

        let formatted = entry.format_time();
        // Should contain year 2023 and time components
        assert!(formatted.contains("2023"));
    }
}
