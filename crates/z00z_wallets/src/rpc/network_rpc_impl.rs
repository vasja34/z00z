//! Stub implementation for network.* RPC methods

use async_trait::async_trait;
use jsonrpsee::core::RpcResult;
use std::sync::Arc;

use super::network_rpc::NetworkRpcServer;
use crate::rpc::types::network::{RuntimeChainSettingsResponse, RuntimeSwitchChainResponse};
use crate::services::AppService;

/// Network RPC service implementation (stub)
pub struct NetworkRpcImpl {
    app_service: Arc<AppService>,
}

impl NetworkRpcImpl {
    pub fn new() -> Self {
        Self {
            app_service: Arc::new(AppService::new()),
        }
    }

    pub fn with_app_service(app_service: Arc<AppService>) -> Self {
        Self { app_service }
    }
}

impl Default for NetworkRpcImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NetworkRpcServer for NetworkRpcImpl {
    async fn switch_to_onionet(&self) -> RpcResult<RuntimeSwitchChainResponse> {
        Ok(self.app_service.switch_to_onionet())
    }

    async fn switch_to_tor(&self, enable: bool) -> RpcResult<RuntimeChainSettingsResponse> {
        Ok(self.app_service.switch_to_tor(enable))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::methods::chain_rpc::ChainRpcServer;
    use crate::rpc::methods::chain_rpc_impl::ChainRpcImpl;
    use crate::services::{AppService, WalletService};
    use crate::ChainType;
    use z00z_utils::time::SystemTimeProvider;

    #[tokio::test]
    async fn test_chain_switches_stub_response() {
        let wallet_service = Arc::new(WalletService::with_dependencies(Arc::new(
            SystemTimeProvider,
        )));
        let app_service = Arc::new(AppService::with_wallet_service(Arc::clone(&wallet_service)));
        let rpc = ChainRpcImpl::new(app_service);
        let mainnet = rpc.switch_to_mainnet().await.unwrap();
        assert!(mainnet.status.success);
        assert_eq!(mainnet.chain, ChainType::Mainnet);

        let testnet = rpc.switch_to_testnet().await.unwrap();
        assert!(testnet.status.success);
        assert_eq!(testnet.chain, ChainType::Testnet);

        let devnet = rpc.switch_to_devnet().await.unwrap();
        assert!(devnet.status.success);
        assert_eq!(devnet.chain, ChainType::Devnet);

        let network_rpc = NetworkRpcImpl::new();
        let onionet = network_rpc.switch_to_onionet().await.unwrap();
        assert!(!onionet.status.success);
    }

    #[tokio::test]
    async fn test_network_tor_stub_response() {
        let rpc = NetworkRpcImpl::new();
        let resp = rpc.switch_to_tor(true).await.unwrap();
        assert!(resp.settings.use_tor);
    }
}
