//! Scenario 1 stage 4 implementation: publish claim outputs after stage 3 prepare.
//!
//! Stage 4 claim-publish files are structurally useful but weaker than later
//! spend/checkpoint semantic acceptance. They remain non-authoritative for
//! accepted tx/checkpoint state mutation until the later fail-closed gates
//! succeed.

use serde::Serialize;
use std::{thread::sleep, time::Duration};
use z00z_utils::codec::Value;
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{create_dir_all, path_exists, read_to_string, save_json, write_file},
};

use crate::{
    config::{Stage3ClaimCfg, Stage3PathsCfg},
    scenario_1::claim_pkg_consumer::{publish_claims_store, ClaimStorePublishSummary},
    scenario_1::stage_4::{export_claim_post_view, publish_genesis_rights},
    DesignStage, SimContext, StageResult,
};
use z00z_storage::settlement::SettlementStore;

const STAGE4_INPUT_WAIT_RETRIES: u32 = 200;
const STAGE4_INPUT_WAIT_MS: u64 = 50;

#[derive(Serialize)]
struct PublishLogRow {
    stage: u32,
    step: String,
    event: String,
    status: String,
    detail: String,
}

pub fn run_claim_publish(ctx: &mut SimContext, stage: &DesignStage) -> StageResult {
    match run_publish(ctx, stage.stage) {
        Ok(()) => StageResult::Ok,
        Err(err) => StageResult::Fail(format!(
            "stage {} ({}) failed: {}",
            stage.stage, stage.name, err
        )),
    }
}

fn claim_publish_cfg(ctx: &SimContext) -> Stage3ClaimCfg {
    ctx.config.stage4_claim_publish.clone().unwrap_or_else(|| {
        let mut cfg = ctx.config.stage3_claim.clone().unwrap_or(Stage3ClaimCfg {
            active: Some("uniform_all".to_string()),
            rng_seed: None,
            consume_bins: Some(true),
            snapshot_fault: None,
            resume_fault: None,
            paths: Stage3PathsCfg::default(),
        });
        cfg.consume_bins = Some(false);
        cfg.paths.claim_dir = "claim_publish".to_string();
        cfg.paths.wallets_dir = "wallets_publish".to_string();
        cfg.paths.events_dir = "events_publish".to_string();
        cfg.paths.logs_dir = "logs_publish".to_string();
        cfg.paths.export_dir = "wallets_publish_export".to_string();
        cfg.paths.snapshot_file = "stage_4_snapshot.json".to_string();
        cfg.paths.claim_state_file = "claim_publish_state.json".to_string();
        cfg.paths.logger_file = "claim_publish_logger.json".to_string();
        cfg.paths.rpc_logger_file = "rpc_publish_logger.json".to_string();
        cfg
    })
}

fn create_publish_dirs(
    ctx: &SimContext,
    cfg: &Stage3ClaimCfg,
) -> Result<(std::path::PathBuf, std::path::PathBuf), String> {
    let publish_dir = ctx.outputs_dir.join(&cfg.paths.claim_dir);
    let logs_dir = ctx.outputs_dir.join(&cfg.paths.logs_dir);
    let events_dir = ctx.outputs_dir.join(&cfg.paths.events_dir);
    let wallets_dir = ctx.outputs_dir.join(&cfg.paths.wallets_dir);
    let export_dir = ctx.outputs_dir.join(&cfg.paths.export_dir);

    create_dir_all(&publish_dir).map_err(|e| e.to_string())?;
    create_dir_all(&logs_dir).map_err(|e| e.to_string())?;
    create_dir_all(&events_dir).map_err(|e| e.to_string())?;
    create_dir_all(&wallets_dir).map_err(|e| e.to_string())?;
    create_dir_all(&export_dir).map_err(|e| e.to_string())?;

    Ok((publish_dir, logs_dir))
}

fn read_prepare_snapshot(
    ctx: &SimContext,
    prepare_paths: &Stage3PathsCfg,
) -> Result<Value, String> {
    let prepare_snapshot = ctx.outputs_dir.join(&prepare_paths.snapshot_file);
    if !wait_for_existing_path(&prepare_snapshot).map_err(|e| e.to_string())? {
        return Err(format!("missing {}", prepare_snapshot.display()));
    }

    JsonCodec
        .deserialize(
            read_to_string(&prepare_snapshot)
                .map_err(|e| e.to_string())?
                .as_bytes(),
        )
        .map_err(|e| format!("invalid {}: {e}", prepare_paths.snapshot_file))
}

fn wallet_stats_len(snapshot: &Value, snapshot_file: &str) -> Result<usize, String> {
    let wallet_stats = snapshot
        .get("wallet_import_stats")
        .ok_or_else(|| format!("{snapshot_file} missing wallet_import_stats"))?
        .as_array()
        .ok_or_else(|| format!("{snapshot_file} wallet_import_stats must be an array"))?;

    if wallet_stats.is_empty() {
        return Err(format!(
            "{snapshot_file} wallet_import_stats must not be empty"
        ));
    }

    Ok(wallet_stats.len())
}

