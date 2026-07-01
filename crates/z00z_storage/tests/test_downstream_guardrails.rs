use tempfile::TempDir;
use z00z_core::assets::AssetLeaf;
use z00z_crypto::CLAIM_ROOT_VERSION;
use z00z_storage::{
    settlement::{
        CheckRoot, ClaimSourceRoot, DefinitionId, HjmtProofFamily, RightClass, RightLeaf, SerialId,
        SettlementLeaf, SettlementPath, SettlementStateRoot, SettlementStore, SnapItem, StoreItem,
        TerminalId, TerminalLeaf,
    },
    snapshot::{build_snapshot, PrepFsStore, PrepSnapshotError, PrepSnapshotStore},
};

use z00z_storage::fixture_support::guardrail::{
    assert_absent, assert_all_present, assert_each_absent, assert_present,
};

const CHECKPOINT_BUILD: &str = include_str!("../src/checkpoint/build.rs");
const CHECKPOINT_BUILD_STATE: &str = include_str!("../src/checkpoint/build_state.rs");
const CHECKPOINT_EXEC: &str = include_str!("../src/checkpoint/exec_input.rs");
const SNAPSHOT_STORE: &str = include_str!("../src/snapshot/store.rs");
const SETTLEMENT_TYPES: &str = include_str!("../src/settlement/identity.rs");
const STORE_QUERY: &str = include_str!("../src/backend/query.rs");
const WALLET_WITNESS: &str = include_str!("../../z00z_wallets/src/tx/state_witness.rs");
const WALLET_RESOLVED: &str = include_str!("../../z00z_wallets/src/tx/state_resolved_input.rs");
const WALLET_BACKEND: &str = include_str!("../../z00z_wallets/src/tx/spend_proof_backend.rs");
const WALLET_AUDIT: &str = include_str!("../../z00z_wallets/src/tx/commit_audit.rs");
const AGG_TYPES: &str = include_str!("../../z00z_runtime/aggregators/src/types.rs");
const AGG_IFACE: &str = include_str!("../../z00z_runtime/aggregators/src/service.rs");
const VERDICTS: &str = include_str!("../../z00z_runtime/validators/src/verdict.rs");
const WATCHER_EXPORT: &str = include_str!("../../z00z_runtime/watchers/src/evidence_export.rs");
const WATCHER_STATUS: &str = include_str!("../../z00z_runtime/watchers/src/status.rs");
const WATCHER_ENGINE: &str = include_str!("../../z00z_runtime/watchers/src/engine.rs");
const SIM_STAGE11: &str =
    include_str!("../../z00z_simulator/src/scenario_1/stage_11/jmt_wallet_scan.rs");

fn bytes(mark: u8) -> [u8; 32] {
    [mark; 32]
}

fn right_path(mark: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new(bytes(mark.wrapping_add(1))),
        SerialId::new(u32::from(mark) + 1),
        TerminalId::new(bytes(mark)),
    )
}

fn right_leaf(mark: u8) -> RightLeaf {
    RightLeaf {
        version: 1,
        terminal_id: TerminalId::new(bytes(mark)),
        right_class: RightClass::MachineCapability,
        issuer_scope: bytes(mark.wrapping_add(1)),
        provider_scope: bytes(mark.wrapping_add(2)),
        holder_commitment: bytes(mark.wrapping_add(3)),
        control_commitment: bytes(mark.wrapping_add(4)),
        beneficiary_commitment: bytes(mark.wrapping_add(5)),
        payload_commitment: bytes(mark.wrapping_add(6)),
        valid_from: 10,
        valid_until: 20,
        challenge_from: 12,
        challenge_until: 18,
        use_nonce: bytes(mark.wrapping_add(7)),
        revocation_policy_id: bytes(mark.wrapping_add(8)),
        transition_policy_id: bytes(mark.wrapping_add(9)),
        challenge_policy_id: bytes(mark.wrapping_add(10)),
        disclosure_policy_id: bytes(mark.wrapping_add(11)),
        retention_policy_id: bytes(mark.wrapping_add(12)),
    }
}

fn asset_path(mark: u8, serial: u32, asset: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new(bytes(mark)),
        SerialId::new(serial),
        TerminalId::new(bytes(asset)),
    )
}

fn asset_leaf(path: SettlementPath, mark: u32) -> TerminalLeaf {
    let mut core = AssetLeaf::dummy_for_scan(mark);
    core.asset_id = path.terminal_id().into_bytes();
    core.serial_id = path.serial_id.get();
    core.into()
}

