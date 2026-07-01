use z00z_core::assets::AssetLeaf;
use z00z_storage::settlement::{
    chk_blob_settlement, DefinitionId, ProofBlob, ProofChkErr, SerialId, SettlementPath,
    SettlementStateRoot, SettlementStore, StoreItem, TerminalId, TerminalLeaf,
};

const README_DOC: &str = include_str!("../src/settlement/README.md");
const MOD_DOC: &str = include_str!("../src/settlement/mod.rs");

fn sample_path(serial_id: u32, byte: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new([7u8; 32]),
        SerialId::new(serial_id),
        TerminalId::new([byte; 32]),
    )
}

fn sample_leaf(path: SettlementPath, value: u32) -> TerminalLeaf {
    let mut leaf = TerminalLeaf::from(AssetLeaf::dummy_for_scan(value));
    leaf.asset_id = path.terminal_id().into_bytes();
    leaf.serial_id = path.serial_id.get();
    leaf
}

fn seeded_blob(serial_id: u32, byte: u8) -> (SettlementPath, ProofBlob) {
    let mut store = SettlementStore::new();
    let path = sample_path(serial_id, byte);
    let leaf = sample_leaf(path, u32::from(byte));
    store
        .put_settlement_item(StoreItem::new(path, leaf).expect("item"))
        .expect("put item");
    let blob = store.settlement_proof_blob(&path).expect("proof blob");
    (path, blob)
}

#[test]
fn readme_states_boundaries() {
    assert!(README_DOC.contains("Do not expose raw `jmt` proof or node types"));
    assert!(README_DOC.contains("Do not treat `backend_root` as the public state root"));
    assert!(README_DOC.contains("Do not collapse `definition_id`, `serial_id`, and `terminal_id`"));
    assert!(!MOD_DOC.contains("pub use jmt::"));
}

#[test]
fn backend_root_stays_private() {
    let (path, blob) = seeded_blob(5, 5);
    let proof = blob.item().clone();
    let bytes = blob.encode().expect("encode blob");

    let err = chk_blob_settlement(
        &bytes,
        SettlementStateRoot::settlement_v1(blob.backend_root()),
        &path,
        proof.def_leaf(),
        proof.ser_leaf(),
        proof.terminal_leaf().expect("asset leaf"),
    )
    .expect_err("backend root must not stand in for semantic root");

    assert_eq!(err, ProofChkErr::RootGenerationMix);
}

#[test]
fn backend_root_rejects_tamper() {
    let (path, blob) = seeded_blob(3, 7);
    let proof = blob.item().clone();
    let mut backend_root = blob.backend_root();
    backend_root[0] ^= 1;
    let bytes = ProofBlob::new(
        proof.clone(),
        blob.terminal_leaf_hash(),
        backend_root,
        blob.definition_proof().to_vec(),
        blob.serial_proof().to_vec(),
        blob.terminal_proof().to_vec(),
    )
    .encode()
    .expect("encode tampered blob");

    let err = chk_blob_settlement(
        &bytes,
        proof.settlement_root(),
        &path,
        proof.def_leaf(),
        proof.ser_leaf(),
        proof.terminal_leaf().expect("asset leaf"),
    )
    .expect_err("wrong backend root must reject");

    assert!(matches!(
        err,
        ProofChkErr::DefProofMix | ProofChkErr::SerProofMix | ProofChkErr::TerminalProofMix
    ));
}

#[test]
fn root_bind_rejects_tamper() {
    let (path, blob) = seeded_blob(1, 9);
    let proof = blob.item().clone();
    let bind_ver = blob.root_bind_ver();
    let mut wrong_bind = blob.root_bind();
    wrong_bind[0] ^= 1;
    let bytes = blob
        .with_root_bind(bind_ver, wrong_bind)
        .encode()
        .expect("encode tampered blob");

    let err = chk_blob_settlement(
        &bytes,
        proof.settlement_root(),
        &path,
        proof.def_leaf(),
        proof.ser_leaf(),
        proof.terminal_leaf().expect("asset leaf"),
    )
    .expect_err("root bind mix");

    assert_eq!(err, ProofChkErr::RootBindMix);
}
