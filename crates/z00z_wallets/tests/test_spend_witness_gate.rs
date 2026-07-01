#[path = "test_inc/test_range_proof_env.inc"]
mod test_common;

use test_common::RangeProofEnvGuard;
use z00z_core::{
    assets::{AssetPackPlain, AssetPkgWire},
    genesis::asset_std::asset_from_dev_class,
    AssetClass, AssetWire,
};
use z00z_crypto::{compute_leaf_ad, compute_tag16, create_range_proof, Z00ZScalar};
use z00z_storage::settlement::CheckRoot;
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::{ScanResult, StealthOutputScanner},
    stealth::ecdh::{compute_dh_receiver, decode_r_pub},
    stealth::kdf::{compute_owner_tag, derive_k_dh, derive_s_out},
    stealth::zkpack::ZkPack,
    stealth::{
        bind_stealth_output_wire, build_card_stealth_leaf, build_tx_output_unchecked, SenderWallet,
    },
    tx::{
        asset_wire_to_leaf, build_public_spend_contract, prepare_spend_membership_witnesses,
        prepare_spend_public_inputs, resolve_input_pack, verify_spend_witness_gate,
        verify_tx_public_spend_contract, wire_decrypt_leaf, OutputBundle, SpendMembershipWitness,
        SpendProofWitness, SpendPublicErr, TxInputWire, TxOutRole, TxOutputWire, TxWire,
    },
};

const CHAIN_TYPE: &str = "witness_gate";
const CHAIN_NAME: &str = "witness_gate";

fn recv_sec() -> [u8; 32] {
    [0x11u8; 32]
}

fn make_wire_and_output() -> (AssetWire, OutputBundle) {
    let _guard = RangeProofEnvGuard::new();
    let keys = ReceiverKeys::from_receiver_secret(
        ReceiverSecret::from_bytes(recv_sec()).expect("receiver secret"),
    )
    .expect("receiver keys");
    let asset = asset_from_dev_class(AssetClass::Coin, 7, 55).expect("asset");
    let card = keys.export_receiver_card().expect("card");
    let leaf = build_card_stealth_leaf(&card, asset.amount, asset.serial_id).expect("leaf");
    let wire = bind_stealth_output_wire(AssetWire::from_asset(&asset), &leaf).expect("bind wire");

    let output = OutputBundle {
        receiver: "bob".to_string(),
        role: TxOutRole::Recipient,
        class: AssetClass::Coin,
        value: asset.amount,
        leaf: asset_wire_to_leaf(&wire).expect("output leaf"),
        k_dh: [7u8; 32],
        s_out: [8u8; 32],
    };

    (wire, output)
}

fn tx_inputs_for_wires(inputs: &[AssetWire]) -> Vec<TxInputWire> {
    inputs
        .iter()
        .map(|wire| TxInputWire {
            asset_id_hex: hex::encode(asset_wire_to_leaf(wire).expect("input leaf").asset_id),
            serial_id: wire.serial_id,
        })
        .collect()
}

fn membership_for_wires(inputs: &[AssetWire]) -> (CheckRoot, Vec<SpendMembershipWitness>) {
    let tx_inputs = tx_inputs_for_wires(inputs);
    prepare_spend_membership_witnesses(inputs, &tx_inputs).expect("membership witnesses")
}

fn membership_root_for_wire(input: &AssetWire) -> CheckRoot {
    membership_for_wires(std::slice::from_ref(input)).0
}

fn make_public_contract_tx() -> (TxWire, CheckRoot) {
    let _guard = RangeProofEnvGuard::new();
    let keys = ReceiverKeys::from_receiver_secret(
        ReceiverSecret::from_bytes(recv_sec()).expect("receiver secret"),
    )
    .expect("receiver keys");
    let card = keys.export_receiver_card().expect("card");

    let input_asset = asset_from_dev_class(AssetClass::Coin, 7, 55).expect("input asset");
    let input_leaf = build_card_stealth_leaf(&card, input_asset.amount, input_asset.serial_id)
        .expect("input leaf");
    let input_wire = bind_stealth_output_wire(AssetWire::from_asset(&input_asset), &input_leaf)
        .expect("input wire");
    let mut output_wire = input_wire.clone();
    output_wire.leaf_ad_id = Some([0x77; 32]);

    let tx_input = tx_inputs_for_wires(std::slice::from_ref(&input_wire))
        .pop()
        .expect("tx input");
    let tx_output = TxOutputWire {
        role: TxOutRole::Recipient,
        asset_wire: AssetPkgWire::from_wire(&output_wire),
    };
    let proof_inputs = prepare_spend_public_inputs(
        3,
        recv_sec(),
        std::slice::from_ref(&input_wire),
        std::slice::from_ref(&tx_input),
    )
    .expect("proof inputs");
    let (prev_root, membership) = membership_for_wires(std::slice::from_ref(&input_wire));

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
        3,
        1,
        CHAIN_TYPE,
        CHAIN_NAME,
        &tx,
        prev_root,
        proof_inputs,
        SpendProofWitness {
            receiver_secret: ReceiverSecret::from_bytes(recv_sec()).expect("receiver secret"),
            input_s_in: vec![
                resolve_input_pack(recv_sec(), &input_wire)
                    .expect("input pack")
                    .s_out,
            ],
            membership,
        },
    )
    .expect("public spend contract");
    tx.proof = proof;
    tx.auth = auth;

    (tx, prev_root)
}

