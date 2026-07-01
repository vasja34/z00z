//! Chain-facing abstractions.
//!
//! These traits define how the wallet talks to the blockchain/network layer.
//! Concrete implementations live in adapters/services as appropriate.

pub mod broadcast;
pub mod broadcast_impl;
pub mod chain_client;
pub mod chain_client_impl;
pub mod local_node_sim;
/// Canonical Stage-2 receiver-card publication record and gate.
pub mod receiver_card_record;
mod scan_engine;
mod scan_engine_impl;

pub use broadcast::{Broadcast, BroadcastError, BroadcastResult, BroadcastResultType};
pub use broadcast_impl::BroadcastImpl;
pub use chain_client::{
    BlockBytes, BlockHeaderBytes, ChainClient, ChainClientError, ChainClientResult,
    ChainNetworkInfo, ChainTxStatus,
};
pub use chain_client_impl::ChainClientImpl;
pub use local_node_sim::LocalNodeSim;
pub use receiver_card_record::{verify_receiver_card_record, ReceiverCardRecord, RevocationState};
pub use scan_engine::{
    RemoteScanEvidence, RemoteScanProgress, RemoteScanProgressCallback, RemoteScanProofHint,
    RemoteScanRange, RemoteScanResumeHint, RemoteScanWorker, RemoteScanWorkerError,
    RemoteScanWorkerResult,
};
pub use scan_engine_impl::RemoteScanWorkerImpl;
