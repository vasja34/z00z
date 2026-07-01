use std::sync::Arc;

use z00z_core::{genesis::asset_std::asset_from_dev_class, AssetClass, AssetWire};
use z00z_crypto::{
    create_range_proof, ClaimProofVer, ClaimSourceProof, Z00ZScalar, CLAIM_ROOT_VERSION,
};
use z00z_simulator::scenario_1::claim_pkg_consumer::{load_claim_packages, wrap_claim_packages};
use z00z_simulator::scenario_1::stage_3::{build_claim_package, write_claim_bundle_store};
use z00z_storage::settlement::{
    DefinitionId, SerialId, SettlementPath, SettlementStore, StoreItem, TerminalId,
};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::write_file,
};
use z00z_wallets::tx::{
    build_claim_stmt, build_claim_tx_digest, derive_output_nonce, sign_claim_auth, ClaimTxPackage,
};
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::{ScanResult, StealthOutputScanner},
    stealth::{build_tx_output_unchecked, SenderWallet},
};

pub(crate) const ZERO_ROOT: [u8; 32] = [0u8; 32];

fn make_claim_source_proof(
    leaf_pkg: &z00z_core::assets::AssetPkgWire,
) -> Result<ClaimSourceProof, String> {
    let item = claim_source_item(leaf_pkg)?;
    let mut store = SettlementStore::new();
    store
        .put_settlement_item(item.clone())
        .map_err(|e| format!("claim source store insert failed: {e}"))?;
    let (_, proof) = store
        .claim_source_contract_for_item(&item)
        .map_err(|e| format!("claim source proof failed: {e}"))?;
    Ok(proof)
}

fn claim_source_item(leaf_pkg: &z00z_core::assets::AssetPkgWire) -> Result<StoreItem, String> {
    let wire = leaf_pkg
        .clone()
        .to_wire()
        .map_err(|e| format!("claim source to_wire failed: {e}"))?;
    let leaf = z00z_wallets::tx::asset_wire_to_leaf(&wire)
        .map_err(|e| format!("claim source leaf failed: {e}"))?;
    let path = SettlementPath::new(
        DefinitionId::new(wire.definition.id),
        SerialId::new(leaf.serial_id),
        TerminalId::new(leaf.asset_id),
    );
    StoreItem::new(path, leaf).map_err(|e| format!("claim source item failed: {e}"))
}

fn make_keys_with_seed(byte: u8) -> ReceiverKeys {
    let recv = ReceiverSecret::from_bytes([byte; 32]).expect("receiver secret");
    ReceiverKeys::from_receiver_secret(recv).expect("receiver keys")
}

fn make_keys() -> ReceiverKeys {
    make_keys_with_seed(0x11)
}

