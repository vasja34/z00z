// KEEP/REMOVE NOTE: This file remains in-tree as a non-canonical duplicate.
// The live WalletService include stack does not wire it in unless
// wallet_actions.rs explicitly includes it, so it must not be used as
// receive ownership authority.

impl WalletService {
impl WalletService {
    fn apply_hardening() {
        #[cfg(all(feature = "os_hardening", not(target_arch = "wasm32")))]
        {
            let report = z00z_utils::os_hardening::apply_best_effort();
            z00z_utils::logger::Logger::info(
                &z00z_utils::logger::TracingLogger,
                &format!(
                    "wallet os hardening applied: core_dumps_disabled={}, non_dumpable={}, notes={}",
                    report.core_dumps_disabled,
                    report.non_dumpable,
                    report.notes.len()
                ),
            );
            for note in report.notes {
                z00z_utils::logger::Logger::debug(
                    &z00z_utils::logger::TracingLogger,
                    &format!("wallet os hardening note: {}", note),
                );
            }
        }
    }

    const MAX_GAP_SCAN_ADDRESSES: u32 = 1_000_000;

    fn default_entropy() -> Arc<dyn WalletEntropy> {
        Arc::new(WalletEntropyFromRngProvider::new(SystemRngProvider))
    }

    fn default_output_dir() -> PathBuf {
        resolve_wallet_output_dir()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn cleanup_lock_file_best_effort(&self, wallet_id: &PersistWalletId) {
        use fs2::FileExt as _;
        use z00z_utils::io::{remove_file, File};

        let wlt_path = self.wlt_file_path(wallet_id);

        // Avoid deleting a lock file that is currently held by this process.
        if crate::db::is_lock_held_local(&wlt_path) {
            return;
        }

        let lock_path = {
            let mut os = wlt_path.as_os_str().to_os_string();
            os.push(".lock");
            PathBuf::from(os)
        };

        if !lock_path.exists() {
            return;
        }

        let lock_file = match File::options()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&lock_path)
        {
            Ok(file) => file,
            Err(_) => return,
        };

        if lock_file.try_lock_exclusive().is_err() {
            return;
        }

        let _ = lock_file.unlock();
        let _ = remove_file(&lock_path);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn cleanup_stale_locks(&self) {
        use z00z_utils::io::read_dir;

        let Ok(entries) = read_dir(&self.output_dir) else {
            return;
        };

        for entry in entries {
            let Some(name) = entry.file_name().and_then(|s| s.to_str()) else {
                continue;
            };

            if !name.starts_with("wallet_") || !name.ends_with(".wlt.lock") {
                continue;
            }

            let Some(stem) = name.strip_suffix(".lock") else {
                continue;
            };

            let wallet_id = PersistWalletId(stem.to_string());
            self.cleanup_lock_file_best_effort(&wallet_id);
        }
    }

    pub(crate) fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    /// Derive receiver keys for the active wallet in this duplicate helper.
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

    async fn live_receiver_keys(&self, wallet_id: &PersistWalletId) -> WalletResult<ReceiverKeys> {
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
