// THREAT-ANCHOR T-03: spend-statement binding — public inputs must match tx facts

#[path = "test_inc/test_spend_public_support.inc"]
mod test_spend_public_support;

use z00z_crypto::{domains::TxDigestDomain, frame_bytes, hash_zk::hash_zk};
use z00z_utils::codec::Codec;
use z00z_wallets::tx::{
    build_public_spend_contract, build_tx_package_digest, verify_tx_public_spend_contract,
    SpendPublicErr, TxAuthWire, TxOutRole, TxProofWire, TxWire,
};

fn compute_wire_digest(tx_wire: &TxWire) -> [u8; 32] {
    let wire_bytes = z00z_utils::codec::JsonCodec
        .serialize(tx_wire)
        .expect("serialize tx wire for digest");
    let framed = frame_bytes(&wire_bytes);
    hash_zk::<TxDigestDomain>("Z00Z/TXPKG_WIRE", &[framed.as_slice()])
}

#[test]
fn test_statement_deterministic_tx_facts() {
    let (tx, prev_root) = test_spend_public_support::canonical_public_contract_tx();
    let proof_inputs = tx.proof.spend.as_ref().expect("spend proof").inputs.clone();
    let bare_tx = TxWire {
        proof: TxProofWire::default(),
        auth: TxAuthWire::default(),
        ..tx.clone()
    };
    let (rebuilt_proof, rebuilt_auth) = build_public_spend_contract(
        &test_spend_public_support::receiver_keys(),
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &bare_tx,
        prev_root,
        proof_inputs,
        test_spend_public_support::canonical_proof_witness(),
    )
    .expect("rebuild canonical spend contract");

    verify_tx_public_spend_contract(
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &tx,
    )
    .expect("canonical tx facts must verify");
    verify_tx_public_spend_contract(
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &TxWire {
            proof: rebuilt_proof.clone(),
            auth: rebuilt_auth.clone(),
            ..bare_tx.clone()
        },
    )
    .expect("identical tx facts must stay verifiable");

    assert_eq!(
        tx.proof.spend.as_ref().expect("spend proof").statement_hex,
        rebuilt_proof
            .spend
            .as_ref()
            .expect("rebuilt spend proof")
            .statement_hex
    );
}

