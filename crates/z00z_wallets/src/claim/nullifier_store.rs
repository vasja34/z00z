#![forbid(unsafe_code)]

//! Canonical replay-protection store for claim nullifiers.

use super::{NullifierEntry, NullifierStatus};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Mutex as StdMutex,
};
use z00z_utils::io::{load_json, path_exists, remove_file, save_json};

/// Canonical public claim data bound into one nullifier reservation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NullifierClaim {
    /// Derived nullifier encoded as lower-hex.
    pub nullifier_hex: String,
    /// Claim id encoded as lower-hex.
    pub claim_id_hex: String,
    /// Numeric chain id used by the active claim package.
    pub chain_id: u32,
    /// Recipient owner binding encoded as lower-hex.
    pub owner_hex: String,
    /// Canonical tx digest for the claim package.
    pub tx_digest_hex: String,
}

/// Reservation handle returned by the active nullifier store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NullifierLease {
    /// Reserved nullifier encoded as lower-hex.
    pub nullifier_hex: String,
}

/// Conflict payload returned when a nullifier is already reserved or spent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NullifierConflict {
    /// Existing store entry that blocks the new reservation.
    pub entry: NullifierEntry,
}

/// Reservation failures for the canonical nullifier store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NullReserveErr {
    /// Internal lock poisoning prevented a safe read or write.
    LockPoison,
    /// The requested nullifier is already present in the store.
    Conflict(NullifierConflict),
    /// Persisted row data could not be loaded safely.
    Corrupt(String),
    /// Persisted row data could not be written safely.
    Persist(String),
}

/// Finalization failures for the canonical nullifier store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NullFinalizeErr {
    /// Internal lock poisoning prevented a safe read or write.
    LockPoison,
    /// Finalization was requested for a non-existent reservation.
    Missing,
    /// Finalization payload does not match the reserved row.
    Mismatch,
    /// Persisted row data could not be loaded safely.
    Corrupt(String),
    /// Persisted row data could not be written safely.
    Persist(String),
}

/// Canonical operations for claim-domain nullifier replay protection.
pub trait NullifierStateStore: Send + Sync {
    /// Read the current lifecycle status for one nullifier.
    fn get_status(&self, nullifier_hex: &str) -> Result<Option<NullifierStatus>, NullReserveErr>;

    /// Reserve one nullifier or reject the claim as a replay.
    fn reserve_or_reject(&self, claim: &NullifierClaim) -> Result<NullifierLease, NullReserveErr>;

    /// Mark a previously reserved nullifier as spent.
    fn mark_spent(
        &self,
        lease: &NullifierLease,
        tx_digest_hex: &str,
    ) -> Result<(), NullFinalizeErr>;

    /// Best-effort rollback for a reservation that was not finalized.
    fn rollback_reservation(&self, lease: &NullifierLease);

    /// Best-effort commit cleanup after canonical storage accepted the claim.
    fn commit_reservation(&self, lease: &NullifierLease);
}

