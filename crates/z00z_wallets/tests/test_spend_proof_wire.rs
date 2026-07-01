#[path = "test_inc/test_spend_wire_support.inc"]
mod test_spend_wire_support;

use z00z_utils::codec::Codec;
use z00z_wallets::tx::{
    verify_tx_public_spend_contract, SpendPublicErr, TxAuthWire, TxProofWire, TxWire,
};

#[test]
fn test_wire_roundtrip_regular_carrier() {
    let (tx, _) = test_spend_wire_support::canonical_public_contract_tx();
    let spend_proof = tx.proof.spend.as_ref().expect("spend proof");
    let spend_auth = tx.auth.spend.as_ref().expect("spend auth");

    assert!(
        !spend_proof.inputs.is_empty(),
        "proof carrier must keep inputs"
    );
    assert!(
        !spend_proof.prev_root_hex.is_empty(),
        "proof carrier must bind prev_root"
    );
    assert!(
        !spend_proof.proof_suite.is_empty(),
        "proof carrier must identify proof suite"
    );
    assert!(
        !spend_proof.statement_hex.is_empty(),
        "proof carrier must carry the canonical public statement payload"
    );
    assert!(
        !spend_proof.proof_hex.is_empty(),
        "proof carrier must carry opaque proof bytes"
    );
    assert!(
        !spend_auth.receiver_card_compact.is_empty(),
        "auth carrier must keep receiver card"
    );
    assert!(
        !spend_auth.spend_sig_hex.is_empty(),
        "auth carrier must keep authorization signature"
    );

    let encoded = Codec::serialize(&z00z_utils::codec::JsonCodec, &tx).expect("serialize tx");
    let decoded: TxWire =
        Codec::deserialize(&z00z_utils::codec::JsonCodec, &encoded).expect("decode tx");

    assert_eq!(decoded.proof, tx.proof);
    assert_eq!(decoded.auth, tx.auth);
    verify_tx_public_spend_contract(
        test_spend_wire_support::CHAIN_ID,
        test_spend_wire_support::TX_VERSION,
        test_spend_wire_support::CHAIN_TYPE,
        test_spend_wire_support::CHAIN_NAME,
        &decoded,
    )
    .expect("roundtripped carrier must stay verifiable");
}

