use std::borrow::Cow;

use z00z_utils::prelude::TimeProvider;

use super::super::assets::AssetError;
use super::nonce_type::try_get_timestamp_micros;

/// 🔢 Persistent nonce counter for wallet
///
/// Maintains a monotonically increasing counter to guarantee nonce uniqueness.
/// CRITICAL: Counter MUST NEVER be reset or decremented.
///
/// # Persistence
///
/// - Stored in wallet database
/// - Incremented atomically within database transaction
/// - Survives wallet restarts
/// - Enables wallet recovery with seed
///
/// # Thread Safety
///
/// Counter operations MUST be wrapped in database transactions
/// to prevent concurrent modification issues.
///
/// # Examples
///
/// ```rust
/// use z00z_core::assets::nonce::NonceCounter;
/// use z00z_utils::time::SystemTimeProvider;
///
/// let mut counter = NonceCounter::new();
/// assert_eq!(counter.value(), 0);
///
/// let time_provider = SystemTimeProvider;
/// counter.increment_unsafe(&time_provider); // Note: Use transaction in production
/// assert_eq!(counter.value(), 1);
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct NonceCounter {
    pub(super) value: u64,
    pub(super) last_updated: u64,
}

impl NonceCounter {
    pub fn new() -> Self {
        Self {
            value: 0,
            last_updated: 0,
        }
    }

    pub fn value(&self) -> u64 {
        self.value
    }

    pub fn last_updated(&self) -> u64 {
        self.last_updated
    }

    #[must_use = "Counter value MUST be persisted in database transaction before use. Nonce reuse = TOTAL PRIVACY LOSS!"]
    pub fn increment_unsafe(
        &mut self,
        time_provider: &dyn TimeProvider,
    ) -> Result<u64, AssetError> {
        #[cfg(debug_assertions)]
        {
            z00z_utils::logger::Logger::warn(
                &z00z_utils::logger::TracingLogger,
                "NonceCounter::increment_unsafe() called; it must stay inside a database transaction to prevent nonce reuse.",
            );
        }

        let timestamp = try_get_timestamp_micros(time_provider).map_err(|err| {
            AssetError::InvalidAsset(Cow::Owned(format!(
                "failed to record nonce counter timestamp: {}",
                err
            )))
        })?;

        self.value = self
            .value
            .checked_add(1)
            .ok_or(AssetError::ArithmeticOverflow(Cow::Borrowed(
                "NonceCounter overflow",
            )))?;
        self.last_updated = timestamp;

        Ok(self.value)
    }

    pub fn set_value_recovery(
        &mut self,
        new_value: u64,
        time_provider: &dyn TimeProvider,
    ) -> Result<(), AssetError> {
        if new_value < self.value {
            return Err(AssetError::InvalidAsset(Cow::Owned(format!(
                "Cannot set counter to lower value: {} < {}",
                new_value, self.value
            ))));
        }

        let timestamp = try_get_timestamp_micros(time_provider).map_err(|err| {
            AssetError::InvalidAsset(Cow::Owned(format!(
                "failed to record nonce counter timestamp: {}",
                err
            )))
        })?;

        self.value = new_value;
        self.last_updated = timestamp;
        Ok(())
    }
}

impl Default for NonceCounter {
    fn default() -> Self {
        Self::new()
    }
}
