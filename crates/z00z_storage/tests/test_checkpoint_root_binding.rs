use z00z_storage::fixture_support::snapshot_fix;

use z00z_core::assets::AssetLeaf;
use z00z_storage::{
    checkpoint::{
        build_cp_draft, decode_exec_json, derive_exec_id, encode_exec_bin, encode_exec_json,
        CheckpointExecInput, CheckpointExecOut, CheckpointExecTx, CheckpointExecVersion,
        CheckpointInRef, CheckpointLink, CheckpointLinkVersion, SpentIndex, SpentIndexError,
        StateError, TxPkgSum, TxProofError, TxProofVerifier,
    },
    settlement::{
        chk_blob_settlement, CheckRoot, DefinitionId, ProofBlob, ProofChkErr, SerialId,
        SettlementPath, SettlementStateRoot, SettlementStore, StoreItem, TerminalId, TerminalLeaf,
    },
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

fn sample_exec_input() -> CheckpointExecInput {
    CheckpointExecInput::new(
        CheckpointExecVersion::CURRENT,
        z00z_storage::snapshot::PrepSnapshotId::new([0x44; 32]),
        CheckRoot::new([0x55; 32]),
        vec![CheckpointExecTx::new(
            vec![CheckpointInRef::new([0x66; 32], SerialId::new(7))],
            vec![CheckpointExecOut::new(
                DefinitionId::new([0x77; 32]),
                TerminalLeaf::from(AssetLeaf::dummy_for_scan(23)),
            )
            .expect("exec out")],
            vec![0x88],
        )
        .expect("exec tx")],
    )
    .expect("exec input")
}

#[test]
fn test_prev_root_bind_rejects() {
    let snapshot = snapshot_fix::snap(&[(1, 1, 1)]);
    let (_dir, store, snap_id) = snapshot_fix::save(&snapshot);
    let loaded = store.load_snapshot(&snap_id).expect("snapshot");
    let replay = store.replay_entries(&loaded).expect("replay");
    let exec = CheckpointExecInput::new(
        CheckpointExecVersion::CURRENT,
        snap_id,
        CheckRoot::new([9u8; 32]),
        vec![CheckpointExecTx::new(
            vec![CheckpointInRef::new([1u8; 32], SerialId::new(1))],
            vec![CheckpointExecOut::new(
                DefinitionId::new([7u8; 32]),
                TerminalLeaf::from(AssetLeaf::dummy_for_scan(11)),
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

    let err = build_cp_draft(
        7, snap_id, &loaded, &replay, &link, &exec, &PassProof, &NoSpent,
    )
    .expect_err("prev root mismatch");

    assert!(matches!(err, StateError::PrevRoot));
}

#[test]
fn test_exec_rejects_root_drift() {
    let exec = sample_exec_input();
    let bytes = encode_exec_json(&exec).expect("encode exec json");
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).expect("json value");
    value["prev_root"] = serde_json::json!(vec![0xAAu8; 32]);

    let err = decode_exec_json(
        serde_json::to_vec(&value)
            .expect("encode tampered json")
            .as_slice(),
    )
    .expect_err("tampered prev_root must reject");

    assert!(matches!(err, z00z_storage::CheckpointError::RootMix));
}

#[test]
fn test_root_bind_rejects_branch() {
    // Validation anchor: checkpoint root binding fails before branch acceptance.
    let mut store = SettlementStore::new();
    let path = SettlementPath::new(
        DefinitionId::new([7u8; 32]),
        SerialId::new(1),
        TerminalId::new([9u8; 32]),
    );
    let mut leaf = TerminalLeaf::from(AssetLeaf::dummy_for_scan(17));
    leaf.asset_id = path.terminal_id().into_bytes();
    leaf.serial_id = path.serial_id.get();
    store
        .put_settlement_item(StoreItem::new(path, leaf.clone()).expect("item"))
        .expect("put item");

    let blob = store.settlement_proof_blob(&path).expect("proof blob");
    let proof = blob.item().clone();
    let bind_ver = blob.root_bind_ver();
    let mut wrong_bind = blob.root_bind();
    wrong_bind[0] ^= 1;
    let bytes = blob
        .with_root_bind(bind_ver, wrong_bind)
        .encode()
        .expect("encode tampered blob");

    let err = chk_blob_settlement(
        &bytes,
        proof.settlement_root(),
        &path,
        proof.def_leaf(),
        proof.ser_leaf(),
        proof.terminal_leaf().expect("asset leaf"),
    )
    .expect_err("root bind mix");

    assert_eq!(err, ProofChkErr::RootBindMix);
}

#[test]
fn test_wrong_semantic_root_rejects() {
    let mut store = SettlementStore::new();
    let path = SettlementPath::new(
        DefinitionId::new([7u8; 32]),
        SerialId::new(2),
        TerminalId::new([8u8; 32]),
    );
    let mut leaf = TerminalLeaf::from(AssetLeaf::dummy_for_scan(18));
    leaf.asset_id = path.terminal_id().into_bytes();
    leaf.serial_id = path.serial_id.get();
    store
        .put_settlement_item(StoreItem::new(path, leaf.clone()).expect("item"))
        .expect("put item");

    let blob = store.settlement_proof_blob(&path).expect("proof blob");
    let proof = blob.item().clone();
    let bytes = blob.encode().expect("encode blob");
    let err = chk_blob_settlement(
        &bytes,
        SettlementStateRoot::settlement_v1([0xAAu8; 32]),
        &path,
        proof.def_leaf(),
        proof.ser_leaf(),
        proof.terminal_leaf().expect("asset leaf"),
    )
    .expect_err("wrong semantic root must reject");

    assert_eq!(err, ProofChkErr::RootGenerationMix);
}

#[test]
fn test_wrong_backend_root_rejects() {
    let mut store = SettlementStore::new();
    let path = SettlementPath::new(
        DefinitionId::new([7u8; 32]),
        SerialId::new(3),
        TerminalId::new([7u8; 32]),
    );
    let mut leaf = TerminalLeaf::from(AssetLeaf::dummy_for_scan(19));
    leaf.asset_id = path.terminal_id().into_bytes();
    leaf.serial_id = path.serial_id.get();
    store
        .put_settlement_item(StoreItem::new(path, leaf.clone()).expect("item"))
        .expect("put item");

    let blob = store.settlement_proof_blob(&path).expect("proof blob");
    let proof = blob.item().clone();
    let mut backend_root = blob.backend_root();
    backend_root[0] ^= 1;
    let bytes = ProofBlob::new(
        proof.clone(),
        blob.terminal_leaf_hash(),
        backend_root,
        blob.definition_proof().to_vec(),
        blob.serial_proof().to_vec(),
        blob.terminal_proof().to_vec(),
    )
    .encode()
    .expect("encode tampered blob");

    let err = chk_blob_settlement(
        &bytes,
        proof.settlement_root(),
        &path,
        proof.def_leaf(),
        proof.ser_leaf(),
        proof.terminal_leaf().expect("asset leaf"),
    )
    .expect_err("wrong backend root must reject");

    assert!(matches!(
        err,
        ProofChkErr::DefProofMix | ProofChkErr::SerProofMix | ProofChkErr::TerminalProofMix
    ));
}

#[test]
fn test_backend_root_mix() {
    let mut store = SettlementStore::new();
    let path = SettlementPath::new(
        DefinitionId::new([7u8; 32]),
        SerialId::new(5),
        TerminalId::new([5u8; 32]),
    );
    let mut leaf = TerminalLeaf::from(AssetLeaf::dummy_for_scan(21));
    leaf.asset_id = path.terminal_id().into_bytes();
    leaf.serial_id = path.serial_id.get();
    let root = store
        .put_settlement_item(StoreItem::new(path, leaf.clone()).expect("item"))
        .expect("put item");

    let blob = store.settlement_proof_blob(&path).expect("proof blob");
    let proof = blob.item().clone();
    let bytes = blob.encode().expect("encode blob");
    assert_ne!(blob.backend_root(), root.into_bytes());

    let err = chk_blob_settlement(
        &bytes,
        SettlementStateRoot::settlement_v1(blob.backend_root()),
        &path,
        proof.def_leaf(),
        proof.ser_leaf(),
        proof.terminal_leaf().expect("asset leaf"),
    )
    .expect_err("backend root must not stand in for checkpoint root context");

    assert_eq!(err, ProofChkErr::RootGenerationMix);
}

#[test]
fn test_bind_ver_rejects() {
    let mut store = SettlementStore::new();
    let path = SettlementPath::new(
        DefinitionId::new([7u8; 32]),
        SerialId::new(4),
        TerminalId::new([6u8; 32]),
    );
    let mut leaf = TerminalLeaf::from(AssetLeaf::dummy_for_scan(20));
    leaf.asset_id = path.terminal_id().into_bytes();
    leaf.serial_id = path.serial_id.get();
    store
        .put_settlement_item(StoreItem::new(path, leaf.clone()).expect("item"))
        .expect("put item");

    let blob = store.settlement_proof_blob(&path).expect("proof blob");
    let proof = blob.item().clone();
    let bind_ver = blob.root_bind_ver();
    let bind = blob.root_bind();
    let bytes = blob
        .with_root_bind(bind_ver.wrapping_add(1), bind)
        .encode()
        .expect("encode blob");

    let err = chk_blob_settlement(
        &bytes,
        proof.settlement_root(),
        &path,
        proof.def_leaf(),
        proof.ser_leaf(),
        proof.terminal_leaf().expect("asset leaf"),
    )
    .expect_err("wrong bind version must reject");

    assert_eq!(err, ProofChkErr::BindVerMix);
}
