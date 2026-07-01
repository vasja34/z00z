use super::*;

use z00z_core::{
    assets::AssetPkgWire, genesis::asset_std::asset_from_dev_class, AssetClass, AssetWire,
};
use z00z_crypto::claim::{ClaimAuthoritySig, ClaimProofVer, ClaimSourceProof, CLAIM_ROOT_VERSION};
use z00z_storage::settlement::{
    DefinitionId, SerialId, SettlementPath, SettlementStore, StoreItem, TerminalId,
};

use crate::{
    key::{ReceiverKeys, ReceiverSecret},
    stealth::{build_tx_output_unchecked, compute_leaf_ad, SenderWallet},
};

const CHAIN_ID: u32 = 3;
const CHAIN_TYPE: &str = "devnet";
const CHAIN_NAME: &str = "z00z-devnet-1";
const INVALID_PROOF_BYTES: &[u8] = b"scenario_1_invalid_proof";
const INVALID_SIG_BYTES: &[u8] = b"scenario_1_invalid_sig";

fn make_claim_source_proof(leaf_pkg: &AssetPkgWire) -> Result<ClaimSourceProof, String> {
    let wire = leaf_pkg
        .clone()
        .to_wire()
        .map_err(|e| format!("claim source to_wire failed: {e}"))?;
    let leaf = asset_wire_to_leaf(&wire).map_err(|e| format!("claim source leaf failed: {e}"))?;
    let path = SettlementPath::new(
        DefinitionId::new(wire.definition.id),
        SerialId::new(leaf.serial_id),
        TerminalId::new(leaf.asset_id),
    );
    let item = StoreItem::new(path, leaf).map_err(|e| format!("claim source item failed: {e}"))?;
    let mut store = SettlementStore::new();
    store
        .put_settlement_item(item.clone())
        .map_err(|e| format!("claim source store insert failed: {e}"))?;
    let (_, proof) = store
        .claim_source_contract_for_item(&item)
        .map_err(|e| format!("claim source proof failed: {e}"))?;
    Ok(proof)
}

fn make_keys() -> ReceiverKeys {
    let recv = ReceiverSecret::from_bytes([0x11u8; 32]).unwrap();
    ReceiverKeys::from_receiver_secret(recv).unwrap()
}

fn rebuild_def(
    definition: &z00z_core::AssetDefinition,
    serial_id: u32,
) -> z00z_core::AssetDefinition {
    z00z_core::AssetDefinition::new(
        [0u8; 32],
        definition.class,
        format!("{}-{serial_id}", definition.name),
        definition.symbol.clone(),
        definition.decimals,
        definition.serials,
        definition.nominal,
        definition.domain_name.clone(),
        definition.version,
        definition.crypto_version,
        definition.policy_flags,
        definition.metadata.clone(),
    )
    .expect("canonical test definition")
}

fn make_stealth_wire(serial_id: u32, keys: &ReceiverKeys) -> AssetWire {
    let mut asset = asset_from_dev_class(AssetClass::Coin, 0, 100).unwrap();
    let def = rebuild_def(asset.definition.as_ref(), serial_id);
    asset.definition = std::sync::Arc::new(def);
    asset.owner_pub = None;
    asset.owner_signature = None;

    let card = keys.export_receiver_card().unwrap();
    let tx_seed = derive_output_nonce(&asset.definition.id, asset.serial_id);
    let mut sender_wallet = SenderWallet::new([41u8; 32]);
    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut sender_wallet,
        &tx_seed,
        0,
        asset.amount,
        &asset.definition.id,
    )
    .unwrap();

    asset.r_pub = Some(output.r_pub);
    asset.owner_tag = Some(output.owner_tag);
    asset.enc_pack = Some(output.enc_pack);
    asset.tag16 = output.tag16;
    let mut c_amount = [0u8; 32];
    c_amount.copy_from_slice(asset.commitment.as_bytes());
    asset.leaf_ad_id = Some(compute_leaf_ad(
        &asset.definition.id,
        asset.serial_id,
        &output.r_pub,
        &output.owner_tag,
        &c_amount,
    ));

    let mut wire = AssetWire::from_asset(&asset);
    wire.secret = None;
    wire
}

