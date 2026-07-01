use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tempfile::TempDir;
use z00z_core::{genesis::asset_std::asset_from_dev_class, AssetClass, AssetWire};
use z00z_crypto::{create_range_proof, poseidon2_hash, Hidden, Z00ZScalar};
use z00z_simulator::{
    scenario_1::claim_pkg_consumer::{wrap_claim_packages, ClaimTxBundle},
    scenario_1::{
        stage_2,
        stage_3::{
            build_claim_package, patch_claim_bundle_membership, write_claim_bundle,
            write_claim_bundle_store, CLAIM_STORE_FILE,
        },
        stage_5, stage_6,
    },
    SimActor, Stage2ActorCfg, StageResult,
};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{create_dir_all, read_file, save_json, write_file},
};
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::{ScanResult, StealthOutputScanner},
    stealth::{build_tx_output_unchecked, SenderWallet},
    tx::{build_claim_tx_digest, derive_output_nonce, ClaimTxPackage},
    wallet::{
        ChainId, WalletId, WalletKernel, WalletRecord, WalletSystemMetadata, WalletUserFields,
    },
};

use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::scenario_support;
use z00z_simulator::scenario_1::support::stage_runner_support;

use crate::stage4_paths::assert_absent;
use crate::stage4_root::set_s4_root;
use scenario_support::make_cfg_in;

const CHAIN_ID: u32 = 3;
const CHAIN_TYPE: &str = "devnet";
const CHAIN_NAME: &str = "z00z-devnet-1";

fn claim_file(root: &Path) -> PathBuf {
    root.join("claim/tx_claim_pkg.json")
}

fn claim_file_in(root: &Path, claim_dir: &str) -> PathBuf {
    root.join(claim_dir).join("tx_claim_pkg.json")
}

fn tx_file(root: &Path) -> PathBuf {
    root.join("transactions/tx_alice_to_bob_pkg.json")
}

fn before_file(root: &Path) -> PathBuf {
    root.join("transactions/wallets_state_before.json")
}

fn after_file(root: &Path) -> PathBuf {
    root.join("transactions/wallets_state_after.json")
}

fn diff_file(root: &Path) -> PathBuf {
    root.join("transactions/wallets_state_diff.json")
}

fn pending_file(root: &Path) -> PathBuf {
    root.join("transactions/wallets_pending.json")
}

fn confirm_file(root: &Path) -> PathBuf {
    root.join("transactions/wallets_confirmed.json")
}

fn report_md(root: &Path) -> PathBuf {
    root.join("transactions/wallets_state_report.md")
}

fn report_xlsx(root: &Path) -> PathBuf {
    root.join("transactions/wallets_state_report.xlsx")
}

fn claim_source_root(gate_root: &Path) -> PathBuf {
    gate_root
        .parent()
        .expect("gate root parent")
        .join("outputs/scenario_1")
}

fn assert_no_state(root: &Path) {
    assert_absent(&tx_file(root));
    assert_absent(&before_file(root));
    assert_absent(&after_file(root));
    assert_absent(&diff_file(root));
    assert_absent(&pending_file(root));
    assert_absent(&confirm_file(root));
    assert_absent(&report_md(root));
    assert_absent(&report_xlsx(root));
}

fn has_miss_ctx(msg: &str) -> bool {
    msg.contains("claim package prerequisite failed") && msg.contains("tx_claim_pkg.json")
}

fn has_bad_ctx(msg: &str) -> bool {
    msg.contains("claim package prerequisite failed")
        && (msg.contains("claim package")
            || msg.contains("claim pkg")
            || msg.contains("decode failed")
            || msg.contains("failed:"))
}

fn has_verify_ctx(msg: &str) -> bool {
    msg.contains("claim package prerequisite failed")
        && (msg.contains("claim_proof_invalid") || msg.contains("failed:"))
}

