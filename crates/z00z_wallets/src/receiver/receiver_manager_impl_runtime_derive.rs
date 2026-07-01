impl<K: KeyManager, T: TimeProvider> ReceiverManagerImpl<K, T> {
    fn batch_limit_count(&mut self, paths: &[Bip44Path]) -> ReceiverManagerResult<usize> {
        if self.timing_safe_mode {
            for path in paths {
                self.validate_path(path)?;

                if path.asset_type().index() != Z00Z_COIN_TYPE {
                    return Err(ReceiverManagerError::InvalidCoinType(
                        path.asset_type().index(),
                    ));
                }
            }

            return Ok(paths.len());
        }

        let now = self.time_provider.compat_unix_timestamp();
        let mut cache = self.cache_write()?;
        let mut miss_paths = std::collections::HashSet::new();

        for path in paths {
            self.validate_path(path)?;

            if path.asset_type().index() != Z00Z_COIN_TYPE {
                return Err(ReceiverManagerError::InvalidCoinType(
                    path.asset_type().index(),
                ));
            }

            let mut is_cached = false;
            if let Some(entry) = cache.peek(path).cloned() {
                if self.is_expired_with_timestamp(&entry, now) {
                    cache.pop(path);
                    self.metrics.inc_ttl_expirations();
                    self.metrics.inc_evictions();
                } else {
                    is_cached = true;
                }
            }

            if !is_cached {
                miss_paths.insert(*path);
            }
        }

        self.metrics
            .current_cache_size
            .store(cache.len() as u64, Ordering::Relaxed);

        Ok(miss_paths.len())
    }

    fn derive_keys_core(
        &mut self,
        path: Bip44Path,
        use_limit: bool,
    ) -> ReceiverManagerResult<DerivedWalletKeys> {
        self.validate_path(&path)?;

        if path.asset_type().index() != Z00Z_COIN_TYPE {
            return Err(ReceiverManagerError::InvalidCoinType(
                path.asset_type().index(),
            ));
        }

        self.metrics.inc_total_derivations();

        let derive_start = Instant::now();
        let now = self.time_provider.compat_unix_timestamp();

        if !self.timing_safe_mode {
            let cached_entry = {
                let mut cache = self.cache_write()?;
                if let Some(cached) = cache.get(&path).cloned() {
                    if self.is_expired_with_timestamp(&cached, now) {
                        cache.pop(&path);
                        self.metrics.inc_ttl_expirations();
                        self.metrics.inc_evictions();
                        self.metrics
                            .current_cache_size
                            .store(cache.len() as u64, Ordering::Relaxed);
                        None
                    } else {
                        Some(cached)
                    }
                } else {
                    None
                }
            };

            if let Some(cached) = cached_entry {
                self.metrics.inc_hits();
                return Ok(cached.keys);
            }
        }

        if use_limit {
            if let Some(bucket) = self.rate_limiter.as_ref() {
                let now_ms = self
                    .time_provider
                    .try_unix_timestamp_ms()
                    .map_err(|error| ReceiverManagerError::ClockUnavailable(error.to_string()))?;
                let allowed = bucket.try_consume(now_ms, 1)?;
                if !allowed {
                    z00z_utils::logger::Logger::warn(
                        &z00z_utils::logger::TracingLogger,
                        "Receiver derivation rate limit exceeded - potential DoS attack",
                    );
                    return Err(ReceiverManagerError::RateLimitExceeded);
                }
            }
        }

        let spend_key_raw = self
            .key_manager
            .derive_key(&path)
            .map_err(|error| ReceiverManagerError::KeyDerivation(error.to_string()))?;
        let spend_key = Z00ZRistrettoPoint::from_ristretto_public_key(spend_key_raw);
        if spend_key.as_bytes() == [0u8; 32] {
            return Err(ReceiverManagerError::IdentityKeyRejected);
        }

        let view_path = path
            .to_view_key_path()
            .map_err(|error| ReceiverManagerError::InvalidPath(error.to_string()))?;
        let view_key_raw = self
            .key_manager
            .derive_key(&view_path)
            .map_err(|error| ReceiverManagerError::KeyDerivation(error.to_string()))?;
        let view_key = Z00ZRistrettoPoint::from_ristretto_public_key(view_key_raw);

        self.metrics
            .add_derive_time(derive_start.elapsed().as_millis() as u64);
        if view_key.as_bytes() == [0u8; 32] {
            return Err(ReceiverManagerError::IdentityKeyRejected);
        }

        if spend_key.ct_eq(&view_key) {
            return Err(ReceiverManagerError::ReceiverKeysNotIndependent);
        }

        let keys = DerivedWalletKeys {
            spend_key,
            view_key,
        };

        let mut insert_cache = true;
        {
            let mut cache = self.cache_write()?;
            if let Some(cached) = cache.get(&path).cloned() {
                if self.is_expired_with_timestamp(&cached, now) {
                    cache.pop(&path);
                    self.metrics.inc_ttl_expirations();
                    self.metrics.inc_evictions();
                    self.metrics
                        .current_cache_size
                        .store(cache.len() as u64, Ordering::Relaxed);
                } else {
                    let spend_match = cached.keys.spend_key.ct_eq(&keys.spend_key);
                    let view_match = cached.keys.view_key.ct_eq(&keys.view_key);

                    if spend_match && view_match {
                        self.metrics.inc_hits();
                        insert_cache = false;
                    } else {
                        cache.pop(&path);
                        self.metrics
                            .current_cache_size
                            .store(cache.len() as u64, Ordering::Relaxed);
                    }
                }
            }
        }

        if insert_cache {
            self.metrics.inc_misses();
        }

        if insert_cache {
            let entry = CachedEntry {
                keys: keys.clone(),
                inserted_at: now,
            };

            self.maybe_purge(now)?;
            let (current_size, evicted) = {
                let mut cache = self.cache_write()?;
                let evicted = cache
                    .push(path, entry)
                    .map(|(evicted_path, evicted_entry)| {
                        (evicted_path, evicted_entry.keys.spend_key.clone())
                    });
                if evicted.is_some() {
                    self.metrics.inc_evictions();
                }
                (cache.len() as u64, evicted)
            };

            if let Some((evicted_path, key)) = evicted {
                self.notify_evict(evicted_path, key);
            }
            self.metrics
                .current_cache_size
                .store(current_size, Ordering::Relaxed);

            let mut peak = self.metrics.peak_cache_size.load(Ordering::Relaxed);
            while current_size > peak {
                match self.metrics.peak_cache_size.compare_exchange_weak(
                    peak,
                    current_size,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(actual) => peak = actual,
                }
            }
        }

        Ok(keys)
    }

    /// Clear both the receiver cache and the underlying key-manager cache.
    pub fn clear_all(&mut self) -> ReceiverManagerResult<()> {
        self.cache_write()?.clear();
        self.key_manager.clear();
        self.metrics.reset();
        Ok(())
    }

    /// Return a snapshot of the current cache metrics.
    pub fn get_metrics(&self) -> CacheMetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Reset all cache metrics counters.
    pub fn reset_metrics(&mut self) {
        self.metrics.reset();
    }
}
