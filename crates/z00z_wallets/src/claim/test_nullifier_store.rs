use super::{
    bind_paths, claim_match, clear_bind, clear_rows, create_nullifier_lease, get_entry,
    global_nullifier_store, read_audit, InMemNullStore, NullAuditRow, NullFinalizeErr,
    NullReserveErr, NullifierClaim, NullifierStateStore,
};
use crate::claim::NullifierStatus;
use std::{
    sync::{Arc, Mutex, OnceLock},
    thread,
};
use z00z_utils::io::write_file;

static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn test_lock() -> &'static Mutex<()> {
    TEST_LOCK.get_or_init(|| Mutex::new(()))
}

fn claim(nullifier_hex: &str, tx_digest_hex: &str) -> NullifierClaim {
    NullifierClaim {
        nullifier_hex: nullifier_hex.to_string(),
        claim_id_hex: "aa".repeat(32),
        chain_id: 3,
        owner_hex: "bb".repeat(32),
        tx_digest_hex: tx_digest_hex.to_string(),
    }
}

#[test]
fn test_reserve_conflict() {
    let _guard = test_lock().lock().unwrap();
    clear_bind();
    clear_rows();
    let store: &InMemNullStore = global_nullifier_store();
    let first = claim(&"11".repeat(32), &"22".repeat(32));
    store.reserve_or_reject(&first).unwrap();

    let err = store.reserve_or_reject(&first).unwrap_err();
    match err {
        NullReserveErr::Conflict(conf) => {
            assert_eq!(conf.entry.nullifier_hex, first.nullifier_hex);
            assert_eq!(conf.entry.status, NullifierStatus::Reserved);
        }
        other => panic!("unexpected reserve result: {other:?}"),
    }
}

#[test]
fn test_mark_spent() {
    let _guard = test_lock().lock().unwrap();
    clear_bind();
    clear_rows();
    let store: &InMemNullStore = global_nullifier_store();
    let row = claim(&"33".repeat(32), &"44".repeat(32));
    let lease = store.reserve_or_reject(&row).unwrap();
    store.mark_spent(&lease, &row.tx_digest_hex).unwrap();

    let entry = get_entry(&row.nullifier_hex).unwrap();
    assert_eq!(entry.status, NullifierStatus::Spent);
}

#[test]
fn test_rollback_reservation() {
    let _guard = test_lock().lock().unwrap();
    clear_bind();
    clear_rows();
    let store: &InMemNullStore = global_nullifier_store();
    let row = claim(&"55".repeat(32), &"66".repeat(32));
    let lease = store.reserve_or_reject(&row).unwrap();
    store.rollback_reservation(&lease);
    assert!(get_entry(&row.nullifier_hex).is_none());
}

#[test]
fn test_mark_spent_mismatch() {
    let _guard = test_lock().lock().unwrap();
    clear_bind();
    clear_rows();
    let store: &InMemNullStore = global_nullifier_store();
    let row = claim(&"77".repeat(32), &"88".repeat(32));
    let lease = store.reserve_or_reject(&row).unwrap();
    let err = store.mark_spent(&lease, &"99".repeat(32)).unwrap_err();
    assert_eq!(err, NullFinalizeErr::Mismatch);
}

#[test]
fn test_claim_match() {
    let row = crate::claim::NullifierEntry {
        nullifier_hex: "10".repeat(32),
        status: NullifierStatus::Reserved,
        claim_id_hex: "20".repeat(32),
        chain_id: 3,
        owner_hex: "30".repeat(32),
        tx_digest_hex: "40".repeat(32),
        created_at_seq: 1,
    };
    let claim = NullifierClaim {
        nullifier_hex: row.nullifier_hex.clone(),
        claim_id_hex: row.claim_id_hex.clone(),
        chain_id: row.chain_id,
        owner_hex: row.owner_hex.clone(),
        tx_digest_hex: row.tx_digest_hex.clone(),
    };
    assert!(claim_match(&row, &claim));
}

