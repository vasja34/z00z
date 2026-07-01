//! Time-related error types.

use thiserror::Error;

/// Errors that can occur when querying system time.
#[derive(Debug, Error)]
pub enum TimeError {
    /// System clock is before Unix epoch.
    #[error("System time is before Unix epoch: {0}")]
    BeforeEpoch(#[from] std::time::SystemTimeError),
}
