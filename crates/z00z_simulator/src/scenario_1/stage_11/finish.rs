use std::path::{Path, PathBuf};

use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{create_dir_all, path_exists, read_to_string, save_json, write_file},
    time::{format_system_time_local, SystemTimeProvider, TimeProvider},
};

use crate::scenario_1::stage_11::Stage11Checkpoint;
use crate::DesignStage;

#[derive(serde::Serialize)]
struct LogRow {
    timestamp: String,
    stage: u32,
    step: String,
    event: String,
    status: String,
    detail: String,
}

pub(super) fn write_stage11(
    tx_dir: &Path,
    file: &str,
    summary: &Stage11Checkpoint,
) -> Result<PathBuf, String> {
    let cp_path = tx_dir.join(file);
    save_json(&cp_path, summary).map_err(|e| e.to_string())?;
    Ok(cp_path)
}

pub(super) fn fill_steps(
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

pub(super) fn prep_dirs(out: &Path, logs_dir: &Path, tx_dir: &Path) -> Result<(), String> {
    create_dir_all(out).map_err(|e| e.to_string())?;
    create_dir_all(logs_dir).map_err(|e| e.to_string())?;
    create_dir_all(tx_dir).map_err(|e| e.to_string())?;
    Ok(())
}

pub(super) fn log_step(
    lines: &mut Vec<String>,
    stage_id: u32,
    step_id: &str,
    event: &str,
    detail: &str,
) -> Result<(), String> {
    push_log(lines, stage_id, step_id, event, "ok", detail)?;
    Ok(())
}

pub(super) fn flush_logs(path: &Path, lines: &[String]) -> Result<(), String> {
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
