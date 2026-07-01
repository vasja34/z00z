//! Chain-related RPC types.
//!
//! This module defines types for the `chain.*` RPC namespace including:
//! - Network switching (switch_to_mainnet, testnet, devnet)
//! - Wallet-local scan orchestration (start/stop parameters, status monitoring)
//! - Wallet-local chain-tip observations

use serde::{Deserialize, Serialize};

use super::common::{PersistWalletId, RuntimeJobStatus};

/// Parameters for starting wallet-local scan orchestration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStartScanParams {
    /// Wallet to scan
    pub wallet_id: PersistWalletId,
    /// Optional starting block height (None = scan from current position)
    pub from_height: Option<u64>,
}

/// Response from starting wallet-local scan orchestration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStartScanResponse {
    #[serde(flatten)]
    pub job: RuntimeJobStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_range: Option<BlockRange>,
}

/// Block scan range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockRange {
    /// Scan start height (inclusive).
    pub from_height: u64,
    /// Scan target height (inclusive).
    pub to_height: u64,
}

/// Current wallet-local scan status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeScanStatus {
    #[serde(flatten)]
    pub job: RuntimeJobStatus,
    /// Current scanning state
    pub state: RuntimeScanState,
    /// Current block height
    pub current_height: u64,
    /// Target block height for the wallet-local tip observation.
    pub target_height: u64,
    /// Optional last wallet-local receive outcome projected onto the scan DTO.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_receive_outcome: Option<RuntimeReceiveScanOutcome>,
}

impl RuntimeScanStatus {
    /// Check if wallet scan is caught up to the latest block.
    pub fn is_scanned(&self) -> bool {
        self.current_height >= self.target_height && self.state == RuntimeScanState::Idle
    }

    /// Get progress as percentage (0-100).
    pub fn progress_percent(&self) -> f32 {
        self.job.progress_or_zero() * 100.0
    }
}

/// Wallet-local scan state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeScanState {
    /// Not scanning (idle or fully scanned)
    Idle,
    /// Actively scanning blocks
    Scanning,
    /// Scan paused (can be resumed)
    Paused,
    /// Scan failed with error
    Failed,
}

/// Last canonical wallet-local receive outcome attached to public scan status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeReceiveScanOutcome {
    /// Accepted scan with no special resume or import condition.
    Scanned,
    /// Accepted resumed scan with no imported assets.
    Resumed,
    /// Accepted origin scan with no imported assets.
    NoHit,
    /// Accepted scan imported at least one new owned asset.
    ImportedHit,
    /// Worker evidence was rejected before authoritative wallet mutation.
    WorkerEvidenceRejected,
    /// Cursor or persistence conflict blocked the receive commit.
    CursorConflict,
    /// A receive-side unsupported version blocked the receive lane.
    UnsupportedVersion,
}

/// Wallet-local chain-tip observation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeBlockInfo {
    /// Block height (index in chain)
    pub height: u64,
    /// Block hash (hex-encoded)
    pub hash: String,
    /// Block timestamp (milliseconds since Unix epoch)
    pub timestamp: u64,
    /// Number of transactions in block
    pub tx_count: u32,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use z00z_utils::codec::{Codec, JsonCodec};

    #[test]
    fn test_start_scan_params_serialization() {
        let params = RuntimeStartScanParams {
            wallet_id: PersistWalletId("test-wallet".to_string()),
            from_height: Some(1000),
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&params).unwrap();
        let deserialized: RuntimeStartScanParams = codec.deserialize(&bytes).unwrap();

        assert_eq!(deserialized.wallet_id.0, "test-wallet");
        assert_eq!(deserialized.from_height, Some(1000));
    }

    #[test]
    fn test_start_scan_response_serialization() {
        let response = RuntimeStartScanResponse {
            job: RuntimeJobStatus {
                job_id: Some("job123".to_string()),
                status: None,
                progress: Some(0.0),
                eta_seconds: Some(600),
            },
            scan_range: Some(BlockRange {
                from_height: 0,
                to_height: 1000,
            }),
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&response).unwrap();
        let deserialized: RuntimeStartScanResponse = codec.deserialize(&bytes).unwrap();

        assert_eq!(deserialized.job.job_id.as_deref(), Some("job123"));
        assert_eq!(
            deserialized.scan_range,
            Some(BlockRange {
                from_height: 0,
                to_height: 1000,
            })
        );
    }

    #[test]
    fn test_scan_status_is_scanned() {
        let scanned = RuntimeScanStatus {
            job: RuntimeJobStatus {
                job_id: None,
                status: None,
                progress: Some(1.0),
                eta_seconds: None,
            },
            state: RuntimeScanState::Idle,
            current_height: 1000,
            target_height: 1000,
            last_receive_outcome: None,
        };
        assert!(scanned.is_scanned());

        let not_scanned = RuntimeScanStatus {
            job: RuntimeJobStatus {
                job_id: None,
                status: None,
                progress: Some(0.5),
                eta_seconds: Some(300),
            },
            state: RuntimeScanState::Scanning,
            current_height: 500,
            target_height: 1000,
            last_receive_outcome: None,
        };
        assert!(!not_scanned.is_scanned());
    }

    #[test]
    fn test_scan_status_progress_percent() {
        let status = RuntimeScanStatus {
            job: RuntimeJobStatus {
                job_id: None,
                status: None,
                progress: Some(0.25),
                eta_seconds: Some(150),
            },
            state: RuntimeScanState::Scanning,
            current_height: 250,
            target_height: 1000,
            last_receive_outcome: Some(RuntimeReceiveScanOutcome::Scanned),
        };

        assert_eq!(status.progress_percent(), 25.0);
    }

    #[test]
    fn test_receive_scan_outcome_serialization() {
        let codec = JsonCodec;
        let bytes = codec
            .serialize(&RuntimeReceiveScanOutcome::WorkerEvidenceRejected)
            .unwrap();
        let serialized = String::from_utf8(bytes).unwrap();

        assert_eq!(serialized, "\"worker_evidence_rejected\"");
    }

    #[test]
    fn test_scan_status_serializes_outcome() {
        let status = RuntimeScanStatus {
            job: RuntimeJobStatus {
                job_id: Some("scan-1".to_string()),
                status: None,
                progress: Some(1.0),
                eta_seconds: None,
            },
            state: RuntimeScanState::Idle,
            current_height: 7,
            target_height: 7,
            last_receive_outcome: Some(RuntimeReceiveScanOutcome::ImportedHit),
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&status).unwrap();
        let serialized = String::from_utf8(bytes).unwrap();

        assert!(serialized.contains("\"last_receive_outcome\":\"imported_hit\""));
        assert!(status.is_scanned());
    }

    #[test]
    fn test_scan_state_equality() {
        assert_eq!(RuntimeScanState::Idle, RuntimeScanState::Idle);
        assert_ne!(RuntimeScanState::Scanning, RuntimeScanState::Paused);
    }

    #[test]
    fn test_block_info_serialization() {
        let block = RuntimeBlockInfo {
            height: 1000,
            hash: "0x1234abcd".to_string(),
            timestamp: 1703260800u64.saturating_mul(1000),
            tx_count: 42,
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&block).unwrap();
        let deserialized: RuntimeBlockInfo = codec.deserialize(&bytes).unwrap();

        assert_eq!(deserialized.height, 1000);
        assert_eq!(deserialized.hash, "0x1234abcd");
        assert_eq!(deserialized.tx_count, 42);
    }
}