fn make_claim_owned_input_wire(serial_id: u32, amount: u64, tx_digest: [u8; 32]) -> AssetWire {
    let _guard = RangeProofEnvGuard::new();
    let keys = ReceiverKeys::from_receiver_secret(
        ReceiverSecret::from_bytes(recv_sec()).expect("receiver secret"),
    )
    .expect("receiver keys");
    let card = keys.export_receiver_card().expect("card");
    let mut sender = SenderWallet::new([serial_id as u8 + 0x40; 32]);
    let mut owned = asset_from_dev_class(AssetClass::Coin, serial_id, amount).expect("asset");

    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut sender,
        &tx_digest,
        0,
        amount,
        &owned.definition.id,
    )
    .expect("claim-owned style output");

    let r_pub = output.r_pub;
    let c_amount = output.c_amount;
    let r_point = decode_r_pub(&r_pub).expect("decode r_pub");
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &r_point).expect("dh");
    let k_dh = derive_k_dh(&dh);
    let owner_tag = compute_owner_tag(&card.owner_handle, &k_dh);

    let base_ad = compute_leaf_ad(&owned.definition.id, 0, &r_pub, &owner_tag, &c_amount);
    let base_plain = ZkPack::decrypt(
        &k_dh,
        &base_ad,
        &r_pub,
        &owned.definition.id,
        0,
        &output.enc_pack,
    )
    .expect("baseline stealth decrypt");
    let base_pack = AssetPackPlain::from_bytes(&base_plain).expect("baseline stealth pack");

    let s_out = derive_s_out(&k_dh, &r_pub, owned.serial_id);
    let payload = AssetPackPlain {
        value: owned.amount,
        blinding: base_pack.blinding,
        s_out,
    }
    .to_bytes();

    let leaf_ad = compute_leaf_ad(
        &owned.definition.id,
        owned.serial_id,
        &r_pub,
        &owner_tag,
        &c_amount,
    );
    let enc_pack = ZkPack::encrypt(
        &k_dh,
        &leaf_ad,
        &r_pub,
        &owned.definition.id,
        owned.serial_id,
        &payload,
    );
    let tag16 = Some(compute_tag16(&k_dh, &leaf_ad));

    owned.commitment = z00z_crypto::Commitment::from_bytes(&c_amount)
        .expect("commitment")
        .0;
    owned.owner_pub = None;
    owned.owner_signature = None;
    owned.r_pub = Some(r_pub);
    owned.owner_tag = Some(owner_tag);
    owned.enc_pack = Some(enc_pack);
    owned.tag16 = tag16;
    owned.leaf_ad_id = Some(owned.definition.id);

    let scanner = StealthOutputScanner::from_keys(&keys);
    let ScanResult::Mine { wallet_output } = scanner.scan_leaf(&owned) else {
        panic!("generated claim-owned wire is not mine");
    };
    let blinding = wallet_output
        .blinding
        .as_ref()
        .copied()
        .and_then(|bytes| Z00ZScalar::try_from_bytes(bytes).ok())
        .expect("blinding scalar");
    owned.range_proof =
        Some(create_range_proof(owned.amount, &blinding, 64, 0).expect("range proof"));

    AssetWire::from_asset(&owned)
}

#[test]
fn test_witness_accepts_pack() {
    let (wire, output) = make_wire_and_output();

    verify_spend_witness_gate(
        3,
        recv_sec(),
        std::slice::from_ref(&wire),
        std::slice::from_ref(&output),
        membership_root_for_wire(&wire),
    )
    .expect("canonical spend witness input must pass the witness gate");
}