#[test]
fn test_statement_fee_drift_auth() {
    let (mut tx, _) = test_spend_public_support::canonical_public_contract_tx();
    tx.fee += 1;

    let err = verify_tx_public_spend_contract(
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("fee drift after signing must invalidate the statement binding");

    assert_eq!(err, SpendPublicErr::StatementMismatch);
}

#[test]
fn test_statement_rejects_prev_root() {
    let (mut tx, _) = test_spend_public_support::canonical_public_contract_tx();
    tx.proof.spend.as_mut().expect("spend proof").prev_root_hex = hex::encode([8u8; 32]);

    let err = verify_tx_public_spend_contract(
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("prev_root drift after signing must invalidate the statement binding");

    assert_eq!(err, SpendPublicErr::StatementMismatch);
}

#[test]
fn test_statement_output_field_drift() {
    let (mut tx, _) = test_spend_public_support::canonical_public_contract_tx();
    tx.outputs[0].role = TxOutRole::Change;

    let err = verify_tx_public_spend_contract(
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("output public field drift must invalidate the signed spend statement");

    assert_eq!(err, SpendPublicErr::StatementMismatch);
}

#[test]
fn test_statement_rejects_chain_scope() {
    let (tx, _) = test_spend_public_support::canonical_public_contract_tx();

    let err = verify_tx_public_spend_contract(
        test_spend_public_support::CHAIN_ID + 1,
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("chain scope drift must invalidate the signed spend statement");

    assert_eq!(err, SpendPublicErr::StatementMismatch);
}

#[test]
fn test_statement_tx_version_drift() {
    let (tx, _) = test_spend_public_support::canonical_public_contract_tx();

    let err = verify_tx_public_spend_contract(
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::TX_VERSION + 1,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("tx version drift must invalidate the signed spend statement");

    assert_eq!(err, SpendPublicErr::StatementMismatch);
}

#[test]
fn test_statement_rejects_ad_hoc() {
    let (mut tx, _) = test_spend_public_support::canonical_public_contract_tx();
    tx.proof.spend.as_mut().expect("spend proof").statement_hex =
        hex::encode(br#"{"statement":"ad hoc"}"#);

    let err = verify_tx_public_spend_contract(
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("ad hoc statement fragments must reject the public spend contract verifier");

    assert_eq!(err, SpendPublicErr::StatementMismatch);
}

#[test]
fn test_statement_unversioned_proof_transcript() {
    let (mut tx, _) = test_spend_public_support::canonical_public_contract_tx();
    tx.proof.spend.as_mut().expect("spend proof").proof_suite = "legacy_blob".to_string();

    let err = verify_tx_public_spend_contract(
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("unversioned proof transcripts must reject the public spend contract verifier");

    assert_eq!(err, SpendPublicErr::BadProofSuite);
}

#[test]
fn test_statement_rejects_wire_digest() {
    let (mut tx, _) = test_spend_public_support::canonical_public_contract_tx();
    let wire_digest = compute_wire_digest(&tx);
    tx.proof.spend.as_mut().expect("spend proof").statement_hex = hex::encode(wire_digest);

    let err = verify_tx_public_spend_contract(
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("bare wire digest must not be accepted as the only public root");

    assert_eq!(err, SpendPublicErr::StatementMismatch);
}

// Threat T-3 anchor: digest is routing identity; auth drift must not change it.
#[test]
fn test_package_spend_auth_fields() {
    let (mut tx, _) = test_spend_public_support::canonical_public_contract_tx();

    let digest_before = build_tx_package_digest(
        "TxPackage",
        "regular_tx",
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &tx,
    )
    .expect("digest before auth drift");

    let spend = tx.proof.spend.as_mut().expect("spend proof");
    spend.statement_hex = hex::encode([0xAB; 32]);
    spend.proof_hex = hex::encode([0xCD; 32]);

    let auth = tx.auth.spend.as_mut().expect("spend auth");
    auth.receiver_card_compact = "tampered-card".to_string();
    auth.spend_sig_hex = "00".repeat(64);

    let digest_after = build_tx_package_digest(
        "TxPackage",
        "regular_tx",
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &tx,
    )
    .expect("digest after auth drift");

    assert_eq!(digest_before, digest_after);
}

#[test]
fn test_package_digest_input_hex() {
    let (mut tx, _) = test_spend_public_support::canonical_public_contract_tx();
    tx.proof.spend.as_mut().expect("spend proof").inputs[0].nullifier_hex =
        tx.proof.spend.as_ref().expect("spend proof").inputs[0]
            .nullifier_hex
            .to_uppercase();

    let err = build_tx_package_digest(
        "TxPackage",
        "regular_tx",
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("digest helper must reject noncanonical spend-carrier hex");

    assert!(
        err.contains("proof.inputs[].nullifier_hex"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_package_digest_id_hex() {
    let (mut tx, _) = test_spend_public_support::canonical_public_contract_tx();
    tx.inputs[0].asset_id_hex = hex::encode([0xAB; 32]).to_uppercase();

    let err = build_tx_package_digest(
        "TxPackage",
        "regular_tx",
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("digest helper must reject noncanonical input state keys");

    assert!(
        err.contains("tx input asset_id_hex must be 32-byte lowercase hex"),
        "unexpected error: {err}"
    );
}

// Threat T-3 anchor: digest is routing identity; auth drift must not change it.
#[test]
fn test_digest_ignores_auth_hex() {
    let (mut tx, _) = test_spend_public_support::canonical_public_contract_tx();

    let digest_before = build_tx_package_digest(
        "TxPackage",
        "regular_tx",
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &tx,
    )
    .expect("digest before auth canonicality drift");

    tx.auth.spend.as_mut().expect("spend auth").spend_sig_hex = "AB".repeat(64);

    let digest_after = build_tx_package_digest(
        "TxPackage",
        "regular_tx",
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &tx,
    )
    .expect("digest after auth canonicality drift");

    assert_eq!(digest_before, digest_after);
}

// Threat T-3 anchor: digest is routing identity; auth drift must not change it.
#[test]
fn test_package_excluded_spend_blobs() {
    let (mut tx, _) = test_spend_public_support::canonical_public_contract_tx();

    let digest_before = build_tx_package_digest(
        "TxPackage",
        "regular_tx",
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &tx,
    )
    .expect("digest before malformed blob drift");

    let spend = tx.proof.spend.as_mut().expect("spend proof");
    spend.statement_hex = "zz".to_string();
    spend.proof_hex = "not-hex".to_string();

    let digest_after = build_tx_package_digest(
        "TxPackage",
        "regular_tx",
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &tx,
    )
    .expect("digest after malformed blob drift");

    assert_eq!(digest_before, digest_after);
}

#[test]
fn test_statement_rejects_asset_hex() {
    let (mut tx, _) = test_spend_public_support::canonical_public_contract_tx();
    tx.inputs[0].asset_id_hex = hex::encode([0xAB; 32]).to_uppercase();

    let err = verify_tx_public_spend_contract(
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("noncanonical input state keys must reject the signed spend statement");

    assert_eq!(
        err,
        SpendPublicErr::InvalidHex {
            label: "tx.inputs[].asset_id_hex",
        }
    );
}
