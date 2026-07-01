impl WalletService {
    pub(crate) async fn runtime_seed_salt(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<[u8; 16]> {
        let store = self.wallet_seed_salts.read().await;
        store.get(wallet_id).copied().ok_or_else(|| {
            WalletError::InvalidConfig(format!("Wallet seed salt missing for {}", wallet_id.0))
        })
    }

    pub(crate) async fn seed_salt_for_save(&self, wallet_id: &PersistWalletId) -> [u8; 16] {
        {
            let store = self.wallet_seed_salts.read().await;
            if let Some(seed_salt) = store.get(wallet_id).copied() {
                return seed_salt;
            }
        }

        let seed_salt = self.make_seed_salt();
        let mut store = self.wallet_seed_salts.write().await;
        let entry = store.entry(wallet_id.clone()).or_insert(seed_salt);
        *entry
    }

    pub(crate) fn require_now_ms(&self) -> WalletResult<u64> {
        self.time_provider
            .try_unix_timestamp_ms()
            .map_err(|e| WalletError::InvalidConfig(format!("clock unavailable: {e}")))
    }

    pub(crate) fn generate_wallet_id(&self, _name: &str, now_ms: u64) -> PersistWalletId {
        let mut random_bytes = [0u8; 16];
        self.entropy.fill_bytes(&mut random_bytes);

        let core_wallet_id =
            CoreWalletId::from_create_wallet_inputs(CoreChainId::DEVNET, now_ms, random_bytes);
        PersistWalletId(format!("wallet_{}", core_wallet_id.to_hex()))
    }

    pub(crate) fn timeout_ms(&self) -> u64 {
        self.auto_lock_policy.timeout.as_millis() as u64
    }

    pub(crate) fn parse_core_wallet_id(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<CoreWalletId> {
        let raw = wallet_id.0.trim();
        let hex_part = raw.strip_prefix("wallet_").unwrap_or(raw);

        let bytes = hex::decode(hex_part).map_err(|e| {
            WalletError::InvalidParams(format!(
                "Invalid wallet id: expected 'wallet_<64 hex chars>' or '<64 hex chars>': {e}"
            ))
        })?;

        if bytes.len() != 32 {
            return Err(WalletError::InvalidParams(format!(
                "Invalid wallet id length: expected 32 bytes, got {}",
                bytes.len()
            )));
        }

        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        Ok(CoreWalletId(out))
    }

    pub(crate) fn create_receiver_deriver_state(
        seed_bip39: Hidden<[u8; 64]>,
        counters: ReceiverDeriverState,
        chain_type: ChainType,
        cache_size: usize,
        rate_limit: Option<crate::services::wallet_runtime_config::ReceiverDeriveRateLimit>,
    ) -> WalletResult<WalletReceiverDeriverState> {
        let mut key_manager = KeyManagerImpl::new();

        // Phase 5: Pure BIP-39 → BIP-32 (no pre-KDF)
        // Pass the 64-byte BIP-39 seed output directly as BIP-32 seed input
        // No additional transformation is applied
        key_manager
            .init_from_seed(seed_bip39.reveal(), chain_type)
            .map_err(|e| WalletError::KeyDerivation(e.to_string()))?;

        let mut builder = ReceiverManagerImpl::new(key_manager).with_limit(cache_size);
        if let Some(limit) = rate_limit {
            builder = builder.with_rate_limit(limit.rate_per_sec, limit.burst);
        }

        let receiver_manager = builder
            .build()
            .map_err(|e| WalletError::InvalidParams(e.to_string()))?;

        Ok(WalletReceiverDeriverState {
            receiver_manager,
            next_payment_index: counters.next_payment_index,
            next_change_index: counters.next_change_index,
        })
    }

    pub(crate) async fn get_create_wallet_receiver_deriver(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<WalletReceiverDeriverHandle> {
        let chain_type = self.resolve_persisted_wallet_chain_type(wallet_id).await?;

        #[cfg(not(target_arch = "wasm32"))]
        let seed_bip39 = {
            let now_ms = self.require_now_ms()?;
            let timeout_ms = self.timeout_ms();
            let session = self
                .wallet_sessions
                .session_for_wallet(wallet_id, now_ms, timeout_ms)
                .await?;

            session.with_wallet_session(|wlt_session| {
                let mut seed = [0u8; 64];
                seed.copy_from_slice(wlt_session.opened().seed_bip39.reveal());
                Ok(Hidden::hide(seed))
            })?
        };

        if let Some(existing) = {
            let store = self.wallet_receiver_derivers.read().await;
            store.get(wallet_id).cloned()
        } {
            return Ok(existing);
        }

        let counters = {
            let store = self.wallet_receiver_deriver_counters.read().await;
            store
                .get(wallet_id)
                .copied()
                .unwrap_or(ReceiverDeriverState {
                    next_payment_index: 0,
                    next_change_index: 0,
                })
        };

        let new_handle = Arc::new(RwLock::new(Self::create_receiver_deriver_state(
            seed_bip39,
            counters,
            chain_type,
            self.receiver_cache_size,
            self.receiver_derive_rate_limit,
        )?));

        let mut store = self.wallet_receiver_derivers.write().await;
        if let Some(existing) = store.get(wallet_id).cloned() {
            return Ok(existing);
        }

        store.insert(wallet_id.clone(), Arc::clone(&new_handle));
        Ok(new_handle)
    }

    /// Derive a public key for an explicit BIP-44 path (Phase 1).
    ///
    /// This is used by `key.derive` to support deterministic derivation
    /// without introducing persistent storage.
    pub async fn derive_public_key_for_path(
        &self,
        wallet_id: &PersistWalletId,
        path: Bip44Path,
    ) -> WalletResult<[u8; 32]> {
        let deriver = self.get_create_wallet_receiver_deriver(wallet_id).await?;
        let mut state = deriver.write().await;

        let public_key = state
            .receiver_manager
            .derive_spend_key(path)
            .map_err(|e| WalletError::KeyDerivation(e.to_string()))?;

        let bytes = public_key.as_bytes();
        let mut out = [0u8; 32];
        out.copy_from_slice(bytes);

        // Persist derivation progress for recovery determinism.
        let next_index = path.address_index().index().saturating_add(1);
        if path.is_payment() {
            state.next_payment_index = state.next_payment_index.max(next_index);
        } else if path.is_change() {
            state.next_change_index = state.next_change_index.max(next_index);
        }

        {
            let mut store = self.wallet_receiver_deriver_counters.write().await;
            store.insert(
                wallet_id.clone(),
                ReceiverDeriverState {
                    next_payment_index: state.next_payment_index,
                    next_change_index: state.next_change_index,
                },
            );
        }

        Ok(out)
    }

    pub(crate) async fn persist_profile_for_open_session(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<()> {
        use z00z_utils::codec::{BincodeCodec, Codec};

        let profile = self.create_profile(wallet_id).await?;
        let codec = BincodeCodec;
        let profile_bytes = codec.serialize(&profile).map_err(|e| {
            WalletError::InvalidConfig(format!("Binary serialization failed: {}", e))
        })?;
        let cache_state_payload = self.build_receiver_cache_state_payload(wallet_id).await;

        self.persist_session_profile(wallet_id, profile_bytes, cache_state_payload)
            .await
    }

    async fn build_receiver_cache_state_payload(
        &self,
        wallet_id: &PersistWalletId,
    ) -> Option<(std::path::PathBuf, Vec<u8>)> {
        use z00z_utils::codec::{BincodeCodec, Codec};

        let codec = BincodeCodec;
        let deriver = {
            let store = self.wallet_receiver_derivers.read().await;
            store.get(wallet_id).cloned()
        };

        match deriver {
            None => None,
            Some(deriver) => match self.parse_core_wallet_id(wallet_id) {
                Ok(core_wallet_id) => {
                    let state = deriver.read().await;
                    match state.receiver_manager.export_cache(&core_wallet_id.0) {
                        Ok(snap) => match codec.serialize(&snap) {
                            Ok(cache_bytes) => {
                                Some((self.receiver_cache_file_path(wallet_id), cache_bytes))
                            }
                            Err(e) => {
                                z00z_utils::logger::Logger::warn(
                                    &z00z_utils::logger::TracingLogger,
                                    &format!(
                                        "Skipping receiver cache state persistence: serialization failed: {}",
                                        e
                                    ),
                                );
                                None
                            }
                        },
                        Err(e) => {
                            z00z_utils::logger::Logger::warn(
                                &z00z_utils::logger::TracingLogger,
                                &format!(
                                    "Skipping receiver cache state persistence: export failed: {}",
                                    e
                                ),
                            );
                            None
                        }
                    }
                }
                Err(e) => {
                    z00z_utils::logger::Logger::warn(
                        &z00z_utils::logger::TracingLogger,
                        &format!(
                            "Skipping receiver cache state persistence: invalid wallet id: {}",
                            e
                        ),
                    );
                    None
                }
            },
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn persist_session_profile(
        &self,
        wallet_id: &PersistWalletId,
        profile_bytes: Vec<u8>,
        cache_state_payload: Option<(std::path::PathBuf, Vec<u8>)>,
    ) -> WalletResult<()> {
        let session = {
            let now_ms = self.require_now_ms()?;
            let timeout_ms = self.timeout_ms();
            self.wallet_sessions
                .session_for_wallet(wallet_id, now_ms, timeout_ms)
                .await?
        };
        let wlt_store = Arc::clone(&self.wlt_store);

        tokio::task::spawn_blocking(move || {
            session.with_wallet_session(|wlt_session| {
                wlt_store
                    .write_wallet_profile(wlt_session, profile_bytes)
                    .map(|_| ())?;

                if let Some((path, bytes)) = cache_state_payload {
                    if let Some(parent) = path.parent() {
                        if let Err(e) = z00z_utils::io::create_dir_all(parent) {
                            z00z_utils::logger::Logger::warn(
                                &z00z_utils::logger::TracingLogger,
                                &format!(
                                    "Failed to create receiver cache state directory {}: {}",
                                    parent.display(),
                                    e
                                ),
                            );
                        }
                    }
                    if let Err(e) = z00z_utils::io::write_file(&path, &bytes) {
                        z00z_utils::logger::Logger::warn(
                            &z00z_utils::logger::TracingLogger,
                            &format!(
                                "Failed to write receiver cache state {}: {}",
                                path.display(),
                                e
                            ),
                        );
                    }
                }

                Ok(())
            })
        })
        .await
        .map_err(|_| WalletError::InvalidConfig(".wlt persistence task failed".to_string()))??;

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    async fn persist_session_profile(
        &self,
        _wallet_id: &PersistWalletId,
        _profile_bytes: Vec<u8>,
        _cache_state_payload: Option<(std::path::PathBuf, Vec<u8>)>,
    ) -> WalletResult<()> {
        Err(WalletError::InvalidConfig(
            ".wlt persistence is not supported on wasm32".to_string(),
        ))
    }
}

include!("wallet_session_derivation_recovery.rs");
