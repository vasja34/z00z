use z00z_core::assets::AssetLeaf;
use z00z_storage::{
    checkpoint::{
        decode_draft_json, decode_exec_json, CheckpointDraft, CheckpointExecInput,
        CheckpointExecOut, CheckpointExecTx, CheckpointExecVersion, CheckpointInRef,
        CheckpointVersion, CreatedEnt, SpentEnt,
    },
    settlement::{
        chk_blob_settlement, CheckRoot, DefinitionId, ProofChkErr, RootGeneration, SerialId,
        SettlementPath, SettlementStateRoot, SettlementStore, StoreItem, StoreOp, TerminalId,
        TerminalLeaf,
    },
    snapshot::PrepSnapshotId,
};

const TYPES_IDENTITY: &str = include_str!("../src/settlement/identity.rs");
const PROOF_SOURCE: &str = include_str!("../src/settlement/proof.rs");
const PROOF_BATCH_SOURCE: &str = include_str!("../src/settlement/proof_batch.rs");
const HJMT_PROOF_SOURCE: &str = include_str!("../src/settlement/hjmt_proof.rs");
const STORE_QUERY_SOURCE: &str = include_str!("../src/backend/query.rs");
const STORE_ROWS_SOURCE: &str = include_str!("../src/backend/rows.rs");
const CHECKPOINT_DRAFT_SOURCE: &str = include_str!("../src/checkpoint/artifact_proof_draft.rs");
const CHECKPOINT_STMT_SOURCE: &str = include_str!("../src/checkpoint/artifact_stmt.rs");
const CHECKPOINT_EXEC_SOURCE: &str = include_str!("../src/checkpoint/exec_input.rs");
const SNAPSHOT_STORE_SOURCE: &str = include_str!("../src/snapshot/store.rs");
const REDB_BACKEND_HJMT_SOURCE: &str = include_str!("../src/backend/redb/hjmt.rs");

fn bytes(value: u8) -> [u8; 32] {
    [value; 32]
}

fn json_array(value: u8) -> String {
    std::iter::repeat_n(value.to_string(), 32)
        .collect::<Vec<_>>()
        .join(", ")
}

fn test_item(mark: u8) -> StoreItem {
    let core = AssetLeaf::dummy_for_scan(u32::from(mark));
    let leaf = TerminalLeaf::from(core.clone());
    let path = SettlementPath::new(
        DefinitionId::new(bytes(mark)),
        SerialId::new(core.serial_id),
        TerminalId::new(core.asset_id),
    );
    StoreItem::new(path, leaf).expect("test item")
}

#[test]
fn test_settle_not_asset_alias() {
    let root = SettlementStateRoot::settlement_v1(bytes(9));
    assert_eq!(root.generation(), RootGeneration::SettlementV1);
    assert_eq!(root.generation_version(), 1);
    assert_eq!(root.into_bytes(), bytes(9));
    assert_eq!(SettlementStateRoot::from_version(1, bytes(9)), Some(root));
    assert_eq!(SettlementStateRoot::from_version(0, bytes(9)), None);

    assert!(!TYPES_IDENTITY.contains("pub type SettlementStateRoot"));
    assert!(!TYPES_IDENTITY.contains("impl From<AssetStateRoot> for SettlementStateRoot"));
    assert!(TYPES_IDENTITY.contains("generation: RootGeneration"));
    assert!(TYPES_IDENTITY.contains("impl From<SettlementStateRoot> for CheckRoot"));
}

