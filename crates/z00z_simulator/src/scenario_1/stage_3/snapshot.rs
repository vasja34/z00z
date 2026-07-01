use super::{write_file, Codec, Deserialize, Error, JsonCodec, Path, Serialize};

/// Minimal row snapshot used to compare claim distribution output.
#[derive(Debug, Serialize)]
pub(crate) struct ClaimRow {
    pub asset_id: String,
    pub symbol: String,
    pub class: String,
    pub serial_id: u32,
    pub amount: u64,
}

/// Per-actor totals that feed the Stage 3 reconciliation check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorSnapItem {
    pub name: String,
    pub assets_count: usize,
    pub total_amount: u64,
    pub unique_terminal_ids: usize,
}

/// Captures the Stage 3 distribution outcome for integrity and resume checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage3Snapshot {
    pub stage: u32,
    pub claim_mode: String,
    pub compatibility_version: u32,
    pub mode: String,
    pub rng_kind: String,
    pub consume_bins: bool,
    pub input_assets_count: usize,
    pub distributed_assets_count: usize,
    pub actor_claims: Vec<ActorSnapItem>,
    pub wallet_import_stats: Vec<WalletImportStat>,
    pub wallet_persist_stats: Vec<ActorPersistStat>,
}

/// Wallet import counts used to reconcile persisted claim rows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletImportStat {
    pub actor: String,
    pub inserted: usize,
    pub already_exists: usize,
    pub rejected: usize,
}

/// Tracks whether each wallet persisted at the expected row count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorPersistStat {
    pub actor: String,
    pub is_ok: bool,
    pub expected_count: usize,
    pub listed_count: usize,
}

/// Keeps snapshot mismatches and persistence failures distinct for diagnostics.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SnapshotError {
    #[error("mismatch:{field}: expected={expected}, actual={actual}")]
    Mismatch {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    #[error("encode failed: {0}")]
    Encode(String),
    #[error("write failed: {0}")]
    Write(String),
    #[error("persist check failed: {0}")]
    Persist(String),
}

/// Cross-check the Stage 3 snapshot totals before writing the checkpoint artifact.
pub fn reconcile_snapshot(snapshot: &Stage3Snapshot) -> Result<(), SnapshotError> {
    if snapshot.input_assets_count != snapshot.distributed_assets_count {
        return Err(SnapshotError::Mismatch {
            field: "input_vs_distributed",
            expected: snapshot.input_assets_count,
            actual: snapshot.distributed_assets_count,
        });
    }

    let actor_total: usize = snapshot
        .actor_claims
        .iter()
        .map(|item| item.assets_count)
        .sum();
    if snapshot.distributed_assets_count != actor_total {
        return Err(SnapshotError::Mismatch {
            field: "distributed_vs_actor_sum",
            expected: snapshot.distributed_assets_count,
            actual: actor_total,
        });
    }

    let import_total: usize = snapshot
        .wallet_import_stats
        .iter()
        .map(|item| {
            item.inserted
                .saturating_add(item.already_exists)
                .saturating_add(item.rejected)
        })
        .sum();
    if snapshot.distributed_assets_count != import_total {
        return Err(SnapshotError::Mismatch {
            field: "distributed_vs_import_sum",
            expected: snapshot.distributed_assets_count,
            actual: import_total,
        });
    }

    for row in &snapshot.wallet_persist_stats {
        if !row.is_ok {
            return Err(SnapshotError::Persist(format!(
                "actor={} expected={} listed={}",
                row.actor, row.expected_count, row.listed_count
            )));
        }
    }

    Ok(())
}

/// Persist the reconciled snapshot atomically so reruns see a stable checkpoint.
pub fn write_snapshot(path: &Path, snapshot: &Stage3Snapshot) -> Result<(), SnapshotError> {
    let bytes = JsonCodec
        .serialize(snapshot)
        .map_err(|e| SnapshotError::Encode(e.to_string()))?;
    write_file(path, &bytes).map_err(|e| SnapshotError::Write(e.to_string()))
}

/// Inject deterministic faults only in test builds to exercise resume handling.
pub(crate) fn apply_snap_fault(
    snapshot: Stage3Snapshot,
    mode: Option<&str>,
) -> Result<Stage3Snapshot, SnapshotError> {
    let mut snapshot = snapshot;
    let Some(mode) = mode else {
        return Ok(snapshot);
    };

    match mode {
        "count_mismatch" => {
            snapshot.distributed_assets_count = snapshot.distributed_assets_count.saturating_add(1);
            Ok(snapshot)
        }
        "mid_abort" => Err(SnapshotError::Write("midpoint_abort".to_string())),
        _ => Ok(snapshot),
    }
}
