impl KeyManagerImpl<SystemRngProvider> {
    /// Create a new key manager.
    pub fn new() -> Self {
        Self::new_with_observability(
            Arc::new(NoopLogger),
            Arc::new(NoopMetrics),
            Arc::new(SystemTimeProvider),
            SystemRngProvider,
        )
    }

    /// Convert an encrypted seed container + metadata into a persisted `KeyManagerState`.
    ///
    /// This is a pure helper that serializes the encrypted seed bytes; it does not perform
    /// decryption and does not require an initialized key manager.
    pub fn state_from_encrypted_seed(
        encrypted_seed: CipherSeedContainer,
        metadata: KeyManagerMetadata,
        chain: ChainType,
    ) -> Result<KeyManagerState> {
        let encrypted_seed_bytes = encrypted_seed
            .to_bytes()
            .map_err(|_| KeyManagerError::StateCorrupted)?;

        Ok(KeyManagerState {
            encrypted_seed_bytes,
            metadata,
            chain,
        })
    }

    /// Create and unlock a key manager from a persisted `.wlt` RedB wallet.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn unlock_from_storage(
        path: &Path,
        wallet_id: &PersistWalletId,
        password: &z00z_crypto::expert::encoding::SafePassword,
        identity: &WalletIdentity,
    ) -> Result<Self> {
        let session =
            open_wallet_store(path, wallet_id, password, identity).map_err(map_storage_unlock_error)?;

        let chain = identity
            .chain
            .parse::<ChainType>()
            .map_err(|_| KeyManagerError::InvalidParameters)?;

        let encrypted_seed = CipherSeedContainer::encrypt_wallet(
            password,
            wallet_id.0.as_bytes(),
            b"main",
            0,
            chain,
            session.opened().seed_bip39.reveal(),
            None,
        )
        .map_err(|_| KeyManagerError::StorageFailed)?;

        let mut key_manager = Self::new();
        key_manager.init_from_encrypted_seed(
            encrypted_seed,
            password,
            wallet_id.0.as_bytes(),
            b"main",
            chain,
        )?;
        Ok(key_manager)
    }
}