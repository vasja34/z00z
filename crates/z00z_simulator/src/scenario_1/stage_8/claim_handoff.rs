use std::{path::Path, sync::Arc};

use z00z_wallets::{
    receiver::ReceiveStatus, rpc::types::common::PersistWalletId, services::WalletService,
};

use super::super::stage_7::{
    load_receive_handoff,
    transfer_lane_impl::{RecvCtx, RpcCtx},
};

pub(crate) async fn load_claim_rpc_ctx(
    wallet_svc: &Arc<WalletService>,
    receiver: &crate::SimActor,
    tx_dir: &Path,
    recv: &RecvCtx,
    expected_stage: u32,
) -> Result<RpcCtx, String> {
    let handoff = load_receive_handoff(tx_dir, recv, expected_stage)?;
    let bob_id = PersistWalletId(receiver.wallet_id.clone());
    let claimed_before = wallet_svc
        .list_claimed_assets(&bob_id)
        .await
        .map_err(|e| format!("stage5: list claimed before route failed: {e}"))?;
    if handoff.claimed_before_route != claimed_before.len() {
        return Err(format!(
            "stage5: receive handoff claimed_before_route mismatch: expected {}, got {}",
            claimed_before.len(),
            handoff.claimed_before_route
        ));
    }
    if handoff.rpc_status != ReceiveStatus::Detected.rpc_code() {
        return Err(format!(
            "stage5: receive handoff rpc_status mismatch: {}",
            handoff.rpc_status
        ));
    }
    Ok(RpcCtx {
        bob_id,
        before_len: claimed_before.len(),
        rpc_status: handoff.rpc_status,
    })
}
