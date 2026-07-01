use std::{
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use z00z_crypto::expert::encoding::SafePassword;
use z00z_simulator::{
    scenario_1::{stage_11, stage_9},
    StageResult,
};
use z00z_utils::io::read_file;
use z00z_wallets::{key::ReceiverKeys, rpc::types::common::PersistWalletId, WalletService};

use z00z_simulator::scenario_1::stage_11::jmt_wallet_scan;
use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::stage_runner_support;

use stage_runner_support::{make_cfg_in, stage_by_id};

const BOB_PASS: &str = "Bob_Pass_Z00Z_43!";

struct RunCase {
    out: PathBuf,
    genesis_rights_len: usize,
}

fn good_cfg(cfg: &mut z00z_simulator::config::ScenarioCfg) {
    cfg.simulation.use_mock_rng = true;
    cfg.simulation.mock_rng_seed = Some(42);
    cfg.genesis_assets.truncate(1);
    for asset in &mut cfg.genesis_assets {
        asset.serials = asset.serials.min(4);
        asset.nominal = asset.nominal.max(50_000);
    }
    if let Some(stage3) = cfg.stage3_claim.as_mut() {
        stage3.rng_seed = Some(42);
    }
    let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_min = 3;
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_target = 3;
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_max = 3;
    stage4.transaction.outputs.bob_outputs_count = 3;
    stage4.transaction.class = "Coin".to_string();
    stage4.transaction.symbol = "Z00Z".to_string();
    stage4.transaction.mode = "fraction".to_string();
    stage4.transaction.fraction = Some(0.1);
    stage4.transaction.amount = None;
}

fn shared_stage11_root() -> &'static PathBuf {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        fixture_cache::ensure_shared_case("stage7_jmt_wallet_scan_stage11_shared_v4", |base| {
            let (cfg_path, design_path, out) = make_cfg_in(base, good_cfg);
            let mut ctx = stage_runner_support::run_stage4_session(&cfg_path, &design_path);

            for stage_id in [9_u32, 11] {
                let stage = stage_by_id(&design_path, stage_id);
                let res = match stage_id {
                    9 => stage_9::run_bundle_build(&mut ctx, &stage),
                    11 => stage_11::run_apply(&mut ctx, &stage),
                    _ => unreachable!(),
                };
                assert!(
                    matches!(res, StageResult::Ok),
                    "stage {stage_id} must succeed: {res:?}"
                );
            }

            z00z_utils::io::write_file(
                base.join("jmt_scan_meta.json"),
                serde_json::to_vec_pretty(&serde_json::json!({
                    "genesis_rights_len": ctx.genesis_rights.len()
                }))
                .expect("encode jmt scan meta")
                .as_slice(),
            )
            .expect("write jmt scan meta");
            assert!(
                out.join("wallet_scan.json").exists(),
                "shared stage11 fixture must export charlie scan"
            );
        })
    })
}

fn stage11_case(case_name: &str) -> RunCase {
    let root = fixture_cache::ensure_case(case_name, |base| {
        fixture_cache::copy_tree(shared_stage11_root(), base);
    });

    let meta: serde_json::Value = serde_json::from_slice(
        &read_file(root.join("jmt_scan_meta.json")).expect("read jmt scan meta"),
    )
    .expect("decode jmt scan meta");

    RunCase {
        out: root.join("outputs/scenario_1"),
        genesis_rights_len: meta["genesis_rights_len"]
            .as_u64()
            .expect("genesis rights len") as usize,
    }
}

fn read_wallet_id(out: &std::path::Path, actor: &str) -> String {
    let path = out.join("keys").join(format!("{actor}_keys.json"));
    let value: serde_json::Value =
        serde_json::from_slice(&read_file(&path).expect("read actor keys"))
            .expect("decode actor keys");
    let wallet = value["wallet_id"]
        .as_str()
        .expect("wallet_id field missing");
    if wallet.starts_with("wallet_") {
        wallet.to_string()
    } else {
        format!("wallet_{wallet}")
    }
}

fn receiver_keys(out: &std::path::Path, actor: &str, password: &str) -> ReceiverKeys {
    static WALLET_LOCK: Mutex<()> = Mutex::new(());
    let _guard = WALLET_LOCK.lock().expect("wallet lock");
    let wallet_id = read_wallet_id(out, actor);
    let svc = WalletService::with_output_dir(out.join("wallets"));
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async {
        let id = PersistWalletId(wallet_id);
        svc.load_wallet(&id, password).await.expect("load wallet");
        svc.unlock_wallet_in_memory(&id, &SafePassword::from(password))
            .await
            .expect("unlock wallet");
        svc.receiver_keys(&id).await.expect("receiver keys")
    })
}

