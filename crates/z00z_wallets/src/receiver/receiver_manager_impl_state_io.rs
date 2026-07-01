impl<K: KeyManager, T: TimeProvider> ReceiverManagerImpl<K, T> {
    /// Export non-expired cache entries into an authenticated state record.
    pub fn export_cache(&self, wallet_id: &[u8]) -> ReceiverManagerResult<ReceiverCacheState> {
        let now = self.time_provider.compat_unix_timestamp();
        let ttl = self.ttl;
        let cache = self.cache_read()?;

        let entries = cache
            .iter()
            .filter(|(_, entry)| {
                let age_seconds = now.saturating_sub(entry.inserted_at);
                let age = Duration::from_secs(age_seconds);
                age <= ttl
            })
            .map(|(path, entry)| {
                (
                    *path,
                    entry.keys.spend_key.as_bytes().to_vec(),
                    entry.keys.view_key.as_bytes().to_vec(),
                )
            })
            .collect();

        let mut state_record = ReceiverCacheState {
            entries,
            version: 3,
            hmac: [0u8; 32],
        };

        let mac_key = ReceiverCacheState::mac_key(&self.key_manager, wallet_id)?;
        state_record.sign(wallet_id, &mac_key)?;
        Ok(state_record)
    }

    /// Import an authenticated receiver-cache state atomically.
    pub fn import_cache(
        &mut self,
        wallet_id: &[u8],
        state_record: ReceiverCacheState,
    ) -> ReceiverManagerResult<()> {
        let reject_snapshot = |metrics: &CacheMetrics, reason: &str, err: ReceiverManagerError| {
            metrics.inc_import_rejects();
            z00z_utils::logger::Logger::warn(
                &z00z_utils::logger::TracingLogger,
                &format!("Rejecting cache import (atomic): {}", reason),
            );
            err
        };

        if state_record.version != 3 {
            return Err(reject_snapshot(
                &self.metrics,
                &format!(
                    "receiver cache state version mismatch: {} (expected: 3)",
                    state_record.version
                ),
                ReceiverManagerError::ImportEntryRejected(format!(
                    "receiver cache state version mismatch: {} (expected: 3)",
                    state_record.version
                )),
            ));
        }

        let entry_count = state_record.entries.len();
        if entry_count > MAX_IMPORT_ENTRIES {
            return Err(reject_snapshot(
                &self.metrics,
                &format!(
                    "too many entries: {} (limit: {})",
                    entry_count, MAX_IMPORT_ENTRIES
                ),
                ReceiverManagerError::ImportTooLarge(entry_count),
            ));
        }

        let mut total_bytes: usize = 0;
        for (_path, spend_bytes, view_bytes) in &state_record.entries {
            total_bytes = total_bytes.saturating_add(spend_bytes.len());
            total_bytes = total_bytes.saturating_add(view_bytes.len());
            if total_bytes > MAX_IMPORT_SIZE_BYTES {
                break;
            }
        }
        if total_bytes > MAX_IMPORT_SIZE_BYTES {
            return Err(reject_snapshot(
                &self.metrics,
                &format!(
                    "payload too large: {} bytes (limit: {})",
                    total_bytes, MAX_IMPORT_SIZE_BYTES
                ),
                ReceiverManagerError::ImportExceedsSizeLimit(total_bytes),
            ));
        }

        let mac_key = ReceiverCacheState::mac_key(&self.key_manager, wallet_id)?;
        if let Err(error) = state_record.verify(wallet_id, &mac_key) {
            return Err(reject_snapshot(
                &self.metrics,
                &format!("receiver cache state authentication failed: {}", error),
                error,
            ));
        }

        let entries = state_record.entries;

        if entries.is_empty() {
            let cache_len = self.cache_read()?.len();
            self.metrics
                .current_cache_size
                .store(cache_len as u64, Ordering::Relaxed);
            return Ok(());
        }

        let now = self.time_provider.compat_unix_timestamp();
        self.maybe_purge(now)?;
        let mut validated: Vec<(Bip44Path, DerivedWalletKeys)> = Vec::with_capacity(entry_count);

        for (path, spend_bytes, view_bytes) in entries {
            if Bip44Validator::validate(&path).is_err() {
                return Err(reject_snapshot(
                    &self.metrics,
                    "invalid derivation path in entry",
                    ReceiverManagerError::ImportEntryRejected("invalid derivation path".to_string()),
                ));
            }

            if spend_bytes.len() != 32 || view_bytes.len() != 32 {
                return Err(reject_snapshot(
                    &self.metrics,
                    &format!(
                        "invalid key bytes length: spend={}, view={}",
                        spend_bytes.len(),
                        view_bytes.len()
                    ),
                    ReceiverManagerError::ImportEntryRejected(format!(
                        "invalid key bytes length: spend={}, view={}",
                        spend_bytes.len(),
                        view_bytes.len()
                    )),
                ));
            }

            let spend_key = Z00ZRistrettoPoint::try_from_bytes(
                spend_bytes
                    .as_slice()
                    .try_into()
                    .map_err(|_| {
                        reject_snapshot(
                            &self.metrics,
                            "invalid spend key bytes length",
                            ReceiverManagerError::ImportEntryRejected(
                                "invalid spend key bytes length".to_string(),
                            ),
                        )
                    })?,
            )
            .map_err(|error| {
                reject_snapshot(
                    &self.metrics,
                    &format!("invalid spend key bytes: {}", error),
                    ReceiverManagerError::ImportEntryRejected(format!(
                        "invalid spend key bytes: {}",
                        error
                    )),
                )
            })?;
            if spend_key.as_bytes() == [0u8; 32] {
                return Err(reject_snapshot(
                    &self.metrics,
                    "identity spend key in entry",
                    ReceiverManagerError::ImportEntryRejected("identity spend key".to_string()),
                ));
            }

            let view_key = Z00ZRistrettoPoint::try_from_bytes(
                view_bytes
                    .as_slice()
                    .try_into()
                    .map_err(|_| {
                        reject_snapshot(
                            &self.metrics,
                            "invalid view key bytes length",
                            ReceiverManagerError::ImportEntryRejected(
                                "invalid view key bytes length".to_string(),
                            ),
                        )
                    })?,
            )
            .map_err(|error| {
                reject_snapshot(
                    &self.metrics,
                    &format!("invalid view key bytes: {}", error),
                    ReceiverManagerError::ImportEntryRejected(format!(
                        "invalid view key bytes: {}",
                        error
                    )),
                )
            })?;
            if view_key.as_bytes() == [0u8; 32] {
                return Err(reject_snapshot(
                    &self.metrics,
                    "identity view key in entry",
                    ReceiverManagerError::ImportEntryRejected("identity view key".to_string()),
                ));
            }

            validated.push((
                path,
                DerivedWalletKeys {
                    spend_key,
                    view_key,
                },
            ));
        }

        self.metrics.add_import_entries(entry_count as u64);
        self.metrics.add_import_bytes(total_bytes as u64);

        let evicted: Vec<(Bip44Path, Z00ZRistrettoPoint)> = {
            let mut cache = self.cache_write()?;
            let mut evicted = Vec::new();

            for (path, keys) in validated {
                let entry = CachedEntry {
                    keys,
                    inserted_at: now,
                };

                if let Some((evicted_path, evicted_entry)) = cache.push(path, entry) {
                    self.metrics.inc_evictions();
                    evicted.push((evicted_path, evicted_entry.keys.spend_key.clone()));
                }
            }

            evicted
        };

        for (path, key) in evicted {
            self.notify_evict(path, key);
        }

        let current_size = self.cache_read()?.len() as u64;
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

        Ok(())
    }

    /// Pre-derive common payment paths into the cache.
    pub fn warm_cache(&mut self, max_index: u32) -> ReceiverManagerResult<()> {
        for index in 0..max_index {
            let path = Bip44Path::payment(index)?;
            match self.get_receiver_key(path) {
                Ok(_) => {}
                Err(ReceiverManagerError::NotFound(_)) => {
                    self.derive_spend_key(path)?;
                }
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
}
