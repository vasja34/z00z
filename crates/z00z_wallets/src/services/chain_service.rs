//! Chain service for managing wallet-local chain runtime state.

use std::collections::BTreeSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use z00z_utils::time::{SystemTimeProvider, TimeProvider};

use crate::key::Bip44Path;
use crate::rpc::types::chain::{
    BlockRange, RuntimeBlockInfo, RuntimeScanState, RuntimeScanStatus, RuntimeStartScanParams,
    RuntimeStartScanResponse,
};
use crate::rpc::types::common::{PersistWalletId, RuntimeJobStatus};
use crate::rpc::types::network::RuntimeChainSettings;
use crate::ChainType;

#[derive(Debug, Clone)]
struct ScanJobState {
    wallet_id: PersistWalletId,
    status: RuntimeScanStatus,
}

fn compute_tip_height(from_height: u64) -> u64 {
    from_height.saturating_add(10_000)
}

/// Chain service for managing active chain settings and wallet-local scan state.
pub struct ChainService {
    settings: Arc<RwLock<RuntimeChainSettings>>,
    time_provider: Arc<dyn TimeProvider>,
    scan_jobs: Arc<RwLock<Vec<ScanJobState>>>,
    used_paths: Arc<RwLock<BTreeSet<Bip44Path>>>,
}

impl ChainService {
    /// Create new chain service with default settings (Devnet).
    pub fn new() -> Self {
        Self::with_dependencies(Arc::new(SystemTimeProvider))
    }

    /// Create chain service with an injected time provider.
    pub fn with_dependencies(time_provider: Arc<dyn TimeProvider>) -> Self {
        Self {
            settings: Arc::new(RwLock::new(RuntimeChainSettings {
                chain_type: ChainType::Devnet,
                rpc_endpoint: "http://localhost:1234".to_string(),
                use_tor: false,
            })),
            time_provider,
            scan_jobs: Arc::new(RwLock::new(Vec::new())),
            used_paths: Arc::new(RwLock::new(BTreeSet::new())),
        }
    }

    /// Create chain service with custom settings.
    pub fn with_settings(settings: RuntimeChainSettings) -> Self {
        Self::with_settings_and_dependencies(settings, Arc::new(SystemTimeProvider))
    }

    /// Create chain service with custom settings and an injected time provider.
    pub fn with_settings_and_dependencies(
        settings: RuntimeChainSettings,
        time_provider: Arc<dyn TimeProvider>,
    ) -> Self {
        Self {
            settings: Arc::new(RwLock::new(settings)),
            time_provider,
            scan_jobs: Arc::new(RwLock::new(Vec::new())),
            used_paths: Arc::new(RwLock::new(BTreeSet::new())),
        }
    }

    /// Configure which derived paths are treated as already used by recovery.
    pub async fn set_used_paths(&self, paths: Vec<Bip44Path>) {
        let mut store = self.used_paths.write().await;
        store.clear();
        store.extend(paths);
    }

    /// Check whether a derived path is treated as already used by recovery.
    pub async fn is_path_used(&self, path: Bip44Path) -> bool {
        let store = self.used_paths.read().await;
        store.contains(&path)
    }

    /// Get shared reference to settings (for RPC impl).
    pub fn get_chain_settings(&self) -> Arc<RwLock<RuntimeChainSettings>> {
        Arc::clone(&self.settings)
    }

    /// Switch to mainnet.
    pub async fn switch_to_mainnet(&self) -> Result<ChainType, ChainServiceError> {
        let mut settings = self.settings.write().await;
        settings.chain_type = ChainType::Mainnet;
        settings.rpc_endpoint = "https://mainnet.z00z.io".to_string();
        Ok(ChainType::Mainnet)
    }

    /// Switch to testnet.
    pub async fn switch_to_testnet(&self) -> Result<ChainType, ChainServiceError> {
        let mut settings = self.settings.write().await;
        settings.chain_type = ChainType::Testnet;
        settings.rpc_endpoint = "https://testnet.z00z.io".to_string();
        Ok(ChainType::Testnet)
    }

    /// Switch to devnet (local development).
    pub async fn switch_to_devnet(&self) -> Result<ChainType, ChainServiceError> {
        let mut settings = self.settings.write().await;
        settings.chain_type = ChainType::Devnet;
        settings.rpc_endpoint = "http://localhost:1234".to_string();
        Ok(ChainType::Devnet)
    }

    /// Get current active chain type.
    pub async fn get_active_chain(&self) -> ChainType {
        self.settings.read().await.chain_type
    }

    /// Get current RPC endpoint.
    pub async fn get_rpc_endpoint(&self) -> String {
        self.settings.read().await.rpc_endpoint.clone()
    }

    // ------------------------------------------------------------------------
    // Wallet-local scan API
    // ------------------------------------------------------------------------

