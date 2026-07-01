use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use serde_json::Value;
use z00z_crypto::expert::encoding::SafePassword;
use z00z_simulator::{scenario_1::stage_6, StageResult};
use z00z_utils::io::{create_dir_all, path_exists, read_file, read_to_string, write_file};
use z00z_wallets::{
    domains::hashing::compute_wallet_file_id,
    key::{ReceiverKeys, ReceiverSecret},
    receiver::check_stealth_own,
    rpc::types::{common::PersistWalletId, wallet::WalletSource},
    services::WalletService,
    tx::{TxOutRole, TxPackage},
};

use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::scenario_support;
use z00z_simulator::scenario_1::support::stage_runner_support;

use crate::stage4_paths::{assert_absent, lock_stage4_tamper};
use scenario_support::{make_cfg, make_cfg_in};

fn tamper_file(out: &Path) -> std::path::PathBuf {
    out.parent()
        .unwrap_or(out)
        .join("test_hooks/stage4_output_tamper.txt")
}

fn tx_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/tx_alice_to_bob_pkg.json")
}

fn after_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/wallets_state_after.json")
}

fn diff_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/wallets_state_diff.json")
}

fn pending_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/wallets_pending.json")
}

fn confirm_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/wallets_confirmed.json")
}

fn report_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/wallets_state_report.md")
}

fn secret_seed(seed: u8) -> [u8; 32] {
    [seed; 32]
}

fn load_tx_package(out: &Path) -> TxPackage {
    serde_json::from_slice(&read_file(tx_file(out)).expect("read tx")).expect("decode tx")
}

fn load_rows(out: &Path, name: &str) -> Vec<Value> {
    serde_json::from_slice(&read_file(out.join("transactions").join(name)).expect("read rows"))
        .expect("decode rows")
}

fn load_diff_rows(out: &Path) -> Vec<Value> {
    let dump: Value = serde_json::from_slice(&read_file(diff_file(out)).expect("read diff"))
        .expect("decode diff");
    dump["rows"].as_array().expect("diff rows").clone()
}

fn load_after_wallet_id(out: &Path, actor: &str) -> String {
    let dump: Value =
        serde_json::from_slice(&read_file(after_file(out)).expect("read after state"))
            .expect("decode after state");
    dump["wallets"]
        .as_array()
        .expect("wallets rows")
        .iter()
        .find(|row| row["actor"].as_str() == Some(actor))
        .and_then(|row| row["wallet_id"].as_str())
        .map(|row| row.to_string())
        .expect("wallet id for actor")
}

fn wallet_path(out: &Path, wallet_id: &str) -> std::path::PathBuf {
    let file_id = compute_wallet_file_id(wallet_id);
    let file_hex = z00z_crypto::expert::encoding::to_hex(&file_id[..8]);
    out.join("wallets").join(format!("wallet_{file_hex}.wlt"))
}

fn load_keys_from_wallet(out: &Path, wallet_id: &str, wallet_pass: &str) -> ReceiverKeys {
    let wallets_dir = out.join("wallets");
    let wallet_path = wallet_path(out, wallet_id);
    let wallet_id = PersistWalletId(wallet_id.to_string());
    tokio::runtime::Runtime::new()
        .expect("tokio runtime")
        .block_on(async {
            let service = Arc::new(WalletService::with_output_dir(wallets_dir));
            service
                .open_wallet_source(WalletSource::Path {
                    path: wallet_path.to_string_lossy().to_string(),
                })
                .await
                .expect("open fee wallet source");
            service
                .unlock_wallet_in_memory(&wallet_id, &SafePassword::from(wallet_pass))
                .await
                .expect("unlock fee wallet in memory");
            let keys = service
                .receiver_keys(&wallet_id)
                .await
                .expect("receiver keys for fee wallet");
            service
                .lock_wallet(&wallet_id)
                .await
                .expect("lock fee wallet");
            keys
        })
}

fn assert_no_post_state(out: &Path) {
    assert_absent(&tx_file(out));
    assert_absent(&after_file(out));
    assert_absent(&diff_file(out));
    assert_absent(&pending_file(out));
    assert_absent(&confirm_file(out));
}

struct OutCase {
    out: PathBuf,
}

