/// Derived wallet public keys for dual-address construction.
#[derive(Clone, Debug)]
pub struct DerivedWalletKeys {
    /// Public spend key derived from the canonical spend path.
    pub spend_key: Z00ZRistrettoPoint,
    /// Public view key derived from the deterministic view-key path.
    pub view_key: Z00ZRistrettoPoint,
}

/// Cache entry with timestamp for TTL management.
#[derive(Debug, Clone)]
pub struct CachedEntry {
    keys: DerivedWalletKeys,
    inserted_at: u64, // Unix timestamp in seconds
}

const CACHE_ENTRY_EST_BYTES: u64 = std::mem::size_of::<(Bip44Path, CachedEntry)>() as u64;

/// Cache performance metrics.
///
/// Uses atomic counters for thread-safe updates from both sync and async contexts.
#[derive(Debug, Default)]
pub struct CacheMetrics {
    /// Number of cache hits (from derive_spend_key)
    pub hits: AtomicU64,
    /// Number of cache misses (from derive_spend_key)
    pub misses: AtomicU64,
    /// Number of entries evicted
    pub evictions: AtomicU64,
    /// Total number of derivation attempts
    pub total_derivations: AtomicU64,
    /// Number of entries expired by TTL
    pub ttl_expirations: AtomicU64,
    /// Total number of cache lookups (get_receiver_key calls)
    pub total_lookups: AtomicU64,
    /// Number of cache hits from get_receiver_key
    pub lookup_hits: AtomicU64,
    /// Number of cache misses from get_receiver_key
    pub lookup_misses: AtomicU64,
    /// Current cache size
    pub current_cache_size: AtomicU64,
    /// Peak cache size observed
    pub peak_cache_size: AtomicU64,

    /// Number of rejected import attempts (all-or-nothing atomicity).
    pub import_rejects: AtomicU64,
    /// Total number of entries attempted to import.
    pub import_entries: AtomicU64,
    /// Total number of bytes attempted to import.
    pub import_bytes: AtomicU64,

    /// Total number of purge scans executed.
    pub purge_runs: AtomicU64,
    /// Total number of entries removed by purge scans.
    pub purge_entries: AtomicU64,
    /// Estimated bytes reclaimed by purge scans.
    pub purge_bytes: AtomicU64,

    /// Total derivation time in milliseconds (for auto-tuning).
    pub total_derive_time_ms: AtomicU64,
    /// Number of derivations tracked for timing (for auto-tuning).
    pub derive_count: AtomicU64,
}

const METRIC_SAT_MARGIN: u64 = 1_000_000;
const METRIC_SAT_THRESHOLD: u64 = u64::MAX - METRIC_SAT_MARGIN;

impl CacheMetrics {
    #[inline(always)]
    fn add_sat(counter: &AtomicU64, delta: u64, name: &str) {
        // Saturating counters prevent wraparound at 2^64, which can silently
        // corrupt long-running telemetry.
        //
        // Monitoring: emit a warning when the counter is close to u64::MAX.
        //
        // To avoid log spam, warn only when crossing the threshold.
        let prev = match counter.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| {
            Some(x.saturating_add(delta))
        }) {
            Ok(prev) => prev,
            Err(prev) => prev,
        };