#[test]
fn test_root_generation_binds_proof() {
    let mut store = SettlementStore::new();
    let item = test_item(11);
    let path = item.path();
    let root = store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(item.clone()))])
        .expect("settlement apply");
    let blob = store.settlement_proof_blob(&path).expect("proof blob");

    assert_eq!(root.generation(), RootGeneration::SettlementV1);
    assert_eq!(blob.item().settlement_root(), root);
    assert_eq!(blob.item().root(), root);
    assert_eq!(store.settlement_root().expect("settlement root"), root);
    assert_eq!(
        store
            .settlement_root_for_version(1)
            .expect("settlement root v1"),
        root
    );

    let checked = chk_blob_settlement(
        &blob.encode().expect("proof encode"),
        root,
        &path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        item.terminal_leaf().expect("asset leaf"),
    )
    .expect("settlement proof check");
    assert_eq!(checked.item().settlement_root(), root);

    let wrong = SettlementStateRoot::settlement_v1(bytes(0xEE));
    let err = chk_blob_settlement(
        &blob.encode().expect("proof encode"),
        wrong,
        &path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        item.terminal_leaf().expect("asset leaf"),
    )
    .expect_err("wrong settlement root must reject");
    assert_eq!(err, ProofChkErr::RootGenerationMix);

    assert!(PROOF_SOURCE.contains("item.settlement_root()"));
    assert!(PROOF_SOURCE.contains("generation_version()"));
    assert!(HJMT_PROOF_SOURCE.contains("ProofItem::new_settlement("));
    assert!(STORE_QUERY_SOURCE.contains("chk_blob_settlement("));
    assert!(SNAPSHOT_STORE_SOURCE.contains("chk_blob_settlement_inclusion("));
}

#[test]
fn test_root_generation_binds_checkpoint() {
    let prior = SettlementStateRoot::settlement_v1(bytes(1));
    let next = SettlementStateRoot::settlement_v1(bytes(2));
    let draft = CheckpointDraft::new_settlement(
        CheckpointVersion::CURRENT,
        77,
        prior,
        next,
        vec![SpentEnt::new(bytes(3))],
        vec![CreatedEnt::new(bytes(4), bytes(5))],
    );

    assert_eq!(draft.prev_root(), CheckRoot::from(prior));
    assert_eq!(draft.new_root(), CheckRoot::from(next));
    assert_eq!(draft.prev_settlement_root(), prior);
    assert_eq!(draft.new_settlement_root(), next);

    let proof = draft
        .attest_proof(
            PrepSnapshotId::new(bytes(6)),
            z00z_storage::checkpoint::CheckpointExecInputId::new(bytes(7)),
        )
        .expect("attest proof");
    let stmt = proof.statement();
    assert_eq!(stmt.prev_settlement_root(), prior);
    assert_eq!(stmt.new_settlement_root(), next);
    assert!(stmt.backend_payload().starts_with(&[1]));

    let artifact = draft.finalize(proof).expect("artifact");
    assert_eq!(artifact.prev_settlement_root(), prior);
    assert_eq!(artifact.new_settlement_root(), next);

    assert!(CHECKPOINT_DRAFT_SOURCE.contains("prev_settlement_root"));
    assert!(CHECKPOINT_DRAFT_SOURCE.contains("pub fn new_settlement("));
    assert!(CHECKPOINT_STMT_SOURCE.contains("backend_payload"));
    assert!(CHECKPOINT_STMT_SOURCE.contains("generation_version()"));
    assert!(STORE_ROWS_SOURCE.contains("CheckpointExecInput::new_settlement("));
    assert!(REDB_BACKEND_HJMT_SOURCE.contains("CheckpointDraft::new_settlement("));
}

#[test]
fn test_exports_generation_bridge() {
    let recovery = z00z_storage::settlement::SettlementRecoveryState::new(
        9,
        SettlementStateRoot::settlement_v1(bytes(0x21)),
        1,
        1,
        4,
        bytes(0x22),
        bytes(0x23),
    );
    let policy_set = recovery.live_policy_set_v1(9);
    let leaf = z00z_storage::settlement::ShardRootLeafV1::new(
        2,
        recovery.state_root.into_bytes(),
        14,
        7,
        bytes(0x24),
        policy_set.digest().expect("policy-set digest"),
        recovery.version,
        18,
        0,
    );
    let publication = z00z_storage::settlement::CheckpointPublicationV1::new(
        z00z_storage::settlement::RootGenerationTagV1::RootGeneration1,
        z00z_storage::settlement::PublicationModeTagV1::CheckpointWindow,
        44,
        bytes(0x24),
        recovery.state_root,
        vec![leaf],
    );

    assert_eq!(policy_set.members[0].policy_digest(), bytes(0x22));
    assert!(publication.digest().is_ok());
    publication
        .check_prior_root_v1(recovery.state_root)
        .expect("prior root continuity");
    assert!(PROOF_BATCH_SOURCE.contains("pub struct ShardRootLeafV1"));
    assert!(PROOF_BATCH_SOURCE.contains("pub struct CheckpointPublicationV1"));
    assert!(PROOF_BATCH_SOURCE.contains("pub struct PolicySetCommitmentV1"));
}