fn output_crypto_case() -> &'static OutCase {
    static CASE: OnceLock<OutCase> = OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage4_output_crypto_ok_v1", |base| {
            let (cfg_path, design_path, out) = make_cfg_in(base, |cfg| {
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
            });
            let _ctx = stage_runner_support::run_stage4_setup(&cfg_path, &design_path);
            assert!(path_exists(tx_file(&out)).expect("tx file exists"));
        });
        OutCase {
            out: root.join("outputs/scenario_1"),
        }
    })
}

fn fee_card_case() -> &'static OutCase {
    static CASE: OnceLock<OutCase> = OnceLock::new();
    CASE.get_or_init(|| {
        let fee_name = "fee_sink_test".to_string();
        let fee_pass = "FeeSink_Pass_Z00Z_61!".to_string();
        let root = fixture_cache::ensure_case("stage4_output_crypto_fee_card_v1", |base| {
            let (cfg_path, design_path, out) = make_cfg_in(base, |cfg| {
                let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
                stage4.transaction.fee_sink.wallet_id = fee_name.clone();
                stage4.transaction.fee_sink.receiver_card_hex = None;
                stage4.transaction.fee_sink.password = Some(fee_pass.clone());
                stage4.transaction.fee_sink.rng_seed = Some(61);
            });
            let _ctx = stage_runner_support::run_stage4_setup(&cfg_path, &design_path);
            assert!(path_exists(tx_file(&out)).expect("tx file exists"));
        });
        OutCase {
            out: root.join("outputs/scenario_1"),
        }
    })
}

fn roles_case() -> &'static OutCase {
    static CASE: OnceLock<OutCase> = OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage4_output_crypto_roles_v1", |base| {
            let (cfg_path, design_path, out) = make_cfg_in(base, |cfg| {
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
                stage4.transaction.mode = "fraction".to_string();
                stage4.transaction.fraction = Some(0.5);
                stage4.transaction.amount = None;
                stage4.transaction.outputs.bob_outputs_count = 3;
            });
            let _ctx = stage_runner_support::run_stage4_setup(&cfg_path, &design_path);
            assert!(path_exists(tx_file(&out)).expect("tx file exists"));
        });
        OutCase {
            out: root.join("outputs/scenario_1"),
        }
    })
}

#[test]
fn test_stage4_output_crypto_ok() {
    let _guard = lock_stage4_tamper();
    let out = &output_crypto_case().out;
    assert!(path_exists(tx_file(out)).expect("tx file exists"));
}

#[test]
fn test_stage4_output_tamper_fail() {
    let _guard = lock_stage4_tamper();
    let (cfg_path, design_path, out) = make_cfg(|cfg| {
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
    });
    let mut ctx = stage_runner_support::run_stage5_session(&cfg_path, &design_path);
    create_dir_all(tamper_file(&out).parent().expect("tamper file parent"))
        .expect("test_hooks dir");
    write_file(tamper_file(&out), b"tag16").expect("write tamper file");
    let stage = stage_runner_support::stage_by_id(&design_path, 6);
    match stage_6::run_tx_prepare(&mut ctx, &stage) {
        StageResult::Fail(msg) => {
            assert!(msg.contains("tag16 mismatch"), "unexpected error: {msg}");
        }
        other => panic!("tx_prepare stage must fail, got {other:?}"),
    }

    assert_no_post_state(&out);
}

