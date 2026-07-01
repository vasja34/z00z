#[path = "test_range_proof_env.inc"]
mod test_common;

use rand::{rngs::StdRng, SeedableRng};
use z00z_core::{genesis::asset_std::asset_from_dev_class, AssetClass, AssetWire};
use z00z_storage::settlement::{
    CheckRoot, DefinitionId, SerialId, SettlementPath, SettlementStore, StoreItem, TerminalId,
};
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    stealth::{bind_stealth_output_wire, build_output_bundle_with_rng},
    tx::{
        asset_wire_to_leaf, build_public_spend_contract, prepare_spend_public_inputs,
        SpendMembershipWitness, SpendProofWitness, TxOutRole, TxOutputWire, TxWire,
    },
};

pub const CHAIN_ID: u32 = 3;
pub const TX_VERSION: u8 = 1;
pub const CHAIN_TYPE: &str = "phase040_test_chain";
pub const CHAIN_NAME: &str = "phase040_test_net";

fn recv_sec() -> [u8; 32] {
    [0x11u8; 32]
}

pub fn receiver_keys() -> ReceiverKeys {
    ReceiverKeys::from_receiver_secret(
        ReceiverSecret::from_bytes(recv_sec()).expect("receiver secret"),
    )
    .expect("receiver keys")
}

fn build_membership_witness(
    input_wire: &AssetWire,
    tx_input: &z00z_wallets::tx::TxInputWire,
) -> (CheckRoot, SpendMembershipWitness) {
    let mut leaf = asset_wire_to_leaf(input_wire).expect("input leaf");
    let asset_id: [u8; 32] = hex::decode(&tx_input.asset_id_hex)
        .expect("input asset id hex")
        .try_into()
        .expect("input asset id bytes");
    leaf.asset_id = asset_id;

    let path = SettlementPath::new(
        DefinitionId::new(input_wire.definition.id),
        SerialId::new(tx_input.serial_id),
        TerminalId::new(asset_id),
    );
    let mut store = SettlementStore::new();
    let item = StoreItem::new(path, leaf.clone()).expect("store item");
    let _ = store.put_settlement_item(item).expect("put input item");
    let root = CheckRoot::from(store.settlement_root().expect("check root"));
    let proof_item = store.settlement_proof_item(&path).expect("proof item");
    let proof = store
        .settlement_proof_blob(&path)
        .expect("proof blob")
        .encode()
        .expect("proof bytes");
    let witness =
        SpendMembershipWitness::new(path, leaf, proof, proof_item).expect("membership witness");
    (root, witness)
}

fn canonical_input_fixture() -> (
    AssetWire,
    z00z_wallets::tx::TxInputWire,
    CheckRoot,
    SpendProofWitness,
) {
    let keys = receiver_keys();
    let card = keys.export_receiver_card().expect("card");
    let input_asset = asset_from_dev_class(AssetClass::Coin, 7, 55).expect("input asset");
    let mut rng = StdRng::seed_from_u64(0x0400_0001);
    let bundle = build_output_bundle_with_rng(
        "phase040-input".to_string(),
        TxOutRole::Recipient,
        AssetClass::Coin,
        &card,
        input_asset.amount,
        input_asset.serial_id,
        &mut rng,
    )
    .expect("output bundle");
    let input_wire = bind_stealth_output_wire(AssetWire::from_asset(&input_asset), &bundle.leaf)
        .expect("input wire");
    let tx_input = z00z_wallets::tx::TxInputWire {
        asset_id_hex: hex::encode(
            asset_wire_to_leaf(&input_wire)
                .expect("input leaf ref")
                .asset_id,
        ),
        serial_id: input_wire.serial_id,
    };
    let (prev_root, membership) = build_membership_witness(&input_wire, &tx_input);
    let witness = SpendProofWitness {
        receiver_secret: ReceiverSecret::from_bytes(recv_sec()).expect("receiver secret"),
        input_s_in: vec![bundle.s_out],
        membership: vec![membership],
    };

    (input_wire, tx_input, prev_root, witness)
}

pub fn canonical_proof_witness() -> SpendProofWitness {
    let (_, _, _, witness) = canonical_input_fixture();
    witness
}

pub fn canonical_public_contract_tx() -> (TxWire, CheckRoot) {
    let _guard = test_common::RangeProofEnvGuard::new();
    let keys = receiver_keys();
    let (input_wire, tx_input, prev_root, proof_witness) = canonical_input_fixture();
    let mut output_wire = input_wire.clone();
    output_wire.leaf_ad_id = Some([0x77; 32]);

    let tx_output = TxOutputWire {
        role: TxOutRole::Recipient,
        asset_wire: z00z_core::assets::AssetPkgWire::from_wire(&output_wire),
    };
    let proof_inputs = prepare_spend_public_inputs(
        CHAIN_ID,
        recv_sec(),
        std::slice::from_ref(&input_wire),
        std::slice::from_ref(&tx_input),
    )
    .expect("proof inputs");
    let mut tx = TxWire {
        tx_type: "regular_tx".to_string(),
        inputs: vec![tx_input],
        outputs: vec![tx_output],
        fee: 0,
        nonce: 0,
        context: Default::default(),
        proof: Default::default(),
        auth: Default::default(),
    };
    let (proof, auth) = build_public_spend_contract(
        &keys,
        CHAIN_ID,
        TX_VERSION,
        CHAIN_TYPE,
        CHAIN_NAME,
        &tx,
        prev_root,
        proof_inputs,
        proof_witness,
    )
    .expect("public spend contract");
    tx.proof = proof;
    tx.auth = auth;

    (tx, prev_root)
}