#[test]
fn test_witness_rejects_leaf_ad() {
    let (mut wire, output) = make_wire_and_output();
    wire.leaf_ad_id = Some([0xAA; 32]);

    let err = verify_spend_witness_gate(
        3,
        recv_sec(),
        std::slice::from_ref(&wire),
        std::slice::from_ref(&output),
        membership_root_for_wire(&wire),
    )
    .expect_err("tampered leaf_ad_id must reject at the witness gate");

    assert!(
        err.contains("input decrypt failed") || err.contains("input is not decryptable"),
        "unexpected leaf_ad_id tamper error: {err}"
    );
}

#[test]
fn test_wire_uses_leaf_ad() {
    let (wire, _) = make_wire_and_output();

    let state_leaf = asset_wire_to_leaf(&wire).expect("state leaf");
    let decrypt_leaf = wire_decrypt_leaf(&wire).expect("decrypt leaf");

    assert_eq!(decrypt_leaf.asset_id, wire.leaf_ad_id.expect("leaf_ad_id"));
    assert_ne!(
        decrypt_leaf.asset_id, state_leaf.asset_id,
        "decrypt namespace must stay distinct from the canonical state key"
    );
}

#[test]
fn test_witness_missing_owner_tag() {
    let (mut wire, output) = make_wire_and_output();
    let prev_root = membership_root_for_wire(&wire);
    wire.owner_tag = None;

    let err = verify_spend_witness_gate(
        3,
        recv_sec(),
        std::slice::from_ref(&wire),
        std::slice::from_ref(&output),
        prev_root,
    )
    .expect_err("missing owner_tag must reject at the witness gate");

    assert!(
        err.contains("missing input owner_tag"),
        "unexpected missing owner_tag error: {err}"
    );
}

#[test]
fn test_wrong_secret_not_echoed() {
    let (mut wire, output) = make_wire_and_output();
    let secret = [0xAB; 32];
    let secret_hex = hex::encode(secret);
    wire.secret = Some(secret);

    let err = verify_spend_witness_gate(
        3,
        recv_sec(),
        std::slice::from_ref(&wire),
        std::slice::from_ref(&output),
        membership_root_for_wire(&wire),
    )
    .expect_err("wrong input secret must reject at the witness gate");

    assert!(
        err.contains("input pack unavailable for provided secret"),
        "unexpected wrong-secret error: {err}"
    );
    assert!(
        !err.contains(&secret_hex),
        "wrong-secret error leaked the provided secret: {err}"
    );
}

#[test]
fn test_public_accepts_statement() {
    let (tx, _) = make_public_contract_tx();

    verify_tx_public_spend_contract(3, 1, CHAIN_TYPE, CHAIN_NAME, &tx)
        .expect("canonical public spend contract");
    assert_eq!(
        tx.proof.spend.as_ref().expect("spend proof").inputs[0]
            .nullifier_hex
            .len(),
        64
    );
}

#[test]
fn test_public_rejects_placeholder() {
    let (mut tx, _) = make_public_contract_tx();
    tx.auth.spend = None;

    let err = verify_tx_public_spend_contract(3, 1, CHAIN_TYPE, CHAIN_NAME, &tx)
        .expect_err("missing spend auth must reject structural-only placeholder");

    assert_eq!(err, SpendPublicErr::MissingAuth);
}

#[test]
fn test_public_rejects_prev_root() {
    let (mut tx, _) = make_public_contract_tx();
    tx.proof.spend.as_mut().expect("spend proof").prev_root_hex = hex::encode([0x33u8; 32]);

    let err = verify_tx_public_spend_contract(3, 1, CHAIN_TYPE, CHAIN_NAME, &tx)
        .expect_err("replayed prev_root must reject");

    assert_eq!(err, SpendPublicErr::StatementMismatch);
}

#[test]
fn test_witness_honors_chain_id() {
    let (wire, output) = make_wire_and_output();

    verify_spend_witness_gate(
        77,
        recv_sec(),
        std::slice::from_ref(&wire),
        std::slice::from_ref(&output),
        membership_root_for_wire(&wire),
    )
    .expect("witness gate must derive spend nullifiers from the caller chain_id");
}

#[test]
fn test_public_rejects_leaf_ad() {
    let (mut tx, _) = make_public_contract_tx();
    tx.proof.spend.as_mut().expect("spend proof").inputs[0].leaf_ad_hash_hex =
        hex::encode([0x44u8; 32]);

    let err = verify_tx_public_spend_contract(3, 1, CHAIN_TYPE, CHAIN_NAME, &tx)
        .expect_err("tampered leaf_ad hash must reject");

    assert_eq!(err, SpendPublicErr::InputLeafAdHashMismatch { idx: 0 });
}