        let new_val = prev.saturating_add(delta);
        if prev < METRIC_SAT_THRESHOLD && CacheMetricsSnapshot::is_near_sat(new_val) {
            z00z_utils::logger::Logger::warn(
                &z00z_utils::logger::TracingLogger,
                &format!(
                    "Metric counter '{}' approaching saturation: {} / {}",
                    name,
                    new_val,
                    u64::MAX
                ),
            );
        }
    }

    #[inline(always)]
    /// Increment cache hit counter.
    pub fn inc_hits(&self) {
        Self::add_sat(&self.hits, 1, "hits");
    }

    #[inline(always)]
    /// Increment cache miss counter.
    pub fn inc_misses(&self) {
        Self::add_sat(&self.misses, 1, "misses");
    }

    #[inline(always)]
    /// Increment cache eviction counter.
    pub fn inc_evictions(&self) {
        Self::add_sat(&self.evictions, 1, "evictions");
    }

    #[inline(always)]
    /// Increment total derivation counter.
    pub fn inc_total_derivations(&self) {
        Self::add_sat(&self.total_derivations, 1, "total_derivations");
    }

    #[inline(always)]
    /// Increment TTL expiration counter.
    pub fn inc_ttl_expirations(&self) {
        Self::add_sat(&self.ttl_expirations, 1, "ttl_expirations");
    }

    #[inline(always)]
    /// Increment total lookup counter.
    pub fn inc_total_lookups(&self) {
        Self::add_sat(&self.total_lookups, 1, "total_lookups");
    }

    #[inline(always)]
    /// Increment lookup hit counter.
    pub fn inc_lookup_hits(&self) {
        Self::add_sat(&self.lookup_hits, 1, "lookup_hits");
    }

    #[inline(always)]
    /// Increment lookup miss counter.
    pub fn inc_lookup_misses(&self) {
        Self::add_sat(&self.lookup_misses, 1, "lookup_misses");
    }

    #[inline(always)]
    /// Increment cache import rejection counter.
    pub fn inc_import_rejects(&self) {
        Self::add_sat(&self.import_rejects, 1, "import_rejects");
    }

    #[inline(always)]
    /// Add a delta to the number of imported entries.
    pub fn add_import_entries(&self, delta: u64) {
        Self::add_sat(&self.import_entries, delta, "import_entries");
    }

    #[inline(always)]
    /// Add a delta to the number of imported bytes.
    pub fn add_import_bytes(&self, delta: u64) {
        Self::add_sat(&self.import_bytes, delta, "import_bytes");
    }

    #[inline(always)]
    /// Increment purge run counter.
    pub fn inc_purge_runs(&self) {
        Self::add_sat(&self.purge_runs, 1, "purge_runs");
    }

    #[inline(always)]
    /// Add a delta to the number of purged entries.
    pub fn add_purge_entries(&self, delta: u64) {
        Self::add_sat(&self.purge_entries, delta, "purge_entries");
    }

    #[inline(always)]
    /// Add a delta to the number of purged bytes.
    pub fn add_purge_bytes(&self, delta: u64) {
        Self::add_sat(&self.purge_bytes, delta, "purge_bytes");
    }

    #[inline(always)]
    /// Add derivation time in milliseconds (for auto-tuning).
    pub fn add_derive_time(&self, ms: u64) {
        Self::add_sat(&self.total_derive_time_ms, ms, "total_derive_time_ms");
        Self::add_sat(&self.derive_count, 1, "derive_count");
    }

    /// Calculate average derivation time in milliseconds.
    pub fn avg_derive_time_ms(&self) -> f64 {
        let count = self.derive_count.load(Ordering::Relaxed);
        if count == 0 {
            return 0.0;
        }
        self.total_derive_time_ms.load(Ordering::Relaxed) as f64 / count as f64
    }

    /// Calculate cache hit rate for derivations (0.0 to 1.0).
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_derivations.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        self.hits.load(Ordering::Relaxed) as f64 / total as f64
    }

    /// Calculate overall cache hit rate including lookups.
    pub fn overall_hit_rate(&self) -> f64 {
        let total = self.total_derivations.load(Ordering::Relaxed)
            + self.total_lookups.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        (self.hits.load(Ordering::Relaxed) + self.lookup_hits.load(Ordering::Relaxed)) as f64
            / total as f64
    }

    /// Get total cache operations count.
    pub fn total_operations(&self) -> u64 {
        self.total_derivations.load(Ordering::Relaxed) + self.total_lookups.load(Ordering::Relaxed)
    }

    /// Get current cache size.
    pub fn current_cache_size(&self) -> u64 {
        self.current_cache_size.load(Ordering::Relaxed)
    }

    /// Get peak cache size.
    pub fn peak_cache_size(&self) -> u64 {
        self.peak_cache_size.load(Ordering::Relaxed)
    }

    /// Reset all metrics to zero.
    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
        self.total_derivations.store(0, Ordering::Relaxed);
        self.ttl_expirations.store(0, Ordering::Relaxed);
        self.total_lookups.store(0, Ordering::Relaxed);
        self.lookup_hits.store(0, Ordering::Relaxed);
        self.lookup_misses.store(0, Ordering::Relaxed);
        self.current_cache_size.store(0, Ordering::Relaxed);
        self.peak_cache_size.store(0, Ordering::Relaxed);
        self.import_rejects.store(0, Ordering::Relaxed);
        self.import_entries.store(0, Ordering::Relaxed);
        self.import_bytes.store(0, Ordering::Relaxed);
        self.purge_runs.store(0, Ordering::Relaxed);
        self.purge_entries.store(0, Ordering::Relaxed);
        self.purge_bytes.store(0, Ordering::Relaxed);
        self.total_derive_time_ms.store(0, Ordering::Relaxed);
        self.derive_count.store(0, Ordering::Relaxed);
    }

    /// Get a snapshot of all metrics.
    pub fn snapshot(&self) -> CacheMetricsSnapshot {
        CacheMetricsSnapshot {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            total_derivations: self.total_derivations.load(Ordering::Relaxed),
            ttl_expirations: self.ttl_expirations.load(Ordering::Relaxed),
            total_lookups: self.total_lookups.load(Ordering::Relaxed),
            lookup_hits: self.lookup_hits.load(Ordering::Relaxed),
            lookup_misses: self.lookup_misses.load(Ordering::Relaxed),
            current_cache_size: self.current_cache_size.load(Ordering::Relaxed),
            peak_cache_size: self.peak_cache_size.load(Ordering::Relaxed),
            import_rejects: self.import_rejects.load(Ordering::Relaxed),
            import_entries: self.import_entries.load(Ordering::Relaxed),
            import_bytes: self.import_bytes.load(Ordering::Relaxed),
            purge_runs: self.purge_runs.load(Ordering::Relaxed),
            purge_entries: self.purge_entries.load(Ordering::Relaxed),
            purge_bytes: self.purge_bytes.load(Ordering::Relaxed),
            total_derive_time_ms: self.total_derive_time_ms.load(Ordering::Relaxed),
            derive_count: self.derive_count.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of cache metrics (for copying/cloning).
#[derive(Debug, Clone, Default)]
pub struct CacheMetricsSnapshot {
    /// Number of successful cache hits.
    pub hits: u64,
    /// Number of cache misses.
    pub misses: u64,
    /// Number of evictions performed by the cache.
    pub evictions: u64,
    /// Total number of key derivations requested.
    pub total_derivations: u64,
    /// Number of entries expired due to TTL.
    pub ttl_expirations: u64,
    /// Total number of lookups (including cache reads).
    pub total_lookups: u64,
    /// Number of lookup hits.
    pub lookup_hits: u64,
    /// Number of lookup misses.
    pub lookup_misses: u64,
    /// Current number of cached entries.
    pub current_cache_size: u64,
    /// Maximum observed cache size.
    pub peak_cache_size: u64,

    /// Number of rejected import attempts (all-or-nothing atomicity).
    pub import_rejects: u64,
    /// Total number of entries attempted to import.
    pub import_entries: u64,
    /// Total number of bytes attempted to import.
    pub import_bytes: u64,

    /// Total number of purge scans executed.
    pub purge_runs: u64,
    /// Total number of entries removed by purge scans.
    pub purge_entries: u64,
    /// Estimated bytes reclaimed by purge scans.
    pub purge_bytes: u64,

    /// Total derivation time in milliseconds (for auto-tuning).
    pub total_derive_time_ms: u64,
    /// Number of derivations tracked for timing (for auto-tuning).
    pub derive_count: u64,
}

impl CacheMetricsSnapshot {
    #[inline(always)]
    fn is_near_sat(val: u64) -> bool {
        val >= METRIC_SAT_THRESHOLD
    }

    /// Cache hit rate for derivations only.
    pub fn hit_rate(&self) -> f64 {
        if self.total_derivations == 0 {
            return 0.0;
        }
        self.hits as f64 / self.total_derivations as f64
    }

    /// Overall hit rate, including derivations and lookups.
    pub fn overall_hit_rate(&self) -> f64 {
        let total = self.total_derivations + self.total_lookups;
        if total == 0 {
            return 0.0;
        }
        (self.hits + self.lookup_hits) as f64 / total as f64
    }

    /// Returns true if any counter is close to u64::MAX.
    ///
    /// Cache metrics are designed to saturate at u64::MAX instead of wrapping.
    /// This helper enables simple monitoring/alerting for long-running processes.
    ///
    /// Monitoring recommendation: alert on "approaching saturation" warnings
    /// and plan operational counter resets via process restart, since counters
    /// intentionally do not wrap.
    pub fn is_near_saturation(&self) -> bool {
        Self::is_near_sat(self.hits)
            || Self::is_near_sat(self.misses)
            || Self::is_near_sat(self.evictions)
            || Self::is_near_sat(self.total_derivations)
            || Self::is_near_sat(self.ttl_expirations)
            || Self::is_near_sat(self.total_lookups)
            || Self::is_near_sat(self.lookup_hits)
            || Self::is_near_sat(self.lookup_misses)
            || Self::is_near_sat(self.current_cache_size)
            || Self::is_near_sat(self.peak_cache_size)
            || Self::is_near_sat(self.import_rejects)
            || Self::is_near_sat(self.import_entries)
            || Self::is_near_sat(self.import_bytes)
            || Self::is_near_sat(self.purge_runs)
            || Self::is_near_sat(self.purge_entries)
            || Self::is_near_sat(self.purge_bytes)
            || Self::is_near_sat(self.total_derive_time_ms)
            || Self::is_near_sat(self.derive_count)
    }

    /// Calculate average derivation time in milliseconds.
    pub fn avg_derive_time_ms(&self) -> f64 {
        if self.derive_count == 0 {
            return 0.0;
        }
        self.total_derive_time_ms as f64 / self.derive_count as f64
    }
}
