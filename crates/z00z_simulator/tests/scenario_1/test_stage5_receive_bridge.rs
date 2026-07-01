#![cfg(feature = "wallet_debug_tools")]

use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::scenario_support;
use z00z_simulator::scenario_1::support::stage_runner_support;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use tempfile::TempDir;
use z00z_crypto::Hidden;
use z00z_simulator::{
    config::ScenarioCfg,
    scenario_1::{stage_7, stage_8, stage_8::assertions},
    SimActor, StageResult,
};
use z00z_utils::io::{load_json, read_to_string, save_json, write_file};
use z00z_wallets::rpc::types::common::PersistWalletId;
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::decode_card_compact,
    tx::{TxOutRole, TxOutputWire, TxPackage},
    wallet::{
        ChainId as WalletChainId, WalletId, WalletKernel, WalletRecord, WalletSystemMetadata,
        WalletUserFields,
    },
    WalletService,
};

struct OutCase {
    out: PathBuf,
}

fn make_case_cfg_in(
    base: &Path,
    edit_cfg: impl FnOnce(&mut ScenarioCfg),
) -> (PathBuf, PathBuf, PathBuf) {
    scenario_support::make_cfg_in(base, |cfg| {
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
            .distinct_serial_ids_max = 10;
        stage4.transaction.outputs.bob_outputs_count = 4;
        stage4.transaction.fraction = Some(0.1);

        let stage5 = cfg.stage5_transfer.as_mut().expect("stage5 cfg");
        stage5.recipient_output_index = 0;
        edit_cfg(cfg);
    })
}

fn stage6_case() -> &'static OutCase {
    static CASE: std::sync::OnceLock<OutCase> = std::sync::OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage5_receive_bridge_stage6_v2", |base| {
            let (cfg_path, design_path, out) = make_case_cfg_in(base, |_| {});
            let _ctx = stage_runner_support::run_stage4_setup(&cfg_path, &design_path);
            assert!(
                stage_runner_support::tx_pkg_path(&out).exists(),
                "stage6 cache must contain stage4 tx package"
            );
        });
        OutCase {
            out: root.join("outputs/scenario_1"),
        }
    })
}

fn stage7_case() -> &'static OutCase {
    static CASE: std::sync::OnceLock<OutCase> = std::sync::OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage5_receive_bridge_stage7_v2", |base| {
            let (cfg_path, design_path, out) = make_case_cfg_in(base, |_| {});
            fixture_cache::copy_tree(&stage6_case().out, &out);
            let mut ctx = mk_ctx(&cfg_path);
            let stage = stage_runner_support::stage_by_id(&design_path, 7);
            let res = stage_7::run_transfer_receive(&mut ctx, &stage);
            assert!(
                matches!(res, StageResult::Ok),
                "stage7 cache build failed: {res:?}"
            );
            assert!(
                out.join("transactions")
                    .join("stage_5_receive_handoff.json")
                    .exists(),
                "stage7 cache must contain receive handoff"
            );
        });
        OutCase {
            out: root.join("outputs/scenario_1"),
        }
    })
}

fn stage8_case() -> &'static OutCase {
    static CASE: std::sync::OnceLock<OutCase> = std::sync::OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage5_receive_bridge_stage8_v2", |base| {
            let (cfg_path, design_path, out) = make_case_cfg_in(base, |_| {});
            fixture_cache::copy_tree(&stage7_case().out, &out);
            let mut ctx = mk_ctx(&cfg_path);
            let stage = stage_runner_support::stage_by_id(&design_path, 8);
            let res = stage_8::run_transfer_claim(&mut ctx, &stage);
            assert!(
                matches!(res, StageResult::Ok),
                "stage8 cache build failed: {res:?}"
            );
            assert!(
                out.join("stage_5_snapshot.json").exists(),
                "stage8 cache must contain stage5 snapshot"
            );
            assert!(
                out.join("transactions")
                    .join("leaf_alice_to_bob.json")
                    .exists(),
                "stage8 cache must contain leaf artifact"
            );
        });
        OutCase {
            out: root.join("outputs/scenario_1"),
        }
    })
}

fn remap_out(runtime_out: &Path, cfg_out: &Path, raw: &str) -> PathBuf {
    let configured = PathBuf::from(raw);
    let marker = Path::new("crates/z00z_simulator/outputs/scenario_1");
    if configured.is_absolute() {
        return configured;
    }
    if let Ok(stripped) = configured.strip_prefix(marker) {
        return runtime_out.join(stripped);
    }
    if configured.starts_with(cfg_out) {
        if let Ok(suffix) = configured.strip_prefix(cfg_out) {
            return runtime_out.join(suffix);
        }
    }
    runtime_out.join(configured)
}

