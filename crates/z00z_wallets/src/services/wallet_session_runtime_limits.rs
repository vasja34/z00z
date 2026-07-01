#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SessionCapKind {
    Touch,
    NoTouch,
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SessionCapState {
    Live,
    Unsupported,
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SessionCapRow {
    pub(crate) kind: SessionCapKind,
    pub(crate) native: SessionCapState,
    pub(crate) wasm: SessionCapState,
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) const TOUCH_CAP_ERR: &str = "verified touch session is not supported on wasm32";
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) const NO_TOUCH_CAP_ERR: &str = "verified no-touch session is not supported on wasm32";

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) const SESSION_CAP_MATRIX: [SessionCapRow; 2] = [
    SessionCapRow {
        kind: SessionCapKind::Touch,
        native: SessionCapState::Live,
        wasm: SessionCapState::Unsupported,
    },
    SessionCapRow {
        kind: SessionCapKind::NoTouch,
        native: SessionCapState::Live,
        wasm: SessionCapState::Unsupported,
    },
];

#[derive(Clone, Debug)]
pub(crate) struct VerifiedSession {
    wallet_id: PersistWalletId,
}

impl VerifiedSession {
    pub(crate) fn new(session: &SessionToken) -> Self {
        Self {
            wallet_id: session.wallet_id.clone(),
        }
    }

    pub(crate) fn wallet_id(&self) -> &PersistWalletId {
        &self.wallet_id
    }
}

#[derive(Clone, Debug)]
pub(crate) struct VerifiedSessionNoTouch {
    wallet_id: PersistWalletId,
    session: SessionToken,
}

impl VerifiedSessionNoTouch {
    pub(crate) fn new(session: &SessionToken) -> Self {
        Self {
            wallet_id: session.wallet_id.clone(),
            session: session.clone(),
        }
    }

    pub(crate) fn wallet_id(&self) -> &PersistWalletId {
        &self.wallet_id
    }

    pub(crate) fn session(&self) -> &SessionToken {
        &self.session
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn unsupported_cap_err(kind: SessionCapKind) -> WalletError {
    let message = match kind {
        SessionCapKind::Touch => TOUCH_CAP_ERR,
        SessionCapKind::NoTouch => NO_TOUCH_CAP_ERR,
    };
    WalletError::InvalidConfig(message.to_string())
}

impl WalletService {
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

    /// Phase 1 rate limit precheck for `wallet.key.rotate_master_key`.
    ///
    /// Enforces 1 request per hour per wallet.
    pub(crate) async fn rotate_master_key_precheck(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<RateLimitPrecheck> {
        const WINDOW_MS: u64 = 3_600_000;
        const MAX_REQUESTS: u32 = 1;

        let now_ms = self.require_now_ms()?;
        let mut limits = self.rotate_master_key_limits.write().await;

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

    pub(crate) async fn rollback_rotate_master_key_precheck(
        &self,
        wallet_id: &PersistWalletId,
    ) {
        let mut limits = self.rotate_master_key_limits.write().await;
        let Some(state) = limits.get_mut(wallet_id) else {
            return;
        };

        state.window_count = state.window_count.saturating_sub(1);
        if state.window_count == 0 {
            limits.remove(wallet_id);
        }
    }

    /// Verify that a session token is valid and refresh activity timestamps.
    ///
    /// Phase 1 contract: all sensitive operations must validate a session token and use the
    /// in-memory key-bearing session owned by the session manager.
    pub(crate) async fn verify_session(
        &self,
        session: &SessionToken,
    ) -> WalletResult<VerifiedSession> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let timeout_ms = self.timeout_ms();
            let wallet_id = session.wallet_id.clone();

            let _ = self.wallet_sessions.verify(session, 0, timeout_ms).await?;

            self.update_activity(&wallet_id).await?;
            Ok(VerifiedSession::new(session))
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = session;
            Err(unsupported_cap_err(SessionCapKind::Touch))
        }
    }

    pub(crate) async fn verify_session_no_touch(
        &self,
        session: &SessionToken,
    ) -> WalletResult<VerifiedSessionNoTouch> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.wallet_sessions.validate(session, 0).await?;
            Ok(VerifiedSessionNoTouch::new(session))
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = session;
            Err(unsupported_cap_err(SessionCapKind::NoTouch))
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
                    Err(err) => {
                        let _ = err;
                    }
                }
            }
        })
    }
}
