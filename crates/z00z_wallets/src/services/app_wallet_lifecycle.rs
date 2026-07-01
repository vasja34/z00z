impl AppService {
    // ------------------------------------------------------------------------
    // Wallet lifecycle (owned by app.*)
    // ------------------------------------------------------------------------

    /// List all wallets owned by the app.
    pub async fn list_wallets(&self) -> WalletResult<Vec<PersistWalletInfo>> {
        let _ = self.core_app.list_wallets();
        self.wallets.list_wallets_in_memory().await
    }

    /// Open an existing wallet file source and return discovery metadata.
    ///
    /// Phase 1: returns `{wallet_id, network, chain}` without unlocking.
    pub async fn open_wallet_source(
        &self,
        source: WalletSource,
    ) -> WalletResult<PersistWalletDiscovery> {
        let _ = self.core_app.list_wallets();
        self.wallets.open_wallet_source(source).await
    }

    /// Create a wallet and return a response payload including a 24-word seed phrase.
    ///
    /// Orchestration:
    /// - core layer validates and canonicalizes inputs into a `CreateWalletRequest`
    /// - service layer performs password policy validation, seed phrase handling, and persistence
    pub async fn create_wallet(
        &self,
        name: String,
        password: String,
        recovery_phrase: Option<String>,
    ) -> WalletResult<RuntimeCreateWalletResponse> {
        let identity = crate::services::wallet_runtime_config::resolve_wallet_identity_checked()?;
        let request = self
            .core_app
            .create_wallet(&name, &identity.network, &identity.chain)?;

        let validator = PasswordValidator::new(
            crate::services::wallet_runtime_config::resolve_wallet_password_policy()?,
        );
        validator
            .validate(&password)
            .map_err(|msg| WalletError::InvalidParams(msg.to_string()))?;
        let password_strength_score = validator.strength_score(&password);

        let seed_phrase = match recovery_phrase {
            Some(phrase) => {
                Self::validate_seed_phrase_24_english(&phrase)?;
                phrase
            }
            None => self.generate_seed_phrase_24()?,
        };

        let safe_password = SafePassword::from(password);

        let wallet_id = self
            .wallets
            .create_wallet_using_explicit_identity(
                &request.name,
                safe_password,
                &seed_phrase,
                &identity,
            )
            .await?;

        Ok(RuntimeCreateWalletResponse {
            wallet_id,
            name: request.name,
            seed_phrase,
            password_strength_score,
            created_at: self.core_app.time_provider.compat_unix_timestamp_millis(),
        })
    }

    /// Recover a wallet from a 24-word English seed phrase.
    ///
    /// Task 2.4 mandated behavior:
    /// - Creates a new `.wlt` container and imports the seed.
    /// - Persists the provided network/chain identity into wallet meta.
    /// - Runs the recovery scan (gap-limit reconciliation) before returning.
    pub async fn recover_from_seed(
        &self,
        name: String,
        password: String,
        mnemonic_a: String,
        mnemonic_b: String,
        network: String,
        chain: String,
    ) -> WalletResult<RuntimeRecoverFromSeedResponse> {
        let request = self
            .core_app
            .create_wallet(&name, &network, &chain)
            .map_err(|e| WalletError::InvalidParams(e.to_string()))?;

        let validator = PasswordValidator::new(
            crate::services::wallet_runtime_config::resolve_wallet_password_policy()?,
        );
        validator
            .validate(&password)
            .map_err(|msg| WalletError::InvalidParams(msg.to_string()))?;
        let password_strength_score = validator.strength_score(&password);

        let mnemonic_a = Self::normalize_mnemonic(&mnemonic_a);
        let mnemonic_b = Self::normalize_mnemonic(&mnemonic_b);

        if mnemonic_a != mnemonic_b {
            return Err(WalletError::InvalidParams(
                "seed phrases do not match".to_string(),
            ));
        }

        Self::validate_seed_phrase_24_english(&mnemonic_a)?;

        let network = network.trim().to_string();
        if network.is_empty() {
            return Err(WalletError::InvalidParams(
                "network is required".to_string(),
            ));
        }

        let chain = chain.trim().to_string();
        if chain.is_empty() {
            return Err(WalletError::InvalidParams("chain is required".to_string()));
        }

        let identity = WalletIdentity { network, chain };
        let safe_password = SafePassword::from(password);

        let wallet_id = self
            .wallets
            .create_wallet_using_explicit_identity(
                &request.name,
                safe_password.clone(),
                &mnemonic_a,
                &identity,
            )
            .await?;

        let _ = self
            .wallets
            .unlock_wallet_in_memory(&wallet_id, &safe_password)
            .await?;

        let chain_service = Arc::clone(&self.chain_service);
        let is_used = Arc::new(move |path, _public_key| {
            let chain_service = Arc::clone(&chain_service);
            let fut = async move { Ok(chain_service.is_path_used(path).await) };
            Box::pin(fut)
                as std::pin::Pin<Box<dyn std::future::Future<Output = WalletResult<bool>> + Send>>
        });

        let _ = self
            .wallets
            .reconcile_persist_gap_limit(
                &wallet_id,
                crate::services::wallet_runtime_config::resolve_wallet_recovery_gap_limit()?,
                is_used,
            )
            .await?;

        self.wallets.lock_wallet(&wallet_id).await?;

        Ok(RuntimeRecoverFromSeedResponse {
            wallet_id,
            name: request.name,
            network: identity.network,
            chain: identity.chain,
            password_strength_score,
            recovered_at: self.core_app.time_provider.compat_unix_timestamp_millis(),
        })
    }

    /// Delete a wallet.
    pub async fn delete_wallet(
        &self,
        id: PersistWalletId,
        password: String,
    ) -> WalletResult<RuntimeDeleteWalletResponse> {
        let _ = self.core_app.delete_wallet();

        let safe_password = SafePassword::from(password);
        Self::verify_password_confirmation(&safe_password)?;

        self.wallets.delete_wallet_data(&id, &safe_password).await?;
        Ok(RuntimeDeleteWalletResponse {
            status: RuntimeOperationStatus {
                success: true,
                message: String::new(),
            },
            wallet_id: id,
            deleted: true,
        })
    }

    /// Export a wallet (encrypted payload).
    pub async fn export_wallet(
        &self,
        id: PersistWalletId,
        password: String,
    ) -> WalletResult<RuntimeExportWalletResponse> {
        let _ = self.core_app.export_wallet();

        self.verify_unlocked(&id).await?;
        self.update_activity(&id).await;

        let safe_password = SafePassword::from(password);
        let encrypted_payload = self
            .wallets
            .export_wallet_payload(&id, &safe_password)
            .await?;

        Ok(RuntimeExportWalletResponse {
            success: true,
            export_path: None,
            encrypted_payload: Some(encrypted_payload),
        })
    }

    /// Import a wallet from a backup payload.
    pub async fn import_wallet(
        &self,
        data: String,
        password: String,
        name: String,
    ) -> WalletResult<RuntimeImportWalletResponse> {
        let _ = self.core_app.import_wallet();

        let safe_password = SafePassword::from(password);
        let wallet_id = self
            .wallets
            .import_wallet_payload(&data, &safe_password, &name)
            .await?;

        Ok(RuntimeImportWalletResponse {
            status: RuntimeOperationStatus {
                success: true,
                message: String::new(),
            },
            wallet_id,
            name,
        })
    }

}
