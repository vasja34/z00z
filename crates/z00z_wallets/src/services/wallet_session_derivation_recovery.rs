use super::wallet_service_core::WalletReceiverDeriverHandle;
use crate::rpc::types::security::SessionToken;
use crate::domains::WalletKeyFingerprintDomain;
use z00z_crypto::expert::encoding::SafePassword;
use z00z_crypto::DomainHasher;
use z00z_utils::rng::SystemRngProvider;

#[derive(Debug)]
pub(crate) struct RotateMasterKeyOutcome {
    pub(crate) records_rewrapped: u32,
    pub(crate) new_fingerprint: [u8; 4],
}

fn rotate_master_key_fingerprint(input: &[u8]) -> [u8; 4] {
    let hash = DomainHasher::<WalletKeyFingerprintDomain>::new_with_label("key_fingerprint")
        .chain(input)
        .finalize();

    let mut out = [0u8; 4];
    out.copy_from_slice(&hash.as_ref()[..4]);
    out
}

impl WalletService {
    fn receiver_cache_file_path(&self, wallet_id: &PersistWalletId) -> std::path::PathBuf {
        self.wlt_file_path(wallet_id)
            .with_extension("receiver_cache")
    }

    async fn reconcile_progress_gap_limit(
        &self,
        wallet_id: &PersistWalletId,
        gap_limit: u32,
        is_used: ReceiverUsageOracle,
    ) -> WalletResult<ReceiverDeriverState> {
        let deriver = self.get_create_wallet_receiver_deriver(wallet_id).await?;

        if gap_limit == 0 {
            return Err(WalletError::InvalidParams(
                "gap_limit must be > 0".to_string(),
            ));
        }

        let payment_next = Self::reconcile_chain_next_index(
            Arc::clone(&deriver),
            true,
            gap_limit,
            Arc::clone(&is_used),
        )
        .await?;
        let change_next = Self::reconcile_chain_next_index(
            Arc::clone(&deriver),
            false,
            gap_limit,
            Arc::clone(&is_used),
        )
        .await?;

        let existing_counters = {
            let store = self.wallet_receiver_deriver_counters.read().await;
            store
                .get(wallet_id)
                .copied()
                .unwrap_or(ReceiverDeriverState {
                    next_payment_index: 0,
                    next_change_index: 0,
                })
        };

        let reconciled = ReceiverDeriverState {
            next_payment_index: existing_counters.next_payment_index.max(payment_next),
            next_change_index: existing_counters.next_change_index.max(change_next),
        };

        {
            let mut store = self.wallet_receiver_deriver_counters.write().await;
            store.insert(wallet_id.clone(), reconciled);
        }

        {
            let mut state = deriver.write().await;
            state.next_payment_index = state.next_payment_index.max(reconciled.next_payment_index);
            state.next_change_index = state.next_change_index.max(reconciled.next_change_index);
        }

        Ok(reconciled)
    }

    async fn reconcile_chain_next_index(
        deriver: WalletReceiverDeriverHandle,
        is_payment_chain: bool,
        gap_limit: u32,
        is_used: ReceiverUsageOracle,
    ) -> WalletResult<u32> {
        let mut max_used: Option<u32> = None;
        let mut consecutive_unused: u32 = 0;

        for index in 0..Self::MAX_GAP_SCAN_ADDRESSES {
            let path = if is_payment_chain {
                Bip44Path::payment_for_account(0, index)?
            } else {
                Bip44Path::change_path_for_account(0, index)?
            };

            let public_key = {
                let mut state = deriver.write().await;
                let pk = state
                    .receiver_manager
                    .derive_spend_key(path)
                    .map_err(|e| WalletError::KeyDerivation(e.to_string()))?;
                let mut bytes = [0u8; 32];
                bytes.copy_from_slice(pk.as_bytes());
                bytes
            };

            if is_used(path, public_key).await? {
                max_used = Some(index);
                consecutive_unused = 0;
            } else {
                consecutive_unused = consecutive_unused.saturating_add(1);
                if consecutive_unused >= gap_limit {
                    break;
                }
            }
        }

        Ok(max_used.map(|value| value.saturating_add(1)).unwrap_or(0))
    }

    /// Reconcile and persist derivation progress using a gap-limit scan policy.
    ///
    /// This implements Phase 2.3 recovery determinism rules:
    /// - For each chain (payment and change), derive sequential addresses and query the
    ///   provided `is_used` oracle.
    /// - Stop scanning a chain after `gap_limit` consecutive unused addresses.
    /// - Set `next_*_index = max_used_index + 1` (or `0` if no used addresses observed).
    /// - Persist updated indexes into the encrypted `.wlt` profile immediately.
    pub async fn reconcile_persist_gap_limit(
        &self,
        wallet_id: &PersistWalletId,
        gap_limit: u32,
        is_used: ReceiverUsageOracle,
    ) -> WalletResult<ReceiverDeriverState> {
        let reconciled = self
            .reconcile_progress_gap_limit(wallet_id, gap_limit, is_used)
            .await?;

        self.persist_profile_for_open_session(wallet_id).await?;

        Ok(reconciled)
    }