fn rebuild_def(
    definition: &z00z_core::AssetDefinition,
    serial_id: u32,
) -> z00z_core::AssetDefinition {
    z00z_core::AssetDefinition::new(
        ZERO_ROOT,
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

fn make_wire(serial_id: u32, keys: &ReceiverKeys) -> AssetWire {
    let mut asset = asset_from_dev_class(AssetClass::Coin, 0, 10).expect("asset");
    let def = rebuild_def(asset.definition.as_ref(), serial_id);
    asset.definition = Arc::new(def);

    let card = keys.export_receiver_card().expect("card");
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
    .expect("output");

    let commitment = z00z_crypto::Commitment::from_bytes(&output.c_amount).expect("commitment");
    asset.commitment = commitment.as_commitment().clone();
    asset.owner_pub = None;
    asset.owner_signature = None;
    asset.r_pub = Some(output.r_pub);
    asset.owner_tag = Some(output.owner_tag);
    asset.enc_pack = Some(output.enc_pack);
    asset.tag16 = output.tag16;
    asset.leaf_ad_id = Some(asset.definition.id);

    let scanner = StealthOutputScanner::from_keys(keys);
    let ScanResult::Mine { wallet_output } = scanner.scan_leaf(&asset) else {
        panic!("owned leaf expected")
    };
    let blinding =
        Z00ZScalar::try_from_bytes(wallet_output.blinding.expect("blinding")).expect("scalar");
    asset.range_proof = Some(create_range_proof(asset.amount, &blinding, 64, 0).expect("proof"));

    let mut wire = AssetWire::from_asset(&asset);
    wire.secret = None;
    wire
}

fn make_pkg(serial_id: u32, keys: &ReceiverKeys) -> ClaimTxPackage {
    let wire = make_wire(serial_id, keys);
    let claim_id = derive_output_nonce(&wire.definition.id, wire.serial_id);
    let asset_id_hex = hex::encode(wire.clone().to_asset().expect("claim asset").asset_id());
    let pkg_bytes = build_claim_package(
        3,
        "devnet",
        "z00z-devnet-1",
        "alice",
        &asset_id_hex,
        wire.amount,
        &claim_id,
        &keys.owner_handle,
        serial_id as u64,
        Some(wire),
        Some(keys),
    )
    .expect("build claim pkg");
    z00z_utils::codec::JsonCodec
        .deserialize(&pkg_bytes)
        .expect("decode claim pkg")
}

pub fn patch_claim_crypto(pkg: &mut ClaimTxPackage) {
    let proof = make_claim_source_proof(pkg.tx.outputs[0].asset_wire.as_ref().expect("asset wire"))
        .expect("claim proof");

    pkg.tx.proof.proof_type = "claim_source".to_string();
    pkg.tx.proof.proof_hex = hex::encode(proof.to_bytes().expect("proof bytes"));
    let stmt = build_claim_stmt(pkg).expect("claim stmt");
    let sig = sign_claim_auth(&stmt).expect("claim auth");
    pkg.tx.auth.claim_authority_sig_hex = hex::encode(sig.to_bytes().expect("sig bytes"));
    pkg.tx_digest_hex = build_claim_tx_digest(
        &pkg.kind,
        &pkg.package_type,
        pkg.version,
        pkg.chain_id,
        &pkg.chain_type,
        &pkg.chain_name,
        &pkg.tx,
    )
    .expect("claim digest");
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
    .expect("claim digest");
}

fn decode_pkg_proof(pkg: &ClaimTxPackage) -> ClaimSourceProof {
    let proof_bytes = hex::decode(&pkg.tx.proof.proof_hex).expect("proof hex");
    ClaimSourceProof::from_bytes(&proof_bytes).expect("proof")
}

fn claim_pkg_path(dir: &tempfile::TempDir) -> std::path::PathBuf {
    dir.path().join("tx_claim_pkg.json")
}

fn write_claim_bundle_with_auth(
    auth_packages: Vec<ClaimTxPackage>,
    bundle_packages: Vec<ClaimTxPackage>,
) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp claim bundle dir");
    write_claim_bundle_store(dir.path(), &auth_packages).expect("write claim store");
    let bundle = wrap_claim_packages(bundle_packages);
    let bytes = JsonCodec.serialize(&bundle).expect("bundle encode");
    write_file(claim_pkg_path(&dir), &bytes).expect("bundle write");
    dir
}

fn write_claim_pkg_bundle(packages: Vec<ClaimTxPackage>) -> tempfile::TempDir {
    write_claim_bundle_with_auth(packages.clone(), packages)
}

fn write_claim_bundle_value(value: &serde_json::Value) -> tempfile::NamedTempFile {
    let file = tempfile::NamedTempFile::new().expect("temp claim bundle");
    let bytes = serde_json::to_vec(value).expect("bundle json encode");
    write_file(file.path(), &bytes).expect("bundle write");
    file
}

#[test]
fn test_authoritative_root_claim_emit() {
    let keys = make_keys();
    let pkg = make_pkg(19, &keys);
    let proof = decode_pkg_proof(&pkg);
    let item = claim_source_item(pkg.tx.outputs[0].asset_wire.as_ref().expect("asset wire"))
        .expect("claim source item");
    let mut store = SettlementStore::new();
    store.put_settlement_item(item.clone()).expect("put item");
    let (claim_root, expected_proof) = store
        .claim_source_contract_for_item(&item)
        .expect("storage-backed canonical contract");

    assert_eq!(proof.root_version(), CLAIM_ROOT_VERSION);
    assert_eq!(proof.proof_ver(), ClaimProofVer::V1);
    assert_ne!(proof.source_root(), [0u8; 32]);
    assert_eq!(proof.source_root(), claim_root.into_bytes());
    assert_eq!(proof.proof_blob(), expected_proof.proof_blob());
}

#[test]
fn test_package_rejects_authority_anchor() {
    let keys = make_keys();
    let mut pkg = make_pkg(23, &keys);
    pkg.chain_name = "z00z-mainnet-1".to_string();
    refresh_digest(&mut pkg);

    let file = write_claim_pkg_bundle(vec![pkg]);
    let err =
        load_claim_packages(&claim_pkg_path(&file)).expect_err("non-simulator anchor must fail");

    assert!(err.contains("claim authority anchor is simulator-only"));
}

#[test]
fn test_package_rejects_version_mismatch() {
    let keys = make_keys();
    let pkg = make_pkg(27, &keys);
    let mut bundle = serde_json::to_value(wrap_claim_packages(vec![pkg])).expect("bundle value");
    bundle["version"] = serde_json::Value::from(2_u32);

    let file = write_claim_bundle_value(&bundle);
    let err = load_claim_packages(file.path()).expect_err("bundle version mismatch must fail");

    assert!(err.contains("claim package bundle version mismatch"));
}

#[test]
fn test_package_rejects_package_shape() {
    let keys = make_keys();
    let pkg = make_pkg(28, &keys);
    let raw = serde_json::to_value(pkg).expect("raw package value");

    let file = write_claim_bundle_value(&raw);
    let err = load_claim_packages(file.path()).expect_err("raw package shape must fail");

    assert!(err.contains("claim package bundle parse failed"));
}

#[test]
fn test_package_rejects_storage_proof() {
    let keys = make_keys();
    let mut pkg = make_pkg(29, &keys);
    let auth_pkg = pkg.clone();
    let other_pkg = make_pkg(31, &keys);
    pkg.tx.proof.proof_hex = other_pkg.tx.proof.proof_hex.clone();
    refresh_digest(&mut pkg);

    let file = write_claim_bundle_with_auth(vec![auth_pkg], vec![pkg]);
    let err = load_claim_packages(&claim_pkg_path(&file)).expect_err("stale proof must fail");

    assert!(
        err.contains("claim_proof_invalid")
            || err.contains("helper-owned canonical")
            || err.contains("claim source root mismatch")
    );
}

#[test]
fn test_package_rejects_precise_error() {
    let keys = make_keys();
    let mut pkg = make_pkg(32, &keys);
    let auth_pkg = pkg.clone();
    let proof = decode_pkg_proof(&pkg);
    let bad = ClaimSourceProof::new(
        proof.root_version(),
        [0x77u8; 32],
        proof.proof_ver(),
        proof.proof_blob().to_vec(),
    )
    .expect("proof");
    pkg.tx.proof.proof_hex = hex::encode(bad.to_bytes().expect("proof bytes"));
    refresh_digest(&mut pkg);

    let file = write_claim_bundle_with_auth(vec![auth_pkg], vec![pkg]);
    let err = load_claim_packages(&claim_pkg_path(&file)).expect_err("stale source root must fail");

    assert!(err.contains("claim_proof_invalid"));
    assert!(err.contains("claim_source_proof root does not match proof blob root"));
}

#[test]
fn test_package_rejects_proof_error() {
    let keys = make_keys();
    let mut pkg = make_pkg(33, &keys);
    let auth_pkg = pkg.clone();
    let other_pkg = make_pkg(34, &keys);
    let proof = decode_pkg_proof(&pkg);
    let other_proof = decode_pkg_proof(&other_pkg);
    let bad = ClaimSourceProof::new(
        proof.root_version(),
        proof.source_root(),
        proof.proof_ver(),
        other_proof.proof_blob().to_vec(),
    )
    .expect("proof");
    pkg.tx.proof.proof_hex = hex::encode(bad.to_bytes().expect("proof bytes"));
    refresh_digest(&mut pkg);

    let file = write_claim_bundle_with_auth(vec![auth_pkg], vec![pkg]);
    let err = load_claim_packages(&claim_pkg_path(&file)).expect_err("stale proof blob must fail");

    assert!(err.contains("claim_proof_invalid"));
    assert!(err.contains("claim_source_proof root does not match proof blob root"));
}

#[test]
fn test_package_rejects_root_version() {
    let keys = make_keys();
    let mut pkg = make_pkg(41, &keys);
    let proof = decode_pkg_proof(&pkg);
    let bad = ClaimSourceProof::new(
        3,
        proof.source_root(),
        proof.proof_ver(),
        proof.proof_blob().to_vec(),
    )
    .expect("proof");
    pkg.tx.proof.proof_hex = hex::encode(bad.to_bytes().expect("proof bytes"));
    refresh_digest(&mut pkg);

    let file = write_claim_bundle_with_auth(vec![make_pkg(41, &keys)], vec![pkg]);
    let err =
        load_claim_packages(&claim_pkg_path(&file)).expect_err("invalid root version must fail");

    assert!(err.contains("claim source root version invalid"));
}

#[test]
fn test_package_rejects_proof_ver() {
    let keys = make_keys();
    let mut pkg = make_pkg(43, &keys);
    let auth_pkg = pkg.clone();
    let proof = decode_pkg_proof(&pkg);
    let bad = ClaimSourceProof::new(
        proof.root_version(),
        proof.source_root(),
        ClaimProofVer::V2,
        proof.proof_blob().to_vec(),
    )
    .expect("proof");
    pkg.tx.proof.proof_hex = hex::encode(bad.to_bytes().expect("proof bytes"));
    refresh_digest(&mut pkg);

    let file = write_claim_bundle_with_auth(vec![auth_pkg], vec![pkg]);
    let err =
        load_claim_packages(&claim_pkg_path(&file)).expect_err("invalid proof version must fail");

    assert!(err.contains("claim source proof version invalid"));
}

#[test]
fn test_package_rejects_authority_signature() {
    let keys = make_keys();
    let mut pkg = make_pkg(37, &keys);
    let auth_pkg = pkg.clone();
    let mut sig_bytes = hex::decode(&pkg.tx.auth.claim_authority_sig_hex).expect("sig hex");
    sig_bytes[0] ^= 0x01;
    pkg.tx.auth.claim_authority_sig_hex = hex::encode(sig_bytes);
    refresh_digest(&mut pkg);

    let file = write_claim_bundle_with_auth(vec![auth_pkg], vec![pkg]);
    let err = load_claim_packages(&claim_pkg_path(&file))
        .expect_err("wrong authority signature must fail");

    assert!(err.contains("claim_authority_invalid"));
}

#[test]
fn test_package_rejects_binding_drift() {
    let keys = make_keys();
    let other_keys = make_keys_with_seed(0x22);
    let mut pkg = make_pkg(45, &keys);
    let auth_pkg = pkg.clone();
    let claim_id = hex::decode(&pkg.tx.inputs[0].claim_id_hex).expect("claim id hex");
    let mut claim_id_bytes = [0u8; 32];
    claim_id_bytes.copy_from_slice(&claim_id);

    pkg.tx.context.recipient_owner_hex = hex::encode(other_keys.owner_handle);
    pkg.tx.context.recipient_card_hex = Some(hex::encode(
        other_keys
            .export_receiver_card()
            .expect("recipient card")
            .canonical_encoding(),
    ));
    pkg.tx.context.nullifier_hex =
        z00z_wallets::claim::derive_nullifier(&claim_id_bytes, &other_keys.owner_handle, 3)
            .to_hex();
    pkg.tx.outputs[0].owner_binding_hex = hex::encode(other_keys.owner_handle);
    refresh_digest(&mut pkg);

    let file = write_claim_bundle_with_auth(vec![auth_pkg], vec![pkg]);
    let err =
        load_claim_packages(&claim_pkg_path(&file)).expect_err("recipient binding drift must fail");

    assert!(err.contains("claim_authority_invalid"));
}

#[test]
fn test_package_rejects_asset_drift() {
    let keys = make_keys();
    let mut pkg = make_pkg(47, &keys);
    let auth_pkg = pkg.clone();
    let other_pkg = make_pkg(49, &keys);
    pkg.tx.inputs[0].claim_source_asset_id_hex =
        other_pkg.tx.inputs[0].claim_source_asset_id_hex.clone();
    refresh_digest(&mut pkg);

    let file = write_claim_bundle_with_auth(vec![auth_pkg], vec![pkg]);
    let err =
        load_claim_packages(&claim_pkg_path(&file)).expect_err("source asset path drift must fail");

    assert!(
        err.contains("claim_proof_invalid") || err.contains("path asset_id mismatch"),
        "unexpected error: {err}"
    );
}
