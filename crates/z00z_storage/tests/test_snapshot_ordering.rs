use z00z_storage::fixture_support::snapshot_fix::{bytes, save, snap, temp_store};
use z00z_storage::snapshot::PrepSnapshotStore;

#[test]
fn test_order_stable() {
    let specs = [(1, 7, 9), (2, 8, 10)];
    let left = snap(&specs);
    let right = snap(&specs);
    let (_, store) = temp_store();

    assert_eq!(left.entries, right.entries);
    assert_eq!(bytes(&left), bytes(&right));
    assert_eq!(
        store.derive_snapshot_id(&left).expect("left id"),
        store.derive_snapshot_id(&right).expect("right id"),
    );
}

#[test]
fn test_order_changes_id() {
    let left = snap(&[(1, 7, 9), (2, 8, 10)]);
    let right = snap(&[(2, 8, 10), (1, 7, 9)]);
    let (_, store, left_id) = save(&left);
    let right_id = store.derive_snapshot_id(&right).expect("right id");

    assert_ne!(left.entries, right.entries);
    assert_ne!(bytes(&left), bytes(&right));
    assert_ne!(left_id, right_id);
}
