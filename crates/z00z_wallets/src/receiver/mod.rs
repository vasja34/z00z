//! Receiver management and derivation
//!
//! This module provides receiver card, payment request, and scan management functionality.
//! For BIP-32/BIP-44 key derivation, use `crate::key`.
//! Spec 6 is a consolidation program over the existing receiver stack, not a
//! greenfield receiver redesign. Ownership is frozen here as follows:
//! - `asset_leaf_scan` is the canonical full-leaf detection authority.
//! - `wallet_asset_scanner` is the wallet-runtime asset adapter.

pub mod manager;
pub use self::manager::canonical_state;
/// Stealth ownership verification helpers for asset import path.
#[path = "asset_ownership.rs"]
pub mod asset_ownership;
pub use self::asset_ownership::asset_ownership_check;
/// Canonical scan helpers and receiver ownership checks.
#[path = "asset_scan.rs"]
pub mod asset_scan;
pub use self::asset_scan::asset_leaf_scan;
pub use self::asset_scan::asset_scan_ephemeral_cache;
/// Receiver card helpers and metadata validation.
pub mod card;
pub use self::asset_scan::asset_batch_scanner;
pub use self::asset_scan::scan_rate_limiter;
pub use self::card::nfc_ndef;
pub use self::card::receiver_card;
/// Stealth payment request data model, encoding, and validation.
pub mod request;
/// Advisory request-bound inbox metadata.
pub mod request_inbox;
/// Output scanning primitives for the stealth receiver protocol.
// ONE SOURCE OF TRUTH network type
pub use crate::ChainType;

// Re-export all public types
pub use self::manager::{
    AsyncReceiverManager, AsyncReceiverManagerImpl, DerivedWalletKeys, ReceiverCacheState,
    ReceiverManager, ReceiverManagerConfig, ReceiverManagerError, ReceiverManagerImpl,
    ReceiverManagerResult, ASYNC_BATCH_THRESHOLD, DEFAULT_CACHE_SIZE, MAX_ASYNC_BATCH_THRESHOLD,
    MAX_CACHE_SIZE,
};
pub use asset_batch_scanner::OptimizedScanner;
pub use asset_leaf_scan::{receiver_scan_leaf, receiver_scan_report};
pub use asset_ownership_check::check_stealth_own;
pub use asset_scan::{
    CacheStats, DetectedAssetPack, DoSMitigation, ReceiveNext, ReceiveReject, ReceiveReport,
    ReceiveStatus, ScanChunk, ScanDecision, ScanRangeOut, ScanRangeStat, ScanResult, ScanStrategy,
    StealthOutputScanner, Tag16Cache, Tag16CacheState, Tag16Context, WalletReveal,
    WalletStealthOutput,
};
pub use asset_scan_ephemeral_cache::EphemeralCache;
pub use card::receiver_card_trust::{PinEntry, PinnedReceiverCards, TrustLevel, VerifyResult};
pub use nfc_ndef::nfc_ndef_record;
pub use receiver_card::{
    decode_card_compact, encode_card_compact, CardMetadata, ReceiverCard, ReceiverCardError,
    ValidateReceiverCard,
};
pub use request::{
    create_invoice_for_merchant, PaymentRequest, PaymentRequestError, RequestMetadata,
    RequestParams, ValidatePaymentRequest, ValidatedRequest, ValidationOutcome, ValidityStatus,
};
pub use request_inbox::{
    RequestInbox, RequestInboxRecord, RequestInboxReject, RequestInboxValidation, RequestRangeHint,
    RequestRecipientBinding,
};
pub use scan_rate_limiter::{ScanRateError, ScanRateLimiter};
