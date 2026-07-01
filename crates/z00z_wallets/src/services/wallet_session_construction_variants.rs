impl WalletService {
    /// Create service with injected dependencies.
    pub fn with_dependencies(time_provider: Arc<dyn TimeProvider>) -> Self {
        Self::with_dependencies_and_rng_provider(time_provider, SystemRngProvider)
    }

    /// Create service with injected dependencies.
    pub fn with_dependencies_and_rng_provider<P>(
        time_provider: Arc<dyn TimeProvider>,
        rng_provider: P,
    ) -> Self
    where
        P: SecureRngProvider + Send + Sync + 'static,
    {
        Self::apply_hardening();
        let output_dir = Self::default_output_dir();
        if let Err(err) = create_dir_all(&output_dir) {
            warn_output_dir_create(&err);
        }

        let wlt_store = Self::default_wallet_store(time_provider.clone());

        Self {
            time_provider: time_provider.clone(),
            sleeper: Arc::new(TokioSleeper),
            entropy: Arc::new(WalletEntropyFromRngProvider::new(rng_provider)),
            wlt_store,
            auto_lock_policy: Self::startup_auto_lock_policy(),
            receiver_cache_size: Self::startup_receiver_cache_size(),
            receiver_derive_rate_limit: Self::startup_receiver_derive_rate_limit(),
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

    /// Create wallet service with custom auto-lock policy.
    pub fn with_auto_lock_policy(policy: AutoLockPolicy) -> Self {
        Self::apply_hardening();
        let output_dir = Self::default_output_dir();
        if let Err(err) = create_dir_all(&output_dir) {
            warn_output_dir_create(&err);
        }

        let time_provider: Arc<dyn TimeProvider> = Arc::new(SystemTimeProvider);
        let wlt_store = Self::default_wallet_store(time_provider.clone());

        Self {
            time_provider: time_provider.clone(),
            sleeper: Arc::new(TokioSleeper),
            entropy: Self::default_entropy(),
            wlt_store,
            auto_lock_policy: policy,
            receiver_cache_size: Self::startup_receiver_cache_size(),
            receiver_derive_rate_limit: Self::startup_receiver_derive_rate_limit(),
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

    /// Create wallet service with custom auto-lock policy and injected dependencies.
    pub fn auto_lock_policy_dependencies(
        policy: AutoLockPolicy,
        time_provider: Arc<dyn TimeProvider>,
    ) -> Self {
        Self::apply_hardening();
        let output_dir = Self::default_output_dir();
        if let Err(err) = create_dir_all(&output_dir) {
            warn_output_dir_create(&err);
        }

        let wlt_store = Self::default_wallet_store(time_provider.clone());

        Self {
            time_provider: time_provider.clone(),
            sleeper: Arc::new(TokioSleeper),
            entropy: Self::default_entropy(),
            wlt_store,
            auto_lock_policy: policy,
            receiver_cache_size: Self::startup_receiver_cache_size(),
            receiver_derive_rate_limit: Self::startup_receiver_derive_rate_limit(),
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
}
