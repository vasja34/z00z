use z00z_storage::fixture_support::snapshot_fix;

use z00z_core::assets::AssetLeaf;
use z00z_storage::{
    checkpoint::{
        build_cp_draft, derive_exec_id, encode_exec_bin, CheckpointExecInput, CheckpointExecOut,
        CheckpointExecTx, CheckpointExecVersion, CheckpointInRef, CheckpointLink,
        CheckpointLinkVersion, SpentIndex, SpentIndexError, TxPkgSum, TxProofError,
        TxProofVerifier,
    },
    settlement::{DefinitionId, SerialId, TerminalLeaf},
    snapshot::PrepSnapshotStore,
};

struct NoSpent;
impl SpentIndex for NoSpent {
    fn is_spent(
        &self,
        _prev: z00z_storage::settlement::CheckRoot,
        _curr: z00z_storage::settlement::CheckRoot,
        _id: &z00z_storage::settlement::TerminalId,
    ) -> Result<bool, SpentIndexError> {
        Ok(false)
    }
}

struct PassProof;
impl TxProofVerifier for PassProof {
    fn verify_tx(&self, _tx: &TxPkgSum) -> Result<(), TxProofError> {
        Ok(())
    }
}

fn run(mark: u8) -> [u8; 32] {
    let snapshot = snapshot_fix::snap(&[(1, 1, 1)]);
    let (_dir, store, snap_id) = snapshot_fix::save(&snapshot);
    let loaded = store.load_snapshot(&snap_id).expect("snapshot");
    let replay = store.replay_entries(&loaded).expect("replay");
    let exec = CheckpointExecInput::new(
        CheckpointExecVersion::CURRENT,
        snap_id,
        loaded.prev_root,
        vec![CheckpointExecTx::new(
            vec![CheckpointInRef::new([1u8; 32], SerialId::new(1))],
            vec![CheckpointExecOut::new(
                DefinitionId::new([7u8; 32]),
                TerminalLeaf::from(AssetLeaf::dummy_for_scan(mark as u32)),
            )
            .expect("exec out")],
            vec![9u8],
        )
        .expect("exec tx")],
    )
    .expect("exec input");
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        z00z_storage::checkpoint::CheckpointId::new([4u8; 32]),
        snap_id,
        derive_exec_id(&encode_exec_bin(&exec).expect("exec bytes")),
    )
    .expect("link");
    let draft = build_cp_draft(
        7, snap_id, &loaded, &replay, &link, &exec, &PassProof, &NoSpent,
    )
    .expect("draft");

    draft.created_delta()[0].leaf_hash().to_owned()
}

#[test]
fn test_leaf_hash_changes_leaf() {
    assert_ne!(run(11), run(12));
}
