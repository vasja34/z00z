use z00z_storage::settlement::{
    CheckRoot, DefinitionId, ProofItem, SerialId, SettlementPath, SettlementStore, TerminalId,
    TerminalLeaf,
};
use z00z_wallets::tx::{MemberWit, ResolvedInput};

pub struct InputCase {
    pub root: CheckRoot,
    pub input: ResolvedInput,
}

pub fn wit_input_case(definition_id: [u8; 32], serial_id: u32, asset_id: [u8; 32]) -> InputCase {
    let path = SettlementPath::new(
        DefinitionId::new(definition_id),
        SerialId::new(serial_id),
        TerminalId::new(asset_id),
    );
    let leaf = TerminalLeaf::from(z00z_core::assets::AssetLeaf {
        asset_id,
        serial_id,
        ..z00z_core::assets::AssetLeaf::default()
    });
    let item = z00z_storage::settlement::StoreItem::new(path, leaf.clone()).expect("store item");
    let mut store = SettlementStore::new();
    store.put_settlement_item(item).expect("put item");

    let root = CheckRoot::from(store.settlement_root().expect("check root"));
    let proof_item: ProofItem = store.settlement_proof_item(&path).expect("proof item");
    let proof = store
        .settlement_proof_blob(&path)
        .expect("proof blob")
        .encode()
        .expect("proof bytes");
    let member_wit = MemberWit::new(proof, proof_item).expect("member wit");

    InputCase {
        root,
        input: ResolvedInput::new(path, leaf, member_wit).expect("resolved input"),
    }
}
