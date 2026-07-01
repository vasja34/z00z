// Copyright © 2025 Z00Z Project. All rights reserved.
// Licensed under the Apache License, Version 2.0

//! Auto-lock policy and timeout management.
//!
//! This module implements automatic wallet locking based on inactivity timeout
//! and system events (suspend, screen lock).

use serde::{Deserialize, Serialize};

// Type alias for std::time::Duration to avoid direct import
type Duration = std::time::Duration;

/// Auto-lock policy configuration.
///
/// Defines timeout duration and triggers for automatic wallet locking.
///
/// # Examples
///
/// ```
/// use z00z_wallets::wallet::{AutoLockPolicy, LockTrigger};
///
/// let policy = AutoLockPolicy {
///     timeout: std::time::Duration::from_secs(15 * 60), // 15 minutes
///     triggers: vec![
///         LockTrigger::SystemSuspend,
///         LockTrigger::ScreenLock,
///     ],
/// };
///
/// assert_eq!(policy.timeout.as_secs(), 900);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoLockPolicy {
    /// Timeout duration for inactivity-based locking.
    ///
    /// Default: 15 minutes (900 seconds)
    pub timeout: Duration,

    /// Events that trigger immediate wallet locking.
    pub triggers: Vec<LockTrigger>,
}

impl Default for AutoLockPolicy {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(15 * 60), // 15 minutes
            triggers: vec![
                LockTrigger::SystemSuspend,
                LockTrigger::ScreenLock,
                LockTrigger::Manual,
            ],
        }
    }
}

impl AutoLockPolicy {
    /// Create a custom auto-lock policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use z00z_wallets::wallet::{AutoLockPolicy, LockTrigger};
    ///
    /// let policy = AutoLockPolicy::new(
    ///     std::time::Duration::from_secs(5 * 60), // 5 minutes
    ///     vec![LockTrigger::SystemSuspend, LockTrigger::Manual],
    /// );
    ///
    /// assert_eq!(policy.timeout.as_secs(), 300);
    /// ```
    pub fn new(timeout: Duration, triggers: Vec<LockTrigger>) -> Self {
        Self { timeout, triggers }
    }

    /// Check if a trigger is enabled in this policy.
    pub fn has_trigger(&self, trigger: &LockTrigger) -> bool {
        self.triggers.contains(trigger)
    }
}

/// Events that trigger immediate wallet locking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LockTrigger {
    /// System suspend or hibernate event.
    ///
    /// Detected via platform-specific APIs:
    /// - Linux: D-Bus org.freedesktop.login1.Manager PrepareForSleep
    /// - macOS: NSWorkspace willSleepNotification
    /// - Windows: WM_POWERBROADCAST PBT_APMSUSPEND
    SystemSuspend,

    /// Screen lock detected.
    ///
    /// Detected via platform-specific APIs:
    /// - Linux: D-Bus org.freedesktop.ScreenSaver ActiveChanged
    /// - macOS: NSWorkspace screenIsLocked
    /// - Windows: WTSSessionLock
    ScreenLock,

    /// Application backgrounded (mobile platforms).
    ///
    /// Triggered when app loses focus or moves to background.
    AppBackgrounded,

    /// Manual lock request from user.
    ///
    /// Triggered via UI "Lock Wallet" action or RPC call.
    Manual,
}

#[cfg(test)]
mod tests {
    use super::*;
    use z00z_utils::codec::{Codec, JsonCodec};

    #[test]
    fn test_default_policy() {
        let policy = AutoLockPolicy::default();

        assert_eq!(policy.timeout.as_secs(), 900); // 15 minutes
        assert_eq!(policy.triggers.len(), 3);
        assert!(policy.has_trigger(&LockTrigger::SystemSuspend));
        assert!(policy.has_trigger(&LockTrigger::ScreenLock));
        assert!(policy.has_trigger(&LockTrigger::Manual));
        assert!(!policy.has_trigger(&LockTrigger::AppBackgrounded));
    }

    #[test]
    fn test_custom_policy() {
        let policy = AutoLockPolicy::new(
            Duration::from_secs(5 * 60), // 5 minutes
            vec![LockTrigger::Manual],
        );

        assert_eq!(policy.timeout.as_secs(), 300);
        assert_eq!(policy.triggers.len(), 1);
        assert!(policy.has_trigger(&LockTrigger::Manual));
        assert!(!policy.has_trigger(&LockTrigger::SystemSuspend));
    }

    #[test]
    fn test_has_trigger() {
        let policy = AutoLockPolicy {
            timeout: Duration::from_secs(600),
            triggers: vec![LockTrigger::SystemSuspend, LockTrigger::ScreenLock],
        };

        assert!(policy.has_trigger(&LockTrigger::SystemSuspend));
        assert!(policy.has_trigger(&LockTrigger::ScreenLock));
        assert!(!policy.has_trigger(&LockTrigger::Manual));
        assert!(!policy.has_trigger(&LockTrigger::AppBackgrounded));
    }

    #[test]
    fn test_serialize_deserialize() {
        let policy = AutoLockPolicy::default();
        let codec = JsonCodec;
        let json_bytes = codec.serialize(&policy).unwrap();
        let deserialized: AutoLockPolicy = codec.deserialize(&json_bytes).unwrap();

        assert_eq!(policy.timeout, deserialized.timeout);
        assert_eq!(policy.triggers.len(), deserialized.triggers.len());
    }
}
