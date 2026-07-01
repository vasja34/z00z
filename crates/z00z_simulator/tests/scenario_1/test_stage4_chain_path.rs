#![cfg(feature = "wallet_debug_tools")]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use serde_json::Value;
use z00z_crypto::Hidden;
use z00z_simulator::{
    config::ScenarioCfg,
    config::Stage6ProofMode,
    scenario_1::{stage_11, stage_12, stage_9},
    SimActor, StageResult,
};
use z00z_utils::io::{load_json, read_to_string, save_json};
use z00z_wallets::rpc::types::common::PersistWalletId;
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::decode_card_compact,
    wallet::{
        ChainId as WalletChainId, WalletId, WalletKernel, WalletRecord, WalletSystemMetadata,
        WalletUserFields,
    },
    WalletService,
};

use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::scenario_support;
use z00z_simulator::scenario_1::support::stage_runner_support;

use scenario_support::make_cfg_in;

fn good_s4(cfg: &mut ScenarioCfg) {
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

    cfg.stage6_bundle
        .get_or_insert_with(Default::default)
        .proof_mode = Stage6ProofMode::DraftOnly;

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

fn summary_file(out: &Path, view: &str) -> PathBuf {
    out.join("storage").join(view).join("summary.json")
}

fn ledger_path_file(out: &Path) -> PathBuf {
    out.join("storage").join("ledger_path.json")
}

struct OutCase {
    out: PathBuf,
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

fn load_actor(cfg: &ScenarioCfg, out: &Path, actor_name: &str) -> (SimActor, String) {
    let cfg_out = PathBuf::from(cfg.outputs.dir.clone());
    let stage2 = cfg.stage2_paths();
    let wallets_dir = remap_out(out, &cfg_out, &stage2.wallets_dir);
    let secrets_path = cfg
        .stage2_secret_artifact_path(&wallets_dir)
        .expect("wallet_debug_tools secrets path");
    let (persist_wallet_id, password, secret_hex) = parse_secret_row(&secrets_path, actor_name);

    let mut secret_bytes = [0u8; 32];
    hex::decode_to_slice(&secret_hex, &mut secret_bytes).expect("decode receiver secret hex");
    let keys = ReceiverKeys::from_receiver_secret(
        ReceiverSecret::from_bytes(secret_bytes).expect("receiver secret"),
    )
    .expect("derive receiver keys");

    let keys_dir = remap_out(out, &cfg_out, &stage2.keys_dir);
    let keys_json: Value =
        load_json(keys_dir.join(format!("{}_keys.json", actor_name.to_ascii_lowercase())))
            .expect("load actor keys json");
    let wallet_id_hex = keys_json["wallet_id"]
        .as_str()
        .expect("wallet_id in keys json");
    let card_compact = keys_json["card_compact"]
        .as_str()
        .expect("card_compact in keys json");
    let card = decode_card_compact(card_compact).expect("decode receiver card");

    let now_ms = cfg.simulation.mock_rng_seed.unwrap_or(0);
    (
        SimActor {
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
    )
}

fn mk_stage11_ctx(cfg_path: &Path) -> stage_runner_support::StageSession {
    let mut ctx = stage_runner_support::resume_stage_session(cfg_path);
    let out = PathBuf::from(ctx.ctx.config.outputs.dir.clone());
    let cfg_out = PathBuf::from(ctx.ctx.config.outputs.dir.clone());
    let wallets_dir = remap_out(&out, &cfg_out, &ctx.ctx.config.stage2_paths().wallets_dir);
    let (charlie, password) = load_actor(&ctx.ctx.config, &out, "charlie");
    let wallet_service = Arc::new(WalletService::with_output_dir(wallets_dir));
    tokio::runtime::Runtime::new()
        .expect("tokio runtime")
        .block_on(
            wallet_service.load_wallet(&PersistWalletId(charlie.wallet_id.clone()), &password),
        )
        .expect("rehydrate charlie wallet");
    ctx.ctx.actors = vec![charlie];
    ctx.ctx.wallet_service = Some(wallet_service);
    ctx
}

fn stage9_case() -> &'static OutCase {
    static CASE: OnceLock<OutCase> = OnceLock::new();
    CASE.get_or_init(|| {
        let root =
            fixture_cache::ensure_shared_case("stage4_chain_path_stage9_shared_v4", |base| {
                let (cfg_path, design_path, out) = make_cfg_in(base, good_s4);
                let mut ctx = stage_runner_support::run_stage4_session(&cfg_path, &design_path);
                let stage = stage_runner_support::stage_by_id(&design_path, 9);
                assert!(matches!(
                    stage_9::run_bundle_build(&mut ctx, &stage),
                    StageResult::Ok
                ));
                assert!(
                    out.join("transactions")
                        .join("checkpoint_bridge_s6.json")
                        .exists(),
                    "stage9 baseline must export checkpoint bridge"
                );
                assert!(
                    out.join("transactions")
                        .join("checkpoint_prep.json")
                        .exists(),
                    "stage9 baseline must export prep reference"
                );
                assert!(ledger_path_file(&out).exists());
            });
        OutCase {
            out: root.join("outputs/scenario_1"),
        }
    })
}

fn rerun_case() -> &'static OutCase {
    static CASE: OnceLock<OutCase> = OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_shared_case("stage4_chain_path_rerun_shared_v2", |base| {
            let (cfg_path, design_path, out) = make_cfg_in(base, good_s4);
            fixture_cache::copy_tree(&stage9_case().out, &out);
            let mut first_ledger: Value =
                load_json(ledger_path_file(&out)).expect("first ledger path");
            first_ledger["checkpoint_id_hex"] = Value::String("stale-checkpoint-id".to_string());
            save_json(ledger_path_file(&out), &first_ledger).expect("write stale ledger path");

            let mut ctx = mk_stage11_ctx(&cfg_path);
            let stage11 = stage_runner_support::stage_by_id(&design_path, 11);
            let stage12 = stage_runner_support::stage_by_id(&design_path, 12);
            let res11 = stage_11::run_apply(&mut ctx, &stage11);
            assert!(matches!(res11, StageResult::Ok), "{res11:?}");
            let res12 = stage_12::run_finalize(&mut ctx, &stage12);
            assert!(matches!(res12, StageResult::Ok), "{res12:?}");
            assert!(summary_file(&out, "claim_post").exists());
            assert!(summary_file(&out, "pre_tx").exists());
            assert!(ledger_path_file(&out).exists());
        });
        OutCase {
            out: root.join("outputs/scenario_1"),
        }
    })
}

