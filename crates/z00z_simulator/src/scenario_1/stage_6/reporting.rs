use serde::Serialize;
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{path_exists, read_to_string, write_file},
    time::{format_system_time_local, SystemTimeProvider, TimeProvider},
};

#[derive(Serialize)]
struct LogRow {
    timestamp: String,
    stage: u32,
    step: String,
    event: String,
    status: String,
    detail: String,
}

#[derive(Serialize)]
pub(crate) struct Stage4Snap {
    pub(crate) stage: u32,
    pub(crate) tx_count: u32,
    pub(crate) output_count: u32,
    pub(crate) tx_digest_hex: String,
    pub(crate) status: String,
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
