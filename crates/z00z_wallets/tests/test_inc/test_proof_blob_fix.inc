use z00z_storage::settlement::{
    CheckRoot, DefinitionId, ProofItem, SerialId, SettlementPath, SettlementStore, TerminalId,
    TerminalLeaf,
};
use z00z_wallets::tx::{MemberWit, ResolvedInput};

pub struct WitCase {
    pub root: CheckRoot,
    pub path: SettlementPath,
    pub leaf: TerminalLeaf,
    pub proof_item: ProofItem,
    pub proof: Vec<u8>,
    pub input: ResolvedInput,
}

pub(crate) fn make_path(
    definition_id: [u8; 32],
    serial_id: u32,
    asset_id: [u8; 32],
) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new(definition_id),
        SerialId::new(serial_id),
        TerminalId::new(asset_id),
    )
}

pub(crate) fn make_leaf(asset_id: [u8; 32], serial_id: u32) -> TerminalLeaf {
    TerminalLeaf::from(z00z_core::assets::AssetLeaf {
        asset_id,
        serial_id,
        ..z00z_core::assets::AssetLeaf::default()
    })
}

pub(crate) fn load_blob(store: &SettlementStore, path: &SettlementPath) -> (ProofItem, Vec<u8>) {
    let proof_item = store.settlement_proof_item(path).expect("proof item");
    let proof = store
        .settlement_proof_blob(path)
        .expect("proof blob")
        .encode()
        .expect("proof bytes");
    (proof_item, proof)
}

pub(crate) fn build_case(
    root: CheckRoot,
    path: SettlementPath,
    leaf: TerminalLeaf,
    proof_item: ProofItem,
    proof: Vec<u8>,
) -> WitCase {
    let member_wit = MemberWit::new(proof.clone(), proof_item.clone()).expect("member wit");

    WitCase {
        root,
        path,
        leaf: leaf.clone(),
        proof_item,
        proof,
        input: ResolvedInput::new(path, leaf, member_wit).expect("resolved input"),
    }
}
