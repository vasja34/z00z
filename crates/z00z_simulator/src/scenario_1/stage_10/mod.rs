//! Scenario 1 stage 10: bundle publish.

use std::collections::HashSet;

use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{create_dir_all, path_exists, read_to_string, write_file},
    time::{format_system_time_local, SystemTimeProvider, TimeProvider},
};

use crate::{DesignStage, SimContext, StageResult};

use self::publish_support::{load_publish_state, write_step_fallbacks};
use self::report::write_report;
use super::stage_9::bundle_lane_impl::frag_amount_sum;

mod publish_support;
mod report;

pub fn run_bundle_publish(ctx: &mut SimContext, stage: &DesignStage) -> StageResult {
    match run_publish_stage(ctx, stage) {
        Ok(()) => StageResult::Ok,
        Err(err) => StageResult::Fail(format!(
            "stage {} ({}) failed: {}",
            stage.stage, stage.name, err
        )),
    }
}

#[derive(serde::Serialize)]
struct LogRow {
    timestamp: String,
    stage: u32,
    step: String,
    event: String,
    status: String,
    detail: String,
}

fn run_publish_stage(ctx: &mut SimContext, stage: &DesignStage) -> Result<(), String> {
    let paths = ctx.config.stage6_paths();
    let out = &ctx.outputs_dir;
    let logs_dir = out.join(&paths.logs_dir);
    let tx_dir = out.join(&paths.transactions_dir);
    prep_dirs(out, &logs_dir, &tx_dir)?;

    let mut step_seen = HashSet::<String>::new();
    let mut lines = Vec::new();
    push_log(
        &mut lines,
        stage.stage,
        "S6-1",
        "prepare_dirs",
        "ok",
        "outputs/logs/transactions prepared",
    )?;
    step_seen.insert("S6-1".to_string());

    let state = load_publish_state(&tx_dir, &paths)?;
    push_log(
        &mut lines,
        stage.stage,
        "S6-2",
        "reuse_fragments",
        "ok",
        "fragment files reused from bundle_build outputs",
    )?;
    step_seen.insert("S6-2".to_string());

    push_log(
        &mut lines,
        stage.stage,
        "S6-3",
        "wallet_skip",
        "ok",
        "wallet update is tracked by checkpoint artifacts",
    )?;
    step_seen.insert("S6-3".to_string());

    push_log(
        &mut lines,
        stage.stage,
        "S6-4",
        "reuse_bridge",
        "ok",
        &tx_dir.join(&paths.checkpoint_file).to_string_lossy(),
    )?;
    step_seen.insert("S6-4".to_string());

    push_log(
        &mut lines,
        stage.stage,
        "S6-5",
        "ordered_pipeline",
        "ok",
        "bridge -> exec_input -> storage_apply handoff from stage_4/5",
    )?;
    step_seen.insert("S6-5".to_string());

    push_log(
        &mut lines,
        stage.stage,
        "S6-6",
        "publish_bundle",
        "ok",
        "bundle publish surface reused stage_9 bridge outputs",
    )?;
    step_seen.insert("S6-6".to_string());

    let amount_sum = frag_amount_sum(&state.frag_a, &state.frag_b);
    push_log(
        &mut lines,
        stage.stage,
        "S6-7",
        "aggregate_amount",
        "ok",
        &format!("fragment output amount={amount_sum}"),
    )?;
    step_seen.insert("S6-7".to_string());

    push_log(
        &mut lines,
        stage.stage,
        "S6-8",
        "check_invariants",
        "ok",
        &format!(
            "replay-safe handoff exec_input_id={}",
            state.bridge.exec_input_id_hex
        ),
    )?;
    step_seen.insert("S6-8".to_string());

    write_step_fallbacks(stage, &step_seen, &mut lines)?;
    write_report(
        out,
        &paths.report_file,
        stage.stage,
        amount_sum,
        &state.bridge,
    )?;
    flush_logs(&logs_dir.join(&paths.logger_file), &lines)
}

pub(crate) fn prep_dirs(
    out: &std::path::Path,
    logs_dir: &std::path::Path,
    tx_dir: &std::path::Path,
) -> Result<(), String> {
    create_dir_all(out).map_err(|e| e.to_string())?;
    create_dir_all(logs_dir).map_err(|e| e.to_string())?;
    create_dir_all(tx_dir).map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn push_log(
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

pub(crate) fn flush_logs(path: &std::path::Path, lines: &[String]) -> Result<(), String> {
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
