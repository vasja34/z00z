use std::{
    collections::{btree_map::Entry, BTreeMap},
    path::Path,
    sync::{Mutex as StdMutex, OnceLock},
};

use z00z_wallets::claim::{
    bind_paths, claim_match, clear_bind, create_nullifier_lease, get_entry, global_nullifier_store,
    NullifierClaim, NullifierLease, NullifierStateStore, NullifierStatus,
};
use z00z_wallets::tx::ClaimTxPackage;

pub const NULL_ROWS_FILE: &str = "nullifier_rows.json";
pub const NULL_AUDIT_FILE: &str = "nullifier_audit.json";

#[derive(Debug, Clone, PartialEq, Eq)]
struct NullWork {
    lease: NullifierLease,
    fresh: bool,
}

fn store_lock() -> &'static StdMutex<()> {
    static LOCK: OnceLock<StdMutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| StdMutex::new(()))
}

struct StoreBind;

impl StoreBind {
    fn bind(path: &Path) -> Result<Self, String> {
        bind_pkg_store(path)?;
        Ok(Self)
    }
}

impl Drop for StoreBind {
    fn drop(&mut self) {
        clear_bind();
    }
}

fn bind_pkg_store(path: &Path) -> Result<(), String> {
    let dir = path
        .parent()
        .ok_or_else(|| "claim package path has no parent directory".to_string())?;
    let row_path = dir.join(NULL_ROWS_FILE);
    let audit_path = dir.join(NULL_AUDIT_FILE);
    bind_paths(&row_path, Some(&audit_path))
}

pub(crate) fn with_pkg_store<T>(
    path: &Path,
    run: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let _guard = store_lock()
        .lock()
        .map_err(|_| "claim package store lock poisoned".to_string())?;
    let _bind = StoreBind::bind(path)?;
    run()
}

fn pkg_claim(pkg: &ClaimTxPackage, idx: usize) -> Result<NullifierClaim, String> {
    let claim = pkg
        .tx
        .inputs
        .first()
        .ok_or_else(|| format!("package[{idx}] missing claim input"))?;

    Ok(NullifierClaim {
        nullifier_hex: pkg.tx.context.nullifier_hex.clone(),
        claim_id_hex: claim.claim_id_hex.clone(),
        chain_id: pkg.chain_id,
        owner_hex: pkg.tx.context.recipient_owner_hex.clone(),
        tx_digest_hex: pkg.tx_digest_hex.clone(),
    })
}

fn same_claim(left: &NullifierClaim, right: &NullifierClaim) -> bool {
    left.claim_id_hex == right.claim_id_hex
        && left.chain_id == right.chain_id
        && left.owner_hex == right.owner_hex
        && left.tx_digest_hex == right.tx_digest_hex
}

fn collision_err(nullifier_hex: &str) -> String {
    format!("claim nullifier collision across package rows: {nullifier_hex}")
}

fn merge_claim(
    uniq: &mut BTreeMap<String, NullifierClaim>,
    next: NullifierClaim,
) -> Result<(), String> {
    match uniq.entry(next.nullifier_hex.clone()) {
        Entry::Occupied(prev) => {
            if !same_claim(prev.get(), &next) {
                return Err(collision_err(&next.nullifier_hex));
            }
        }
        Entry::Vacant(slot) => {
            slot.insert(next);
        }
    }

    Ok(())
}

pub(crate) fn claim_nulls(packages: &[ClaimTxPackage]) -> Result<Vec<NullifierClaim>, String> {
    let mut uniq = BTreeMap::<String, NullifierClaim>::new();

    for (idx, pkg) in packages.iter().enumerate() {
        let next = pkg_claim(pkg, idx)?;
        merge_claim(&mut uniq, next)?;
    }

    Ok(uniq.into_values().collect())
}

pub(crate) fn reserve_nulls(claims: &[NullifierClaim]) -> Result<Vec<NullifierLease>, String> {
    let store = global_nullifier_store();
    let mut leases = Vec::with_capacity(claims.len());

    for claim in claims {
        match store.reserve_or_reject(claim) {
            Ok(lease) => leases.push(lease),
            Err(err) => {
                for lease in &leases {
                    store.rollback_reservation(lease);
                }
                return Err(match err {
                    z00z_wallets::claim::NullReserveErr::LockPoison => {
                        "claim nullifier store lock poisoned".to_string()
                    }
                    z00z_wallets::claim::NullReserveErr::Conflict(conf) => format!(
                        "claim nullifier replay rejected: nullifier={} status={:?} tx_digest={}",
                        conf.entry.nullifier_hex, conf.entry.status, conf.entry.tx_digest_hex
                    ),
                    z00z_wallets::claim::NullReserveErr::Corrupt(msg) => msg,
                    z00z_wallets::claim::NullReserveErr::Persist(msg) => msg,
                });
            }
        }
    }

    Ok(leases)
}

fn need_reserved_nulls(claims: &[NullifierClaim]) -> Result<Vec<NullWork>, String> {
    let mut work = Vec::with_capacity(claims.len());

    for claim in claims {
        let entry = get_entry(&claim.nullifier_hex).ok_or_else(|| {
            format!(
                "claim nullifier reservation missing before publish: {}",
                claim.nullifier_hex
            )
        })?;
        if entry.status == NullifierStatus::Reserved && claim_match(&entry, claim) {
            work.push(NullWork {
                lease: create_nullifier_lease(&claim.nullifier_hex),
                fresh: false,
            });
            continue;
        }
        return Err(format!(
            "claim nullifier replay rejected: nullifier={} status={:?} tx_digest={}",
            entry.nullifier_hex, entry.status, entry.tx_digest_hex
        ));
    }

    Ok(work)
}

pub(crate) fn load_reserved_nulls(
    claims: &[NullifierClaim],
) -> Result<Vec<NullifierLease>, String> {
    let work = need_reserved_nulls(claims)?;
    Ok(work.into_iter().map(|item| item.lease).collect())
}

pub(crate) fn rollback_leases(leases: &[NullifierLease]) {
    let store = global_nullifier_store();
    for lease in leases {
        store.rollback_reservation(lease);
    }
}

pub(crate) fn commit_leases(leases: &[NullifierLease]) {
    let store = global_nullifier_store();
    for lease in leases {
        store.commit_reservation(lease);
    }
}
