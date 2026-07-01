use super::{
    actor_runtime_password, parse_list_settlement_rows, AssetWire, Codec, OutputBundle,
    RpcTransport, SenderPersist, Stage4TxPrepareCfg, TxOutputWire, TxStorage,
};
use std::collections::{BTreeMap, BTreeSet};
use z00z_wallets::chain::ReceiverCardRecord;

pub(crate) async fn persist_sender_state(
    transport: &impl RpcTransport,
    cfg: &Stage4TxPrepareCfg,
    sender: &crate::SimActor,
    receiver: &crate::SimActor,
    selected: &[AssetWire],
    outputs: &[OutputBundle],
    tx_outputs: &[TxOutputWire],
) -> Result<SenderPersist, String> {
    if outputs.len() != tx_outputs.len() {
        return Err(format!(
            "stage4: output bundle/wire length mismatch: bundles={} wires={}",
            outputs.len(),
            tx_outputs.len()
        ));
    }

    let sender_password = actor_runtime_password(sender)
        .ok_or_else(|| format!("stage4: no password for actor {}", sender.name))?;

    let session = transport
        .call(
            &cfg.rpc.unlock_method,
            z00z_utils::codec::json!({
                "wallet_id": sender.wallet_id,
                "password": sender_password,
            }),
        )
        .await
        .map_err(|e| format!("stage4: unlock sender (persist) RPC failed: {e}"))?;

    let run_res: Result<SenderPersist, String> = async {
        let mut rep = SenderPersist::default();
        let tracked_inputs: BTreeSet<([u8; 32], u32)> = selected
            .iter()
            .map(|row| (row.definition.id, row.serial_id))
            .collect();

        let bob_total = outputs
            .iter()
            .filter(|out| out.role == z00z_wallets::tx::TxOutRole::Recipient)
            .fold(0u64, |acc, out| acc.saturating_add(out.value));
        if bob_total == 0 {
            return Err("stage4: missing bob outputs for sender persistence".to_string());
        }

        let recipient =
            ReceiverCardRecord::new(&receiver.card, receiver.card.canonical_encoding(), 0)
                .and_then(|record| record.to_compact())
                .map_err(|e| format!("stage4: build recipient record failed: {e}"))?;
        let send_asset_definition_id = selected[0].definition.id;
        let built = transport
            .call(
                &cfg.rpc.build_transaction_method,
                z00z_utils::codec::json!({
                    "session": session,
                    "asset_id": hex::encode(send_asset_definition_id),
                    "recipient": recipient,
                    "amount": bob_total,
                }),
            )
            .await
            .map_err(|e| format!("stage4: sender build_transaction RPC failed: {e}"))?;

        let has_tx_id = built.get("tx_id").is_some();
        let has_raw_tx = built
            .get("raw_tx")
            .and_then(|value| value.as_str())
            .map(|value| !value.is_empty())
            .unwrap_or(false);
        if !has_tx_id || !has_raw_tx {
            return Err(
                "stage4: sender build_transaction response missing tx_id or raw_tx".to_string(),
            );
        }

        for (meta, wire) in outputs.iter().zip(tx_outputs.iter()) {
            if meta.role != z00z_wallets::tx::TxOutRole::Change {
                continue;
            }
            let asset_data = String::from_utf8(
                z00z_core::assets::encode_asset_pkg_json(&wire.asset_wire)
                    .map_err(|e| format!("stage4: serialize alice change dto failed: {e}"))?,
            )
            .map_err(|e| format!("stage4: utf8 encode alice change dto failed: {e}"))?;

            transport
                .call(
                    &cfg.rpc.import_asset_method,
                    z00z_utils::codec::json!({
                        "session": session,
                        "asset_data": asset_data,
                    }),
                )
                .await
                .map_err(|e| format!("stage4: sender import change RPC failed: {e}"))?;
            rep.change_imported += 1;
        }

        let listed_after = transport
            .call(
                &cfg.rpc.list_assets_method,
                z00z_utils::codec::json!({
                    "wallet_id": sender.wallet_id,
                    "limit": cfg.rpc.list_limit,
                    "cursor": z00z_utils::codec::Value::Null,
                    "filter": {
                        "asset_class": cfg.rpc.list_filter.asset_class,
                        "min_balance": cfg.rpc.list_filter.min_balance,
                    },
                }),
            )
            .await
            .map_err(|e| format!("stage4: sender post-persist list_assets RPC failed: {e}"))?;

        let rows_after = parse_list_settlement_rows(&listed_after).unwrap_or_default();
        let alive_inputs: BTreeSet<([u8; 32], u32)> = rows_after
            .iter()
            .map(|row| (row.definition.id, row.serial_id))
            .collect();
        let after_amounts: BTreeMap<([u8; 32], u32), u64> = rows_after
            .iter()
            .map(|row| ((row.definition.id, row.serial_id), row.amount))
            .collect();
        let before_amounts: BTreeMap<([u8; 32], u32), u64> = selected
            .iter()
            .map(|row| ((row.definition.id, row.serial_id), row.amount))
            .collect();
        rep.spent_marked = tracked_inputs
            .iter()
            .filter(|item| !alive_inputs.contains(item))
            .count();
        rep.tracked_amount_changed = before_amounts
            .iter()
            .filter(|(key, before)| {
                after_amounts
                    .get(*key)
                    .map(|after| after != *before)
                    .unwrap_or(false)
            })
            .count();

        Ok(rep)
    }
    .await;

    let lock_res = transport
        .call(
            &cfg.rpc.lock_method,
            z00z_utils::codec::json!({"session": session}),
        )
        .await;

    match run_res {
        Ok(rep) => {
            lock_res.map_err(|e| format!("stage4: lock sender (persist) RPC failed: {e}"))?;
            Ok(rep)
        }
        Err(err) => {
            if let Err(lock_err) = lock_res {
                return Err(format!(
                    "{err}; stage4: sender lock on persist-fail RPC failed: {lock_err}"
                ));
            }
            Err(err)
        }
    }
}
