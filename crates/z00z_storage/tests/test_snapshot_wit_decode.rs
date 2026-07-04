use z00z_storage::{
    fixture_support::snapshot_fix::{snap, temp_store},
    settlement::SnapItem,
    snapshot::{PrepSnapshot, PrepSnapshotError, PrepSnapshotStore, PrepSnapshotVersion},
};

#[test]
fn test_wit_decode_mix() {
    let snapshot = snap(&[(1, 7, 9)]);
    let (_, store) = temp_store();
    let item = &snapshot.entries[0];
    let bad = SnapItem::new(item.path(), item.leaf().clone(), vec![1u8, 2, 3]).expect("snap item");
    let snapshot = PrepSnapshot::new(PrepSnapshotVersion::CURRENT, snapshot.prev_root, vec![bad]);

    let err = store
        .validate_snapshot(&snapshot)
        .expect_err("wit decode mix");

    assert!(matches!(err, PrepSnapshotError::WitDecode(_)));
}
