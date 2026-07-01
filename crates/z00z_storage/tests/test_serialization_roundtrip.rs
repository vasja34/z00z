use tempfile::TempDir;
use z00z_storage::{
    serialization::{
        build_artifact, decode_artifact, derive_artifact_id, encode_artifact, JmtFsStore,
        JmtSerNodeKind, JmtSerStore,
    },
    settlement::{
        DefinitionId, RightClass, RightLeaf, SerialId, SettlementPath, SettlementStore, StoreItem,
        TerminalId, TerminalLeaf,
    },
};

fn test_path(def_mark: u8, serial_num: u32, asset_mark: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new([def_mark; 32]),
        SerialId::new(serial_num),
        TerminalId::new([asset_mark; 32]),
    )
}

fn test_item(def_mark: u8, serial_num: u32, asset_mark: u8, leaf_mark: u32) -> StoreItem {
    let path = test_path(def_mark, serial_num, asset_mark);
    let mut leaf = TerminalLeaf::dummy_for_scan(leaf_mark);
    leaf.asset_id = path.terminal_id().into_bytes();
    leaf.serial_id = path.serial_id.get();
    StoreItem::new(path, leaf).expect("serialization item")
}

fn right_path(mark: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new([mark.wrapping_add(1); 32]),
        SerialId::new(u32::from(mark) + 1),
        TerminalId::new([mark; 32]),
    )
}

fn right_leaf(mark: u8) -> RightLeaf {
    RightLeaf {
        version: 1,
        terminal_id: TerminalId::new([mark; 32]),
        right_class: RightClass::MachineCapability,
        issuer_scope: [mark.wrapping_add(1); 32],
        provider_scope: [mark.wrapping_add(2); 32],
        holder_commitment: [mark.wrapping_add(3); 32],
        control_commitment: [mark.wrapping_add(4); 32],
        beneficiary_commitment: [mark.wrapping_add(5); 32],
        payload_commitment: [mark.wrapping_add(6); 32],
        valid_from: 10,
        valid_until: 20,
        challenge_from: 12,
        challenge_until: 18,
        use_nonce: [mark.wrapping_add(6); 32],
        revocation_policy_id: [mark.wrapping_add(7); 32],
        transition_policy_id: [mark.wrapping_add(8); 32],
        challenge_policy_id: [mark.wrapping_add(9); 32],
        disclosure_policy_id: [mark.wrapping_add(10); 32],
        retention_policy_id: [mark.wrapping_add(11); 32],
    }
}

fn right_item(mark: u8) -> StoreItem {
    StoreItem::new(right_path(mark), right_leaf(mark)).expect("right serialization item")
}

fn seed_store() -> SettlementStore {
    let mut store = SettlementStore::new();
    for (def_mark, serial_num, asset_mark, leaf_mark) in
        [(1, 11, 21, 111), (1, 11, 22, 112), (2, 31, 41, 113)]
    {
        store
            .put_settlement_item(test_item(def_mark, serial_num, asset_mark, leaf_mark))
            .expect("put settlement item");
    }
    store
}

fn temp_store() -> (TempDir, JmtFsStore) {
    let dir = TempDir::new().expect("temp dir");
    let store = JmtFsStore::new(dir.path());
    (dir, store)
}

#[test]
fn test_roundtrip_preserves_typed_structure() {
    let store = seed_store();
    let artifact = build_artifact(&store).expect("build artifact");

    let bytes = encode_artifact(&artifact).expect("encode artifact");
    let decoded = decode_artifact(&bytes).expect("decode artifact");

    assert_eq!(decoded, artifact);
    assert_eq!(
        decoded.meta.path_order.len(),
        artifact.meta.path_order.len()
    );
    assert_eq!(decoded.meta.node_count as usize, decoded.nodes.len());
    assert_eq!(decoded.meta.edge_count as usize, decoded.edges.len());
}

#[test]
fn test_payloads_match_live_state() {
    let store = seed_store();
    let artifact = build_artifact(&store).expect("build artifact");

    for node in artifact
        .nodes
        .iter()
        .filter(|node| matches!(node.kind, z00z_storage::serialization::JmtSerNodeKind::Leaf))
    {
        assert!(
            !node.payload.is_empty(),
            "live store leaf payloads must survive into the artifact"
        );
        assert!(
            node.value_hash.is_some(),
            "leaf nodes sourced from live state must keep a value hash"
        );
    }
}

#[test]
fn test_artifact_loads_id_structure() {
    let store = seed_store();
    let artifact = build_artifact(&store).expect("build artifact");
    let (_dir, mut fs_store) = temp_store();

    let saved_id = fs_store.save_artifact(&artifact).expect("save artifact");
    let loaded = fs_store.load_artifact(&saved_id).expect("load artifact");
    let loaded_id = fs_store
        .derive_artifact_id(&loaded)
        .expect("derive reloaded artifact id");

    assert_eq!(loaded, artifact);
    assert_eq!(loaded_id, saved_id);
    assert_eq!(
        loaded_id,
        derive_artifact_id(&artifact).expect("derive original id")
    );
}

#[test]
fn test_store_builds_settle_artifact() {
    let store = SettlementStore::new();
    let artifact = build_artifact(&store).expect("live store must build artifact");

    assert_eq!(
        artifact.roots.sem_root,
        store.settlement_root().expect("settlement root")
    );
}

#[test]
fn test_keeps_right_leaf_payloads() {
    let mut store = SettlementStore::new();
    store
        .put_settlement_item(right_item(61))
        .expect("put right settlement item");

    let artifact = build_artifact(&store).expect("build artifact");
    let bytes = encode_artifact(&artifact).expect("encode artifact");
    let decoded = decode_artifact(&bytes).expect("decode artifact");

    assert_eq!(decoded, artifact);
    assert!(decoded.nodes.iter().any(|node| {
        matches!(node.kind, JmtSerNodeKind::Leaf) && node.payload.first() == Some(&2)
    }));
}
