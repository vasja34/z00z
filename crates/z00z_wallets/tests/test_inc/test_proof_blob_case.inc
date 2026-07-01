use z00z_storage::settlement::{CheckRoot, SettlementStore, StoreItem};

use crate::proof_blob_fix::{self, WitCase};

pub fn wit_case(definition_id: [u8; 32], serial_id: u32, asset_id: [u8; 32]) -> WitCase {
    let path = proof_blob_fix::make_path(definition_id, serial_id, asset_id);
    let leaf = proof_blob_fix::make_leaf(asset_id, serial_id);
    let mut store = SettlementStore::new();
    let item = StoreItem::new(path, leaf.clone()).expect("store item");
    let _ = store.put_settlement_item(item).expect("put item");
    let (proof_item, proof) = proof_blob_fix::load_blob(&store, &path);

    proof_blob_fix::build_case(
        CheckRoot::from(store.settlement_root().expect("check root")),
        path,
        leaf,
        proof_item,
        proof,
    )
}
