impl<R: SecureRngProvider> KeyManagerImpl<R> {
    fn validate_empty_manager_state(&self) -> Result<()> {
        if self.encrypted_seed.is_some() && self.bip44_manager.is_none() {
            return Err(KeyManagerError::StateCorrupted);
        }

        if self.bip44_manager.is_some() {
            return Ok(());
        }

        let cache = self.cache_read()?;
        if !cache.is_empty() {
            return Err(KeyManagerError::StateCorrupted);
        }

        Err(KeyManagerError::NotInitialized)
    }

    fn validate_manager_chain(&self, manager: &Bip44KeyManager) -> Result<()> {
        if manager.chain() != self.chain {
            self.logger.error("Chain mismatch detected");
            return Err(KeyManagerError::StateCorrupted);
        }

        Ok(())
    }

    fn validate_gap_counters(&self) -> Result<()> {
        let gap_ext = self.gap_external.load(Ordering::Acquire);
        let gap_int = self.gap_internal.load(Ordering::Acquire);
        let last_ext = self.last_used_ext.load(Ordering::Acquire);
        let last_int = self.last_used_int.load(Ordering::Acquire);
        if gap_ext < last_ext || gap_int < last_int {
            self.logger.error("Gap counters corrupted");
            return Err(KeyManagerError::StateCorrupted);
        }

        Ok(())
    }

    /// Validate the persisted and in-memory key-manager state for consistency.
    pub fn validate_state(&self) -> Result<()> {
        self.validate_empty_manager_state()?;

        let manager = self
            .bip44_manager
            .as_ref()
            .ok_or(KeyManagerError::NotInitialized)?;
        self.validate_manager_chain(manager)?;
        self.validate_gap_counters()?;
        self.validate_cached_keys()?;
        Ok(())
    }

    /// Initialize the key manager directly from raw BIP-39 seed bytes.
    pub fn init_from_seed(&mut self, seed: &[u8], chain: ChainType) -> Result<()> {
        let bip39_seed =
            Bip39Seed64::from_slice(seed).map_err(|_| KeyManagerError::DerivationFailed)?;

        bip39_seed
            .validate_entropy()
            .map_err(|e| KeyManagerError::WeakEntropy(e.to_string()))?;

        let manager = Bip44KeyManager::new(bip39_seed, Z00Z_BIP44_ASSET, chain)
            .map_err(|_| KeyManagerError::DerivationFailed)?;

        self.encrypted_seed = None;
        self.bip44_manager = Some(manager);
        self.chain = chain;
        self.cache_write()?.clear();
        self.validate_state()?;

        Ok(())
    }

    /// Initialize the key manager by decrypting and loading an encrypted seed container.
    pub fn init_from_encrypted_seed(
        &mut self,
        encrypted_seed: CipherSeedContainer,
        password: &z00z_crypto::expert::encoding::SafePassword,
        wallet_id: &[u8],
        purpose: &[u8],
        chain: ChainType,
    ) -> Result<()> {
        let decrypted_seed = encrypted_seed
            .decrypt_wallet(password, wallet_id, purpose, chain)
            .map_err(|_| KeyManagerError::DerivationFailed)?;

        let bip39_seed = decrypted_seed
            .with_revealed(|seed_bytes| Bip39Seed64::from_slice(seed_bytes.as_ref()))
            .map_err(|_| KeyManagerError::DerivationFailed)?;

        bip39_seed
            .validate_entropy()
            .map_err(|e| KeyManagerError::WeakEntropy(e.to_string()))?;

        let manager = Bip44KeyManager::new(bip39_seed, Z00Z_BIP44_ASSET, chain)
            .map_err(|_| KeyManagerError::DerivationFailed)?;

        self.encrypted_seed = Some(encrypted_seed);
        self.bip44_manager = Some(manager);
        self.chain = chain;
        self.cache_write()?.clear();
        self.validate_state()?;

        Ok(())
    }

    /// Derive the external payment public key for the provided address index.
    pub fn derive_payment_key(&mut self, index: u32) -> Result<RistrettoPublicKey> {
        let manager = self
            .bip44_manager
            .as_ref()
            .ok_or(KeyManagerError::NotInitialized)?;

        let address_key = manager.derive_address_key(0, 0, index).map_err(|e| {
            KeyManagerError::DerivationFailedWithReason {
                reason: e.to_string(),
            }
        })?;

        let path = Bip44Path::new_z00z(0, 0, index).map_err(|e| {
            KeyManagerError::DerivationFailedWithReason {
                reason: e.to_string(),
            }
        })?;

        let secret_key = Zeroizing::new(
            RistrettoBridge::to_ristretto_key(&address_key, self.chain, &path).map_err(|e| {
                KeyManagerError::DerivationFailedWithReason {
                    reason: e.to_string(),
                }
            })?,
        );

        Ok(RistrettoPublicKey::from_secret_key(&secret_key))
    }

