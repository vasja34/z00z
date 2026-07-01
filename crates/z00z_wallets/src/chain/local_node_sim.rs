//! Deterministic local node simulation for wallet chain tests.

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use super::scan_engine::{
    RemoteScanEvidence, RemoteScanRange, RemoteScanWorkerError, RemoteScanWorkerResult,
};
use super::{
    BlockBytes, BlockHeaderBytes, ChainClientError, ChainClientResult, ChainNetworkInfo,
    ChainTxStatus,
};
use crate::tx::{build_tx_package_digest, TxPackage};
use z00z_utils::codec::{Codec, JsonCodec};

#[derive(Debug)]
struct LocalNodeState {
    tip_height: u64,
    blocks: BTreeMap<u64, BlockBytes>,
    headers: BTreeMap<u64, BlockHeaderBytes>,
    tx_statuses: BTreeMap<String, ChainTxStatus>,
    network_info: ChainNetworkInfo,
    next_network_info_error: Option<String>,
    next_submit_error: Option<ChainClientError>,
    fee_per_weight: u64,
    remote_scan_evidence: BTreeMap<(u64, u64), RemoteScanEvidence>,
    next_remote_scan_error: Option<RemoteScanWorkerError>,
}

/// Canonical local wallet-node simulator used by `ChainClientImpl`.
///
/// This keeps Phase 062 chain behavior on a single live path:
/// - `ChainClientImpl` is the public client surface
/// - `LocalNodeSim` is the deterministic in-process backend for tests and local simulation
/// - real remote-node transport remains an explicit adapter-only seam
#[derive(Debug, Clone)]
pub struct LocalNodeSim {
    state: Arc<RwLock<LocalNodeState>>,
}

impl Default for LocalNodeSim {
    fn default() -> Self {
        Self::new(
            ChainNetworkInfo {
                chain_id: "devnet".to_string(),
                version: "local-node-sim-v1".to_string(),
                peer_count: 1,
            },
            1,
        )
    }
}

impl LocalNodeSim {
    /// Create a local node simulator with fixed network info and initial fee rate.
    pub fn new(network_info: ChainNetworkInfo, fee_per_weight: u64) -> Self {
        Self {
            state: Arc::new(RwLock::new(LocalNodeState {
                tip_height: 0,
                blocks: BTreeMap::new(),
                headers: BTreeMap::new(),
                tx_statuses: BTreeMap::new(),
                network_info,
                next_network_info_error: None,
                next_submit_error: None,
                fee_per_weight,
                remote_scan_evidence: BTreeMap::new(),
                next_remote_scan_error: None,
            })),
        }
    }

    /// Insert or replace a block/header pair and advance tip height when needed.
    pub fn insert_block(
        &self,
        height: u64,
        block_bytes: BlockBytes,
        header_bytes: BlockHeaderBytes,
    ) {
        let mut state = self.state.write().expect("local node sim write lock");
        state.tip_height = state.tip_height.max(height);
        state.blocks.insert(height, block_bytes);
        state.headers.insert(height, header_bytes);
    }

    /// Mark a submitted transaction as confirmed and advance the simulated tip.
    pub fn confirm_transaction(&self, tx_hash: &str, block_height: u64) -> ChainClientResult<()> {
        let mut state = self.state.write().expect("local node sim write lock");
        let status = state
            .tx_statuses
            .get_mut(tx_hash)
            .ok_or_else(|| ChainClientError::TxNotFound(tx_hash.to_string()))?;
        *status = ChainTxStatus::Confirmed;
        state.tip_height = state.tip_height.max(block_height);
        state
            .blocks
            .entry(block_height)
            .or_insert_with(|| format!("sim-block:{block_height}:{tx_hash}").into_bytes());
        state
            .headers
            .entry(block_height)
            .or_insert_with(|| format!("sim-header:{block_height}").into_bytes());
        Ok(())
    }

    /// Mark a submitted transaction as failed.
    pub fn fail_transaction(&self, tx_hash: &str) -> ChainClientResult<()> {
        let mut state = self.state.write().expect("local node sim write lock");
        let status = state
            .tx_statuses
            .get_mut(tx_hash)
            .ok_or_else(|| ChainClientError::TxNotFound(tx_hash.to_string()))?;
        *status = ChainTxStatus::Failed;
        Ok(())
    }

    /// Mark a submitted transaction as replaced before confirmation.
    pub fn replace_transaction(&self, tx_hash: &str) -> ChainClientResult<()> {
        let mut state = self.state.write().expect("local node sim write lock");
        let status = state
            .tx_statuses
            .get_mut(tx_hash)
            .ok_or_else(|| ChainClientError::TxNotFound(tx_hash.to_string()))?;
        *status = ChainTxStatus::Replaced;
        Ok(())
    }

    /// Mark a submitted transaction as reorged after a prior local commit path.
    pub fn reorg_transaction(&self, tx_hash: &str) -> ChainClientResult<()> {
        let mut state = self.state.write().expect("local node sim write lock");
        let status = state
            .tx_statuses
            .get_mut(tx_hash)
            .ok_or_else(|| ChainClientError::TxNotFound(tx_hash.to_string()))?;
        *status = ChainTxStatus::Reorged;
        Ok(())
    }

    /// Change the simulated network info returned by the node.
    pub fn set_network_info(&self, network_info: ChainNetworkInfo) {
        let mut state = self.state.write().expect("local node sim write lock");
        state.network_info = network_info;
    }

