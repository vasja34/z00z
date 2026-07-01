use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use thiserror::Error;
use z00z_utils::time::{SystemTimeProvider, TimeProvider};

/// Errors for scan rate limiting.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ScanRateError {
    /// Rate limit exceeded for current second bucket.
    #[error("scan rate exceeded")]
    RateLimitExceeded,
    /// System clock is unavailable for fail-closed rate limiting.
    #[error("scan rate clock unavailable")]
    ClockUnavailable,
}

/// Rate limiter for scan operations.
#[derive(Debug)]
pub struct ScanRateLimiter {
    max_scans_per_sec: u32,
    second_bucket: AtomicU64,
    current_load: AtomicU32,
}

impl ScanRateLimiter {
    fn current_second_bucket() -> Result<u64, ScanRateError> {
        SystemTimeProvider
            .try_unix_timestamp()
            .map_err(|_| ScanRateError::ClockUnavailable)
    }

    /// Creates rate limiter with max scans per second.
    pub fn new(max_scans_per_sec: u32) -> Self {
        Self {
            max_scans_per_sec,
            second_bucket: AtomicU64::new(Self::current_second_bucket().unwrap_or(u64::MAX)),
            current_load: AtomicU32::new(0),
        }
    }

    /// Checks and increments scan load counter.
    pub fn check_scan_load(&self) -> Result<(), ScanRateError> {
        let now = Self::current_second_bucket()?;
        let mut bucket = self.second_bucket.load(Ordering::Relaxed);
        while bucket != now {
            match self.second_bucket.compare_exchange_weak(
                bucket,
                now,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    self.current_load.store(0, Ordering::Release);
                    break;
                }
                Err(observed) => bucket = observed,
            }
        }

        let mut current = self.current_load.load(Ordering::Relaxed);
        loop {
            if current >= self.max_scans_per_sec {
                return Err(ScanRateError::RateLimitExceeded);
            }

            match self.current_load.compare_exchange_weak(
                current,
                current + 1,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(()),
                Err(observed) => current = observed,
            }
        }
    }

    /// Resets limiter bucket for next time window.
    pub fn reset(&self) {
        self.second_bucket.store(
            Self::current_second_bucket().unwrap_or(u64::MAX),
            Ordering::Relaxed,
        );
        self.current_load.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::{ScanRateError, ScanRateLimiter};

    #[test]
    fn test_rate_limiter() {
        let limiter = ScanRateLimiter::new(2);
        assert!(limiter.check_scan_load().is_ok());
        assert!(limiter.check_scan_load().is_ok());
        assert_eq!(
            limiter.check_scan_load(),
            Err(ScanRateError::RateLimitExceeded)
        );

        limiter.reset();
        assert!(limiter.check_scan_load().is_ok());
    }
}
