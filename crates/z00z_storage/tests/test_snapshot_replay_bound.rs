use z00z_storage::fixture_support::snapshot_fix::{save, snap};
use z00z_storage::snapshot::PrepSnapshotStore;

const TX_LANE_SRC: &str =
    include_str!("../../z00z_simulator/src/scenario_1/stage_6/tx_lane_impl.rs");
const BUNDLE_LANE_SRC: &str =
    include_str!("../../z00z_simulator/src/scenario_1/stage_9/bundle_lane_impl.rs");
const PREP_LOADER_SRC: &str =
    include_str!("../../z00z_simulator/src/scenario_1/stage_9/prep_snapshot_loader.rs");
const SNAP_MOD: &str = include_str!("../src/snapshot/mod.rs");
const SNAP_STORE: &str = include_str!("../src/snapshot/store.rs");
const SNAP_CODEC: &str = include_str!("../src/snapshot/codec.rs");
const SNAP_TYPES: &str = include_str!("../src/snapshot/types.rs");
const SNAP_ERR: &str = include_str!("../src/snapshot/error.rs");

#[test]
fn test_replay_ok_no_exec() {
    // Validation anchor: snapshot validation and replay-entry recovery stay storage-owned.
    let snapshot = snap(&[(1, 7, 9), (2, 8, 10)]);
    let (_, store, _) = save(&snapshot);

    store
        .validate_snapshot(&snapshot)
        .expect("validate snapshot");
    let replay = store.replay_entries(&snapshot).expect("replay entries");

    assert_eq!(replay.len(), snapshot.entries.len());
}

#[test]
fn test_stage_io_use_store() {
    assert!(TX_LANE_SRC.contains("PrepFsStore"));
    assert!(TX_LANE_SRC.contains("save_snapshot("));
    assert!(BUNDLE_LANE_SRC.contains("load_prep("));
    assert!(PREP_LOADER_SRC.contains("PrepFsStore"));
    assert!(PREP_LOADER_SRC.contains("load_snapshot("));
    assert!(!BUNDLE_LANE_SRC.contains("prep_store(rows)"));
}

#[test]
fn test_mods_skip_sim() {
    for src in [SNAP_MOD, SNAP_STORE, SNAP_CODEC, SNAP_TYPES, SNAP_ERR] {
        assert!(!src.contains("z00z_simulator"));
        assert!(!src.contains("PrepFile"));
        assert!(!src.contains("PrepRow"));
    }
}
