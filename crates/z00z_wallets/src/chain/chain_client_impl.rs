//! ChainClient implementation.
//!
//! Phase 062 implementation notes:
//! - Local deterministic node simulation is live for Phase 062 execution
//! - Real remote-node transport remains an explicit adapter-only seam
//! - Logging is handled outside core traits/interfaces

use super::{
    BlockBytes, BlockHeaderBytes, ChainClient, ChainClientError, ChainClientResult,
    ChainNetworkInfo, ChainTxStatus, LocalNodeSim,
};

#[derive(Debug, Clone)]
enum ChainBackend {
    Local(LocalNodeSim),
    Remote { node_url: String },
}

/// Default ChainClient implementation.
///
/// Communicates with a deterministic local node simulation or a remote-node
/// adapter seam.
#[derive(Debug, Clone)]
pub struct ChainClientImpl {
    backend: ChainBackend,
}

impl ChainClientImpl {
    /// Create a remote-adapter client.
    ///
    /// # Arguments
    /// - `node_url` - RPC endpoint URL (e.g., "http://localhost:18081")
    pub fn new(node_url: String) -> Self {
        Self {
            backend: ChainBackend::Remote { node_url },
        }
    }

    /// Create a client backed by the canonical deterministic local node simulation.
    pub fn with_local_sim(node: LocalNodeSim) -> Self {
        Self {
            backend: ChainBackend::Local(node),
        }
    }

    fn remote_adapter_error(&self, method: &str) -> ChainClientError {
        match &self.backend {
            ChainBackend::Remote { node_url } => ChainClientError::Rpc(format!(
                "{method} remote node adapter is not configured for {node_url}"
            )),
            ChainBackend::Local(_) => ChainClientError::Rpc(format!(
                "{method} remote node adapter is unavailable on local simulation backend"
            )),
        }
    }
}

impl ChainClient for ChainClientImpl {
    fn get_tip_height(&self) -> ChainClientResult<u64> {
        match &self.backend {
            ChainBackend::Local(node) => node.get_tip_height(),
            ChainBackend::Remote { .. } => Err(self.remote_adapter_error("get_tip_height")),
        }
    }

    fn get_block(&self, height: u64) -> ChainClientResult<BlockBytes> {
        match &self.backend {
            ChainBackend::Local(node) => node.get_block(height),
            ChainBackend::Remote { .. } => Err(self.remote_adapter_error("get_block")),
        }
    }

    fn get_header(&self, height: u64) -> ChainClientResult<BlockHeaderBytes> {
        match &self.backend {
            ChainBackend::Local(node) => node.get_header(height),
            ChainBackend::Remote { .. } => Err(self.remote_adapter_error("get_header")),
        }
    }

    fn submit_transaction(&self, tx_bytes: &[u8]) -> ChainClientResult<String> {
        match &self.backend {
            ChainBackend::Local(node) => node.submit_transaction(tx_bytes),
            ChainBackend::Remote { .. } => Err(self.remote_adapter_error("submit_transaction")),
        }
    }

    fn get_transaction_status(&self, tx_hash: &str) -> ChainClientResult<ChainTxStatus> {
        match &self.backend {
            ChainBackend::Local(node) => node.get_transaction_status(tx_hash),
            ChainBackend::Remote { .. } => Err(self.remote_adapter_error("get_transaction_status")),
        }
    }

    fn get_network_info(&self) -> ChainClientResult<ChainNetworkInfo> {
        match &self.backend {
            ChainBackend::Local(node) => node.get_network_info(),
            ChainBackend::Remote { .. } => Err(self.remote_adapter_error("get_network_info")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_remote_client() {
        let client = ChainClientImpl::new("http://localhost:18081".to_string());
        assert!(matches!(
            client.backend,
            ChainBackend::Remote { ref node_url } if node_url == "http://localhost:18081"
        ));
    }

    #[test]
    fn local_tip_height() {
        let node = LocalNodeSim::default();
        node.insert_block(42, b"block-42".to_vec(), b"header-42".to_vec());

        let client = ChainClientImpl::with_local_sim(node);
        assert_eq!(client.get_tip_height().expect("tip"), 42);
    }

    #[test]
    fn local_round_trip() {
        let node = LocalNodeSim::default();
        node.insert_block(7, b"block-7".to_vec(), b"header-7".to_vec());
        let client = ChainClientImpl::with_local_sim(node.clone());

        assert_eq!(client.get_block(7).expect("block"), b"block-7".to_vec());
        assert_eq!(client.get_header(7).expect("header"), b"header-7".to_vec());

        let tx_hash = client.submit_transaction(b"hello chain").expect("submit");
        assert_eq!(
            client.get_transaction_status(&tx_hash).expect("pending"),
            ChainTxStatus::Pending
        );

        node.confirm_transaction(&tx_hash, 8).expect("confirm");
        assert_eq!(
            client.get_transaction_status(&tx_hash).expect("confirmed"),
            ChainTxStatus::Confirmed
        );
    }

    #[test]
    fn local_typed_errors() {
        let node = LocalNodeSim::default();
        let client = ChainClientImpl::with_local_sim(node.clone());

        assert!(matches!(
            client.get_block(100).unwrap_err(),
            ChainClientError::BlockNotFound(100)
        ));
        assert!(matches!(
            client.get_transaction_status("missing").unwrap_err(),
            ChainClientError::TxNotFound(tx_hash) if tx_hash == "missing"
        ));

        node.fail_next_network_info("network partition");
        assert!(matches!(
            client.get_network_info().unwrap_err(),
            ChainClientError::Network(message) if message == "network partition"
        ));
    }

    #[test]
    fn remote_adapter_error() {
        let client = ChainClientImpl::new("http://localhost:18081".to_string());
        let err = client.get_tip_height().unwrap_err();
        assert!(
            matches!(err, ChainClientError::Rpc(message) if message.contains("remote node adapter"))
        );
    }
}