fn make_valid_pkg() -> ClaimTxPackage {
    let keys = make_keys();
    let wire = make_stealth_wire(7, &keys);
    let claim_id = derive_output_nonce(&wire.definition.id, wire.serial_id);
    let owner = keys.owner_handle;
    let nullifier = crate::claim::derive_nullifier(&claim_id, &owner, CHAIN_ID);
    let asset_id_hex = hex::encode(wire.clone().to_asset().unwrap().asset_id());

    let mut tx = ClaimTxWire {
        tx_type: "claim_tx".to_string(),
        inputs: vec![ClaimInputWire {
            claim_id_hex: hex::encode(claim_id),
            claim_source_asset_id_hex: asset_id_hex.clone(),
            claim_source_commitment_hex: hex::encode(wire.commitment.as_bytes()),
        }],
        outputs: vec![ClaimOutputWire {
            asset_id_hex,
            amount: wire.amount,
            asset_class: "coin".to_string(),
            owner_binding_hex: hex::encode(owner),
            nonce_hex: hex::encode(derive_output_nonce(&claim_id, 0)),
            asset_wire: Some(AssetPkgWire::from_wire(&wire)),
            owner_attest_hex: None,
        }],
        fee: 0,
        nonce: 0,
        context: ClaimContextWire {
            recipient_wallet_id: "alice".to_string(),
            recipient_owner_hex: hex::encode(owner),
            claim_scope_hash_hex: hex::encode(compute_claim_scope_hash(&ClaimScopeKey {
                chain_id: CHAIN_ID,
                scenario_tag: "scenario_1_genesis_claim".to_string(),
                ruleset_version: 1,
            })),
            recipient_card_hex: Some(hex::encode(
                keys.export_receiver_card().unwrap().canonical_encoding(),
            )),
            nullifier_hex: nullifier.to_hex(),
        },
        proof: ClaimProofWire {
            proof_type: "genesis_claim".to_string(),
            proof_hex: String::new(),
        },
        auth: ClaimAuthWire {
            claim_authority_sig_hex: String::new(),
        },
    };

    let owner_attest_hex = sign_owner_attest(
        &keys,
        CHAIN_ID,
        &tx,
        0,
        tx.outputs[0].asset_wire.as_ref().unwrap(),
    )
    .unwrap();
    tx.outputs[0].owner_attest_hex = Some(owner_attest_hex);

    let mut pkg = ClaimTxPackage {
        kind: "TxPackage".to_string(),
        package_type: "claim_tx".to_string(),
        version: CLAIM_PKG,
        chain_id: CHAIN_ID,
        chain_type: CHAIN_TYPE.to_string(),
        chain_name: CHAIN_NAME.to_string(),
        tx,
        tx_digest_hex: String::new(),
        status: "prepared".to_string(),
    };
    fill_claim_crypto(&mut pkg);
    refresh_digest(&mut pkg);
    pkg
}

fn fill_claim_crypto(pkg: &mut ClaimTxPackage) {
    let proof = make_claim_source_proof(pkg.tx.outputs[0].asset_wire.as_ref().unwrap()).unwrap();

    pkg.tx.proof.proof_type = "claim_source".to_string();
    pkg.tx.proof.proof_hex = hex::encode(proof.to_bytes().unwrap());
    let stmt = build_claim_stmt(pkg).unwrap();
    let sig = sign_claim_auth(&stmt).unwrap();
    pkg.tx.auth.claim_authority_sig_hex = hex::encode(sig.to_bytes().unwrap());
}

fn refresh_digest(pkg: &mut ClaimTxPackage) {
    pkg.tx_digest_hex = build_claim_tx_digest(
        &pkg.kind,
        &pkg.package_type,
        pkg.version,
        pkg.chain_id,
        &pkg.chain_type,
        &pkg.chain_name,
        &pkg.tx,
    )
    .unwrap();
}

fn pkg_bytes(pkg: &ClaimTxPackage) -> Vec<u8> {
    let codec = JsonCodec;
    codec.serialize(pkg).unwrap()
}

#[test]
fn test_empty_bytes() {
    let res = ClaimTxVerifierImpl::new().verify(&[]);
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_malformed_json");
}

