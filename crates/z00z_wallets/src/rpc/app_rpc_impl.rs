//! App RPC implementations backed by `AppService`.

use async_trait::async_trait;
use jsonrpsee::core::RpcResult;
use std::sync::Arc;

use super::app_rpc::AppRpcServer;
use crate::rpc::error_mapping::map_wallet_error_to_rpc;
use crate::rpc::types::common::PersistWalletId;
use crate::rpc::types::wallet::{
    PersistWalletDiscovery, PersistWalletInfo, RuntimeCreateWalletResponse,
    RuntimeDeleteWalletResponse, RuntimeExportWalletResponse, RuntimeImportWalletResponse,
    RuntimeRecoverFromSeedResponse, WalletSource,
};
use crate::services::AppService;

/// App RPC service implementation.
pub struct AppRpcImpl {
    app_service: Arc<AppService>,
}

impl AppRpcImpl {
    pub fn new(app_service: Arc<AppService>) -> Self {
        Self { app_service }
    }
}

#[async_trait]
impl AppRpcServer for AppRpcImpl {
    async fn list_wallets(&self) -> RpcResult<Vec<PersistWalletInfo>> {
        self.app_service
            .list_wallets()
            .await
            .map_err(map_wallet_error_to_rpc)
    }

    async fn create_wallet(
        &self,
        name: String,
        password: String,
        seed_phrase: Option<String>,
    ) -> RpcResult<RuntimeCreateWalletResponse> {
        self.app_service
            .create_wallet(name, password, seed_phrase)
            .await
            .map_err(map_wallet_error_to_rpc)
    }

    async fn delete_wallet(
        &self,
        id: PersistWalletId,
        password: String,
    ) -> RpcResult<RuntimeDeleteWalletResponse> {
        self.app_service
            .delete_wallet(id, password)
            .await
            .map_err(map_wallet_error_to_rpc)
    }

    async fn export_wallet(
        &self,
        id: PersistWalletId,
        password: String,
    ) -> RpcResult<RuntimeExportWalletResponse> {
        self.app_service
            .export_wallet(id, password)
            .await
            .map_err(map_wallet_error_to_rpc)
    }

    async fn import_wallet(
        &self,
        data: String,
        password: String,
        name: String,
    ) -> RpcResult<RuntimeImportWalletResponse> {
        self.app_service
            .import_wallet(data, password, name)
            .await
            .map_err(map_wallet_error_to_rpc)
    }

    async fn recover_from_seed(
        &self,
        name: String,
        password: String,
        mnemonic_a: String,
        mnemonic_b: String,
        network: String,
        chain: String,
    ) -> RpcResult<RuntimeRecoverFromSeedResponse> {
        self.app_service
            .recover_from_seed(name, password, mnemonic_a, mnemonic_b, network, chain)
            .await
            .map_err(map_wallet_error_to_rpc)
    }

    async fn open_wallet_source(&self, source: WalletSource) -> RpcResult<PersistWalletDiscovery> {
        self.app_service
            .open_wallet_source(source)
            .await
            .map_err(map_wallet_error_to_rpc)
    }
}
