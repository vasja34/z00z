//! TimeProvider trait definition

use super::TimeError;
use std::time::SystemTime;

/// Time provider abstraction for production and testing
///
/// This trait allows code to be independent of the actual time source,
/// enabling deterministic testing via mock implementations while using
/// real system time in production.
///
/// # Clock Error Handling
///
/// The `try_unix_timestamp*()` methods are the canonical production contract
/// for nonce, expiry, ordering, anti-replay, and other security-sensitive
/// flows. They surface clock failures instead of silently collapsing them into
/// a sentinel value.
///
/// The `compat_unix_timestamp*()` helpers are an explicit compatibility surface
/// for non-security paths that intentionally tolerate a zero fallback when the
/// system clock is before the Unix epoch (January 1, 1970).
///
/// # Examples
///
/// ```no_run
/// use z00z_utils::time::{TimeProvider, SystemTimeProvider};
///
/// let time = SystemTimeProvider;
/// let timestamp = time.try_unix_timestamp()?;
/// let millis = time.try_unix_timestamp_ms()?;
/// let micros = time.try_unix_timestamp_micros()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait TimeProvider: Send + Sync {
    /// Get the current system time
    fn now(&self) -> SystemTime;

    /// Get the current unix timestamp in seconds with explicit error handling.
    fn try_unix_timestamp(&self) -> Result<u64, TimeError> {
        Ok(self.now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs())
    }

    /// Get the current unix timestamp in milliseconds with explicit error handling.
    fn try_unix_timestamp_ms(&self) -> Result<u64, TimeError> {
        Ok(self
            .now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis() as u64)
    }

    /// Get the current unix timestamp in microseconds with explicit error handling.
    fn try_unix_timestamp_micros(&self) -> Result<u64, TimeError> {
        let duration = self.now().duration_since(SystemTime::UNIX_EPOCH)?;

        match u64::try_from(duration.as_micros()) {
            Ok(value) => Ok(value),
            Err(_) => Ok(u64::MAX),
        }
    }

    /// Get the current unix timestamp in seconds for compatibility-only callers.
    ///
    /// Returns 0 if the system clock is set before the Unix epoch.
    /// Use `try_unix_timestamp()` in security-critical code.
    fn compat_unix_timestamp(&self) -> u64 {
        self.try_unix_timestamp().unwrap_or(0)
    }

    /// Get the current unix timestamp in milliseconds for compatibility-only callers.
    ///
    /// Returns 0 if the system clock is set before the Unix epoch.
    /// Use `try_unix_timestamp_ms()` in security-critical code.
    fn compat_unix_timestamp_millis(&self) -> u64 {
        self.try_unix_timestamp_ms().unwrap_or(0)
    }

    /// Get the current unix timestamp in microseconds for compatibility-only callers.
    ///
    /// This returns the number of microseconds since Unix epoch, truncated from the underlying
    /// system time resolution (typically nanoseconds).
    ///
    /// Returns 0 if the system clock is set before the Unix epoch.
    /// Use `try_unix_timestamp_micros()` in security-critical code.
    fn compat_unix_timestamp_micros(&self) -> u64 {
        self.try_unix_timestamp_micros().unwrap_or(0)
    }
}
