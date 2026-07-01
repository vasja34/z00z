use std::path::PathBuf;

use z00z_core::{assets::AssetClass, genesis::validator::verify_genesis_assets};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{path_exists, save_bincode, write_file},
    time::{format_system_time_local, SystemTimeProvider, TimeProvider},
};

use super::LogRow;

pub(super) fn resolve_genesis_cfg_path(path: &str, fallback_dir: &str) -> String {
    if path.is_empty() {
        return PathBuf::from(fallback_dir)
            .join(z00z_core::config_paths::DEVNET_GENESIS_CONFIG)
            .to_string_lossy()
            .to_string();
    }

    let direct = PathBuf::from(path);
    if direct.exists() {
        return path.to_string();
    }

    let fallback = PathBuf::from(fallback_dir).join(path);
    if fallback.exists() {
        return fallback.to_string_lossy().to_string();
    }

    path.to_string()
}

pub(super) fn run_cli_checks(
    out_gen: &std::path::Path,
    all_assets: &[z00z_core::Asset],
    logs: &mut Vec<String>,
    stage_id: u32,
) -> Result<(), String> {
    let report_file = out_gen.join("genesis_analysis_genesis.md");
    let report_body = build_analyze_report(all_assets);
    write_file(&report_file, report_body.as_bytes()).map_err(|e| e.to_string())?;
    push_log(
        logs,
        stage_id,
        "S1-8",
        "assets_analyzer_report",
        "ok",
        &report_file.to_string_lossy(),
    )?;

    let extract_file = out_gen.join("assets_extract_coins_0_99.bin");
    let mut coin_rows: Vec<_> = all_assets
        .iter()
        .filter(|asset| asset.definition.class == AssetClass::Coin && asset.serial_id <= 99)
        .cloned()
        .collect();
    coin_rows.sort_by_key(|asset| asset.serial_id);
    save_bincode(&extract_file, &coin_rows).map_err(|e| e.to_string())?;
    push_log(
        logs,
        stage_id,
        "S1-8",
        "assets_extractor_bin",
        "ok",
        &extract_file.to_string_lossy(),
    )?;

    if !path_exists(&report_file).map_err(|e| e.to_string())? {
        return Err("stage-1 analyzer report is missing".to_string());
    }

    if !path_exists(&extract_file).map_err(|e| e.to_string())? {
        return Err("stage-1 extractor output is missing".to_string());
    }
    push_log(
        logs,
        stage_id,
        "S1-8",
        "cli_outputs",
        "ok",
        "analyzer + extractor outputs exist",
    )?;

    Ok(())
}

fn build_analyze_report(all_assets: &[z00z_core::Asset]) -> String {
    let mut coin_count = 0usize;
    let mut token_count = 0usize;
    let mut nft_count = 0usize;
    let mut void_count = 0usize;

    let mut coin_sum = 0u128;
    let mut token_sum = 0u128;
    let mut nft_sum = 0u128;
    let mut void_sum = 0u128;

    for row in all_assets {
        match row.definition.class {
            AssetClass::Coin => {
                coin_count += 1;
                coin_sum += u128::from(row.amount);
            }
            AssetClass::Token => {
                token_count += 1;
                token_sum += u128::from(row.amount);
            }
            AssetClass::Nft => {
                nft_count += 1;
                nft_sum += u128::from(row.amount);
            }
            AssetClass::Void => {
                void_count += 1;
                void_sum += u128::from(row.amount);
            }
        }
    }

    let total_count = coin_count + token_count + nft_count + void_count;

    let mut body = String::new();
    body.push_str("# Genesis Assets Analysis\n\n");
    body.push_str("This report is generated in-process by scenario stage 1 without spawning cargo subprocesses.\n\n");
    body.push_str("| Class | Count | Total Amount |\n");
    body.push_str("|---|---:|---:|\n");
    body.push_str(&format!("| Coin | {} | {} |\n", coin_count, coin_sum));
    body.push_str(&format!("| Token | {} | {} |\n", token_count, token_sum));
    body.push_str(&format!("| Nft | {} | {} |\n", nft_count, nft_sum));
    body.push_str(&format!("| Void | {} | {} |\n", void_count, void_sum));
    body.push_str(&format!("| TOTAL | {} | - |\n\n", total_count));

    body.push_str("## Sample Rows\n\n");
    body.push_str("| Symbol | Class | Serial Id | Amount | Asset Id Prefix |\n");
    body.push_str("|---|---|---:|---:|---|\n");
    for row in all_assets.iter().take(64) {
        let id_hex = hex_str(&row.definition.id);
        let short_id = &id_hex[..8];
        body.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            row.definition.symbol, row.definition.class, row.serial_id, row.amount, short_id,
        ));
    }

    body
}

pub(super) fn push_log(
    logs: &mut Vec<String>,
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
    let codec = JsonCodec;
    let bytes = codec.serialize(&row).map_err(|e| e.to_string())?;
    let line = String::from_utf8(bytes).map_err(|e| e.to_string())?;
    logs.push(line);
    Ok(())
}

pub(super) fn flush_logs(path: &std::path::Path, logs: &[String]) -> Result<(), String> {
    let mut body = logs.join("\n");
    body.push('\n');
    write_file(path, body.as_bytes()).map_err(|e| e.to_string())
}

pub(super) fn verify_assets_all(assets: &[z00z_core::Asset]) -> Result<(), String> {
    match verify_genesis_assets(assets) {
        Ok(()) => Ok(()),
        Err(err) => {
            let msg = err.to_string();
            if !msg.contains("Batch too large") {
                return Err(msg);
            }

            let chunk_size = 1000usize;
            for chunk in assets.chunks(chunk_size) {
                verify_genesis_assets(chunk).map_err(|e| e.to_string())?;
            }
            Ok(())
        }
    }
}

pub(super) fn hex_str(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(nib_hex(byte >> 4));
        out.push(nib_hex(byte & 0x0f));
    }
    out
}

#[inline(always)]
fn nib_hex(nib: u8) -> char {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    HEX[nib as usize] as char
}
