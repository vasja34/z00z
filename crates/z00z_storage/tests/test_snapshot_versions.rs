use z00z_storage::{
    fixture_support::snapshot_fix::{snap, temp_store},
    settlement::SnapItem,
    snapshot::{PrepSnapshot, PrepSnapshotError, PrepSnapshotStore, PrepSnapshotVersion},
};

#[test]
fn test_version_ok() {
    let snapshot = snap(&[(1, 7, 9)]);
    let (_, store) = temp_store();

    store
        .derive_snapshot_id(&snapshot)
        .expect("derive snapshot id");
}

#[test]
fn test_version_gate_first() {
    let good = snap(&[(1, 7, 9)]);
    let (_, store) = temp_store();
    let bad = SnapItem::new(
        good.entries[0].path(),
        good.entries[0].leaf().clone(),
        vec![1u8, 2, 3],
    )
    .expect("snap item");
    let snapshot = PrepSnapshot::new(PrepSnapshotVersion::new(9), good.prev_root, vec![bad]);

    let err = store
        .validate_snapshot(&snapshot)
        .expect_err("version gate must fire first");

    assert!(matches!(err, PrepSnapshotError::VersionMix));
}
