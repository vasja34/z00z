//! Remote scan worker contract.
//!
//! The wallet-local `WalletService::recv_range(...)` lane remains the only
//! receive authority. Remote infrastructure may fetch chunks, proof hints, or
//! resume hints, but it cannot claim ownership or mutate wallet state.

use thiserror::Error;

use crate::receiver::ScanChunk;

/// Requested checkpoint range for remote evidence fetch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteScanRange {
    /// Inclusive start height requested from the worker.
    pub start_height: u64,
    /// Inclusive end height requested from the worker.
    pub end_height: u64,
}

/// Advisory proof bytes returned with remote scan evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteScanProofHint {
    /// Checkpoint height that the proof bytes refer to.
    pub checkpoint_height: u64,
    /// Opaque proof bytes for wallet-side verification.
    pub proof_bytes: Vec<u8>,
}

/// Advisory resume information for a follow-up remote fetch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteScanResumeHint {
    /// Suggested next checkpoint height for a follow-up fetch.
    pub next_height: u64,
    /// Hash of the last checkpoint observed by the worker.
    pub last_chunk_hash: Vec<u8>,
}

/// Evidence returned by a remote worker.
///
/// These values are inputs to the wallet-local receive lane only. They are not
/// accepted-hit claims and cannot advance wallet state by themselves.
#[derive(Clone, Debug, Default)]
pub struct RemoteScanEvidence {
    /// Materialized checkpoint chunks for local scan evaluation.
    pub chunks: Vec<ScanChunk>,
    /// Advisory proof bytes bound to returned checkpoints.
    pub proof_hints: Vec<RemoteScanProofHint>,
    /// Advisory resume hint for the next fetch window.
    pub resume_hint: Option<RemoteScanResumeHint>,
}

/// Progress snapshot for remote evidence fetch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RemoteScanProgress {
    /// Completed checkpoints in the active fetch.
    pub fetched_ckpt: u64,
    /// Total checkpoints requested for the active fetch.
    pub total_ckpt: u64,
}

/// Remote scan worker errors.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RemoteScanWorkerError {
    /// Worker transport is not configured on the current backend.
    #[error("remote scan worker deferred: {0}")]
    Deferred(String),

    /// Worker could not provide evidence for the requested range.
    #[error("remote scan evidence unavailable: {0}")]
    EvidenceUnavailable(String),

    /// Chain client error.
    #[error("chain client error: {0}")]
    ChainClient(String),

    /// Proof-hint transport or parsing error.
    #[error("proof hint error: {0}")]
    ProofHint(String),

    /// Worker transport error.
    #[error("remote scan transport error: {0}")]
    Transport(String),
}

/// Remote scan worker result type.
pub type RemoteScanWorkerResult<T> = std::result::Result<T, RemoteScanWorkerError>;

/// Progress callback for remote evidence fetch.
pub type RemoteScanProgressCallback = Box<dyn Fn(RemoteScanProgress) + Send>;

/// Evidence-only remote scan worker seam.
///
/// The implemented receive authority remains `WalletService::recv_range(...)`
/// plus `StealthOutputScanner`. Callers must treat this seam as a fetch-only
/// helper that returns advisory inputs for later wallet-local validation.
pub trait RemoteScanWorker {
    /// Fetch advisory evidence for one checkpoint range.
    fn fetch_range_evidence(
        &mut self,
        range: &RemoteScanRange,
    ) -> RemoteScanWorkerResult<RemoteScanEvidence>;

    /// Report whether the worker is actively fetching evidence.
    fn is_fetching(&self) -> bool;

    /// Stop the current fetch attempt.
    fn stop_fetch(&mut self) -> RemoteScanWorkerResult<()>;

    /// Get the current fetch progress.
    fn progress(&self) -> RemoteScanProgress;

    /// Store a progress callback.
    fn set_progress_callback(&mut self, callback: RemoteScanProgressCallback);
}