/// In-memory implementation used by the active simulator and tests.
#[derive(Debug, Default)]
pub struct InMemNullStore;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NullRowFile {
    version: u32,
    rows: Vec<NullifierEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Machine-parseable nullifier lifecycle audit row.
pub struct NullAuditRow {
    /// Reserved or finalized nullifier encoded as lower-hex.
    pub nullifier_hex: String,
    /// Claim id encoded as lower-hex.
    pub claim_id_hex: String,
    /// Numeric chain id bound into the nullifier derivation.
    pub chain_id: u32,
    /// Recipient owner binding encoded as lower-hex.
    pub owner_hex: String,
    /// Canonical tx digest for the claim package lifecycle event.
    pub tx_digest_hex: String,
    /// Deterministic lifecycle event label.
    pub event: String,
    /// Monotonic event order sequence.
    pub sequence: u64,
}

#[derive(Debug, Default)]
struct NullStoreCfg {
    row_path: Option<PathBuf>,
    audit_path: Option<PathBuf>,
    mem_rows: BTreeMap<String, NullifierEntry>,
    mem_seq: u64,
}

static NULL_CFG: Lazy<StdMutex<NullStoreCfg>> =
    Lazy::new(|| StdMutex::new(NullStoreCfg::default()));
static GLOBAL_STORE: Lazy<InMemNullStore> = Lazy::new(InMemNullStore::default);

impl InMemNullStore {
    fn load_rows(cfg: &NullStoreCfg) -> Result<(BTreeMap<String, NullifierEntry>, u64), String> {
        let Some(path) = cfg.row_path.as_ref() else {
            return Ok((cfg.mem_rows.clone(), cfg.mem_seq));
        };

        match path_exists(path).map_err(|e| e.to_string())? {
            false => Ok((BTreeMap::new(), 0)),
            true => {
                let file: NullRowFile =
                    load_json(path).map_err(|e| format!("nullifier row load failed: {e}"))?;
                if file.version != 1 {
                    return Err(format!("nullifier row version invalid: {}", file.version));
                }

                let mut rows = BTreeMap::new();
                let mut max_seq = 0u64;
                for row in file.rows {
                    if rows.contains_key(&row.nullifier_hex) {
                        return Err(format!("duplicate nullifier row: {}", row.nullifier_hex));
                    }
                    max_seq = max_seq.max(row.created_at_seq);
                    rows.insert(row.nullifier_hex.clone(), row);
                }
                Ok((rows, max_seq))
            }
        }
    }

    fn save_rows(
        cfg: &mut NullStoreCfg,
        rows: &BTreeMap<String, NullifierEntry>,
        max_seq: u64,
    ) -> Result<(), String> {
        if let Some(path) = cfg.row_path.as_ref() {
            let file = NullRowFile {
                version: 1,
                rows: rows.values().cloned().collect(),
            };
            save_json(path, &file).map_err(|e| format!("nullifier row write failed: {e}"))
        } else {
            cfg.mem_rows = rows.clone();
            cfg.mem_seq = max_seq;
            Ok(())
        }
    }

    fn load_audit(cfg: &NullStoreCfg) -> Result<Vec<NullAuditRow>, String> {
        let Some(path) = cfg.audit_path.as_ref() else {
            return Ok(Vec::new());
        };

        match path_exists(path).map_err(|e| e.to_string())? {
            false => Ok(Vec::new()),
            true => load_json(path).map_err(|e| format!("nullifier audit load failed: {e}")),
        }
    }

    fn save_audit(cfg: &NullStoreCfg, rows: &[NullAuditRow]) -> Result<(), String> {
        let Some(path) = cfg.audit_path.as_ref() else {
            return Ok(());
        };

        save_json(path, &rows.to_vec()).map_err(|e| format!("nullifier audit write failed: {e}"))
    }

    fn add_audit(cfg: &NullStoreCfg, claim: &NullifierClaim, event: &str) -> Result<(), String> {
        if cfg.audit_path.is_none() {
            return Ok(());
        }

        let mut rows = Self::load_audit(cfg)?;
        let sequence = rows
            .last()
            .map(|row| row.sequence)
            .unwrap_or(0)
            .saturating_add(1);
        rows.push(NullAuditRow {
            nullifier_hex: claim.nullifier_hex.clone(),
            claim_id_hex: claim.claim_id_hex.clone(),
            chain_id: claim.chain_id,
            owner_hex: claim.owner_hex.clone(),
            tx_digest_hex: claim.tx_digest_hex.clone(),
            event: event.to_string(),
            sequence,
        });
        Self::save_audit(cfg, &rows)
    }

