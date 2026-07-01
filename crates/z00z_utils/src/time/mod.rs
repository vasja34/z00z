//! Time provider trait and implementations
//!
//! This module provides a time provider abstraction to enable deterministic
//! testing and flexible time handling in production code.
//!
//! Production guidance:
//! - use `try_unix_timestamp*()` for fail-closed security-sensitive flows
//! - use `compat_unix_timestamp*()` only for explicitly non-security callers
//!
//! # Timestamp Naming & Units Convention
//!
//! This crate follows a strict naming convention to keep time units unambiguous:
//!
//! - `*_at` fields represent a point in time as a unix timestamp in **milliseconds** (`u64`, ms since epoch).
//!   Examples: `created_at`, `updated_at`, `expires_at`.
//! - `*_ms` / `*_secs` fields represent **durations** (timeouts, backoffs), not points in time.
//!   Examples: `timeout_secs`, `backoff_ms`.
//! - If a timestamp is intentionally stored in seconds, it must be explicit in the name
//!   (for example `*_at_secs`).
//!
//! Human-facing output must always be rendered in UTC using the shared formatters:
//! - `format_unix_timestamp_millis_utc(..)` for `*_at` fields
//! - `format_unix_timestamp_secs_utc(..)` for seconds-based timestamps
//!
//! # Examples
//!
//! ```
//! use z00z_utils::time::{TimeProvider, SystemTimeProvider};
//!
//! let time = SystemTimeProvider;
//! let now = time.now();
//! let unix = time.try_unix_timestamp()?;
//! let compat = time.compat_unix_timestamp();
//! assert!(compat >= unix);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod error;
mod format;
mod mock;
mod system;
mod traits;

pub use error::TimeError;
pub use format::{
    format_system_time_local, format_system_time_utc, format_unix_timestamp_millis_utc,
    format_unix_timestamp_milliseconds_compact, format_unix_timestamp_secs_utc,
};
pub use mock::MockTimeProvider;
pub use system::SystemTimeProvider;
pub use traits::TimeProvider;

pub use std::time::{Instant, SystemTime};

#[cfg(test)]
mod test_time;