#[test]
fn test_old_asset_inputs_rejected() {
    assert!(SettlementStateRoot::from_version(0, bytes(1)).is_none());
    assert!(!PROOF_SOURCE
        .contains("pub fn chk_blob_settlement(\n    bytes: &[u8],\n    root: AssetStateRoot"));
    assert!(!CHECKPOINT_EXEC_SOURCE.contains("pub fn new_settlement(\n        version: CheckpointExecVersion,\n        prep_snapshot_id: PrepSnapshotId,\n        prev_root: CheckRoot"));

    let old_draft_json = format!(
        r#"{{
  "version": 1,
  "height": 1,
  "prev_root": [{}],
  "new_root": [{}],
  "spent_delta": [],
  "created_delta": []
}}"#,
        json_array(1),
        json_array(2)
    );
    assert!(decode_draft_json(old_draft_json.as_bytes()).is_err());
}

#[test]
fn test_rejected_with_state_preserved() {
    let mut store = SettlementStore::new();
    let item = test_item(21);
    let path = item.path();
    let root = store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(item.clone()))])
        .expect("settlement apply");
    let blob = store.settlement_proof_blob(&path).expect("proof blob");
    let before = store.settlement_root().expect("before root");

    assert!(SettlementStateRoot::from_version(0, root.into_bytes()).is_none());
    let err = chk_blob_settlement(
        &blob.encode().expect("proof encode"),
        SettlementStateRoot::settlement_v1(bytes(0x44)),
        &path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        item.terminal_leaf().expect("asset leaf"),
    )
    .expect_err("downgrade-like root substitution must reject");
    assert_eq!(err, ProofChkErr::RootGenerationMix);
    assert_eq!(store.settlement_root().expect("after root"), before);
}

#[test]
fn test_rejected_no_legacy_lane() {
    let old_exec_json = format!(
        r#"{{
  "version": 1,
  "prep_snapshot_id": [{}],
  "prev_root": [{}],
  "txs": []
}}"#,
        json_array(1),
        json_array(2)
    );
    assert!(decode_exec_json(old_exec_json.as_bytes()).is_err());

    let out = CheckpointExecOut::new(
        DefinitionId::new(bytes(9)),
        test_item(31).terminal_leaf().expect("asset leaf").clone(),
    )
    .expect("exec out");
    let tx = CheckpointExecTx::new(
        vec![CheckpointInRef::new(bytes(8), SerialId::new(31))],
        vec![out],
        vec![1],
    )
    .expect("exec tx");
    let exec = CheckpointExecInput::new_settlement(
        CheckpointExecVersion::CURRENT,
        PrepSnapshotId::new(bytes(1)),
        SettlementStateRoot::settlement_v1(bytes(2)),
        vec![tx],
    )
    .expect("settlement exec");
    assert_eq!(
        exec.prev_root(),
        CheckRoot::from(exec.prev_settlement_root())
    );
}

#[test]
fn test_reload_rejects_wrong_generation() {
    let bad_generation_json = format!(
        r#"{{
  "version": 1,
  "prep_snapshot_id": [{}],
  "prev_root": [{}],
  "prev_settlement_root": {{
    "generation": "LegacyAsset",
    "root": [{}]
  }},
  "txs": []
}}"#,
        json_array(1),
        json_array(2),
        json_array(2)
    );
    assert!(decode_exec_json(bad_generation_json.as_bytes()).is_err());

    let missing_generation_json = format!(
        r#"{{
  "version": 1,
  "height": 1,
  "prev_root": [{}],
  "new_root": [{}],
  "prev_settlement_root": {{
    "root": [{}]
  }},
  "new_settlement_root": {{
    "generation": "SettlementV1",
    "root": [{}]
  }},
  "spent_delta": [],
  "created_delta": []
}}"#,
        json_array(1),
        json_array(2),
        json_array(1),
        json_array(2)
    );
    assert!(decode_draft_json(missing_generation_json.as_bytes()).is_err());
}