#[test]
fn test_bad_kind() {
    let mut pkg = make_valid_pkg();
    pkg.kind = "BadKind".to_string();
    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_structure_invalid");
}

#[test]
fn test_bad_version() {
    let mut pkg = make_valid_pkg();
    pkg.version = 3;
    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_structure_invalid");
}

#[test]
fn test_whitespace_chain_metadata() {
    let mut pkg = make_valid_pkg();
    pkg.chain_type = "   ".to_string();
    pkg.chain_name = "\t".to_string();

    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_structure_invalid");
}

#[test]
fn test_chain_id_zero_rejected() {
    let mut pkg = make_valid_pkg();
    pkg.chain_id = 0;

    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_structure_invalid");
}

#[test]
fn test_claim_pkg_requires_leaf() {
    let mut pkg = make_valid_pkg();
    pkg.tx.outputs[0].asset_wire = None;
    pkg.tx.outputs[0].owner_attest_hex = None;
    refresh_digest(&mut pkg);
    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_output_invalid");
}

#[test]
fn test_nonzero_fee() {
    let mut pkg = make_valid_pkg();
    pkg.tx.fee = 9;
    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_fee_invalid");
}

#[test]
fn test_bad_proof_type() {
    let mut pkg = make_valid_pkg();
    pkg.tx.proof.proof_type = "other".to_string();
    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_proof_invalid");
}

#[test]
fn test_bad_nullifier_hex() {
    let mut pkg = make_valid_pkg();
    pkg.tx.context.nullifier_hex = "ZZZZ".to_string();
    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_nullifier_invalid");
}

#[test]
fn test_nullifier_mismatch() {
    let mut pkg = make_valid_pkg();
    pkg.tx.context.nullifier_hex = "bb".repeat(32);
    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_nullifier_invalid");
}

#[test]
fn test_bad_asset_len() {
    let mut pkg = make_valid_pkg();
    pkg.tx.outputs[0].asset_id_hex = "11".repeat(31);
    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_output_invalid");
}

#[test]
fn test_id_as_asset_fails() {
    let mut pkg = make_valid_pkg();
    let wrong = pkg.tx.outputs[0]
        .asset_wire
        .as_ref()
        .expect("asset wire")
        .definition
        .id;
    pkg.tx.outputs[0].asset_id_hex = hex::encode(wrong);
    refresh_digest(&mut pkg);

    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_output_invalid");
}

#[test]
fn test_bad_scope_hash() {
    let mut pkg = make_valid_pkg();
    pkg.tx.context.claim_scope_hash_hex = "ff".repeat(32);
    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_structure_invalid");
}

#[test]
fn test_odd_sig_hex() {
    let mut pkg = make_valid_pkg();
    pkg.tx.auth.claim_authority_sig_hex = "abc".to_string();
    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_authority_invalid");
}

#[test]
fn test_bad_proof_stmt() {
    let mut pkg = make_valid_pkg();
    let proof = ClaimSourceProof::new(
        CLAIM_ROOT_VERSION,
        [0x55u8; 32],
        ClaimProofVer::V1,
        vec![9u8, 8, 7, 6],
    )
    .unwrap();
    pkg.tx.proof.proof_hex = hex::encode(proof.to_bytes().unwrap());
    refresh_digest(&mut pkg);

    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_proof_invalid");
}

#[test]
fn test_proof_blob_root_mix() {
    let mut pkg = make_valid_pkg();
    let proof_bytes = hex::decode(&pkg.tx.proof.proof_hex).unwrap();
    let proof = ClaimSourceProof::from_bytes(&proof_bytes).unwrap();
    let bad = ClaimSourceProof::new(
        proof.root_version(),
        [0x77u8; 32],
        proof.proof_ver(),
        proof.proof_blob().to_vec(),
    )
    .unwrap();
    pkg.tx.proof.proof_hex = hex::encode(bad.to_bytes().unwrap());
    refresh_digest(&mut pkg);

    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_proof_invalid");
}

#[test]
fn test_source_commitment_drift_rejected() {
    let mut pkg = make_valid_pkg();
    pkg.tx.inputs[0].claim_source_commitment_hex = "cc".repeat(32);
    refresh_digest(&mut pkg);

    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_proof_invalid");
}

