impl WalletService {
    pub async fn register_unlocked_wallet(&self, wallet_id: PersistWalletId) -> WalletResult<()> {
        let now_ms = self.require_now_ms()?;
        self.track_unlocked_wallet(wallet_id, now_ms).await;
        Ok(())
    }

    async fn track_unlocked_wallet(&self, wallet_id: PersistWalletId, now_ms: u64) {
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

    /// Phase 1 precheck for `wallet.unlock`.
    ///
    /// Enforces:
    /// - Rate limiting: 5 requests per minute per wallet
    pub(crate) async fn unlock_attempt_precheck(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<UnlockAttemptPrecheck> {
        const WINDOW_MS: u64 = 60_000;
        const MAX_REQUESTS: u32 = 5;

        let now_ms = self.require_now_ms()?;
        let mut attempts = self.unlock_attempts.write().await;

        let state = attempts
            .entry(wallet_id.clone())
            .or_insert_with(|| UnlockAttemptState::new(now_ms));

        if now_ms.saturating_sub(state.window_start_ms) >= WINDOW_MS {
            state.window_start_ms = now_ms;
            state.window_count = 0;
        }

        if state.window_count >= MAX_REQUESTS {
            let window_end_ms = state.window_start_ms.saturating_add(WINDOW_MS);
            let retry_ms = window_end_ms.saturating_sub(now_ms);
            let retry_after_seconds = retry_ms.div_ceil(1_000) as u32;
            return Ok(UnlockAttemptPrecheck::RateLimited {
                retry_after_seconds,
                current_count: state.window_count,
                max_requests: MAX_REQUESTS,
            });
        }

        state.window_count = state.window_count.saturating_add(1);
        Ok(UnlockAttemptPrecheck::Allowed)
    }

    /// Record Phase 1 `wallet.unlock` attempt outcome.
    ///
    /// - On success: resets failure counter.
    /// - On failure: increments failure counter.
    pub(crate) async fn record_unlock_attempt_result(
        &self,
        wallet_id: &PersistWalletId,
        success: bool,
    ) {
        let mut attempts = self.unlock_attempts.write().await;

        let state = attempts
            .entry(wallet_id.clone())
            .or_insert_with(|| UnlockAttemptState::new(self.now_ms()));

        if success {
            state.failed_attempts = 0;
            return;
        }

        state.failed_attempts = state.failed_attempts.saturating_add(1);
    }

    /// Phase 1 rate limit precheck for `wallet.show_seed_phrase`.
    ///
    /// Enforces 3 requests per minute per wallet.
    pub(crate) async fn show_seed_phrase_precheck(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<RateLimitPrecheck> {
        const WINDOW_MS: u64 = 60_000;
        const MAX_REQUESTS: u32 = 3;

        let now_ms = self.require_now_ms()?;
        let mut limits = self.show_seed_phrase_limits.write().await;

        let state = limits
            .entry(wallet_id.clone())
            .or_insert_with(|| RateLimitWindowState::new(now_ms));

        if now_ms.saturating_sub(state.window_start_ms) >= WINDOW_MS {
            state.window_start_ms = now_ms;
            state.window_count = 0;
        }

        if state.window_count >= MAX_REQUESTS {
            let window_end_ms = state.window_start_ms.saturating_add(WINDOW_MS);
            let retry_ms = window_end_ms.saturating_sub(now_ms);
            let retry_after_seconds = retry_ms.div_ceil(1_000) as u32;

            return Ok(RateLimitPrecheck::RateLimited {
                retry_after_seconds,
                current_count: state.window_count,
                max_requests: MAX_REQUESTS,
            });
        }

        Ok(RateLimitPrecheck::Allowed)
    }

    pub(crate) async fn record_show_seed_phrase_attempt(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<RateLimitPrecheck> {
        const WINDOW_MS: u64 = 60_000;
        const MAX_REQUESTS: u32 = 3;

        let now_ms = self.require_now_ms()?;
        let mut limits = self.show_seed_phrase_limits.write().await;

        let state = limits
            .entry(wallet_id.clone())
            .or_insert_with(|| RateLimitWindowState::new(now_ms));

        if now_ms.saturating_sub(state.window_start_ms) >= WINDOW_MS {
            state.window_start_ms = now_ms;
            state.window_count = 0;
        }

        if state.window_count >= MAX_REQUESTS {
            let window_end_ms = state.window_start_ms.saturating_add(WINDOW_MS);
            let retry_ms = window_end_ms.saturating_sub(now_ms);
            let retry_after_seconds = retry_ms.div_ceil(1_000) as u32;

            return Ok(RateLimitPrecheck::RateLimited {
                retry_after_seconds,
                current_count: state.window_count,
                max_requests: MAX_REQUESTS,
            });
        }

        state.window_count = state.window_count.saturating_add(1);
        Ok(RateLimitPrecheck::Allowed)
    }

    /// Verify that a session token is valid and refresh activity timestamps.
    ///
    /// Phase 1 contract: all sensitive operations must validate a session token and use the
    /// in-memory key-bearing session owned by the session manager.
    pub(crate) async fn verify_session(&self, session: &SessionToken) -> WalletResult<()> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let timeout_ms = self.timeout_ms();
            let wallet_id = session.wallet_id.clone();

            let _ = self.wallet_sessions.verify(session, 0, timeout_ms).await?;

            // Keep the state-based auto-lock tracker aligned with session activity.
            self.update_activity(&wallet_id).await?;
            Ok(())
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = session;
            Err(WalletError::InvalidConfig(
                "verified touch session is not supported on wasm32".to_string(),
            ))
        }
    }

    pub(crate) async fn verify_session_no_touch(&self, session: &SessionToken) -> WalletResult<()> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.wallet_sessions.validate(session, 0).await
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = session;
            Err(WalletError::InvalidConfig(
                "verified no-touch session is not supported on wasm32".to_string(),
            ))
        }
    }

