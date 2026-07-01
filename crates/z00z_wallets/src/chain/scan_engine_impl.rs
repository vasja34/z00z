//! Remote scan worker implementation.
//!
//! This seam is evidence-only. It may fetch checkpoint chunks and proof hints
//! from a deterministic local node simulation or a remote adapter boundary, but
//! it never replaces the authoritative wallet receive path in
//! `WalletService::recv_range(...)`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use super::{
    LocalNodeSim, RemoteScanEvidence, RemoteScanProgress, RemoteScanProgressCallback,
    RemoteScanRange, RemoteScanWorker, RemoteScanWorkerError, RemoteScanWorkerResult,
};

#[derive(Debug, Clone)]
enum RemoteScanBackend {
    Local(LocalNodeSim),
    RemoteAdapter,
}

/// Evidence-fetching remote scan worker implementation.
///
/// This type never mutates wallet state; it only materializes advisory inputs
/// for the authoritative wallet-local receive lane.
pub struct RemoteScanWorkerImpl {
    backend: RemoteScanBackend,
    is_fetching: Arc<AtomicBool>,
    progress: Arc<RwLock<RemoteScanProgress>>,
    progress_callback: Option<RemoteScanProgressCallback>,
}

impl RemoteScanWorkerImpl {
    /// Create a remote-adapter worker without a configured transport.
    pub fn new() -> Self {
        Self {
            backend: RemoteScanBackend::RemoteAdapter,
            is_fetching: Arc::new(AtomicBool::new(false)),
            progress: Arc::new(RwLock::new(RemoteScanProgress::default())),
            progress_callback: None,
        }
    }

    /// Create a worker backed by the canonical deterministic local node simulation.
    pub fn with_local_sim(node: LocalNodeSim) -> Self {
        Self {
            backend: RemoteScanBackend::Local(node),
            is_fetching: Arc::new(AtomicBool::new(false)),
            progress: Arc::new(RwLock::new(RemoteScanProgress::default())),
            progress_callback: None,
        }
    }

    fn progress_total(range: &RemoteScanRange) -> RemoteScanWorkerResult<u64> {
        if range.end_height < range.start_height {
            return Err(RemoteScanWorkerError::EvidenceUnavailable(
                "remote scan range end_height must not precede start_height".to_string(),
            ));
        }

        Ok(range.end_height - range.start_height + 1)
    }

    fn set_progress(&self, progress: RemoteScanProgress) {
        {
            let mut slot = self
                .progress
                .write()
                .expect("remote scan progress write lock");
            *slot = progress;
        }

        if let Some(callback) = &self.progress_callback {
            callback(progress);
        }
    }

    fn remote_adapter_error(&self, method: &str) -> RemoteScanWorkerError {
        match &self.backend {
            RemoteScanBackend::RemoteAdapter => RemoteScanWorkerError::Deferred(format!(
                "{method} remote scan transport adapter is not configured"
            )),
            RemoteScanBackend::Local(_) => RemoteScanWorkerError::Deferred(format!(
                "{method} remote scan adapter is unavailable on local simulation backend"
            )),
        }
    }

    fn validate_fetched_range(
        range: &RemoteScanRange,
        evidence: &RemoteScanEvidence,
    ) -> RemoteScanWorkerResult<()> {
        let total = Self::progress_total(range)?;
        if evidence.chunks.len() as u64 != total {
            return Err(RemoteScanWorkerError::EvidenceUnavailable(format!(
                "remote worker returned {} checkpoints for requested range {}..={}",
                evidence.chunks.len(),
                range.start_height,
                range.end_height
            )));
        }

        let Some(first_chunk) = evidence.chunks.first() else {
            return Err(RemoteScanWorkerError::EvidenceUnavailable(
                "remote worker returned no checkpoints for a non-empty range".to_string(),
            ));
        };

        if first_chunk.height != range.start_height {
            return Err(RemoteScanWorkerError::EvidenceUnavailable(format!(
                "remote worker started at checkpoint {} instead of {}",
                first_chunk.height, range.start_height
            )));
        }

        let mut expected_height = range.start_height;
        for chunk in &evidence.chunks {
            if chunk.height != expected_height {
                return Err(RemoteScanWorkerError::EvidenceUnavailable(format!(
                    "remote worker returned checkpoint {} while {} was expected",
                    chunk.height, expected_height
                )));
            }
            expected_height = expected_height.saturating_add(1);
        }

        Ok(())
    }
}

impl Default for RemoteScanWorkerImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl RemoteScanWorker for RemoteScanWorkerImpl {
    fn fetch_range_evidence(
        &mut self,
        range: &RemoteScanRange,
    ) -> RemoteScanWorkerResult<RemoteScanEvidence> {
        let total_ckpt = Self::progress_total(range)?;
        self.is_fetching.store(true, Ordering::SeqCst);
        self.set_progress(RemoteScanProgress {
            fetched_ckpt: 0,
            total_ckpt,
        });

        let result = match &self.backend {
            RemoteScanBackend::Local(node) => node.fetch_remote_scan_evidence(range),
            RemoteScanBackend::RemoteAdapter => {
                Err(self.remote_adapter_error("fetch_range_evidence"))
            }
        }
        .and_then(|evidence| {
            Self::validate_fetched_range(range, &evidence)?;
            Ok(evidence)
        });

        match &result {
            Ok(evidence) => self.set_progress(RemoteScanProgress {
                fetched_ckpt: evidence.chunks.len() as u64,
                total_ckpt,
            }),
            Err(_) => self.set_progress(RemoteScanProgress {
                fetched_ckpt: 0,
                total_ckpt,
            }),
        }
        self.is_fetching.store(false, Ordering::SeqCst);

        result
    }

