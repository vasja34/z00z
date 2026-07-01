//! Blockchain RPC client abstraction.

use thiserror::Error;

/// Opaque serialized block bytes.
pub type BlockBytes = Vec<u8>;

/// Opaque serialized block header bytes.
pub type BlockHeaderBytes = Vec<u8>;

/// Chain client errors.
#[derive(Debug, Error)]
pub enum ChainClientError {
    /// Connection to the node failed.
    #[error("connection failed: {0}")]
    Connection(String),

    /// RPC call failed.
    #[error("RPC error: {0}")]
    Rpc(String),

    /// Transaction was rejected before entering the canonical mempool path.
    #[error("transaction rejected: {0}")]
    Rejected(String),

    /// Requested block does not exist.
    #[error("block not found at height: {0}")]
    BlockNotFound(u64),

    /// Requested transaction does not exist.
    #[error("transaction not found: {0}")]
    TxNotFound(String),

    /// Network request failed.
    #[error("network error: {0}")]
    Network(String),
}

/// Chain client result type.
pub type ChainClientResult<T> = std::result::Result<T, ChainClientError>;

/// Transaction status as reported by the node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainTxStatus {
    /// In mempool / unconfirmed.
    Pending,
    /// Confirmed in a block.
    Confirmed,
    /// Rejected, invalid, or dropped.
    Failed,
    /// Replaced by another transaction before confirmation.
    Replaced,
    /// Lost confirmation due to a reorg or rollback.
    Reorged,
}

/// Network information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainNetworkInfo {
    /// Chain identifier.
    pub chain_id: String,
    /// Node version string.
    pub version: String,
    /// Number of connected peers.
    pub peer_count: u32,
}

/// Chain client trait.
///
/// Communicates with a blockchain node through the canonical wallet chain seam.
///
/// NOTE: Until protocol-level block/transaction types are stabilized in `z00z_core`, this
/// abstraction uses opaque byte payloads.
///
/// Current Phase 062 contract:
/// - local deterministic simulation is a live backend for this trait
/// - real remote-node transport may remain adapter-only
/// - the trait stays side-effect free and free of logging dependencies
pub trait ChainClient {
    /// Get current chain tip height.
    fn get_tip_height(&self) -> ChainClientResult<u64>;

    /// Get block by height (opaque bytes).
    fn get_block(&self, height: u64) -> ChainClientResult<BlockBytes>;

    /// Get block header by height (opaque bytes).
    fn get_header(&self, height: u64) -> ChainClientResult<BlockHeaderBytes>;

    /// Submit a serialized transaction to the mempool.
    fn submit_transaction(&self, tx_bytes: &[u8]) -> ChainClientResult<String>;

    /// Check transaction status.
    fn get_transaction_status(&self, tx_hash: &str) -> ChainClientResult<ChainTxStatus>;

    /// Get network info.
    ///
    /// Note: Any logging/observability for this call must happen in services/middleware.
    fn get_network_info(&self) -> ChainClientResult<ChainNetworkInfo>;
}