#[test]
fn test_public_missing_output_ad() {
    let (mut tx, _) = make_public_contract_tx();
    tx.outputs[0].asset_wire.leaf_ad_id = None;

    let err = verify_tx_public_spend_contract(3, 1, CHAIN_TYPE, CHAIN_NAME, &tx)
        .expect_err("missing output leaf_ad_id must reject the canonical public contract");

    assert_eq!(
        err,
        SpendPublicErr::MissingOutputField {
            idx: 0,
            field: "leaf_ad_id",
        }
    );
}

#[test]
fn test_rejects_bad_nullifier_hex() {
    let (mut tx, _) = make_public_contract_tx();
    tx.proof.spend.as_mut().expect("spend proof").inputs[0].nullifier_hex = "zz".to_string();

    let err = verify_tx_public_spend_contract(3, 1, CHAIN_TYPE, CHAIN_NAME, &tx)
        .expect_err("malformed nullifier hex must reject the canonical public contract");

    assert_eq!(
        err,
        SpendPublicErr::InvalidHex {
            label: "proof.inputs[].nullifier_hex"
        }
    );
}

#[test]
fn test_public_rejects_nullifier_value() {
    let (mut tx, _) = make_public_contract_tx();
    tx.proof.spend.as_mut().expect("spend proof").inputs[0]
        .nullifier_hex
        .clear();

    let err = verify_tx_public_spend_contract(3, 1, CHAIN_TYPE, CHAIN_NAME, &tx)
        .expect_err("missing nullifier value must reject the canonical public contract");

    assert_eq!(
        err,
        SpendPublicErr::InvalidHex {
            label: "proof.inputs[].nullifier_hex"
        }
    );
}

#[test]
fn test_rejects_signed_nullifier_drift() {
    let (mut tx, _) = make_public_contract_tx();
    tx.proof.spend.as_mut().expect("spend proof").inputs[0].nullifier_hex = hex::encode([0xAB; 32]);

    let err = verify_tx_public_spend_contract(3, 1, CHAIN_TYPE, CHAIN_NAME, &tx)
        .expect_err("post-signature nullifier drift must reject the canonical public contract");

    assert_eq!(err, SpendPublicErr::StatementMismatch);
}

#[test]
fn test_public_allows_duplicate_ads() {
    let input_left = make_claim_owned_input_wire(7, 55, [0x31; 32]);
    let input_right = make_claim_owned_input_wire(8, 66, [0x32; 32]);

    let left_leaf = asset_wire_to_leaf(&input_left).expect("left leaf");
    let right_leaf = asset_wire_to_leaf(&input_right).expect("right leaf");
    assert_ne!(
        left_leaf.asset_id, right_leaf.asset_id,
        "canonical asset_id must differ"
    );
    assert_eq!(
        input_left.leaf_ad_id, input_right.leaf_ad_id,
        "claim-owned inputs should share the same decrypt AD id for one asset definition"
    );

    let tx_inputs = vec![
        TxInputWire {
            asset_id_hex: hex::encode(left_leaf.asset_id),
            serial_id: input_left.serial_id,
        },
        TxInputWire {
            asset_id_hex: hex::encode(right_leaf.asset_id),
            serial_id: input_right.serial_id,
        },
    ];

    let mut left_output = input_left.clone();
    left_output.leaf_ad_id = Some(left_leaf.asset_id);
    let mut right_output = input_right.clone();
    right_output.leaf_ad_id = Some(right_leaf.asset_id);

    let tx_outputs = vec![
        TxOutputWire {
            role: TxOutRole::Recipient,
            asset_wire: AssetPkgWire::from_wire(&left_output),
        },
        TxOutputWire {
            role: TxOutRole::Recipient,
            asset_wire: AssetPkgWire::from_wire(&right_output),
        },
    ];

    let proof_inputs = prepare_spend_public_inputs(
        3,
        recv_sec(),
        &[input_left.clone(), input_right.clone()],
        &tx_inputs,
    )
    .expect("proof inputs");
    assert_ne!(
        proof_inputs[0].leaf_ad_id_hex, proof_inputs[1].leaf_ad_id_hex,
        "theorem proof AD ids must stay per-input even when claim-owned wire leaf_ad_id repeats"
    );
    let (prev_root, membership) = membership_for_wires(&[input_left.clone(), input_right.clone()]);

    let mut tx = TxWire {
        tx_type: "regular_tx".to_string(),
        inputs: tx_inputs,
        outputs: tx_outputs,
        fee: 0,
        nonce: 0,
        context: Default::default(),
        proof: Default::default(),
        auth: Default::default(),
    };
    let keys = ReceiverKeys::from_receiver_secret(
        ReceiverSecret::from_bytes(recv_sec()).expect("receiver secret"),
    )
    .expect("receiver keys");
    let (proof, auth) = build_public_spend_contract(
        &keys,
        3,
        1,
        CHAIN_TYPE,
        CHAIN_NAME,
        &tx,
        prev_root,
        proof_inputs,
        SpendProofWitness {
            receiver_secret: ReceiverSecret::from_bytes(recv_sec()).expect("receiver secret"),
            input_s_in: vec![
                resolve_input_pack(recv_sec(), &input_left)
                    .expect("left input pack")
                    .s_out,
                resolve_input_pack(recv_sec(), &input_right)
                    .expect("right input pack")
                    .s_out,
            ],
            membership,
        },
    )
    .expect("public spend contract");
    tx.proof = proof;
    tx.auth = auth;

    verify_tx_public_spend_contract(3, 1, CHAIN_TYPE, CHAIN_NAME, &tx)
        .expect("distinct input refs must remain valid even when input leaf_ad_id repeats");
}

