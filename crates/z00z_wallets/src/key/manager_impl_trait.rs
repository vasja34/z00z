impl<R: SecureRngProvider> KeyManager for KeyManagerImpl<R> {
    fn clear(&mut self) {
        self.encrypted_seed = None;

        if let Some(mut manager) = self.bip44_manager.take() {
            manager.zeroize_seed();
        }

        if let Ok(mut cache) = self.cache_write() {
            cache.clear();
        }
        self.chain = ChainType::Devnet;
        let _ = self.validate_state();
    }

    fn derive_key(&self, path: &Bip44Path) -> Result<RistrettoPublicKey> {
        let start_ms = self.time_provider.compat_unix_timestamp_millis();
        self.validate_path(path)?;

        let derive_count = self
            .derive_count
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1);
        let should_spot_check = derive_count.is_multiple_of(CACHE_SPOT_CHECK_FREQUENCY);

        #[cfg(feature = "verbose-logging")]
        self.logger.debug(&format!("derive_key path={path}"));

        let public_key = self.load_or_derive_public_key(path)?;
        self.run_spot_check_if_needed(should_spot_check)?;
        self.record_derive_metrics(start_ms);

        Ok(public_key)
    }

    fn get_public_key(&self, path: &Bip44Path) -> Option<RistrettoPublicKey> {
        let now = self.time_provider.compat_unix_timestamp();

        self.cache_read().ok().and_then(|cache| {
            cache.peek(path).and_then(|cached_key| {
                if now.saturating_sub(cached_key.cached_at) < DERIVED_KEY_TTL_SECONDS {
                    let pk = cached_key.public_key.clone();
                    if self.verify_key(&pk).is_ok() {
                        Some(pk)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        })
    }

    fn derive_secret_transient(&self, path: &Bip44Path) -> Result<Zeroizing<RistrettoSecretKey>> {
        self.validate_path(path)?;

        let manager = self
            .bip44_manager
            .as_ref()
            .ok_or(KeyManagerError::NotInitialized)?;

        let address_key = manager.derive_address_key_for_path(path).map_err(|e| {
            KeyManagerError::DerivationFailedWithReason {
                reason: e.to_string(),
            }
        })?;

        let secret_key = RistrettoBridge::to_ristretto_key(&address_key, self.chain, path)
            .map_err(|e| KeyManagerError::DerivationFailedWithReason {
                reason: e.to_string(),
            })?;

        Ok(Zeroizing::new(secret_key))
    }

    fn sign(&self, path: &Bip44Path, msg: &[u8]) -> Result<KernelSignature> {
        let sk = self.derive_secret_transient(path)?;
        let pk = RistrettoPublicKey::from_secret_key(&*sk);

        #[cfg(not(test))]
        let nonce_hash = DomainHasher::<WalletSignNonceProdDomain>::new_with_label(
            "wallet_sign_nonce",
        )
        .chain(sk.as_bytes())
        .chain(path.to_bytes())
        .chain(msg)
        .finalize();

        #[cfg(test)]
        let nonce_hash = DomainHasher::<WalletSignNonceTestDomain>::new_with_label(
            "wallet_sign_nonce",
        )
        .chain(sk.as_bytes())
        .chain(path.to_bytes())
        .chain(msg)
        .finalize();

        let mut nonce_seed = [0u8; 64];
        nonce_seed.copy_from_slice(&nonce_hash.as_ref()[..64]);

        let nonce = Self::nonce_from_seed(&nonce_seed)?;
        let public_nonce = RistrettoPublicKey::from_secret_key(&nonce);

        let public_nonce_bytes: &[u8; 32] = public_nonce
            .as_bytes()
            .try_into()
            .map_err(|_| KeyManagerError::InvalidParameters)?;
        let public_key_bytes: &[u8; 32] = pk
            .as_bytes()
            .try_into()
            .map_err(|_| KeyManagerError::InvalidParameters)?;

        let challenge = compute_schnorr_challenge(
            public_nonce_bytes,
            public_key_bytes,
            msg,
            ChallengeSize::B512,
        )
        .into_b512()
        .ok_or(KeyManagerError::InvalidParameters)?;

        let sig = KernelSignature::sign_raw_uniform(&*sk, nonce, &challenge)
            .map_err(|_| KeyManagerError::SignatureFailed)?;

        if sig
            .get_signature()
            .ct_eq(&RistrettoSecretKey::default())
            .unwrap_u8()
            != 0
        {
            return Err(KeyManagerError::SignatureFailed);
        }

        Ok(sig)
    }
}