fn parse_secret_row(path: &Path, actor_name: &str) -> (String, String, String) {
    let text = read_to_string(path).expect("read wallet secrets debug md");
    for line in text.lines() {
        if !line.contains('|') {
            continue;
        }
        let cols: Vec<&str> = line.split('|').map(str::trim).collect();
        if cols.len() < 5 || cols[0] == "name" || cols[0].starts_with("-----") {
            continue;
        }
        if cols[0].eq_ignore_ascii_case(actor_name) {
            return (
                cols[1].to_string(),
                cols[2].to_string(),
                cols[4].to_string(),
            );
        }
    }
    panic!(
        "wallet secret row not found for actor {} in {}",
        actor_name,
        path.display()
    );
}

fn mk_record(wallet_id_hex: &str, actor_name: &str, now_ms: u64) -> WalletRecord {
    let mut id_bytes = [0u8; 32];
    hex::decode_to_slice(wallet_id_hex, &mut id_bytes).expect("decode wallet id hex");
    let kernel = WalletKernel::new(WalletId(id_bytes), WalletChainId::DEVNET);
    let user = WalletUserFields {
        wallet_name: actor_name.to_string(),
        memo: None,
    };
    let system = WalletSystemMetadata {
        created_at: now_ms,
        updated_at: now_ms,
    };
    WalletRecord::new(kernel, user, system)
}

struct LoadedActor {
    actor: SimActor,
    password: String,
}

fn load_actor(cfg: &ScenarioCfg, out: &Path, actor_name: &str) -> LoadedActor {
    let cfg_out = PathBuf::from(cfg.outputs.dir.clone());
    let stage2_wallets_dir = remap_out(out, &cfg_out, &cfg.stage2_paths().wallets_dir);
    let secrets_path = cfg
        .stage2_secret_artifact_path(&stage2_wallets_dir)
        .expect("wallet_debug_tools secrets path");
    let (persist_wallet_id, password, secret_hex) = parse_secret_row(&secrets_path, actor_name);

    let mut secret_bytes = [0u8; 32];
    hex::decode_to_slice(&secret_hex, &mut secret_bytes).expect("decode receiver secret hex");
    let keys = ReceiverKeys::from_receiver_secret(
        ReceiverSecret::from_bytes(secret_bytes).expect("receiver secret for keys"),
    )
    .expect("derive receiver keys");
    let stage4 = cfg.stage4_tx_prepare.as_ref().expect("stage4 cfg");
    let keys_file = match actor_name.to_ascii_lowercase().as_str() {
        "alice" => remap_out(out, &cfg_out, &stage4.paths.alice_keys_file),
        "bob" => remap_out(out, &cfg_out, &stage4.paths.bob_keys_file),
        other => panic!("unexpected actor for stage5 rehydrate: {other}"),
    };
    let keys_json: serde_json::Value = load_json(&keys_file).expect("load receiver keys json");
    let wallet_id_hex = keys_json["wallet_id"]
        .as_str()
        .expect("wallet_id in keys json");
    let card_compact = keys_json["card_compact"]
        .as_str()
        .expect("card_compact in keys json");
    let card = decode_card_compact(card_compact).expect("decode receiver card");

    let now_ms = cfg.simulation.mock_rng_seed.unwrap_or(0);
    LoadedActor {
        actor: SimActor {
            name: actor_name.to_ascii_lowercase(),
            password: Some(password.clone()),
            wallet_id: persist_wallet_id,
            record: mk_record(wallet_id_hex, actor_name, now_ms),
            keys,
            card,
            balance: HashMap::new(),
            receiver_secret: Hidden::hide(secret_bytes),
            session: None,
        },
        password,
    }
}

fn mk_ctx(cfg_path: &Path) -> stage_runner_support::StageSession {
    let mut ctx = stage_runner_support::resume_stage_session(cfg_path);
    let out_dir = PathBuf::from(ctx.ctx.config.outputs.dir.clone());
    let cfg_out = PathBuf::from(ctx.ctx.config.outputs.dir.clone());
    let stage4 = ctx
        .ctx
        .config
        .stage4_tx_prepare
        .as_ref()
        .expect("stage4 cfg");
    let wallets_dir = remap_out(&out_dir, &cfg_out, &stage4.paths.wallets_dir);
    let loaded = load_actor(&ctx.ctx.config, &out_dir, &stage4.receiver_actor);
    let wallet_service = Arc::new(WalletService::with_output_dir(wallets_dir));
    tokio::runtime::Runtime::new()
        .expect("tokio runtime")
        .block_on(wallet_service.load_wallet(
            &PersistWalletId(loaded.actor.wallet_id.clone()),
            &loaded.password,
        ))
        .expect("rehydrate receiver wallet");
    ctx.ctx.actors = vec![loaded.actor];
    ctx.ctx.wallet_service = Some(wallet_service);
    ctx
}

