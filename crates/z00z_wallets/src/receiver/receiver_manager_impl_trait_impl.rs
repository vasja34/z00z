impl<K: KeyManager, T: TimeProvider> ReceiverManager for ReceiverManagerImpl<K, T> {
    fn derive_spend_key(&mut self, path: Bip44Path) -> ReceiverManagerResult<Z00ZRistrettoPoint> {
        let keys = self.derive_wallet_keys(path)?;
        Ok(keys.spend_key)
    }

    fn derive_wallet_keys(&mut self, path: Bip44Path) -> ReceiverManagerResult<DerivedWalletKeys> {
        self.derive_keys_core(path, true)
    }

    fn get_receiver_key(&mut self, path: Bip44Path) -> ReceiverManagerResult<Z00ZRistrettoPoint> {
        self.validate_path(&path)?;

        if path.asset_type().index() != Z00Z_COIN_TYPE {
            return Err(ReceiverManagerError::InvalidCoinType(
                path.asset_type().index(),
            ));
        }

        self.metrics.inc_total_lookups();

        let now = self.time_provider.compat_unix_timestamp();
        let mut cache = self.cache_write()?;

        if let Some((spend_key, is_expired)) = cache.peek(&path).map(|entry| {
            (
                entry.keys.spend_key.clone(),
                self.is_expired_with_timestamp(entry, now),
            )
        }) {
            if is_expired {
                cache.pop(&path);
                self.metrics.inc_ttl_expirations();
                self.metrics.inc_evictions();
                self.metrics
                    .current_cache_size
                    .store(cache.len() as u64, Ordering::Relaxed);

                self.metrics.inc_lookup_misses();
                return Err(ReceiverManagerError::NotFound(path));
            }

            self.metrics.inc_lookup_hits();
            return Ok(spend_key);
        }

        self.metrics.inc_lookup_misses();
        Err(ReceiverManagerError::NotFound(path))
    }

    fn list_receivers(&self) -> ReceiverManagerResult<Vec<(Bip44Path, Z00ZRistrettoPoint)>> {
        let cache = self.cache_read()?;
        let valid_entries: Vec<_> = cache
            .iter()
            .filter(|(_, entry)| !self.is_expired(entry))
            .map(|(path, entry)| (*path, entry.keys.spend_key.clone()))
            .collect();

        Ok(valid_entries)
    }

    fn clear_cache(&mut self) -> ReceiverManagerResult<()> {
        self.cache_write()?.clear();
        self.metrics.current_cache_size.store(0, Ordering::Relaxed);
        Ok(())
    }

    fn derive_batch(
        &mut self,
        paths: &[Bip44Path],
    ) -> ReceiverManagerResult<Vec<Z00ZRistrettoPoint>> {
        if let Some(bucket) = self.rate_limiter.as_ref() {
            let requested: u32 = paths.len().try_into().unwrap_or(u32::MAX);
            let max_allowed = bucket.burst();
            if requested > max_allowed {
                return Err(ReceiverManagerError::BatchTooLarge {
                    requested,
                    max_allowed,
                });
            }
        }

        let limit_count = self.batch_limit_count(paths)?;

        if let Some(bucket) = self.rate_limiter.as_ref() {
            let requested: u32 = limit_count.try_into().unwrap_or(u32::MAX);

            if requested > 0 {
                let now_ms = self
                    .time_provider
                    .try_unix_timestamp_ms()
                    .map_err(|error| ReceiverManagerError::ClockUnavailable(error.to_string()))?;
                let allowed = bucket.try_consume(now_ms, requested)?;
                if !allowed {
                    z00z_utils::logger::Logger::warn(
                        &z00z_utils::logger::TracingLogger,
                        "Receiver derivation batch rate limit exceeded - potential DoS attack",
                    );
                    return Err(ReceiverManagerError::RateLimitExceeded);
                }
            }
        }

        let mut results = Vec::with_capacity(paths.len());

        for path in paths {
            let keys = self.derive_keys_core(*path, false)?;
            results.push(keys.spend_key);
        }

        Ok(results)
    }

    fn metrics(&self) -> CacheMetricsSnapshot {
        self.metrics.snapshot()
    }

    fn reset_metrics(&mut self) {
        self.metrics.reset();
    }
}
