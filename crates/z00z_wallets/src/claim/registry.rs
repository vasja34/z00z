//! Claim registry and replay-protection gate.

use crate::claim::{verify_claim_receipt, ClaimReceipt};
use once_cell::sync::Lazy;
use std::collections::BTreeMap;
use std::sync::RwLock;
use z00z_crypto::KernelSignature;

/// One claim row for one asset in registry.
#[derive(Debug, Clone)]
pub struct ClaimRow {
    /// Wallet id that reserved/finalized the claim.
    pub wallet_id: String,
    /// Identity public key bytes used for claim signature.
    pub identity_pk: [u8; 32],
    /// Claim signature.
    pub claim_sig: KernelSignature,
    /// Claim scope hash.
    pub claim_scope: [u8; 32],
    /// `true` when finalize completed.
    pub is_final: bool,
}

/// Claim reservation handle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimReservation {
    /// Reserved asset id.
    pub asset_id: [u8; 32],
    /// Winner wallet id.
    pub wallet_id: String,
}

/// Claim conflict details for loser wallet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimConflict {
    /// Existing claimant wallet id.
    pub claimed_by: String,
}

/// Reserve failure kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimReserveErr {
    /// Registry lock is poisoned.
    LockPoison,
    /// Receipt payload/signature is invalid.
    InvalidReceipt,
    /// Asset is already reserved by another wallet.
    Conflict(ClaimConflict),
}

/// Finalize failure kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimFinalizeErr {
    /// Registry lock is poisoned.
    LockPoison,
    /// Missing row for target asset.
    MissingClaim,
    /// Wallet id does not match row owner.
    OwnerMiss,
}

/// Global claim registry contract.
pub trait GlobalClaimRegistry: Send + Sync {
    /// Reserve claim for asset and wallet with verified receipt.
    fn reserve(
        &self,
        asset_id: [u8; 32],
        wallet_id: &str,
        receipt: &ClaimReceipt,
        receipt_sig: &KernelSignature,
    ) -> Result<ClaimReservation, ClaimReserveErr>;

    /// Finalize claim reservation.
    fn finalize(&self, res: &ClaimReservation) -> Result<(), ClaimFinalizeErr>;

    /// Release pending reservation (best-effort).
    fn release(&self, res: &ClaimReservation);
}

static CLAIM_ROWS: Lazy<RwLock<BTreeMap<[u8; 32], ClaimRow>>> =
    Lazy::new(|| RwLock::new(BTreeMap::new()));
static FINAL_ROWS: Lazy<RwLock<BTreeMap<[u8; 32], String>>> =
    Lazy::new(|| RwLock::new(BTreeMap::new()));

/// In-memory registry implementation.
#[derive(Debug, Default)]
pub struct InMemClaimRegistry;

impl GlobalClaimRegistry for InMemClaimRegistry {
    fn reserve(
        &self,
        asset_id: [u8; 32],
        wallet_id: &str,
        receipt: &ClaimReceipt,
        receipt_sig: &KernelSignature,
    ) -> Result<ClaimReservation, ClaimReserveErr> {
        if receipt.asset_id != asset_id || receipt.wallet_id != wallet_id.as_bytes() {
            return Err(ClaimReserveErr::InvalidReceipt);
        }

        verify_claim_receipt(receipt, receipt_sig).map_err(|_| ClaimReserveErr::InvalidReceipt)?;

        {
            let rows = FINAL_ROWS.read().map_err(|_| ClaimReserveErr::LockPoison)?;
            if let Some(owner) = rows.get(&asset_id) {
                if owner != wallet_id {
                    return Err(ClaimReserveErr::Conflict(ClaimConflict {
                        claimed_by: owner.clone(),
                    }));
                }
                return Ok(ClaimReservation {
                    asset_id,
                    wallet_id: wallet_id.to_string(),
                });
            }
        }

        let mut rows = CLAIM_ROWS
            .write()
            .map_err(|_| ClaimReserveErr::LockPoison)?;
        match rows.get(&asset_id) {
            Some(old) if old.wallet_id != wallet_id => {
                Err(ClaimReserveErr::Conflict(ClaimConflict {
                    claimed_by: old.wallet_id.clone(),
                }))
            }
            Some(_) => Ok(ClaimReservation {
                asset_id,
                wallet_id: wallet_id.to_string(),
            }),
            None => {
                rows.insert(
                    asset_id,
                    ClaimRow {
                        wallet_id: wallet_id.to_string(),
                        identity_pk: receipt.identity_pk,
                        claim_sig: receipt_sig.clone(),
                        claim_scope: receipt.claim_scope,
                        is_final: false,
                    },
                );
                Ok(ClaimReservation {
                    asset_id,
                    wallet_id: wallet_id.to_string(),
                })
            }
        }
    }

