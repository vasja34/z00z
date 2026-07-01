//! RPC error types.
//!
//! These errors describe request dispatch and transport-facing failures only.
//! Higher-level policy such as peer identity, authentication, retry policy, and
//! connection lifecycle should be mapped into this surface by owning adapters
//! instead of being implemented by the generic RPC crate itself.

use thiserror::Error;

/// RPC error types.
#[derive(Debug, Error)]
pub enum RpcError {
    /// Transport-level error (network, connection, etc.)
    #[error("Transport error: {0}")]
    TransportError(String),

    /// Request failed at protocol level
    #[error("Request failed: {0}")]
    RequestFailed(String),

    /// Invalid or malformed response
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// Invalid parameters provided in the request
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    /// Requested method does not exist
    #[error("Method not found: {0}")]
    MethodNotFound(String),

    /// Internal server error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Wallet with specified ID not found
    #[error("Wallet not found: {0}")]
    WalletNotFound(String),

    /// Authentication failed (invalid password or session)
    #[error("Authentication failed")]
    AuthFailed,

    /// Session token expired.
    #[error("Session expired")]
    SessionExpired,

    /// Session token is invalid or revoked.
    #[error("Session invalid")]
    SessionInvalid,

    /// Request was rejected by a rate limiter.
    #[error("Rate limited: {0}")]
    RateLimited(String),

    /// Wallet is locked and requires unlocking
    #[error("Wallet locked")]
    WalletLocked,
}