#[test]
fn test_public_rejects_duplicate_nullifier() {
    let input_left = make_claim_owned_input_wire(7, 55, [0x31; 32]);
    let input_right = make_claim_owned_input_wire(8, 66, [0x32; 32]);

    let left_leaf = asset_wire_to_leaf(&input_left).expect("left leaf");
    let right_leaf = asset_wire_to_leaf(&input_right).expect("right leaf");

    let tx_inputs = vec![
        TxInputWire {
            asset_id_hex: hex::encode(left_leaf.asset_id),
            serial_id: input_left.serial_id,
        },
        TxInputWire {
            asset_id_hex: hex::encode(right_leaf.asset_id),
            serial_id: input_right.serial_id,
        },
    ];

    let mut left_output = input_left.clone();
    left_output.leaf_ad_id = Some(left_leaf.asset_id);
    let mut right_output = input_right.clone();
    right_output.leaf_ad_id = Some(right_leaf.asset_id);

    let tx_outputs = vec![
        TxOutputWire {
            role: TxOutRole::Recipient,
            asset_wire: AssetPkgWire::from_wire(&left_output),
        },
        TxOutputWire {
            role: TxOutRole::Recipient,
            asset_wire: AssetPkgWire::from_wire(&right_output),
        },
    ];

    let proof_inputs = prepare_spend_public_inputs(
        3,
        recv_sec(),
        &[input_left.clone(), input_right.clone()],
        &tx_inputs,
    )
    .expect("proof inputs");
    let (prev_root, membership) = membership_for_wires(&[input_left.clone(), input_right.clone()]);

    let mut tx = TxWire {
        tx_type: "regular_tx".to_string(),
        inputs: tx_inputs,
        outputs: tx_outputs,
        fee: 0,
        nonce: 0,
        context: Default::default(),
        proof: Default::default(),
        auth: Default::default(),
    };
    let keys = ReceiverKeys::from_receiver_secret(
        ReceiverSecret::from_bytes(recv_sec()).expect("receiver secret"),
    )
    .expect("receiver keys");
    let (proof, auth) = build_public_spend_contract(
        &keys,
        3,
        1,
        CHAIN_TYPE,
        CHAIN_NAME,
        &tx,
        prev_root,
        proof_inputs,
        SpendProofWitness {
            receiver_secret: ReceiverSecret::from_bytes(recv_sec()).expect("receiver secret"),
            input_s_in: vec![
                resolve_input_pack(recv_sec(), &input_left)
                    .expect("left input pack")
                    .s_out,
                resolve_input_pack(recv_sec(), &input_right)
                    .expect("right input pack")
                    .s_out,
            ],
            membership,
        },
    )
    .expect("public spend contract");
    tx.proof = proof;
    tx.auth = auth;
    let duplicate = tx.proof.spend.as_ref().expect("proof").inputs[0]
        .nullifier_hex
        .clone();
    tx.proof.spend.as_mut().expect("proof").inputs[1].nullifier_hex = duplicate;

    let err = verify_tx_public_spend_contract(3, 1, CHAIN_TYPE, CHAIN_NAME, &tx)
        .expect_err("duplicate nullifier must reject one spend contract");

    assert_eq!(err, SpendPublicErr::DuplicateNullifier);
}
