use std::{path::Path, sync::OnceLock};

use z00z_simulator::config::ScenarioCfg;
use z00z_utils::io::load_json;
use z00z_wallets::tx::{
    build_tx_package_digest, verify_full_tx_package, verify_tx_public_spend_contract,
    SpendPublicErr, TxPackage, TxVerifier, TxVerifierImpl,
};

use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::scenario_support;
use z00z_simulator::scenario_1::support::stage_runner_support;

use scenario_support::make_cfg_in;

fn fixed_s4(cfg: &mut ScenarioCfg) {
    cfg.simulation.use_mock_rng = true;
    cfg.simulation.mock_rng_seed = Some(42);

    if let Some(stage3) = cfg.stage3_claim.as_mut() {
        stage3.rng_seed = Some(42);
    }

    let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_min = 4;
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_target = 4;
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_max = 4;
    stage4.transaction.outputs.bob_outputs_count = 4;
    stage4.transaction.class = "Coin".to_string();
    stage4.transaction.symbol = "Z00Z".to_string();
    stage4.transaction.mode = "fraction".to_string();
    stage4.transaction.fraction = Some(0.1);
    stage4.transaction.amount = None;
}

fn pkg_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/tx_alice_to_bob_pkg.json")
}

fn base_stage4_pkg() -> &'static TxPackage {
    static PKG: OnceLock<TxPackage> = OnceLock::new();
    PKG.get_or_init(|| {
        let root = fixture_cache::ensure_case("scenario1_spend_gate_v1", |base| {
            let (cfg_path, design_path, out) = make_cfg_in(base, |cfg| {
                fixed_s4(cfg);
            });
            let _ctx = stage_runner_support::run_stage_setup(
                &cfg_path,
                &design_path,
                &[1_u32, 2, 3, 4, 5, 6],
            );
            assert!(pkg_file(&out).exists());
        });
        let out = root.join("outputs/scenario_1");
        load_json(pkg_file(&out)).expect("tx package")
    })
}

fn run_stage4_pkg() -> TxPackage {
    base_stage4_pkg().clone()
}

#[test]
fn test_scenario1_gate_wallet_verifier() {
    let pkg = run_stage4_pkg();

    assert!(
        pkg.tx.proof.spend.is_some(),
        "stage4 tx must persist spend proof"
    );
    assert!(
        pkg.tx.auth.spend.is_some(),
        "stage4 tx must persist spend auth"
    );
    assert!(
        !pkg.tx.proof.spend.as_ref().expect("spend proof").inputs[0]
            .nullifier_hex
            .is_empty(),
        "stage4 tx must persist deterministic spend nullifier semantics"
    );
    verify_tx_public_spend_contract(pkg.chain_id, 1, &pkg.chain_type, &pkg.chain_name, &pkg.tx)
        .expect("scenario1 spend contract");
}

#[test]
fn test_scenario1_local_shortcut_rejects() {
    let pkg = run_stage4_pkg();
    let verifier = TxVerifierImpl::new();

    let local = verifier
        .verify(&serde_json::to_vec(&pkg).expect("package bytes"))
        .expect("local verifier must run");
    assert!(local.valid, "stage4 package must pass local wire checks");

    let mut shortcut = pkg.clone();
    shortcut.tx.proof.spend = None;
    shortcut.tx_digest_hex = build_tx_package_digest(
        &shortcut.kind,
        &shortcut.package_type,
        shortcut.version,
        shortcut.chain_id,
        &shortcut.chain_type,
        &shortcut.chain_name,
        &shortcut.tx,
    )
    .expect("recompute digest after proof removal");
    let shortcut_bytes = serde_json::to_vec(&shortcut).expect("shortcut bytes");

    let local_shortcut = verifier
        .verify(&shortcut_bytes)
        .expect("local verifier must run on shortcut payload");
    assert!(
        local_shortcut.valid,
        "local wire checks must still pass before the public spend boundary"
    );

    let full = verify_full_tx_package(&shortcut_bytes).expect("full verifier must run");
    assert!(!full.valid, "public spend gap must block full acceptance");
    assert!(
        full.errors
            .iter()
            .any(|err| err.contains("public spend contract failed: missing spend proof")),
        "unexpected full-verifier errors: {:?}",
        full.errors
    );
}

