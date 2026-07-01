//! Integration tests for ClaimTxPackage build + verify pipeline.

use std::sync::{Arc, Mutex, OnceLock};

use z00z_core::{assets::AssetWire, genesis::asset_std::asset_from_dev_class, Asset, AssetClass};
use z00z_simulator::scenario_1::stage_3::write_claim_bundle_fault;
use z00z_simulator::{
    scenario_1::claim_pkg_consumer::{
        build_claim_store_ops, publish_claims_store, wrap_claim_packages,
    },
    scenario_1::stage_3::{
        build_claim_package, patch_claim_bundle_membership, to_claim_wire, verify_resume_wire,
        write_claim_bundle, write_claim_bundle_store,
    },
};
use z00z_storage::settlement::{
    ClaimNullStatus, ClaimNullifier, DefinitionId, SerialId, SettlementPath, SettlementStore,
    TerminalId,
};
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_wallets::claim::nullifier_store::{
    bind_paths, clear_bind, clear_rows as clear_nulls, get_entry as get_null,
};
use z00z_wallets::{
    claim::{derive_nullifier, NullifierStatus},
    key::{ReceiverKeys, ReceiverSecret},
    receiver::ReceiverCard,
    tx::{
        asset_wire_to_leaf, compute_claim_scope_hash, derive_output_nonce, ClaimScopeKey,
        ClaimTxPackage, ClaimTxVerifier, ClaimTxVerifierImpl,
    },
};

const CHAIN_ID: u32 = 3;
const CHAIN_TYPE: &str = "devnet";
const CHAIN_NAME: &str = "z00z-devnet-1";

static NULL_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn null_test_lock() -> &'static Mutex<()> {
    NULL_TEST_LOCK.get_or_init(|| Mutex::new(()))
}

fn bind_null_dir(dir: &std::path::Path) {
    let row_path = dir.join("nullifier_rows.json");
    let audit_path = dir.join("nullifier_audit.json");
    bind_paths(&row_path, Some(&audit_path)).unwrap();
}

fn claim_store_path(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("claim_source_store.redb")
}

fn store_null(raw: &str) -> ClaimNullifier {
    ClaimNullifier::from_hex(raw).expect("claim nullifier")
}

fn make_keys() -> ReceiverKeys {
    let recv = ReceiverSecret::from_bytes([0x11u8; 32]).unwrap();
    ReceiverKeys::from_receiver_secret(recv).unwrap()
}

fn make_other_keys() -> ReceiverKeys {
    let recv = ReceiverSecret::from_bytes([0x22u8; 32]).unwrap();
    ReceiverKeys::from_receiver_secret(recv).unwrap()
}

