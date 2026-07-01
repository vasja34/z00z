/// Async facade around `ReceiverManagerImpl` for Tokio-based callers.
pub struct AsyncReceiverManagerImpl<K: KeyManager + Send + Sync, T: TimeProvider + Send + Sync> {
    inner: Arc<tokio::sync::RwLock<ReceiverManagerImpl<K, T>>>,
    batch_threshold: Arc<AtomicUsize>,
    fixed_batch_threshold: Option<usize>,
    #[cfg(test)]
    trace: Arc<AsyncBatchTrace>,
}

#[cfg(test)]
#[derive(Default)]
struct AsyncBatchTrace {
    inline_calls: AtomicU64,
    spawn_blocking_calls: AtomicU64,
}

impl<K: KeyManager + Send + Sync, T: TimeProvider + Send + Sync> AsyncReceiverManagerImpl<K, T> {
    /// Wrap a synchronous receiver manager for async access.
    pub fn new(inner: ReceiverManagerImpl<K, T>) -> Self {
        Self {
            inner: Arc::new(tokio::sync::RwLock::new(inner)),
            batch_threshold: Arc::new(AtomicUsize::new(ASYNC_BATCH_THRESHOLD)),
            fixed_batch_threshold: None,
            #[cfg(test)]
            trace: Arc::new(AsyncBatchTrace::default()),
        }
    }

    /// Wrap a synchronous receiver manager with explicit async batching configuration.
    pub fn new_with_config(
        inner: ReceiverManagerImpl<K, T>,
        config: ReceiverManagerConfig,
    ) -> ReceiverManagerResult<Self> {
        config.validate()?;
        let fixed_batch_threshold = config.async_batch_threshold;
        let batch_threshold = fixed_batch_threshold.unwrap_or(ASYNC_BATCH_THRESHOLD);
        Ok(Self {
            inner: Arc::new(tokio::sync::RwLock::new(inner)),
            batch_threshold: Arc::new(AtomicUsize::new(batch_threshold)),
            fixed_batch_threshold,
            #[cfg(test)]
            trace: Arc::new(AsyncBatchTrace::default()),
        })
    }

    /// Recompute the derive-batch threshold from observed runtime latency.
    pub async fn auto_tune_threshold(&self) {
        if let Some(fixed) = self.fixed_batch_threshold {
            self.batch_threshold.store(fixed, Ordering::Relaxed);
            return;
        }

        let guard = self.inner.read().await;
        let sample_count = guard.metrics.derive_count.load(Ordering::Relaxed);
        if sample_count == 0 {
            return;
        }
        let avg_time_ms = guard.metrics.avg_derive_time_ms();

        const FAST_THRESHOLD: usize = 20;
        const SLOW_THRESHOLD: usize = 10;
        const FAST_DERIVE_MS: f64 = 5.0;

        let new_threshold = if avg_time_ms < FAST_DERIVE_MS {
            FAST_THRESHOLD
        } else {
            SLOW_THRESHOLD
        };

        let current = self.batch_threshold.load(Ordering::Relaxed);
        if new_threshold != current {
            z00z_utils::logger::Logger::debug(
                &z00z_utils::logger::TracingLogger,
                &format!(
                    "Auto-tuned batch threshold: {} -> {} (avg_derive_time={:.2}ms)",
                    current,
                    new_threshold,
                    avg_time_ms
                ),
            );
            self.batch_threshold.store(new_threshold, Ordering::Relaxed);
        }
    }

    /// Return the current async batching configuration snapshot.
    pub fn config(&self) -> ReceiverManagerConfig {
        ReceiverManagerConfig {
            async_batch_threshold: Some(self.batch_threshold.load(Ordering::Relaxed)),
        }
    }

    #[cfg(test)]
    /// Expose the wrapped manager to async tests.
    pub async fn inner(&self) -> tokio::sync::RwLockReadGuard<'_, ReceiverManagerImpl<K, T>> {
        self.inner.read().await
    }
}

