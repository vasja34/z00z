//! MockTimeProvider - deterministic time for testing

use super::{traits::TimeProvider, TimeError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// Mock time provider for deterministic testing
///
/// This provider returns a fixed time that can be manually controlled.
/// Useful for testing code that depends on time without waiting for actual time to pass.
///
/// # Examples
///
/// ```
/// use z00z_utils::time::{TimeProvider, MockTimeProvider};
/// use std::time::{SystemTime, Duration};
///
/// let time = MockTimeProvider::new(SystemTime::UNIX_EPOCH);
/// assert_eq!(time.compat_unix_timestamp(), 0);
///
/// time.advance_by(Duration::from_secs(60));
/// assert_eq!(time.compat_unix_timestamp(), 60);
/// ```
#[derive(Debug, Clone)]
pub struct MockTimeProvider {
    current_time: Arc<Mutex<SystemTime>>,
}

impl MockTimeProvider {
    /// Create a new mock time provider with the given initial time
    pub fn new(initial_time: SystemTime) -> Self {
        Self {
            current_time: Arc::new(Mutex::new(initial_time)),
        }
    }

    /// Create a provider pinned to the Unix epoch plus whole seconds.
    pub fn from_unix_secs(unix_secs: u64) -> Self {
        Self::new(SystemTime::UNIX_EPOCH + Duration::from_secs(unix_secs))
    }

    /// Create a provider pinned to the Unix epoch plus milliseconds.
    pub fn from_unix_millis(unix_millis: u64) -> Self {
        Self::new(SystemTime::UNIX_EPOCH + Duration::from_millis(unix_millis))
    }

    /// Create a provider pinned before the Unix epoch by whole seconds.
    pub fn before_unix_secs(delta_secs: u64) -> Self {
        Self::new(SystemTime::UNIX_EPOCH - Duration::from_secs(delta_secs))
    }

    /// Create a provider pinned before the Unix epoch by milliseconds.
    pub fn before_unix_millis(delta_millis: u64) -> Self {
        Self::new(SystemTime::UNIX_EPOCH - Duration::from_millis(delta_millis))
    }

    /// Create a provider from the current system clock through the abstraction layer.
    pub fn system_now() -> Self {
        Self::new(SystemTime::now())
    }

    /// Set the time to a specific value
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned (which indicates a panic occurred
    /// while the lock was held). This is acceptable for test code.
    pub fn set_time(&self, time: SystemTime) {
        *self
            .current_time
            .lock()
            .expect("MockTimeProvider lock poisoned - test panic occurred") = time;
    }

    /// Set the time using a Unix-seconds offset from the epoch.
    pub fn set_unix_secs(&self, unix_secs: u64) {
        self.set_time(SystemTime::UNIX_EPOCH + Duration::from_secs(unix_secs));
    }

    /// Set the time using a Unix-milliseconds offset from the epoch.
    pub fn set_unix_millis(&self, unix_millis: u64) {
        self.set_time(SystemTime::UNIX_EPOCH + Duration::from_millis(unix_millis));
    }

    /// Set the time before the Unix epoch by whole seconds.
    pub fn set_before_unix_secs(&self, delta_secs: u64) {
        self.set_time(SystemTime::UNIX_EPOCH - Duration::from_secs(delta_secs));
    }

    /// Set the time before the Unix epoch by milliseconds.
    pub fn set_before_unix_millis(&self, delta_millis: u64) {
        self.set_time(SystemTime::UNIX_EPOCH - Duration::from_millis(delta_millis));
    }

    /// Advance the time by a duration
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned (which indicates a panic occurred
    /// while the lock was held). This is acceptable for test code.
    pub fn advance_by(&self, duration: Duration) {
        *self
            .current_time
            .lock()
            .expect("MockTimeProvider lock poisoned - test panic occurred") += duration;
    }
}

impl Default for MockTimeProvider {
    fn default() -> Self {
        Self::new(SystemTime::UNIX_EPOCH)
    }
}

impl TimeProvider for MockTimeProvider {
    fn now(&self) -> SystemTime {
        *self
            .current_time
            .lock()
            .expect("MockTimeProvider lock poisoned - test panic occurred")
    }

    fn try_unix_timestamp(&self) -> Result<u64, TimeError> {
        Ok(self.now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs())
    }

    fn try_unix_timestamp_ms(&self) -> Result<u64, TimeError> {
        Ok(self
            .now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis() as u64)
    }

    fn try_unix_timestamp_micros(&self) -> Result<u64, TimeError> {
        let duration = self.now().duration_since(SystemTime::UNIX_EPOCH)?;

        match u64::try_from(duration.as_micros()) {
            Ok(value) => Ok(value),
            Err(_) => Ok(u64::MAX),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_time_provider_new() {
        let time = MockTimeProvider::new(SystemTime::UNIX_EPOCH);
        assert_eq!(time.compat_unix_timestamp(), 0);
    }

    #[test]
    fn test_mock_time_provider_set() {
        let time = MockTimeProvider::new(SystemTime::UNIX_EPOCH);
        let new_time = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
        time.set_time(new_time);
        assert_eq!(time.compat_unix_timestamp(), 100);
    }

    #[test]
    fn test_mock_time_provider_advance() {
        let time = MockTimeProvider::new(SystemTime::UNIX_EPOCH);
        time.advance_by(Duration::from_secs(50));
        assert_eq!(time.compat_unix_timestamp(), 50);
        time.advance_by(Duration::from_secs(30));
        assert_eq!(time.compat_unix_timestamp(), 80);
    }

    #[test]
    fn test_mock_time_provider_millis() {
        let time = MockTimeProvider::new(SystemTime::UNIX_EPOCH);
        time.advance_by(Duration::from_millis(500));
        assert_eq!(time.compat_unix_timestamp_millis(), 500);
    }

    #[test]
    fn test_mock_time_provider_shared() {
        let time1 = MockTimeProvider::new(SystemTime::UNIX_EPOCH);
        let time2 = time1.clone();

        time1.advance_by(Duration::from_secs(100));
        // Both should reflect the same time
        assert_eq!(time1.compat_unix_timestamp(), 100);
        assert_eq!(time2.compat_unix_timestamp(), 100);
    }
}
