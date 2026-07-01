use z00z_storage::settlement::{CheckRoot, SettlementStore, StoreItem, StoreOp};

use crate::proof_blob_fix::{self, WitCase};

pub struct WitPair {
    pub root: CheckRoot,
    pub left: WitCase,
    pub right: WitCase,
}

pub fn wit_pair(
    definition_id: [u8; 32],
    serial_id: u32,
    left_id: [u8; 32],
    right_id: [u8; 32],
) -> WitPair {
    let left_path = proof_blob_fix::make_path(definition_id, serial_id, left_id);
    let right_path = proof_blob_fix::make_path(definition_id, serial_id, right_id);
    let left_leaf = proof_blob_fix::make_leaf(left_id, serial_id);
    let right_leaf = proof_blob_fix::make_leaf(right_id, serial_id);

    let mut store = SettlementStore::new();
    let _ = store
        .apply_settlement_ops(vec![
            StoreOp::Put(Box::new(
                StoreItem::new(left_path, left_leaf.clone()).expect("left item"),
            )),
            StoreOp::Put(Box::new(
                StoreItem::new(right_path, right_leaf.clone()).expect("right item"),
            )),
        ])
        .expect("apply ops");
    let root = CheckRoot::from(store.settlement_root().expect("check root"));

    let (left_item, left_proof) = proof_blob_fix::load_blob(&store, &left_path);
    let (right_item, right_proof) = proof_blob_fix::load_blob(&store, &right_path);
    let left = proof_blob_fix::build_case(root, left_path, left_leaf, left_item, left_proof);
    let right = proof_blob_fix::build_case(root, right_path, right_leaf, right_item, right_proof);

    WitPair { root, left, right }
}