impl<K: KeyManager + Send + Sync + 'static>
    AsyncReceiverManagerImpl<K, z00z_utils::time::SystemTimeProvider>
{
    /// Build an async receiver manager from a key manager with default configuration.
    pub fn from_key_manager(key_manager: K) -> ReceiverManagerResult<Self> {
        Self::from_key_manager_with_config(key_manager, ReceiverManagerConfig::default())
    }

    /// Build an async receiver manager from a key manager with explicit configuration.
    pub fn from_key_manager_with_config(
        key_manager: K,
        config: ReceiverManagerConfig,
    ) -> ReceiverManagerResult<Self> {
        let inner = ReceiverManagerImpl::new(key_manager).build()?;
        Self::new_with_config(inner, config)
    }
}

impl<K: KeyManager + Send + Sync, T: TimeProvider + Send + Sync> Clone
    for AsyncReceiverManagerImpl<K, T>
{
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            batch_threshold: Arc::clone(&self.batch_threshold),
            fixed_batch_threshold: self.fixed_batch_threshold,
            #[cfg(test)]
            trace: Arc::clone(&self.trace),
        }
    }
}

#[async_trait]
impl<K: KeyManager + Send + Sync + 'static, T: TimeProvider + Send + Sync + 'static>
    AsyncReceiverManager for AsyncReceiverManagerImpl<K, T>
{
    async fn derive_spend_key(&self, path: Bip44Path)
        -> ReceiverManagerResult<Z00ZRistrettoPoint> {
        let mut guard = self.inner.write().await;
        guard.derive_spend_key(path)
    }

    async fn derive_wallet_keys(&self, path: Bip44Path) -> ReceiverManagerResult<DerivedWalletKeys> {
        let mut guard = self.inner.write().await;
        guard.derive_wallet_keys(path)
    }

    async fn get_receiver_key(&self, path: Bip44Path)
        -> ReceiverManagerResult<Z00ZRistrettoPoint> {
        let mut guard = self.inner.write().await;
        guard.get_receiver_key(path)
    }

    async fn list_receivers(&self)
        -> ReceiverManagerResult<Vec<(Bip44Path, Z00ZRistrettoPoint)>> {
        let guard = self.inner.read().await;
        guard.list_receivers()
    }

    async fn clear_cache(&self) -> ReceiverManagerResult<()> {
        let mut guard = self.inner.write().await;
        guard.clear_cache()
    }

    async fn derive_batch(
        &self,
        paths: &[Bip44Path],
    ) -> ReceiverManagerResult<Vec<Z00ZRistrettoPoint>> {
        self.auto_tune_threshold().await;
        let threshold = self.batch_threshold.load(Ordering::Relaxed);

        if use_inline_batch(paths.len(), threshold) {
            #[cfg(test)]
            self.trace.inline_calls.fetch_add(1, Ordering::Relaxed);
            let mut guard = self.inner.write().await;
            return guard.derive_batch(paths);
        }

        #[cfg(test)]
        self.trace
            .spawn_blocking_calls
            .fetch_add(1, Ordering::Relaxed);
        let paths: Vec<Bip44Path> = paths.to_vec();
        let inner_clone = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            let mut guard = inner_clone.blocking_write();
            guard.derive_batch(&paths)
        })
        .await
        .map_err(|error| ReceiverManagerError::KeyDerivation(format!("Task join error: {}", error)))?
    }

    async fn metrics(&self) -> CacheMetricsSnapshot {
        let guard = self.inner.read().await;
        guard.metrics()
    }

    async fn reset_metrics(&self) {
        let mut guard = self.inner.write().await;
        guard.reset_metrics();
    }
}

impl<K: KeyManager + Send + Sync + 'static, T: TimeProvider + Send + Sync + 'static>
    AsyncReceiverManagerImpl<K, T>
{
    /// Export the current cache state with wallet-bound authentication.
    pub async fn export_cache(&self, wallet_id: &[u8]) -> ReceiverManagerResult<ReceiverCacheState> {
        let guard = self.inner.read().await;
        guard.export_cache(wallet_id)
    }

    /// Import a previously exported authenticated receiver-cache state.
    pub async fn import_cache(
        &self,
        wallet_id: &[u8],
        state_record: ReceiverCacheState,
    ) -> ReceiverManagerResult<()> {
        let mut guard = self.inner.write().await;
        guard.import_cache(wallet_id, state_record)
    }

    /// Pre-derive a bounded range of cache entries for future reads.
    pub async fn warm_cache(&self, max_index: u32) -> ReceiverManagerResult<()> {
        let mut guard = self.inner.write().await;
        guard.warm_cache(max_index)
    }
}
