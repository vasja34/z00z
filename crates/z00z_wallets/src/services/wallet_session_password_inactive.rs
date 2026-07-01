impl WalletService {
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

    /// Create wallet service with custom auto-lock policy
    pub fn with_auto_lock_policy(policy: AutoLockPolicy) -> Self {
        Self::apply_hardening();
        let output_dir = Self::default_output_dir();
        if let Err(e) = create_dir_all(&output_dir) {
            z00z_utils::logger::Logger::warn(
                &z00z_utils::logger::TracingLogger,
                &format!("Failed to create wallets output directory: {e}"),
            );
        }

        let time_provider: Arc<dyn TimeProvider> = Arc::new(SystemTimeProvider);
        let wlt_store = Self::default_wallet_store(time_provider.clone());

        Self {
            time_provider: time_provider.clone(),
            sleeper: Arc::new(TokioSleeper),
            entropy: Self::default_entropy(),
            wlt_store,
            auto_lock_policy: policy,
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
            entropy: Self::default_entropy(),
            wlt_store,
            auto_lock_policy: policy,
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

    fn compute_password_verifier(
        password: &z00z_crypto::expert::encoding::SafePassword,
        salt: &[u8; 32],
    ) -> [u8; 32] {
        let password_bytes = password.reveal().as_slice();
        let hash = DomainHasher::<WalletPasswordVerifierDomain>::new_with_label("wallet_password")
            .chain(salt)
            .chain(password_bytes)
            .finalize();

        let mut out = [0u8; 32];
        out.copy_from_slice(&hash.as_ref()[..32]);
        out
    }

    /// Constant-time comparison for 32-byte arrays using `subtle` crate.
    ///
    /// Prevents timing attacks by ensuring comparison time is independent of input values.
    /// Uses `subtle::ConstantTimeEq` for cryptographic-grade constant-time guarantees.
    fn ct_cmp_32(a: &[u8; 32], b: &[u8; 32]) -> bool {
        bool::from(a.ct_eq(b))
    }

    /// Confirm (and remember) wallet password for sensitive operations (Phase 1).
    ///
    /// Behavior:
    /// - First call per wallet_id: stores a verifier derived from the provided password.
    /// - Subsequent calls: password must match the stored verifier.
    ///
    /// This is intentionally in-memory only and exists to support RPC validation
    /// without introducing persistence.
    pub async fn confirm_wallet_password(
        &self,
        wallet_id: &PersistWalletId,
        password: &z00z_crypto::expert::encoding::SafePassword,
    ) -> WalletResult<()> {
        let password_bytes = password.reveal().as_slice();
        if password_bytes.is_empty() {
            return Err(WalletError::InvalidPassword);
        }

        let mut store = self.wallet_password_verifiers.write().await;

        if let Some(state) = store.get_mut(wallet_id) {
            let expected = Self::compute_password_verifier(password, &state.salt);
            if !Self::ct_cmp_32(&expected, &state.verifier) {
                return Err(WalletError::InvalidPassword);
            }
            return Ok(());
        }

        let mut salt = [0u8; 32];
        self.entropy.fill_bytes(&mut salt);

        let verifier = Self::compute_password_verifier(password, &salt);
        store.insert(
            wallet_id.clone(),
            WalletPasswordVerifierState { salt, verifier },
        );

        Ok(())
    }

    pub(crate) async fn confirm_wallet_password_with_backoff(
        &self,
        wallet_id: &PersistWalletId,
        password: &z00z_crypto::expert::encoding::SafePassword,
    ) -> WalletResult<()> {
        match self.confirm_wallet_password(wallet_id, password).await {
            Ok(()) => {
                self.record_unlock_attempt_result(wallet_id, true).await;
                Ok(())
            }
            Err(WalletError::InvalidPassword) => {
                let failures = self.current_unlock_failures(wallet_id).await;
                let delay_ms = Self::compute_unlock_delay_ms(failures);
                self.sleeper
                    .sleep(std::time::Duration::from_millis(delay_ms))
                    .await;
                self.record_unlock_attempt_result(wallet_id, false).await;
                Err(WalletError::InvalidPassword)
            }
            Err(error) => Err(error),
        }
    }

    pub(crate) fn now_ms(&self) -> u64 {
        self.time_provider.compat_unix_timestamp_millis()
    }

    fn make_seed_salt(&self) -> [u8; 16] {
        let mut seed_salt = [0u8; 16];
        self.entropy.fill_bytes(&mut seed_salt);
        seed_salt
    }
}
