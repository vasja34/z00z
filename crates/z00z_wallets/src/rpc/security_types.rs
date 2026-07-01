//! Security types for audit logging and rate limiting
//!
//! This module defines types for security infrastructure including:
//! - Audit log entries for CRITICAL operations
//! - Rate limiting tiers and quotas
//! - Risk level classification
//! - Security error codes
//!
//! # Architecture Compliance
//!
//! - ✅ Serializable types (for persistent audit logs)
//! - ✅ Timestamp tracking (Unix milliseconds)
//! - ✅ IP and User-Agent tracking for forensics
//! - ✅ Risk-based classification

use std::fmt;

use serde::{Deserialize, Serialize};

use super::common::PersistWalletId;

pub use super::common::{RuntimeEncryptedResponse, RuntimeEncryptionMetadata};

pub use crate::security::password::RuntimePasswordPolicy;

/// Audit log entry for security-sensitive operations
///
/// All CRITICAL and HIGH risk operations use this schema.
/// Phase 1 may keep entries in a process-local sink before a durable append-only backend is wired.
///
/// Note: `Persist*` names the durable target schema. The current wallet RPC implementation still
/// uses a process-local store for testing, and that is intentionally distinct from
/// `app.rs::RuntimeLogEntry`, which is an ephemeral view-model for log viewing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistAuditLogEntry {
    /// Event timestamp (milliseconds since Unix epoch)
    pub timestamp: u64,
    /// Wallet identifier (if applicable)
    pub wallet_id: Option<PersistWalletId>,
    /// RPC method name
    pub method: String,
    /// Client IP address (for rate limiting and forensics)
    pub client_ip: Option<String>,
    /// Client User-Agent header
    pub user_agent: Option<String>,
    /// Operation result
    pub result: AuditResult,
    /// Risk level classification
    pub risk_level: RiskLevel,
    /// Additional context (optional)
    pub context: Option<String>,
}

/// Audit operation result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditResult {
    /// Operation succeeded
    Success,
    /// Operation failed (validation, business logic)
    Failure,
    /// Operation blocked by rate limiter
    RateLimited,
    /// Operation denied (authentication, authorization)
    Denied,
}

/// Risk level for security classification
///
/// Determines audit retention, alerting, and monitoring thresholds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// CRITICAL: Irreversible operations (seed export, wallet delete)
    /// - Always log
    /// - Alert on every occurrence
    /// - Retain logs indefinitely
    Critical,
    /// HIGH: Sensitive operations (unlock, send tx)
    /// - Always log
    /// - Alert on failed attempts
    /// - Retain logs 365 days
    High,
    /// MEDIUM: Configuration changes (settings update, policy change)
    /// - Log on change
    /// - No automatic alerts
    /// - Retain logs 90 days
    Medium,
    /// LOW: Read operations (list wallets, get balance)
    /// - Optional logging
    /// - No alerts
    /// - Retain logs 7 days
    Low,
}

/// Rate limiting tier configuration
///
/// Defines request quotas per method category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeRateLimitTier {
    /// Tier name
    pub name: &'static str,
    /// Requests per minute
    pub requests_per_minute: u32,
    /// Requests per hour (optional, overrides minute calculation)
    pub requests_per_hour: Option<u32>,
    /// Exponential backoff on failure (for auth methods)
    pub exponential_backoff: bool,
}

#[cfg(test)]
#[path = "test_security_types.rs"]
mod tests;

impl RuntimeRateLimitTier {
    /// Tier 1: High-risk operations (seed export, wallet delete)
    pub const HIGH_RISK: Self = Self {
        name: "high_risk",
        requests_per_minute: 3,
        requests_per_hour: Some(10),
        exponential_backoff: false,
    };

    /// Tier 2: Write operations (tx send, wallet create)
    pub const WRITE_OPS: Self = Self {
        name: "write_ops",
        requests_per_minute: 10,
        requests_per_hour: None,
        exponential_backoff: true, // For unlock failures
    };

    /// Tier 3: Read operations (list, get balance)
    pub const READ_OPS: Self = Self {
        name: "read_ops",
        requests_per_minute: 100,
        requests_per_hour: None,
        exponential_backoff: false,
    };

    /// Tier 4: Bulk operations (build unsigned tx)
    pub const BULK_OPS: Self = Self {
        name: "bulk_ops",
        requests_per_minute: 20,
        requests_per_hour: None,
        exponential_backoff: false,
    };
}

