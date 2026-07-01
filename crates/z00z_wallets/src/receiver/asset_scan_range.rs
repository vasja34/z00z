/// Decision for one scan attempt under DoS policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScanDecision {
    /// Proceed with decryption attempt.
    Proceed,
    /// Defer expensive operation.
    Defer {
        /// Deferral reason.
        reason: &'static str,
        /// Retry window in seconds.
        retry_after: u64,
    },
}

/// One checkpoint worth of range-scan input.
#[derive(Clone, Debug)]
pub struct ScanChunk {
    /// Checkpoint height.
    pub height: u64,
    /// Checkpoint hash bytes.
    pub hash: Vec<u8>,
    /// Materialized runtime leaves for this checkpoint.
    pub leaves: Vec<Asset>,
}

/// Stable progress DTO for receiver range scans.
#[derive(Clone, Debug)]
pub struct ScanRangeStat {
    /// Persistable scan cursor.
    pub cursor: ScanStatePayload,
    /// Processed checkpoints in this call.
    pub done_ckpt: u64,
    /// Total checkpoints in the input range.
    pub total_ckpt: u64,
    /// Processed leaves in this call.
    pub done_leaf: u64,
    /// Total leaves in the input range.
    pub total_leaf: u64,
    /// Owned outputs found in this call.
    pub found_cnt: u64,
    /// Rejected or skipped leaves in this call.
    pub reject_cnt: u64,
}

/// Range-scan result over existing scanner primitives.
#[derive(Clone, Debug)]
pub struct ScanRangeOut {
    /// Owned outputs found across processed checkpoints.
    pub outputs: Vec<WalletStealthOutput>,
    /// Progress and counters for this range-scan call.
    pub stat: ScanRangeStat,
}

/// Range-scan orchestration errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScanRangeErr {
    /// Resume cursor does not match the provided checkpoint range.
    BadCursor,
}

impl std::fmt::Display for ScanRangeErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadCursor => write!(f, "scan range cursor mismatch"),
        }
    }
}

/// DoS protection policy for scanning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoSMitigation {
    /// Max candidate leaves processed per checkpoint.
    pub max_candidates_per_checkpoint: usize,
    /// Max decrypt attempts per checkpoint.
    pub max_decrypt_per_checkpoint: usize,
    /// Threshold that triggers deferred mode.
    pub defer_threshold: usize,
}

impl DoSMitigation {
    /// Construct policy with explicit limits.
    pub fn new(
        max_candidates_per_checkpoint: usize,
        max_decrypt_per_checkpoint: usize,
        defer_threshold: usize,
    ) -> Self {
        Self {
            max_candidates_per_checkpoint,
            max_decrypt_per_checkpoint,
            defer_threshold,
        }
    }

    /// Decide whether scanner should try decrypt.
    pub fn should_try_decrypt(&self, candidate_count: usize, decrypt_count: usize) -> ScanDecision {
        if candidate_count > self.max_candidates_per_checkpoint {
            return ScanDecision::Defer {
                reason: "candidate_limit",
                retry_after: 1,
            };
        }

        if decrypt_count >= self.max_decrypt_per_checkpoint {
            return ScanDecision::Defer {
                reason: "decrypt_limit",
                retry_after: 1,
            };
        }

        ScanDecision::Proceed
    }
}