    fn claim_from_entry(row: &NullifierEntry) -> NullifierClaim {
        NullifierClaim {
            nullifier_hex: row.nullifier_hex.clone(),
            claim_id_hex: row.claim_id_hex.clone(),
            chain_id: row.chain_id,
            owner_hex: row.owner_hex.clone(),
            tx_digest_hex: row.tx_digest_hex.clone(),
        }
    }
}

impl NullifierStateStore for InMemNullStore {
    fn get_status(&self, nullifier_hex: &str) -> Result<Option<NullifierStatus>, NullReserveErr> {
        let cfg = NULL_CFG.lock().map_err(|_| NullReserveErr::LockPoison)?;
        let (rows, _) = Self::load_rows(&cfg).map_err(NullReserveErr::Corrupt)?;
        Ok(rows.get(nullifier_hex).map(|entry| entry.status.clone()))
    }

    fn reserve_or_reject(&self, claim: &NullifierClaim) -> Result<NullifierLease, NullReserveErr> {
        let mut cfg = NULL_CFG.lock().map_err(|_| NullReserveErr::LockPoison)?;
        let (mut rows, max_seq) = Self::load_rows(&cfg).map_err(NullReserveErr::Corrupt)?;
        if let Some(entry) = rows.get(&claim.nullifier_hex) {
            return Err(NullReserveErr::Conflict(NullifierConflict {
                entry: entry.clone(),
            }));
        }

        let entry = NullifierEntry {
            nullifier_hex: claim.nullifier_hex.clone(),
            status: NullifierStatus::Reserved,
            claim_id_hex: claim.claim_id_hex.clone(),
            chain_id: claim.chain_id,
            owner_hex: claim.owner_hex.clone(),
            tx_digest_hex: claim.tx_digest_hex.clone(),
            created_at_seq: max_seq.saturating_add(1),
        };
        rows.insert(claim.nullifier_hex.clone(), entry);
        if let Err(err) = Self::save_rows(&mut cfg, &rows, max_seq.saturating_add(1)) {
            return Err(NullReserveErr::Persist(err));
        }
        if let Err(err) = Self::add_audit(&cfg, claim, "reserve") {
            rows.remove(&claim.nullifier_hex);
            let _ = Self::save_rows(&mut cfg, &rows, max_seq);
            return Err(NullReserveErr::Persist(err));
        }

        Ok(NullifierLease {
            nullifier_hex: claim.nullifier_hex.clone(),
        })
    }

    fn mark_spent(
        &self,
        lease: &NullifierLease,
        tx_digest_hex: &str,
    ) -> Result<(), NullFinalizeErr> {
        let mut cfg = NULL_CFG.lock().map_err(|_| NullFinalizeErr::LockPoison)?;
        let (mut rows, max_seq) = Self::load_rows(&cfg).map_err(NullFinalizeErr::Corrupt)?;
        let row = rows
            .get_mut(&lease.nullifier_hex)
            .ok_or(NullFinalizeErr::Missing)?;
        if row.tx_digest_hex != tx_digest_hex {
            return Err(NullFinalizeErr::Mismatch);
        }
        let old = row.status.clone();
        row.status = NullifierStatus::Spent;
        let claim = Self::claim_from_entry(row);
        if let Err(err) = Self::save_rows(&mut cfg, &rows, max_seq) {
            return Err(NullFinalizeErr::Persist(err));
        }
        if let Err(err) = Self::add_audit(&cfg, &claim, "finalize") {
            if let Some(revert) = rows.get_mut(&lease.nullifier_hex) {
                revert.status = old;
            }
            let _ = Self::save_rows(&mut cfg, &rows, max_seq);
            return Err(NullFinalizeErr::Persist(err));
        }
        Ok(())
    }

    fn rollback_reservation(&self, lease: &NullifierLease) {
        if let Ok(mut cfg) = NULL_CFG.lock() {
            let Ok((mut rows, max_seq)) = Self::load_rows(&cfg) else {
                return;
            };
            let should_drop = rows
                .get(&lease.nullifier_hex)
                .map(|row| row.status == NullifierStatus::Reserved)
                .unwrap_or(false);
            if should_drop {
                let claim = rows
                    .remove(&lease.nullifier_hex)
                    .map(|row| Self::claim_from_entry(&row));
                let _ = Self::save_rows(&mut cfg, &rows, max_seq);
                if let Some(claim) = claim {
                    let _ = Self::add_audit(&cfg, &claim, "rollback");
                }
            }
        }
    }

