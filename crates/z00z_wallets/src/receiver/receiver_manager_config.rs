/// Default receiver cache size (entries).
pub const DEFAULT_CACHE_SIZE: usize = 1000;
/// Maximum allowed cache size to prevent memory exhaustion.
pub const MAX_CACHE_SIZE: usize = 100_000;
/// Default cache entry TTL in seconds (1 hour).
pub const DEFAULT_TTL_SECONDS: u64 = 3600;

/// Default threshold for switching async batch derivation to `spawn_blocking`.
///
/// Rationale: small batches are often faster to run inline because the overhead of
/// `spawn_blocking` dominates; larger batches benefit from not blocking the async runtime.
///
/// Tuning guidance:
/// - Use auto-tuning (recommended): `ReceiverManagerConfig { async_batch_threshold: None }`
/// - Use a fixed value only when you have stable hardware and benchmark data
///
/// Benchmarking: see `crates/z00z_wallets/benches/async_batch_threshold_bench.rs`.
pub const ASYNC_BATCH_THRESHOLD: usize = 10;

/// Hard upper bound for async batch threshold configuration.
///
/// This prevents accidental misconfiguration leading to large synchronous workloads on the async runtime.
pub const MAX_ASYNC_BATCH_THRESHOLD: usize = 1000;

/// Receiver manager configuration.
///
/// This struct is intentionally small and serde-friendly so it can be loaded from a config file
/// in higher layers (e.g. wallet service config) when wiring is added.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ReceiverManagerConfig {
    /// Batch size threshold for switching to `spawn_blocking` in async derivation.
    ///
    /// - `None` (default): Auto-tune based on observed derivation performance
    /// - `Some(n)`: Fixed threshold (disables auto-tuning)
    pub async_batch_threshold: Option<usize>,
}

impl ReceiverManagerConfig {
    /// Validate configuration.
    pub fn validate(&self) -> ReceiverManagerResult<()> {
        if let Some(threshold) = self.async_batch_threshold {
            validate_async_batch_threshold(threshold)?;
        }
        Ok(())
    }
}

fn validate_async_batch_threshold(value: usize) -> ReceiverManagerResult<()> {
    if value == 0 || value > MAX_ASYNC_BATCH_THRESHOLD {
        return Err(ReceiverManagerError::InvalidAsyncBatchThreshold(value));
    }
    Ok(())
}

#[inline]
fn use_inline_batch(batch_len: usize, threshold: usize) -> bool {
    batch_len < threshold
}

/// Default purge interval in seconds.
///
/// **Semantics:**
/// - Zero (0): INVALID - rejected at construction (causes `InvalidPurgeInterval` error)
/// - Below minimum (< 1 second): INVALID - rejected (causes `InvalidPurgeInterval` error)
/// - Positive value (≥ 1 second): purge expired entries every N seconds
///
/// **Recommended intervals:**
/// - Single-user wallet: 3600 (1 hour) - minimal memory overhead (DEFAULT)
/// - Multi-user service: 300 (5 minutes) - balance between memory and CPU
/// - High-churn environment: 60 (1 minute) - aggressive cleanup (higher CPU cost)
///
/// **Performance implications:**
/// - Shorter intervals: lower memory usage, higher CPU overhead
/// - Longer intervals: higher memory usage, lower CPU overhead
/// - Purge scans the cache linearly; cost is O(cache_size)
///
/// **Minimum:** 1 second (prevents DoS via excessive purge operations)
pub const DEFAULT_PURGE_INTERVAL_SECONDS: u64 = 3600;

/// Default minimum cache size for purge scans.
pub const DEFAULT_PURGE_MIN_SIZE: usize = 1;

/// Minimum purge interval in seconds (prevents DoS).
pub const MIN_PURGE_INTERVAL_SECONDS: u64 = 1;

/// Maximum number of entries allowed in an imported receiver-cache state.
///
/// Rationale: a typical wallet cache size is in the low thousands; 10k is a generous
/// upper bound that still prevents DoS via unbounded state payload sizes.
pub const MAX_IMPORT_ENTRIES: usize = 10_000;

/// Maximum total payload size allowed in an imported receiver-cache state.
///
/// This is a defense-in-depth limit against malicious state payloads with oversized byte vectors.
pub const MAX_IMPORT_SIZE_BYTES: usize = 10 * 1024 * 1024;

/// Z00Z SLIP-0044 coin type used in BIP-44 paths.
///
/// Value: 1337. This constant is an alias of `Z00Z_BIP44_ASSET`.
const Z00Z_COIN_TYPE: u32 = Z00Z_BIP44_ASSET;