/// Rate limit violation error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeRateLimitError {
    /// Method that was rate limited
    pub method: String,
    /// Rate limit tier
    pub tier: String,
    /// Current request count in window
    pub current_count: u32,
    /// Maximum allowed requests
    pub max_requests: u32,
    /// Window size (seconds)
    pub window_seconds: u32,
    /// Retry after (seconds)
    pub retry_after_seconds: u32,
}

/// Session token with permissions and expiration
///
/// Bound to specific wallet, expires after inactivity.
#[derive(Clone, Serialize, Deserialize)]
pub struct SessionToken {
    /// Token string (cryptographically random, 32 bytes hex)
    pub token: String,
    /// Wallet identifier this token is bound to
    pub wallet_id: PersistWalletId,
    /// Creation timestamp (milliseconds since Unix epoch)
    pub created_at: u64,
    /// Expiration timestamp (milliseconds since Unix epoch)
    pub expires_at: u64,
    /// Last activity timestamp (milliseconds since Unix epoch)
    pub last_activity_at: u64,
    /// Granular permissions (optional, for future use)
    pub permissions: Vec<String>,
}

impl fmt::Debug for SessionToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionToken")
            .field("token", &"<redacted>")
            .field("wallet_id", &self.wallet_id)
            .field("created_at", &self.created_at)
            .field("expires_at", &self.expires_at)
            .field("last_activity_at", &self.last_activity_at)
            .field("permissions", &self.permissions)
            .finish()
    }
}

/// Security error codes for RPC responses
///
/// Maps to JSON-RPC 2.0 error codes in -32400 to -32499 range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityErrorCode {
    /// Authentication failed (wrong password) - -32401
    AuthenticationFailed = -32401,
    /// Session token expired - -32402
    SessionExpired = -32402,
    /// Session token invalid - -32403
    SessionInvalid = -32403,
    /// Insufficient permissions - -32404
    PermissionDenied = -32404,
    /// Account locked (too many failed attempts) - -32423
    AccountLocked = -32423,
    /// Rate limit exceeded - -32429
    RateLimitExceeded = -32429,
}

impl SecurityErrorCode {
    /// Get human-readable error message
    pub fn message(&self) -> &'static str {
        match self {
            Self::AuthenticationFailed => "Authentication failed: invalid password",
            Self::SessionExpired => "Session expired: please unlock wallet again",
            Self::SessionInvalid => "Session invalid: token not found or revoked",
            Self::RateLimitExceeded => "Rate limit exceeded: too many requests",
            Self::PermissionDenied => "Permission denied: insufficient privileges",
            Self::AccountLocked => "Account locked: too many failed attempts",
        }
    }

    /// Get error code as i32 for JSON-RPC
    pub fn code(&self) -> i32 {
        *self as i32
    }
}

/// Idempotency key for preventing duplicate transactions
///
/// Clients provide this key with tx.send to prevent double-spending
/// due to retries or network issues.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IdempotencyKey(pub String);

impl IdempotencyKey {
    /// Create new idempotency key from string
    pub fn new(key: String) -> Self {
        Self(key)
    }

    /// Validate key format (UUID v4 recommended)
    pub fn is_valid(&self) -> bool {
        // Basic validation: 32+ chars, alphanumeric + hyphens
        self.0.len() >= 32 && self.0.chars().all(|c| c.is_alphanumeric() || c == '-')
    }
}

/// Idempotency cache entry
///
/// Stores result of previous operation to return cached response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeIdempotencyCacheEntry {
    /// Idempotency key
    pub key: IdempotencyKey,
    /// Wallet identifier
    pub wallet_id: PersistWalletId,
    /// Method name
    pub method: String,
    /// Cached response (JSON string)
    pub response: String,
    /// Creation timestamp (Unix milliseconds)
    pub created_at: u64,
    /// Expiration timestamp (Unix milliseconds)
    pub expires_at: u64,
}

#[cfg(test)]
mod debug_tests {
    use super::*;

    #[test]
    fn test_session_debug_redacts() {
        let token = SessionToken {
            token: String::from("raw-session-token"),
            wallet_id: PersistWalletId(String::from("wallet-1")),
            created_at: 1,
            expires_at: 2,
            last_activity_at: 3,
            permissions: vec![String::from("admin")],
        };

        let rendered = format!("{token:?}");
        assert!(rendered.contains("SessionToken"));
        assert!(rendered.contains("<redacted>"));
        assert!(!rendered.contains("raw-session-token"));
    }
}
