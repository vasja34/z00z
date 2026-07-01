impl WalletService {
    /// Derive the canonical live receiver keys for the active wallet.
    ///
    /// Phase 037 receive detection should treat these keys as the authoritative
    /// wallet-native receiver material for `scan_asset_report(...)` and
    /// `recv_range(...)`.
    pub async fn receiver_keys(&self, wallet_id: &PersistWalletId) -> WalletResult<ReceiverKeys> {
        self.live_receiver_keys(wallet_id).await
    }

    /// Derive receiver keys for an explicit persisted view-key version.
    pub async fn receiver_keys_for_view_version(
        &self,
        wallet_id: &PersistWalletId,
        view_ver: u32,
    ) -> WalletResult<ReceiverKeys> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            const MAX_VIEW_VER: u32 = 1024;

            if view_ver > MAX_VIEW_VER {
                return Err(WalletError::InvalidConfig(
                    "stealth view version too large".to_string(),
                ));
            }

            let mut recv_keys = self.live_receiver_keys(wallet_id).await?;

            for _ in 0..view_ver {
                let _ = recv_keys
                    .rotate_view()
                    .map_err(|e| WalletError::KeyDerivation(e.to_string()))?;
            }

            Ok(recv_keys)
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = (wallet_id, view_ver);
            Err(WalletError::InvalidConfig(
                "receiver_keys_for_view_version is not supported on wasm32".to_string(),
            ))
        }
    }

    fn derive_live_secret(
        wallet_id: &PersistWalletId,
        seed: &[u8],
    ) -> WalletResult<ReceiverSecret> {
        const MAX_RETRY: u32 = 16;

        let mut retry = 0u32;
        let mut bytes = z00z_crypto::hash::poseidon2_hash(
            b"z00z.wallet.receiver_secret.v1",
            &[wallet_id.0.as_bytes(), seed],
        );

        loop {
            match ReceiverSecret::from_bytes(bytes) {
                Ok(secret) => return Ok(secret),
                Err(
                    StealthKeyError::ZeroSecret
                    | StealthKeyError::InvalidSecretKey
                    | StealthKeyError::ZeroScalarRejected
                    | StealthKeyError::IdentityPointRejected,
                ) if retry < MAX_RETRY => {
                    retry += 1;
                    let retry_bytes = retry.to_le_bytes();
                    bytes = z00z_crypto::hash::poseidon2_hash(
                        b"z00z.wallet.receiver_secret.retry.v1",
                        &[wallet_id.0.as_bytes(), seed, &retry_bytes, &bytes],
                    );
                }
                Err(err) => return Err(WalletError::KeyDerivation(err.to_string())),
            }
        }
    }

    /// Internal backend for the live receive lane.
    pub(crate) async fn live_receiver_keys(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<ReceiverKeys> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let now_ms = self.require_now_ms()?;
            let timeout_ms = self.timeout_ms();
            let session = self
                .wallet_sessions
                .session_for_wallet(wallet_id, now_ms, timeout_ms)
                .await?;

            let recv_secret = session.with_wallet_session(|wlt_session| {
                let seed = wlt_session.opened().seed_bip39.reveal();
                Self::derive_live_secret(wallet_id, seed)
            })?;

            ReceiverKeys::from_receiver_secret(recv_secret)
                .map_err(|e| WalletError::KeyDerivation(e.to_string()))
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = wallet_id;
            Err(WalletError::InvalidConfig(
                "live_receiver_keys is not supported on wasm32".to_string(),
            ))
        }
    }

}