fn synth_receiver_secret(spec: &Stage2ActorCfg) -> [u8; 32] {
    const MAX_RETRY: u32 = 16;

    let mut retry = 0u32;
    let seed = spec.mock_rng_seed.to_le_bytes();
    let mut bytes = poseidon2_hash(
        b"z00z.sim.stage4.claim_gate.receiver_secret.v1",
        &[spec.name.as_bytes(), &seed],
    );

    loop {
        match ReceiverSecret::from_bytes(bytes) {
            Ok(secret) => return *secret.as_bytes(),
            Err(
                z00z_wallets::key::StealthKeyError::ZeroSecret
                | z00z_wallets::key::StealthKeyError::InvalidSecretKey
                | z00z_wallets::key::StealthKeyError::ZeroScalarRejected
                | z00z_wallets::key::StealthKeyError::IdentityPointRejected,
            ) if retry < MAX_RETRY => {
                retry += 1;
                let step = retry.to_le_bytes();
                bytes = poseidon2_hash(
                    b"z00z.sim.stage4.claim_gate.receiver_secret.retry.v1",
                    &[spec.name.as_bytes(), &seed, &step, &bytes],
                );
            }
            Err(err) => panic!("synthetic receiver secret for {} failed: {err}", spec.name),
        }
    }
}

fn synth_wallet_id_bytes(spec: &Stage2ActorCfg) -> [u8; 32] {
    let seed = spec.mock_rng_seed.to_le_bytes();
    poseidon2_hash(
        b"z00z.sim.stage4.claim_gate.wallet_id.v1",
        &[spec.name.as_bytes(), &seed],
    )
}

fn synth_wallet_id(spec: &Stage2ActorCfg) -> String {
    format!("wallet_{}", hex::encode(synth_wallet_id_bytes(spec)))
}

fn rebuild_actor(spec: &Stage2ActorCfg) -> SimActor {
    let secret_bytes = synth_receiver_secret(spec);
    let receiver_secret = ReceiverSecret::from_bytes(secret_bytes).expect("receiver secret");
    let keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("receiver keys");
    let card = keys.export_receiver_card().expect("export receiver card");
    let wallet_id = synth_wallet_id_bytes(spec);
    let record = WalletRecord::new(
        WalletKernel::new(WalletId(wallet_id), ChainId::DEVNET),
        WalletUserFields {
            wallet_name: spec.name.clone(),
            memo: None,
        },
        WalletSystemMetadata {
            created_at: 0,
            updated_at: 0,
        },
    );

    SimActor {
        name: spec.name.clone(),
        password: Some(spec.password.clone()),
        wallet_id: synth_wallet_id(spec),
        record,
        keys,
        card,
        balance: HashMap::new(),
        receiver_secret: Hidden::hide(secret_bytes),
        session: None,
    }
}

fn claim_gate_ctx(cfg_path: &Path) -> stage_runner_support::StageSession {
    let mut ctx = stage_runner_support::resume_stage_session(cfg_path);
    let actors = ctx
        .ctx
        .config
        .stage2_wallet_create
        .as_ref()
        .expect("stage2 wallet config")
        .actors
        .clone();
    stage_2::set_actor_passwords_for_test(&actors);
    ctx.ctx.actors = actors.iter().map(rebuild_actor).collect();
    ctx
}

fn cached_stage6_fail(case_name: &str, seed_gate: impl FnOnce(&Path)) -> (PathBuf, String) {
    let root = fixture_cache::ensure_shared_case(case_name, |base| {
        let gate_root = base.join("claim_root");
        create_dir_all(&gate_root).expect("gate root dir");
        let (cfg_path, design_path, out) = make_cfg_in(base, |cfg| set_s4_root(cfg, &gate_root));
        // Stage 4 now consumes the canonical Stage 3 claim bundle path from the
        // scenario output root, while all Stage 4 artifacts still land under
        // the isolated gate root configured above.
        seed_gate(&out);
        let mut ctx = claim_gate_ctx(&cfg_path);
        let stage5 = stage_runner_support::stage_by_id(&design_path, 5);
        let stage6 = stage_runner_support::stage_by_id(&design_path, 6);

        assert!(
            matches!(stage_5::run_tx_plan(&mut ctx, &stage5), StageResult::Ok),
            "stage 5 must stay green for cached claim gate baseline",
        );

        let msg = match stage_6::run_tx_prepare(&mut ctx, &stage6) {
            StageResult::Fail(msg) => msg,
            other => panic!("stage 6 must fail, got {other:?}"),
        };
        save_json(
            base.join("stage6_meta.json"),
            &serde_json::json!({ "stage6_msg": msg }),
        )
        .expect("write cached stage6 meta");
    });

    let meta: serde_json::Value = serde_json::from_slice(
        &read_file(root.join("stage6_meta.json")).expect("read cached stage6 meta"),
    )
    .expect("decode cached stage6 meta");
    let msg = meta["stage6_msg"]
        .as_str()
        .expect("stage6 cached message")
        .to_string();
    (root.join("claim_root"), msg)
}