fn load_wallet_stats(ctx: &SimContext, prepare_paths: &Stage3PathsCfg) -> Result<usize, String> {
    let source_snapshot = read_prepare_snapshot(ctx, prepare_paths)?;
    wallet_stats_len(&source_snapshot, &prepare_paths.snapshot_file)
}

fn write_publish_snapshot(
    ctx: &SimContext,
    cfg: &Stage3ClaimCfg,
    stage_id: u32,
    source_snapshot_file: &str,
    wallet_stats: usize,
    claim_pub: &ClaimStorePublishSummary,
) -> Result<(), String> {
    let stage1_paths = ctx.config.stage1_paths();
    let summary = z00z_utils::codec::json!({
        "stage": stage_id,
        "source_stage": 3,
        "source_snapshot_file": source_snapshot_file,
        "wallet_import_stats": wallet_stats,
        "package_count": claim_pub.package_count,
        "inserted_count": claim_pub.inserted_count,
        "genesis_rights_count": ctx.genesis_rights.len(),
        "genesis_rights_included": true,
        "rights_artifact_file": z00z_core::genesis::GENESIS_RIGHTS_FILE,
        "settlement_manifest_file": z00z_core::genesis::GENESIS_SETTLEMENT_MANIFEST_FILE,
        "genesis_dir": stage1_paths.genesis_dir,
        "status": "published",
    });

    let snapshot_path = ctx.outputs_dir.join(&cfg.paths.snapshot_file);
    save_json(snapshot_path, &summary).map_err(|e| e.to_string())
}

fn push_publish_log(
    lines: &mut Vec<String>,
    stage_id: u32,
    step: &str,
    event: &str,
    detail: &str,
) -> Result<(), String> {
    let row = PublishLogRow {
        stage: stage_id,
        step: step.to_string(),
        event: event.to_string(),
        status: "ok".to_string(),
        detail: detail.to_string(),
    };
    let bytes = JsonCodec.serialize(&row).map_err(|e| e.to_string())?;
    let line = String::from_utf8(bytes).map_err(|e| e.to_string())?;
    lines.push(line);
    Ok(())
}

fn flush_publish_logs(
    logs_dir: &std::path::Path,
    cfg: &Stage3ClaimCfg,
    lines: &[String],
) -> Result<(), String> {
    let log_path = logs_dir.join(&cfg.paths.logger_file);
    write_file(&log_path, format!("{}\n", lines.join("\n")).as_bytes()).map_err(|e| e.to_string())
}

fn write_publish_audit(
    publish_dir: &std::path::Path,
    stage_id: u32,
    source_snapshot_file: &str,
    genesis_rights_count: usize,
) -> Result<(), String> {
    let audit_path = publish_dir.join("audit_log.json");
    save_json(
        audit_path,
        &z00z_utils::codec::json!({
            "stage": stage_id,
            "source_snapshot_file": source_snapshot_file,
            "genesis_rights_count": genesis_rights_count,
            "genesis_rights_included": true,
            "status": "ok",
        }),
    )
    .map_err(|e| e.to_string())
}

fn publish_claim_pkg(
    ctx: &SimContext,
    claim_pkg_path: &std::path::Path,
    claim_pub_path: &std::path::Path,
) -> Result<ClaimStorePublishSummary, String> {
    if !wait_for_existing_path(claim_pkg_path).map_err(|e| e.to_string())? {
        return Err(format!("missing {}", claim_pkg_path.display()));
    }

    let mut claim_store =
        SettlementStore::try_new().map_err(|e| format!("claim publish store open failed: {e}"))?;
    let claim_pub = publish_claims_store(claim_pkg_path, &mut claim_store)
        .map_err(|e| format!("claim publish failed: {e}"))?;
    publish_genesis_rights(&mut claim_store, &ctx.genesis_rights)?;
    save_json(claim_pub_path, &claim_pub).map_err(|e| e.to_string())?;
    export_claim_post_view(&ctx.outputs_dir, &claim_store)?;
    Ok(claim_pub)
}

fn wait_for_existing_path(path: &std::path::Path) -> Result<bool, String> {
    for attempt in 0..=STAGE4_INPUT_WAIT_RETRIES {
        if path_exists(path).map_err(|e| e.to_string())? {
            return Ok(true);
        }
        if attempt < STAGE4_INPUT_WAIT_RETRIES {
            sleep(Duration::from_millis(STAGE4_INPUT_WAIT_MS));
        }
    }
    Ok(false)
}

fn log_publish_dirs(
    lines: &mut Vec<String>,
    stage_id: u32,
    publish_dir: &std::path::Path,
    logs_dir: &std::path::Path,
) -> Result<(), String> {
    push_publish_log(
        lines,
        stage_id,
        "P4-1",
        "prepare_publish_dirs",
        &format!(
            "publish_dir={} logs_dir={}",
            publish_dir.display(),
            logs_dir.display()
        ),
    )
}

