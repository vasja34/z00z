use z00z_storage::{
    fixture_support::snapshot_fix::{hash_mix, snap, temp_store},
    settlement::SnapItem,
    snapshot::{PrepSnapshot, PrepSnapshotError, PrepSnapshotStore, PrepSnapshotVersion},
};

#[test]
fn test_leaf_ok() {
    let snapshot = snap(&[(1, 7, 9)]);
    let (_, store) = temp_store();

    store
        .validate_snapshot(&snapshot)
        .expect("validate snapshot");
}

#[test]
fn test_leaf_mix() {
    let snapshot = snap(&[(1, 7, 9)]);
    let (_, store) = temp_store();
    let item = &snapshot.entries[0];
    let mut leaf = item.terminal_leaf().expect("asset leaf").clone();
    leaf.owner_tag[0] ^= 1;
    let bad = SnapItem::new(item.path(), leaf, item.wit().to_vec()).expect("snap item");
    let snapshot = PrepSnapshot::new(PrepSnapshotVersion::CURRENT, snapshot.prev_root, vec![bad]);

    let err = store.validate_snapshot(&snapshot).expect_err("leaf mix");

    assert!(matches!(err, PrepSnapshotError::LeafMix));
}

#[test]
fn test_leaf_hash_mix() {
    let snapshot = snap(&[(1, 7, 9)]);
    let (_, store) = temp_store();
    let bad = hash_mix(&snapshot.entries[0]);
    let snapshot = PrepSnapshot::new(PrepSnapshotVersion::CURRENT, snapshot.prev_root, vec![bad]);

    let err = store
        .validate_snapshot(&snapshot)
        .expect_err("leaf-hash mix");

    assert!(matches!(err, PrepSnapshotError::LeafHashMix));
}
