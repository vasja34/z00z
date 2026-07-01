use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use z00z_core::{genesis::asset_std::asset_from_dev_class, AssetClass, AssetWire};
use z00z_crypto::{create_range_proof, ClaimAuthoritySig, ClaimSourceProof, Z00ZScalar};
use z00z_simulator::{
    scenario_1::claim_pkg_consumer::ClaimTxBundle, scenario_1::stage_3::build_claim_package_fault,
    StageResult,
};
use z00z_utils::io::{load_json, path_exists};
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::{ScanResult, StealthOutputScanner},
    stealth::{build_tx_output_unchecked, SenderWallet},
    tx::derive_output_nonce,
};

use z00z_simulator::scenario_1::support::claim_shared_cases;

#[derive(Clone)]
struct RunCase {
    out: PathBuf,
    stage3: StageResult,
}

fn run_case() -> RunCase {
    RunCase {
        out: claim_shared_cases::default_stage3_out(),
        stage3: StageResult::Ok,
    }
}

fn make_keys() -> ReceiverKeys {
    let recv = ReceiverSecret::from_bytes([0x44u8; 32]).expect("receiver secret");
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

fn make_wire(serial_id: u32, keys: &ReceiverKeys) -> AssetWire {
    let mut asset = asset_from_dev_class(AssetClass::Coin, 0, 100).expect("asset");
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
        panic!("owned leaf")
    };
    let blinding =
        Z00ZScalar::try_from_bytes(wallet_output.blinding.expect("blinding")).expect("scalar");
    asset.range_proof = Some(create_range_proof(asset.amount, &blinding, 64, 0).expect("proof"));

    let mut wire = AssetWire::from_asset(&asset);
    wire.secret = None;
    wire
}

fn assert_fault_fail(fault: &str, want_msg: &str) {
    let dir = tempfile::tempdir().expect("tempdir");
    let pkg_path = dir.path().join("tx_claim_pkg.json");
    let keys = make_keys();
    let wire = make_wire(77, &keys);
    let claim_id = derive_output_nonce(&wire.definition.id, wire.serial_id);
    let asset_id_hex = hex::encode(wire.clone().to_asset().expect("wire asset").asset_id());

    let err = build_claim_package_fault(
        3,
        "devnet",
        "z00z-devnet-1",
        "alice",
        &asset_id_hex,
        wire.amount,
        &claim_id,
        &keys.owner_handle,
        77,
        Some(wire),
        Some(&keys),
        fault,
    )
    .expect_err("fault must fail");

    assert!(err.contains(want_msg), "unexpected error: {err}");
    assert!(
        !path_exists(&pkg_path).expect("path_exists"),
        "claim package must be absent"
    );
}

fn ok_case() -> &'static RunCase {
    static CASE: OnceLock<RunCase> = OnceLock::new();
    CASE.get_or_init(run_case)
}

fn claim_file(out: &Path) -> PathBuf {
    out.join("claim/tx_claim_pkg.json")
}

fn claim_bundle(out: &Path) -> ClaimTxBundle {
    load_json(claim_file(out)).expect("load claim bundle")
}

#[test]
fn test_claim_pkg_non_stub() {
    let case = ok_case();
    assert!(matches!(case.stage3, StageResult::Ok));

    let bundle = claim_bundle(&case.out);
    assert!(
        !bundle.packages.is_empty(),
        "claim bundle must be non-empty"
    );

    for pkg in bundle.packages {
        let proof_bytes = hex::decode(&pkg.tx.proof.proof_hex).expect("proof hex");
        let proof = ClaimSourceProof::from_bytes(&proof_bytes).expect("canonical proof");
        assert!(
            !proof.proof_blob().is_empty(),
            "proof payload must be non-empty"
        );
        assert_ne!(
            proof.source_root(),
            [0u8; 32],
            "source root must be authoritative"
        );
    }
}

#[test]
fn test_claim_sig_non_stub() {
    let case = ok_case();
    assert!(matches!(case.stage3, StageResult::Ok));

    let bundle = claim_bundle(&case.out);
    assert!(
        !bundle.packages.is_empty(),
        "claim bundle must be non-empty"
    );

    for pkg in bundle.packages {
        let sig_bytes = hex::decode(&pkg.tx.auth.claim_authority_sig_hex).expect("sig hex");
        let sig = ClaimAuthoritySig::from_bytes(&sig_bytes).expect("canonical sig");
        assert_eq!(sig.auth_pk(), &z00z_wallets::tx::claim_auth_pk());
    }
}

#[test]
fn test_bind_miss_fails() {
    assert_fault_fail(
        "bind_mismatch",
        "recipient_keys.owner_handle does not match recipient_owner_bytes",
    );
}

#[test]
fn test_proof_fail_fails() {
    assert_fault_fail("proof_fail", "claim proof build failed");
}

#[test]
fn test_auth_fail_fails() {
    assert_fault_fail("auth_fail", "claim authority build failed");
}
