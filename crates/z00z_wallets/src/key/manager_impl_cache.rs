impl<R: SecureRngProvider> KeyManagerImpl<R> {
    fn deriving_paths_lock(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, std::collections::HashMap<Bip44Path, Arc<DerivationFlight>>>>
    {
        self.deriving_paths
            .lock()
            .map_err(|_| KeyManagerError::LockPoisoned {
                lock: "deriving_paths",
            })
    }

    fn finish_derivation_flight(&self, path: &Bip44Path, flight: &Arc<DerivationFlight>) {
        match flight.is_deriving.lock() {
            Ok(mut is_deriving) => {
                *is_deriving = false;
            }
            Err(poisoned) => {
                let mut is_deriving = poisoned.into_inner();
                *is_deriving = false;
            }
        }
        flight.wait.notify_all();

        if let Ok(mut deriving_paths) = self.deriving_paths.lock() {
            let should_remove = deriving_paths
                .get(path)
                .is_some_and(|active_flight| Arc::ptr_eq(active_flight, flight));
            if should_remove {
                deriving_paths.remove(path);
            }
        }
    }

    fn store_derivation_result(
        &self,
        flight: &Arc<DerivationFlight>,
        result: &Result<RistrettoPublicKey>,
    ) -> Result<()> {
        let mut stored_result = flight.result.lock().map_err(|_| KeyManagerError::LockPoisoned {
            lock: "derivation_result",
        })?;
        *stored_result = Some(DerivationFlightResult {
            completed_at: self.time_provider.compat_unix_timestamp(),
            result: result.clone(),
        });
        Ok(())
    }

    fn load_derivation_result(
        &self,
        flight: &Arc<DerivationFlight>,
    ) -> Result<Option<Result<RistrettoPublicKey>>> {
        let stored_result = flight.result.lock().map_err(|_| KeyManagerError::LockPoisoned {
            lock: "derivation_result",
        })?;
        let now = self.time_provider.compat_unix_timestamp();
        Ok(stored_result.clone().and_then(|stored_result| match stored_result.result {
            Ok(public_key)
                if now.saturating_sub(stored_result.completed_at) < DERIVED_KEY_TTL_SECONDS =>
            {
                Some(Ok(public_key))
            }
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        }))
    }

    fn cache_read(&self) -> Result<RwLockReadGuard<'_, LruCache<Bip44Path, CachedKey>>> {
        self.derived_public_keys
            .read()
            .map_err(|_| KeyManagerError::LockPoisoned {
                lock: "derived_public_keys",
            })
    }

    fn cache_write(&self) -> Result<RwLockWriteGuard<'_, LruCache<Bip44Path, CachedKey>>> {
        self.derived_public_keys
            .write()
            .map_err(|_| KeyManagerError::LockPoisoned {
                lock: "derived_public_keys",
            })
    }

    fn validate_path(&self, path: &Bip44Path) -> Result<()> {
        use crate::key::Bip44Validator;

        Bip44Validator::validate(path).map_err(|_| KeyManagerError::InvalidParameters)
    }
    /// Insert a public key into the derivation cache for deterministic test setups.
    pub fn insert_cache(&self, path: Bip44Path, public_key: RistrettoPublicKey) -> Result<()> {
        let mut cache = self.cache_write()?;
        let now = self.time_provider.compat_unix_timestamp();
        let cached_key = CachedKey {
            public_key,
            cached_at: now,
        };
        cache.put(path, cached_key);
        Ok(())
    }
    /// Return the number of cache misses that triggered a fresh derivation in tests.
    pub fn derivation_count(&self) -> usize {
        self.derivation_count.load(Ordering::Relaxed)
    }
    /// Force a full cache validation pass in test builds.
    pub fn validate_cache(&self) -> Result<()> {
        self.validate_cached_keys()
    }
    /// Poison the cache lock to exercise lock-poisoning recovery paths in tests.
    pub fn poison_cache(&self) {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = self.derived_public_keys.write().unwrap();
            panic!("poison");
        }));
    }
    /// Set only the encrypted seed container for state-corruption tests.
    pub fn set_encrypted_seed_only(&mut self, encrypted_seed: CipherSeedContainer) {
        self.encrypted_seed = Some(encrypted_seed);
        self.bip44_manager = None;
    }
    /// Override external gap counters for targeted test scenarios.
    pub fn set_gap_ext(&self, next_index: u32, last_used_plus1: u32) {
        self.gap_external.store(next_index, Ordering::Release);
        self.last_used_ext.store(last_used_plus1, Ordering::Release);
    }

    /// Return the current number of cached public keys.
    pub fn cache_size(&self) -> usize {
        self.cache_read().map(|cache| cache.len()).unwrap_or(0)
    }

    fn report_cache_corruption(&self) -> KeyManagerError {
        self.logger.error("Cache corruption detected");
        self.metrics.inc_counter("wallet.key.cache.corrupt", 1);
        KeyManagerError::CacheCorrupted
    }

    fn expected_public_key_for_path(
        &self,
        manager: &Bip44KeyManager,
        path: &Bip44Path,
    ) -> Result<RistrettoPublicKey> {
        let address_key = manager.derive_address_key_for_path(path).map_err(|e| {
            KeyManagerError::DerivationFailedWithReason {
                reason: e.to_string(),
            }
        })?;

        let secret_key = Zeroizing::new(
            RistrettoBridge::to_ristretto_key(&address_key, self.chain, path).map_err(|e| {
                KeyManagerError::DerivationFailedWithReason {
                    reason: e.to_string(),
                }
            })?,
        );

        Ok(RistrettoPublicKey::from_secret_key(&secret_key))
    }

    fn validate_cached_pair(
        &self,
        expected: &RistrettoPublicKey,
        cached: &RistrettoPublicKey,
    ) -> Result<()> {
        self.verify_key(expected)?;

        if self.verify_key(cached).is_err() {
            return Err(self.report_cache_corruption());
        }

        if expected.ct_eq(cached).unwrap_u8() == 0 {
            return Err(self.report_cache_corruption());
        }

        Ok(())
    }

    fn cache_entry_is_fresh(cached_key: &CachedKey, now: u64) -> bool {
        now.saturating_sub(cached_key.cached_at) < DERIVED_KEY_TTL_SECONDS
    }

    fn load_cached_public_key(&self, path: &Bip44Path) -> Result<Option<RistrettoPublicKey>> {
        let now = self.time_provider.compat_unix_timestamp();

        if let Some(cached_key) = self.cache_read()?.peek(path).cloned() {
            if Self::cache_entry_is_fresh(&cached_key, now)
                && self.verify_key(&cached_key.public_key).is_ok()
            {
                return Ok(Some(cached_key.public_key));
            }
        } else {
            return Ok(None);
        }

        let mut cache = self.cache_write()?;
        let Some(cached_key) = cache.peek(path).cloned() else {
            return Ok(None);
        };

        if !Self::cache_entry_is_fresh(&cached_key, now) {
            cache.pop(path);
            return Ok(None);
        }

        if self.verify_key(&cached_key.public_key).is_ok() {
            return Ok(Some(cached_key.public_key));
        }

        cache.pop(path);
        let _ = self.report_cache_corruption();
        Ok(None)
    }

    fn load_or_derive_public_key(&self, path: &Bip44Path) -> Result<RistrettoPublicKey> {
        struct DerivationFlightGuard<'a, R: SecureRngProvider> {
            manager: &'a KeyManagerImpl<R>,
            path: Bip44Path,
            flight: Arc<DerivationFlight>,
        }

        impl<R: SecureRngProvider> Drop for DerivationFlightGuard<'_, R> {
            fn drop(&mut self) {
                self.manager.finish_derivation_flight(&self.path, &self.flight);
            }
        }

        if let Some(public_key) = self.load_cached_public_key(path)? {
            return Ok(public_key);
        }

        loop {
            let flight = {
                let mut deriving_paths = self.deriving_paths_lock()?;
                if let Some(flight) = deriving_paths.get(path).cloned() {
                    flight
                } else {
                    let flight = Arc::new(DerivationFlight::active());
                    deriving_paths.insert(*path, Arc::clone(&flight));
                    drop(deriving_paths);

                    let _flight_guard = DerivationFlightGuard {
                        manager: self,
                        path: *path,
                        flight: Arc::clone(&flight),
                    };

                    let derive_result = (|| {
                        if let Some(public_key) = self.load_cached_public_key(path)? {
                            return Ok(public_key);
                        }
                        {
                            self.derivation_count.fetch_add(1, Ordering::Relaxed);
                        }

                        let public_key = self.derive_from_manager(path)?;
                        let cached_key = CachedKey {
                            public_key: public_key.clone(),
                            cached_at: self.time_provider.compat_unix_timestamp(),
                        };
                        self.cache_write()?.put(*path, cached_key);
                        Ok(public_key)
                    })();

                    self.store_derivation_result(&flight, &derive_result)?;
                    return derive_result;
                }
            };

            let mut is_deriving = flight.is_deriving.lock().map_err(|_| {
                KeyManagerError::LockPoisoned {
                    lock: "derivation_flight",
                }
            })?;
            while *is_deriving {
                is_deriving = flight.wait.wait(is_deriving).map_err(|_| {
                    KeyManagerError::LockPoisoned {
                        lock: "derivation_flight",
                    }
                })?;
            }
            drop(is_deriving);

            if let Some(public_key) = self.load_cached_public_key(path)? {
                return Ok(public_key);
            }

            if let Some(derive_result) = self.load_derivation_result(&flight)? {
                return derive_result;
            }

            let mut deriving_paths = self.deriving_paths_lock()?;
            if deriving_paths
                .get(path)
                .is_some_and(|active_flight| Arc::ptr_eq(active_flight, &flight))
            {
                deriving_paths.remove(path);
                break;
            }
        }

        self.load_or_derive_public_key(path)
    }

}