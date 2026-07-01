//! Scenario 1 stage 12: checkpoint finalize.
//!
//! Finalization stays on the current package-coupled continuity path and does not
//! upgrade the accepted package-coupled continuity path into standalone checkpoint authority.
//! Detached proof bytes remain insufficient by themselves, and any prior-final exports stay
//! noncanonical so they cannot blur the canonical draft/final checkpoint class boundary.

use serde::{Deserialize, Serialize};
use std::path::Path;

use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{create_dir_all, path_exists, read_to_string, save_json, write_file},
    time::{format_system_time_local, SystemTimeProvider, TimeProvider},
};

use crate::{config::Stage6ProofMode, DesignStage, SimContext, StageResult};

use super::stage_11::Stage11Checkpoint;

mod final_refs;
mod finalize_flow;

use self::final_refs::load_stage11_checkpoint;
use self::finalize_flow::finalize_stage12;

struct Stage12Cfg {
    p6: crate::config::Stage6PathsCfg,
    p7: crate::config::Stage7PathsCfg,
    p8: crate::config::Stage8PathsCfg,
    proof_mode: Stage6ProofMode,
    s4: crate::config::Stage4TxPrepareCfg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Stage12Summary {
    stage: u32,
    draft_id_hex: String,
    exec_input_id_hex: String,
    snapshot_id_hex: String,
    evidence_class: String,
    checkpoint_id_hex: Option<String>,
    artifact_path: Option<String>,
    link_path: Option<String>,
    audit_path: Option<String>,
    fragment_ids: Vec<String>,
    status: String,
}

#[derive(Serialize)]
struct LogRow {
    timestamp: String,
    stage: u32,
    step: String,
    event: String,
    status: String,
    detail: String,
}

pub fn run_finalize(ctx: &mut SimContext, stage: &DesignStage) -> StageResult {
    match run_core(ctx, stage) {
        Ok(()) => StageResult::Ok,
        Err(err) => StageResult::Fail(format!(
            "stage {} ({}) failed: {}",
            stage.stage, stage.name, err
        )),
    }
}

fn run_core(ctx: &mut SimContext, stage: &DesignStage) -> Result<(), String> {
    let cfg = load_cfg(ctx)?;
    let out = &ctx.outputs_dir;
    let logs_dir = out.join(&cfg.p8.logs_dir);
    let tx_dir = out.join(&cfg.p8.transactions_dir);
    prep_dirs(out, &logs_dir, &tx_dir)?;

    let mut lines = Vec::new();

    log_step(
        &mut lines,
        stage.stage,
        "S8-1",
        "prepare_dirs",
        "outputs/logs/transactions prepared",
    )?;

    let checkpoint_path = tx_dir.join(&cfg.p7.checkpoint_file);
    let checkpoint = load_stage11_checkpoint(&checkpoint_path)?;

    log_step(
        &mut lines,
        stage.stage,
        "S8-2",
        "load_stage11_checkpoint",
        &checkpoint_path.to_string_lossy(),
    )?;

    let mut summary = build_summary(stage.stage, &checkpoint, cfg.proof_mode);
    maybe_finalize(out, &tx_dir, &cfg, &checkpoint, &mut summary)?;

    log_step(
        &mut lines,
        stage.stage,
        "S8-3",
        "finalize_checkpoint",
        &summary.status,
    )?;

    let out_path = tx_dir.join(&cfg.p8.checkpoint_file);
    save_json(&out_path, &summary).map_err(|e| e.to_string())?;

    log_step(
        &mut lines,
        stage.stage,
        "S8-4",
        "write_summary",
        &out_path.to_string_lossy(),
    )?;

    fill_steps(&mut lines, stage, &["S8-1", "S8-2", "S8-3", "S8-4"])?;

    flush_logs(&logs_dir.join(&cfg.p8.logger_file), &lines)?;
    Ok(())
}

fn load_cfg(ctx: &SimContext) -> Result<Stage12Cfg, String> {
    Ok(Stage12Cfg {
        p6: ctx.config.stage6_paths(),
        p7: ctx.config.stage7_paths(),
        p8: ctx.config.stage8_paths(),
        proof_mode: ctx.config.stage6_proof_mode(),
        s4: ctx
            .config
            .stage4_tx_prepare
            .clone()
            .ok_or_else(|| "stage4_tx_prepare config missing".to_string())?,
    })
}

fn build_summary(
    stage_id: u32,
    checkpoint: &Stage11Checkpoint,
    proof_mode: Stage6ProofMode,
) -> Stage12Summary {
    Stage12Summary {
        stage: stage_id,
        draft_id_hex: checkpoint.draft_id_hex.clone(),
        exec_input_id_hex: checkpoint.exec_input_id_hex.clone(),
        snapshot_id_hex: checkpoint.snapshot_id_hex.clone(),
        evidence_class: proof_mode.stage12_evidence_class().to_string(),
        checkpoint_id_hex: None,
        artifact_path: None,
        link_path: None,
        audit_path: None,
        fragment_ids: checkpoint.fragment_ids.clone(),
        status: "draft_only".to_string(),
    }
}

fn maybe_finalize(
    out: &Path,
    tx_dir: &Path,
    cfg: &Stage12Cfg,
    checkpoint: &Stage11Checkpoint,
    summary: &mut Stage12Summary,
) -> Result<(), String> {
    if !cfg.proof_mode.allows_public_checkpoint_evidence() {
        // Draft-only runs stop at a private checkpoint summary and must not
        // spill into the public checkpoint/publication contract.
        return Ok(());
    }
    finalize_stage12(out, tx_dir, cfg, checkpoint, summary)
}

fn prep_dirs(out: &Path, logs_dir: &Path, tx_dir: &Path) -> Result<(), String> {
    create_dir_all(out).map_err(|e| e.to_string())?;
    create_dir_all(logs_dir).map_err(|e| e.to_string())?;
    create_dir_all(tx_dir).map_err(|e| e.to_string())?;
    Ok(())
}

fn log_step(
    lines: &mut Vec<String>,
    stage_id: u32,
    step_id: &str,
    event: &str,
    detail: &str,
) -> Result<(), String> {
    push_log(lines, stage_id, step_id, event, "ok", detail)
}

fn fill_steps(
    lines: &mut Vec<String>,
    stage: &DesignStage,
    covered: &[&str],
) -> Result<(), String> {
    let mut missing = stage
        .steps
        .iter()
        .filter(|step| !covered.contains(&step.id.as_str()))
        .map(|step| step.id.clone())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        missing.sort();
        return Err(format!(
            "stage {} missing canonical coverage for steps: {}",
            stage.stage,
            missing.join(", ")
        ));
    }
    let _ = lines;
    Ok(())
}

pub fn run(ctx: &mut SimContext, stage: &DesignStage) -> StageResult {
    run_finalize(ctx, stage)
}

fn push_log(
    lines: &mut Vec<String>,
    stage: u32,
    step: &str,
    event: &str,
    status: &str,
    detail: &str,
) -> Result<(), String> {
    let row = LogRow {
        timestamp: format_system_time_local(SystemTimeProvider.now()),
        stage,
        step: step.to_string(),
        event: event.to_string(),
        status: status.to_string(),
        detail: detail.to_string(),
    };
    let bytes = JsonCodec.serialize(&row).map_err(|e| e.to_string())?;
    let line = String::from_utf8(bytes).map_err(|e| e.to_string())?;
    lines.push(line);
    Ok(())
}

fn flush_logs(path: &Path, lines: &[String]) -> Result<(), String> {
    let mut body = String::new();
    if path_exists(path).map_err(|e| e.to_string())? {
        body = read_to_string(path).map_err(|e| e.to_string())?;
        if !body.is_empty() && !body.ends_with('\n') {
            body.push('\n');
        }
    }
    body.push_str(&lines.join("\n"));
    body.push('\n');
    write_file(path, body.as_bytes()).map_err(|e| e.to_string())
}
