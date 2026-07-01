use z00z_storage::{
    fixture_support::snapshot_fix::{dup_item, hash_mix, snap, temp_store, test_leaf, test_path},
    settlement::{CheckRoot, HjmtProofFamily, SettlementStore, SnapItem, StoreItem},
    snapshot::{
        build_snapshot, PrepSnapshot, PrepSnapshotError, PrepSnapshotStore, PrepSnapshotVersion,
    },
};

#[test]
fn version_gate_fires_first() {
    let good = snap(&[(1, 7, 9)]);
    let (_, store) = temp_store();
    let bad = SnapItem::new(
        good.entries[0].path(),
        good.entries[0].leaf().clone(),
        vec![1, 2, 3],
    )
    .expect("snap item");
    let snapshot = PrepSnapshot::new(PrepSnapshotVersion::new(9), good.prev_root, vec![bad]);

    let err = store
        .validate_snapshot(&snapshot)
        .expect_err("version gate must fire first");

    assert!(matches!(err, PrepSnapshotError::VersionMix));
}

#[test]
fn witness_decode_rejects() {
    let snapshot = snap(&[(1, 7, 9)]);
    let (_, store) = temp_store();
    let item = &snapshot.entries[0];
    let bad = SnapItem::new(item.path(), item.leaf().clone(), vec![1, 2, 3]).expect("snap item");
    let snapshot = PrepSnapshot::new(PrepSnapshotVersion::CURRENT, snapshot.prev_root, vec![bad]);

    let err = store
        .validate_snapshot(&snapshot)
        .expect_err("witness decode mix");

    assert!(matches!(err, PrepSnapshotError::WitDecode(_)));
}

#[test]
fn witness_family_rejects() {
    let mut store = SettlementStore::new();
    let path = test_path(0x41, 17, 0x51);
    let leaf = test_leaf(path, 0x51);
    store
        .put_settlement_item(StoreItem::new(path, leaf.clone()).expect("asset item"))
        .expect("put item");

    let witness = store
        .settlement_proof_blob(&path)
        .expect("proof blob")
        .with_hjmt_proof_family(HjmtProofFamily::NonExistence)
        .encode()
        .expect("encode witness");
    let entry = SnapItem::new(path, leaf, witness).expect("snap item");
    let err = build_snapshot(
        CheckRoot::from(store.settlement_root().expect("settlement root")),
        vec![entry],
    )
    .expect_err("wrong proof family must reject");

    assert!(matches!(err, PrepSnapshotError::WitMix));
}

#[test]
fn path_mix_rejects() {
    let snapshot = snap(&[(1, 7, 9)]);
    let (_, store) = temp_store();
    let item = &snapshot.entries[0];
    let path = test_path(8, item.path().serial_id.get(), 9);
    let bad = SnapItem::new(path, item.leaf().clone(), item.wit().to_vec()).expect("snap item");
    let snapshot = PrepSnapshot::new(PrepSnapshotVersion::CURRENT, snapshot.prev_root, vec![bad]);

    let err = store.validate_snapshot(&snapshot).expect_err("path mix");

    assert!(matches!(err, PrepSnapshotError::PathMix));
}

#[test]
fn serial_mix_rejects() {
    let snapshot = snap(&[(1, 7, 9)]);
    let (_, store) = temp_store();
    let item = &snapshot.entries[0];
    let path = test_path(1, 99, 9);
    let leaf = test_leaf(path, 9);
    let bad = SnapItem::new(path, leaf, item.wit().to_vec()).expect("snap item");
    let snapshot = PrepSnapshot::new(PrepSnapshotVersion::CURRENT, snapshot.prev_root, vec![bad]);

    let err = store.validate_snapshot(&snapshot).expect_err("serial mix");

    assert!(matches!(err, PrepSnapshotError::SerialMix));
}

#[test]
fn terminal_mix_rejects() {
    let snapshot = snap(&[(1, 7, 9)]);
    let (_, store) = temp_store();
    let item = &snapshot.entries[0];
    let path = test_path(1, 7, 3);
    let leaf = test_leaf(path, 3);
    let bad = SnapItem::new(path, leaf, item.wit().to_vec()).expect("snap item");
    let snapshot = PrepSnapshot::new(PrepSnapshotVersion::CURRENT, snapshot.prev_root, vec![bad]);

    let err = store
        .validate_snapshot(&snapshot)
        .expect_err("terminal mix");

    assert!(matches!(err, PrepSnapshotError::TerminalIdMix));
}

#[test]
fn dup_path_rejects() {
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
fn dup_terminal_rejects() {
    let (_, store) = temp_store();
    let root = z00z_storage::settlement::SettlementStateRoot::settlement_v1([9u8; 32]);
    let snapshot = PrepSnapshot::new(
        PrepSnapshotVersion::CURRENT,
        CheckRoot::from(root),
        vec![dup_item(root, 1, 7, 9), dup_item(root, 2, 8, 9)],
    );

    let err = store
        .validate_snapshot(&snapshot)
        .expect_err("dup terminal");

    assert!(matches!(err, PrepSnapshotError::DupTerminalId));
}

#[test]
fn leaf_mix_rejects() {
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
fn leaf_hash_rejects() {
    let snapshot = snap(&[(1, 7, 9)]);
    let (_, store) = temp_store();
    let bad = hash_mix(&snapshot.entries[0]);
    let snapshot = PrepSnapshot::new(PrepSnapshotVersion::CURRENT, snapshot.prev_root, vec![bad]);

    let err = store
        .validate_snapshot(&snapshot)
        .expect_err("leaf hash mix");

    assert!(matches!(err, PrepSnapshotError::LeafHashMix));
}

#[test]
fn root_mix_rejects() {
    let mut snapshot = snap(&[(1, 7, 9), (2, 8, 10)]);
    let (_, store) = temp_store();
    snapshot.prev_root = CheckRoot::new([0u8; 32]);

    let err = store.validate_snapshot(&snapshot).expect_err("root mix");

    assert!(matches!(err, PrepSnapshotError::RootMix));
}
