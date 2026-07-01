impl WalletService {
    fn discovered_wallet_identity(discovery: &PersistWalletDiscovery) -> WalletIdentity {
        WalletIdentity {
            network: discovery.network.clone(),
            chain: discovery.chain.clone(),
        }
    }

    /// Phase 1.2: Open an existing wallet file source and return discovery metadata.
    ///
    /// Native behavior: discovers `wallet.id/network/chain` from the provided `.wlt` and
    /// imports the file into the managed wallet output directory so that subsequent
    /// `wallet.session.unlock_wallet` (Task 1.1) can open it by wallet id.
    ///
    /// `WalletSource::Path` also imports the sibling JSONL tx-history sidecar when it is
    /// present. `WalletSource::Bytes` imports only the `.wlt` bytes because it has no sibling
    /// path context.
    pub async fn open_wallet_source(
        &self,
        source: WalletSource,
    ) -> WalletResult<PersistWalletDiscovery> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            match source {
                WalletSource::Path { path } => {
                    let source_path = PathBuf::from(path);
                    let source_path_for_discovery = source_path.clone();
                    let wlt_store = Arc::clone(&self.wlt_store);

                    let discovery = tokio::task::spawn_blocking(move || {
                        wlt_store.discover_wallet_store(source_path_for_discovery.as_path())
                    })
                    .await
                    .map_err(|_| {
                        WalletError::InvalidConfig("wallet discovery task failed".to_string())
                    })??;
                    let discovery_identity = Self::discovered_wallet_identity(&discovery);

                    let dest_path = self.wlt_file_path(&discovery.wallet_id);
                    let history_name =
                        Self::wallet_history_jsonl_name(&Self::wallet_stem(&discovery.wallet_id));
                    let source_history_path = source_path.with_file_name(history_name);
                    let dest_history_path = self.wallet_history_jsonl_path(&discovery.wallet_id);
                    let source_history_bytes = if source_history_path.exists() {
                        let source_history_path = source_history_path.clone();
                        let wallet_id = discovery.wallet_id.clone();
                        Some(
                            tokio::task::spawn_blocking(move || {
                                let history_bytes =
                                    crate::db::wallet_io::read_file(source_history_path.as_path())?;
                                Self::validate_tx_history_bytes(&wallet_id, &history_bytes)
                                    .map_err(|e| {
                                        WalletError::InvalidConfig(format!(
                                            "wallet source tx-history import failed: {e}"
                                        ))
                                    })?;
                                Ok::<Vec<u8>, WalletError>(history_bytes)
                            })
                            .await
                            .map_err(|_| {
                                WalletError::InvalidConfig(
                                    "wallet history validation task failed".to_string(),
                                )
                            })??,
                        )
                    } else {
                        None
                    };
                    let imported_wlt = if dest_path.exists() {
                        let persisted_identity = self
                            .resolve_persisted_wallet_identity(&discovery.wallet_id)
                            .await?;
                        crate::services::wallet_runtime_config::wallet_identity_chain_matches(
                            &persisted_identity,
                            &discovery_identity,
                        )?;
                        false
                    } else {
                        let source_path = source_path.clone();
                        let dest_path = dest_path.clone();
                        tokio::task::spawn_blocking(move || {
                            let bytes = crate::db::wallet_io::read_file(source_path.as_path())?;
                            crate::db::wallet_io::atomic_write_file_private(
                                dest_path.as_path(),
                                &bytes,
                            )?;
                            Ok::<(), WalletError>(())
                        })
                        .await
                        .map_err(|_| {
                            WalletError::InvalidConfig("wallet import task failed".to_string())
                        })??;
                        true
                    };

                    if let Some(history_bytes) = source_history_bytes.as_ref() {
                        if source_history_path != dest_history_path {
                            if let Err(err) = self.write_tx_history_jsonl_bytes(
                                &discovery.wallet_id,
                                history_bytes,
                            ) {
                                if imported_wlt {
                                    crate::db::wallet_io::remove_file_best_effort(dest_path.as_path());
                                    crate::db::wallet_io::remove_file_best_effort(
                                        dest_history_path.as_path(),
                                    );
                                }
                                return Err(err);
                            }
                        }
                    }

                    {
                        let mut states = self.wallet_states.write().await;
                        states
                            .entry(discovery.wallet_id.clone())
                            .or_insert(WalletState::Locked);
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let mut identities = self.wallet_identities.write().await;
                        identities.insert(discovery.wallet_id.clone(), discovery_identity);
                    }

                    {
                        let now = self.now_ms();
                        let mut settings = self.wallet_settings.write().await;
                        settings
                            .entry(discovery.wallet_id.clone())
                            .or_insert(
                                crate::services::wallet_runtime_config::resolve_wallet_settings_with_timestamps(
                                    now,
                                )?,
                            );
                    }

                    Ok(discovery)
                }
                WalletSource::Bytes { bytes } => {
                    if bytes.is_empty() {
                        return Err(WalletError::InvalidParams(
                            "wallet source bytes are required".to_string(),
                        ));
                    }

                    let mut tmp_id = [0u8; 16];
                    self.entropy.fill_bytes(&mut tmp_id);
                    let tmp_name = format!("wallet_source_{}.wlt", hex::encode(tmp_id));
                    let tmp_path = self.output_dir.join(tmp_name);

                    let wlt_store = Arc::clone(&self.wlt_store);
                    let tmp_path_for_task = tmp_path.clone();

                    let discovery = tokio::task::spawn_blocking(move || {
                        crate::db::wallet_io::atomic_write_file_private(
                            tmp_path_for_task.as_path(),
                            bytes.as_slice(),
                        )?;
                        wlt_store.discover_wallet_store(tmp_path_for_task.as_path())
                    })
                    .await
                    .map_err(|_| {
                        WalletError::InvalidConfig("wallet bytes discovery task failed".to_string())
                    })?;

                    let cleanup_tmp = || {
                        crate::db::wallet_io::remove_file_best_effort(tmp_path.as_path());
                    };

                    let discovery = match discovery {
                        Ok(result) => result,
                        Err(err) => {
                            cleanup_tmp();
                            return Err(err);
                        }
                    };
                    let discovery_identity = Self::discovered_wallet_identity(&discovery);

                    let dest_path = self.wlt_file_path(&discovery.wallet_id);
                    if dest_path.exists() {
                        let persisted_identity = self
                            .resolve_persisted_wallet_identity(&discovery.wallet_id)
                            .await?;
                        crate::services::wallet_runtime_config::wallet_identity_chain_matches(
                            &persisted_identity,
                            &discovery_identity,
                        )?;
                    } else {
                        let tmp_path = tmp_path.clone();
                        let dest_path = dest_path.clone();
                        tokio::task::spawn_blocking(move || {
                            let bytes = crate::db::wallet_io::read_file(tmp_path.as_path())?;
                            crate::db::wallet_io::atomic_write_file_private(
                                dest_path.as_path(),
                                &bytes,
                            )
                        })
                        .await
                        .map_err(|_| {
                            WalletError::InvalidConfig(
                                "wallet bytes import task failed".to_string(),
                            )
                        })??;
                    }

                    cleanup_tmp();

                    {
                        let mut states = self.wallet_states.write().await;
                        states
                            .entry(discovery.wallet_id.clone())
                            .or_insert(WalletState::Locked);
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let mut identities = self.wallet_identities.write().await;
                        identities.insert(discovery.wallet_id.clone(), discovery_identity);
                    }

                    {
                        let now = self.now_ms();
                        let mut settings = self.wallet_settings.write().await;
                        settings
                            .entry(discovery.wallet_id.clone())
                            .or_insert(
                                crate::services::wallet_runtime_config::resolve_wallet_settings_with_timestamps(
                                    now,
                                )?,
                            );
                    }

                    Ok(discovery)
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = source;
            Err(WalletError::InvalidConfig(
                "app.wallet.open_wallet_source is not supported on wasm32 yet".to_string(),
            ))
        }
    }

    /// Unlock wallet (Phase 1 in-memory implementation).
    ///
    /// Unlocks a wallet by verifying the password and creating a session token.
    pub async fn unlock_wallet_in_memory(
        &self,
        wallet_id: &PersistWalletId,
        password: &SafePassword,
    ) -> WalletResult<crate::rpc::types::wallet::SessionToken> {
        self.open_wallet_session(wallet_id, password, true).await
    }

    /// Ensure a live wallet session for internal runtime orchestration without
    /// consuming the user-facing unlock rate-limit window.
    pub async fn ensure_wallet_session(
        &self,
        wallet_id: &PersistWalletId,
        password: &SafePassword,
    ) -> WalletResult<crate::rpc::types::wallet::SessionToken> {
        self.open_wallet_session(wallet_id, password, false).await
    }

    async fn open_wallet_session(
        &self,
        wallet_id: &PersistWalletId,
        password: &SafePassword,
        count_unlock_attempt: bool,
    ) -> WalletResult<crate::rpc::types::wallet::SessionToken> {
        if count_unlock_attempt {
            let precheck = self.unlock_attempt_precheck(wallet_id).await?;
            match precheck {
                UnlockAttemptPrecheck::Allowed => {}
                UnlockAttemptPrecheck::RateLimited {
                    retry_after_seconds,
                    current_count,
                    max_requests,
                } => {
                    let _ = (current_count, max_requests);
                    return Err(WalletError::RateLimited {
                        retry_after_seconds,
                    });
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let now_ms = self.require_now_ms()?;
            let timeout_ms = self.timeout_ms();
            let has_password_verifier = {
                let verifiers = self.wallet_password_verifiers.read().await;
                verifiers.contains_key(wallet_id)
            };

            if has_password_verifier {
                match self.confirm_wallet_password(wallet_id, password).await {
                    Ok(()) => {}
                    Err(WalletError::InvalidPassword) => {
                        let failures = self.current_unlock_failures(wallet_id).await;
                        let delay_ms = Self::compute_unlock_delay_ms(failures);
                        self.sleeper.sleep(Duration::from_millis(delay_ms)).await;
                        if count_unlock_attempt {
                            self.record_unlock_attempt_result(wallet_id, false).await;
                        }
                        return Err(WalletError::InvalidPassword);
                    }
                    Err(error) => return Err(error),
                }
            }

            let existing_token = self
                .wallet_sessions
                .existing_token(wallet_id, now_ms, timeout_ms)
                .await;

            if let Some(token) = existing_token {
                if count_unlock_attempt {
                    self.record_unlock_attempt_result(wallet_id, true).await;
                }
                self.touch_last_activity(wallet_id, token.last_activity_at)
                    .await;
                return Ok(token);
            }

            let wlt_path = self.wlt_file_path(wallet_id);
            let wlt_store = Arc::clone(&self.wlt_store);
            let wallet_id_cloned = wallet_id.clone();
            let identity = self.resolve_persisted_wallet_identity(wallet_id).await?;
            let unlock_password = password.clone();

            let opened_session_res = tokio::task::spawn_blocking(move || {
                wlt_store.open_wallet_store(&wlt_path, &wallet_id_cloned, &unlock_password, &identity)
            })
            .await
            .map_err(|_| WalletError::InvalidConfig("unlock task failed".to_string()))?;

            let opened_session = match opened_session_res {
                Ok(session) => session,
                Err(WalletError::InvalidPassword) => {
                    let failures = self.current_unlock_failures(wallet_id).await;
                    let delay_ms = Self::compute_unlock_delay_ms(failures);
                    self.sleeper.sleep(Duration::from_millis(delay_ms)).await;
                    if count_unlock_attempt {
                        self.record_unlock_attempt_result(wallet_id, false).await;
                    }
                    return Err(WalletError::InvalidPassword);
                }
                Err(error) => return Err(error),
            };

            let has_loaded_wallet = {
                let verifiers = self.wallet_password_verifiers.read().await;
                verifiers.contains_key(wallet_id)
            };

            if !has_loaded_wallet {
                let profile_bytes = self.wlt_store.read_wallet_profile(&opened_session)?;
                let profile =
                    Self::decode_wallet_profile_bytes(wallet_id, profile_bytes.as_ref())?;
                self.restore_profile(profile).await?;
                let claimed_assets = self.load_claimed_assets_session(&opened_session)?;
                self.install_claimed_assets(wallet_id, claimed_assets).await;
            }

            self.confirm_wallet_password(wallet_id, password).await?;
            let now_ms = self.require_now_ms()?;

            if count_unlock_attempt {
                self.record_unlock_attempt_result(wallet_id, true).await;
            }
            self.track_unlocked_wallet(wallet_id.clone(), now_ms)
                .await;

            let mut token_bytes = [0u8; 32];
            self.entropy.fill_bytes(&mut token_bytes);
            let token_hex = hex::encode(token_bytes);

            let token = self
                .wallet_sessions
                .insert(wallet_id, token_hex, now_ms, timeout_ms, opened_session)
                .await;
            Ok(token)
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = (wallet_id, password, count_unlock_attempt);
            Err(WalletError::InvalidConfig(
                "wallet unlock is not supported on wasm32".to_string(),
            ))
        }
    }
}
