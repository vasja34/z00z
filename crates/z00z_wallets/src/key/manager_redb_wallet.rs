// ============================================================================
// 3. WALLET REDB KEY MANAGER (WRAPPER)
// ============================================================================

/// Wallet-facing RedB key manager (thin wrapper).
#[derive(Debug, Clone)]
pub struct WalletRedbKeyManager {
    inner: RedbKeyManager,
}

/// Wallet-facing RedB key manager error type.
pub type WalletRedbKeyManagerError = RedbKeyManagerError;

impl Default for WalletRedbKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WalletRedbKeyManager {
    /// Create a new wallet-facing RedB key manager.
    pub fn new() -> Self {
        Self {
            inner: RedbKeyManager::new(),
        }
    }

    /// Create default KDF parameters with a fresh random salt.
    pub fn create_default_kdf_params(
        &self,
    ) -> std::result::Result<KdfParams, WalletRedbKeyManagerError> {
        self.inner.create_default_kdf_params()
    }

    /// Generate a fresh 32-byte master key.
    pub fn generate_master_key(&self) -> Hidden<RedbKey32> {
        self.inner.generate_master_key()
    }

    /// Wrap and encrypt the master key for storage.
    pub fn wrap_master_key(
        &self,
        wallet_id: &PersistWalletId,
        password: &SafePassword,
        master_key: &Hidden<RedbKey32>,
        kdf_params: &KdfParams,
    ) -> std::result::Result<MasterKeyRecord, WalletRedbKeyManagerError> {
        self.inner
            .wrap_master_key(wallet_id.0.as_bytes(), password, master_key, kdf_params)
    }

    /// Unwrap and decrypt a stored master key record.
    pub fn unwrap_master_key(
        &self,
        wallet_id: &PersistWalletId,
        password: &SafePassword,
        kdf_params: &KdfParams,
        record: &MasterKeyRecord,
    ) -> std::result::Result<Hidden<RedbKey32>, WalletRedbKeyManagerError> {
        self.inner
            .unwrap_master_key(wallet_id.0.as_bytes(), password, kdf_params, record)
    }

    /// Decrypt the persisted master key record.
    pub fn decrypt_master_key(
        &self,
        wallet_id: &PersistWalletId,
        password: &SafePassword,
        kdf_params: &KdfParams,
        record: &MasterKeyRecord,
    ) -> std::result::Result<Hidden<RedbKey32>, WalletRedbKeyManagerError> {
        self.unwrap_master_key(wallet_id, password, kdf_params, record)
    }

    /// Derive wallet keys (DATA/INDEX/INTEGRITY) from the master key.
    pub fn derive_wallet_keys(
        &self,
        master_key: &Hidden<RedbKey32>,
    ) -> std::result::Result<WalletDerivedKeys, WalletRedbKeyManagerError> {
        self.inner.derive_wallet_keys(master_key)
    }
}