#[test]
fn test_asset_id_drift_rejected() {
    let mut pkg = make_valid_pkg();
    pkg.tx.inputs[0].claim_source_asset_id_hex = "dd".repeat(32);
    refresh_digest(&mut pkg);

    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_proof_invalid");
    assert!(
        res.errors[0].contains("path asset_id mismatch"),
        "unexpected error: {}",
        res.errors[0]
    );

    let report = res.report.expect("report");
    assert!(report.nullifier_checked);
    assert!(report.card_checked);
    assert!(report.leaf_checked);
    assert!(!report.proof_checked);
}

#[test]
fn test_id_rejected_pre_proof() {
    let mut pkg = make_valid_pkg();
    pkg.chain_id = CHAIN_ID + 1;
    pkg.tx.context.claim_scope_hash_hex = hex::encode(compute_claim_scope_hash(&ClaimScopeKey {
        chain_id: pkg.chain_id,
        scenario_tag: "scenario_1_genesis_claim".to_string(),
        ruleset_version: 1,
    }));
    refresh_digest(&mut pkg);

    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_nullifier_invalid");
    assert!(
        res.errors[0].contains("nullifier_hex must equal derive_nullifier"),
        "unexpected error: {}",
        res.errors[0]
    );

    let report = res.report.expect("report");
    assert!(!report.nullifier_checked);
    assert!(!report.card_checked);
    assert!(!report.leaf_checked);
    assert!(!report.proof_checked);
}

#[test]
fn test_id_breaks_authority_tuple() {
    let mut pkg = make_valid_pkg();
    let claim_id = hex::decode(&pkg.tx.inputs[0].claim_id_hex).unwrap();
    let owner = hex::decode(&pkg.tx.context.recipient_owner_hex).unwrap();
    let mut claim_id_bytes = [0u8; 32];
    claim_id_bytes.copy_from_slice(&claim_id);
    let mut owner_bytes = [0u8; 32];
    owner_bytes.copy_from_slice(&owner);

    pkg.chain_id = CHAIN_ID + 1;
    pkg.tx.context.claim_scope_hash_hex = hex::encode(compute_claim_scope_hash(&ClaimScopeKey {
        chain_id: pkg.chain_id,
        scenario_tag: "scenario_1_genesis_claim".to_string(),
        ruleset_version: 1,
    }));
    pkg.tx.context.nullifier_hex =
        derive_nullifier(&claim_id_bytes, &owner_bytes, pkg.chain_id).to_hex();
    refresh_digest(&mut pkg);

    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_authority_invalid");

    let report = res.report.expect("report");
    assert!(report.nullifier_checked);
    assert!(report.card_checked);
    assert!(report.leaf_checked);
    assert!(report.proof_checked);
    assert!(!report.authority_checked);
    assert!(!report.owner_attest_checked);
    assert!(!report.digest_checked);
}

#[test]
fn test_ver_rejected_precise_error() {
    let mut pkg = make_valid_pkg();
    let proof_bytes = hex::decode(&pkg.tx.proof.proof_hex).unwrap();
    let proof = ClaimSourceProof::from_bytes(&proof_bytes).unwrap();
    let bad = ClaimSourceProof::new(
        3,
        proof.source_root(),
        proof.proof_ver(),
        proof.proof_blob().to_vec(),
    )
    .unwrap();
    pkg.tx.proof.proof_hex = hex::encode(bad.to_bytes().unwrap());
    refresh_digest(&mut pkg);

    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_proof_invalid");
    assert!(
        res.errors[0].contains("claim source root version invalid"),
        "unexpected error: {}",
        res.errors[0]
    );

    let report = res.report.expect("report");
    assert!(report.nullifier_checked);
    assert!(report.card_checked);
    assert!(report.leaf_checked);
    assert!(!report.proof_checked);
}

