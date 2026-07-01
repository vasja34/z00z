impl WalletService {
    /// List all wallets.
    pub async fn list_wallets_in_memory(&self) -> WalletResult<Vec<PersistWalletInfo>> {
        self.sync_expired_unlocked_wallets().await;

        let names = self.wallet_names.read().await;
        let settings = self.wallet_settings.read().await;
        let states = self.wallet_states.read().await;

        // Union of keys across the stores to avoid relying on any single map.
        let mut wallet_ids: BTreeSet<PersistWalletId> = BTreeSet::new();
        wallet_ids.extend(names.keys().cloned());
        wallet_ids.extend(settings.keys().cloned());
        wallet_ids.extend(states.keys().cloned());

        let mut wallets = Vec::with_capacity(wallet_ids.len());
        for wallet_id in wallet_ids {
            let name = names
                .get(&wallet_id)
                .cloned()
                .unwrap_or_else(|| "Unnamed Wallet".to_string());

            let created_at = settings.get(&wallet_id).map(|s| s.created_at).unwrap_or(0);

            let is_locked = states
                .get(&wallet_id)
                .map(|s| s.is_locked())
                .unwrap_or(true);

            wallets.push(PersistWalletInfo {
                id: wallet_id,
                name,
                created_at,
                is_locked,
            });
        }

        Ok(wallets)
    }

    #[cfg(test)]
    pub(crate) async fn create_wallet_in_memory(
        &self,
        name: &str,
        password: SafePassword,
        seed_phrase: &str,
    ) -> WalletResult<PersistWalletId> {
        let identity = crate::services::wallet_runtime_config::resolve_wallet_identity_checked()?;
        self.create_wallet_using_explicit_identity(name, password, seed_phrase, &identity)
            .await
    }

    /// Create a wallet using an explicit wallet identity (network + chain).
    ///
    /// This is intended for the app-level orchestrator so identity is computed once
    /// and threaded through to `.wlt` persistence.
    ///
    /// # Behavior
    /// - Persists the wallet immediately as a `.wlt` file.
    /// - Publishes the wallet into in-memory stores only after persistence succeeds.
    /// - Assumes password/seed phrase validation is performed by the orchestrator.
    pub(crate) async fn create_wallet_using_explicit_identity(
        &self,
        name: &str,
        password: SafePassword,
        seed_phrase: &str,
        identity: &WalletIdentity,
    ) -> WalletResult<PersistWalletId> {
        // Generate unique wallet ID using domain-separated hashing.
        let now_ms = self.now_ms();
        let wallet_id = self.generate_wallet_id(name, now_ms);

        // Stage all state first. We only publish it to in-memory maps after the `.wlt`
        // persistence step succeeds.
        let mut verifier_salt = [0u8; 32];
        self.entropy.fill_bytes(&mut verifier_salt);
        let verifier = Self::compute_password_verifier(&password, &verifier_salt);
        let seed_salt = self.make_seed_salt();

        // Store wallet settings (staged)
        let now_ms = self.now_ms();
        let settings =
            crate::services::wallet_runtime_config::resolve_wallet_settings_with_timestamps(now_ms)?;

        // Orchestrator-owned policy: seed phrase is provided by the caller.
        // WalletService executes validated commands and does not generate seeds.
        let seed_phrase_to_persist = seed_phrase;

        // Persist `.wlt` first (no in-memory publish until success).
        let now_ms = self.now_ms();
        let profile = WalletProfilePayload::new_with_checksum(
            wallet_id.clone(),
            name.to_string(),
            settings.created_at,
            now_ms,
            PasswordVerifierState {
                salt: verifier_salt,
                verifier,
            },
            ReceiverDeriverState {
                next_payment_index: 0,
                next_change_index: 0,
            },
            settings.clone(),
            seed_salt,
            WalletState::Locked,
        );

        use z00z_utils::codec::{BincodeCodec, Codec};
        let codec = BincodeCodec;
        let profile_bytes = codec
            .serialize(&profile)
            .map_err(|e| WalletError::InvalidConfig(format!("Binary serialization failed: {e}")))?;

        self.persist_wallet_profile_state(
            wallet_id.clone(),
            password,
            seed_phrase_to_persist,
            profile_bytes,
            identity,
        )
        .await?;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut identities = self.wallet_identities.write().await;
            identities.insert(wallet_id.clone(), identity.clone());
        }

        // Publish in-memory state.
        {
            let mut store = self.wallet_password_verifiers.write().await;
            if store.contains_key(&wallet_id) {
                return Err(WalletError::WalletAlreadyExists);
            }
            store.insert(
                wallet_id.clone(),
                WalletPasswordVerifierState {
                    salt: verifier_salt,
                    verifier,
                },
            );
        }

        {
            let mut store = self.wallet_seed_salts.write().await;
            store.insert(wallet_id.clone(), seed_salt);
        }

        // Address deriver state is created lazily on first use after unlock.

        {
            let mut counters = self.wallet_receiver_deriver_counters.write().await;
            counters.insert(
                wallet_id.clone(),
                ReceiverDeriverState {
                    next_payment_index: 0,
                    next_change_index: 0,
                },
            );
        }

        {
            let mut states = self.wallet_states.write().await;
            states.insert(wallet_id.clone(), WalletState::Locked);
        }

        {
            let mut settings_store = self.wallet_settings.write().await;
            settings_store.insert(wallet_id.clone(), settings);
        }

        {
            let mut names = self.wallet_names.write().await;
            names.insert(wallet_id.clone(), name.to_string());
        }

        Ok(wallet_id)
    }

    pub(crate) async fn persist_wallet_profile_state(
        &self,
        wallet_id: PersistWalletId,
        password: SafePassword,
        seed_phrase: &str,
        profile_bytes: Vec<u8>,
        identity: &WalletIdentity,
    ) -> WalletResult<()> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let wlt_path = self.wlt_file_path(&wallet_id);
            let wlt_store = Arc::clone(&self.wlt_store);
            let seed_phrase = seed_phrase.to_string();
            let identity = identity.clone();

            tokio::task::spawn_blocking(move || {
                if !wlt_path.exists() {
                    wlt_store.create_wallet_store(
                        &wlt_path,
                        &wallet_id,
                        &password,
                        &seed_phrase,
                        &identity,
                    )?;
                }

                let session =
                    wlt_store.open_wallet_store(&wlt_path, &wallet_id, &password, &identity)?;
                drop(password);

                wlt_store
                    .write_wallet_profile(&session, profile_bytes)
                    .map(|_| ())
            })
            .await
            .map_err(|_| {
                WalletError::InvalidConfig(".wlt profile persistence task failed".to_string())
            })??;

            Ok(())
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = (wallet_id, password, seed_phrase, profile_bytes, identity);
            Err(WalletError::InvalidConfig(
                ".wlt persistence is not supported on wasm32".to_string(),
            ))
        }
    }
}

include!("wallet_store_open_source.rs");