    /// Get state of specific wallet.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example(service: &z00z_wallets::services::WalletService, wallet_id: &z00z_wallets::rpc::types::common::PersistWalletId) {
    /// let state = service.get_wallet_state(wallet_id).await.unwrap();
    /// assert!(state.is_locked() || state.is_unlocked());
    /// # }
    /// ```
    pub async fn get_wallet_state(&self, wallet_id: &PersistWalletId) -> WalletResult<WalletState> {
        self.sync_expired_unlocked_wallets().await;

        let states = self.wallet_states.read().await;
        states
            .iter()
            .find_map(|(id, state)| {
                if id == wallet_id {
                    Some(state.clone())
                } else {
                    None
                }
            })
            .ok_or(crate::WalletError::NotFound(0))
    }

    /// Unregister a wallet from auto-lock tracker.
    ///
    /// Called when wallet is manually locked or deleted.
    pub async fn unregister_wallet(&self, wallet_id: &PersistWalletId) -> WalletResult<()> {
        let mut states = self.wallet_states.write().await;
        states.remove(wallet_id);

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Ensure unregister never leaves a key-bearing session alive (and holding `.wlt.lock`).
            self.wallet_sessions.drop_session(wallet_id).await;
            self.cleanup_lock_file_best_effort(wallet_id);
        }

        Ok(())
    }

    /// Start background auto-lock monitor.
    ///
    /// Spawns a background task that periodically checks for auto-lock
    /// timeout and automatically locks expired wallets.
    ///
    /// Returns JoinHandle to control the monitor lifecycle.
    pub fn start_auto_lock_monitor(self: Arc<Self>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

            loop {
                interval.tick().await;

                match self.check_auto_lock().await {
                    Ok(locked_wallets) => {
                        let _ = locked_wallets;
                    }
                    Err(e) => {
                        let _ = e;
                        // Don't crash monitor on error
                    }
                }
            }
        })
    }
}
