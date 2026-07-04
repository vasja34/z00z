use z00z_storage::{
    fixture_support::snapshot_fix::{snap, temp_store},
    settlement::CheckRoot,
    snapshot::{PrepSnapshot, PrepSnapshotError, PrepSnapshotStore, PrepSnapshotVersion},
};

#[test]
fn test_root_mix() {
    let mut snapshot = snap(&[(1, 7, 9)]);
    let (_, store) = temp_store();
    snapshot.prev_root = CheckRoot::new([0u8; 32]);

    let err = store.validate_snapshot(&snapshot).expect_err("root mix");

    assert!(matches!(err, PrepSnapshotError::RootMix));
}

#[test]
fn test_root_agrees_item() {
    let snapshot = snap(&[(1, 7, 9), (2, 8, 10)]);
    let (_, store) = temp_store();
    let replay = store.replay_entries(&snapshot).expect("replay entries");

    assert!(replay
        .iter()
        .all(|entry| CheckRoot::from(entry.proof_item().root()) == snapshot.prev_root));
}

#[test]
fn test_needs_full_entry_set() {
    let snapshot = snap(&[(1, 7, 9), (2, 8, 10)]);
    let (_, store) = temp_store();
    let partial = PrepSnapshot::new(
        PrepSnapshotVersion::CURRENT,
        snapshot.prev_root,
        vec![snapshot.entries[0].clone()],
    );

    let err = store
        .validate_snapshot(&partial)
        .expect_err("missing entries must fail root validation");

    assert!(matches!(err, PrepSnapshotError::RootMix));
}
