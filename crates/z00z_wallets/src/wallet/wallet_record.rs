/// Wallet user fields (mutable, user-controlled).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalletUserFields {
    /// User-visible wallet name.
    pub wallet_name: String,
    /// Optional user memo (free-form text).
    pub memo: Option<String>,
}

/// Wallet system metadata (mutable, system-controlled).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalletSystemMetadata {
    /// Wallet creation time (Unix time in milliseconds).
    pub created_at: u64,
    /// Last update time (Unix time in milliseconds).
    pub updated_at: u64,
}

/// Stored wallet record (single-record storage).
///
/// This aggregates kernel/profile/system into one persisted document while keeping
/// identity fields authoritative in `kernel` only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalletRecord {
    /// Stable wallet identity and immutable bindings.
    kernel: WalletKernel,
    /// User-controlled fields.
    pub user: WalletUserFields,
    /// System-controlled metadata.
    pub system: WalletSystemMetadata,
}

impl WalletRecord {
    /// Create a new wallet record.
    pub fn new(kernel: WalletKernel, user: WalletUserFields, system: WalletSystemMetadata) -> Self {
        Self {
            kernel,
            user,
            system,
        }
    }

    /// Get the immutable wallet kernel.
    pub fn kernel(&self) -> &WalletKernel {
        &self.kernel
    }

    /// Get the stable wallet identifier.
    pub fn wallet_id(&self) -> &WalletId {
        self.kernel.wallet_id()
    }

    /// Get the immutable chain binding.
    pub fn chain_id(&self) -> &ChainId {
        self.kernel.chain_id()
    }
}
