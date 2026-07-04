use z00z_storage::{
    fixture_support::snapshot_fix::{dup_item, snap, temp_store, test_leaf},
    settlement::{CheckRoot, SerialId, SettlementPath, SettlementStateRoot, SnapItem, TerminalId},
    snapshot::{PrepSnapshot, PrepSnapshotError, PrepSnapshotStore, PrepSnapshotVersion},
};

#[test]
fn test_path_mix() {
    let snapshot = snap(&[(1, 7, 9)]);
    let (_, store) = temp_store();
    let item = &snapshot.entries[0];
    let path = SettlementPath::new(
        z00z_storage::settlement::DefinitionId::new([8u8; 32]),
        item.path().serial_id,
        item.path().terminal_id,
    );
    let bad = SnapItem::new(path, item.leaf().clone(), item.wit().to_vec()).expect("snap item");
    let snapshot = PrepSnapshot::new(PrepSnapshotVersion::CURRENT, snapshot.prev_root, vec![bad]);

    let err = store.validate_snapshot(&snapshot).expect_err("path mix");

    assert!(matches!(err, PrepSnapshotError::PathMix));
}

#[test]
fn test_serial_mix() {
    let snapshot = snap(&[(1, 7, 9)]);
    let (_, store) = temp_store();
    let item = &snapshot.entries[0];
    let path = SettlementPath::new(
        item.path().definition_id,
        SerialId::new(99),
        item.path().terminal_id,
    );
    let leaf = test_leaf(path, 9);
    let bad = SnapItem::new(path, leaf, item.wit().to_vec()).expect("snap item");
    let snapshot = PrepSnapshot::new(PrepSnapshotVersion::CURRENT, snapshot.prev_root, vec![bad]);

    let err = store.validate_snapshot(&snapshot).expect_err("serial mix");

    assert!(matches!(err, PrepSnapshotError::SerialMix));
}

#[test]
fn test_asset_id_mix() {
    let snapshot = snap(&[(1, 7, 9)]);
    let (_, store) = temp_store();
    let item = &snapshot.entries[0];
    let path = SettlementPath::new(
        item.path().definition_id,
        item.path().serial_id,
        TerminalId::new([3u8; 32]),
    );
    let leaf = test_leaf(path, 3);
    let bad = SnapItem::new(path, leaf, item.wit().to_vec()).expect("snap item");
    let snapshot = PrepSnapshot::new(PrepSnapshotVersion::CURRENT, snapshot.prev_root, vec![bad]);

    let err = store
        .validate_snapshot(&snapshot)
        .expect_err("asset-id mix");

    assert!(matches!(err, PrepSnapshotError::TerminalIdMix));
}

#[test]
fn test_dup_path() {
    let snapshot = snap(&[(1, 7, 9)]);
    let (_, store) = temp_store();
    let item = snapshot.entries[0].clone();
    let snapshot = PrepSnapshot::new(
        PrepSnapshotVersion::CURRENT,
        snapshot.prev_root,
        vec![item.clone(), item],
    );

    let err = store.validate_snapshot(&snapshot).expect_err("dup path");

    assert!(matches!(err, PrepSnapshotError::DupPath));
}

#[test]
fn test_dup_asset() {
    let (_, store) = temp_store();
    let root = SettlementStateRoot::settlement_v1([9u8; 32]);
    let snapshot = PrepSnapshot::new(
        PrepSnapshotVersion::CURRENT,
        CheckRoot::from(root),
        vec![dup_item(root, 1, 7, 9), dup_item(root, 2, 8, 9)],
    );

    let err = store.validate_snapshot(&snapshot).expect_err("dup asset");

    assert!(matches!(err, PrepSnapshotError::DupTerminalId));
}
