//! SystemTimeProvider - production time implementation

use super::{traits::TimeProvider, TimeError};
use std::time::SystemTime;

/// Production time provider using system clock
///
/// Returns the actual system time. Use this in production code.
/// For testing, use `MockTimeProvider` instead.
///
/// # Examples
///
/// ```
/// use z00z_utils::time::{TimeProvider, SystemTimeProvider};
///
/// let time = SystemTimeProvider;
/// let now = time.now();
/// let timestamp = time.try_unix_timestamp()?;
/// let compat = time.compat_unix_timestamp();
/// assert!(compat >= timestamp);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemTimeProvider;

impl TimeProvider for SystemTimeProvider {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }

    fn try_unix_timestamp(&self) -> Result<u64, TimeError> {
        Ok(SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs())
    }

    fn try_unix_timestamp_ms(&self) -> Result<u64, TimeError> {
        Ok(SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis() as u64)
    }

    fn try_unix_timestamp_micros(&self) -> Result<u64, TimeError> {
        let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;

        match u64::try_from(duration.as_micros()) {
            Ok(value) => Ok(value),
            Err(_) => Ok(u64::MAX),
        }
    }
}
