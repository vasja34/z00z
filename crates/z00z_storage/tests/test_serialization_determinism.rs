use z00z_storage::{
    serialization::{
        build_artifact, decode_artifact, derive_artifact_id, encode_artifact, JmtSerVersion,
    },
    settlement::{
        DefinitionId, SerialId, SettlementPath, SettlementStore, StoreItem, TerminalId,
        TerminalLeaf,
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

fn seed_store() -> SettlementStore {
    let mut store = SettlementStore::new();

    for (def_mark, serial_num, asset_mark, leaf_mark) in
        [(1, 11, 21, 131), (1, 12, 23, 132), (2, 31, 41, 133)]
    {
        store
            .put_settlement_item(test_item(def_mark, serial_num, asset_mark, leaf_mark))
            .expect("put settlement item");
    }

    store
}

#[test]
fn test_unchanged_stable_bytes_id() {
    let store = seed_store();

    let first = build_artifact(&store).expect("build first artifact");
    let second = build_artifact(&store).expect("build second artifact");

    let first_bytes = encode_artifact(&first).expect("encode first artifact");
    let second_bytes = encode_artifact(&second).expect("encode second artifact");

    assert_eq!(first_bytes, second_bytes);
    assert_eq!(
        derive_artifact_id(&first).expect("first id"),
        derive_artifact_id(&second).expect("second id")
    );
}

#[test]
fn test_malformed_unsupported_typed_errors() {
    let malformed = decode_artifact(&[1u8, 2, 3]).expect_err("malformed bytes must fail");
    assert!(matches!(malformed, SerializationError::Codec(_)));

    let store = seed_store();
    let mut artifact = build_artifact(&store).expect("build artifact");
    artifact.version = JmtSerVersion::new(9);

    let err = encode_artifact(&artifact).expect_err("unsupported version must fail");
    assert!(matches!(err, SerializationError::VersionMix));
}

#[test]
fn test_nested_keys_root_nodes() {
    let mut store = SettlementStore::new();

    for asset_mark in [0x11, 0x12, 0x13] {
        store
            .put_settlement_item(test_item(1, 11, asset_mark, asset_mark.into()))
            .expect("put settlement item");
    }

    let artifact = build_artifact(&store).expect("build artifact");
    let nested_internal = artifact
        .nodes
        .iter()
        .filter(|node| {
            node.kind == z00z_storage::serialization::JmtSerNodeKind::Internal
                && !node.key.is_empty()
        })
        .count();

    assert!(
        nested_internal > 0,
        "expected nested internal JMT nodes, got only synthetic roots"
    );
}
