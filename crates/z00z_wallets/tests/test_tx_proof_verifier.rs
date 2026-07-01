#[path = "test_inc/test_spend_proof_support.inc"]
mod test_spend_proof_support;

use z00z_wallets::tx::{
    default_spend_proof_backend, verify_tx_public_spend_contract, SpendProofBackend, SpendPublicErr,
};

#[test]
fn test_verifier_invalid_prev_root() {
    let (mut tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    tx.proof.spend.as_mut().expect("spend proof").prev_root_hex = "zz".to_string();

    let err = verify_tx_public_spend_contract(
        test_spend_proof_support::CHAIN_ID,
        test_spend_proof_support::TX_VERSION,
        test_spend_proof_support::CHAIN_TYPE,
        test_spend_proof_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("malformed prev_root_hex must reject the canonical verifier");

    assert_eq!(
        err,
        SpendPublicErr::InvalidHex {
            label: "proof.prev_root_hex",
        }
    );
}

#[test]
fn test_public_prev_root_hex() {
    let (mut tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    tx.proof.spend.as_mut().expect("spend proof").prev_root_hex =
        hex::encode([0xAB; 32]).to_uppercase();

    let err = verify_tx_public_spend_contract(
        test_spend_proof_support::CHAIN_ID,
        test_spend_proof_support::TX_VERSION,
        test_spend_proof_support::CHAIN_TYPE,
        test_spend_proof_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("uppercase prev_root_hex must reject the canonical verifier");

    assert_eq!(
        err,
        SpendPublicErr::InvalidHex {
            label: "proof.prev_root_hex",
        }
    );
}

#[test]
fn test_public_input_nullifier_hex() {
    let (mut tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    tx.proof.spend.as_mut().expect("spend proof").inputs[0].nullifier_hex =
        tx.proof.spend.as_ref().expect("spend proof").inputs[0]
            .nullifier_hex
            .to_uppercase();

    let err = verify_tx_public_spend_contract(
        test_spend_proof_support::CHAIN_ID,
        test_spend_proof_support::TX_VERSION,
        test_spend_proof_support::CHAIN_TYPE,
        test_spend_proof_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("uppercase input nullifier_hex must reject the canonical verifier");

    assert_eq!(
        err,
        SpendPublicErr::InvalidHex {
            label: "proof.inputs[].nullifier_hex",
        }
    );
}

#[test]
fn test_verifier_input_count_mismatch() {
    let (mut tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    tx.proof.spend.as_mut().expect("spend proof").inputs.clear();

    let err = verify_tx_public_spend_contract(
        test_spend_proof_support::CHAIN_ID,
        test_spend_proof_support::TX_VERSION,
        test_spend_proof_support::CHAIN_TYPE,
        test_spend_proof_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("proof input count must match tx input count");

    assert_eq!(err, SpendPublicErr::InputCountMismatch);
}

#[test]
fn test_public_malformed_proof_blob() {
    let (mut tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    tx.proof.spend.as_mut().expect("spend proof").proof_hex = hex::encode(b"bad-proof-blob");

    let err = verify_tx_public_spend_contract(
        test_spend_proof_support::CHAIN_ID,
        test_spend_proof_support::TX_VERSION,
        test_spend_proof_support::CHAIN_TYPE,
        test_spend_proof_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("malformed proof blob must reject the canonical verifier");

    assert_eq!(err, SpendPublicErr::BadProofBlob);
}

#[test]
fn test_verifier_proof_blob_drift() {
    let (mut tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    let mut proof_blob = hex::decode(&tx.proof.spend.as_ref().expect("spend proof").proof_hex)
        .expect("decode proof blob");
    let suite_len = proof_blob[b"z00z.spend.proof.backend.v2".len()] as usize;
    let statement_hash_offset = b"z00z.spend.proof.backend.v2".len() + 1 + suite_len;
    let statement_hash_byte = proof_blob
        .get_mut(statement_hash_offset)
        .expect("canonical statement hash byte");
    *statement_hash_byte ^= 0x01;
    tx.proof.spend.as_mut().expect("spend proof").proof_hex = hex::encode(proof_blob);

    let err = verify_tx_public_spend_contract(
        test_spend_proof_support::CHAIN_ID,
        test_spend_proof_support::TX_VERSION,
        test_spend_proof_support::CHAIN_TYPE,
        test_spend_proof_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("proof blob with a mismatched statement digest must reject");

    assert_eq!(err, SpendPublicErr::ProofBlobStatementMismatch);
}

#[test]
fn test_verifier_tampered_theorem_payload() {
    let (mut tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    let mut proof_blob = hex::decode(&tx.proof.spend.as_ref().expect("spend proof").proof_hex)
        .expect("decode proof blob");
    let theorem_byte = proof_blob.last_mut().expect("theorem payload byte");
    *theorem_byte ^= 0x01;
    tx.proof.spend.as_mut().expect("spend proof").proof_hex = hex::encode(proof_blob);

    let err = verify_tx_public_spend_contract(
        test_spend_proof_support::CHAIN_ID,
        test_spend_proof_support::TX_VERSION,
        test_spend_proof_support::CHAIN_TYPE,
        test_spend_proof_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("tampered theorem payload must reject");

    assert_eq!(err, SpendPublicErr::TheoremProofFailed);
}

#[test]
fn test_verifier_accepts_canonical_artifact() {
    let (mut tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    let backend = default_spend_proof_backend();
    let artifact = backend
        .prove(
            &test_spend_proof_support::canonical_proof_stmt(),
            &test_spend_proof_support::canonical_proof_witness(),
        )
        .expect("canonical artifact");
    let spend = tx.proof.spend.as_mut().expect("spend proof");
    spend.proof_hex = artifact.proof_hex;
    spend.proof_suite = backend.suite_id().to_string();

    verify_tx_public_spend_contract(
        test_spend_proof_support::CHAIN_ID,
        test_spend_proof_support::TX_VERSION,
        test_spend_proof_support::CHAIN_TYPE,
        test_spend_proof_support::CHAIN_NAME,
        &tx,
    )
    .expect("canonical artifact must verify on the live verifier seam");
}

#[test]
fn test_verifier_wrong_prefix_artifact() {
    let (mut tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    let artifact = test_spend_proof_support::wrong_prefix_artifact();
    tx.proof.spend.as_mut().expect("spend proof").proof_hex = artifact.proof_hex;

    let err = verify_tx_public_spend_contract(
        test_spend_proof_support::CHAIN_ID,
        test_spend_proof_support::TX_VERSION,
        test_spend_proof_support::CHAIN_TYPE,
        test_spend_proof_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("wrong prefix artifact must reject");

    assert_eq!(err, SpendPublicErr::BadProofBlob);
}

#[test]
fn test_verifier_noncanonical_suite_artifact() {
    let (mut tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    let artifact = test_spend_proof_support::noncanonical_suite_artifact();
    tx.proof.spend.as_mut().expect("spend proof").proof_hex = artifact.proof_hex;

    let err = verify_tx_public_spend_contract(
        test_spend_proof_support::CHAIN_ID,
        test_spend_proof_support::TX_VERSION,
        test_spend_proof_support::CHAIN_TYPE,
        test_spend_proof_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("noncanonical suite artifact must reject");

    assert_eq!(err, SpendPublicErr::BadProofSuite);
}

#[test]
fn test_public_leaf_ad_id() {
    let (mut tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    tx.outputs[0].asset_wire.leaf_ad_id = None;

    let err = verify_tx_public_spend_contract(
        test_spend_proof_support::CHAIN_ID,
        test_spend_proof_support::TX_VERSION,
        test_spend_proof_support::CHAIN_TYPE,
        test_spend_proof_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("missing output leaf_ad_id must reject");

    assert_eq!(
        err,
        SpendPublicErr::MissingOutputField {
            idx: 0,
            field: "leaf_ad_id",
        }
    );
}

#[test]
fn test_verifier_missing_r_pub() {
    let (mut tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    tx.outputs[0].asset_wire.r_pub = None;

    let err = verify_tx_public_spend_contract(
        test_spend_proof_support::CHAIN_ID,
        test_spend_proof_support::TX_VERSION,
        test_spend_proof_support::CHAIN_TYPE,
        test_spend_proof_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("missing output r_pub must reject");

    assert_eq!(
        err,
        SpendPublicErr::MissingOutputField {
            idx: 0,
            field: "r_pub",
        }
    );
}

#[test]
fn test_verifier_missing_owner_tag() {
    let (mut tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    tx.outputs[0].asset_wire.owner_tag = None;

    let err = verify_tx_public_spend_contract(
        test_spend_proof_support::CHAIN_ID,
        test_spend_proof_support::TX_VERSION,
        test_spend_proof_support::CHAIN_TYPE,
        test_spend_proof_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("missing output owner_tag must reject");

    assert_eq!(
        err,
        SpendPublicErr::MissingOutputField {
            idx: 0,
            field: "owner_tag",
        }
    );
}

#[test]
fn test_verifier_output_drift_statement() {
    let (mut tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    tx.outputs[0].asset_wire.leaf_ad_id = Some([0x55; 32]);

    let err = verify_tx_public_spend_contract(
        test_spend_proof_support::CHAIN_ID,
        test_spend_proof_support::TX_VERSION,
        test_spend_proof_support::CHAIN_TYPE,
        test_spend_proof_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("output leaf_ad drift must reject on the live verifier seam");

    assert_eq!(err, SpendPublicErr::StatementMismatch);
}

#[test]
fn test_verifier_missing_range_proof() {
    let (mut tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    tx.outputs[0].asset_wire.range_proof = None;

    let err = verify_tx_public_spend_contract(
        test_spend_proof_support::CHAIN_ID,
        test_spend_proof_support::TX_VERSION,
        test_spend_proof_support::CHAIN_TYPE,
        test_spend_proof_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("coin outputs must carry a range proof");

    assert_eq!(err, SpendPublicErr::MissingRangeProof { idx: 0 });
}

#[test]
fn test_verifier_output_range_proof() {
    let (mut tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    let proof = tx.outputs[0]
        .asset_wire
        .range_proof
        .as_mut()
        .expect("canonical range proof");
    proof[0] ^= 0x01;

    let err = verify_tx_public_spend_contract(
        test_spend_proof_support::CHAIN_ID,
        test_spend_proof_support::TX_VERSION,
        test_spend_proof_support::CHAIN_TYPE,
        test_spend_proof_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("tampered range proof must reject");

    assert!(matches!(err, SpendPublicErr::BadRangeProof { idx: 0, .. }));
}

#[test]
fn test_public_output_leaf_overlap() {
    let (mut tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    let input_leaf_ad =
        hex::decode(&tx.proof.spend.as_ref().expect("spend proof").inputs[0].leaf_ad_id_hex)
            .expect("decode leaf_ad_id");
    tx.outputs[0].asset_wire.leaf_ad_id = Some(input_leaf_ad.try_into().expect("leaf_ad_id"));

    let err = verify_tx_public_spend_contract(
        test_spend_proof_support::CHAIN_ID,
        test_spend_proof_support::TX_VERSION,
        test_spend_proof_support::CHAIN_TYPE,
        test_spend_proof_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("input/output leaf_ad overlap must reject");

    assert_eq!(err, SpendPublicErr::InputOutputLeafOverlap);
}

#[test]
fn test_verifier_bad_balance() {
    let (mut tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    let mut extra_output = tx.outputs[0].clone();
    extra_output.asset_wire.leaf_ad_id = Some([0x88; 32]);
    tx.outputs.push(extra_output);

    let err = verify_tx_public_spend_contract(
        test_spend_proof_support::CHAIN_ID,
        test_spend_proof_support::TX_VERSION,
        test_spend_proof_support::CHAIN_TYPE,
        test_spend_proof_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("imbalanced output commitments must reject");

    assert_eq!(err, SpendPublicErr::BadBalance);
}

#[test]
fn test_verifier_input_binding_mismatch() {
    let (mut tx, _) = test_spend_proof_support::canonical_public_contract_tx();
    tx.proof.spend.as_mut().expect("spend proof").inputs[0].input_asset_id_hex =
        hex::encode([0xBB; 32]);

    let err = verify_tx_public_spend_contract(
        test_spend_proof_support::CHAIN_ID,
        test_spend_proof_support::TX_VERSION,
        test_spend_proof_support::CHAIN_TYPE,
        test_spend_proof_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("mismatched tx input binding must reject");

    assert_eq!(err, SpendPublicErr::InputRefMismatch { idx: 0 });
}