fn log_prepare_snapshot(
    lines: &mut Vec<String>,
    stage_id: u32,
    prepare_paths: &Stage3PathsCfg,
    wallet_stats: usize,
) -> Result<(), String> {
    push_publish_log(
        lines,
        stage_id,
        "P4-2",
        "load_claim_prepare_snapshot",
        &format!(
            "source_snapshot={} wallet_import_stats={wallet_stats}",
            prepare_paths.snapshot_file
        ),
    )
}

fn start_publish(
    ctx: &SimContext,
    cfg: &Stage3ClaimCfg,
    prepare_paths: &Stage3PathsCfg,
    stage_id: u32,
    lines: &mut Vec<String>,
) -> Result<(std::path::PathBuf, std::path::PathBuf, usize), String> {
    let (publish_dir, logs_dir) = create_publish_dirs(ctx, cfg)?;
    log_publish_dirs(lines, stage_id, &publish_dir, &logs_dir)?;
    let wallet_stats = load_wallet_stats(ctx, prepare_paths)?;
    log_prepare_snapshot(lines, stage_id, prepare_paths, wallet_stats)?;
    Ok((publish_dir, logs_dir, wallet_stats))
}

fn finish_publish(
    ctx: &SimContext,
    cfg: &Stage3ClaimCfg,
    prepare_paths: &Stage3PathsCfg,
    publish_dir: &std::path::Path,
    logs_dir: &std::path::Path,
    stage_id: u32,
    lines: &mut Vec<String>,
    wallet_stats: usize,
    claim_pub: &ClaimStorePublishSummary,
) -> Result<(), String> {
    write_publish_snapshot(
        ctx,
        cfg,
        stage_id,
        &prepare_paths.snapshot_file,
        wallet_stats,
        claim_pub,
    )?;
    write_publish_audit(
        publish_dir,
        stage_id,
        &prepare_paths.snapshot_file,
        ctx.genesis_rights.len(),
    )?;
    push_publish_log(
        lines,
        stage_id,
        "P4-4",
        "write_publish_artifacts",
        &format!(
            "snapshot={} audit={}",
            ctx.outputs_dir.join(&cfg.paths.snapshot_file).display(),
            publish_dir.join("audit_log.json").display()
        ),
    )?;
    flush_publish_logs(logs_dir, cfg, lines)
}

fn claim_publish_paths(
    ctx: &SimContext,
    _prepare_paths: &Stage3PathsCfg,
) -> (std::path::PathBuf, std::path::PathBuf) {
    (
        crate::scenario_1::stage_4::resolve_stage3_claim_pkg_file(ctx),
        crate::scenario_1::stage_4::resolve_stage3_claim_pub_file(ctx),
    )
}

fn log_claim_publish(
    ctx: &SimContext,
    stage_id: u32,
    lines: &mut Vec<String>,
    claim_pkg_path: &std::path::Path,
    claim_pub_path: &std::path::Path,
    claim_pub: &ClaimStorePublishSummary,
) -> Result<(), String> {
    push_publish_log(
        lines,
        stage_id,
        "P4-3",
        "publish_claim_store",
        &format!(
            "claim_pkg={} packages={} inserted={} rights={} pub={}",
            claim_pkg_path.display(),
            claim_pub.package_count,
            claim_pub.inserted_count,
            ctx.genesis_rights.len(),
            claim_pub_path.display()
        ),
    )?;
    ctx.logger.info(&format!(
        "stage4.claim_pub_done: file={} packages={} inserted={} rights={}",
        claim_pub_path.display(),
        claim_pub.package_count,
        claim_pub.inserted_count,
        ctx.genesis_rights.len()
    ));
    Ok(())
}

fn run_publish(ctx: &SimContext, stage_id: u32) -> Result<(), String> {
    let cfg = claim_publish_cfg(ctx);
    let prepare_paths = ctx.config.stage3_paths();
    let mut lines = Vec::new();
    let (publish_dir, logs_dir, wallet_stats) =
        start_publish(ctx, &cfg, &prepare_paths, stage_id, &mut lines)?;
    let (claim_pkg_path, claim_pub_path) = claim_publish_paths(ctx, &prepare_paths);
    let claim_pub = publish_claim_pkg(ctx, &claim_pkg_path, &claim_pub_path)?;
    log_claim_publish(
        ctx,
        stage_id,
        &mut lines,
        &claim_pkg_path,
        &claim_pub_path,
        &claim_pub,
    )?;
    finish_publish(
        ctx,
        &cfg,
        &prepare_paths,
        &publish_dir,
        &logs_dir,
        stage_id,
        &mut lines,
        wallet_stats,
        &claim_pub,
    )
}
