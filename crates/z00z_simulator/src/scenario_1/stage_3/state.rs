use super::{
    claim_registry, load_json, path_exists, save_json, AssetWire, Deserialize, HashMap, Path,
    ReceiverKeys, ScanResult, Serialize, StealthOutputScanner,
};

/// Checkpoint steps written to `claim_state.json` for crash safety.
///
/// Transition order: `Started → ArtifactsWritten → WalletsUpdated → BinsConsumed`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClaimStep {
    /// Stage started; artifacts not yet written.
    Started,
    /// JSON artifacts written; genesis bins still intact.
    ArtifactsWritten,
    /// Wallet import completed for all actors.
    WalletsUpdated,
    /// Genesis bins emptied; claim fully committed.
    BinsConsumed,
}

/// Crash-safe state file written to `{genesis_dir}/claim_state.json`.
///
/// Records the last completed checkpoint so a re-run can detect mid-crash state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimStateFile {
    /// Run fingerprint: `mode|rng_kind|consume=<bool>`.
    pub run_id: String,
    /// Distribution mode string.
    pub mode: String,
    /// RNG kind: `"mock:<seed>"` or `"system"`.
    pub rng_kind: String,
    /// Last completed checkpoint.
    pub step: ClaimStep,
    /// Unix timestamp (seconds) when the stage started.
    pub started_at_unix: u64,
    /// Claimed rows persisted for resume/idempotency.
    #[serde(default)]
    pub claimed_rows: Vec<ClaimStateRow>,
}

/// One persisted claimed row for resume/idempotency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimStateRow {
    /// Wallet id that owns this finalized claim.
    pub wallet_id: String,
    /// Asset id hex.
    pub asset_id_hex: String,
}

pub(crate) fn load_claim_state_file(path: &Path) -> Result<Option<ClaimStateFile>, String> {
    match path_exists(path) {
        Ok(true) => {
            let state: ClaimStateFile =
                load_json(path).map_err(|e| format!("claim_state.json corrupt: {e}"))?;
            Ok(Some(state))
        }
        Ok(false) => Ok(None),
        Err(e) => Err(format!("claim_state check failed: {e}")),
    }
}

pub(crate) fn persist_claim_state_file(path: &Path, state: &ClaimStateFile) -> Result<(), String> {
    let next = if let Some(old) = load_claim_state_file(path)? {
        merge_state(&old, state)?
    } else {
        state.clone()
    };
    save_json(path, &next).map_err(|e| format!("claim_state write failed: {e}"))
}

fn step_rank(step: &ClaimStep) -> u8 {
    match step {
        ClaimStep::Started => 0,
        ClaimStep::ArtifactsWritten => 1,
        ClaimStep::WalletsUpdated => 2,
        ClaimStep::BinsConsumed => 3,
    }
}

fn merge_state(old: &ClaimStateFile, next: &ClaimStateFile) -> Result<ClaimStateFile, String> {
    if step_rank(&next.step) < step_rank(&old.step) {
        return Err(format!(
            "claim_state step regression: old={:?} next={:?}",
            old.step, next.step
        ));
    }

    let mut rows: HashMap<(String, String), ClaimStateRow> = HashMap::new();
    for row in &old.claimed_rows {
        rows.insert(
            (row.wallet_id.clone(), row.asset_id_hex.clone()),
            row.clone(),
        );
    }
    for row in &next.claimed_rows {
        rows.insert(
            (row.wallet_id.clone(), row.asset_id_hex.clone()),
            row.clone(),
        );
    }

    let mut claimed_rows: Vec<ClaimStateRow> = rows.into_values().collect();
    claimed_rows.sort_by(|a, b| {
        a.wallet_id
            .cmp(&b.wallet_id)
            .then_with(|| a.asset_id_hex.cmp(&b.asset_id_hex))
    });

    Ok(ClaimStateFile {
        run_id: next.run_id.clone(),
        mode: next.mode.clone(),
        rng_kind: next.rng_kind.clone(),
        step: next.step.clone(),
        started_at_unix: next.started_at_unix,
        claimed_rows,
    })
}

/// Merge resumed claim state without letting checkpoint order move backwards.
pub fn merge_state_files(
    old: &ClaimStateFile,
    next: &ClaimStateFile,
) -> Result<ClaimStateFile, String> {
    merge_state(old, next)
}

pub(crate) fn claim_row_exists(
    rows: &[ClaimStateRow],
    wallet_id: &str,
    asset_id: [u8; 32],
) -> bool {
    let aid = hex::encode(asset_id);
    rows.iter()
        .any(|row| row.wallet_id == wallet_id && row.asset_id_hex == aid)
}

pub(crate) fn push_claim_row(state: &mut ClaimStateFile, wallet_id: &str, asset_id: [u8; 32]) {
    if claim_row_exists(&state.claimed_rows, wallet_id, asset_id) {
        return;
    }
    state.claimed_rows.push(ClaimStateRow {
        wallet_id: wallet_id.to_string(),
        asset_id_hex: hex::encode(asset_id),
    });
}

pub(crate) fn rehydrate_rows(rows: &[ClaimStateRow]) -> Result<(), String> {
    for row in rows {
        let asset_id = parse_hex_id(&row.asset_id_hex)?;
        claim_registry::mark_final(&row.wallet_id, asset_id).map_err(|e| match e {
            claim_registry::ClaimReserveErr::Conflict(conf) => {
                format!("resume registry conflict: {}", conf.claimed_by)
            }
            claim_registry::ClaimReserveErr::InvalidReceipt => {
                "resume registry invalid row".to_string()
            }
            claim_registry::ClaimReserveErr::LockPoison => {
                "resume registry lock poison".to_string()
            }
        })?;
    }
    Ok(())
}

/// Rebuild claim registry reservations so resume checks see the finalized rows.
pub fn rehydrate_rows_from_state(rows: &[ClaimStateRow]) -> Result<(), String> {
    rehydrate_rows(rows)
}

/// Re-run wallet decoding and ownership checks before accepting a resumed wire.
pub fn verify_resume_wire(wire: &AssetWire, keys: &ReceiverKeys) -> Result<(), String> {
    let rebuilt = wire
        .clone()
        .to_asset()
        .map_err(|e| format!("resume verify decode failed: {e}"))?;
    rebuilt
        .verify_complete()
        .map_err(|e| format!("resume verify failed: {e}"))?;
    let scanner = StealthOutputScanner::from_keys(keys);
    match scanner.scan_leaf(&rebuilt) {
        ScanResult::Mine { .. } => Ok(()),
        ScanResult::NotMine | ScanResult::MaybeMine { .. } => {
            Err("resume verify owner binding mismatch".to_string())
        }
    }
}

fn parse_hex_id(text: &str) -> Result<[u8; 32], String> {
    if text.len() != 64 {
        return Err(format!("invalid asset hex len: {}", text.len()));
    }
    let mut out = [0u8; 32];
    let bytes = text.as_bytes();
    for index in 0..32 {
        let hi = hex_nib(bytes[index * 2])?;
        let lo = hex_nib(bytes[index * 2 + 1])?;
        out[index] = (hi << 4) | lo;
    }
    Ok(out)
}

fn hex_nib(ch: u8) -> Result<u8, String> {
    match ch {
        b'0'..=b'9' => Ok(ch - b'0'),
        b'a'..=b'f' => Ok(ch - b'a' + 10),
        b'A'..=b'F' => Ok(ch - b'A' + 10),
        _ => Err("invalid asset hex".to_string()),
    }
}