fn scan_guard() -> std::sync::MutexGuard<'static, ()> {
    static SCAN_LOCK: Mutex<()> = Mutex::new(());
    SCAN_LOCK.lock().expect("scan lock")
}

#[test]
fn test_committed_scan_proofs_first() {
    if cfg!(debug_assertions) {
        return;
    }

    let case = stage11_case("stage7_jmt_wallet_scan_committed_local_v1");
    let keys = receiver_keys(&case.out, "bob", BOB_PASS);
    let _guard = scan_guard();
    let scan = jmt_wallet_scan::scan_post_tx_keys(&case.out, "bob", &keys).expect("jmt scan");

    assert_eq!(scan.actor, "bob");
    assert_eq!(scan.scan_path, "jmt_scan");
    assert_eq!(scan.proof_validated_count, scan.candidate_count);
    assert!(
        scan.candidate_count > 0,
        "post_tx store must enumerate committed rows"
    );
    assert!(scan
        .distinction
        .contains("not equivalent to detached Stage 5 leaf scan"));
    assert!(scan.rows.iter().all(|row| row.proof_validated));
    assert!(scan
        .rows
        .iter()
        .all(|row| row.scan_path == "committed_post_tx_jmt"));
    assert!(scan
        .rows
        .iter()
        .any(|row| row.receive_status.starts_with("RECEIVE_")));
}

#[test]
fn test_detached_leaf_missing_proof() {
    if cfg!(debug_assertions) {
        return;
    }

    let case = stage11_case("stage7_jmt_wallet_scan_detached_local_v1");
    let keys = receiver_keys(&case.out, "bob", BOB_PASS);
    let _guard = scan_guard();
    jmt_wallet_scan::scan_post_tx_keys(&case.out, "bob", &keys).expect("jmt scan");
    let mut candidate = jmt_wallet_scan::load_post_tx_candidates(&case.out)
        .expect("post_tx candidates")
        .into_iter()
        .next()
        .expect("committed candidate");
    candidate.proof_bytes.clear();

    let err = jmt_wallet_scan::scan_candidate_keys(&keys, &candidate).expect_err("proof required");
    assert!(err.contains("proof") || err.contains("committed-state"));
}

#[test]
fn test_stage7_updates_jmt_scan() {
    if cfg!(debug_assertions) {
        return;
    }

    let case = stage11_case("stage7_jmt_wallet_scan_report_local_v1");
    let artifact: serde_json::Value = serde_json::from_slice(
        &read_file(case.out.join("wallet_scan.json")).expect("read charlie scan"),
    )
    .expect("decode charlie scan");
    let diff: serde_json::Value = serde_json::from_slice(
        &read_file(
            case.out
                .join("transactions")
                .join("wallets_state_diff.json"),
        )
        .expect("read diff"),
    )
    .expect("decode diff");
    let s7: serde_json::Value = serde_json::from_slice(
        &read_file(case.out.join("transactions").join("checkpoint_s7.json"))
            .expect("read stage7 summary"),
    )
    .expect("decode stage7 summary");

    assert_eq!(artifact["actor"].as_str(), Some("charlie"));
    assert_eq!(artifact["scan_path"].as_str(), Some("jmt_scan"));
    assert!(artifact["detected_count"].as_u64().unwrap_or(0) > 0);
    assert!(artifact["total_detected_amount"].as_u64().unwrap_or(0) > 0);
    assert!(
        artifact["skipped_non_asset_count"].as_u64().unwrap_or(0) >= case.genesis_rights_len as u64
    );
    assert!(artifact["rows"].as_array().is_some_and(|rows| rows
        .iter()
        .any(|row| row["owner_detected"].as_bool() == Some(true))));
    assert!(diff["rows"]
        .as_array()
        .is_some_and(|rows| rows
            .iter()
            .any(|row| row["actor"].as_str() == Some("charlie")
                && row["status"].as_str() == Some("new")
                && row["lifecycle_status"].as_str() == Some("confirmed_receive"))));
    assert_eq!(s7["wallet_invariant_ok"].as_bool(), Some(true));
    assert!(s7["charlie_detected_count"].as_u64().unwrap_or(0) > 0);
}