#[test]
fn test_stage4_root_continuity() {
    let out = &rerun_case().out;

    let claim_post: Value = load_json(summary_file(out, "claim_post")).expect("claim_post summary");
    let pre_tx: Value = load_json(summary_file(out, "pre_tx")).expect("pre_tx summary");
    let ledger_path: Value = load_json(ledger_path_file(out)).expect("ledger path");

    let claim_root = claim_post
        .get("source_check_root_hex")
        .and_then(Value::as_str)
        .expect("claim_post source root");
    let prep_root = pre_tx
        .get("source_check_root_hex")
        .and_then(Value::as_str)
        .expect("pre_tx source root");

    assert_eq!(
        prep_root, claim_root,
        "pre_tx continuity root must reuse the claim-backed live store root"
    );
    assert_eq!(
        ledger_path
            .get("claim_root_hex")
            .and_then(Value::as_str)
            .expect("ledger claim root"),
        claim_root,
        "ledger path must record the claim root"
    );
    assert_eq!(
        ledger_path
            .get("prep_root_hex")
            .and_then(Value::as_str)
            .expect("ledger prep root"),
        prep_root,
        "ledger path must record the prep root"
    );
    assert!(
        ledger_path
            .get("exec_input_id_hex")
            .and_then(Value::as_str)
            .map(str::len)
            == Some(64),
        "ledger path must record exec_input_id_hex after downstream apply stages run"
    );
    assert!(
        ledger_path
            .get("draft_id_hex")
            .and_then(Value::as_str)
            .map(str::len)
            == Some(64),
        "ledger path must record draft_id_hex after downstream apply stages run"
    );
    assert!(
        ledger_path
            .get("post_apply_root_hex")
            .and_then(Value::as_str)
            .map(str::len)
            == Some(64),
        "ledger path must record post_apply_root_hex after downstream apply stages run"
    );
    assert!(
        ledger_path.get("checkpoint_id_hex").is_none(),
        "ledger path must clear checkpoint_id_hex in draft-only finalization mode"
    );
    assert!(
        ledger_path.get("checkpoint_id_hex").is_none(),
        "ledger path must clear stale checkpoint_id_hex on rerun in draft-only finalization mode"
    );
}
