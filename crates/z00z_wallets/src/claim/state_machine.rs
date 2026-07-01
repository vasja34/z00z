//! Claim state machine contract for resume-safe claim flow.

use std::{collections::BTreeMap, path::Path};

use z00z_core::assets::AssetWire;
use z00z_utils::io::{load_json, path_exists, save_json};

use crate::claim::registry as claim_registry;
use serde::{Deserialize, Serialize};

use super::service::ClaimLifeStep;

/// One persisted claim row used for idempotent replay.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ClaimStateRow {
    /// Wallet id that owns this finalized claim.
    pub wallet_id: String,
    /// Asset id in lower-hex format.
    pub asset_id_hex: String,
}

/// Resume-safe claim state persisted by claim orchestration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimStateFile {
    /// Run fingerprint.
    pub run_id: String,
    /// Distribution mode key.
    pub mode: String,
    /// RNG mode key.
    pub rng_kind: String,
    /// Last completed lifecycle step.
    pub step: ClaimLifeStep,
    /// Unix timestamp when claim run started.
    pub started_at_unix: u64,
    /// Claimed rows for replay safety.
    #[serde(default)]
    pub claimed_rows: Vec<ClaimStateRow>,
}

impl ClaimStateFile {
    /// Merge two state snapshots without allowing step regression.
    pub fn merge(self, next: Self) -> Result<Self, String> {
        if !self.step.can_move_to(next.step) {
            return Err(format!(
                "claim state step regression: old={:?} next={:?}",
                self.step, next.step
            ));
        }

        let mut rows = BTreeMap::new();
        for row in self.claimed_rows {
            rows.insert((row.wallet_id.clone(), row.asset_id_hex.clone()), row);
        }
        for row in next.claimed_rows {
            rows.insert((row.wallet_id.clone(), row.asset_id_hex.clone()), row);
        }

        Ok(Self {
            run_id: next.run_id,
            mode: next.mode,
            rng_kind: next.rng_kind,
            step: next.step,
            started_at_unix: next.started_at_unix,
            claimed_rows: rows.into_values().collect(),
        })
    }
}

/// Read claim state if file exists.
pub fn read_state(path: &Path) -> Result<Option<ClaimStateFile>, String> {
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

/// Write claim state with merge-on-write semantics.
pub fn write_state(path: &Path, state: &ClaimStateFile) -> Result<(), String> {
    let next = if let Some(old) = read_state(path)? {
        old.merge(state.clone())?
    } else {
        state.clone()
    };
    save_json(path, &next).map_err(|e| format!("claim_state write failed: {e}"))
}

/// Returns true if wallet/asset pair already exists in claim rows.
pub fn has_row(rows: &[ClaimStateRow], wallet_id: &str, asset_id_hex: &str) -> bool {
    rows.iter()
        .any(|row| row.wallet_id == wallet_id && row.asset_id_hex == asset_id_hex)
}

/// Insert wallet/asset claim row when missing.
pub fn add_row(state: &mut ClaimStateFile, wallet_id: &str, asset_id_hex: &str) {
    if has_row(&state.claimed_rows, wallet_id, asset_id_hex) {
        return;
    }
    state.claimed_rows.push(ClaimStateRow {
        wallet_id: wallet_id.to_string(),
        asset_id_hex: asset_id_hex.to_string(),
    });
}

/// Rehydrate claim rows into claim registry.
pub fn rehydrate_rows(rows: &[ClaimStateRow]) -> Result<(), String> {
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

/// Verify wire compatibility for resume path.
pub fn verify_resume_wire(wire: &AssetWire, owner_tag: Option<[u8; 32]>) -> Result<(), String> {
    let rebuilt = wire
        .clone()
        .to_asset()
        .map_err(|e| format!("resume verify decode failed: {e}"))?;
    rebuilt
        .verify_complete()
        .map_err(|e| format!("resume verify failed: {e}"))?;
    if rebuilt.owner_tag != owner_tag {
        return Err("resume verify owner binding mismatch".to_string());
    }
    Ok(())
}

fn parse_hex_id(text: &str) -> Result<[u8; 32], String> {
    if text.len() != 64 {
        return Err(format!("invalid asset hex len: {}", text.len()));
    }
    let mut out = [0u8; 32];
    let bytes = text.as_bytes();
    for i in 0..32 {
        let hi = hex_nib(bytes[i * 2])?;
        let lo = hex_nib(bytes[i * 2 + 1])?;
        out[i] = (hi << 4) | lo;
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

#[cfg(test)]
mod tests {
    use super::{add_row, has_row, ClaimStateFile, ClaimStateRow};
    use crate::claim::service::ClaimLifeStep;

    fn mk_state(step: ClaimLifeStep) -> ClaimStateFile {
        ClaimStateFile {
            run_id: "uniform_all|mock:7|consume=true".to_string(),
            mode: "uniform_all".to_string(),
            rng_kind: "mock:7".to_string(),
            step,
            started_at_unix: 1,
            claimed_rows: vec![],
        }
    }

    #[test]
    fn test_merge_blocks_step_regression() {
        let old = mk_state(ClaimLifeStep::WalletsUpdated);
        let next = mk_state(ClaimLifeStep::ArtifactsWritten);
        assert!(old.merge(next).is_err());
    }

    #[test]
    fn test_merge_keeps_unique_rows() {
        let mut old = mk_state(ClaimLifeStep::ArtifactsWritten);
        old.claimed_rows.push(ClaimStateRow {
            wallet_id: "alice".to_string(),
            asset_id_hex: "11".repeat(32),
        });

        let mut next = mk_state(ClaimLifeStep::WalletsUpdated);
        next.claimed_rows.push(ClaimStateRow {
            wallet_id: "alice".to_string(),
            asset_id_hex: "11".repeat(32),
        });
        next.claimed_rows.push(ClaimStateRow {
            wallet_id: "bob".to_string(),
            asset_id_hex: "22".repeat(32),
        });

        let merged = old.merge(next).expect("merge ok");
        assert_eq!(merged.claimed_rows.len(), 2);
    }

    #[test]
    fn test_row_add_is_idempotent() {
        let mut state = mk_state(ClaimLifeStep::Started);
        add_row(&mut state, "alice", &"11".repeat(32));
        add_row(&mut state, "alice", &"11".repeat(32));
        assert_eq!(state.claimed_rows.len(), 1);
        assert!(has_row(&state.claimed_rows, "alice", &"11".repeat(32)));
    }
}
