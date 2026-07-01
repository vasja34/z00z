// Copyright © 2025 Z00Z Project. All rights reserved.
// Licensed under the Apache License, Version 2.0

//! Wallet state machine definitions and transition logic.
//!
//! This module provides atomic state management for wallets using a formal
//! state machine with validated transitions.

use serde::{Deserialize, Serialize};

/// Wallet lifecycle states.
///
/// # State Transitions
///
/// ```text
/// NotExists → Locked (create_wallet)
/// Locked → Unlocked (unlock)
/// Unlocked → Locked (lock/timeout)
/// Unlocked → Closing (close)
/// Locked → Closing (close)
/// Any → Error (error)
/// ```
///
/// # Examples
///
/// ```
/// use z00z_wallets::wallet::WalletState;
///
/// let state = WalletState::Locked;
/// let unlocked = WalletState::Unlocked {
///     session_start_ms: 0,
///     last_activity_ms: 0,
/// };
///
/// // Validate transition
/// assert!(state.can_transition_to(&unlocked));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WalletState {
    /// Wallet does not exist yet.
    NotExists,

    /// Created but encrypted, requires password to unlock.
    Locked,

    /// Active session with keys loaded in memory.
    ///
    /// Contains session metadata for auto-lock functionality.
    Unlocked {
        /// When the wallet was unlocked (Unix timestamp in milliseconds).
        session_start_ms: u64,

        /// Last activity timestamp (Unix timestamp in milliseconds).
        last_activity_ms: u64,
    },

    /// Graceful shutdown in progress.
    ///
    /// Wallet is cleaning up resources before deletion.
    Closing,

    /// Unrecoverable error state.
    ///
    /// Contains error reason for diagnostics.
    Error {
        /// Human-readable error description.
        reason: String,
    },
}

impl WalletState {
    /// Validates if a state transition is allowed.
    ///
    /// # State Transition Rules
    ///
    /// - `NotExists → Locked`: Create new wallet
    /// - `Locked → Unlocked`: Unlock with password
    /// - `Unlocked → Locked`: Manual lock or auto-lock timeout
    /// - `Unlocked → Closing`: Close unlocked wallet
    /// - `Locked → Closing`: Close locked wallet
    /// - `Any → Error`: Error can occur from any state
    ///
    /// # Examples
    ///
    /// ```
    /// use z00z_wallets::wallet::WalletState;
    ///
    /// let locked = WalletState::Locked;
    /// let unlocked = WalletState::Unlocked {
    ///     session_start_ms: 0,
    ///     last_activity_ms: 0,
    /// };
    ///
    /// // Valid transition
    /// assert!(locked.can_transition_to(&unlocked));
    ///
    /// // Invalid transition
    /// let not_exists = WalletState::NotExists;
    /// assert!(!not_exists.can_transition_to(&unlocked));
    /// ```
    pub fn can_transition_to(&self, new_state: &WalletState) -> bool {
        use WalletState::*;

        matches!(
            (self, new_state),
            // Valid transitions
            (NotExists, Locked)
                | (Locked, Unlocked { .. })
                | (Unlocked { .. }, Locked)
                | (Unlocked { .. }, Closing)
                | (Locked, Closing)
                | (_, Error { .. }) // Error from any state
        )
    }

    /// Returns `true` if wallet is locked.
    pub fn is_locked(&self) -> bool {
        matches!(self, WalletState::Locked)
    }

    /// Returns `true` if wallet is unlocked.
    pub fn is_unlocked(&self) -> bool {
        matches!(self, WalletState::Unlocked { .. })
    }

    /// Returns `true` if wallet is closing.
    pub fn is_closing(&self) -> bool {
        matches!(self, WalletState::Closing)
    }

    /// Returns `true` if wallet is in error state.
    pub fn is_error(&self) -> bool {
        matches!(self, WalletState::Error { .. })
    }

    /// Returns `true` if wallet does not exist.
    pub fn is_not_exists(&self) -> bool {
        matches!(self, WalletState::NotExists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        let not_exists = WalletState::NotExists;
        let locked = WalletState::Locked;
        let unlocked = WalletState::Unlocked {
            session_start_ms: 0,
            last_activity_ms: 0,
        };
        let closing = WalletState::Closing;
        let error = WalletState::Error {
            reason: "test error".into(),
        };

        // NotExists → Locked (create)
        assert!(not_exists.can_transition_to(&locked));

        // Locked → Unlocked (unlock)
        assert!(locked.can_transition_to(&unlocked));

        // Unlocked → Locked (lock)
        assert!(unlocked.can_transition_to(&locked));

        // Unlocked → Closing (close)
        assert!(unlocked.can_transition_to(&closing));

        // Locked → Closing (close locked wallet)
        assert!(locked.can_transition_to(&closing));

        // Any → Error
        assert!(not_exists.can_transition_to(&error));
        assert!(locked.can_transition_to(&error));
        assert!(unlocked.can_transition_to(&error));
        assert!(closing.can_transition_to(&error));
    }

    #[test]
    fn test_invalid_transitions() {
        let not_exists = WalletState::NotExists;
        let locked = WalletState::Locked;
        let unlocked = WalletState::Unlocked {
            session_start_ms: 0,
            last_activity_ms: 0,
        };
        let closing = WalletState::Closing;

        // Cannot unlock non-existent wallet
        assert!(!not_exists.can_transition_to(&unlocked));

        // Cannot close non-existent wallet
        assert!(!not_exists.can_transition_to(&closing));

        // Cannot unlock closing wallet
        assert!(!closing.can_transition_to(&unlocked));

        // Cannot lock closing wallet
        assert!(!closing.can_transition_to(&locked));

        // Cannot return to NotExists
        assert!(!locked.can_transition_to(&not_exists));
        assert!(!unlocked.can_transition_to(&not_exists));
        assert!(!closing.can_transition_to(&not_exists));
    }

    #[test]
    fn test_state_query_methods() {
        let locked = WalletState::Locked;
        assert!(locked.is_locked());
        assert!(!locked.is_unlocked());
        assert!(!locked.is_closing());
        assert!(!locked.is_error());
        assert!(!locked.is_not_exists());

        let unlocked = WalletState::Unlocked {
            session_start_ms: 0,
            last_activity_ms: 0,
        };
        assert!(!unlocked.is_locked());
        assert!(unlocked.is_unlocked());
        assert!(!unlocked.is_closing());

        let closing = WalletState::Closing;
        assert!(closing.is_closing());
        assert!(!closing.is_locked());

        let error = WalletState::Error {
            reason: "test".into(),
        };
        assert!(error.is_error());

        let not_exists = WalletState::NotExists;
        assert!(not_exists.is_not_exists());
    }
}