#[test]
fn test_scenario1_rejects_nullifier_hex() {
    let mut pkg = run_stage4_pkg();
    pkg.tx.proof.spend.as_mut().expect("spend proof").inputs[0].nullifier_hex = "zz".to_string();

    let err =
        verify_tx_public_spend_contract(pkg.chain_id, 1, &pkg.chain_type, &pkg.chain_name, &pkg.tx)
            .expect_err("malformed nullifier hex must reject scenario1 spend contract");

    assert_eq!(
        err,
        SpendPublicErr::InvalidHex {
            label: "proof.inputs[].nullifier_hex"
        }
    );
}

#[test]
fn test_scenario1_public_nullifier_value() {
    let mut pkg = run_stage4_pkg();
    pkg.tx.proof.spend.as_mut().expect("spend proof").inputs[0]
        .nullifier_hex
        .clear();

    let err =
        verify_tx_public_spend_contract(pkg.chain_id, 1, &pkg.chain_type, &pkg.chain_name, &pkg.tx)
            .expect_err("missing nullifier value must reject scenario1 spend contract");

    assert_eq!(
        err,
        SpendPublicErr::InvalidHex {
            label: "proof.inputs[].nullifier_hex"
        }
    );
}

#[test]
fn test_scenario1_rejects_nullifier_drift() {
    let mut pkg = run_stage4_pkg();
    pkg.tx.proof.spend.as_mut().expect("spend proof").inputs[0].nullifier_hex =
        hex::encode([0xAB; 32]);

    let err =
        verify_tx_public_spend_contract(pkg.chain_id, 1, &pkg.chain_type, &pkg.chain_name, &pkg.tx)
            .expect_err("post-signature nullifier drift must reject scenario1 spend contract");

    assert_eq!(err, SpendPublicErr::StatementMismatch);
}

#[test]
fn test_scenario1_rejects_auth_gap() {
    let mut pkg = run_stage4_pkg();
    pkg.tx.auth.spend = None;

    let err =
        verify_tx_public_spend_contract(pkg.chain_id, 1, &pkg.chain_type, &pkg.chain_name, &pkg.tx)
            .expect_err("missing auth must reject placeholder acceptance");

    assert_eq!(err, SpendPublicErr::MissingAuth);
}

// These tests lock the fail-closed public spend contract the verifier actually
// enforces today without widening the shipped boundary into a full-ZK theorem.

#[test]
fn test_scenario1_gate_missing_proof() {
    let mut pkg = run_stage4_pkg();
    pkg.tx.proof.spend = None;

    let err =
        verify_tx_public_spend_contract(pkg.chain_id, 1, &pkg.chain_type, &pkg.chain_name, &pkg.tx)
            .expect_err("missing proof must reject current public spend contract");

    assert_eq!(err, SpendPublicErr::MissingProof);
}

#[test]
fn test_scenario1_rejects_proof_version() {
    let mut pkg = run_stage4_pkg();
    pkg.tx.proof.spend.as_mut().expect("stage4 spend proof").ver = u8::MAX;

    let err =
        verify_tx_public_spend_contract(pkg.chain_id, 1, &pkg.chain_type, &pkg.chain_name, &pkg.tx)
            .expect_err("bad proof version must reject current public spend contract");

    assert_eq!(err, SpendPublicErr::BadProofVersion);
}

#[test]
fn test_scenario1_rejects_prev_root() {
    let mut pkg = run_stage4_pkg();
    pkg.tx
        .proof
        .spend
        .as_mut()
        .expect("stage4 spend proof")
        .prev_root_hex = hex::encode([0u8; 32]);

    let err =
        verify_tx_public_spend_contract(pkg.chain_id, 1, &pkg.chain_type, &pkg.chain_name, &pkg.tx)
            .expect_err("zero prev_root must reject current public spend contract");

    assert_eq!(err, SpendPublicErr::BadPrevRoot);
}

#[test]
fn test_scenario1_rejects_id_drift() {
    let mut pkg = run_stage4_pkg();
    pkg.chain_id = 77;

    let err =
        verify_tx_public_spend_contract(pkg.chain_id, 1, &pkg.chain_type, &pkg.chain_name, &pkg.tx)
            .expect_err("package chain_id drift must reject scenario1 spend contract");

    assert_eq!(err, SpendPublicErr::StatementMismatch);
}
