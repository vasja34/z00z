use std::sync::{Arc, Mutex, OnceLock};

use z00z_core::{genesis::asset_std::asset_from_dev_class, Asset, AssetClass, AssetWire};
use z00z_simulator::scenario_1::stage_3::{
    build_claim_package, patch_claim_bundle_membership, to_claim_wire, write_claim_bundle,
};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::write_file,
};
use z00z_wallets::{
    claim::{
        bind_paths, clear_bind, clear_rows, get_entry, global_nullifier_store, NullifierStateStore,
        NullifierStatus,
    },
    key::{ReceiverKeys, ReceiverSecret},
    tx::{derive_output_nonce, ClaimTxPackage},
};

const CHAIN_ID: u32 = 3;
const CHAIN_TYPE: &str = "devnet";
const CHAIN_NAME: &str = "z00z-devnet-1";

fn test_lock() -> &'static Mutex<()> {
    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_LOCK.get_or_init(|| Mutex::new(()))
}

fn bind_dir(dir: &std::path::Path) {
    let row_path = dir.join("nullifier_rows.json");
    let audit_path = dir.join("nullifier_audit.json");
    bind_paths(&row_path, Some(&audit_path)).expect("bind nullifier files");
}

fn make_keys() -> ReceiverKeys {
    let recv = ReceiverSecret::from_bytes([0x55u8; 32]).expect("receiver secret");
    ReceiverKeys::from_receiver_secret(recv).expect("receiver keys")
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

fn make_asset(serial_id: u32) -> Asset {
    let mut asset = asset_from_dev_class(AssetClass::Coin, 0, 10).expect("asset");
    let def = rebuild_def(asset.definition.as_ref(), serial_id);
    asset.definition = Arc::new(def);
    asset.serial_id = serial_id;

    asset
}

fn make_wire(serial_id: u32, keys: &ReceiverKeys) -> AssetWire {
    let asset = make_asset(serial_id);
    let card = keys.export_receiver_card().expect("card");
    let tx_seed = derive_output_nonce(&asset.definition.id, asset.serial_id);
    to_claim_wire(&asset, keys, &card, &tx_seed).expect("wire")
}

fn make_pkg(serial_id: u32, keys: &ReceiverKeys) -> ClaimTxPackage {
    let wire = make_wire(serial_id, keys);
    let claim_id = derive_output_nonce(&wire.definition.id, wire.serial_id);
    let asset_id_hex = hex::encode(wire.clone().to_asset().expect("claim asset").asset_id());
    let pkg_bytes = build_claim_package(
        CHAIN_ID,
        CHAIN_TYPE,
        CHAIN_NAME,
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
    JsonCodec.deserialize(&pkg_bytes).expect("decode claim pkg")
}

fn patch_packages(packages: &mut [ClaimTxPackage]) {
    patch_claim_bundle_membership(packages).expect("patch bundle membership");
}

#[test]
fn test_restart_replay_path() {
    let _guard = test_lock().lock().expect("test lock");
    clear_bind();
    clear_rows();

    let temp = tempfile::tempdir().expect("tempdir");
    let keys = make_keys();
    let mut packages = vec![make_pkg(11, &keys)];
    patch_packages(&mut packages);
    let null_hex = packages[0].tx.context.nullifier_hex.clone();

    write_claim_bundle(temp.path(), packages.clone()).expect("first write");
    bind_dir(temp.path());
    let first = get_entry(&null_hex).expect("reserved row");
    assert_eq!(first.status, NullifierStatus::Reserved);

    clear_bind();
    bind_dir(temp.path());
    let err = write_claim_bundle(temp.path(), packages).expect_err("replay must fail");
    assert!(err.contains("nullifier replay rejected"));
}

#[test]
fn test_corrupt_rows_closed() {
    let _guard = test_lock().lock().expect("test lock");
    clear_bind();
    clear_rows();

    let temp = tempfile::tempdir().expect("tempdir");
    let row_path = temp.path().join("nullifier_rows.json");
    let audit_path = temp.path().join("nullifier_audit.json");
    bind_paths(&row_path, Some(&audit_path)).expect("bind paths");
    write_file(&row_path, b"{bad json").expect("write corrupt rows");

    let err = global_nullifier_store()
        .get_status(&"cc".repeat(32))
        .expect_err("corrupt rows must fail closed");
    match err {
        z00z_wallets::claim::NullReserveErr::Corrupt(msg) => {
            assert!(msg.contains("nullifier row load failed"));
        }
        other => panic!("unexpected status result: {other:?}"),
    }
}
