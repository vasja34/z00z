impl WalletService {

    /// Create new wallet service stub.
    pub fn new() -> Self {
        Self::with_output_dir(Self::default_output_dir())
    }

    #[cfg(test)]
    pub(crate) fn set_sleeper(&mut self, sleeper: Arc<dyn Sleeper>) {
        self.sleeper = sleeper;
    }

    #[cfg(test)]
    pub(crate) fn set_test_auto_lock_policy(&mut self, policy: AutoLockPolicy) {
        self.auto_lock_policy = policy;
    }

    /// Create wallet service with custom output directory
    pub fn with_output_dir(output_dir: PathBuf) -> Self {
        Self::apply_hardening();
        // Create directory if not exists
        if let Err(e) = create_dir_all(&output_dir) {
            z00z_utils::logger::Logger::warn(
                &z00z_utils::logger::TracingLogger,
                &format!("Failed to create wallets output directory: {e}"),
            );
        }

        let time_provider: Arc<dyn TimeProvider> = Arc::new(SystemTimeProvider);
        let wlt_store = Self::default_wallet_store(time_provider.clone());

        let service = Self {
            time_provider: time_provider.clone(),
            sleeper: Arc::new(TokioSleeper),
            entropy: Self::default_entropy(),
            wlt_store,
            auto_lock_policy: AutoLockPolicy::default(),
            wallet_states: Arc::new(RwLock::new(BTreeMap::new())),
            wallet_settings: Arc::new(RwLock::new(BTreeMap::new())),
            unlock_attempts: Arc::new(RwLock::new(BTreeMap::new())),
            show_seed_phrase_limits: Arc::new(RwLock::new(BTreeMap::new())),
            rotate_master_key_limits: Arc::new(RwLock::new(BTreeMap::new())),
            key_derive_limits: Arc::new(RwLock::new(BTreeMap::new())),
            backup_create_limits: Arc::new(RwLock::new(BTreeMap::new())),
            backup_settings: Arc::new(RwLock::new(BTreeMap::new())),
            receiver_labels: Arc::new(RwLock::new(BTreeMap::new())),
            key_audit_logs: Arc::new(RwLock::new(Vec::new())),
            wallet_receiver_derivers: Arc::new(RwLock::new(BTreeMap::new())),
            wallet_receiver_deriver_counters: Arc::new(RwLock::new(BTreeMap::new())),
            wallet_password_verifiers: Arc::new(RwLock::new(BTreeMap::new())),
            wallet_seed_salts: Arc::new(RwLock::new(BTreeMap::new())),
            #[cfg(not(target_arch = "wasm32"))]
            wallet_sessions: WalletSessionManager::new(time_provider.clone()),
            #[cfg(not(target_arch = "wasm32"))]
            wallet_identities: Arc::new(RwLock::new(BTreeMap::new())),
            output_dir,
            wallet_names: Arc::new(RwLock::new(BTreeMap::new())),
            wallet_claimed_assets: Arc::new(RwLock::new(BTreeMap::new())),
            last_receive_scan_outcomes: Arc::new(RwLock::new(BTreeMap::new())),
            reachability_wallet: ReachabilityWallet {
                kernel: crate::wallet::WalletKernel::new(
                    crate::wallet::WalletId([0u8; 32]),
                    crate::wallet::ChainId::DEVNET,
                ),
                secret_store: (),
                key_manager: (),
                receiver_manager: (),
                wallet_storage: (),
                asset_storage: (),
                tx_storage: (),
                receipt_storage: (),
                asset_selector: (),
                fee_estimator: (),
                tx_assembler: (),
                signer: (),
                prover: (),
                local_verifier: (),
                backup_exporter: (),
                backup_importer: (),
                policy: (),
                state: Arc::new(std::sync::RwLock::new(WalletState::Locked)),
            },
        };

        #[cfg(not(target_arch = "wasm32"))]
        service.cleanup_stale_locks();

        service
    }

    /// Create service with custom output directory and injected dependencies.
    pub fn with_output_dir_and_time(
        output_dir: PathBuf,
        time_provider: Arc<dyn TimeProvider>,
    ) -> Self {
        let service = Self::create_service_custom_output_directory(
            output_dir,
            time_provider.clone(),
            SystemRngProvider,
        );

        #[cfg(not(target_arch = "wasm32"))]
        service.cleanup_stale_locks();

        service
    }

    /// Create service with custom output directory and injected dependencies.
    pub fn create_service_custom_output_directory<P>(
        output_dir: PathBuf,
        time_provider: Arc<dyn TimeProvider>,
        rng_provider: P,
    ) -> Self
    where
        P: SecureRngProvider + Send + Sync + 'static,
    {
        Self::apply_hardening();
        if let Err(e) = create_dir_all(&output_dir) {
            z00z_utils::logger::Logger::warn(
                &z00z_utils::logger::TracingLogger,
                &format!("Failed to create wallets output directory: {e}"),
            );
        }

        let wlt_store = Self::default_wallet_store(time_provider.clone());

        Self {
            time_provider: time_provider.clone(),
            sleeper: Arc::new(TokioSleeper),
            entropy: Arc::new(WalletEntropyFromRngProvider::new(rng_provider)),
            wlt_store,
            auto_lock_policy: AutoLockPolicy::default(),
            wallet_states: Arc::new(RwLock::new(BTreeMap::new())),
            wallet_settings: Arc::new(RwLock::new(BTreeMap::new())),
            unlock_attempts: Arc::new(RwLock::new(BTreeMap::new())),
            show_seed_phrase_limits: Arc::new(RwLock::new(BTreeMap::new())),
            rotate_master_key_limits: Arc::new(RwLock::new(BTreeMap::new())),
            key_derive_limits: Arc::new(RwLock::new(BTreeMap::new())),
            backup_create_limits: Arc::new(RwLock::new(BTreeMap::new())),
            backup_settings: Arc::new(RwLock::new(BTreeMap::new())),
            receiver_labels: Arc::new(RwLock::new(BTreeMap::new())),
            key_audit_logs: Arc::new(RwLock::new(Vec::new())),
            wallet_receiver_derivers: Arc::new(RwLock::new(BTreeMap::new())),
            wallet_receiver_deriver_counters: Arc::new(RwLock::new(BTreeMap::new())),
            wallet_password_verifiers: Arc::new(RwLock::new(BTreeMap::new())),
            wallet_seed_salts: Arc::new(RwLock::new(BTreeMap::new())),
            #[cfg(not(target_arch = "wasm32"))]
            wallet_sessions: WalletSessionManager::new(time_provider.clone()),
            #[cfg(not(target_arch = "wasm32"))]
            wallet_identities: Arc::new(RwLock::new(BTreeMap::new())),
            output_dir,
            wallet_names: Arc::new(RwLock::new(BTreeMap::new())),
            wallet_claimed_assets: Arc::new(RwLock::new(BTreeMap::new())),
            last_receive_scan_outcomes: Arc::new(RwLock::new(BTreeMap::new())),
            reachability_wallet: ReachabilityWallet {
                kernel: crate::wallet::WalletKernel::new(
                    crate::wallet::WalletId([0u8; 32]),
                    crate::wallet::ChainId::DEVNET,
                ),
                secret_store: (),
                key_manager: (),
                receiver_manager: (),
                wallet_storage: (),
                asset_storage: (),
                tx_storage: (),
                receipt_storage: (),
                asset_selector: (),
                fee_estimator: (),
                tx_assembler: (),
                signer: (),
                prover: (),
                local_verifier: (),
                backup_exporter: (),
                backup_importer: (),
                policy: (),
                state: Arc::new(std::sync::RwLock::new(WalletState::Locked)),
            },
        }
    }

    pub(crate) async fn push_key_audit_entry(&self, entry: PersistAuditLogEntry) {
        let mut store = self.key_audit_logs.write().await;
        store.push(entry);
    }

    pub(crate) async fn key_derive_rate_limit_precheck(
        &self,
        wallet_id: &PersistWalletId,
        max_requests_per_minute: u32,
    ) -> Result<(), String> {
        let now_ms = self
            .time_provider
            .try_unix_timestamp_ms()
            .map_err(|e| format!("clock unavailable: {e}"))?;

        let mut data = self.key_derive_limits.write().await;
        let state = data
            .entry(wallet_id.clone())
            .or_insert_with(|| RateLimitWindowState::new(now_ms));

        const WINDOW_MS: u64 = 60_000;
        if now_ms.saturating_sub(state.window_start_ms) >= WINDOW_MS {
            state.window_start_ms = now_ms;
            state.window_count = 0;
        }

        if state.window_count >= max_requests_per_minute {
            let retry_after_ms =
                WINDOW_MS.saturating_sub(now_ms.saturating_sub(state.window_start_ms));
            let retry_after_seconds = (retry_after_ms / 1000).max(1);
            return Err(format!(
                "Rate limited: retry after {} seconds",
                retry_after_seconds
            ));
        }

        state.window_count = state.window_count.saturating_add(1);
        Ok(())
    }

    pub(crate) async fn get_receiver_labels(
        &self,
        wallet_id: &PersistWalletId,
    ) -> AddressLabelList {
        let store = self.receiver_labels.read().await;
        store.get(wallet_id).cloned().unwrap_or_default()
    }

    pub(crate) async fn upsert_receiver_label(
        &self,
        wallet_id: &PersistWalletId,
        receiver_id: String,
        label: String,
    ) {
        let mut store = self.receiver_labels.write().await;
        let labels = store.entry(wallet_id.clone()).or_insert_with(Vec::new);

        if let Some((_, existing)) = labels.iter_mut().find(|(item, _)| item == &receiver_id) {
            *existing = label;
        } else {
            labels.push((receiver_id, label));
        }
    }

    /// Create service with injected dependencies.

}
