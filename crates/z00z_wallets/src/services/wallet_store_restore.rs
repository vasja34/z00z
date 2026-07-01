impl WalletService {
    /// Load live wallet state from `.wlt` storage and restore it into memory.
    pub async fn load_wallet(
        &self,
        wallet_id: &PersistWalletId,
        password: &str,
    ) -> WalletResult<()> {
        let profile_bytes = self.load_wallet_profile_bytes(wallet_id, password).await?;
        let profile = Self::decode_wallet_profile_bytes(wallet_id, &profile_bytes)?;
        self.restore_profile(profile).await?;
        let claimed_assets = self
            .load_claimed_assets_with_password(wallet_id, password)
            .await?;
        self.install_claimed_assets(wallet_id, claimed_assets).await;
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn load_wallet_profile_bytes(
        &self,
        wallet_id: &PersistWalletId,
        password: &str,
    ) -> WalletResult<crate::security::SecretBytes> {
        let wlt_path = self.wlt_file_path(wallet_id);
        let wallet_id = wallet_id.clone();
        let identity = self.resolve_persisted_wallet_identity(&wallet_id).await?;
        let wallet_id_for_open = wallet_id.clone();
        let password = SafePassword::from(password);
        let wlt_store = Arc::clone(&self.wlt_store);

        tokio::task::spawn_blocking(move || {
            let session =
                wlt_store.open_wallet_store(&wlt_path, &wallet_id_for_open, &password, &identity)?;

            drop(password);

            wlt_store.read_wallet_profile(&session)
        })
        .await
        .map_err(|_| WalletError::InvalidConfig(".wlt load task failed".to_string()))?
    }

    #[cfg(target_arch = "wasm32")]
    async fn load_wallet_profile_bytes(
        &self,
        _wallet_id: &PersistWalletId,
        _password: &str,
    ) -> WalletResult<crate::security::SecretBytes> {
        Err(WalletError::InvalidConfig(
            ".wlt persistence is not supported on wasm32".to_string(),
        ))
    }

    fn decode_wallet_profile_bytes(
        wallet_id: &PersistWalletId,
        profile_bytes: &[u8],
    ) -> WalletResult<WalletProfilePayload> {
        use z00z_utils::codec::{BincodeCodec, Codec};

        let profile = BincodeCodec
            .deserialize::<WalletProfilePayload>(profile_bytes)
            .map_err(|_| WalletError::InvalidPassword)?;
        let profile = profile
            .migrate_to_current()
            .map_err(|_| WalletError::InvalidPassword)?;
        profile
            .verify_checksum()
            .map_err(|_| WalletError::InvalidPassword)?;
        if profile.wallet_id != *wallet_id {
            return Err(WalletError::InvalidPassword);
        }
        Ok(profile)
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn load_claimed_assets_with_password(
        &self,
        wallet_id: &PersistWalletId,
        password: &str,
    ) -> WalletResult<Vec<Asset>> {
        let wlt_path = self.wlt_file_path(wallet_id);
        let wallet_id = wallet_id.clone();
        let identity = self.resolve_persisted_wallet_identity(&wallet_id).await?;
        let password = SafePassword::from(password);
        let wlt_store = Arc::clone(&self.wlt_store);

        tokio::task::spawn_blocking(move || {
            let session = wlt_store.open_wallet_store(&wlt_path, &wallet_id, &password, &identity)?;
            crate::db::wallet_asset_store()
                .list_owned_assets(&session, crate::db::AssetFilter::default(), None, usize::MAX)
                .and_then(WalletService::live_claimed_assets_from_page)
        })
        .await
        .map_err(|_| WalletError::InvalidConfig(".wlt claimed-asset load task failed".to_string()))?
    }

    #[cfg(target_arch = "wasm32")]
    async fn load_claimed_assets_with_password(
        &self,
        _wallet_id: &PersistWalletId,
        _password: &str,
    ) -> WalletResult<Vec<Asset>> {
        Err(WalletError::InvalidConfig(
            ".wlt owned-asset loading is not supported on wasm32".to_string(),
        ))
    }

    pub(crate) async fn restore_profile(
        &self,
        profile: WalletProfilePayload,
    ) -> WalletResult<PersistWalletId> {
        let WalletProfilePayload {
            wallet_id,
            name,
            password_verifier,
            receiver_deriver,
            settings,
            seed_salt,
            state,
            ..
        } = profile;

        {
            let mut names = self.wallet_names.write().await;
            names.insert(wallet_id.clone(), name);
        }

        {
            let mut states = self.wallet_states.write().await;
            states.insert(wallet_id.clone(), state);
        }

        {
            let mut verifiers = self.wallet_password_verifiers.write().await;
            verifiers.insert(
                wallet_id.clone(),
                WalletPasswordVerifierState {
                    salt: password_verifier.salt,
                    verifier: password_verifier.verifier,
                },
            );
        }

        {
            let mut store = self.wallet_seed_salts.write().await;
            if let Some(seed_salt) = seed_salt {
                store.insert(wallet_id.clone(), seed_salt);
            } else {
                store.remove(&wallet_id);
            }
        }

        {
            let mut counters = self.wallet_receiver_deriver_counters.write().await;
            counters.insert(wallet_id.clone(), receiver_deriver);
        }

        {
            let mut settings_store = self.wallet_settings.write().await;
            settings_store.insert(wallet_id.clone(), settings);
        }

        Ok(wallet_id)
    }

    pub(crate) async fn install_claimed_assets(
        &self,
        wallet_id: &PersistWalletId,
        claimed_assets: Vec<Asset>,
    ) {
        let mut store = self.wallet_claimed_assets.write().await;
        if claimed_assets.is_empty() {
            store.remove(wallet_id);
        } else {
            store.insert(wallet_id.clone(), claimed_assets);
        }
    }

}