fn clone_case(
    src_out: &Path,
    edit_cfg: impl FnOnce(&mut ScenarioCfg),
) -> (TempDir, PathBuf, PathBuf, PathBuf) {
    let temp = TempDir::new().expect("temp dir");
    let (cfg_path, design_path, out) = make_case_cfg_in(temp.path(), edit_cfg);
    fixture_cache::copy_tree(src_out, &out);
    (temp, cfg_path, design_path, out)
}

fn recv_row(pkg: &mut TxPackage) -> &mut TxOutputWire {
    pkg.tx
        .outputs
        .iter_mut()
        .find(|row| row.role == TxOutRole::Recipient)
        .expect("recipient output")
}

fn assert_any_fail_text(msg: &str, expected: &[&str]) {
    assert!(
        expected.iter().any(|needle| msg.contains(needle)),
        "unexpected stage5 failure text: {msg}"
    );
}

fn run_stage7_fail(edit_cfg: impl FnOnce(&mut ScenarioCfg), edit_tx: impl FnOnce(&Path)) -> String {
    let (_temp, cfg_path, design_path, out) = clone_case(&stage6_case().out, edit_cfg);
    let mut ctx = mk_ctx(&cfg_path);
    let tx_path = stage_runner_support::tx_pkg_path(&out);
    edit_tx(&tx_path);
    let stage = stage_runner_support::stage_by_id(&design_path, 7);
    stage_runner_support::fail_text(stage_7::run_transfer_receive(&mut ctx, &stage))
}

fn run_stage8_fail(edit_handoff: impl FnOnce(&Path)) -> String {
    let (_temp, cfg_path, design_path, out) = clone_case(&stage7_case().out, |_| {});
    let mut ctx = mk_ctx(&cfg_path);
    let handoff_path = out
        .join("transactions")
        .join("stage_5_receive_handoff.json");
    edit_handoff(&handoff_path);
    let stage = stage_runner_support::stage_by_id(&design_path, 8);
    stage_runner_support::fail_text(stage_8::run_transfer_claim(&mut ctx, &stage))
}

#[test]
fn test_stage5_detect_report() {
    let out = &stage8_case().out;

    let snap: serde_json::Value =
        load_json(out.join("stage_5_snapshot.json")).expect("stage 5 snapshot");
    assertions::assert_snap(&snap);

    let tx: serde_json::Value = load_json(out.join("transactions").join("leaf_alice_to_bob.json"))
        .expect("stage 5 leaf artifact");
    assertions::assert_tx(&tx);
    assertions::assert_selected_from_stage4(out, &tx);
    assertions::assert_safe(&snap, &tx);

    let events = assertions::read_events(out);
    assert!(events.contains("canonical_scan"));
    assert!(events.contains("runtime_scan"));
    assert!(events.contains("rpc_report_only"));
    assert!(events.contains("explicit_claim"));
    assert!(!events.contains("stage5_from_stage4"));
}

#[test]
fn test_stage5_claim_idempotent() {
    let out = &stage8_case().out;
    let snap: serde_json::Value =
        load_json(out.join("stage_5_snapshot.json")).expect("stage 5 snapshot");

    let detail = assertions::log_detail(out, "explicit_claim");
    assert!(detail.contains("persisted=false"));
    let before = assertions::parse_count(&detail, "claimed_before_route");
    let after = assertions::parse_count(&detail, "claimed_after_route");
    assert_eq!(before, after);
    assert_eq!(snap["claimed_count_after_route"].as_u64(), Some(after));
}

#[test]
fn test_stage5_claim_not_implicit() {
    let out = &stage8_case().out;
    let detail = assertions::log_detail(out, "rpc_report_only");
    let before = assertions::parse_count(&detail, "claimed_before");
    let after = assertions::parse_count(&detail, "claimed_after_rpc");
    assert_eq!(before, after);
    assert!(before > 0);
}

#[test]
fn test_stage5_bad_index_fails() {
    let msg = run_stage7_fail(
        |cfg| {
            let stage5_cfg = cfg.stage5_transfer.as_mut().expect("stage5 cfg");
            stage5_cfg.recipient_output_index = 99;
        },
        |_| {},
    );
    assert!(msg.contains("stage5: recipient output index out of range: 99"));
}

#[test]
fn test_stage5_role_guard() {
    let msg = run_stage7_fail(
        |_| {},
        |tx_path| {
            let mut pkg: TxPackage = load_json(tx_path).expect("load stage4 tx package");
            for row in &mut pkg.tx.outputs {
                if row.role == TxOutRole::Recipient {
                    row.role = TxOutRole::Change;
                }
            }
            save_json(tx_path, &pkg).expect("rewrite stage4 tx package");
        },
    );

    assert_any_fail_text(
        &msg,
        &[
            "stage5: no recipient outputs in stage4 tx package",
            "stage5: tx package verification failed:",
        ],
    );
}

