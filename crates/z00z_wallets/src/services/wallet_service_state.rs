use super::{KeyManagerImpl, ReceiverManagerImpl};

#[derive(Debug, Clone, Copy)]
pub(crate) struct WalletPasswordVerifierState {
    pub(super) salt: [u8; 32],
    pub(super) verifier: [u8; 32],
}

#[derive(Debug)]
pub(crate) struct WalletReceiverDeriverState {
    pub(super) receiver_manager: ReceiverManagerImpl<KeyManagerImpl>,
    pub(super) next_payment_index: u32,
    pub(super) next_change_index: u32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct UnlockAttemptState {
    pub(super) window_start_ms: u64,
    pub(super) window_count: u32,
    pub(super) failed_attempts: u32,
}

impl UnlockAttemptState {
    pub(super) fn new(now_ms: u64) -> Self {
        Self {
            window_start_ms: now_ms,
            window_count: 0,
            failed_attempts: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RateLimitWindowState {
    pub(super) window_start_ms: u64,
    pub(super) window_count: u32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct BackupCreateRateLimitState {
    pub(super) last_created_at: u64,
    pub(super) in_progress: bool,
}

impl RateLimitWindowState {
    pub(super) fn new(now_ms: u64) -> Self {
        Self {
            window_start_ms: now_ms,
            window_count: 0,
        }
    }
}

/// Result of the wallet service rate-limit precheck.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitPrecheck {
    /// The operation can proceed immediately.
    Allowed,
    /// The operation must wait because the rate-limit window is exhausted.
    RateLimited {
        /// Seconds until the caller can retry safely.
        retry_after_seconds: u32,
        /// Requests already observed in the current window.
        current_count: u32,
        /// Maximum requests allowed in the current window.
        max_requests: u32,
    },
}

/// Phase 1 unlock guard precheck result.
///
/// Returned by `WalletService::unlock_attempt_precheck`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnlockAttemptPrecheck {
    Allowed,
    RateLimited {
        retry_after_seconds: u32,
        current_count: u32,
        max_requests: u32,
    },
}