    fn is_fetching(&self) -> bool {
        self.is_fetching.load(Ordering::SeqCst)
    }

    fn stop_fetch(&mut self) -> RemoteScanWorkerResult<()> {
        self.is_fetching.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn progress(&self) -> RemoteScanProgress {
        *self
            .progress
            .read()
            .expect("remote scan progress read lock")
    }

    fn set_progress_callback(&mut self, callback: RemoteScanProgressCallback) {
        self.progress_callback = Some(callback);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_worker_new_idle() {
        let worker = RemoteScanWorkerImpl::new();
        assert!(!worker.is_fetching());
    }

    #[test]
    fn test_remote_worker_fetch_deferred() {
        let mut worker = RemoteScanWorkerImpl::new();
        let range = RemoteScanRange {
            start_height: 0,
            end_height: 100,
        };
        let result = worker.fetch_range_evidence(&range);
        assert!(result.is_err());
    }

    #[test]
    fn test_remote_worker_fetch_ok() {
        let node = LocalNodeSim::default();
        let range = RemoteScanRange {
            start_height: 7,
            end_height: 8,
        };
        node.set_remote_scan_evidence(
            range.clone(),
            RemoteScanEvidence {
                chunks: vec![
                    crate::receiver::ScanChunk {
                        height: 7,
                        hash: vec![7u8; 32],
                        leaves: Vec::new(),
                    },
                    crate::receiver::ScanChunk {
                        height: 8,
                        hash: vec![8u8; 32],
                        leaves: Vec::new(),
                    },
                ],
                proof_hints: Vec::new(),
                resume_hint: None,
            },
        );

        let mut worker = RemoteScanWorkerImpl::with_local_sim(node);
        let evidence = worker
            .fetch_range_evidence(&range)
            .expect("local simulated evidence");
        assert_eq!(evidence.chunks.len(), 2);
        assert_eq!(evidence.chunks[0].height, 7);
        assert_eq!(evidence.chunks[1].height, 8);
        assert_eq!(
            worker.progress(),
            RemoteScanProgress {
                fetched_ckpt: 2,
                total_ckpt: 2,
            }
        );
        assert!(!worker.is_fetching());
    }

    #[test]
    fn test_worker_restart_keeps_node() {
        let node = LocalNodeSim::default();
        let range = RemoteScanRange {
            start_height: 11,
            end_height: 11,
        };
        node.set_remote_scan_evidence(
            range.clone(),
            RemoteScanEvidence {
                chunks: vec![crate::receiver::ScanChunk {
                    height: 11,
                    hash: vec![11u8; 32],
                    leaves: Vec::new(),
                }],
                proof_hints: Vec::new(),
                resume_hint: None,
            },
        );

        let mut first = RemoteScanWorkerImpl::with_local_sim(node.clone());
        let first_evidence = first
            .fetch_range_evidence(&range)
            .expect("first worker fetch");
        drop(first);

        let mut second = RemoteScanWorkerImpl::with_local_sim(node);
        let second_evidence = second
            .fetch_range_evidence(&range)
            .expect("second worker fetch");
        assert_eq!(first_evidence.chunks.len(), second_evidence.chunks.len());
        assert_eq!(second_evidence.chunks[0].height, 11);
    }

    #[test]
    fn test_remote_worker_transport_error() {
        let node = LocalNodeSim::default();
        node.fail_next_remote_scan_transport("simulated partition");

        let mut worker = RemoteScanWorkerImpl::with_local_sim(node);
        let range = RemoteScanRange {
            start_height: 3,
            end_height: 3,
        };
        let err = worker
            .fetch_range_evidence(&range)
            .expect_err("transport failure must surface");
        assert_eq!(
            err,
            RemoteScanWorkerError::Transport("simulated partition".to_string())
        );
        assert_eq!(
            worker.progress(),
            RemoteScanProgress {
                fetched_ckpt: 0,
                total_ckpt: 1,
            }
        );
    }

    #[test]
    fn test_remote_worker_stop_ok() {
        let mut worker = RemoteScanWorkerImpl::new();
        assert!(worker.stop_fetch().is_ok());
    }

    #[test]
    fn test_remote_worker_progress_zero() {
        let worker = RemoteScanWorkerImpl::new();
        assert_eq!(worker.progress(), RemoteScanProgress::default());
    }

    #[test]
    fn test_remote_worker_store_callback() {
        let mut worker = RemoteScanWorkerImpl::new();
        worker.set_progress_callback(Box::new(|_progress| {}));
        assert!(worker.progress_callback.is_some());
    }
}
