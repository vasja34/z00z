use z00z_storage::{
    serialization::{build_artifact, restore_artifact, JmtSerNodeKind, JmtSerTreeId},
    settlement::{
        DefinitionId, RightClass, RightLeaf, SerialId, SettlementLeaf, SettlementPath,
        SettlementStore, StoreItem, TerminalId, TerminalLeaf,
    },
    SerializationError,
};

fn test_item(def_mark: u8, serial_num: u32, asset_mark: u8, leaf_mark: u32) -> StoreItem {
    let definition_id = DefinitionId::new([def_mark; 32]);
    let serial_id = SerialId::new(serial_num);
    let asset_id = TerminalId::new([asset_mark; 32]);
    let path = SettlementPath::new(definition_id, serial_id, asset_id);
    let mut leaf = TerminalLeaf::dummy_for_scan(leaf_mark);
    leaf.asset_id = asset_id.into_bytes();
    leaf.serial_id = serial_id.get();
    StoreItem::new(path, leaf).expect("serialization item")
}

fn right_item(mark: u8) -> StoreItem {
    let path = SettlementPath::new(
        DefinitionId::new([mark.wrapping_add(1); 32]),
        SerialId::new(u32::from(mark) + 1),
        TerminalId::new([mark; 32]),
    );
    let leaf = RightLeaf {
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
    };
    StoreItem::new(path, leaf).expect("right serialization item")
}

fn seed_store() -> SettlementStore {
    let mut store = SettlementStore::new();

    for (def_mark, serial_num, asset_mark, leaf_mark) in
        [(1, 11, 21, 121), (1, 12, 23, 122), (2, 31, 41, 123)]
    {
        store
            .put_settlement_item(test_item(def_mark, serial_num, asset_mark, leaf_mark))
            .expect("put settlement item");
    }

    store
}

#[test]
fn test_restore_keeps_root_bindings() {
    let store = seed_store();
    let artifact = build_artifact(&store).expect("build artifact");

    let restored = restore_artifact(&artifact).expect("restore artifact");

    assert_eq!(restored.node_count(), artifact.nodes.len());
    assert_eq!(restored.edge_count(), artifact.edges.len());
    assert_eq!(restored.tree_count(), artifact.roots.trees.len());
    assert!(restored.trees().iter().all(|tree| tree.is_root_bound));
    assert!(restored
        .trees()
        .iter()
        .all(|tree| tree.root == tree.jmt_root));
}

#[test]
fn test_tamper_tree_root_binding() {
    let store = seed_store();
    let mut artifact = build_artifact(&store).expect("build artifact");
    artifact.roots.trees[0].root[0] ^= 0xFF;

    let err = restore_artifact(&artifact).expect_err("tampered tree root must fail");
    assert!(matches!(err, SerializationError::RootMix));
}

#[test]
fn test_fails_required_root_missing() {
    let store = seed_store();
    let mut artifact = build_artifact(&store).expect("build artifact");
    artifact
        .nodes
        .retain(|node| node.kind != z00z_storage::serialization::JmtSerNodeKind::Internal);
    artifact.meta.node_count = artifact.nodes.len() as u32;

    let err = restore_artifact(&artifact).expect_err("missing root node must fail");
    assert!(matches!(
        err,
        SerializationError::RootMix | SerializationError::RebuildMix
    ));
}

#[test]
fn test_fails_missing_serialized_root() {
    let store = seed_store();
    let mut artifact = build_artifact(&store).expect("build artifact");
    artifact.roots.trees.pop();

    let err = restore_artifact(&artifact).expect_err("undeclared tree must fail restore");
    assert!(matches!(err, SerializationError::RebuildMix));
}

#[test]
fn test_fails_not_tree_root() {
    let store = seed_store();
    let mut artifact = build_artifact(&store).expect("build artifact");
    let tree_root = artifact
        .roots
        .trees
        .iter()
        .find(|tree| tree.tree_id == artifact.edges[0].tree_id)
        .cloned()
        .expect("tree root for first edge");
    let root_id = artifact
        .nodes
        .iter()
        .find(|node| node.tree_id == tree_root.tree_id && node.node_hash == tree_root.jmt_root)
        .map(|node| node.id)
        .expect("root node id");

    let edge = artifact
        .edges
        .iter_mut()
        .find(|edge| edge.tree_id == tree_root.tree_id)
        .expect("edge in rooted tree");
    edge.child = root_id;

    let err = restore_artifact(&artifact).expect_err("root with incoming edge must fail");
    assert!(matches!(
        err,
        SerializationError::RootMix | SerializationError::RebuildMix
    ));
}

#[test]
fn test_handles_right_leaf_artifact() {
    let mut store = SettlementStore::new();
    store
        .put_settlement_item(right_item(62))
        .expect("put right settlement item");
    let artifact = build_artifact(&store).expect("build artifact");

    let restored = restore_artifact(&artifact).expect("restore artifact");

    assert_eq!(restored.node_count(), artifact.nodes.len());
    assert!(restored.trees().iter().all(|tree| tree.is_root_bound));

    let right_payload = restored
        .artifact()
        .nodes
        .iter()
        .find(|node| {
            matches!(node.kind, JmtSerNodeKind::Leaf)
                && matches!(node.tree_id, JmtSerTreeId::Terminal { .. })
        })
        .expect("terminal right leaf node");
    let decoded = SettlementLeaf::decode(&right_payload.payload).expect("decode right payload");
    let right = decoded.as_right().expect("right leaf payload");
    assert_eq!(right.terminal_id, TerminalId::new([62u8; 32]));
    assert_eq!(right.right_class, RightClass::MachineCapability);
}