#[test]
fn test_mismatch_rejected_precise_error() {
    let mut pkg = make_valid_pkg();
    let proof_bytes = hex::decode(&pkg.tx.proof.proof_hex).unwrap();
    let proof = ClaimSourceProof::from_bytes(&proof_bytes).unwrap();
    let bad = ClaimSourceProof::new(
        proof.root_version(),
        [0x5au8; 32],
        proof.proof_ver(),
        proof.proof_blob().to_vec(),
    )
    .unwrap();
    pkg.tx.proof.proof_hex = hex::encode(bad.to_bytes().unwrap());
    refresh_digest(&mut pkg);

    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_proof_invalid");
    assert!(
        res.errors[0].contains("claim_source_proof root does not match proof blob root"),
        "unexpected error: {}",
        res.errors[0]
    );

    let report = res.report.expect("report");
    assert!(report.nullifier_checked);
    assert!(report.card_checked);
    assert!(report.leaf_checked);
    assert!(!report.proof_checked);
}

#[test]
fn test_proof_rejected_precise_error() {
    let mut pkg = make_valid_pkg();
    let proof_bytes = hex::decode(&pkg.tx.proof.proof_hex).unwrap();
    let proof = ClaimSourceProof::from_bytes(&proof_bytes).unwrap();
    let bad = ClaimSourceProof::new(
        proof.root_version(),
        proof.source_root(),
        ClaimProofVer::V2,
        proof.proof_blob().to_vec(),
    )
    .unwrap();
    pkg.tx.proof.proof_hex = hex::encode(bad.to_bytes().unwrap());
    refresh_digest(&mut pkg);

    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_proof_invalid");
    assert!(
        res.errors[0].contains("claim source proof version invalid"),
        "unexpected error: {}",
        res.errors[0]
    );

    let report = res.report.expect("report");
    assert!(report.nullifier_checked);
    assert!(report.card_checked);
    assert!(report.leaf_checked);
    assert!(!report.proof_checked);
}

#[test]
fn test_proof_blob_precise_error() {
    let mut pkg = make_valid_pkg();
    let other_pkg = make_valid_pkg();
    let proof_bytes = hex::decode(&pkg.tx.proof.proof_hex).unwrap();
    let proof = ClaimSourceProof::from_bytes(&proof_bytes).unwrap();
    let other_proof_bytes = hex::decode(&other_pkg.tx.proof.proof_hex).unwrap();
    let other_proof = ClaimSourceProof::from_bytes(&other_proof_bytes).unwrap();
    let bad = ClaimSourceProof::new(
        proof.root_version(),
        proof.source_root(),
        proof.proof_ver(),
        other_proof.proof_blob().to_vec(),
    )
    .unwrap();
    pkg.tx.proof.proof_hex = hex::encode(bad.to_bytes().unwrap());
    refresh_digest(&mut pkg);

    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_proof_invalid");
    assert!(
        res.errors[0].contains("claim_source_proof root does not match proof blob root"),
        "unexpected error: {}",
        res.errors[0]
    );

    let report = res.report.expect("report");
    assert!(report.nullifier_checked);
    assert!(report.card_checked);
    assert!(report.leaf_checked);
    assert!(!report.proof_checked);
}

#[test]
fn test_proof_beats_digest_mismatch() {
    let mut pkg = make_valid_pkg();
    let proof = ClaimSourceProof::new(
        CLAIM_ROOT_VERSION,
        [0x66u8; 32],
        ClaimProofVer::V1,
        vec![4u8, 3, 2, 1],
    )
    .unwrap();
    pkg.tx.proof.proof_hex = hex::encode(proof.to_bytes().unwrap());
    pkg.tx_digest_hex = "aa".repeat(32);

    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_proof_invalid");

    let report = res.report.expect("report");
    assert!(report.leaf_checked);
    assert!(!report.digest_checked);
}

#[test]
fn test_bad_auth_sig() {
    let mut pkg = make_valid_pkg();
    let mut sig_bytes = hex::decode(&pkg.tx.auth.claim_authority_sig_hex).unwrap();
    sig_bytes[4] ^= 1;
    let _ = ClaimAuthoritySig::from_bytes(&sig_bytes);
    pkg.tx.auth.claim_authority_sig_hex = hex::encode(sig_bytes);
    refresh_digest(&mut pkg);

    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_authority_invalid");
}

#[test]
fn test_invalid_proof_bytes() {
    let mut pkg = make_valid_pkg();
    pkg.tx.proof.proof_hex = hex::encode(INVALID_PROOF_BYTES);
    refresh_digest(&mut pkg);

    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_proof_invalid");
}