#[test]
fn test_stage5_pkg_malformed() {
    let msg = run_stage7_fail(
        |_| {},
        |tx_path| {
            write_file(tx_path, b"{").expect("corrupt tx package");
        },
    );

    assert_any_fail_text(
        &msg,
        &[
            "stage5: json parse",
            "stage5: tx package verification failed:",
        ],
    );
}

#[test]
fn test_stage5_rejects_chain_metadata() {
    let msg = run_stage7_fail(
        |_| {},
        |tx_path| {
            let mut pkg: TxPackage = load_json(tx_path).expect("load stage4 tx package");
            pkg.chain_id = 0;
            save_json(tx_path, &pkg).expect("rewrite stage4 tx package");
        },
    );

    assert!(msg.contains("stage5: tx package verification failed:"));
    assert!(msg.contains("stage4: tx package invalid:"));
}

#[test]
fn test_stage5_asset_wire_bad() {
    let msg = run_stage7_fail(
        |_| {},
        |tx_path| {
            let mut pkg: TxPackage = load_json(tx_path).expect("load stage4 tx package");
            let row = recv_row(&mut pkg);
            row.asset_wire.serial_id = row.asset_wire.definition.serials;
            save_json(tx_path, &pkg).expect("rewrite stage4 tx package");
        },
    );

    assert_any_fail_text(
        &msg,
        &[
            "stage5: selected output to_asset failed:",
            "stage5: tx package verification failed:",
        ],
    );
}

#[test]
fn test_stage5_stealth_shape_bad() {
    let msg = run_stage7_fail(
        |_| {},
        |tx_path| {
            let mut pkg: TxPackage = load_json(tx_path).expect("load stage4 tx package");
            let row = recv_row(&mut pkg);
            row.asset_wire.owner_tag = None;
            save_json(tx_path, &pkg).expect("rewrite stage4 tx package");
        },
    );

    assert_any_fail_text(
        &msg,
        &[
            "stage5: selected asset stealth consistency failed:",
            "stage5: tx package verification failed:",
        ],
    );
}

#[test]
fn test_stage5_leaf_shape_bad() {
    let msg = run_stage7_fail(
        |_| {},
        |tx_path| {
            let mut pkg: TxPackage = load_json(tx_path).expect("load stage4 tx package");
            let row = recv_row(&mut pkg);
            row.asset_wire.r_pub = None;
            row.asset_wire.owner_tag = None;
            row.asset_wire.enc_pack = None;
            row.asset_wire.tag16 = None;
            row.asset_wire.leaf_ad_id = None;
            save_json(tx_path, &pkg).expect("rewrite stage4 tx package");
        },
    );

    assert_any_fail_text(
        &msg,
        &[
            "stage5: selected output to_leaf failed:",
            "stage5: tx package verification failed:",
        ],
    );
}

#[test]
fn test_stage8_handoff_stage_mismatch() {
    let msg = run_stage8_fail(|handoff_path| {
        let mut handoff: serde_json::Value = load_json(handoff_path).expect("load handoff");
        handoff["stage"] = serde_json::Value::from(99_u64);
        save_json(handoff_path, &handoff).expect("rewrite handoff");
    });
    assert!(msg.contains("receive handoff stage mismatch"));
}

#[test]
fn test_stage8_handoff_mismatch_fails() {
    let msg = run_stage8_fail(|handoff_path| {
        let mut handoff: serde_json::Value = load_json(handoff_path).expect("load handoff");
        handoff["status"] = serde_json::Value::from("warn");
        save_json(handoff_path, &handoff).expect("rewrite handoff");
    });
    assert!(msg.contains("receive handoff status mismatch"));
}

#[test]
fn test_stage8_handoff_status_mismatch() {
    let msg = run_stage8_fail(|handoff_path| {
        let mut handoff: serde_json::Value = load_json(handoff_path).expect("load handoff");
        handoff["rpc_status"] = serde_json::Value::from("RECEIVE_NOT_MINE");
        save_json(handoff_path, &handoff).expect("rewrite handoff");
    });
    assert!(msg.contains("receive handoff rpc_status mismatch"));
}

#[test]
fn test_stage8_handoff_claimed_mismatch() {
    let msg = run_stage8_fail(|handoff_path| {
        let mut handoff: serde_json::Value = load_json(handoff_path).expect("load handoff");
        handoff["claimed_before_route"] = serde_json::Value::from(999_u64);
        save_json(handoff_path, &handoff).expect("rewrite handoff");
    });
    assert!(msg.contains("receive handoff claimed_before_route mismatch"));
}