    /// Inject a one-shot network-info failure.
    pub fn fail_next_network_info(&self, message: impl Into<String>) {
        let mut state = self.state.write().expect("local node sim write lock");
        state.next_network_info_error = Some(message.into());
    }

    /// Inject a one-shot transient submit failure.
    pub fn fail_next_submit_network(&self, message: impl Into<String>) {
        let mut state = self.state.write().expect("local node sim write lock");
        state.next_submit_error = Some(ChainClientError::Network(message.into()));
    }

    /// Inject a one-shot submit rejection.
    pub fn reject_next_submit(&self, message: impl Into<String>) {
        let mut state = self.state.write().expect("local node sim write lock");
        state.next_submit_error = Some(ChainClientError::Rejected(message.into()));
    }

    /// Update the simulated live fee rate.
    pub fn set_fee_per_weight(&self, fee_per_weight: u64) {
        let mut state = self.state.write().expect("local node sim write lock");
        state.fee_per_weight = fee_per_weight;
    }

    /// Register one deterministic remote-scan evidence window for the worker seam.
    pub fn set_remote_scan_evidence(&self, range: RemoteScanRange, evidence: RemoteScanEvidence) {
        let mut state = self.state.write().expect("local node sim write lock");
        state
            .remote_scan_evidence
            .insert((range.start_height, range.end_height), evidence);
    }

    /// Inject a one-shot remote worker transport failure.
    pub fn fail_next_remote_scan_transport(&self, message: impl Into<String>) {
        let mut state = self.state.write().expect("local node sim write lock");
        state.next_remote_scan_error = Some(RemoteScanWorkerError::Transport(message.into()));
    }

    /// Inject a one-shot remote worker error.
    pub fn fail_next_remote_scan_fetch(&self, error: RemoteScanWorkerError) {
        let mut state = self.state.write().expect("local node sim write lock");
        state.next_remote_scan_error = Some(error);
    }

    /// Read the simulated live fee rate.
    pub fn get_fee_per_weight(&self) -> ChainClientResult<u64> {
        let state = self.state.read().expect("local node sim read lock");
        Ok(state.fee_per_weight)
    }

    pub(crate) fn get_tip_height(&self) -> ChainClientResult<u64> {
        let state = self.state.read().expect("local node sim read lock");
        Ok(state.tip_height)
    }

    pub(crate) fn get_block(&self, height: u64) -> ChainClientResult<BlockBytes> {
        let state = self.state.read().expect("local node sim read lock");
        state
            .blocks
            .get(&height)
            .cloned()
            .ok_or(ChainClientError::BlockNotFound(height))
    }

    pub(crate) fn get_header(&self, height: u64) -> ChainClientResult<BlockHeaderBytes> {
        let state = self.state.read().expect("local node sim read lock");
        state
            .headers
            .get(&height)
            .cloned()
            .ok_or(ChainClientError::BlockNotFound(height))
    }

    pub(crate) fn submit_transaction(&self, tx_bytes: &[u8]) -> ChainClientResult<String> {
        if tx_bytes.is_empty() {
            return Err(ChainClientError::Rpc(
                "transaction bytes must not be empty".to_string(),
            ));
        }

        let mut state = self.state.write().expect("local node sim write lock");
        if let Some(error) = state.next_submit_error.take() {
            return Err(error);
        }
        let tx_hash = match JsonCodec.deserialize::<TxPackage>(tx_bytes) {
            Ok(package) => {
                let expected = build_tx_package_digest(
                    &package.kind,
                    &package.package_type,
                    package.version,
                    package.chain_id,
                    &package.chain_type,
                    &package.chain_name,
                    &package.tx,
                )
                .map_err(|error| {
                    ChainClientError::Rejected(format!(
                        "local simulation tx digest build failed: {error}"
                    ))
                })?;
                if package.tx_digest_hex != expected {
                    return Err(ChainClientError::Rejected(
                        "tx_digest_hex does not match payload".to_string(),
                    ));
                }
                package.tx_digest_hex
            }
            Err(_) => hex::encode(blake3::hash(tx_bytes).as_bytes()),
        };
        state
            .tx_statuses
            .entry(tx_hash.clone())
            .or_insert(ChainTxStatus::Pending);
        Ok(tx_hash)
    }

    pub(crate) fn get_transaction_status(&self, tx_hash: &str) -> ChainClientResult<ChainTxStatus> {
        let state = self.state.read().expect("local node sim read lock");
        state
            .tx_statuses
            .get(tx_hash)
            .copied()
            .ok_or_else(|| ChainClientError::TxNotFound(tx_hash.to_string()))
    }

    pub(crate) fn get_network_info(&self) -> ChainClientResult<ChainNetworkInfo> {
        let mut state = self.state.write().expect("local node sim write lock");
        if let Some(message) = state.next_network_info_error.take() {
            return Err(ChainClientError::Network(message));
        }
        Ok(state.network_info.clone())
    }

    pub(crate) fn fetch_remote_scan_evidence(
        &self,
        range: &RemoteScanRange,
    ) -> RemoteScanWorkerResult<RemoteScanEvidence> {
        let mut state = self.state.write().expect("local node sim write lock");
        if let Some(error) = state.next_remote_scan_error.take() {
            return Err(error);
        }

        state
            .remote_scan_evidence
            .get(&(range.start_height, range.end_height))
            .cloned()
            .ok_or_else(|| {
                RemoteScanWorkerError::EvidenceUnavailable(format!(
                    "missing local simulated remote scan evidence for range {}..={}",
                    range.start_height, range.end_height
                ))
            })
    }
}