fn make_card(keys: &ReceiverKeys) -> ReceiverCard {
    keys.export_receiver_card().unwrap()
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

fn make_claim_asset(serial_id: u32) -> Asset {
    let mut asset = asset_from_dev_class(AssetClass::Coin, 0, 10).unwrap();
    let def = rebuild_def(asset.definition.as_ref(), serial_id);
    asset.definition = Arc::new(def);
    asset.serial_id = serial_id;
    asset.owner_pub = None;
    asset.owner_signature = None;

    asset
}

fn make_stealth_wire(serial_id: u32, keys: &ReceiverKeys) -> AssetWire {
    let asset = make_claim_asset(serial_id);
    let card = make_card(keys);
    let tx_seed = derive_output_nonce(&asset.definition.id, asset.serial_id);
    to_claim_wire(&asset, keys, &card, &tx_seed).expect("wire")
}

fn make_pkg_portable(serial_id: u32, keys: &ReceiverKeys) -> ClaimTxPackage {
    let wire = make_stealth_wire(serial_id, keys);
    let leaf = asset_wire_to_leaf(&wire).unwrap();
    let claim_id = derive_output_nonce(&wire.definition.id, wire.serial_id);
    let pkg_bytes = build_claim_package(
        CHAIN_ID,
        CHAIN_TYPE,
        CHAIN_NAME,
        "alice-wallet",
        &hex::encode(leaf.asset_id),
        wire.amount,
        &claim_id,
        &keys.owner_handle,
        serial_id as u64,
        Some(wire),
        Some(keys),
    )
    .unwrap();
    JsonCodec.deserialize(&pkg_bytes).unwrap()
}

fn patch_bundle(packages: &mut [ClaimTxPackage]) {
    patch_claim_bundle_membership(packages).expect("patch bundle membership");
}

#[test]
fn test_portable_wire_serial_lane() {
    let keys = make_keys();
    let wire = make_stealth_wire(19, &keys);

    assert_eq!(wire.serial_id, 19);
    verify_resume_wire(&wire, &keys).expect("live stage-3 helper output must remain decryptable");
}

#[test]
fn test_claim_pkg_accepted() {
    let keys = make_keys();
    let pkg = make_pkg_portable(7, &keys);
    let bytes = JsonCodec.serialize(&pkg).unwrap();
    let result = ClaimTxVerifierImpl::new().verify(&bytes);
    assert!(
        result.valid,
        "stage-3 output must be accepted: {:?}",
        result.errors
    );
}

#[test]
fn test_digest_stable_same_input() {
    let keys = make_keys();
    let pkg = make_pkg_portable(7, &keys);
    let digest = z00z_wallets::tx::build_claim_tx_digest(
        &pkg.kind,
        &pkg.package_type,
        pkg.version,
        pkg.chain_id,
        &pkg.chain_type,
        &pkg.chain_name,
        &pkg.tx,
    )
    .unwrap();
    assert_eq!(
        pkg.tx_digest_hex, digest,
        "digest must match package payload"
    );
}

#[test]
fn test_nullifier_no_collision() {
    let owner = [0x02u8; 32];
    let n1 = derive_nullifier(&[0x01u8; 32], &owner, CHAIN_ID);
    let n2 = derive_nullifier(&[0x03u8; 32], &owner, CHAIN_ID);
    assert_ne!(n1.0, n2.0);
}

#[test]
fn test_malformed_proof_rejected() {
    let keys = make_keys();
    let mut pkg = make_pkg_portable(7, &keys);
    pkg.tx.proof.proof_hex = "ZZZZnotvalidhex".to_string();
    let bytes = JsonCodec.serialize(&pkg).unwrap();
    let result = ClaimTxVerifierImpl::new().verify(&bytes);
    assert!(!result.valid);
    assert_eq!(result.reject_class, "claim_proof_invalid");
}

#[test]
fn test_duplicate_nullifier_detected() {
    let keys = make_keys();
    let pkg1 = make_pkg_portable(7, &keys);
    let pkg2 = make_pkg_portable(7, &keys);
    assert_eq!(
        pkg1.tx.context.nullifier_hex, pkg2.tx.context.nullifier_hex,
        "identical inputs must produce identical nullifier"
    );
}

#[test]
fn test_nullifier_mismatch_rejected() {
    let keys = make_keys();
    let mut pkg = make_pkg_portable(7, &keys);
    pkg.tx.context.nullifier_hex = "bb".repeat(32);
    let bytes = JsonCodec.serialize(&pkg).unwrap();
    let result = ClaimTxVerifierImpl::new().verify(&bytes);
    assert!(!result.valid);
    assert_eq!(result.reject_class, "claim_nullifier_invalid");
}

#[test]
fn test_scope_chain_sep() {
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

#[test]
fn test_pkg_bytes_roundtrip() {
    let keys = make_keys();
    let bytes = JsonCodec.serialize(&make_pkg_portable(7, &keys)).unwrap();
    let decoded: ClaimTxPackage = JsonCodec.deserialize(&bytes).unwrap();
    let re_bytes = JsonCodec.serialize(&decoded).unwrap();
    assert_eq!(bytes, re_bytes);
}

#[test]
fn test_nonzero_fee_rejected() {
    let keys = make_keys();
    let mut pkg = make_pkg_portable(7, &keys);
    pkg.tx.fee = 100;
    let bytes = JsonCodec.serialize(&pkg).unwrap();
    let result = ClaimTxVerifierImpl::new().verify(&bytes);
    assert!(!result.valid);
    assert_eq!(result.reject_class, "claim_fee_invalid");
}

#[test]
fn test_pkg_with_leaf_accepted() {
    let keys = make_keys();
    let pkg = make_pkg_portable(7, &keys);
    let pkg_bytes = JsonCodec.serialize(&pkg).unwrap();

    assert_eq!(pkg.version, 1);
    assert_eq!(pkg.tx.outputs[0].asset_class, "coin");
    assert!(pkg.tx.context.recipient_card_hex.is_some());
    assert!(pkg.tx.outputs[0].asset_wire.is_some());
    assert!(pkg.tx.outputs[0].owner_attest_hex.is_some());
    assert!(pkg.tx.outputs[0]
        .asset_wire
        .clone()
        .unwrap()
        .to_wire()
        .unwrap()
        .secret
        .is_none());

    let result = ClaimTxVerifierImpl::new().verify(&pkg_bytes);
    assert!(
        result.valid,
        "claim package must be accepted: {:?}",
        result.errors
    );
}

#[test]
fn test_owner_attest_mismatch_rejected() {
    let keys = make_keys();
    let mut pkg = make_pkg_portable(7, &keys);
    pkg.tx.outputs[0].owner_attest_hex = Some("ab".repeat(64));
    let pkg_bytes = JsonCodec.serialize(&pkg).unwrap();

    let result = ClaimTxVerifierImpl::new().verify(&pkg_bytes);
    assert!(!result.valid);
    assert_eq!(result.reject_class, "claim_output_invalid");
}

#[test]
fn test_pkg_missing_card() {
    let keys = make_keys();
    let mut pkg = make_pkg_portable(7, &keys);
    pkg.tx.context.recipient_card_hex = None;
    let pkg_bytes = JsonCodec.serialize(&pkg).unwrap();

    let result = ClaimTxVerifierImpl::new().verify(&pkg_bytes);
    assert!(!result.valid);
    assert_eq!(result.reject_class, "claim_output_invalid");
}

#[test]
fn test_resume_wrong_recv() {
    let owner_keys = make_keys();
    let wrong_keys = make_other_keys();
    let wire = make_stealth_wire(7, &owner_keys);

    verify_resume_wire(&wire, &owner_keys).expect("owner keys must accept owned wire");

    let err = verify_resume_wire(&wire, &wrong_keys).expect_err("wrong receiver must fail");
    assert!(
        err.contains("owner binding mismatch"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_claim_pkg_jmt_publish() {
    let _guard = null_test_lock().lock().unwrap();
    clear_bind();
    clear_nulls();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tx_claim_pkg.json");
    let keys = make_keys();
    let mut packages = vec![make_pkg_portable(7, &keys), make_pkg_portable(8, &keys)];
    patch_bundle(&mut packages);
    write_claim_bundle(dir.path(), packages.clone()).unwrap();

    let mut store = SettlementStore::new();
    let report = publish_claims_store(&path, &mut store).unwrap();
    assert_eq!(report.package_count, 2);
    assert_eq!(report.leaf_count, 2);
    assert_eq!(report.inserted_count, 2);
    bind_null_dir(dir.path());

    for pkg in &packages {
        let asset_wire = pkg.tx.outputs[0].asset_wire.as_ref().unwrap();
        let wire = asset_wire.clone().to_wire().unwrap();
        let leaf = z00z_wallets::tx::asset_wire_to_leaf(&wire).unwrap();
        let item_path = SettlementPath::new(
            DefinitionId::new(wire.definition.id),
            SerialId::new(asset_wire.serial_id),
            TerminalId::new(leaf.asset_id),
        );
        let inserted = store.get_settlement_item(&item_path).unwrap().unwrap();
        let z00z_storage::settlement::SettlementLeaf::Terminal(inserted_leaf) = inserted.leaf()
        else {
            panic!("asset leaf");
        };
        assert_eq!(inserted_leaf.asset_id, leaf.asset_id);
        assert_eq!(inserted_leaf.serial_id, asset_wire.serial_id);
        assert_eq!(inserted_leaf.owner_tag, asset_wire.owner_tag.unwrap());

        let entry = store
            .settlement_claim_null_rec(&store_null(&pkg.tx.context.nullifier_hex))
            .unwrap()
            .expect("storage nullifier entry");
        assert_eq!(entry.status, ClaimNullStatus::Spent);
        assert_eq!(entry.tx_digest_hex, pkg.tx_digest_hex);
        assert!(get_null(&pkg.tx.context.nullifier_hex).is_none());
    }
}

#[test]
fn test_dup_asset_id() {
    let keys = make_keys();
    let first = make_pkg_portable(7, &keys);
    let mut second = make_pkg_portable(8, &keys);
    let dup_out = first.tx.outputs[0].clone();

    second.tx.outputs[0].asset_id_hex = dup_out.asset_id_hex.clone();
    second.tx.outputs[0].asset_wire = dup_out.asset_wire.clone();

    let err =
        build_claim_store_ops(&[first, second]).expect_err("dup settlement terminal must reject");
    assert!(
        err.contains("duplicate settlement terminal across claim packages"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_claim_pkg_replay_rejected() {
    let _guard = null_test_lock().lock().unwrap();
    clear_bind();
    clear_nulls();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tx_claim_pkg.json");
    let keys = make_keys();
    let packages = vec![make_pkg_portable(7, &keys)];
    write_claim_bundle(dir.path(), packages.clone()).unwrap();

    let store_root = dir.path().join("asset_store");
    let mut first_store = SettlementStore::load(&store_root).expect("open first store");
    publish_claims_store(&path, &mut first_store).expect("first publish");
    drop(first_store);

    let mut second_store = SettlementStore::load(&store_root).expect("open second store");
    let err = publish_claims_store(&path, &mut second_store).expect_err("replay must fail");
    assert!(
        err.contains("nullifier replay rejected"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_publish_needs_prior_reserve() {
    let _guard = null_test_lock().lock().unwrap();
    clear_bind();
    clear_nulls();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tx_claim_pkg.json");
    let keys = make_keys();
    let mut packages = vec![make_pkg_portable(17, &keys)];
    patch_bundle(&mut packages);
    write_claim_bundle_store(dir.path(), &packages).unwrap();
    z00z_utils::io::save_json(&path, &wrap_claim_packages(packages)).unwrap();

    let mut store = SettlementStore::new();
    let err =
        publish_claims_store(&path, &mut store).expect_err("publish must require reservation");
    assert!(
        err.contains("reservation missing before publish"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_stage3_replay_rejected() {
    let _guard = null_test_lock().lock().unwrap();
    clear_bind();
    clear_nulls();
    let dir = tempfile::tempdir().unwrap();
    let keys = make_keys();
    let mut packages = vec![make_pkg_portable(11, &keys)];
    patch_bundle(&mut packages);
    let null_hex = packages[0].tx.context.nullifier_hex.clone();

    write_claim_bundle(dir.path(), packages.clone()).unwrap();
    bind_null_dir(dir.path());
    let first = get_null(&null_hex).expect("reserved row");
    assert_eq!(first.status, NullifierStatus::Reserved);

    let err = write_claim_bundle(dir.path(), packages).expect_err("replay must fail");
    assert!(
        err.contains("nullifier replay rejected"),
        "unexpected error: {err}"
    );
    assert!(dir.path().join("tx_claim_pkg.json").exists());
}

#[test]
fn test_stage3_serialize_rollback() {
    let _guard = null_test_lock().lock().unwrap();
    clear_bind();
    clear_nulls();
    let dir = tempfile::tempdir().unwrap();
    let keys = make_keys();
    let mut packages = vec![make_pkg_portable(12, &keys)];
    patch_bundle(&mut packages);
    let null_hex = packages[0].tx.context.nullifier_hex.clone();

    let err = write_claim_bundle_fault(dir.path(), packages, "serialize_fail")
        .expect_err("serialize fault must fail");
    assert!(err.contains("claim package serialize failed"));
    bind_null_dir(dir.path());
    assert!(get_null(&null_hex).is_none());
    assert!(!dir.path().join("tx_claim_pkg.json").exists());
    assert!(!claim_store_path(dir.path()).exists());
}

#[test]
fn test_stage3_write_rollback() {
    let _guard = null_test_lock().lock().unwrap();
    clear_bind();
    clear_nulls();
    let dir = tempfile::tempdir().unwrap();
    let keys = make_keys();
    let mut packages = vec![make_pkg_portable(13, &keys)];
    patch_bundle(&mut packages);
    let null_hex = packages[0].tx.context.nullifier_hex.clone();

    let err = write_claim_bundle_fault(dir.path(), packages, "write_fail")
        .expect_err("write fault must fail");
    assert!(err.contains("claim package write failed"));
    bind_null_dir(dir.path());
    assert!(get_null(&null_hex).is_none());
    assert!(!dir.path().join("tx_claim_pkg.json").exists());
    assert!(!claim_store_path(dir.path()).exists());
}

#[test]
fn test_stage3_verify_rollback() {
    let _guard = null_test_lock().lock().unwrap();
    clear_bind();
    clear_nulls();
    let dir = tempfile::tempdir().unwrap();
    let keys = make_keys();
    let mut packages = vec![make_pkg_portable(14, &keys)];
    patch_bundle(&mut packages);
    let null_hex = packages[0].tx.context.nullifier_hex.clone();

    let err = write_claim_bundle_fault(dir.path(), packages, "verify_fail")
        .expect_err("verify fault must fail");
    assert!(err.contains("claim package verify failed"));
    bind_null_dir(dir.path());
    assert!(get_null(&null_hex).is_none());
    assert!(!dir.path().join("tx_claim_pkg.json").exists());
    assert!(!claim_store_path(dir.path()).exists());
}

#[test]
fn test_publish_finalizes_reserved_row() {
    let _guard = null_test_lock().lock().unwrap();
    clear_bind();
    clear_nulls();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tx_claim_pkg.json");
    let keys = make_keys();
    let mut packages = vec![make_pkg_portable(15, &keys)];
    patch_bundle(&mut packages);
    let null_hex = packages[0].tx.context.nullifier_hex.clone();

    write_claim_bundle(dir.path(), packages).unwrap();
    bind_null_dir(dir.path());
    let before = get_null(&null_hex).expect("reserved row");
    assert_eq!(before.status, NullifierStatus::Reserved);

    let mut store = SettlementStore::new();
    let report = publish_claims_store(&path, &mut store).unwrap();
    assert_eq!(report.inserted_count, 1);

    assert!(get_null(&null_hex).is_none());
    let after = store
        .settlement_claim_null_rec(&store_null(&null_hex))
        .unwrap()
        .expect("storage spent row");
    assert_eq!(after.status, ClaimNullStatus::Spent);
}

#[test]
fn test_nullifier_survives_asset_delete() {
    let _guard = null_test_lock().lock().unwrap();
    clear_bind();
    clear_nulls();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tx_claim_pkg.json");
    let keys = make_keys();
    let mut packages = vec![make_pkg_portable(16, &keys)];
    patch_bundle(&mut packages);

    write_claim_bundle(dir.path(), packages.clone()).unwrap();

    let mut store = SettlementStore::new();
    publish_claims_store(&path, &mut store).unwrap();

    let asset_wire = packages[0].tx.outputs[0].asset_wire.as_ref().unwrap();
    let wire = asset_wire.clone().to_wire().unwrap();
    let leaf = asset_wire_to_leaf(&wire).unwrap();
    let item_path = SettlementPath::new(
        DefinitionId::new(wire.definition.id),
        SerialId::new(asset_wire.serial_id),
        TerminalId::new(leaf.asset_id),
    );
    store.del_settlement_item(&item_path).unwrap();

    let entry = store
        .settlement_claim_null_rec(&store_null(&packages[0].tx.context.nullifier_hex))
        .unwrap()
        .expect("storage nullifier entry");
    assert_eq!(entry.status, ClaimNullStatus::Spent);
}
