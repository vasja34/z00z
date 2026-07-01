use z00z_storage::{
    serialization::{build_artifact, render_jmt_view, JmtViewFmt},
    settlement::{
        DefinitionId, SerialId, SettlementPath, SettlementStore, StoreItem, TerminalId,
        TerminalLeaf,
    },
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
        [(1, 11, 21, 101), (1, 12, 23, 102), (2, 31, 41, 103)]
    {
        store
            .put_settlement_item(test_item(def_mark, serial_num, asset_mark, leaf_mark))
            .expect("put settlement item");
    }
    store
}

#[test]
fn test_output_contains_namespace_edges() {
    let store = seed_store();
    let artifact = build_artifact(&store).expect("build artifact");

    let first = render_jmt_view(&artifact, JmtViewFmt::Dot).expect("render dot");
    let second = render_jmt_view(&artifact, JmtViewFmt::Dot).expect("render dot again");

    assert_eq!(first.body(), second.body());
    assert!(first.body().contains("digraph jmt_artifact"));
    assert!(first.body().contains("tree:definition"));
    assert!(first.body().contains("tree:terminal"));
    assert!(first.body().contains("jmt_root="));
    assert!(first.body().contains("->"));
}

#[test]
fn test_output_contains_paths_hashes() {
    let store = seed_store();
    let artifact = build_artifact(&store).expect("build artifact");

    let first = render_jmt_view(&artifact, JmtViewFmt::Text).expect("render text");
    let second = render_jmt_view(&artifact, JmtViewFmt::Text).expect("render text again");

    assert_eq!(first.body(), second.body());
    assert!(first.body().contains("tree definition"));
    assert!(first.body().contains("settlement_path_order"));
    assert!(first.body().contains("terminal="));
    assert!(first.body().contains("jmt_root="));
    assert!(first.body().contains("key_hash="));
}
