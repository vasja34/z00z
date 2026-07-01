impl Default for WalletService {
    fn default() -> Self {
        Self::new()
    }
}

pub(super) fn recv_range_start(
    chunks: &[ScanChunk],
    resume: &ScanStatePayload,
) -> WalletResult<usize> {
    if resume.is_origin() {
        return Ok(0);
    }

    chunks
        .iter()
        .position(|chunk| resume.matches_chunk(chunk.height, &chunk.hash))
        .map(|pos| pos.saturating_add(1))
        .ok_or_else(|| WalletError::InvalidConfig("scan range cursor mismatch".to_string()))
}

/// Adapt one detected asset into the canonical claimed-asset shape.
///
/// The accepted receive path must stay fail-closed: assets with invalid
/// signatures are rejected instead of being silently scrubbed into a different
/// persisted shape.
pub(super) fn recv_claim_asset(asset: &Asset) -> Option<Asset> {
    asset.validate().ok().map(|_| asset.clone())
}

impl WalletService {
    pub(crate) async fn profile_header(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<(
        String,
        WalletState,
        WalletPasswordVerifierState,
        PersistWalletSettings,
    )> {
        let name = {
            let names = self.wallet_names.read().await;
            names
                .get(wallet_id)
                .cloned()
                .unwrap_or_else(|| "Unnamed Wallet".to_string())
        };

        let state = {
            let states = self.wallet_states.read().await;
            states
                .get(wallet_id)
                .cloned()
                .ok_or(WalletError::NotFound(0))?
        };

        let verifier_state = {
            let verifiers = self.wallet_password_verifiers.read().await;
            verifiers
                .get(wallet_id)
                .copied()
                .ok_or(WalletError::NotFound(0))?
        };

        let settings = {
            let settings_store = self.wallet_settings.read().await;
            settings_store
                .get(wallet_id)
                .cloned()
                .ok_or(WalletError::NotFound(0))?
        };

        Ok((name, state, verifier_state, settings))
    }

    pub(crate) async fn profile_receiver_deriver_state(
        &self,
        wallet_id: &PersistWalletId,
    ) -> ReceiverDeriverState {
        let deriver = {
            let derivers = self.wallet_receiver_derivers.read().await;
            derivers.get(wallet_id).cloned()
        };

        if let Some(deriver) = deriver {
            let state = deriver.read().await;
            return ReceiverDeriverState {
                next_payment_index: state.next_payment_index,
                next_change_index: state.next_change_index,
            };
        }

        let store = self.wallet_receiver_deriver_counters.read().await;
        store
            .get(wallet_id)
            .copied()
            .unwrap_or(ReceiverDeriverState {
                next_payment_index: 0,
                next_change_index: 0,
            })
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl WalletService {
    pub(crate) async fn resolve_persisted_wallet_identity(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<WalletIdentity> {
        let cached_identity = {
            let identities = self.wallet_identities.read().await;
            identities.get(wallet_id).cloned()
        };

        if let Some(identity) = cached_identity.as_ref() {
            if self
                .wallet_sessions
                .is_wallet_session_active(wallet_id, 0)
                .await
            {
                return Ok(identity.clone());
            }
        }

        let wlt_path = self.wlt_file_path(wallet_id);
        if wlt_path.exists() {
            let wlt_store = Arc::clone(&self.wlt_store);
            let discovery =
                tokio::task::spawn_blocking(move || wlt_store.discover_wallet_store(&wlt_path))
                    .await
                    .map_err(|_| {
                        WalletError::InvalidConfig(".wlt discover task failed".to_string())
                    })??;

            let identity = WalletIdentity {
                network: discovery.network,
                chain: discovery.chain,
            };

            let mut identities = self.wallet_identities.write().await;
            identities.insert(wallet_id.clone(), identity.clone());
            return Ok(identity);
        }

        if let Some(identity) = cached_identity {
            return Ok(identity);
        }

        crate::services::wallet_runtime_config::resolve_wallet_identity_checked()
    }

    pub(crate) async fn resolve_persisted_wallet_chain_type(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<ChainType> {
        let identity = self.resolve_persisted_wallet_identity(wallet_id).await?;
        crate::services::wallet_runtime_config::wallet_identity_chain_type(&identity)
    }

    pub(crate) async fn export_wallet_identity(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<crate::wallet::persistence::WalletExportIdentity> {
        let identity = self.resolve_persisted_wallet_identity(wallet_id).await?;
        Ok(crate::wallet::persistence::WalletExportIdentity {
            network: identity.network,
            chain: identity.chain,
        })
    }

    pub(crate) async fn list_wallet_inventory(
        &self,
        wallet_id: &PersistWalletId,
        filter: crate::db::ObjectInventoryFilter,
        cursor: Option<String>,
        limit: usize,
    ) -> WalletResult<crate::db::ObjectInventoryPage> {
        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();
        let session = self
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await?;

        session.with_wallet_session(|wlt_session| {
            crate::db::object_inventory_store().list_wallet_inventory(
                wlt_session,
                filter,
                cursor,
                limit,
            )
        })
    }

    #[allow(dead_code)]
    pub(crate) async fn import_owned_voucher(
        &self,
        wallet_id: &PersistWalletId,
        payload: crate::db::OwnedVoucherPayload,
    ) -> WalletResult<()> {
        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();
        let session = self
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await?;

        session.with_wallet_session(|wlt_session| {
            let _ = crate::db::object_inventory_store().put_voucher(wlt_session, payload)?;
            Ok(())
        })
    }

    #[allow(dead_code)]
    pub(crate) async fn import_owned_right(
        &self,
        wallet_id: &PersistWalletId,
        payload: crate::db::OwnedRightPayload,
    ) -> WalletResult<()> {
        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();
        let session = self
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await?;

        session.with_wallet_session(|wlt_session| {
            let _ = crate::db::object_inventory_store().put_right(wlt_session, payload)?;
            Ok(())
        })
    }

    pub(crate) async fn list_voucher_claim_rows(
        &self,
        wallet_id: &PersistWalletId,
        status: Option<crate::db::OwnedVoucherStatus>,
        cursor: Option<String>,
        limit: usize,
    ) -> WalletResult<Vec<crate::db::OwnedVoucherPayload>> {
        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();
        let session = self
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await?;

        session.with_wallet_session(|wlt_session| {
            crate::db::object_inventory_store().list_voucher_claims(
                wlt_session,
                status,
                cursor,
                limit,
            )
        })
    }

    pub(crate) async fn list_right_inventory_rows(
        &self,
        wallet_id: &PersistWalletId,
        status: Option<crate::db::OwnedRightStatus>,
        cursor: Option<String>,
        limit: usize,
    ) -> WalletResult<Vec<crate::db::OwnedRightPayload>> {
        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();
        let session = self
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await?;

        session.with_wallet_session(|wlt_session| {
            crate::db::object_inventory_store().list_right_inventory(
                wlt_session,
                status,
                cursor,
                limit,
            )
        })
    }

    pub(crate) async fn lookup_non_asset_owned_object(
        &self,
        wallet_id: &PersistWalletId,
        stable_key: [u8; 32],
    ) -> WalletResult<Option<crate::db::WalletOwnedObject>> {
        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();
        let session = self
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await?;

        session.with_wallet_session(|wlt_session| {
            crate::db::object_inventory_store().lookup_non_asset_object(wlt_session, &stable_key)
        })
    }

    pub(crate) async fn lookup_owned_asset_payload(
        &self,
        wallet_id: &PersistWalletId,
        asset_id: [u8; 32],
    ) -> WalletResult<Option<crate::db::OwnedAssetPayload>> {
        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();
        let session = self
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await?;

        session.with_wallet_session(|wlt_session| {
            crate::db::wallet_asset_store().get_owned_asset(wlt_session, &asset_id)
        })
    }

    #[allow(dead_code)]
    pub(crate) async fn apply_object_package_confirmation(
        &self,
        wallet_id: &PersistWalletId,
        payload: crate::db::OwnedNonAssetPayload,
    ) -> WalletResult<()> {
        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();
        let session = self
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await?;

        session.with_wallet_session(|wlt_session| match payload {
            crate::db::OwnedNonAssetPayload::Voucher(payload) => {
                crate::db::object_inventory_store().replace_voucher(wlt_session, payload)
            }
            crate::db::OwnedNonAssetPayload::Right(payload) => {
                crate::db::object_inventory_store().replace_right(wlt_session, payload)
            }
        })
    }

    #[cfg(test)]
    pub(crate) async fn put_owned_object_for_tests(
        &self,
        wallet_id: &PersistWalletId,
        payload: crate::db::OwnedObjectPayload,
    ) -> WalletResult<()> {
        match payload {
            crate::db::WalletInventoryPayload::Asset(_) => Err(WalletError::InvalidConfig(
                "test object helper only accepts voucher/right payloads".to_string(),
            )),
            crate::db::WalletInventoryPayload::Voucher(payload) => {
                self.import_owned_voucher(wallet_id, payload).await
            }
            crate::db::WalletInventoryPayload::Right(payload) => {
                self.import_owned_right(wallet_id, payload).await
            }
        }
    }

    #[cfg(test)]
    pub(crate) async fn put_test_asset_payload(
        &self,
        wallet_id: &PersistWalletId,
        payload: crate::db::OwnedAssetPayload,
    ) -> WalletResult<()> {
        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();
        let session = self
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await?;

        session.with_wallet_session(|wlt_session| {
            crate::db::wallet_asset_store()
                .replace_payloads_for_restore(wlt_session, &[payload])?;
            Ok(())
        })
    }
}

#[cfg(target_arch = "wasm32")]
impl WalletService {
    pub(crate) async fn list_wallet_inventory(
        &self,
        wallet_id: &PersistWalletId,
        filter: crate::db::ObjectInventoryFilter,
        cursor: Option<String>,
        limit: usize,
    ) -> WalletResult<crate::db::ObjectInventoryPage> {
        let _ = (wallet_id, filter, cursor, limit);
        Err(WalletError::InvalidConfig(
            "live object inventory query is not supported on wasm32".to_string(),
        ))
    }

    #[allow(dead_code)]
    pub(crate) async fn import_owned_voucher(
        &self,
        wallet_id: &PersistWalletId,
        payload: crate::db::OwnedVoucherPayload,
    ) -> WalletResult<()> {
        let _ = (wallet_id, payload);
        Err(WalletError::InvalidConfig(
            "live voucher import is not supported on wasm32".to_string(),
        ))
    }

    #[allow(dead_code)]
    pub(crate) async fn import_owned_right(
        &self,
        wallet_id: &PersistWalletId,
        payload: crate::db::OwnedRightPayload,
    ) -> WalletResult<()> {
        let _ = (wallet_id, payload);
        Err(WalletError::InvalidConfig(
            "live right import is not supported on wasm32".to_string(),
        ))
    }

    pub(crate) async fn list_voucher_claim_rows(
        &self,
        wallet_id: &PersistWalletId,
        status: Option<crate::db::OwnedVoucherStatus>,
        cursor: Option<String>,
        limit: usize,
    ) -> WalletResult<Vec<crate::db::OwnedVoucherPayload>> {
        let _ = (wallet_id, status, cursor, limit);
        Err(WalletError::InvalidConfig(
            "live voucher inventory query is not supported on wasm32".to_string(),
        ))
    }

    pub(crate) async fn list_right_inventory_rows(
        &self,
        wallet_id: &PersistWalletId,
        status: Option<crate::db::OwnedRightStatus>,
        cursor: Option<String>,
        limit: usize,
    ) -> WalletResult<Vec<crate::db::OwnedRightPayload>> {
        let _ = (wallet_id, status, cursor, limit);
        Err(WalletError::InvalidConfig(
            "live right inventory query is not supported on wasm32".to_string(),
        ))
    }

    pub(crate) async fn lookup_non_asset_owned_object(
        &self,
        wallet_id: &PersistWalletId,
        stable_key: [u8; 32],
    ) -> WalletResult<Option<crate::db::WalletOwnedObject>> {
        let _ = (wallet_id, stable_key);
        Err(WalletError::InvalidConfig(
            "live object lookup is not supported on wasm32".to_string(),
        ))
    }

    pub(crate) async fn lookup_owned_asset_payload(
        &self,
        wallet_id: &PersistWalletId,
        asset_id: [u8; 32],
    ) -> WalletResult<Option<crate::db::OwnedAssetPayload>> {
        let _ = (wallet_id, asset_id);
        Err(WalletError::InvalidConfig(
            "live asset lookup is not supported on wasm32".to_string(),
        ))
    }

    #[allow(dead_code)]
    pub(crate) async fn apply_object_package_confirmation(
        &self,
        wallet_id: &PersistWalletId,
        payload: crate::db::OwnedNonAssetPayload,
    ) -> WalletResult<()> {
        let _ = (wallet_id, payload);
        Err(WalletError::InvalidConfig(
            "object confirmation apply is not supported on wasm32".to_string(),
        ))
    }
}
