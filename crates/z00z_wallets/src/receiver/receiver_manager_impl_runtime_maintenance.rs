impl<K: KeyManager, T: TimeProvider> ReceiverManagerImpl<K, T> {
    #[inline]
    fn notify_evict(&self, path: Bip44Path, key: Z00ZRistrettoPoint) {
        let listener = self.eviction_listener.clone();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            listener.on_evict(path, &key);
        }));

        if result.is_err() {
            z00z_utils::logger::Logger::warn(
                &z00z_utils::logger::TracingLogger,
                "Cache eviction listener panicked; eviction continued",
            );
        }
    }

    fn cache_read(
        &self,
    ) -> ReceiverManagerResult<std::sync::RwLockReadGuard<'_, LruCache<Bip44Path, CachedEntry>>>
    {
        self.cache
            .read()
            .map_err(|_| ReceiverManagerError::LockPoisoned)
    }

    fn cache_write(
        &self,
    ) -> ReceiverManagerResult<std::sync::RwLockWriteGuard<'_, LruCache<Bip44Path, CachedEntry>>>
    {
        self.cache
            .write()
            .map_err(|_| ReceiverManagerError::LockPoisoned)
    }

    fn validate_path(&self, path: &Bip44Path) -> ReceiverManagerResult<()> {
        match Bip44Validator::validate(path) {
            Ok(()) => Ok(()),
            Err(Bip44Error::NonStandardPath {
                reason: Bip44ViolationReason::AssetTypeValueMismatch,
                ..
            }) => Err(ReceiverManagerError::InvalidCoinType(
                path.asset_type().index(),
            )),
            Err(error) => Err(ReceiverManagerError::InvalidPath(error.to_string())),
        }
    }

    /// Return the current token-bucket status when rate limiting is enabled.
    pub fn rate_limiter_status(&self) -> ReceiverManagerResult<Option<RateLimiterStatus>> {
        let Some(bucket) = self.rate_limiter.as_ref() else {
            return Ok(None);
        };

        let now_ms = self
            .time_provider
            .try_unix_timestamp_ms()
            .map_err(|error| ReceiverManagerError::ClockUnavailable(error.to_string()))?;
        Ok(Some(bucket.status(now_ms)?))
    }

    /// Construct a new receiver manager with explicit cache and time configuration.
    pub fn new_with_config(
        key_manager: K,
        max_cache_size: usize,
        ttl: Duration,
        time_provider: T,
    ) -> ReceiverManagerResult<Self> {
        if max_cache_size == 0 || max_cache_size > MAX_CACHE_SIZE {
            return Err(ReceiverManagerError::InvalidCacheSize(max_cache_size));
        }

        let cache_capacity = NonZeroUsize::new(max_cache_size)
            .ok_or(ReceiverManagerError::InvalidCacheSize(max_cache_size))?;

        let purge_interval =
            std::cmp::min(Duration::from_secs(DEFAULT_PURGE_INTERVAL_SECONDS), ttl);
        let last_purge_ts = time_provider.compat_unix_timestamp();

        Ok(Self {
            key_manager,
            cache: RwLock::new(LruCache::new(cache_capacity)),
            eviction_listener: Arc::new(NoopEvictionListener),
            max_cache_size,
            ttl,
            purge_interval,
            purge_min_size: DEFAULT_PURGE_MIN_SIZE,
            last_purge_ts: AtomicU64::new(last_purge_ts),
            time_provider,
            timing_safe_mode: false,
            rate_limiter: None,
            metrics: Arc::new(CacheMetrics::default()),
        })
    }

    /// Expose the internal cache for diagnostics and tests.
    pub fn cache(
        &self,
    ) -> ReceiverManagerResult<std::sync::RwLockReadGuard<'_, LruCache<Bip44Path, CachedEntry>>>
    {
        self.cache_read()
    }

    /// Return the current number of cached entries.
    pub fn cache_size(&self) -> ReceiverManagerResult<usize> {
        Ok(self.cache_read()?.len())
    }

    /// Return the configured cache capacity.
    pub fn max_cache_size(&self) -> usize {
        self.max_cache_size
    }

    /// Return the configured cache TTL.
    pub fn ttl(&self) -> Duration {
        self.ttl
    }

    /// Update the periodic purge interval.
    pub fn set_purge_interval(&mut self, purge_interval: Duration) {
        assert!(
            !purge_interval.is_zero()
                && purge_interval >= Duration::from_secs(MIN_PURGE_INTERVAL_SECONDS),
            "purge_interval must be >= {} seconds, got: {:?}",
            MIN_PURGE_INTERVAL_SECONDS,
            purge_interval
        );
        self.purge_interval = std::cmp::min(purge_interval, self.ttl);
    }

    /// Update the minimum cache size required before purge scans are considered.
    pub fn set_purge_min_size(&mut self, purge_min_size: usize) {
        self.purge_min_size = purge_min_size;
    }

    fn is_expired(&self, entry: &CachedEntry) -> bool {
        let now = self.time_provider.compat_unix_timestamp();
        let age_seconds = now.saturating_sub(entry.inserted_at);
        let age = Duration::from_secs(age_seconds);
        age > self.ttl
    }

    fn is_expired_with_timestamp(&self, entry: &CachedEntry, now: u64) -> bool {
        let age_seconds = now.saturating_sub(entry.inserted_at);
        let age = Duration::from_secs(age_seconds);
        age > self.ttl
    }

    fn should_purge(&self, now: u64, cache_len: usize) -> bool {
        if cache_len < self.purge_min_size {
            return false;
        }

        let last_purge = self.last_purge_ts.load(Ordering::Relaxed);
        let elapsed = now.saturating_sub(last_purge);
        let interval = self.purge_interval.as_secs();

        elapsed >= interval
    }

    fn maybe_purge(&self, now: u64) -> ReceiverManagerResult<()> {
        let cache_len = self.cache_read()?.len();
        if !self.should_purge(now, cache_len) {
            return Ok(());
        }

        self.evict_expired_entries(now)
    }

    fn evict_expired_entries(&self, now: u64) -> ReceiverManagerResult<()> {
        let ttl = self.ttl;
        let mut expired_keys = Vec::new();

        self.metrics.inc_purge_runs();

        let mut cache = self.cache_write()?;

        for (path, entry) in cache.iter() {
            let age_seconds = now.saturating_sub(entry.inserted_at);
            let age = Duration::from_secs(age_seconds);
            if age > ttl {
                expired_keys.push(*path);
            }
        }

        let purged = expired_keys.len() as u64;

        for path in expired_keys {
            cache.pop(&path);
            self.metrics.inc_ttl_expirations();
            self.metrics.inc_evictions();
        }

        self.metrics.add_purge_entries(purged);
        self.metrics
            .add_purge_bytes(purged.saturating_mul(CACHE_ENTRY_EST_BYTES));

        self.last_purge_ts.store(now, Ordering::Relaxed);

        let current_size = cache.len() as u64;
        self.metrics
            .current_cache_size
            .store(current_size, Ordering::Relaxed);

        Ok(())
    }

    /// Evict expired cache entries immediately.
    pub fn evict_expired(&self) -> ReceiverManagerResult<()> {
        let now = self.time_provider.compat_unix_timestamp();
        self.evict_expired_entries(now)
    }

    /// Run the standard purge path immediately.
    pub fn purge_expired(&self) -> ReceiverManagerResult<()> {
        self.evict_expired()
    }

    /// Check whether a specific derivation path is currently cached.
    pub fn contains_path(&self, path: &Bip44Path) -> ReceiverManagerResult<bool> {
        Ok(self.cache_read()?.contains(path))
    }

    /// Return the configured eviction listener used for cache invalidation hooks.
    pub fn eviction_listener(&self) -> Arc<dyn CacheEvictionListener> {
        Arc::clone(&self.eviction_listener)
    }
}
