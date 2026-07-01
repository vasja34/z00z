#[cfg(not(target_arch = "wasm32"))]
struct WalletExportState {
    seed_phrase: String,
    reused_session: bool,
    owned_assets: Vec<crate::db::OwnedAssetPayload>,
    owned_objects: Vec<crate::db::WalletInventoryPayload>,
    scan_state: Option<crate::db::ScanStatePayload>,
    stealth_meta: Option<crate::db::StealthMetaPayload>,
    tofu_pins: Option<crate::db::TofuPinsPayload>,
    keys: Option<crate::db::KeysPayload>,
}

impl WalletService {
    /// Create profile from current in-memory non-asset state.
    pub(crate) async fn create_profile(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<WalletProfilePayload> {
        let (name, state, verifier_state, settings) = self.profile_header(wallet_id).await?;
        let deriver_state = self.profile_receiver_deriver_state(wallet_id).await;
        let seed_salt = self.seed_salt_for_save(wallet_id).await;
        let persisted_state = match state {
            WalletState::Unlocked { .. } => WalletState::Locked,
            other => other,
        };

        Ok(WalletProfilePayload::new_with_checksum(
            wallet_id.clone(),
            name,
            settings.created_at,
            self.now_ms(),
            PasswordVerifierState {
                salt: verifier_state.salt,
                verifier: verifier_state.verifier,
            },
            deriver_state,
            settings,
            seed_salt,
            persisted_state,
        ))
    }

    pub(crate) async fn build_wallet_export_pack(
        &self,
        wallet_id: &PersistWalletId,
        password: &SafePassword,
    ) -> WalletResult<WalletExportPack> {
        self.build_export_pack_with_history(wallet_id, password, false)
            .await
    }

    pub(crate) async fn build_backup_export_pack(
        &self,
        wallet_id: &PersistWalletId,
        password: &SafePassword,
    ) -> WalletResult<WalletExportPack> {
        self.build_export_pack_with_history(wallet_id, password, true)
            .await
    }

    async fn build_export_pack_with_history(
        &self,
        wallet_id: &PersistWalletId,
        password: &SafePassword,
        include_history_sidecar: bool,
    ) -> WalletResult<WalletExportPack> {
        #[cfg(not(target_arch = "wasm32"))]
        let export_state = self.load_wallet_export_state(wallet_id, password).await?;

        #[cfg(not(target_arch = "wasm32"))]
        if export_state.reused_session {
            self.update_activity(wallet_id).await?;
        }

        #[cfg(target_arch = "wasm32")]
        let seed_phrase = String::new();

        let profile = self.create_profile(wallet_id).await?;
        #[cfg(not(target_arch = "wasm32"))]
        let wallet_identity = Some(self.export_wallet_identity(wallet_id).await?);
        #[cfg(target_arch = "wasm32")]
        let wallet_identity = None;

        #[cfg(not(target_arch = "wasm32"))]
        let mut manifest = crate::db::BackupManifestPayload {
            version: crate::db::BackupManifestPayload::VERSION,
            wallet_id: wallet_id.clone(),
            created_at_ms: self.now_ms(),
            network: wallet_identity
                .as_ref()
                .map(|value| value.network.clone())
                .unwrap_or_default(),
            chain: wallet_identity
                .as_ref()
                .map(|value| value.chain.clone())
                .unwrap_or_default(),
            profile_count: 1,
            owned_asset_count: export_state.owned_assets.len() as u32,
            owned_object_count: export_state.owned_objects.len() as u32,
            scan_state_count: u32::from(export_state.scan_state.is_some()),
            stealth_meta_count: u32::from(export_state.stealth_meta.is_some()),
            tofu_pins_count: u32::from(export_state.tofu_pins.is_some()),
            key_ref_count: export_state
                .keys
                .as_ref()
                .map(|value| value.signing_keys.len() as u32)
                .unwrap_or(0),
            tx_record_count: 0,
            has_tx_history_sidecar: include_history_sidecar,
            tx_history_plane: crate::db::BackupManifestPayload::TX_HISTORY_JSONL.to_string(),
            checksum: None,
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            manifest.checksum = Some(manifest.compute_checksum());
        }

        Ok(WalletExportPack {
            version: WalletExportPack::VERSION,
            #[cfg(not(target_arch = "wasm32"))]
            manifest: Some(manifest),
            #[cfg(target_arch = "wasm32")]
            manifest: None,
            wallet_profile: Some(profile),
            #[cfg(not(target_arch = "wasm32"))]
            owned_assets: export_state.owned_assets,
            #[cfg(target_arch = "wasm32")]
            owned_assets: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            owned_objects: export_state.owned_objects,
            #[cfg(target_arch = "wasm32")]
            owned_objects: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            scan_state: export_state.scan_state,
            #[cfg(target_arch = "wasm32")]
            scan_state: None,
            #[cfg(not(target_arch = "wasm32"))]
            stealth_meta: export_state.stealth_meta,
            #[cfg(target_arch = "wasm32")]
            stealth_meta: None,
            #[cfg(not(target_arch = "wasm32"))]
            tofu_pins: export_state.tofu_pins,
            #[cfg(target_arch = "wasm32")]
            tofu_pins: None,
            #[cfg(not(target_arch = "wasm32"))]
            keys: export_state.keys,
            #[cfg(target_arch = "wasm32")]
            keys: None,
            tx_history_plane: Some(crate::db::BackupManifestPayload::TX_HISTORY_JSONL.to_string()),
            #[cfg(not(target_arch = "wasm32"))]
            seed_phrase: export_state.seed_phrase,
            #[cfg(target_arch = "wasm32")]
            seed_phrase,
            wallet_identity,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn load_wallet_export_state(
        &self,
        wallet_id: &PersistWalletId,
        password: &SafePassword,
    ) -> WalletResult<WalletExportState> {
        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();
        let wlt_store = Arc::clone(&self.wlt_store);

        let read_from_session =
            move |session: &crate::db::WalletSession| -> WalletResult<WalletExportState> {
                let seed_phrase = wlt_store.reveal_seed_phrase(session)?;
                let owned_assets = crate::db::wallet_asset_store()
                    .list_owned_assets(
                        session,
                        crate::db::AssetFilter::default(),
                        None,
                        usize::MAX,
                    )?
                    .items;
                let owned_objects = crate::db::object_inventory_store()
                    .list_wallet_inventory(
                        session,
                        crate::db::ObjectInventoryFilter::default(),
                        None,
                        usize::MAX,
                    )?
                    .items
                    .into_iter()
                    .filter_map(|object| match object.payload {
                        crate::db::OwnedObjectPayload::Asset(_) => None,
                        payload => Some(payload),
                    })
                    .collect();
                let scan_state = crate::db::read_scan_state(session)?;
                let stealth_meta = crate::db::read_stealth_meta(session)?;
                let tofu_pins = crate::db::read_tofu_pins(session)?;
                let keys = crate::db::read_keys_payload(session)?;
                Ok(WalletExportState {
                    seed_phrase,
                    reused_session: false,
                    owned_assets,
                    owned_objects,
                    scan_state,
                    stealth_meta,
                    tofu_pins,
                    keys,
                })
            };

        match self
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await
        {
            Ok(session_handle) => {
                let export_state = tokio::task::spawn_blocking(move || {
                    session_handle.with_wallet_session(read_from_session)
                })
                .await
                .map_err(|_| WalletError::InvalidConfig("export task failed".to_string()))??;
                Ok(WalletExportState {
                    reused_session: true,
                    ..export_state
                })
            }
            Err(WalletError::SessionInvalid) | Err(WalletError::SessionExpired) => {
                let wlt_path = self.wlt_file_path(wallet_id);
                let wallet_id_cloned = wallet_id.clone();
                let identity = self.resolve_persisted_wallet_identity(wallet_id).await?;
                let password = password.clone();
                let wlt_store = Arc::clone(&self.wlt_store);

                tokio::task::spawn_blocking(move || {
                    let session = wlt_store.open_wallet_store(
                        &wlt_path,
                        &wallet_id_cloned,
                        &password,
                        &identity,
                    )?;
                    let seed_phrase = wlt_store.reveal_seed_phrase(&session)?;
                    let owned_assets = crate::db::wallet_asset_store()
                        .list_owned_assets(
                            &session,
                            crate::db::AssetFilter::default(),
                            None,
                            usize::MAX,
                        )?
                        .items;
                    let owned_objects = crate::db::object_inventory_store()
                        .list_wallet_inventory(
                            &session,
                            crate::db::ObjectInventoryFilter::default(),
                            None,
                            usize::MAX,
                        )?
                        .items
                        .into_iter()
                        .filter_map(|object| match object.payload {
                            crate::db::OwnedObjectPayload::Asset(_) => None,
                            payload => Some(payload),
                        })
                        .collect();
                    let scan_state = crate::db::read_scan_state(&session)?;
                    let stealth_meta = crate::db::read_stealth_meta(&session)?;
                    let tofu_pins = crate::db::read_tofu_pins(&session)?;
                    let keys = crate::db::read_keys_payload(&session)?;
                    Ok(WalletExportState {
                        seed_phrase,
                        reused_session: false,
                        owned_assets,
                        owned_objects,
                        scan_state,
                        stealth_meta,
                        tofu_pins,
                        keys,
                    })
                })
                .await
                .map_err(|_| WalletError::InvalidConfig("export task failed".to_string()))?
            }
            Err(err) => Err(err),
        }
    }

    pub(crate) async fn restore_wallet_export_pack(
        &self,
        export_pack: WalletExportPack,
        password: &SafePassword,
        wallet_name: Option<&str>,
        identity: &WalletIdentity,
    ) -> WalletResult<PersistWalletId> {
        let export_pack = Self::project_wallet_only_pack(export_pack);
        self.restore_wallet_pack_atomic(export_pack, password, wallet_name, identity, None)
            .await
    }

    fn project_wallet_only_pack(mut export_pack: WalletExportPack) -> WalletExportPack {
        // WalletOnly restore must validate the wallet pack without requiring a
        // published tx-history sidecar from an otherwise canonical backup.
        if let Some(manifest) = export_pack.manifest.as_mut() {
            manifest.tx_record_count = 0;
            manifest.has_tx_history_sidecar = false;
            manifest.checksum = Some(manifest.compute_checksum());
        }

        export_pack
    }
}