    /// Derive the internal change public key for the provided address index.
    pub fn derive_change_key(&mut self, index: u32) -> Result<RistrettoPublicKey> {
        let manager = self
            .bip44_manager
            .as_ref()
            .ok_or(KeyManagerError::NotInitialized)?;

        let address_key = manager.derive_address_key(0, 1, index).map_err(|e| {
            KeyManagerError::DerivationFailedWithReason {
                reason: e.to_string(),
            }
        })?;

        let path = Bip44Path::new_z00z(0, 1, index).map_err(|e| {
            KeyManagerError::DerivationFailedWithReason {
                reason: e.to_string(),
            }
        })?;

        let secret_key = Zeroizing::new(
            RistrettoBridge::to_ristretto_key(&address_key, self.chain, &path).map_err(|e| {
                KeyManagerError::DerivationFailedWithReason {
                    reason: e.to_string(),
                }
            })?,
        );

        Ok(RistrettoPublicKey::from_secret_key(&secret_key))
    }

    /// Reject identity public keys before they enter cache or signing flows.
    pub fn verify_key(&self, key: &RistrettoPublicKey) -> Result<()> {
        if key.ct_eq(&RistrettoPublicKey::default()).unwrap_u8() != 0 {
            return Err(KeyManagerError::InvalidPublicKey);
        }
        Ok(())
    }

    /// Serialize the current encrypted-seed state for wallet persistence.
    pub fn to_state(&self, metadata: KeyManagerMetadata) -> Result<KeyManagerState> {
        let encrypted_seed = self
            .encrypted_seed
            .as_ref()
            .ok_or(KeyManagerError::NotInitialized)?;

        let encrypted_seed_bytes = encrypted_seed
            .to_bytes()
            .map_err(|_| KeyManagerError::StateCorrupted)?;

        Ok(KeyManagerState {
            encrypted_seed_bytes,
            metadata,
            chain: self.chain,
        })
    }

    /// Rebuild a key manager from persisted wallet state using a caller-provided RNG provider.
    pub fn try_from_state_with_rng(
        state: KeyManagerState,
        password: &z00z_crypto::expert::encoding::SafePassword,
        wallet_id: &[u8],
        purpose: &[u8],
        rng_provider: R,
    ) -> Result<Self> {
        let encrypted_seed = CipherSeedContainer::from_bytes(&state.encrypted_seed_bytes)
            .map_err(|_| KeyManagerError::StateCorrupted)?;

        let decrypted_seed = encrypted_seed
            .decrypt_wallet(password, wallet_id, purpose, state.chain)
            .map_err(|_| KeyManagerError::DerivationFailed)?;

        let bip39_seed = decrypted_seed
            .with_revealed(|seed_bytes| Bip39Seed64::from_slice(seed_bytes.as_ref()))
            .map_err(|_| KeyManagerError::DerivationFailed)?;

        bip39_seed
            .validate_entropy()
            .map_err(|e| KeyManagerError::WeakEntropy(e.to_string()))?;

        let manager = Bip44KeyManager::new(bip39_seed, Z00Z_BIP44_ASSET, state.chain)
            .map_err(|_| KeyManagerError::DerivationFailed)?;

        let key_manager = Self {
            encrypted_seed: Some(encrypted_seed),
            bip44_manager: Some(manager),
            derived_public_keys: RwLock::new(LruCache::new(Self::derived_pubkey_cache_capacity())),
            deriving_paths: Mutex::new(HashMap::new()),
            derivation_count: AtomicUsize::new(0),
            chain: state.chain,
            logger: Arc::new(NoopLogger),
            metrics: Arc::new(NoopMetrics),
            time_provider: Arc::new(SystemTimeProvider),
            rng_provider,
            gap_external: AtomicU32::new(0),
            gap_internal: AtomicU32::new(0),
            last_used_ext: AtomicU32::new(0),
            last_used_int: AtomicU32::new(0),
            derive_count: AtomicU32::new(0),
        };

        key_manager.validate_state()?;
        Ok(key_manager)
    }

    fn nonce_from_seed(nonce_seed: &[u8; 64]) -> Result<RistrettoSecretKey> {
        let nonce = RistrettoSecretKey::from_uniform_bytes(nonce_seed)
            .map_err(|_| KeyManagerError::SignatureFailed)?;

        if nonce.ct_eq(&RistrettoSecretKey::default()).unwrap_u8() != 0 {
            return Err(KeyManagerError::SignatureFailed);
        }

        Ok(nonce)
    }

    /// Re-encrypt the stored seed container under a new wallet password.
    pub fn change_password(
        &mut self,
        old_password: &z00z_crypto::expert::encoding::SafePassword,
        new_password: &z00z_crypto::expert::encoding::SafePassword,
        wallet_id: &[u8],
        purpose: &[u8],
    ) -> Result<()> {
        let current_seed = self
            .encrypted_seed
            .as_ref()
            .ok_or(KeyManagerError::NotInitialized)?;

        let new_container = current_seed
            .re_encrypt(old_password, new_password, wallet_id, purpose, self.chain)
            .map_err(|_| KeyManagerError::AuthenticationFailed)?;

        self.encrypted_seed = Some(new_container);
        self.cache_write()?.clear();

        Ok(())
    }
}
