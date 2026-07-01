//! Scenario 1 stage 8: transfer claim.

use z00z_crypto::expert::encoding::SafePassword;

use crate::{DesignStage, SimContext, StageResult};

use super::stage_7::transfer_lane_impl::{load_recv, load_stage_ctx, prep_dirs};
use super::stage_7::transfer_lane_runtime_support::{flush_logs, log_extra};

pub mod assertions;
mod claim_flow;
mod claim_handoff;

use self::claim_flow::{persist_stage_artifacts, run_claim_flow};
use self::claim_handoff::load_claim_rpc_ctx;

pub fn run_transfer_claim(ctx: &mut SimContext, stage: &DesignStage) -> StageResult {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(err) => return StageResult::Fail(format!("stage5: tokio runtime: {err}")),
    };

    match rt.block_on(run_claim_stage(ctx, stage)) {
        Ok(()) => StageResult::Ok,
        Err(err) => StageResult::Fail(format!(
            "stage {} ({}) failed: {}",
            stage.stage, stage.name, err
        )),
    }
}

async fn run_claim_stage(ctx: &mut SimContext, stage: &DesignStage) -> Result<(), String> {
    let (paths, stage4, wallet_svc, receiver, receiver_pass, out_idx) = load_stage_ctx(ctx)?;
    let out = &ctx.outputs_dir;
    let mut lines = Vec::new();
    let (logs_dir, tx_dir) = prep_dirs(out, &paths, stage.stage, &mut lines)?;
    let recv = load_recv(out, stage4, out_idx, receiver, stage.stage, &mut lines)?;
    let rpc = load_claim_rpc_ctx(
        &wallet_svc,
        receiver,
        &tx_dir,
        &recv,
        stage.stage.saturating_sub(1),
    )
    .await?;
    let safe_pass = SafePassword::from(receiver_pass.as_str());
    wallet_svc
        .ensure_wallet_session(&rpc.bob_id, &safe_pass)
        .await
        .map_err(|e| format!("stage5: unlock receiver before explicit claim failed: {e}"))?;
    let claimed_res =
        run_claim_flow(wallet_svc.clone(), &rpc, &recv, stage.stage, &mut lines).await;
    let lock_res = wallet_svc.lock_wallet(&rpc.bob_id).await;
    let claimed_len = match (claimed_res, lock_res) {
        (Ok(claimed_len), Ok(())) => claimed_len,
        (Err(error), Ok(())) => return Err(error),
        (Ok(_), Err(error)) => {
            return Err(format!(
                "stage5: lock receiver after explicit claim failed: {error}"
            ))
        }
        (Err(error), Err(lock_error)) => {
            return Err(format!(
                "{error}; stage5: lock receiver after explicit claim failed: {lock_error}"
            ))
        }
    };
    persist_stage_artifacts(
        out,
        &tx_dir,
        &paths,
        &recv,
        &rpc,
        claimed_len,
        stage.stage,
        &mut lines,
    )
    .await?;
    log_extra(stage, &mut lines)?;
    flush_logs(&logs_dir.join(&paths.logger_file), &lines)
}
