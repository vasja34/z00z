impl WalletService {
    fn lvl_to_u8(level: &TrustLevel) -> u8 {
        match level {
            TrustLevel::Tentative => 0,
            TrustLevel::Pinned => 1,
            TrustLevel::Expired => 2,
            TrustLevel::Revoked => 3,
        }
    }

    fn lvl_from_u8(level: u8) -> WalletResult<TrustLevel> {
        match level {
            0 => Ok(TrustLevel::Tentative),
            1 => Ok(TrustLevel::Pinned),
            2 => Ok(TrustLevel::Expired),
            3 => Ok(TrustLevel::Revoked),
            _ => Err(WalletError::InvalidConfig(
                "invalid tofu trust level".to_string(),
            )),
        }
    }

    fn tofu_to_pins(payload: TofuPinsPayload) -> WalletResult<PinnedReceiverCards> {
        let mut pairs = Vec::with_capacity(payload.pins.len());
        for pin in payload.pins {
            pairs.push((
                pin.owner_handle,
                PinEntry {
                    view_pk: pin.view_pk,
                    identity_pk: pin.identity_pk,
                    directory_id: pin.directory_id,
                    first_seen: pin.first_seen,
                    trust_level: Self::lvl_from_u8(pin.trust_level)?,
                },
            ));
        }
        Ok(PinnedReceiverCards::from_pairs(pairs))
    }

    fn tofu_from_pins(pins: &PinnedReceiverCards, now_ms: u64) -> TofuPinsPayload {
        let pins = pins
            .to_pairs()
            .into_iter()
            .map(|(owner_handle, entry)| TofuPinRecord {
                owner_handle,
                view_pk: entry.view_pk,
                identity_pk: entry.identity_pk,
                directory_id: entry.directory_id,
                first_seen: entry.first_seen,
                trust_level: Self::lvl_to_u8(&entry.trust_level),
            })
            .collect();

        TofuPinsPayload {
            pins,
            updated_at: now_ms,
        }
    }

    /// Load persisted TOFU pin set for wallet session.
    pub async fn load_tofu(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<PinnedReceiverCards> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let now_ms = self.require_now_ms()?;
            let timeout_ms = self.timeout_ms();
            let session = self
                .wallet_sessions
                .session_for_wallet(wallet_id, now_ms, timeout_ms)
                .await?;

            session.with_wallet_session(|wlt_session| {
                let payload = crate::db::read_tofu_pins(wlt_session)?;
                match payload {
                    Some(payload) => Self::tofu_to_pins(payload),
                    None => Ok(PinnedReceiverCards::new()),
                }
            })
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = wallet_id;
            Err(WalletError::InvalidConfig(
                "load_tofu is not supported on wasm32".to_string(),
            ))
        }
    }

    /// Save TOFU pin set for wallet session.
    pub async fn save_tofu(
        &self,
        wallet_id: &PersistWalletId,
        pins: &PinnedReceiverCards,
    ) -> WalletResult<()> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let now_ms = self.require_now_ms()?;
            let timeout_ms = self.timeout_ms();
            let payload = Self::tofu_from_pins(pins, now_ms);

            let session = self
                .wallet_sessions
                .session_for_wallet(wallet_id, now_ms, timeout_ms)
                .await?;

            session.with_wallet_session(|wlt_session| {
                let _ =
                    crate::db::upsert_tofu_pins(wlt_session, &payload, SystemRngProvider)?;
                Ok(())
            })
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = wallet_id;
            let _ = pins;
            Err(WalletError::InvalidConfig(
                "save_tofu is not supported on wasm32".to_string(),
            ))
        }
    }

    /// Verify receiver card against TOFU store and persist changes atomically.
    pub async fn tofu_verify_pin(
        &self,
        wallet_id: &PersistWalletId,
        card: &ReceiverCard,
        directory_id: Option<&str>,
    ) -> WalletResult<VerifyResult> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let now_ms = self.require_now_ms()?;
            let timeout_ms = self.timeout_ms();
            let session = self
                .wallet_sessions
                .session_for_wallet(wallet_id, now_ms, timeout_ms)
                .await?;

            session.with_wallet_session(|wlt_session| {
                let mut pins = match crate::db::read_tofu_pins(wlt_session)? {
                    Some(payload) => Self::tofu_to_pins(payload)?,
                    None => PinnedReceiverCards::new(),
                };

                let result = pins
                    .verify_or_pin(card, directory_id)
                    .map_err(|e| WalletError::InvalidParams(e.to_string()))?;

                let payload = Self::tofu_from_pins(&pins, now_ms);
                let _ =
                    crate::db::upsert_tofu_pins(wlt_session, &payload, SystemRngProvider)?;

                Ok(result)
            })
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = wallet_id;
            let _ = card;
            let _ = directory_id;
            Err(WalletError::InvalidConfig(
                "tofu_verify_pin is not supported on wasm32".to_string(),
            ))
        }
    }

    /// Confirm TOFU rotation and persist changes atomically.
    pub async fn tofu_confirm(
        &self,
        wallet_id: &PersistWalletId,
        owner: &[u8; 32],
        new_view: &[u8; 32],
    ) -> WalletResult<()> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let now_ms = self.require_now_ms()?;
            let timeout_ms = self.timeout_ms();
            let session = self
                .wallet_sessions
                .session_for_wallet(wallet_id, now_ms, timeout_ms)
                .await?;

            session.with_wallet_session(|wlt_session| {
                let mut pins = match crate::db::read_tofu_pins(wlt_session)? {
                    Some(payload) => Self::tofu_to_pins(payload)?,
                    None => PinnedReceiverCards::new(),
                };

                if pins.get(owner).is_none() {
                    return Err(WalletError::InvalidParams(
                        "tofu owner not found".to_string(),
                    ));
                }

                pins.confirm_rotation(owner, new_view);
                let payload = Self::tofu_from_pins(&pins, now_ms);
                let _ =
                    crate::db::upsert_tofu_pins(wlt_session, &payload, SystemRngProvider)?;
                Ok(())
            })
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = wallet_id;
            let _ = owner;
            let _ = new_view;
            Err(WalletError::InvalidConfig(
                "tofu_confirm is not supported on wasm32".to_string(),
            ))
        }
    }

    /// Revoke TOFU pin and persist changes atomically.
    pub async fn tofu_revoke(
        &self,
        wallet_id: &PersistWalletId,
        owner: &[u8; 32],
    ) -> WalletResult<()> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let now_ms = self.require_now_ms()?;
            let timeout_ms = self.timeout_ms();
            let session = self
                .wallet_sessions
                .session_for_wallet(wallet_id, now_ms, timeout_ms)
                .await?;

            session.with_wallet_session(|wlt_session| {
                let mut pins = match crate::db::read_tofu_pins(wlt_session)? {
                    Some(payload) => Self::tofu_to_pins(payload)?,
                    None => PinnedReceiverCards::new(),
                };

                if pins.get(owner).is_none() {
                    return Err(WalletError::InvalidParams(
                        "tofu owner not found".to_string(),
                    ));
                }

                pins.revoke(owner);
                let payload = Self::tofu_from_pins(&pins, now_ms);
                let _ =
                    crate::db::upsert_tofu_pins(wlt_session, &payload, SystemRngProvider)?;
                Ok(())
            })
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = wallet_id;
            let _ = owner;
            Err(WalletError::InvalidConfig(
                "tofu_revoke is not supported on wasm32".to_string(),
            ))
        }
    }

    /// Rotate stealth receiver view version and persist the updated StealthMeta.
    pub async fn rotate_recv_view(&self, wallet_id: &PersistWalletId) -> WalletResult<u32> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let now_ms = self.require_now_ms()?;
            let timeout_ms = self.timeout_ms();
            let session = self
                .wallet_sessions
                .session_for_wallet(wallet_id, now_ms, timeout_ms)
                .await?;

            session.with_wallet_session(|wlt_session| {
                let mut meta = crate::db::read_stealth_meta(wlt_session)?.unwrap_or(
                    StealthMetaPayload {
                        view_key_version: 0,
                        receiver_mode: "stealth_ecdh".to_string(),
                        stealth_activated_at: None,
                        mode_audit: Vec::new(),
                    },
                );

                let next_ver = meta.view_key_version.checked_add(1).ok_or_else(|| {
                    WalletError::InvalidConfig("view version overflow".to_string())
                })?;
                meta.view_key_version = next_ver;

                if meta.receiver_mode != "stealth_ecdh" {
                    meta.receiver_mode = "stealth_ecdh".to_string();
                }
                if meta.stealth_activated_at.is_none() {
                    meta.stealth_activated_at = Some(now_ms);
                }

                let _ =
                    crate::db::upsert_stealth_meta(wlt_session, &meta, SystemRngProvider)?;
                Ok(next_ver)
            })
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = wallet_id;
            Err(WalletError::InvalidConfig(
                "rotate_recv_view is not supported on wasm32".to_string(),
            ))
        }
    }


}
