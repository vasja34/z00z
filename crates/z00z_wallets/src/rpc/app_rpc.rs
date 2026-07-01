//! App RPC method definitions.
//!
//! The `app.*` namespace owns wallet lifecycle operations (create/import/export/list/remove),
//! as described in `outputs/audit_rpc/audit_rpc_update2.csv`.

#[cfg(not(target_arch = "wasm32"))]
use jsonrpsee::{core::RpcResult, proc_macros::rpc};

#[cfg(not(target_arch = "wasm32"))]
use super::super::types::common::PersistWalletId;

#[cfg(not(target_arch = "wasm32"))]
use super::super::types::wallet::{
    PersistWalletDiscovery, PersistWalletInfo, RuntimeCreateWalletResponse,
    RuntimeDeleteWalletResponse, RuntimeExportWalletResponse, RuntimeImportWalletResponse,
    RuntimeRecoverFromSeedResponse, WalletSource,
};

/// App RPC trait defining application-level operations.
#[cfg(not(target_arch = "wasm32"))]
#[rpc(server, client)]
pub trait AppRpc {
    /// List all wallets owned by the app.
    #[method(name = "app.wallet.list_wallets")]
    async fn list_wallets(&self) -> RpcResult<Vec<PersistWalletInfo>>;

    /// Create a new wallet.
    #[method(name = "app.wallet.create_wallet")]
    async fn create_wallet(
        &self,
        name: String,
        password: String,
        seed_phrase: Option<String>,
    ) -> RpcResult<RuntimeCreateWalletResponse>;

    /// Delete a wallet permanently.
    #[method(name = "app.wallet.delete_wallet")]
    async fn delete_wallet(
        &self,
        id: PersistWalletId,
        password: String,
    ) -> RpcResult<RuntimeDeleteWalletResponse>;

    /// Export wallet data.
    #[method(name = "app.wallet.export_wallet")]
    async fn export_wallet(
        &self,
        id: PersistWalletId,
        password: String,
    ) -> RpcResult<RuntimeExportWalletResponse>;

    /// Import an existing wallet from a backup payload.
    #[method(name = "app.wallet.import_wallet")]
    async fn import_wallet(
        &self,
        data: String,
        password: String,
        name: String,
    ) -> RpcResult<RuntimeImportWalletResponse>;

    /// Recover a wallet from a seed phrase.
    #[method(name = "app.wallet.recover_from_seed")]
    async fn recover_from_seed(
        &self,
        name: String,
        password: String,
        mnemonic_a: String,
        mnemonic_b: String,
        network: String,
        chain: String,
    ) -> RpcResult<RuntimeRecoverFromSeedResponse>;

    /// Open an existing wallet file source and return discovery metadata.
    #[method(name = "app.wallet.open_wallet_source")]
    async fn open_wallet_source(&self, source: WalletSource) -> RpcResult<PersistWalletDiscovery>;
}
