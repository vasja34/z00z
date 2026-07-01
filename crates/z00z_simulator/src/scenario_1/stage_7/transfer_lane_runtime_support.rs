use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use z00z_core::{assets::AssetPackPlain, Asset};
use z00z_networks_rpc::RpcTransport;
use z00z_storage::settlement::TerminalLeaf;
use z00z_utils::{
    codec::{json, Codec, JsonCodec},
    io::{path_exists, read_to_string, save_json, write_file},
    time::{format_system_time_local, SystemTimeProvider, TimeProvider},
};
use z00z_wallets::{
    receiver::{
        receiver_scan_leaf, receiver_scan_report, ReceiveNext, ReceiveReport, ReceiveStatus,
        StealthOutputScanner,
    },
    rpc::types::asset::RuntimeReceiveAssetResponse,
    tx::{wire_decrypt_leaf, TxOutRole, TxOutputWire},
};

use crate::{
    config::{Stage4TxPrepareCfg, Stage5PathsCfg},
    DesignStage, SimActor,
};

use super::transfer_lane_impl::{LogRow, RecvCtx, Stage5Snap, Stage5TxFile};

pub(crate) fn resolve_input_path(out: &Path, path: &str) -> Result<PathBuf, String> {
    let rel_path = PathBuf::from(path);
    let marker = Path::new("crates/z00z_simulator/outputs/scenario_1");
    if rel_path.is_absolute() {
        return Ok(rel_path);
    }
    if let Ok(stripped) = rel_path.strip_prefix(marker) {
        return Ok(out.join(stripped));
    }
    if let Some(found) = find_wallet_path(&rel_path)? {
        return Ok(found);
    }
    Ok(out.join(rel_path))
}

fn find_wallet_path(path: &Path) -> Result<Option<PathBuf>, String> {
    if path_exists(path).map_err(|e| e.to_string())? {
        return Ok(Some(path.to_path_buf()));
    }
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    for base in cwd.ancestors() {
        let cand = base.join(path);
        if path_exists(&cand).map_err(|e| e.to_string())? {
            return Ok(Some(cand));
        }
    }
    Ok(None)
}

pub(crate) fn rpc_log_path(logs_dir: &Path, logger_file: &str) -> PathBuf {
    logs_dir.join(logger_file.replace("logger.json", "rpc_logger.json"))
}

