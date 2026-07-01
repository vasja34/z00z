impl AppService {
    // ------------------------------------------------------------------------
    // App-owned network API (network.*)
    // ------------------------------------------------------------------------

    /// Configure OnionNet transport.
    ///
    /// Phase 1: deterministic placeholder that reaches the core app.
    pub fn switch_to_onionet(&self) -> RuntimeSwitchChainResponse {
        let _ = self.core_app.time_provider.compat_unix_timestamp_millis();
        let chain = self.core_app.configure_onionet();
        RuntimeSwitchChainResponse {
            status: RuntimeOperationStatus {
                success: false,
                message: String::new(),
            },
            chain,
        }
    }

    /// Enable or disable Tor transport.
    ///
    /// Phase 1: deterministic placeholder that reaches the core app.
    pub fn switch_to_tor(&self, enable: bool) -> RuntimeChainSettingsResponse {
        let _ = self.core_app.time_provider.compat_unix_timestamp_millis();
        let _ = self.core_app.configure_tor(enable);
        RuntimeChainSettingsResponse {
            settings: RuntimeChainSettings {
                chain_type: ChainType::Devnet,
                rpc_endpoint: "http://localhost:1234".to_string(),
                use_tor: enable,
            },
        }
    }

    // ------------------------------------------------------------------------
    // App-owned chain API (chain.*)
    // ------------------------------------------------------------------------

    /// Switch the active chain network to mainnet.
    pub async fn switch_chain_to_mainnet(&self) -> Result<ChainType, ChainServiceError> {
        let _ = self.core_app.switch_to_mainnet();
        self.core_app.chain_client.switch_to_mainnet().await
    }

    /// Switch the active chain network to testnet.
    pub async fn switch_chain_to_testnet(&self) -> Result<ChainType, ChainServiceError> {
        let _ = self.core_app.switch_to_testnet();
        self.core_app.chain_client.switch_to_testnet().await
    }

    /// Switch the active chain network to devnet.
    pub async fn switch_chain_to_devnet(&self) -> Result<ChainType, ChainServiceError> {
        let _ = self.core_app.switch_to_devnet();
        self.core_app.chain_client.switch_to_devnet().await
    }

    /// Start wallet-local scan orchestration for the given wallet.
    pub async fn start_local_scan(
        &self,
        params: RuntimeStartScanParams,
    ) -> RuntimeStartScanResponse {
        let _ = self.core_app.start_local_scan();
        self.core_app.chain_client.start_local_scan(params).await
    }

    /// Stop (pause) any active wallet-local scan for the given wallet.
    pub async fn stop_local_scan(&self, wallet_id: PersistWalletId) {
        let _ = self.core_app.stop_local_scan();
        self.core_app.chain_client.stop_local_scan(wallet_id).await;
    }

    /// Get current wallet-local scan status for the given wallet.
    pub async fn get_local_scan_status(&self, wallet_id: PersistWalletId) -> RuntimeScanStatus {
        let _ = self.core_app.get_local_scan_status();
        let mut status = self
            .core_app
            .chain_client
            .get_local_scan_status(wallet_id.clone())
            .await;
        status.last_receive_outcome = self.wallets.last_receive_scan_outcome(&wallet_id).await;
        status
    }

    /// Get the current wallet-local chain-tip observation.
    pub async fn get_local_scan_tip(&self) -> RuntimeBlockInfo {
        let _ = self.core_app.get_local_scan_tip_height();
        self.core_app.chain_client.get_local_scan_tip().await
    }
}