fn make_keys() -> ReceiverKeys {
    let recv = ReceiverSecret::from_bytes([0x33u8; 32]).expect("receiver secret");
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

fn make_bad_pkg() -> ClaimTxPackage {
    let mut pkg = make_ok_pkg();
    pkg.tx.proof.proof_hex = "ZZnothex".to_string();
    pkg.tx_digest_hex = build_claim_tx_digest(
        &pkg.kind,
        &pkg.package_type,
        pkg.version,
        pkg.chain_id,
        &pkg.chain_type,
        &pkg.chain_name,
        &pkg.tx,
    )
    .expect("digest");
    pkg
}

fn patch_packages(packages: &mut [ClaimTxPackage]) {
    patch_claim_bundle_membership(packages).expect("patch bundle membership");
}

fn make_ok_pkg() -> ClaimTxPackage {
    make_ok_pkg_with_serial(31)
}

fn make_ok_pkg_with_serial(serial_id: u32) -> ClaimTxPackage {
    let keys = make_keys();
    let wire = make_wire(serial_id, &keys);
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
        Some(&keys),
    )
    .expect("claim pkg");
    let mut packages = vec![JsonCodec.deserialize(&pkg_bytes).expect("decode claim pkg")];
    patch_packages(&mut packages);
    packages.pop().expect("single claim pkg")
}

#[test]
fn test_stage4_temp_gate_ok() {
    // Temporary contract: Stage-4 accepts only after a Stage-3-valid claim bundle exists.
    let (_root, msg) = cached_stage6_fail("stage4_claim_gate_temp_ok_v1", |root| {
        let claim_dir = root.join("claim");
        create_dir_all(&claim_dir).expect("claim dir");
        let mut packages = vec![make_ok_pkg()];
        patch_packages(&mut packages);
        write_claim_bundle(&claim_dir, packages).expect("write canonical claim bundle");
    });

    assert!(
        !msg.contains("claim package prerequisite failed"),
        "claim gate must already be passed: {msg}"
    );
    assert!(
        msg.contains("stage 6 (tx_prepare) failed:"),
        "unexpected downstream stage6 error surface: {msg}"
    );
}

#[test]
fn test_uses_stage3_claim_dir() {
    let base = TempDir::new().expect("temp dir").keep();
    let root = base.join("claim_root");
    create_dir_all(&root).expect("claim root dir");

    let claim_dir_name = "claim_custom";
    let (cfg_path, design_path, out) = make_cfg_in(&base, |cfg| {
        set_s4_root(cfg, &root);
        cfg.stage3_claim
            .as_mut()
            .expect("stage3 cfg")
            .paths
            .claim_dir = claim_dir_name.to_string();
    });
    let claim_dir = out.join(claim_dir_name);
    create_dir_all(&claim_dir).expect("claim dir");
    let mut packages = vec![make_ok_pkg()];
    patch_packages(&mut packages);
    write_claim_bundle(&claim_dir, packages).expect("write canonical claim bundle");
    let mut ctx = claim_gate_ctx(&cfg_path);
    let stage5 = stage_runner_support::stage_by_id(&design_path, 5);
    let stage6 = stage_runner_support::stage_by_id(&design_path, 6);

    assert!(
        matches!(stage_5::run_tx_plan(&mut ctx, &stage5), StageResult::Ok),
        "stage 5 must stay green for configured claim-dir baseline",
    );

    let msg = match stage_6::run_tx_prepare(&mut ctx, &stage6) {
        StageResult::Fail(msg) => msg,
        other => panic!("stage 6 must fail after claim gate, got {other:?}"),
    };

    assert!(
        !msg.contains("claim package prerequisite failed"),
        "stage 6 must consume the configured stage3 claim directory: {msg}"
    );
    assert!(
        msg.contains("stage 6 (tx_prepare) failed:"),
        "stage 6 must still report a downstream tx_prepare failure surface: {msg}"
    );
    assert!(
        claim_file_in(&out, claim_dir_name).exists(),
        "configured stage3 claim bundle must stay at the configured path"
    );
}

#[test]
fn test_stage4_rejects_missing_package() {
    let (root, msg) = cached_stage6_fail("stage4_claim_gate_missing_package_v1", |_| {});
    assert!(has_miss_ctx(&msg), "unexpected error: {msg}");

    assert_no_state(&root);
}

