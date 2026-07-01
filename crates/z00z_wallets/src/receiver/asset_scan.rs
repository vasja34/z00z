//! Scan utilities for address discovery and ownership checks.

/// Batch-optimized scanner wrapper for parallel leaf scanning.
#[path = "asset_batch_scanner.rs"]
pub mod asset_batch_scanner;
/// Canonical full-leaf detection authority for receiver ownership checks.
#[path = "asset_leaf_scan.rs"]
pub mod asset_leaf_scan;
/// Ephemeral sender key dedup cache.
#[path = "asset_scan_ephemeral_cache.rs"]
pub mod asset_scan_ephemeral_cache;
/// Private shared receiver scan primitives.
#[path = "asset_scan_support.rs"]
pub(crate) mod asset_scan_support;
/// Rate limiting primitives for scan flows.
#[path = "scan_rate_limiter.rs"]
pub mod scan_rate_limiter;
/// Wallet-runtime asset adapter for receiver scan entry points.
#[path = "wallet_asset_scanner.rs"]
mod wallet_asset_scanner;

#[path = "asset_scan_types.rs"]
mod asset_scan_types;

pub use self::wallet_asset_scanner::{
    CacheStats, DetectedAssetPack, DoSMitigation, ReceiveNext, ReceiveReject, ReceiveReport,
    ReceiveStatus, ScanChunk, ScanDecision, ScanRangeErr, ScanRangeOut, ScanRangeStat, ScanResult,
    ScanStrategy, StealthOutputScanner, Tag16Cache, Tag16CacheState, Tag16Context, WalletReveal,
    WalletStealthOutput,
};
