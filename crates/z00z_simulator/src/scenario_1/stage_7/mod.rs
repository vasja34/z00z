//! Scenario 1 stage 7: transfer receive.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{path_exists, read_to_string, save_json},
};

use crate::{DesignStage, SimContext, StageResult};

use self::transfer_lane_impl::{load_recv, load_stage_ctx, prep_dirs, RecvCtx, RpcCtx};
use self::transfer_lane_runtime_support::{flush_logs, log_extra, log_ok, rpc_log_path};
use self::transfer_lane_support::run_report_flow;
use super::stage_2::build_logged_transport_with_wallet;

pub(crate) mod transfer_lane_impl;
pub(crate) mod transfer_lane_runtime_support;
pub(crate) mod transfer_lane_support;

const RECEIVE_HANDOFF_FILE: &str = "stage_5_receive_handoff.json";

#[derive(Serialize, Deserialize)]
pub(crate) struct ReceiveHandoff {
    pub(crate) stage: u32,
    pub(crate) input_tx_digest_hex: String,
    pub(crate) recipient_output_index: usize,
    pub(crate) asset_id_hex: String,
    pub(crate) rpc_status: String,
    pub(crate) claimed_before_route: usize,
    pub(crate) status: String,
}

pub fn run_transfer_receive(ctx: &mut SimContext, stage: &DesignStage) -> StageResult {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(err) => return StageResult::Fail(format!("stage5: tokio runtime: {err}")),
    };

    match rt.block_on(run_receive_stage(ctx, stage)) {
        Ok(()) => StageResult::Ok,
        Err(err) => StageResult::Fail(format!(
            "stage {} ({}) failed: {}",
            stage.stage, stage.name, err
        )),
    }
}

async fn run_receive_stage(ctx: &mut SimContext, stage: &DesignStage) -> Result<(), String> {
    let (paths, stage4, wallet_svc, receiver, receiver_pass, out_idx) = load_stage_ctx(ctx)?;
    let out = &ctx.outputs_dir;
    let mut lines = Vec::new();
    let (logs_dir, tx_dir) = prep_dirs(out, &paths, stage.stage, &mut lines)?;
    let recv = load_recv(out, stage4, out_idx, receiver, stage.stage, &mut lines)?;
    let rpc_log = rpc_log_path(&logs_dir, &paths.logger_file);
    let transport = build_logged_transport_with_wallet(wallet_svc.clone(), &rpc_log)?;
    let rpc = run_report_flow(
        wallet_svc,
        &transport,
        stage4,
        receiver,
        &receiver_pass,
        &recv,
        stage.stage,
        &mut lines,
    )
    .await?;
    let handoff = write_receive_handoff(&tx_dir, &recv, &rpc, stage.stage)?;
    log_ok(
        &mut lines,
        stage.stage,
        "S5-6",
        "write_receive_handoff",
        &handoff.to_string_lossy(),
    )?;
    log_extra(stage, &mut lines)?;
    flush_logs(&logs_dir.join(&paths.logger_file), &lines)
}

fn write_receive_handoff(
    tx_dir: &Path,
    recv: &RecvCtx,
    rpc: &RpcCtx,
    stage_id: u32,
) -> Result<PathBuf, String> {
    let path = tx_dir.join(RECEIVE_HANDOFF_FILE);
    let body = ReceiveHandoff {
        stage: stage_id,
        input_tx_digest_hex: recv.tx_pkg.tx_digest_hex.clone(),
        recipient_output_index: recv.out_idx,
        asset_id_hex: hex::encode(recv.asset.asset_id()),
        rpc_status: rpc.rpc_status.clone(),
        claimed_before_route: rpc.before_len,
        status: "ok".to_string(),
    };
    save_json(&path, &body).map_err(|e| e.to_string())?;
    Ok(path)
}

pub(crate) fn load_receive_handoff(
    tx_dir: &Path,
    recv: &RecvCtx,
    expected_stage: u32,
) -> Result<ReceiveHandoff, String> {
    let path = tx_dir.join(RECEIVE_HANDOFF_FILE);
    if !path_exists(&path).map_err(|e| e.to_string())? {
        return Err(format!(
            "stage5: receive handoff missing; run transfer_receive first: {}",
            path.display()
        ));
    }
    let body = read_to_string(&path).map_err(|e| e.to_string())?;
    let handoff: ReceiveHandoff = JsonCodec
        .deserialize(body.as_bytes())
        .map_err(|e| format!("stage5: receive handoff decode failed: {e}"))?;
    if handoff.stage != expected_stage {
        return Err(format!(
            "stage5: receive handoff stage mismatch: expected {expected_stage}, got {}",
            handoff.stage
        ));
    }
    if handoff.status != "ok" {
        return Err(format!(
            "stage5: receive handoff status mismatch: {}",
            handoff.status
        ));
    }
    if handoff.asset_id_hex != hex::encode(recv.asset.asset_id()) {
        return Err("stage5: receive handoff asset_id mismatch".to_string());
    }
    if handoff.recipient_output_index != recv.out_idx {
        return Err("stage5: receive handoff recipient_output_index mismatch".to_string());
    }
    if handoff.input_tx_digest_hex != recv.tx_pkg.tx_digest_hex {
        return Err("stage5: receive handoff tx digest mismatch".to_string());
    }
    Ok(handoff)
}
