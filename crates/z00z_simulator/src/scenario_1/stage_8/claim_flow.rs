use std::{path::Path, sync::Arc};

use z00z_core::Asset;
use z00z_wallets::{
    receiver::ReceiveNext, rpc::types::common::PersistWalletId, services::WalletService,
};

use super::super::stage_7::transfer_lane_impl::{RecvCtx, RpcCtx};
use super::super::stage_7::transfer_lane_runtime_support::{log_ok, write_snap, write_tx};

pub(crate) async fn run_claim_flow(
    wallet_svc: Arc<WalletService>,
    rpc: &RpcCtx,
    recv: &RecvCtx,
    stage_id: u32,
    lines: &mut Vec<String>,
) -> Result<usize, String> {
    run_claim(
        wallet_svc,
        &rpc.bob_id,
        recv.asset.clone(),
        rpc.before_len,
        recv.asset.asset_id(),
        stage_id,
        lines,
    )
    .await
}

pub(crate) async fn persist_stage_artifacts(
    out: &Path,
    tx_dir: &Path,
    cfg: &crate::config::Stage5PathsCfg,
    recv: &RecvCtx,
    rpc: &RpcCtx,
    claimed_len: usize,
    stage_id: u32,
    lines: &mut Vec<String>,
) -> Result<(), String> {
    let tx_file = write_tx(tx_dir, cfg, recv, stage_id).await?;
    log_ok(
        lines,
        stage_id,
        "S5-7",
        "write_leaf_artifact",
        &tx_file.to_string_lossy(),
    )?;
    let snap_file = write_snap(out, cfg, recv, &rpc.rpc_status, claimed_len, stage_id).await?;
    log_ok(
        lines,
        stage_id,
        "S5-8",
        "write_snapshot",
        &snap_file.to_string_lossy(),
    )
}

async fn run_claim(
    wallet_svc: Arc<WalletService>,
    bob_id: &PersistWalletId,
    asset: Asset,
    before_len: usize,
    expected_asset_id: [u8; 32],
    stage_id: u32,
    lines: &mut Vec<String>,
) -> Result<usize, String> {
    let persisted = wallet_svc
        .recv_route(bob_id, asset, ReceiveNext::PersistClaim)
        .await
        .map_err(|e| format!("stage5: recv_route persist failed: {e}"))?;
    if persisted {
        return Err(
            "stage5: recv_route unexpectedly persisted an already imported Stage-4 output"
                .to_string(),
        );
    }
    let claimed_after = wallet_svc
        .list_claimed_assets(bob_id)
        .await
        .map_err(|e| format!("stage5: list claimed after route failed: {e}"))?;
    if claimed_after.len() != before_len {
        return Err(format!(
            "stage5: claimed count after idempotent route mismatch: before={} after={}",
            before_len,
            claimed_after.len()
        ));
    }
    if !claimed_after
        .iter()
        .any(|item| item.asset_id() == expected_asset_id)
    {
        return Err(format!(
            "stage5: expected claimed asset missing after route: {}",
            hex::encode(expected_asset_id)
        ));
    }
    log_ok(
        lines,
        stage_id,
        "S5-6",
        "explicit_claim",
        &format!(
            "recv_route PersistClaim stayed idempotent; persisted=false claimed_before_route={} claimed_after_route={} asset_id={}",
            before_len,
            claimed_after.len(),
            hex::encode(expected_asset_id)
        ),
    )?;
    Ok(claimed_after.len())
}