#[test]
fn test_stage4_rejects_claim_store() {
    let (root, msg) = cached_stage6_fail("stage4_claim_gate_missing_store_v1", |root| {
        let claim_dir = root.join("claim");
        create_dir_all(&claim_dir).expect("claim dir");

        let mut packages = vec![make_ok_pkg()];
        patch_packages(&mut packages);
        save_json(claim_file(root), &wrap_claim_packages(packages)).expect("write claim pkg");

        let missing_store = claim_dir.join(CLAIM_STORE_FILE);
        assert_absent(&missing_store);
    });
    assert!(has_bad_ctx(&msg), "unexpected error: {msg}");
    assert!(
        msg.contains("persisted claim membership store missing"),
        "missing persisted-store context: {msg}"
    );

    assert_no_state(&root);
}

#[test]
fn test_stage4_rejects_persisted_store() {
    let (root, msg) = cached_stage6_fail("stage4_claim_gate_persisted_store_v1", |root| {
        let claim_dir = root.join("claim");
        create_dir_all(&claim_dir).expect("claim dir");

        let auth_packages = vec![make_ok_pkg_with_serial(31), make_ok_pkg_with_serial(32)];
        write_claim_bundle_store(&claim_dir, &auth_packages).expect("write claim store");
        save_json(
            claim_file(root),
            &wrap_claim_packages(vec![auth_packages[0].clone()]),
        )
        .expect("write trimmed claim pkg");
    });
    assert!(has_bad_ctx(&msg), "unexpected error: {msg}");
    assert!(
        msg.contains("persisted claim store membership mismatch"),
        "missing bundle/store mismatch context: {msg}"
    );

    assert_no_state(&root);
}

#[test]
fn test_stage4_rejects_claim_bundle() {
    let (_root, msg) = cached_stage6_fail("stage4_claim_gate_empty_bundle_v1", |root| {
        let claim_dir = root.join("claim");
        create_dir_all(&claim_dir).expect("claim dir");
        save_json(
            claim_file(root),
            &wrap_claim_packages(Vec::<ClaimTxPackage>::new()),
        )
        .expect("write empty claim bundle");
    });
    assert!(has_bad_ctx(&msg), "unexpected error: {msg}");
    assert!(
        msg.contains("must carry at least one package"),
        "missing empty-bundle context: {msg}"
    );
}

#[test]
fn test_write_bundle_empty_bundle() {
    let gate = TempDir::new().expect("temp dir");
    let root = gate.keep();
    let claim_dir = root.join("claim");
    create_dir_all(&claim_dir).expect("claim dir");

    let err = write_claim_bundle(&claim_dir, Vec::<ClaimTxPackage>::new())
        .expect_err("empty bundle must be rejected");
    assert!(
        err.contains("must carry at least one package"),
        "unexpected error: {err}"
    );
    assert_absent(&claim_file(&root));
}

#[test]
fn test_patch_membership_empty_bundle() {
    let err = patch_claim_bundle_membership(&mut [])
        .expect_err("empty bundle must be rejected during canonicalization");
    assert!(
        err.contains("must carry at least one package"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_stage4_rejects_bad_package() {
    let (root, msg) = cached_stage6_fail("stage4_claim_gate_bad_package_v1", |root| {
        create_dir_all(root.join("claim")).expect("claim dir");
        write_file(claim_file(root), b"not-json").expect("write claim file");
    });
    assert!(has_bad_ctx(&msg), "unexpected error: {msg}");

    assert_no_state(&root);
}

#[test]
fn test_stage4_bad_pkg_fails() {
    let (root, msg) = cached_stage6_fail("stage4_claim_gate_bad_pkg_v1", |root| {
        create_dir_all(root.join("claim")).expect("claim dir");

        let packages = vec![make_bad_pkg()];
        save_json(claim_file(root), &wrap_claim_packages(packages)).expect("write claim pkg");
    });
    assert!(has_verify_ctx(&msg), "unexpected error: {msg}");

    let source_root = claim_source_root(&root);
    let decoded: ClaimTxBundle = JsonCodec
        .deserialize(&read_file(claim_file(&source_root)).expect("read claim file"))
        .expect("decode written claim file");
    assert_eq!(decoded.kind, "TxPackageBundle");
    assert_eq!(decoded.package_type, "claim_tx");
    assert_eq!(decoded.packages.len(), 1);
    assert_no_state(&root);
}
