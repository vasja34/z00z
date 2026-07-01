use z00z_storage::fixture_support::snapshot_fix::{bin_path, bytes, save, snap};
use z00z_storage::snapshot::PrepSnapshotStore;
use z00z_utils::io::{read_file, write_file};

#[test]
fn test_id_roundtrip() {
    let snapshot = snap(&[(1, 7, 9)]);
    let (_dir, store, snap_id) = save(&snapshot);
    let loaded = store.load_snapshot(&snap_id).expect("load snapshot");

    assert_eq!(snapshot, loaded);
    assert_eq!(
        store.derive_snapshot_id(&loaded).expect("loaded id"),
        snap_id,
    );
}

#[test]
fn test_sidecar_skip_id() {
    let snapshot = snap(&[(1, 7, 9), (2, 8, 10)]);
    let (dir, store, snap_id) = save(&snapshot);
    let bin = bin_path(&dir, &snap_id);
    let before = read_file(&bin).expect("read before");

    write_file(
        dir.path().join("stage_4_snapshot.json"),
        br#"{"stage":4,"report_ref":"report.md","note":"sidecar"}"# as &[u8],
    )
    .expect("write stage file");
    write_file(
        dir.path().join("report.md"),
        b"# report\n\nstage=4\nref=stage_4_snapshot.json" as &[u8],
    )
    .expect("write report file");
    write_file(dir.path().join("custom-name.bin"), &bytes(&snapshot)).expect("write sidecar bin");

    let after = read_file(&bin).expect("read after");
    let loaded = store.load_snapshot(&snap_id).expect("load snapshot");

    assert_eq!(before, after);
    assert_eq!(
        store.derive_snapshot_id(&loaded).expect("loaded id"),
        snap_id,
    );
}