#[test]
fn test_legacy_valid_opaque_output() {
    let (mut tx, _) = test_spend_wire_support::canonical_public_contract_tx();
    tx.proof = TxProofWire::default();
    tx.auth = TxAuthWire::default();

    let err = verify_tx_public_spend_contract(
        test_spend_wire_support::CHAIN_ID,
        test_spend_wire_support::TX_VERSION,
        test_spend_wire_support::CHAIN_TYPE,
        test_spend_wire_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("empty carrier must not pass as Phase 040 spend proof output");

    assert_eq!(err, SpendPublicErr::MissingProof);
}

#[test]
fn test_wire_bad_proof_version() {
    let (mut tx, _) = test_spend_wire_support::canonical_public_contract_tx();
    tx.proof.spend.as_mut().expect("spend proof").ver += 1;

    let err = verify_tx_public_spend_contract(
        test_spend_wire_support::CHAIN_ID,
        test_spend_wire_support::TX_VERSION,
        test_spend_wire_support::CHAIN_TYPE,
        test_spend_wire_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("proof version drift must reject");

    assert_eq!(err, SpendPublicErr::BadProofVersion);
}

#[test]
fn test_spend_proof_missing_suite() {
    let (tx, _) = test_spend_wire_support::canonical_public_contract_tx();
    let encoded = Codec::serialize(&z00z_utils::codec::JsonCodec, &tx).expect("serialize tx");
    let json = String::from_utf8(encoded).expect("tx json");
    let without_suite = json.replace(
        &format!(
            "\"proof_suite\":\"{}\",",
            tx.proof.spend.as_ref().expect("spend proof").proof_suite
        ),
        "",
    );

    let err = Codec::deserialize::<TxWire>(&z00z_utils::codec::JsonCodec, without_suite.as_bytes())
        .expect_err("proof carrier must require an explicit proof_suite field");

    let err_text = err.to_string();
    assert!(
        err_text.contains("proof_suite"),
        "decode error should mention the missing proof_suite field, got: {err_text}"
    );
}

#[test]
fn test_wire_missing_statement_field() {
    let (tx, _) = test_spend_wire_support::canonical_public_contract_tx();
    let encoded = Codec::serialize(&z00z_utils::codec::JsonCodec, &tx).expect("serialize tx");
    let json = String::from_utf8(encoded).expect("tx json");
    let without_statement = json.replace(
        &format!(
            "\"statement_hex\":\"{}\",",
            tx.proof.spend.as_ref().expect("spend proof").statement_hex
        ),
        "",
    );

    let err =
        Codec::deserialize::<TxWire>(&z00z_utils::codec::JsonCodec, without_statement.as_bytes())
            .expect_err("proof carrier must require an explicit statement_hex field");

    let err_text = err.to_string();
    assert!(
        err_text.contains("statement_hex"),
        "decode error should mention the missing statement_hex field, got: {err_text}"
    );
}

#[test]
fn test_spend_proof_missing_blob() {
    let (tx, _) = test_spend_wire_support::canonical_public_contract_tx();
    let encoded = Codec::serialize(&z00z_utils::codec::JsonCodec, &tx).expect("serialize tx");
    let json = String::from_utf8(encoded).expect("tx json");
    let without_proof_blob = json.replace(
        &format!(
            "\"proof_hex\":\"{}\",",
            tx.proof.spend.as_ref().expect("spend proof").proof_hex
        ),
        "",
    );

    let err =
        Codec::deserialize::<TxWire>(&z00z_utils::codec::JsonCodec, without_proof_blob.as_bytes())
            .expect_err("proof carrier must require an explicit proof_hex field");

    let err_text = err.to_string();
    assert!(
        err_text.contains("proof_hex"),
        "decode error should mention the missing proof_hex field, got: {err_text}"
    );
}

#[test]
fn test_wire_empty_placeholder_blob() {
    let (mut tx, _) = test_spend_wire_support::canonical_public_contract_tx();
    let spend_proof = tx.proof.spend.as_mut().expect("spend proof");
    spend_proof.statement_hex.clear();
    spend_proof.proof_hex.clear();

    let err = verify_tx_public_spend_contract(
        test_spend_wire_support::CHAIN_ID,
        test_spend_wire_support::TX_VERSION,
        test_spend_wire_support::CHAIN_TYPE,
        test_spend_wire_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("non-empty carrier mode must reject empty placeholder proof payloads");

    assert_eq!(err, SpendPublicErr::MissingStatement);
}

#[test]
fn test_spend_not_stub_pinned() {
    let (mut tx, _) = test_spend_wire_support::canonical_public_contract_tx();
    tx.proof.spend.as_mut().expect("spend proof").proof_hex = "aa".repeat(32);

    let err = verify_tx_public_spend_contract(
        test_spend_wire_support::CHAIN_ID,
        test_spend_wire_support::TX_VERSION,
        test_spend_wire_support::CHAIN_TYPE,
        test_spend_wire_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("statement-unbound proof_hex must reject at the admission boundary");

    assert_eq!(err, SpendPublicErr::BadProofBlob);
}

#[test]
fn test_wire_bad_auth_version() {
    let (mut tx, _) = test_spend_wire_support::canonical_public_contract_tx();
    tx.auth.spend.as_mut().expect("spend auth").ver += 1;

    let err = verify_tx_public_spend_contract(
        test_spend_wire_support::CHAIN_ID,
        test_spend_wire_support::TX_VERSION,
        test_spend_wire_support::CHAIN_TYPE,
        test_spend_wire_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("auth version drift must reject");

    assert_eq!(err, SpendPublicErr::BadAuthVersion);
}

#[test]
fn test_spend_statement_hex_rejects() {
    let (mut tx, _) = test_spend_wire_support::canonical_public_contract_tx();
    tx.proof.spend.as_mut().expect("spend proof").statement_hex = tx
        .proof
        .spend
        .as_ref()
        .expect("spend proof")
        .statement_hex
        .to_uppercase();

    let err = verify_tx_public_spend_contract(
        test_spend_wire_support::CHAIN_ID,
        test_spend_wire_support::TX_VERSION,
        test_spend_wire_support::CHAIN_TYPE,
        test_spend_wire_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("uppercase statement_hex must reject the canonical proof carrier");

    assert_eq!(
        err,
        SpendPublicErr::InvalidHex {
            label: "proof.statement_hex",
        }
    );
}

#[test]
fn test_wire_noncanonical_proof_hex() {
    let (mut tx, _) = test_spend_wire_support::canonical_public_contract_tx();
    tx.proof.spend.as_mut().expect("spend proof").proof_hex = tx
        .proof
        .spend
        .as_ref()
        .expect("spend proof")
        .proof_hex
        .to_uppercase();

    let err = verify_tx_public_spend_contract(
        test_spend_wire_support::CHAIN_ID,
        test_spend_wire_support::TX_VERSION,
        test_spend_wire_support::CHAIN_TYPE,
        test_spend_wire_support::CHAIN_NAME,
        &tx,
    )
    .expect_err("uppercase proof_hex must reject the canonical proof carrier");

    assert_eq!(
        err,
        SpendPublicErr::InvalidHex {
            label: "proof.proof_hex",
        }
    );
}
