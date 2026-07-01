//! Timestamp formatting helpers.
//!
//! 📌 Purpose:
//! - Keep a single, consistent conversion from numeric unix timestamps to a human-readable
//!   string format.
//! - Always format in UTC to avoid local timezone confusion in logs.
//!
//! 🎯 Format:
//! - `YYYY-MM-DD HH:MM:SS.mmm` (UTC)

use chrono::{Local, TimeZone as _, Utc};
use std::time::SystemTime;

/// Format a unix timestamp in seconds as a UTC timestamp string.
///
/// 📌 Output format: `YYYY-MM-DD HH:MM:SS.mmm`.
///
/// Since seconds do not carry millisecond precision, this function renders `.000`.
pub fn format_unix_timestamp_secs_utc(unix_secs: u64) -> String {
    let unix_millis = unix_secs.saturating_mul(1000);
    format_unix_timestamp_millis_utc(unix_millis)
}

/// Format a unix timestamp in milliseconds as a UTC timestamp string.
///
/// 📌 Output format: `YYYY-MM-DD HH:MM:SS.mmm`.
///
/// Returns `"invalid-timestamp"` if the value is outside Chrono's supported range.
pub fn format_unix_timestamp_millis_utc(unix_millis: u64) -> String {
    let unix_secs: i64 = match i64::try_from(unix_millis / 1000) {
        Ok(v) => v,
        Err(_) => return "invalid-timestamp".to_string(),
    };

    let ms: u32 = (unix_millis % 1000) as u32;
    let nanos = ms.saturating_mul(1_000_000);

    match Utc.timestamp_opt(unix_secs, nanos).single() {
        Some(dt) => dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
        None => "invalid-timestamp".to_string(),
    }
}

/// Format a unix timestamp in milliseconds as a compact UTC timestamp string.
///
/// 📌 Output format: `YYYYMMDD_HHMMSS`.
///
/// Intended for filename-safe timestamps.
///
/// Returns `"invalid-timestamp"` if the value is outside Chrono's supported range.
pub fn format_unix_timestamp_milliseconds_compact(unix_millis: u64) -> String {
    let unix_secs: i64 = match i64::try_from(unix_millis / 1000) {
        Ok(v) => v,
        Err(_) => return "invalid-timestamp".to_string(),
    };

    match Utc.timestamp_opt(unix_secs, 0).single() {
        Some(dt) => dt.format("%Y%m%d_%H%M%S").to_string(),
        None => "invalid-timestamp".to_string(),
    }
}

/// Format a `SystemTime` as a UTC timestamp string.
///
/// 📌 Output format: `YYYY-MM-DD HH:MM:SS.mmm`.
///
/// Returns `"invalid-timestamp"` if conversion fails.
pub fn format_system_time_utc(time: SystemTime) -> String {
    let dt = chrono::DateTime::<Utc>::from(time);
    dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

/// Format a `SystemTime` as a local timestamp string.
///
/// 📌 Output format: `YYYY-MM-DD HH:MM:SS.mmm`.
pub fn format_system_time_local(time: SystemTime) -> String {
    let dt = chrono::DateTime::<Utc>::from(time).with_timezone(&Local);
    dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}
