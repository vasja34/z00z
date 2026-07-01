//! Receiver key management with caching.
//!
//! This module provides centralized receiver-key derivation and caching for Z00Z wallets.
//! All receiver-key operations flow through the `ReceiverManager` trait, which ensures:
//! - Deterministic key derivation via BIP-44 paths
//! - Performance optimization through in-memory caching
//! - Consistent error handling
//!
//! # Architecture
//!
//! ```text
//! RPC Layer → WalletService → ReceiverManager → KeyManager → Crypto
//! ```
//!
//! # Cache Behavior
//!
//! The cache is **in-memory** and is not automatically persisted across restarts.
//! For diagnostics and future persistence integrations, the cache supports explicit
//! authenticated state export/import via `ReceiverCacheState`.
//! For deterministic recovery, use gap-limit scanning with batch derivation.
//!
//! # Features
//!
//! - **Cache Size Limits**: Prevents memory exhaustion via LRU eviction
//! - **TTL Expiration**: Time-based cache entry expiration
//! - **Batch Derivation**: Efficient gap-limit scanning
//! - **Async Support**: Non-blocking operations for long derivations
//! - **Metrics**: Cache performance observability
//!
//! # BIP-44 Path Enforcement
//!
//! Receiver-key derivation enforces strict Z00Z BIP-44 paths:
//! - `m/44'/1337'/account'/0/index` (payment/external)
//! - `m/44'/1337'/account'/1/index` (change/internal)
//! - Invalid example: `m/44'/0'/0'/0/0` (wrong coin type)

use crate::db::ScanStatePayload;
use crate::key::{
    Bip44Error, Bip44Path, Bip44Validator, Bip44ViolationReason, KeyManager, ReceiverKeys,
    Z00Z_BIP44_ASSET,
};
use z00z_core::Asset;
#[path = "manager_canonical_state.rs"]
pub mod canonical_state;

#[cfg(test)]
use crate::domains::ReceiverCacheHmacTestDomain;

use self::canonical_state::{to_canonical, ReceiverCacheEntry};
use super::{
    PaymentRequest, ReceiverCard, RequestParams, ScanChunk, ScanRangeOut, StealthOutputScanner,
    WalletStealthOutput,
};
#[cfg(not(test))]
use crate::domains::ReceiverCacheHmacProdDomain;
use async_trait::async_trait;
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc, RwLock,
    },
    time::{Duration, Instant},
};
use subtle::ConstantTimeEq;
use thiserror::Error;
use z00z_crypto::expert::encoding::ByteArray;
use z00z_crypto::expert::traits::DomainSeparation;
use z00z_crypto::{hash::hmac_sha256, hkdf_expand_32, Z00ZRistrettoPoint};
use z00z_utils::time::TimeProvider;

#[cfg(all(
    not(target_arch = "wasm32"),
    debug_assertions,
    feature = "eviction-logs"
))]
use std::sync::mpsc::SyncSender;

#[cfg(test)]
type ReceiverCacheHmacDomain = ReceiverCacheHmacTestDomain;

#[cfg(not(test))]
type ReceiverCacheHmacDomain = ReceiverCacheHmacProdDomain;

/// Receiver manager errors.
#[derive(Debug, Error)]
pub enum ReceiverManagerError {
    /// Receiver key is not available in the cache.
    #[error("receiver key not found for path: {0}")]
    NotFound(Bip44Path),

    /// Provided derivation path is invalid for the current wallet configuration.
    #[error("invalid derivation path: {0}")]
    InvalidPath(String),

    /// Provided derivation path uses an unexpected coin type.
    #[error("invalid coin type: {0}")]
    InvalidCoinType(u32),

    /// Invalid cache size provided (must be > 0).
    #[error("invalid cache size: {0}, must be > 0")]
    InvalidCacheSize(usize),

    /// Invalid async batch threshold provided.
    #[error("invalid async batch threshold: {0}")]
    InvalidAsyncBatchThreshold(usize),

    /// Key derivation failed.
    #[error("key derivation failed: {0}")]
    KeyDerivation(String),

    /// Derived key is the identity point (all-zero bytes).
    #[error("derived key is identity point")]
    IdentityKeyRejected,

    /// Derived receiver keys are not independent.
    #[error("receiver keys are not independent")]
    ReceiverKeysNotIndependent,

    /// Import rejected due to too many entries.
    #[error("import too large: {0} entries exceeds limit")]
    ImportTooLarge(usize),

    /// Import rejected due to exceeding total payload size.
    #[error("import exceeds size limit: {0} bytes")]
    ImportExceedsSizeLimit(usize),

    /// Import rejected due to invalid snapshot entry.
    #[error("import entry rejected: {0}")]
    ImportEntryRejected(String),

    /// Receiver-cache state authentication failed.
    #[error("cache authentication failed")]
    CacheAuthenticationFailed,

    /// Receiver-cache state is missing or has an invalid signature.
    #[error("invalid cache signature")]
    InvalidCacheSignature,

    /// System clock is unavailable for fail-closed receiver manager paths.
    #[error("receiver manager clock unavailable: {0}")]
    ClockUnavailable(String),

    /// Receiver-cache state encoding/decoding failed.
    #[error("receiver cache state codec error: {0}")]
    ReceiverCacheCodec(String),

    /// Receiver-cache state serialization failed (Phase 7: canonical format validation).
    #[error("receiver cache state serialization failed: {0}")]
    InvalidReceiverCacheState(String),

    /// Cache lock is poisoned.
    #[error("cache lock poisoned")]
    LockPoisoned,

    /// MAC key derivation via HKDF failed.
    #[error("MAC key derivation failed")]
    MacKeyDerivationFailed,

    /// Rate limiter lock is poisoned.
    #[error("rate limiter lock poisoned")]
    RateLimiterPoisoned,

    /// Invalid rate limit configuration.
    #[error("invalid rate limit configuration: rate_per_sec={rate_per_sec}, burst={burst}")]
    InvalidRateLimit {
        /// Sustained rate (tokens per second).
        rate_per_sec: u32,
        /// Burst allowance (maximum bucket capacity).
        burst: u32,
    },

    /// Rate limit exceeded (Phase 14: DoS protection).
    #[error("rate limit exceeded - too many derivation requests")]
    RateLimitExceeded,

    /// Batch size exceeds burst allowance (Phase 14: DoS protection).
    #[error("batch size {requested} exceeds maximum {max_allowed}")]
    BatchTooLarge {
        /// Requested batch size.
        requested: u32,
        /// Maximum allowed batch size.
        max_allowed: u32,
    },

    /// Purge interval cannot be zero (Phase 26).
    #[error("purge interval cannot be zero")]
    InvalidPurgeInterval,

    /// Stealth integration operation failed.
    #[error("stealth integration failed: {0}")]
    StealthIntegration(String),
}

/// Address-manager result type.
pub type ReceiverManagerResult<T> = std::result::Result<T, ReceiverManagerError>;

include!("receiver_manager_cache.rs");
include!("receiver_manager_trait.rs");
include!("manager_eviction_listener.rs");
include!("receiver_manager_config.rs");
include!("manager_rate_limiter_bucket.rs");
include!("receiver_manager_impl.rs");

#[cfg(test)]
mod tests {
    include!("test_receiver_manager_suite.rs");
}
