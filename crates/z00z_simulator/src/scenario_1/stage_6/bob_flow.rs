use super::{
    actor_runtime_password, check_stealth_own, Codec, OutputBundle, RpcTransport,
    Stage4TxPrepareCfg, TxOutputWire,
};

pub(crate) async fn run_bob_checks(
    transport: &impl RpcTransport,
    cfg: &Stage4TxPrepareCfg,
    receiver: &crate::SimActor,
    outputs: &[OutputBundle],
    tx_outputs: &[TxOutputWire],
) -> Result<(), String> {
    if outputs.len() != tx_outputs.len() {
        return Err(format!(
            "stage4: output bundle/wire length mismatch for bob checks: bundles={} wires={}",
            outputs.len(),
            tx_outputs.len()
        ));
    }

    let receiver_password = actor_runtime_password(receiver)
        .ok_or_else(|| format!("stage4: no password for actor {}", receiver.name))?;

    let session = transport
        .call(
            &cfg.rpc.unlock_method,
            z00z_utils::codec::json!({
                "wallet_id": receiver.wallet_id,
                "password": receiver_password,
            }),
        )
        .await
        .map_err(|e| format!("stage4: unlock receiver RPC failed: {e}"))?;

    let run_res: Result<usize, String> = async {
        let mut bob_count = 0usize;
        for (meta, wire) in outputs.iter().zip(tx_outputs.iter()) {
            if meta.role != z00z_wallets::tx::TxOutRole::Recipient {
                continue;
            }
            bob_count += 1;

            let asset_data = String::from_utf8(
                z00z_core::assets::encode_asset_pkg_json(&wire.asset_wire)
                    .map_err(|e| format!("stage4: serialize bob output dto failed: {e}"))?,
            )
            .map_err(|e| format!("stage4: utf8 encode bob output dto failed: {e}"))?;

            transport
                .call(
                    &cfg.rpc.import_asset_method,
                    z00z_utils::codec::json!({
                        "session": session,
                        "asset_data": asset_data,
                    }),
                )
                .await
                .map_err(|e| format!("stage4: bob import_asset RPC failed: {e}"))?;

            let asset = wire
                .asset_wire
                .clone()
                .to_asset()
                .map_err(|e| format!("stage4: bob output to_asset failed: {e}"))?;
            check_stealth_own(&asset, &receiver.keys)
                .map_err(|e| format!("stage4: bob stealth decrypt/own check failed: {e}"))?;
        }

        Ok(bob_count)
    }
    .await;

    let lock_res = transport
        .call(
            &cfg.rpc.lock_method,
            z00z_utils::codec::json!({"session": session}),
        )
        .await;

    match run_res {
        Ok(bob_count) => {
            lock_res.map_err(|e| format!("stage4: lock receiver RPC failed: {e}"))?;
            let expected_bob = cfg.transaction.outputs.bob_outputs_count as usize;
            if bob_count != expected_bob {
                return Err(format!(
                    "stage4: bob output count mismatch in upload/decrypt checks: got={} expected={}",
                    bob_count, expected_bob
                ));
            }
            Ok(())
        }
        Err(err) => {
            if let Err(lock_err) = lock_res {
                return Err(format!(
                    "{err}; stage4: receiver lock on failure RPC failed: {lock_err}"
                ));
            }
            Err(err)
        }
    }
}

pub(crate) async fn run_fee_checks(
    transport: &impl RpcTransport,
    cfg: &Stage4TxPrepareCfg,
    fee_name: &str,
    fee_id: &str,
    fee_pass: &str,
    outputs: &[OutputBundle],
    tx_outputs: &[TxOutputWire],
) -> Result<(), String> {
    if outputs.len() != tx_outputs.len() {
        return Err(format!(
            "stage4: output bundle/wire length mismatch for fee checks: bundles={} wires={}",
            outputs.len(),
            tx_outputs.len()
        ));
    }

    let session = transport
        .call(
            &cfg.rpc.unlock_method,
            z00z_utils::codec::json!({
                "wallet_id": fee_id,
                "password": fee_pass,
            }),
        )
        .await
        .map_err(|e| format!("stage4: unlock fee wallet RPC failed: {e}"))?;

    let run_res: Result<usize, String> = async {
        let mut fee_count = 0usize;
        for (meta, wire) in outputs.iter().zip(tx_outputs.iter()) {
            if meta.role != z00z_wallets::tx::TxOutRole::Fee {
                continue;
            }
            fee_count += 1;

            let asset_data = String::from_utf8(
                z00z_core::assets::encode_asset_pkg_json(&wire.asset_wire)
                    .map_err(|e| format!("stage4: serialize fee output dto failed: {e}"))?,
            )
            .map_err(|e| format!("stage4: utf8 encode fee output dto failed: {e}"))?;

            transport
                .call(
                    &cfg.rpc.import_asset_method,
                    z00z_utils::codec::json!({
                        "session": session,
                        "asset_data": asset_data,
                    }),
                )
                .await
                .map_err(|e| format!("stage4: fee import_asset RPC failed: {e}"))?;
        }

        Ok(fee_count)
    }
    .await;

    let lock_res = transport
        .call(
            &cfg.rpc.lock_method,
            z00z_utils::codec::json!({"session": session}),
        )
        .await;

    match run_res {
        Ok(fee_count) => {
            lock_res.map_err(|e| format!("stage4: lock fee wallet RPC failed: {e}"))?;
            if fee_count == 0 {
                return Err(format!("stage4: missing fee outputs for {fee_name}"));
            }
            Ok(())
        }
        Err(err) => {
            if let Err(lock_err) = lock_res {
                return Err(format!(
                    "{err}; stage4: fee wallet lock on failure RPC failed: {lock_err}"
                ));
            }
            Err(err)
        }
    }
}