    fn finalize(&self, res: &ClaimReservation) -> Result<(), ClaimFinalizeErr> {
        let mut rows = CLAIM_ROWS
            .write()
            .map_err(|_| ClaimFinalizeErr::LockPoison)?;

        let Some(row) = rows.get_mut(&res.asset_id) else {
            let mut finals = FINAL_ROWS
                .write()
                .map_err(|_| ClaimFinalizeErr::LockPoison)?;
            return match finals.get(&res.asset_id) {
                Some(owner) if owner == &res.wallet_id => Ok(()),
                Some(_) => Err(ClaimFinalizeErr::OwnerMiss),
                None => {
                    finals.insert(res.asset_id, res.wallet_id.clone());
                    Ok(())
                }
            };
        };
        if row.wallet_id != res.wallet_id {
            return Err(ClaimFinalizeErr::OwnerMiss);
        }

        row.is_final = true;
        let mut finals = FINAL_ROWS
            .write()
            .map_err(|_| ClaimFinalizeErr::LockPoison)?;
        finals.insert(res.asset_id, res.wallet_id.clone());
        Ok(())
    }

    fn release(&self, res: &ClaimReservation) {
        if let Ok(mut rows) = CLAIM_ROWS.write() {
            let should_drop = rows
                .get(&res.asset_id)
                .map(|row| row.wallet_id == res.wallet_id && !row.is_final)
                .unwrap_or(false);
            if should_drop {
                rows.remove(&res.asset_id);
            }
        }
    }
}

static GLOBAL_REGISTRY: Lazy<InMemClaimRegistry> = Lazy::new(InMemClaimRegistry::default);

/// Returns process-global claim registry.
pub fn global_claim_registry() -> &'static InMemClaimRegistry {
    &GLOBAL_REGISTRY
}

/// Rehydrates a finalized claim row from trusted state.
///
/// Recovery semantics:
/// - Stage resume loads persisted `claim_state.json` rows and calls this helper.
/// - Rehydrated rows are inserted into `FINAL_ROWS` so reserve/finalize conflict
///   checks remain active after process restart.
/// - Rehydrate is idempotent for same `(wallet_id, asset_id)` and fails closed
///   on cross-wallet conflicts.
pub fn mark_final(wallet_id: &str, asset_id: [u8; 32]) -> Result<(), ClaimReserveErr> {
    let mut rows = FINAL_ROWS
        .write()
        .map_err(|_| ClaimReserveErr::LockPoison)?;
    if let Some(old) = rows.get(&asset_id) {
        if old != wallet_id {
            return Err(ClaimReserveErr::Conflict(ClaimConflict {
                claimed_by: old.clone(),
            }));
        }
        return Ok(());
    }
    rows.insert(asset_id, wallet_id.to_string());
    Ok(())
}

/// Returns true when a pending (not finalized) row exists for wallet and asset.
pub fn has_pending_owner(wallet_id: &str, asset_id: [u8; 32]) -> bool {
    if let Ok(rows) = CLAIM_ROWS.read() {
        return rows
            .get(&asset_id)
            .map(|row| row.wallet_id == wallet_id && !row.is_final)
            .unwrap_or(false);
    }
    false
}

/// Returns `true` when claim row exists and is pending for wallet.
pub fn is_pending(wallet_id: &str, asset_id: [u8; 32]) -> bool {
    has_pending_owner(wallet_id, asset_id)
}

/// Returns `true` when any row exists for asset.
pub fn has_row(asset_id: [u8; 32]) -> bool {
    if let Ok(rows) = FINAL_ROWS.read() {
        if rows.contains_key(&asset_id) {
            return true;
        }
    }
    if let Ok(rows) = CLAIM_ROWS.read() {
        return rows.contains_key(&asset_id);
    }
    false
}

/// Returns claim row for asset.
pub fn get_row(asset_id: [u8; 32]) -> Option<ClaimRow> {
    if let Ok(rows) = CLAIM_ROWS.read() {
        return rows.get(&asset_id).cloned();
    }
    None
}

/// Clears registry rows (test helper).
pub fn clear_rows() {
    if let Ok(mut rows) = FINAL_ROWS.write() {
        rows.clear();
    }
    if let Ok(mut rows) = CLAIM_ROWS.write() {
        rows.clear();
    }
}