#[test]
fn test_downstream_guardrails_shapes() {
    assert_present(
        "checkpoint build",
        CHECKPOINT_BUILD,
        "ClaimSourceRoot::new_settlement(",
    );
    assert_present("checkpoint exec", CHECKPOINT_EXEC, "SettlementPath::new(");
    assert_present("checkpoint state", CHECKPOINT_BUILD_STATE, "right_rows");
    assert_present(
        "settlement types",
        SETTLEMENT_TYPES,
        "pub const fn new_settlement(",
    );
    assert_present(
        "settlement types",
        SETTLEMENT_TYPES,
        "pub const fn settlement_root(self) -> SettlementStateRoot",
    );
    assert_present(
        "store query",
        STORE_QUERY,
        "ClaimSourceRoot::new_settlement(",
    );
    assert!(SNAPSHOT_STORE
        .contains("ModelErr::Right(_) | ModelErr::WrongLeafFamily => PrepSnapshotError::LeafMix"));
    assert_present("aggregator iface", AGG_IFACE, "PublicationRequest");
    assert_present("watcher engine", WATCHER_ENGINE, "PublicationRecord");
    assert_present("watcher status", WATCHER_STATUS, "PublicationState");
    assert_present(
        "watcher export",
        WATCHER_EXPORT,
        "publication: Option<PublicationRecord>",
    );

    assert_absent("checkpoint build", CHECKPOINT_BUILD, "AssetStateRoot::new(");
    assert_absent("checkpoint exec", CHECKPOINT_EXEC, "AssetPath::new(");
    assert_absent(
        "hjmt claim-source root",
        STORE_QUERY,
        "ClaimSourceRoot::new(CLAIM_ROOT_VERSION, self.hjmt_root()?)",
    );

    assert_each_absent(
        &[
            ("wallet witness", WALLET_WITNESS),
            ("wallet resolved", WALLET_RESOLVED),
            ("wallet backend", WALLET_BACKEND),
            ("sim stage11", SIM_STAGE11),
            ("aggregator types", AGG_TYPES),
            ("validator verdict", VERDICTS),
            ("aggregator iface", AGG_IFACE),
            ("watcher export", WATCHER_EXPORT),
            ("watcher status", WATCHER_STATUS),
            ("watcher engine", WATCHER_ENGINE),
        ],
        &[
            "TreeId",
            "ForestTreeId",
            "BucketId",
            "StorageBackend",
            "JournalBackend",
            "ReadTxn",
            "WriteTxn",
            "StoreBackendError",
        ],
    );

    assert_each_absent(
        &[
            ("wallet witness", WALLET_WITNESS),
            ("wallet resolved", WALLET_RESOLVED),
            ("wallet backend", WALLET_BACKEND),
            ("sim stage11", SIM_STAGE11),
        ],
        &["ProofBlob::decode", "AssetStateRoot", "AssetPath"],
    );

    assert_each_absent(
        &[
            ("checkpoint build", CHECKPOINT_BUILD),
            ("checkpoint state", CHECKPOINT_BUILD_STATE),
            ("checkpoint exec", CHECKPOINT_EXEC),
            ("snapshot store", SNAPSHOT_STORE),
            ("settlement types", SETTLEMENT_TYPES),
            ("store query", STORE_QUERY),
        ],
        &["PublicationRecord", "PublicationRequest", "OnionNet"],
    );

    assert_all_present(
        "sim stage11",
        SIM_STAGE11,
        &[
            "skipped_non_asset_count",
            "proof_blob+chk_blob_settlement before ownership detection",
        ],
    );
    assert_present("wallet audit", WALLET_AUDIT, "not a state authority root");
    assert_present("aggregator types", AGG_TYPES, "CheckpointPubIn");
    assert_present("aggregator types", AGG_TYPES, "created_leaves");
    assert_present("aggregator types", AGG_TYPES, "SettlementLeaf");
    assert_present("validator verdict", VERDICTS, "CheckpointArtifact");
    assert_absent("aggregator types", AGG_TYPES, "AssetLeaf");
}

#[test]
fn test_downstream_guardrails_claimroot() {
    let settlement_root = SettlementStateRoot::settlement_v1([0x51; 32]);
    let claim_root = ClaimSourceRoot::new_settlement(CLAIM_ROOT_VERSION, settlement_root);

    assert_eq!(claim_root.into_bytes(), settlement_root.into_bytes());
    assert_eq!(claim_root.root().into_bytes(), settlement_root.into_bytes());
    assert_eq!(claim_root.settlement_root(), settlement_root);
}

#[test]
fn test_snapshot_reloads_right_leaf() {
    let mut store = SettlementStore::new();
    let path = right_path(0x21);
    let leaf = right_leaf(0x21);
    store
        .put_settlement_item(StoreItem::new(path, leaf).expect("right item"))
        .expect("put right item");

    let witness = store
        .settlement_proof_blob(&path)
        .expect("right proof blob")
        .encode()
        .expect("encode right proof blob");
    let entry = z00z_storage::settlement::SnapItem::new(path, leaf, witness).expect("snap item");
    let (snapshot, snapshot_id) = z00z_storage::snapshot::build_snapshot(
        z00z_storage::settlement::CheckRoot::from(
            store.settlement_root().expect("settlement root"),
        ),
        vec![entry],
    )
    .expect("build snapshot");

    let temp = TempDir::new().expect("temp dir");
    let mut fs_store = PrepFsStore::new(temp.path());
    let saved_id = fs_store.save_snapshot(&snapshot).expect("save snapshot");
    assert_eq!(saved_id, snapshot_id);

    let loaded = fs_store.load_snapshot(&snapshot_id).expect("load snapshot");
    let replay = fs_store.replay_entries(&loaded).expect("replay entries");
    let replay_leaf = replay
        .first()
        .expect("one replay row")
        .proof_item()
        .leaf()
        .clone();

    assert_eq!(replay.len(), 1);
    assert_eq!(loaded.entries.len(), 1);
    assert_eq!(
        replay
            .first()
            .expect("one replay row")
            .proof_item()
            .settlement_root(),
        store.settlement_root().expect("settlement root"),
    );
    assert_eq!(replay.first().expect("one replay row").item().path(), path);
    assert_eq!(replay_leaf, SettlementLeaf::Right(leaf));
}

#[test]
fn test_snapshot_rejects_bad_family() {
    let mut store = SettlementStore::new();
    let path = asset_path(0x41, 17, 0x51);
    let leaf = asset_leaf(path, 7_777);
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