#[test]
fn test_zero_root_rejected() {
    let mut pkg = make_valid_pkg();
    let proof =
        ClaimSourceProof::new(CLAIM_ROOT_VERSION, [0u8; 32], ClaimProofVer::V1, vec![1u8]).unwrap();
    pkg.tx.proof.proof_hex = hex::encode(proof.to_bytes().unwrap());
    refresh_digest(&mut pkg);

    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_proof_invalid");
}

#[test]
fn test_invalid_sig_bytes() {
    let mut pkg = make_valid_pkg();
    pkg.tx.auth.claim_authority_sig_hex = hex::encode(INVALID_SIG_BYTES);
    refresh_digest(&mut pkg);

    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_authority_invalid");
}

#[test]
fn test_report_ok() {
    let pkg = make_valid_pkg();
    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(res.valid);

    let report = res.report.expect("report");
    assert!(report.nullifier_checked);
    assert!(report.card_checked);
    assert!(report.leaf_checked);
    assert!(report.proof_checked);
    assert!(report.authority_checked);
    assert!(report.owner_attest_checked);
    assert!(report.digest_checked);
}

#[test]
fn test_report_stops_auth() {
    let mut pkg = make_valid_pkg();
    pkg.tx.auth.claim_authority_sig_hex = "ab".repeat(3);
    refresh_digest(&mut pkg);

    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_authority_invalid");

    let report = res.report.expect("report");
    assert!(report.nullifier_checked);
    assert!(report.card_checked);
    assert!(report.leaf_checked);
    assert!(report.proof_checked);
    assert!(!report.authority_checked);
    assert!(!report.owner_attest_checked);
    assert!(!report.digest_checked);
}

#[test]
fn test_zero_nonce_out() {
    let mut pkg = make_valid_pkg();
    pkg.tx.outputs[0].nonce_hex = "00".repeat(32);
    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_output_invalid");
}

#[test]
fn test_fee_beats_digest_mismatch() {
    let mut pkg = make_valid_pkg();
    pkg.tx.fee = 9;
    pkg.tx_digest_hex = "bb".repeat(32);

    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_fee_invalid");

    let report = res.report.expect("report");
    assert!(!report.digest_checked);
    assert!(!report.leaf_checked);
}

#[test]
fn test_owner_binding_mismatch() {
    let mut pkg = make_valid_pkg();
    pkg.tx.outputs[0].owner_binding_hex = "03".repeat(32);
    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_output_invalid");
}

#[test]
fn test_nonce_mismatch() {
    let mut pkg = make_valid_pkg();
    pkg.tx.outputs[0].nonce_hex = "12".repeat(32);
    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_output_invalid");
}

#[test]
fn test_empty_outputs() {
    let mut pkg = make_valid_pkg();
    pkg.tx.outputs.clear();
    let res = ClaimTxVerifierImpl::new().verify(&pkg_bytes(&pkg));
    assert!(!res.valid);
    assert_eq!(res.reject_class, "claim_output_invalid");
}

#[test]
fn test_pkg_roundtrip() {
    let pkg = make_valid_pkg();
    let bytes = pkg_bytes(&pkg);
    let codec = JsonCodec;
    let out: ClaimTxPackage = codec.deserialize(&bytes).unwrap();
    assert_eq!(pkg, out);
}

#[test]
fn test_digest_same() {
    let pkg = make_valid_pkg();
    let digest = build_claim_tx_digest(
        &pkg.kind,
        &pkg.package_type,
        pkg.version,
        pkg.chain_id,
        &pkg.chain_type,
        &pkg.chain_name,
        &pkg.tx,
    )
    .unwrap();
    assert_eq!(pkg.tx_digest_hex, digest);
}

#[test]
fn test_scope_sep() {
    let k1 = ClaimScopeKey {
        chain_id: 1,
        scenario_tag: "scenario_1_genesis_claim".to_string(),
        ruleset_version: 1,
    };
    let k2 = ClaimScopeKey {
        chain_id: 2,
        scenario_tag: "scenario_1_genesis_claim".to_string(),
        ruleset_version: 1,
    };
    assert_ne!(compute_claim_scope_hash(&k1), compute_claim_scope_hash(&k2));
}
