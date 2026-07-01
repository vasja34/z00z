/// Receiver manager implementation with cache, TTL, and optional rate limiting.
#[derive(Debug)]
pub struct ReceiverManagerImpl<K: KeyManager, T: TimeProvider = z00z_utils::time::SystemTimeProvider>
{
    key_manager: K,
    cache: RwLock<LruCache<Bip44Path, CachedEntry>>,
    eviction_listener: Arc<dyn CacheEvictionListener>,
    max_cache_size: usize,
    ttl: Duration,
    purge_interval: Duration,
    purge_min_size: usize,
    last_purge_ts: AtomicU64,
    time_provider: T,
    timing_safe_mode: bool,
    rate_limiter: Option<TokenBucket>,
    metrics: Arc<CacheMetrics>,
}

impl<K: KeyManager> ReceiverManagerImpl<K, z00z_utils::time::SystemTimeProvider> {
    /// Create a new receiver-manager builder with the default time provider.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(key_manager: K) -> ReceiverManagerBuilder<K, z00z_utils::time::SystemTimeProvider> {
        ReceiverManagerBuilder::new(key_manager)
    }
}

/// Builder for configuring a `ReceiverManagerImpl` instance.
pub struct ReceiverManagerBuilder<K: KeyManager, T: TimeProvider> {
    key_manager: K,
    eviction_listener: Arc<dyn CacheEvictionListener>,
    max_cache_size: usize,
    ttl: Duration,
    purge_interval: Duration,
    purge_min_size: usize,
    time_provider: T,
    timing_safe_mode: bool,
    rate_limit_per_sec: Option<u32>,
    rate_limit_burst: Option<u32>,
}

impl<K: KeyManager> ReceiverManagerBuilder<K, z00z_utils::time::SystemTimeProvider> {
    fn new(key_manager: K) -> Self {
        Self {
            key_manager,
            eviction_listener: Arc::new(NoopEvictionListener),
            max_cache_size: DEFAULT_CACHE_SIZE,
            ttl: Duration::from_secs(DEFAULT_TTL_SECONDS),
            purge_interval: Duration::from_secs(DEFAULT_PURGE_INTERVAL_SECONDS),
            purge_min_size: DEFAULT_PURGE_MIN_SIZE,
            time_provider: z00z_utils::time::SystemTimeProvider,
            timing_safe_mode: false,
            rate_limit_per_sec: None,
            rate_limit_burst: None,
        }
    }

    /// Set a listener that receives LRU eviction notifications.
    pub fn with_eviction_listener(mut self, listener: Arc<dyn CacheEvictionListener>) -> Self {
        self.eviction_listener = listener;
        self
    }

    /// Override the maximum cache size.
    pub fn with_limit(mut self, max_size: usize) -> Self {
        self.max_cache_size = max_size;
        self
    }

    /// Override the cache TTL.
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// Enable or disable timing-safe cache behavior.
    pub fn with_timing_safe_mode(mut self, timing_safe_mode: bool) -> Self {
        self.timing_safe_mode = timing_safe_mode;
        self
    }

    /// Enable token-bucket rate limiting for derivation requests.
    pub fn with_rate_limit(mut self, rate_per_sec: u32, burst: u32) -> Self {
        self.rate_limit_per_sec = Some(rate_per_sec);
        self.rate_limit_burst = Some(burst);
        self
    }
}

impl<K: KeyManager, T: TimeProvider> ReceiverManagerBuilder<K, T> {
    /// Replace the time provider, primarily for deterministic tests.
    pub fn with_time_provider<U: TimeProvider>(
        self,
        time_provider: U,
    ) -> ReceiverManagerBuilder<K, U> {
        ReceiverManagerBuilder {
            key_manager: self.key_manager,
            eviction_listener: self.eviction_listener,
            max_cache_size: self.max_cache_size,
            ttl: self.ttl,
            purge_interval: self.purge_interval,
            purge_min_size: self.purge_min_size,
            time_provider,
            timing_safe_mode: self.timing_safe_mode,
            rate_limit_per_sec: self.rate_limit_per_sec,
            rate_limit_burst: self.rate_limit_burst,
        }
    }

    /// Override the periodic purge interval.
    pub fn with_purge_interval(mut self, purge_interval: Duration) -> Self {
        self.purge_interval = purge_interval;
        self
    }

    /// Override the minimum cache size required before purge scans run.
    pub fn with_purge_min_size(mut self, purge_min_size: usize) -> Self {
        self.purge_min_size = purge_min_size;
        self
    }

    /// Build the configured receiver manager.
    pub fn build(self) -> ReceiverManagerResult<ReceiverManagerImpl<K, T>> {
        if self.max_cache_size == 0 || self.max_cache_size > MAX_CACHE_SIZE {
            return Err(ReceiverManagerError::InvalidCacheSize(self.max_cache_size));
        }

        if self.purge_interval.is_zero()
            || self.purge_interval < Duration::from_secs(MIN_PURGE_INTERVAL_SECONDS)
        {
            return Err(ReceiverManagerError::InvalidPurgeInterval);
        }

        let cache_capacity = NonZeroUsize::new(self.max_cache_size)
            .ok_or(ReceiverManagerError::InvalidCacheSize(self.max_cache_size))?;

        let purge_interval = std::cmp::min(self.purge_interval, self.ttl);
        let last_purge_ts = self.time_provider.compat_unix_timestamp();

        let rate_limiter = match (self.rate_limit_per_sec, self.rate_limit_burst) {
            (Some(rate), Some(burst)) => {
                if rate == 0 || burst == 0 {
                    return Err(ReceiverManagerError::InvalidRateLimit {
                        rate_per_sec: rate,
                        burst,
                    });
                }
                Some(TokenBucket::new(rate, burst))
            }
            _ => None,
        };

        Ok(ReceiverManagerImpl {
            key_manager: self.key_manager,
            cache: RwLock::new(LruCache::new(cache_capacity)),
            eviction_listener: self.eviction_listener,
            max_cache_size: self.max_cache_size,
            ttl: self.ttl,
            purge_interval,
            purge_min_size: self.purge_min_size,
            last_purge_ts: AtomicU64::new(last_purge_ts),
            time_provider: self.time_provider,
            timing_safe_mode: self.timing_safe_mode,
            rate_limiter,
            metrics: Arc::new(CacheMetrics::default()),
        })
    }
}