    fn commit_reservation(&self, lease: &NullifierLease) {
        if let Ok(mut cfg) = NULL_CFG.lock() {
            let Ok((mut rows, max_seq)) = Self::load_rows(&cfg) else {
                return;
            };
            let claim = rows
                .remove(&lease.nullifier_hex)
                .map(|row| Self::claim_from_entry(&row));
            if claim.is_none() {
                return;
            }
            let _ = Self::save_rows(&mut cfg, &rows, max_seq);
            if let Some(claim) = claim {
                let _ = Self::add_audit(&cfg, &claim, "commit");
            }
        }
    }
}

/// Bind the process-global nullifier store to one persisted row file and optional audit file.
pub fn bind_paths(row_path: &Path, audit_path: Option<&Path>) -> Result<(), String> {
    let mut cfg = NULL_CFG
        .lock()
        .map_err(|_| "nullifier store lock poisoned".to_string())?;
    cfg.row_path = Some(row_path.to_path_buf());
    cfg.audit_path = audit_path.map(Path::to_path_buf);
    Ok(())
}

/// Clear any bound persisted paths and return the process-global store to in-memory mode.
pub fn clear_bind() {
    if let Ok(mut cfg) = NULL_CFG.lock() {
        cfg.row_path = None;
        cfg.audit_path = None;
    }
}

/// Compare one persisted row against one claim identity.
pub fn claim_match(entry: &NullifierEntry, claim: &NullifierClaim) -> bool {
    entry.nullifier_hex == claim.nullifier_hex
        && entry.claim_id_hex == claim.claim_id_hex
        && entry.chain_id == claim.chain_id
        && entry.owner_hex == claim.owner_hex
        && entry.tx_digest_hex == claim.tx_digest_hex
}

/// Build a lease handle for one nullifier key.
pub fn create_nullifier_lease(nullifier_hex: &str) -> NullifierLease {
    NullifierLease {
        nullifier_hex: nullifier_hex.to_string(),
    }
}

/// Read one persisted or in-memory nullifier row by lower-hex key.
pub fn read_entry(nullifier_hex: &str) -> Result<Option<NullifierEntry>, String> {
    let cfg = NULL_CFG
        .lock()
        .map_err(|_| "nullifier store lock poisoned".to_string())?;
    let (rows, _) = InMemNullStore::load_rows(&cfg)?;
    Ok(rows.get(nullifier_hex).cloned())
}

/// Read the configured nullifier audit stream.
pub fn read_audit() -> Result<Vec<NullAuditRow>, String> {
    let cfg = NULL_CFG
        .lock()
        .map_err(|_| "nullifier store lock poisoned".to_string())?;
    InMemNullStore::load_audit(&cfg)
}

/// Return the process-global nullifier replay store.
pub fn global_nullifier_store() -> &'static InMemNullStore {
    &GLOBAL_STORE
}

/// Clear all in-memory nullifier rows.
pub fn clear_rows() {
    if let Ok(mut cfg) = NULL_CFG.lock() {
        cfg.mem_rows.clear();
        cfg.mem_seq = 0;
        if let Some(path) = cfg.row_path.as_ref() {
            if matches!(path_exists(path), Ok(true)) {
                let _ = remove_file(path);
            }
        }
        if let Some(path) = cfg.audit_path.as_ref() {
            if matches!(path_exists(path), Ok(true)) {
                let _ = remove_file(path);
            }
        }
    }
}

/// Fetch one in-memory nullifier row by lower-hex key.
pub fn get_entry(nullifier_hex: &str) -> Option<NullifierEntry> {
    read_entry(nullifier_hex).ok().flatten()
}

#[cfg(test)]
#[path = "test_nullifier_store.rs"]
mod tests;
