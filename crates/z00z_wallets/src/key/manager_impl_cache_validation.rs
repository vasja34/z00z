impl<R: SecureRngProvider> KeyManagerImpl<R> {
    fn run_spot_check_if_needed(&self, should_spot_check: bool) -> Result<()> {
        if !should_spot_check {
            return Ok(());
        }

        if let Err(error) = self.spot_check_cache() {
            self.logger
                .error(&format!("Cache spot-check failed: {}", error));
            self.metrics
                .inc_counter("wallet.key.cache.spot_check_fail", 1);

            if let Ok(mut cache) = self.cache_write() {
                cache.clear();
            }

            return Err(error);
        }

        self.metrics
            .inc_counter("wallet.key.cache.spot_check_ok", 1);
        Ok(())
    }

    fn record_derive_metrics(&self, start_ms: u64) {
        let end_ms = self.time_provider.compat_unix_timestamp_millis();
        let duration_ms = end_ms.saturating_sub(start_ms) as f64;
        self.metrics.inc_counter("wallet.key.derive.count", 1);
        self.metrics
            .observe_histogram("wallet.key.derive.ms", duration_ms);
    }

    fn derive_from_manager(&self, path: &Bip44Path) -> Result<RistrettoPublicKey> {
        let manager = self
            .bip44_manager
            .as_ref()
            .ok_or(KeyManagerError::NotInitialized)?;

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

        let public_key = RistrettoPublicKey::from_secret_key(&secret_key);
        self.verify_key(&public_key)?;
        Ok(public_key)
    }

    fn validate_cached_keys(&self) -> Result<()> {
        let manager = match self.bip44_manager.as_ref() {
            Some(manager) => manager,
            None => return Ok(()),
        };

        let cache = self.cache_read()?;
        for (path, cached_key) in cache.iter() {
            let expected = self.expected_public_key_for_path(manager, path)?;
            self.validate_cached_pair(&expected, &cached_key.public_key)?;
        }

        self.metrics.inc_counter("wallet.key.cache.validate_ok", 1);
        Ok(())
    }

    fn pick_cached_entry(&self) -> Result<Option<(Bip44Path, CachedKey)>> {
        let cache = self.cache_read()?;
        if cache.is_empty() {
            return Ok(None);
        }

        let paths: Vec<_> = cache.iter().map(|(path, _)| *path).collect();

        use rand::seq::SliceRandom;
        let mut rng = self.rng_provider.rng();
        let random_path = paths
            .choose(&mut rng)
            .ok_or(KeyManagerError::CacheCorrupted)?;

        let cached_key = cache
            .peek(random_path)
            .ok_or(KeyManagerError::CacheCorrupted)?
            .clone();

        Ok(Some((*random_path, cached_key)))
    }

    fn spot_check_cache(&self) -> Result<()> {
        let manager = self
            .bip44_manager
            .as_ref()
            .ok_or(KeyManagerError::NotInitialized)?;

        let Some((random_path, cached_key)) = self.pick_cached_entry()? else {
            return Ok(());
        };

        let expected = self.expected_public_key_for_path(manager, &random_path)?;
        self.validate_cached_pair(&expected, &cached_key.public_key)?;

        Ok(())
    }
}