#[test]
fn test_file_restart_replay() {
    let _guard = test_lock().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let row_path = dir.path().join("nullifier_rows.json");
    let audit_path = dir.path().join("nullifier_audit.json");
    bind_paths(&row_path, Some(&audit_path)).unwrap();
    clear_rows();
    let store: &InMemNullStore = global_nullifier_store();
    let row = claim(&"91".repeat(32), &"92".repeat(32));
    store.reserve_or_reject(&row).unwrap();

    clear_bind();
    bind_paths(&row_path, Some(&audit_path)).unwrap();
    let err = store.reserve_or_reject(&row).unwrap_err();
    match err {
        NullReserveErr::Conflict(conf) => {
            assert_eq!(conf.entry.status, NullifierStatus::Reserved);
        }
        other => panic!("unexpected reserve result: {other:?}"),
    }
}

#[test]
fn test_file_corrupt_closed() {
    let _guard = test_lock().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let row_path = dir.path().join("nullifier_rows.json");
    bind_paths(&row_path, None).unwrap();
    clear_rows();
    write_file(&row_path, b"{bad json").unwrap();

    let store: &InMemNullStore = global_nullifier_store();
    let err = store.get_status(&"ab".repeat(32)).unwrap_err();
    match err {
        NullReserveErr::Corrupt(msg) => {
            assert!(msg.contains("nullifier row load failed"));
        }
        other => panic!("unexpected status result: {other:?}"),
    }
}

#[test]
fn test_file_mark_spent_mismatch() {
    let _guard = test_lock().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let row_path = dir.path().join("nullifier_rows.json");
    bind_paths(&row_path, None).unwrap();
    clear_rows();

    let store: &InMemNullStore = global_nullifier_store();
    let row = claim(&"e1".repeat(32), &"e2".repeat(32));
    let lease = store.reserve_or_reject(&row).unwrap();
    let err = store.mark_spent(&lease, &"ff".repeat(32)).unwrap_err();
    assert_eq!(err, NullFinalizeErr::Mismatch);
}

#[test]
fn test_concurrent_reserve() {
    let _guard = test_lock().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let row_path = dir.path().join("nullifier_rows.json");
    bind_paths(&row_path, None).unwrap();
    clear_rows();
    let row = claim(&"a1".repeat(32), &"b1".repeat(32));
    let shared = Arc::new(row);

    let left_claim = Arc::clone(&shared);
    let left = thread::spawn(move || global_nullifier_store().reserve_or_reject(&left_claim));
    let right_claim = Arc::clone(&shared);
    let right = thread::spawn(move || global_nullifier_store().reserve_or_reject(&right_claim));

    let left_ok = left.join().unwrap().is_ok();
    let right_ok = right.join().unwrap().is_ok();
    assert_ne!(left_ok, right_ok);
}

#[test]
fn test_audit_rows() {
    let _guard = test_lock().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let row_path = dir.path().join("nullifier_rows.json");
    let audit_path = dir.path().join("nullifier_audit.json");
    bind_paths(&row_path, Some(&audit_path)).unwrap();
    clear_rows();

    let store: &InMemNullStore = global_nullifier_store();
    let row = claim(&"c1".repeat(32), &"d1".repeat(32));
    let lease_handle = store.reserve_or_reject(&row).unwrap();
    store
        .mark_spent(
            &create_nullifier_lease(&lease_handle.nullifier_hex),
            &row.tx_digest_hex,
        )
        .unwrap();

    let audit = read_audit().unwrap();
    assert_eq!(audit.len(), 2);
    assert_eq!(audit[0].event, "reserve");
    assert_eq!(audit[1].event, "finalize");
    assert_eq!(audit[0].sequence, 1);
    assert_eq!(audit[1].sequence, 2);
    assert_eq!(
        audit[0],
        NullAuditRow {
            nullifier_hex: row.nullifier_hex,
            claim_id_hex: row.claim_id_hex,
            chain_id: row.chain_id,
            owner_hex: row.owner_hex,
            tx_digest_hex: row.tx_digest_hex,
            event: "reserve".to_string(),
            sequence: 1,
        }
    );
}
