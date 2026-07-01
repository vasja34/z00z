impl WalletService {
    /// Rebuild cached derivation state after the persisted root has already been rewritten.
    ///
    /// This helper is a post-commit cache refresh only. It does not mutate the persisted
    /// master-key root on its own.
    pub async fn rotate_master_key_in_memory(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<u32> {
        let deriver = self.get_create_wallet_receiver_deriver(wallet_id).await?;

        // Collect cached paths and counters without mutating the current state.
        let (cached_paths, next_payment_index, next_change_index) = {
            let state = deriver.read().await;
            let entries = state
                .receiver_manager
                .list_receivers()
                .map_err(|e| WalletError::KeyDerivation(e.to_string()))?;
            let paths: Vec<Bip44Path> = entries.into_iter().map(|(p, _)| p).collect();
            (paths, state.next_payment_index, state.next_change_index)
        };

        // Build new state and re-derive cached paths.
        let mut new_state = {
            let now_ms = self.require_now_ms()?;
            let timeout_ms = self.timeout_ms();
            let session = self
                .wallet_sessions
                .session_for_wallet(wallet_id, now_ms, timeout_ms)
                .await?;

            session.with_wallet_session(|wlt_session| {
                let mut seed = [0u8; 64];
                seed.copy_from_slice(wlt_session.opened().seed_bip39.reveal());
                let chain_type = resolve_wallet_chain_type_checked()?;
                Self::create_receiver_deriver_state(
                    Hidden::hide(seed),
                    ReceiverDeriverState {
                        next_payment_index,
                        next_change_index,
                    },
                    chain_type,
                )
            })?
        };
        // Counters are already applied via `create_receiver_deriver_state`.

        for path in &cached_paths {
            new_state
                .receiver_manager
                .derive_spend_key(*path)
                .map_err(|e| WalletError::KeyDerivation(e.to_string()))?;
        }

        // Swap in the rotated state (commit point).
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