    /// List cached derived receiver public keys for a wallet.
    ///
    /// Returns `(Bip44Path, public_key_bytes)` for all cached paths.
    /// This is intentionally in-memory only and reflects only receiver
    /// material derived during the current process lifetime.
    pub async fn list_cached_receivers(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<Vec<(Bip44Path, [u8; 32])>> {
        let deriver = self.get_create_wallet_receiver_deriver(wallet_id).await?;
        let state = deriver.read().await;

        let entries = state
            .receiver_manager
            .list_receivers()
            .map_err(|e| WalletError::KeyDerivation(e.to_string()))?;

        let mut out = Vec::with_capacity(entries.len());
        for (path, public_key) in entries {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(public_key.as_bytes());
            out.push((path, bytes));
        }

        Ok(out)
    }

    /// Read the persisted derivation counters for a wallet.
    ///
    /// This reflects the latest in-memory counters restored from the `.wlt` profile
    /// and updated by derivation/recovery operations.
    pub async fn get_deriver_state(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<ReceiverDeriverState> {
        let store = self.wallet_receiver_deriver_counters.read().await;
        Ok(store
            .get(wallet_id)
            .copied()
            .unwrap_or(ReceiverDeriverState {
                next_payment_index: 0,
                next_change_index: 0,
            }))
    }

    /// Derive account-level pub material for pub material export.
    ///
    /// This is NOT a full BIP32 extended public key; it is deterministic
    /// account-scoped public material derived from the wallet's unlocked runtime
    /// seed material.
    pub async fn derive_account_pub_material(
        &self,
        wallet_id: &PersistWalletId,
        account: u32,
    ) -> WalletResult<[u8; 32]> {
        let path = Bip44Path::payment_for_account(account, 0)?;
        self.derive_public_key_for_path(wallet_id, path).await
    }

    /// Rewrite the persisted wallet encryption root.
    pub(crate) async fn rotate_master_key_persisted(
        &self,
        session: &SessionToken,
        password: &SafePassword,
    ) -> WalletResult<RotateMasterKeyOutcome> {
        let now_ms = self.require_now_ms()?;
        let session_handle = self
            .wallet_sessions
            .get_session_handle_without_touch(session, now_ms)
            .await?;
        let wallet_id = session.wallet_id.clone();

        let outcome = match session_handle.take_wallet_session_mut(|wlt_session| {
            let records_rewrapped =
                wlt_session.rotate_master_key_persisted(password, SystemRngProvider)?;
            Ok(RotateMasterKeyOutcome {
                records_rewrapped,
                new_fingerprint: rotate_master_key_fingerprint(
                    wlt_session.opened().master_key.reveal(),
                ),
            })
        }) {
            Ok(outcome) => outcome,
            Err(error) => {
                let _ = self.lock_wallet(&wallet_id).await;
                return Err(error);
            }
        };

        self.lock_wallet(&wallet_id).await?;

        Ok(outcome)
    }

    /// Rebuild cached derivation state after a successful persisted rotation.
    pub async fn rotate_master_key_in_memory(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<u32> {
        let deriver = self.get_create_wallet_receiver_deriver(wallet_id).await?;

        let (cached_paths, next_payment_index, next_change_index) = {
            let state = deriver.read().await;
            let entries = state
                .receiver_manager
                .list_receivers()
                .map_err(|e| WalletError::KeyDerivation(e.to_string()))?;
            let paths: Vec<Bip44Path> = entries.into_iter().map(|(path, _)| path).collect();
            (paths, state.next_payment_index, state.next_change_index)
        };

        let mut new_state = {
            let now_ms = self.require_now_ms()?;
            let timeout_ms = self.timeout_ms();
            let chain_type = self.resolve_persisted_wallet_chain_type(wallet_id).await?;
            let session = self
                .wallet_sessions
                .session_for_wallet(wallet_id, now_ms, timeout_ms)
                .await?;

            session.with_wallet_session(|wlt_session| {
                let mut seed = [0u8; 64];
                seed.copy_from_slice(wlt_session.opened().seed_bip39.reveal());
                Self::create_receiver_deriver_state(
                    Hidden::hide(seed),
                    ReceiverDeriverState {
                        next_payment_index,
                        next_change_index,
                    },
                    chain_type,
                    self.receiver_cache_size,
                    self.receiver_derive_rate_limit,
                )
            })?
        };

        for path in &cached_paths {
            new_state
                .receiver_manager
                .derive_spend_key(*path)
                .map_err(|e| WalletError::KeyDerivation(e.to_string()))?;
        }

        let mut replaced = deriver.write().await;
        *replaced = new_state;

        {
            let mut counters = self.wallet_receiver_deriver_counters.write().await;
            counters.insert(
                wallet_id.clone(),
                ReceiverDeriverState {
                    next_payment_index: replaced.next_payment_index,
                    next_change_index: replaced.next_change_index,
                },
            );
        }

        Ok(cached_paths.len() as u32)
    }
}
