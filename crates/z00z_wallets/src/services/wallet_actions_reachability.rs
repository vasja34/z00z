impl WalletService {
    /// Access the structural audit wallet facade used by explicit guard paths.
    pub fn reachability(&self) -> WalletServiceReachability<'_> {
        WalletServiceReachability {
            reachability_wallet: &self.reachability_wallet,
        }
    }

    /// Deterministic service-guard hook for `wallet.storage.compact_storage`.
    ///
    /// Adapter-level `StorageRpcImpl` owns the current file-layout work; this
    /// seam keeps the `RPC -> WalletService` boundary explicit.
    pub fn compact_storage(&self, params: &RuntimeCompactStorageParams) -> bool {
        let _ = params;
        false
    }

    /// Deterministic service-guard hook for `wallet.storage.get_storage_stats`.
    ///
    /// Adapter-level `StorageRpcImpl` owns the current file-layout work; this
    /// seam keeps the `RPC -> WalletService` boundary explicit.
    pub fn get_storage_stats(&self, params: &RuntimeGetStorageStatsParams) -> bool {
        let _ = params;
        false
    }

    /// Deterministic service-guard hook for `wallet.storage.export_storage`.
    ///
    /// Adapter-level `StorageRpcImpl` owns the current file-layout work; this
    /// seam keeps the `RPC -> WalletService` boundary explicit.
    pub fn export_storage(&self, params: &RuntimeExportStorageParams) -> bool {
        let _ = params;
        false
    }

    /// Privileged seed-phrase export gated by session, password, rate limits,
    /// and encrypted response wrapping.
    pub async fn show_seed_phrase(
        &self,
        session: &SessionToken,
        password: SafePassword,
        confirmation: String,
    ) -> WalletResult<RuntimeShowSeedPhraseResponse> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let wlt_store = Arc::clone(&self.wlt_store);
            let now_ms = self.require_now_ms()?;
            let wallet_id = session.wallet_id.clone();

            let session_handle = self
                .wallet_sessions
                .get_session_handle_without_touch(session, now_ms)
                .await?;

            match self.record_show_seed_phrase_attempt(&wallet_id).await? {
                RateLimitPrecheck::Allowed => {}
                RateLimitPrecheck::RateLimited {
                    retry_after_seconds,
                    current_count: _,
                    max_requests: _,
                } => {
                    return Err(WalletError::RateLimited {
                        retry_after_seconds,
                    });
                }
            }

            let confirmation = confirmation.trim();
            if !confirmation.eq_ignore_ascii_case("I understand") {
                return Err(WalletError::InvalidParams(
                    "Confirmation phrase must be: I understand".to_string(),
                ));
            }

            let verify_password = password.clone();

            let seed_phrase = tokio::task::spawn_blocking(move || {
                session_handle.with_wallet_session(|wlt_session| {
                    wlt_store.verify_password(wlt_session, &verify_password)?;
                    wlt_store.reveal_seed_phrase(wlt_session)
                })
            })
            .await
            .map_err(|_| {
                WalletError::InvalidConfig("show_seed_phrase task failed".to_string())
            })??;

            self.update_activity(&wallet_id).await?;

            let salt = self.runtime_seed_salt(&wallet_id).await?;

            let aad = z00z_crypto::aead::build_aad_multipart(
                "wallet.seed_phrase_response",
                &[wallet_id.0.as_bytes()],
            )
            .map_err(|e| {
                WalletError::InvalidConfig(format!("seed phrase encryption failed: {e}"))
            })?;

            use crate::security::encryption::WalletEncryption;
            let mut key = WalletEncryption::derive_key(&password, &salt).map_err(|e| {
                WalletError::InvalidConfig(format!("seed phrase encryption failed: {e}"))
            })?;

            let envelope =
                z00z_crypto::aead::seal(&key, &aad, seed_phrase.as_bytes()).map_err(|e| {
                    WalletError::InvalidConfig(format!("seed phrase encryption failed: {e}"))
                })?;

            let nonce: [u8; z00z_crypto::aead::XCHACHA_NONCE_SIZE] = envelope
                [1..z00z_crypto::aead::ENVELOPE_HEADER_SIZE]
                .try_into()
                .map_err(|_| WalletError::InvalidConfig("invalid envelope nonce".to_string()))?;

            key.fill(0);

            let ciphertext = envelope[z00z_crypto::aead::ENVELOPE_HEADER_SIZE..].to_vec();

            Ok(RuntimeShowSeedPhraseResponse {
                encrypted_payload: crate::rpc::types::common::RuntimeEncryptedResponse {
                    ciphertext: hex::encode(ciphertext),
                    metadata: crate::rpc::types::common::RuntimeEncryptionMetadata {
                        algorithm: "xchacha20poly1305".to_string(),
                        nonce: format!("0x{}", hex::encode(nonce)),
                        key_derivation: "argon2id+hkdf-sha256(salt=wallet.seed_salt)".to_string(),
                    },
                },
            })
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = session;
            Err(WalletError::InvalidConfig(
                "show_seed_phrase is not supported on wasm32".to_string(),
            ))
        }
    }

    async fn put_owned_asset_source(
        &self,
        wallet_id: &PersistWalletId,
        asset: Asset,
        source: OwnedAssetSource,
    ) -> WalletResult<bool> {
        if asset_save_failpoint_enabled(wallet_id) {
            return Err(WalletError::InvalidConfig(
                "asset save failpoint enabled".to_string(),
            ));
        }

        asset
            .validate()
            .map_err(|err| WalletError::InvalidConfig(format!("invalid claimed asset: {err}")))?;

        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();
        let session = self
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await?;

        let outcome = session.with_wallet_session(|wlt_session| {
            crate::db::wallet_asset_store().put_owned_asset(
                wlt_session,
                asset,
                source,
                crate::db::AssetPersistContext {
                    now_ms,
                    ..crate::db::AssetPersistContext::default()
                },
            )
        })?;

        self.sync_claimed_asset_cache(wallet_id).await?;

        Ok(matches!(outcome, crate::db::PutAssetOutcome::Inserted { .. }))
    }

    /// Store a claimed asset through the `.wlt` owned-asset authority.
    /// This remains a manual-claim boundary. Canonical scan receive
    /// persistence stays in `recv_range(...)` so the wallet does not grow a
    /// second write authority for detected ownership.
    pub async fn put_claimed_asset(
        &self,
        wallet_id: &PersistWalletId,
        asset: Asset,
    ) -> WalletResult<bool> {
        self.put_owned_asset_source(wallet_id, asset, OwnedAssetSource::ManualClaim)
            .await
    }

    /// Import wallet-owned outputs through the same `.wlt` asset authority.
    pub async fn import_claimed_assets(
        &self,
        wallet_id: &PersistWalletId,
        assets: &[Asset],
    ) -> WalletResult<()> {
        if asset_save_failpoint_enabled(wallet_id) {
            return Err(WalletError::InvalidConfig(
                "asset save failpoint enabled".to_string(),
            ));
        }

        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();
        let session = self
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await?;

        let live_assets = session.with_wallet_session(|wlt_session| {
            crate::db::wallet_asset_store().put_owned_assets_batch(
                wlt_session,
                assets,
                OwnedAssetSource::Import,
                crate::db::AssetPersistContext {
                    now_ms,
                    ..crate::db::AssetPersistContext::default()
                },
            )?;
            self.load_claimed_assets_session(wlt_session)
        })?;

        self.install_claimed_assets(wallet_id, live_assets).await;
        Ok(())
    }

    /// Replace the visible claimed-asset set through the restricted `.wlt` restore helper.
    pub async fn set_claimed_assets(
        &self,
        wallet_id: &PersistWalletId,
        claimed_assets: Vec<Asset>,
    ) -> WalletResult<()> {
        let mut seen = std::collections::BTreeSet::new();
        for asset in &claimed_assets {
            asset.validate().map_err(|err| {
                WalletError::InvalidConfig(format!("invalid claimed asset: {err}"))
            })?;

            let asset_id = asset.asset_id();
            if !seen.insert(asset_id) {
                return Err(WalletError::InvalidConfig(
                    "duplicate claimed asset id in claimed set".to_string(),
                ));
            }
        }

        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();
        let session = self
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await?;

        let live_assets = session.with_wallet_session(|wlt_session| {
            crate::db::wallet_asset_store().replace_assets_for_restore(wlt_session, &claimed_assets)?;
            self.load_claimed_assets_session(wlt_session)
        })?;

        self.install_claimed_assets(wallet_id, live_assets).await;

        Ok(())
    }

    /// Confirm spent inputs and wallet-owned outputs through the live `.wlt` asset authority.
    pub async fn confirm_claimed_asset_spend(
        &self,
        wallet_id: &PersistWalletId,
        tx_id: &PersistTxId,
        spent_ids: &[[u8; 32]],
        new_outputs: &[Asset],
        new_output_source: OwnedAssetSource,
    ) -> WalletResult<()> {
        if asset_save_failpoint_enabled(wallet_id) {
            return Err(WalletError::InvalidConfig(
                "asset save failpoint enabled".to_string(),
            ));
        }

        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();
        let session = self
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await?;

        let live_assets = session.with_wallet_session(|wlt_session| {
            crate::db::wallet_asset_store()
                .confirm_asset_spend(
                    wlt_session,
                    tx_id,
                    spent_ids,
                    new_outputs,
                    new_output_source,
                )?;
            self.load_claimed_assets_session(wlt_session)
        })?;

        self.install_claimed_assets(wallet_id, live_assets).await;
        Ok(())
    }

    /// Reserve local owned assets for one pending wallet tx.
    pub async fn reserve_claimed_asset_inputs(
        &self,
        wallet_id: &PersistWalletId,
        tx_id: &PersistTxId,
        asset_ids: &[[u8; 32]],
    ) -> WalletResult<()> {
        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();
        let session = self
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await?;

        let live_assets = session.with_wallet_session(|wlt_session| {
            crate::db::wallet_asset_store().reserve_asset_inputs(wlt_session, tx_id, asset_ids)?;
            self.load_claimed_assets_session(wlt_session)
        })?;

        self.install_claimed_assets(wallet_id, live_assets).await;
        Ok(())
    }

    /// Release the local owned-asset reservation for one cancelled wallet tx.
    pub async fn release_claimed_asset_reservation(
        &self,
        wallet_id: &PersistWalletId,
        tx_id: &PersistTxId,
    ) -> WalletResult<()> {
        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();
        let session = self
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await?;

        let live_assets = session.with_wallet_session(|wlt_session| {
            crate::db::wallet_asset_store().release_asset_reservation(wlt_session, tx_id)?;
            self.load_claimed_assets_session(wlt_session)
        })?;

        self.install_claimed_assets(wallet_id, live_assets).await;
        Ok(())
    }

    /// Frozen receive-to-persist gate for Spec 6.
    ///
    /// `ReportOnly` keeps receive as a scan/report boundary only.
    /// `PersistClaim` stays a manual-claim route into the same owned-asset
    /// store. Canonical scan persistence remains `recv_range(...)` so
    /// report-only receive never becomes a shadow authority plane.
    /// Claimed state is reached only for `ReceiveNext::PersistClaim` after the
    /// supplied asset passes the wallet-native persistence boundary.
    pub async fn recv_route(
        &self,
        wallet_id: &PersistWalletId,
        asset: Asset,
        next: ReceiveNext,
    ) -> WalletResult<bool> {
        match next {
            ReceiveNext::ReportOnly => Ok(false),
            ReceiveNext::PersistClaim => self.put_claimed_asset(wallet_id, asset).await,
        }
    }

    /// Return the shared public receive status code used by service and RPC layers.
    pub fn recv_code(status: ReceiveStatus) -> &'static str {
        status.rpc_code()
    }


}
fn asset_save_failpoint_enabled(wallet_id: &PersistWalletId) -> bool {
    z00z_utils::config::ConfigSource::get(
        &z00z_utils::config::EnvConfig,
        "Z00Z_FAIL_ASSET_SAVE",
    )
        .ok()
        .flatten()
        .is_some_and(|value| value == "1" || value == wallet_id.0)
}
