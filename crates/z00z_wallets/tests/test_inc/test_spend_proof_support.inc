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
        asset_wire_to_leaf, build_public_spend_contract, build_tx_package_digest,
        prepare_spend_public_inputs, SpendInputLeaf, SpendInputRef, SpendMembershipWitness,
        SpendProofArtifact, SpendProofStmt, SpendProofWire, SpendProofWitness, TxInputWire,
        TxOutRole, TxOutputWire, TxProofWire, TxWire,
    },
};

pub const CHAIN_ID: u32 = 3;
pub const TX_VERSION: u8 = 1;
pub const CHAIN_TYPE: &str = "phase040_test_chain";
pub const CHAIN_NAME: &str = "phase040_test_net";

pub fn recv_sec() -> [u8; 32] {
    [0x11u8; 32]
}

fn receiver_keys() -> ReceiverKeys {
    ReceiverKeys::from_receiver_secret(
        ReceiverSecret::from_bytes(recv_sec()).expect("receiver secret"),
    )
    .expect("receiver keys")
}

fn build_membership_witness(
    input_wire: &AssetWire,
    tx_input: &TxInputWire,
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

fn canonical_input_fixture() -> (AssetWire, TxInputWire, CheckRoot, SpendProofWitness) {
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
    let tx_input = TxInputWire {
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

pub fn canonical_proof_stmt() -> SpendProofStmt {
    let (tx, prev_root) = canonical_public_contract_tx();
    let spend = tx.proof.spend.clone().expect("spend proof");
    let input_refs = tx
        .inputs
        .iter()
        .map(|input| SpendInputRef {
            asset_id: hex::decode(&input.asset_id_hex)
                .expect("asset id hex")
                .try_into()
                .expect("asset id bytes"),
            serial_id: input.serial_id,
        })
        .collect::<Vec<_>>();
    let input_leaves = spend
        .inputs
        .iter()
        .map(|input| SpendInputLeaf {
            asset_id: hex::decode(&input.input_asset_id_hex)
                .expect("proof input asset id hex")
                .try_into()
                .expect("proof input asset id bytes"),
            serial_id: input.serial_id,
            leaf_ad_id: hex::decode(&input.leaf_ad_id_hex)
                .expect("leaf ad id hex")
                .try_into()
                .expect("leaf ad id bytes"),
            r_pub: hex::decode(&input.r_pub_hex)
                .expect("r_pub hex")
                .try_into()
                .expect("r_pub bytes"),
            owner_tag: hex::decode(&input.owner_tag_hex)
                .expect("owner tag hex")
                .try_into()
                .expect("owner tag bytes"),
            c_amt: hex::decode(&input.commitment_hex)
                .expect("commitment hex")
                .try_into()
                .expect("commitment bytes"),
        })
        .collect::<Vec<_>>();
    let output_leaves = tx
        .outputs
        .iter()
        .map(|output| {
            let mut leaf =
                asset_wire_to_leaf(&output.asset_wire.clone().to_wire().expect("output wire"))
                    .expect("output leaf");
            leaf.asset_id = output.asset_wire.leaf_ad_id.expect("output leaf_ad_id");
            leaf
        })
        .collect::<Vec<_>>();
    let nullifiers = spend
        .inputs
        .iter()
        .map(|input| {
            hex::decode(&input.nullifier_hex)
                .expect("nullifier hex")
                .try_into()
                .expect("nullifier bytes")
        })
        .collect::<Vec<[u8; 32]>>();
    let statement_tx = TxWire {
        proof: TxProofWire {
            spend: Some(SpendProofWire {
                statement_hex: String::new(),
                proof_hex: String::new(),
                ..spend.clone()
            }),
        },
        auth: Default::default(),
        ..tx.clone()
    };
    let pkg_digest = hex::decode(
        build_tx_package_digest(
            "TxPackage",
            "regular_tx",
            TX_VERSION,
            CHAIN_ID,
            CHAIN_TYPE,
            CHAIN_NAME,
            &statement_tx,
        )
        .expect("package digest"),
    )
    .expect("package digest hex")
    .try_into()
    .expect("package digest bytes");

    SpendProofStmt::from_parts(
        hex::decode(&spend.statement_hex).expect("statement hex"),
        pkg_digest,
        prev_root,
        CHAIN_ID,
        TX_VERSION,
        CHAIN_TYPE.to_string(),
        CHAIN_NAME.to_string(),
        input_refs,
        input_leaves,
        output_leaves,
        nullifiers,
    )
    .expect("typed statement")
}

pub fn wrong_prefix_artifact() -> SpendProofArtifact {
    let stmt = canonical_proof_stmt();
    let statement_hash = *blake3::hash(&stmt.statement).as_bytes();
    let mut pub_hash_bytes = Vec::with_capacity(b"z00z.spend.public.hash.v1".len() + 32);
    pub_hash_bytes.extend_from_slice(b"z00z.spend.public.hash.v1");
    pub_hash_bytes.extend_from_slice(&statement_hash);
    let public_hash = *blake3::hash(&pub_hash_bytes).as_bytes();

    let mut proof_bytes = Vec::new();
    proof_bytes.extend_from_slice(b"z00z.spend.proof.rejected");
    proof_bytes.extend_from_slice(&statement_hash);
    proof_bytes.extend_from_slice(&public_hash);

    SpendProofArtifact {
        proof_hex: hex::encode(proof_bytes),
        pub_hash_hex: hex::encode(public_hash),
    }
}

pub fn noncanonical_suite_artifact() -> SpendProofArtifact {
    let stmt = canonical_proof_stmt();
    let statement_hash = *blake3::hash(&stmt.statement).as_bytes();
    let mut pub_hash_bytes = Vec::with_capacity(b"z00z.spend.public.hash.v1".len() + 32);
    pub_hash_bytes.extend_from_slice(b"z00z.spend.public.hash.v1");
    pub_hash_bytes.extend_from_slice(&statement_hash);
    let public_hash = *blake3::hash(&pub_hash_bytes).as_bytes();

    let suite_id = b"regular_spend_theorem_bpplus_alias";
    let theorem_bytes = [0xA5u8];
    let mut proof_bytes = Vec::new();
    proof_bytes.extend_from_slice(b"z00z.spend.proof.backend.v2");
    proof_bytes.push(suite_id.len() as u8);
    proof_bytes.extend_from_slice(suite_id);
    proof_bytes.extend_from_slice(&statement_hash);
    proof_bytes.extend_from_slice(&public_hash);
    proof_bytes.extend_from_slice(&(theorem_bytes.len() as u32).to_le_bytes());
    proof_bytes.extend_from_slice(&theorem_bytes);

    SpendProofArtifact {
        proof_hex: hex::encode(proof_bytes),
        pub_hash_hex: hex::encode(public_hash),
    }
}
