impl WalletService {
    pub(crate) fn compute_unlock_delay_ms(failures: u32) -> u64 {
        // Task 1.4 spec: delay_ms = min(3000, 200 * 2^failures)
        // Cap triggers at failures >= 4 (200 * 16 = 3200).
        if failures >= 4 {
            return 3_000;
        }

        200u64.saturating_mul(1u64 << failures).min(3_000)
    }

    pub(crate) async fn current_unlock_failures(&self, wallet_id: &PersistWalletId) -> u32 {
        let attempts = self.unlock_attempts.read().await;
        attempts
            .get(wallet_id)
            .map(|s| s.failed_attempts)
            .unwrap_or(0)
    }

    /// Get wallet settings.
    ///
    /// If no settings are present, returns stub defaults with timestamps set.
    pub async fn get_wallet_settings(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<PersistWalletSettings> {
        let settings = self.wallet_settings.read().await;
        if let Some(existing) = settings.get(wallet_id) {
            return Ok(existing.clone());
        }

        let now = self.now_ms();
        let defaults = crate::services::wallet_runtime_config::resolve_wallet_settings_with_timestamps(now)?;
        Ok(defaults)
    }

    /// Set wallet settings (full replacement).
    ///
    /// Preserves `created_at` for existing wallets and updates `updated_at`.
    pub async fn set_wallet_settings(
        &self,
        wallet_id: PersistWalletId,
        mut settings: PersistWalletSettings,
    ) -> WalletResult<()> {
        let now = self.now_ms();
        let mut store = self.wallet_settings.write().await;

        if let Some(existing) = store.get_mut(&wallet_id) {
            settings.created_at = existing.created_at;
            settings.updated_at = now;
            *existing = settings;
        } else {
            settings.created_at = now;
            settings.updated_at = now;
            store.insert(wallet_id, settings);
        }

        Ok(())
    }

    /// Check for wallets that exceeded auto-lock timeout and lock them.
    ///
    /// This method should be called periodically (e.g., every 30 seconds)
    /// by a background monitor task.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example(service: &z00z_wallets::services::WalletService) {
    /// // Check and auto-lock inactive wallets
    /// service.check_auto_lock().await.unwrap();
    /// # }
    /// ```
    pub async fn check_auto_lock(&self) -> WalletResult<Vec<PersistWalletId>> {
        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();

        // Identify expired wallets without holding locks across await points.
        let expired_wallets = {
            let states = self.wallet_states.read().await;
            let mut expired = Vec::new();

            for (wallet_id, state) in states.iter() {
                let WalletState::Unlocked {
                    session_start_ms: _,
                    last_activity_ms,
                } = state
                else {
                    continue;
                };

                let last_activity_ms = *last_activity_ms;

                if now_ms < last_activity_ms {
                    continue;
                }

                let elapsed_ms = now_ms - last_activity_ms;
                if elapsed_ms >= timeout_ms {
                    expired.push(wallet_id.clone());
                }
            }

            expired
        };

        // Use the same teardown path as manual locking.
        let mut locked_wallets = Vec::new();
        for wallet_id in expired_wallets {
            self.lock_wallet(&wallet_id).await?;
            locked_wallets.push(wallet_id);
        }

        Ok(locked_wallets)
    }

    /// Update last activity timestamp for a wallet.
    ///
    /// Call this method whenever user performs an operation on the wallet
    /// to reset the auto-lock timeout.
    pub async fn update_activity(&self, wallet_id: &PersistWalletId) -> WalletResult<()> {
        let now_ms = self.require_now_ms()?;
        self.touch_last_activity(wallet_id, now_ms).await;
        Ok(())
    }

    pub(crate) async fn sync_expired_unlocked_wallets(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let unlocked_wallet_ids: Vec<PersistWalletId> = {
                let states = self.wallet_states.read().await;
                states
                    .iter()
                    .filter_map(|(wallet_id, state)| {
                        matches!(state, WalletState::Unlocked { .. }).then_some(wallet_id.clone())
                    })
                    .collect()
            };

            let mut stale_wallet_ids = Vec::new();
            for wallet_id in unlocked_wallet_ids {
                if !self
                    .wallet_sessions
                    .is_wallet_session_active(&wallet_id, 0)
                    .await
                {
                    stale_wallet_ids.push(wallet_id);
                }
            }

            for wallet_id in stale_wallet_ids {
                let _ = self.lock_wallet(&wallet_id).await;
            }
        }
    }

    pub(crate) async fn touch_last_activity(&self, wallet_id: &PersistWalletId, now_ms: u64) {
        {
            let mut states = self.wallet_states.write().await;
            if let Some(WalletState::Unlocked {
                session_start_ms: _,
                ref mut last_activity_ms,
            }) = states.get_mut(wallet_id)
            {
                *last_activity_ms = now_ms;
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let timeout_ms = self.timeout_ms();
            self.wallet_sessions
                .touch(wallet_id, now_ms, timeout_ms)
                .await;
        }
    }

    /// Register a wallet in the auto-lock tracker.
    ///
    /// Called when a wallet is unlocked.
    pub async fn register_unlocked_wallet(&self, wallet_id: PersistWalletId) -> WalletResult<()> {
        let now_ms = self.require_now_ms()?;
        self.track_unlocked_wallet(wallet_id, now_ms).await;
        Ok(())
    }

    pub(crate) async fn track_unlocked_wallet(&self, wallet_id: PersistWalletId, now_ms: u64) {
        let mut states = self.wallet_states.write().await;

        states.insert(
            wallet_id.clone(),
            WalletState::Unlocked {
                session_start_ms: now_ms,
                last_activity_ms: now_ms,
            },
        );
    }

    /// Lock a wallet and invalidate any in-memory session state.
    ///
    /// Phase 1 behavior: this revokes the "unlocked" state so that any
    /// session token returned by `wallet.unlock` becomes unusable for
    /// sensitive operations guarded by `verify_unlocked`.
    pub async fn lock_wallet(&self, wallet_id: &PersistWalletId) -> WalletResult<()> {
        {
            let mut states = self.wallet_states.write().await;
            states.insert(wallet_id.clone(), WalletState::Locked);
        }

        // Drop any in-memory address derivation state.
        {
            let mut derivers = self.wallet_receiver_derivers.write().await;
            derivers.remove(wallet_id);
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.wallet_sessions.drop_session(wallet_id).await;
            self.cleanup_lock_file_best_effort(wallet_id);
        }

        Ok(())
    }

    /// Phase 5.3: Forward lifecycle events into the wallet service.
    ///
    /// Mandate: on `Backgrounded` / `Suspended` / `ScreenLocked`, immediately lock all wallets
    /// (drop in-memory sessions holding key material).
    pub async fn on_lifecycle_event(&self, event: WalletLifecycleEvent) -> WalletResult<()> {
        match event {
            WalletLifecycleEvent::Foregrounded => Ok(()),
            WalletLifecycleEvent::Backgrounded
            | WalletLifecycleEvent::Suspended
            | WalletLifecycleEvent::ScreenLocked => self.lock_all_wallets().await,
        }
    }

    async fn lock_all_wallets(&self) -> WalletResult<()> {
        let wallet_ids: Vec<PersistWalletId> = {
            let states = self.wallet_states.read().await;
            states.keys().cloned().collect()
        };

        for wallet_id in wallet_ids {
            self.lock_wallet(&wallet_id).await?;
        }

        Ok(())
    }
}

include!("wallet_session_runtime_limits.rs");
