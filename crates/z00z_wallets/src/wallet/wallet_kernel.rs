/// Wallet kernel (core state that is always present for a wallet).
///
/// Phase 1: this owns the stable wallet identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalletKernel {
    /// Stable wallet identity.
    wallet_id: WalletId,

    /// Immutable chain binding.
    chain_id: ChainId,
}

impl WalletKernel {
    /// Create a new wallet kernel.
    pub fn new(wallet_id: WalletId, chain_id: ChainId) -> Self {
        Self {
            wallet_id,
            chain_id,
        }
    }

    /// Get the stable wallet identifier.
    pub fn wallet_id(&self) -> &WalletId {
        &self.wallet_id
    }

    /// Get the immutable chain binding.
    pub fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }
}