    /// Start a wallet-local scan job for a wallet.
    ///
    /// Tracks scan jobs in-memory and returns deterministic progress for local
    /// scan orchestration only.
    pub async fn start_local_scan(
        &self,
        params: RuntimeStartScanParams,
    ) -> RuntimeStartScanResponse {
        let from_height = params.from_height.unwrap_or(0);
        let target_height = compute_tip_height(from_height);

        let job_id = format!(
            "scan_{}_{}",
            params.wallet_id.0,
            uuid::Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .unwrap_or("unknown")
        );

        let scan_range = params.from_height.map(|from_height| BlockRange {
            from_height,
            to_height: target_height,
        });

        let job = RuntimeJobStatus {
            job_id: Some(job_id.clone()),
            status: None,
            progress: Some(0.0),
            eta_seconds: Some(600),
        };

        let initial_status = RuntimeScanStatus {
            job: job.clone(),
            state: RuntimeScanState::Scanning,
            current_height: from_height,
            target_height,
            last_receive_outcome: None,
        };

        let mut jobs = self.scan_jobs.write().await;
        jobs.retain(|j| j.wallet_id != params.wallet_id);
        jobs.push(ScanJobState {
            wallet_id: params.wallet_id.clone(),
            status: initial_status.clone(),
        });

        RuntimeStartScanResponse { job, scan_range }
    }

    /// Stop (pause) an active wallet-local scan for a wallet.
    ///
    /// Updates process-local in-memory scan state.
    pub async fn stop_local_scan(&self, wallet_id: PersistWalletId) {
        let mut jobs = self.scan_jobs.write().await;
        if let Some(job) = jobs.iter_mut().find(|j| j.wallet_id == wallet_id) {
            job.status.state = RuntimeScanState::Paused;
            job.status.job.eta_seconds = None;
        }
    }

    /// Get the current wallet-local scan status for a wallet.
    ///
    /// Returns process-local in-memory status or an `Idle` default when no scan
    /// is known.
    pub async fn get_local_scan_status(&self, wallet_id: PersistWalletId) -> RuntimeScanStatus {
        let jobs = self.scan_jobs.read().await;
        if let Some(job) = jobs.iter().find(|j| j.wallet_id == wallet_id) {
            return job.status.clone();
        }

        RuntimeScanStatus {
            job: RuntimeJobStatus {
                job_id: None,
                status: None,
                progress: Some(1.0),
                eta_seconds: None,
            },
            state: RuntimeScanState::Idle,
            current_height: 0,
            target_height: 0,
            last_receive_outcome: None,
        }
    }

    /// Get the current wallet-local chain-tip observation.
    ///
    /// Derives a local tip observation from the largest known scan target
    /// height, or a default value when no scan exists.
    pub async fn get_local_scan_tip(&self) -> RuntimeBlockInfo {
        let jobs = self.scan_jobs.read().await;
        let height = jobs
            .iter()
            .map(|j| j.status.target_height)
            .max()
            .unwrap_or(10_000);

        RuntimeBlockInfo {
            height,
            hash: format!("0x{height:064x}"),
            timestamp: self.time_provider.compat_unix_timestamp_millis(),
            tx_count: 1,
        }
    }
}

impl Default for ChainService {
    fn default() -> Self {
        Self::new()
    }
}

/// Chain service errors.
#[derive(Debug, thiserror::Error)]
pub enum ChainServiceError {
    /// Invalid network type was provided
    #[error("Invalid network type")]
    InvalidChainType,

    /// Network switch operation failed
    #[error("Chain switch failed: {0}")]
    SwitchFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_is_devnet() {
        let service = ChainService::new();
        assert_eq!(service.get_active_chain().await, ChainType::Devnet);
    }

    #[tokio::test]
    async fn test_switch_to_mainnet() {
        let service = ChainService::new();
        let result = service.switch_to_mainnet().await.unwrap();
        assert_eq!(result, ChainType::Mainnet);
        assert_eq!(service.get_active_chain().await, ChainType::Mainnet);
        assert_eq!(service.get_rpc_endpoint().await, "https://mainnet.z00z.io");
    }

    #[tokio::test]
    async fn test_switch_to_testnet() {
        let service = ChainService::new();
        let result = service.switch_to_testnet().await.unwrap();
        assert_eq!(result, ChainType::Testnet);
        assert_eq!(service.get_active_chain().await, ChainType::Testnet);
        assert_eq!(service.get_rpc_endpoint().await, "https://testnet.z00z.io");
    }

    #[tokio::test]
    async fn test_switch_to_devnet() {
        let service = ChainService::new();
        service.switch_to_mainnet().await.unwrap();
        let result = service.switch_to_devnet().await.unwrap();
        assert_eq!(result, ChainType::Devnet);
        assert_eq!(service.get_active_chain().await, ChainType::Devnet);
    }
}