#[test]
fn test_stage4_fee_card_scan() {
    let _guard = lock_stage4_tamper();
    let fee_name = "fee_sink_test".to_string();
    let fee_pass = "FeeSink_Pass_Z00Z_61!".to_string();
    let out = &fee_card_case().out;
    let pkg = load_tx_package(out);
    let fee_wallet_id = load_after_wallet_id(out, fee_name.as_str());
    let fee_keys = load_keys_from_wallet(out, &fee_wallet_id, &fee_pass);
    let fee_out = pkg
        .tx
        .outputs
        .iter()
        .find(|out| matches!(out.role, TxOutRole::Fee))
        .expect("fee output");
    let fee_asset = fee_out.asset_wire.clone().to_asset().expect("fee asset");
    let wrong_sec = ReceiverSecret::from_bytes(secret_seed(0x62)).expect("wrong secret");
    let wrong_keys = ReceiverKeys::from_receiver_secret(wrong_sec).expect("wrong keys");

    assert!(check_stealth_own(&fee_asset, &fee_keys).is_ok());
    assert!(check_stealth_own(&fee_asset, &wrong_keys).is_err());

    let pending = load_rows(out, "wallets_pending.json");
    let confirmed = load_rows(out, "wallets_confirmed.json");
    let report = read_to_string(report_file(out)).expect("read report");

    let fee_pending = pending.iter().find(|row| {
        row["output_role"].as_str() == Some("Fee")
            && row["actor"].as_str() == Some(fee_name.as_str())
            && row["wallet_id"].as_str() == Some(fee_wallet_id.as_str())
            && row["lifecycle_status"].as_str() == Some("pending_fee")
    });
    let fee_confirm = confirmed.iter().find(|row| {
        row["output_role"].as_str() == Some("Fee")
            && row["actor"].as_str() == Some(fee_name.as_str())
            && row["wallet_id"].as_str() == Some(fee_wallet_id.as_str())
            && row["lifecycle_status"].as_str() == Some("confirmed_fee")
    });

    assert!(
        fee_pending.is_some(),
        "pending rows must expose configured fee sink"
    );
    assert!(
        fee_confirm.is_some(),
        "confirmed rows must expose configured fee sink"
    );
    assert!(
        report.contains(fee_name.as_str()),
        "report must show configured fee sink"
    );
    assert!(
        report.contains(fee_wallet_id.as_str()),
        "report must show real fee wallet id"
    );
}

#[test]
fn test_stage4_roles_render_distinctly() {
    let _guard = lock_stage4_tamper();
    let out = &roles_case().out;
    let pending = load_rows(out, "wallets_pending.json");
    let confirmed = load_rows(out, "wallets_confirmed.json");
    let diff_rows = load_diff_rows(out);
    let report = read_to_string(report_file(out)).expect("read report");

    assert!(pending.iter().any(|row| {
        row["actor"].as_str() == Some("bob")
            && row["output_role"].as_str() == Some("Recipient")
            && row["lifecycle_status"].as_str() == Some("pending_receive")
    }));
    assert!(pending.iter().any(|row| {
        row["actor"].as_str() == Some("alice")
            && row["output_role"].as_str() == Some("Change")
            && row["lifecycle_status"].as_str() == Some("pending_change")
    }));
    assert!(pending.iter().any(|row| {
        row["output_role"].as_str() == Some("Fee")
            && row["lifecycle_status"].as_str() == Some("pending_fee")
    }));

    assert!(confirmed.iter().any(|row| {
        row["actor"].as_str() == Some("bob")
            && row["output_role"].as_str() == Some("Recipient")
            && row["lifecycle_status"].as_str() == Some("confirmed_receive")
    }));
    assert!(confirmed.iter().any(|row| {
        row["actor"].as_str() == Some("alice")
            && row["output_role"].as_str() == Some("Change")
            && row["lifecycle_status"].as_str() == Some("confirmed_change")
    }));
    assert!(confirmed.iter().any(|row| {
        row["output_role"].as_str() == Some("Fee")
            && row["lifecycle_status"].as_str() == Some("confirmed_fee")
    }));

    assert!(diff_rows.iter().any(|row| {
        row["actor"].as_str() == Some("bob")
            && row["output_role"].as_str() == Some("Recipient")
            && row["lifecycle_status"].as_str() == Some("confirmed_receive")
    }));
    assert!(diff_rows.iter().any(|row| {
        row["actor"].as_str() == Some("alice")
            && row["output_role"].as_str() == Some("Change")
            && row["lifecycle_status"].as_str() == Some("confirmed_change")
    }));
    assert!(diff_rows.iter().any(|row| {
        row["output_role"].as_str() == Some("Fee")
            && row["lifecycle_status"].as_str() == Some("confirmed_fee")
    }));

    assert!(report.contains("| actor | serial_id | asset_id | class | output_role | before | after | status | lifecycle_status | tx_digest |"));
    assert!(
        report.contains("| alice |"),
        "report must include alice rows"
    );
    assert!(report.contains("| bob |"), "report must include bob rows");
    assert!(
        report.contains("| Change |"),
        "report must render Change role"
    );
    assert!(
        report.contains("| Recipient |"),
        "report must render Recipient role"
    );
    assert!(report.contains("| Fee |"), "report must render Fee role");
}
