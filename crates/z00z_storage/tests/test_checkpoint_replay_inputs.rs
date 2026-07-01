use z00z_storage::fixture_support::snapshot_fix;

use z00z_core::assets::AssetLeaf;
use z00z_storage::{
    checkpoint::{
        check_exec_root, check_link_ids, decode_exec_bin, derive_exec_id, encode_exec_bin,
        CheckpointExecInput, CheckpointExecOut, CheckpointExecTx, CheckpointExecVersion,
        CheckpointInRef, CheckpointLink, CheckpointLinkVersion,
    },
    settlement::{CheckRoot, DefinitionId, SerialId, TerminalLeaf},
    snapshot::{PrepSnapshotId, PrepSnapshotStore},
    CheckpointError,
};

fn exec_with_proof(
    snapshot_id: PrepSnapshotId,
    prev_root: CheckRoot,
    tx_proof: Vec<u8>,
) -> CheckpointExecInput {
    CheckpointExecInput::new(
        CheckpointExecVersion::CURRENT,
        snapshot_id,
        prev_root,
        vec![CheckpointExecTx::new(
            vec![CheckpointInRef::new([1u8; 32], SerialId::new(1))],
            vec![CheckpointExecOut::new(
                DefinitionId::new([7u8; 32]),
                TerminalLeaf::from(AssetLeaf::dummy_for_scan(11)),
            )
            .expect("exec out")],
            tx_proof,
        )
        .expect("exec tx")],
    )
    .expect("exec input")
}

fn exec(snapshot_id: PrepSnapshotId, prev_root: CheckRoot) -> CheckpointExecInput {
    exec_with_proof(snapshot_id, prev_root, vec![9u8, 7u8, 5u8])
}

#[test]
fn test_preserves_exact_proof_bytes() {
    let tx_proof = b"verified-proof-v1".to_vec();
    let exec = exec_with_proof(
        PrepSnapshotId::new([2u8; 32]),
        CheckRoot::new([3u8; 32]),
        tx_proof.clone(),
    );

    let decoded = decode_exec_bin(&encode_exec_bin(&exec).expect("exec bytes")).expect("exec");

    assert_eq!(decoded.txs()[0].tx_proof(), tx_proof.as_slice());
}

#[test]
fn test_snap_id_mismatch_rejects() {
    let snapshot = snapshot_fix::snap(&[(1, 1, 1)]);
    let (_dir, store, snap_id) = snapshot_fix::save(&snapshot);
    let loaded = store.load_snapshot(&snap_id).expect("snapshot");
    let exec = exec(PrepSnapshotId::new([8u8; 32]), loaded.prev_root);
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        z00z_storage::checkpoint::CheckpointId::new([4u8; 32]),
        snap_id,
        derive_exec_id(&encode_exec_bin(&exec).expect("exec bytes")),
    )
    .expect("link");

    let err = check_link_ids(snap_id, &link, &exec).expect_err("snap id mismatch");

    assert!(matches!(err, CheckpointError::LinkMix));
}

#[test]
fn test_exec_id_mismatch_rejects() {
    let snapshot = snapshot_fix::snap(&[(1, 1, 1)]);
    let (_dir, store, snap_id) = snapshot_fix::save(&snapshot);
    let loaded = store.load_snapshot(&snap_id).expect("snapshot");
    let exec = exec(snap_id, loaded.prev_root);
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        z00z_storage::checkpoint::CheckpointId::new([4u8; 32]),
        snap_id,
        z00z_storage::checkpoint::CheckpointExecInputId::new([0u8; 32]),
    )
    .expect("link");

    let err = check_link_ids(snap_id, &link, &exec).expect_err("exec id mismatch");

    assert!(matches!(err, CheckpointError::ReplayMix));
}

#[test]
fn test_prev_root_mismatch_rejects() {
    let snapshot = snapshot_fix::snap(&[(1, 1, 1)]);
    let (_dir, store, snap_id) = snapshot_fix::save(&snapshot);
    let loaded = store.load_snapshot(&snap_id).expect("snapshot");
    let exec = exec(snap_id, CheckRoot::new([9u8; 32]));

    let err = check_exec_root(&loaded, &exec).expect_err("root mismatch");

    assert!(matches!(err, CheckpointError::RootMix));
}