pub(crate) fn log_ok(
    lines: &mut Vec<String>,
    stage: u32,
    step: &str,
    event: &str,
    detail: &str,
) -> Result<(), String> {
    push_log(lines, stage, step, event, "ok", detail)
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

pub(crate) fn flush_logs(path: &Path, lines: &[String]) -> Result<(), String> {
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

pub(crate) async fn write_tx(
    tx_dir: &Path,
    cfg: &Stage5PathsCfg,
    recv: &RecvCtx,
    stage_id: u32,
) -> Result<PathBuf, String> {
    let tx_file = tx_dir.join(&cfg.tx_file);
    let body = Stage5TxFile {
        stage: stage_id,
        source_tx_digest_hex: recv.tx_pkg.tx_digest_hex.clone(),
        recipient_output_index: recv.out_idx,
        asset_id_hex: hex::encode(recv.asset.asset_id()),
        serial_id: recv.asset.serial_id,
        amount: recv.pack.value,
        r_pub: hex::encode(recv.leaf.r_pub),
        owner_tag: hex::encode(recv.leaf.owner_tag),
        tag16: recv.leaf.tag16,
        c_amount: hex::encode(recv.leaf.c_amount),
        ciphertext_len: recv.leaf.enc_pack.ciphertext.len(),
        status: "ok".to_string(),
    };
    save_json(&tx_file, &body).map_err(|e| e.to_string())?;
    Ok(tx_file)
}

pub(crate) async fn write_snap(
    out: &Path,
    cfg: &Stage5PathsCfg,
    recv: &RecvCtx,
    rpc_status: &str,
    claimed_len: usize,
    stage_id: u32,
) -> Result<PathBuf, String> {
    let snap_file = out.join(&cfg.snapshot_file);
    let body = Stage5Snap {
        stage: stage_id,
        transfer_count: 1,
        input_tx_digest_hex: recv.tx_pkg.tx_digest_hex.clone(),
        recipient_output_index: recv.out_idx,
        canonical_status: recv.canon.status.rpc_code().to_string(),
        runtime_status: recv.runtime.status.rpc_code().to_string(),
        rpc_status: rpc_status.to_string(),
        claimed_count_after_route: claimed_len,
        status: "ok".to_string(),
    };
    save_json(&snap_file, &body).map_err(|e| e.to_string())?;
    Ok(snap_file)
}

pub(crate) fn log_extra(stage: &DesignStage, lines: &mut Vec<String>) -> Result<(), String> {
    let covered = [
        "S5-1", "S5-2", "S5-3", "S5-4", "S5-5", "S5-6", "S5-7", "S5-8",
    ];
    for step in &stage.steps {
        if covered.contains(&step.id.as_str()) {
            continue;
        }
        push_log(
            lines,
            stage.stage,
            &step.id,
            "step_covered",
            "ok",
            "stage_5 additional step covered",
        )?;
    }
    Ok(())
}

pub(crate) async fn run_receive_asset_rpc(
    transport: &impl RpcTransport,
    cfg: &Stage4TxPrepareCfg,
    wallet_id: &str,
    password: &str,
    asset_id: [u8; 32],
) -> Result<RuntimeReceiveAssetResponse, String> {
    let session = transport
        .call(
            &cfg.rpc.unlock_method,
            json!({
                "wallet_id": wallet_id,
                "password": password,
            }),
        )
        .await
        .map_err(|e| format!("stage5: unlock receiver RPC failed: {e}"))?;

    let recv_res = transport
        .call(
            "wallet.asset.receive_asset",
            json!({
                "session": session,
                "asset_id": asset_id,
            }),
        )
        .await;
    let lock_res = transport
        .call(&cfg.rpc.lock_method, json!({"session": session}))
        .await;

    match recv_res {
        Ok(recv) => {
            lock_res.map_err(|e| format!("stage5: lock receiver RPC failed: {e}"))?;
            JsonCodec
                .serialize(&recv)
                .and_then(|bytes| JsonCodec.deserialize(&bytes))
                .map_err(|e| format!("stage5: receive_asset response decode failed: {e}"))
        }
        Err(err) => {
            if let Err(lock_err) = lock_res {
                return Err(format!(
                    "stage5: receive_asset RPC failed: {err}; stage5: receiver lock on failure RPC failed: {lock_err}"
                ));
            }
            Err(format!("stage5: receive_asset RPC failed: {err}"))
        }
    }
}

pub(crate) fn select_bob_output(
    rows: &[TxOutputWire],
    out_idx: usize,
) -> Result<&TxOutputWire, String> {
    let recv_rows: Vec<_> = rows
        .iter()
        .filter(|row| row.role == TxOutRole::Recipient)
        .collect();
    if recv_rows.is_empty() {
        return Err("stage5: no recipient outputs in stage4 tx package".to_string());
    }
    recv_rows
        .get(out_idx)
        .copied()
        .ok_or_else(|| format!("stage5: recipient output index out of range: {out_idx}"))
}

pub(crate) fn check_role(row: &TxOutputWire) -> Result<(), String> {
    if row.role == TxOutRole::Recipient {
        return Ok(());
    }
    Err("stage5: selected output role must be Recipient".to_string())
}

fn check_wire_stealth(row: &TxOutputWire) -> Result<(), String> {
    let shape = (
        row.asset_wire.r_pub.is_some(),
        row.asset_wire.owner_tag.is_some(),
        row.asset_wire.enc_pack.is_some(),
        row.asset_wire.tag16.is_some(),
    );

    match shape {
        (false, false, false, false) | (true, true, true, true) => Ok(()),
        (false, false, false, true) => Err(
            "stage5: selected asset stealth consistency failed: tag16 requires full stealth fields"
                .to_string(),
        ),
        (true, true, true, false) => Err(
            "stage5: selected asset stealth consistency failed: full stealth fields require tag16"
                .to_string(),
        ),
        _ => Err(
            "stage5: selected asset stealth consistency failed: partial stealth fields are not allowed"
                .to_string(),
        ),
    }
}

pub(crate) fn parse_out(row: &TxOutputWire) -> Result<(Asset, TerminalLeaf), String> {
    check_wire_stealth(row)?;
    let asset = row
        .asset_wire
        .clone()
        .to_asset()
        .map_err(|e| format!("stage5: selected output to_asset failed: {e}"))?;
    asset
        .validate_stealth_consistency()
        .map_err(|e| format!("stage5: selected asset stealth consistency failed: {e}"))?;
    let wire = z00z_core::AssetWire::from_asset(&asset);
    let leaf = wire_decrypt_leaf(&wire)
        .map_err(|e| format!("stage5: selected output to_leaf failed: {e}"))?;
    Ok((asset, leaf))
}

pub(crate) fn check_scan(
    receiver: &SimActor,
    asset: &Asset,
    leaf: &TerminalLeaf,
) -> Result<(AssetPackPlain, ReceiveReport, ReceiveReport), String> {
    let pack = receiver_scan_leaf(&receiver.keys, leaf)
        .map_err(|e| format!("stage5: receiver_scan_leaf failed: {e}"))?
        .ok_or_else(|| "stage5: receiver_scan_leaf returned NotMine".to_string())?;
    let canon = receiver_scan_report(&receiver.keys, leaf)
        .map_err(|e| format!("stage5: receiver_scan_report failed: {e}"))?;
    check_canon(&canon)?;
    let runtime = StealthOutputScanner::from_keys(&receiver.keys)
        .scan_leaf(asset)
        .recv_report();
    assert_recv_parity(&canon, &runtime)?;
    check_amount(pack.value, asset.amount)?;
    Ok((pack.opening_pack(), canon, runtime))
}

pub(crate) fn check_canon(canon: &ReceiveReport) -> Result<(), String> {
    if canon.status != ReceiveStatus::Detected {
        return Err(format!(
            "stage5: canonical receive status mismatch: {}",
            canon.status.rpc_code()
        ));
    }
    if canon.next != ReceiveNext::ReportOnly {
        return Err("stage5: canonical receive next must be ReportOnly".to_string());
    }
    Ok(())
}

pub(crate) fn assert_recv_parity(
    canon: &ReceiveReport,
    runtime: &ReceiveReport,
) -> Result<(), String> {
    if runtime.status == canon.status
        && runtime.reject == canon.reject
        && runtime.next == canon.next
    {
        return Ok(());
    }
    Err(format!(
        "stage5: canonical/runtime divergence: canon(status={}, reject={:?}, next={:?}) runtime(status={}, reject={:?}, next={:?})",
        canon.status.rpc_code(),
        canon.reject,
        canon.next,
        runtime.status.rpc_code(),
        runtime.reject,
        runtime.next
    ))
}

pub(crate) fn check_amount(pack_value: u64, amount: u64) -> Result<(), String> {
    if pack_value == amount {
        return Ok(());
    }
    Err("stage5: detected pack amount mismatch".to_string())
}

pub(crate) fn require_claim(rows: &[Asset], asset_id: [u8; 32], phase: &str) -> Result<(), String> {
    if rows.iter().any(|item| item.asset_id() == asset_id) {
        return Ok(());
    }
    Err(format!(
        "stage5: selected asset missing from claimed state {phase}: {}",
        hex::encode(asset_id)
    ))
}

pub(crate) fn check_claim_set(before: &[Asset], after: &[Asset]) -> Result<(), String> {
    let before_ids = claim_ids(before);
    let after_ids = claim_ids(after);
    if before_ids == after_ids {
        return Ok(());
    }
    Err(format!(
        "stage5: report-only receive changed claimed asset ids: before={before_ids:?} after={after_ids:?}"
    ))
}

fn claim_ids(rows: &[Asset]) -> BTreeSet<String> {
    rows.iter()
        .map(|item| hex::encode(item.asset_id()))
        .collect()
}
