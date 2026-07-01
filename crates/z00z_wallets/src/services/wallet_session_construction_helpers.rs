impl WalletService {
    pub(crate) fn compute_password_verifier(
        password: &SafePassword,
        salt: &[u8; 32],
    ) -> [u8; 32] {
        let password_bytes = password.reveal().as_slice();
        let hash = DomainHasher::<WalletPasswordVerifierDomain>::new_with_label("wallet_password")
            .chain(salt)
            .chain(password_bytes)
            .finalize();

        let mut out = [0u8; 32];
        out.copy_from_slice(&hash.as_ref()[..32]);
        out
    }

    /// Constant-time comparison for 32-byte arrays using `subtle` crate.
    ///
    /// Prevents timing attacks by ensuring comparison time is independent of input values.
    /// Uses `subtle::ConstantTimeEq` for cryptographic-grade constant-time guarantees.
    pub(crate) fn ct_cmp_32(a: &[u8; 32], b: &[u8; 32]) -> bool {
        bool::from(a.ct_eq(b))
    }

    /// Confirm (and remember) wallet password for sensitive operations (Phase 1).
    ///
    /// Behavior:
    /// - First call per wallet_id: stores a verifier derived from the provided password.
    /// - Subsequent calls: password must match the stored verifier.
    ///
    /// This is intentionally in-memory only and exists to support RPC validation
    /// without introducing persistence.
    pub async fn confirm_wallet_password(
        &self,
        wallet_id: &PersistWalletId,
        password: &SafePassword,
    ) -> WalletResult<()> {
        let password_bytes = password.reveal().as_slice();
        if password_bytes.is_empty() {
            return Err(WalletError::InvalidPassword);
        }

        let mut store = self.wallet_password_verifiers.write().await;

        if let Some(state) = store.get_mut(wallet_id) {
            let expected = Self::compute_password_verifier(password, &state.salt);
            if !Self::ct_cmp_32(&expected, &state.verifier) {
                return Err(WalletError::InvalidPassword);
            }
            return Ok(());
        }

        let mut salt = [0u8; 32];
        self.entropy.fill_bytes(&mut salt);

        let verifier = Self::compute_password_verifier(password, &salt);
        store.insert(
            wallet_id.clone(),
            WalletPasswordVerifierState { salt, verifier },
        );

        Ok(())
    }

    pub(crate) async fn confirm_wallet_password_with_backoff(
        &self,
        wallet_id: &PersistWalletId,
        password: &SafePassword,
    ) -> WalletResult<()> {
        match self.confirm_wallet_password(wallet_id, password).await {
            Ok(()) => {
                self.record_unlock_attempt_result(wallet_id, true).await;
                Ok(())
            }
            Err(WalletError::InvalidPassword) => {
                let failures = self.current_unlock_failures(wallet_id).await;
                let delay_ms = Self::compute_unlock_delay_ms(failures);
                self.sleeper
                    .sleep(std::time::Duration::from_millis(delay_ms))
                    .await;
                self.record_unlock_attempt_result(wallet_id, false).await;
                Err(WalletError::InvalidPassword)
            }
            Err(error) => Err(error),
        }
    }

    pub(crate) fn now_ms(&self) -> u64 {
        self.time_provider.compat_unix_timestamp_millis()
    }

    pub(crate) fn make_seed_salt(&self) -> [u8; 16] {
        let mut seed_salt = [0u8; 16];
        self.entropy.fill_bytes(&mut seed_salt);
        seed_salt
    }
}