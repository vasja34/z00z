use super::{
    create_dir_all, hex_str, parse_reason_code, persist_claim_state_file, push_claim_row,
    want_half_abort, want_reject_first, want_replay_first, Arc, Asset, AssetWire, AuditLogRow,
    ClaimStateFile, Path, RpcTransport, SimContext, SystemTimeProvider, TimeProvider,
    WalletImportReport, WalletImportStat,
};

use super::wallet_flow_restart::verify_restart;
use z00z_utils::codec::{json, Codec, JsonCodec};
use z00z_wallets::rpc::types::wallet::SessionToken;

pub(crate) fn import_claim_to_wallets(
    ctx: &SimContext,
    actor_idxs: &[usize],
    per_actor_assets: &[Vec<Asset>],
    wallets_dir: &Path,
    state_path: &Path,
    claim_state: &mut ClaimStateFile,
    resume_fault: Option<&str>,
) -> Result<WalletImportReport, String> {
    let logs_dir = ctx.outputs_dir.join("logs");
    create_dir_all(&logs_dir).map_err(|e| e.to_string())?;
    let rpc_log = logs_dir.join("rpc_logger.json");

    let (wallet_svc_for_keys, transport) = if let Some(wallet_svc) = &ctx.wallet_service {
        (
            Arc::clone(wallet_svc),
            crate::scenario_1::stage_2::build_logged_transport_with_wallet(
                Arc::clone(wallet_svc),
                &rpc_log,
            )?,
        )
    } else {
        let (fallback_wallet_svc, fallback_transport) =
            crate::scenario_1::stage_2::build_logged_transport(ctx, wallets_dir, &rpc_log)?;
        (fallback_wallet_svc, fallback_transport)
    };
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;

    let mut report = rt.block_on(async move {
        let mut stats = Vec::with_capacity(per_actor_assets.len());
        let mut wallet_ids = Vec::with_capacity(per_actor_assets.len());
        let mut accepted_assets: Vec<Vec<Asset>> = vec![Vec::new(); per_actor_assets.len()];
        let total_assets: usize = per_actor_assets.iter().map(|rows| rows.len()).sum();
        let mut decisions = Vec::with_capacity(total_assets);
        let abort_half = want_half_abort(resume_fault);
        let inject_reject = want_reject_first(resume_fault);
        let inject_replay = want_replay_first(resume_fault);
        let mut done = 0usize;
        let abort_at = if abort_half {
            let half = total_assets / 2;
            if half == 0 {
                1
            } else {
                half
            }
        } else {
            usize::MAX
        };

        for (slot, assets) in per_actor_assets.iter().enumerate() {
            let actor_idx = *actor_idxs
                .get(slot)
                .ok_or_else(|| format!("actor slot {slot} missing"))?;
            let actor = ctx
                .actors
                .get(actor_idx)
                .ok_or_else(|| format!("actor index {actor_idx} missing"))?;

            let pass = crate::scenario_1::stage_2::actor_runtime_password(actor)
                .ok_or_else(|| format!("password mapping missing for actor {}", actor.name))?;
            let actor_wallet_id = actor.wallet_id.clone();

            let mut session = None;
            let mut unlock_err = None;
            for wallet_id_try in [actor_wallet_id.clone(), actor.record.wallet_id().to_hex()] {
                match transport
                    .call(
                        "wallet.session.unlock_wallet",
                        json!({
                            "wallet_id": wallet_id_try,
                            "password": pass,
                        }),
                    )
                    .await
                {
                    Ok(value) => {
                        session = Some(value);
                        break;
                    }
                    Err(err) => {
                        unlock_err = Some(err.to_string());
                    }
                }
            }

            let session = match session {
                Some(value) => value,
                None => actor.session.clone().ok_or_else(|| {
                    format!(
                        "unlock_wallet({}) failed for both id formats and no cached session: {}",
                        actor.name,
                        unlock_err.unwrap_or_else(|| "unknown error".to_string())
                    )
                })?,
            };
            let parsed_session: SessionToken = JsonCodec
                .serialize(&session)
                .and_then(|bytes| JsonCodec.deserialize(&bytes))
                .map_err(|e| format!("session decode ({}): {e}", actor.name))?;
            let wallet_id = parsed_session.wallet_id.0.clone();
            let wallet_id_obj = parsed_session.wallet_id.clone();
            wallet_ids.push(wallet_id.clone());

            let mut inserted = 0usize;
            let mut already_exists = 0usize;
            let mut rejected = 0usize;

            let recv_keys = wallet_svc_for_keys
                .receiver_keys(&wallet_id_obj)
                .await
                .map_err(|e| format!("receiver_keys({}): {e}", actor.name))?;
            let recv_card = recv_keys
                .export_receiver_card()
                .map_err(|e| format!("export_receiver_card({}): {e}", actor.name))?;

            for asset in assets {
                let asset_id = asset.asset_id();
                let asset_hex = hex_str(&asset_id);
                let _ = (&recv_keys, &recv_card);
                let wire = AssetWire::from_asset(asset);
                let dto = z00z_core::assets::AssetPkgWire::from_wire(&wire);
                let bytes =
                    z00z_core::assets::encode_asset_pkg_json(&dto).map_err(|e| e.to_string())?;
                let asset_data = String::from_utf8(bytes).map_err(|e| e.to_string())?;
                if inject_reject && done == 0 {
                    rejected = rejected.saturating_add(1);
                    decisions.push(AuditLogRow {
                        timestamp: SystemTimeProvider.compat_unix_timestamp(),
                        wallet_id: wallet_id.clone(),
                        asset_id: asset_hex,
                        action: "import_rejected".to_string(),
                        reason_code: "IMPORT_INJECTED_REJECT".to_string(),
                    });
                    done = done.saturating_add(1);
                    continue;
                }

                let (action, reason_code, is_claimed) = match transport
                    .call(
                        "wallet.asset.import_asset",
                        json!({
                            "session": session.clone(),
                            "asset_data": asset_data,
                        }),
                    )
                    .await
                {
                    Ok(resp) => {
                        let success = resp
                            .get("success")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true);
                        if !success {
                            rejected = rejected.saturating_add(1);
                            let msg = resp
                                .get("message")
                                .and_then(|v| v.as_str())
                                .or_else(|| {
                                    resp.get("status")
                                        .and_then(|v| v.get("message"))
                                        .and_then(|v| v.as_str())
                                })
                                .unwrap_or("IMPORT_UNKNOWN_ERROR");
                            ("import_rejected".to_string(), parse_reason_code(msg), false)
                        } else {
                            let message = resp
                                .get("message")
                                .and_then(|v| v.as_str())
                                .or_else(|| {
                                    resp.get("status")
                                        .and_then(|v| v.get("message"))
                                        .and_then(|v| v.as_str())
                                })
                                .unwrap_or("");
                            if message == "asset_already_exists" {
                                already_exists = already_exists.saturating_add(1);
                                (
                                    "import_accepted".to_string(),
                                    "IMPORT_ALREADY_EXISTS".to_string(),
                                    true,
                                )
                            } else {
                                inserted = inserted.saturating_add(1);
                                (
                                    "import_accepted".to_string(),
                                    "IMPORT_ACCEPTED_NEW".to_string(),
                                    true,
                                )
                            }
                        }
                    }
                    Err(err) => {
                        let reason = parse_reason_code(&err.to_string());
                        rejected = rejected.saturating_add(1);
                        ("import_rejected".to_string(), reason, false)
                    }
                };

                decisions.push(AuditLogRow {
                    timestamp: SystemTimeProvider.compat_unix_timestamp(),
                    wallet_id: wallet_id.clone(),
                    asset_id: asset_hex.clone(),
                    action,
                    reason_code,
                });

                if is_claimed {
                    push_claim_row(claim_state, &wallet_id, asset_id);
                    persist_claim_state_file(state_path, claim_state)?;
                    let accepted = wire.to_asset().map_err(|e| {
                        format!("accepted wire decode failed ({}): {e}", actor.name)
                    })?;
                    accepted_assets[slot].push(accepted);
                }

                if inject_replay && done == 0 && is_claimed {
                    let replay_resp = transport
                        .call(
                            "wallet.asset.import_asset",
                            json!({
                                "session": session.clone(),
                                "asset_data": asset_data,
                            }),
                        )
                        .await
                        .map_err(|e| format!("import_asset_replay({}): {e}", actor.name))?;

                    let replay_reason = replay_resp
                        .get("message")
                        .and_then(|v| v.as_str())
                        .or_else(|| {
                            replay_resp
                                .get("status")
                                .and_then(|v| v.get("message"))
                                .and_then(|v| v.as_str())
                        })
                        .unwrap_or("");

                    let reason_code = if replay_reason == "asset_already_exists" {
                        "IMPORT_ALREADY_EXISTS".to_string()
                    } else {
                        "IMPORT_UNKNOWN_ERROR".to_string()
                    };

                    decisions.push(AuditLogRow {
                        timestamp: SystemTimeProvider.compat_unix_timestamp(),
                        wallet_id: wallet_id.clone(),
                        asset_id: asset_hex,
                        action: "import_accepted".to_string(),
                        reason_code,
                    });
                }

                done = done.saturating_add(1);
                if done >= abort_at {
                    return Err("resume_half_abort".to_string());
                }
            }

            let list = transport
                .call(
                    "wallet.asset.list_assets",
                    json!({
                        "wallet_id": wallet_id.clone(),
                        "limit": 50,
                        "cursor": null,
                        "filter": null,
                    }),
                )
                .await
                .map_err(|e| format!("list_assets({}): {e}", actor.name))?;

            let imported_total = list
                .get("total_count")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .unwrap_or_else(|| {
                    list.get("assets")
                        .and_then(|v| v.as_array())
                        .map(|v| v.len())
                        .unwrap_or(0)
                });
            if imported_total < inserted.saturating_add(already_exists) {
                return Err(format!(
                    "wallet import count mismatch for {}: imported_total={} expected_at_least={}",
                    actor.name,
                    imported_total,
                    inserted.saturating_add(already_exists)
                ));
            }

            transport
                .call(
                    "wallet.session.lock_wallet",
                    json!({
                        "session": session,
                    }),
                )
                .await
                .map_err(|e| format!("lock_wallet({}): {e}", actor.name))?;

            stats.push(WalletImportStat {
                actor: actor.name.clone(),
                inserted,
                already_exists,
                rejected,
            });
        }

        Ok(WalletImportReport {
            claim_mode: "wallet_only_intermediate".to_string(),
            compatibility_version: 1,
            stats,
            persist_stats: Vec::new(),
            wallet_ids,
            accepted_assets,
            decisions,
            persisted_assets: Vec::new(),
        })
    })?;

    let (persist_stats, persisted_assets) = verify_restart(
        ctx,
        wallets_dir,
        actor_idxs,
        &report.wallet_ids,
        &report.accepted_assets,
        &rpc_log,
    )?;
    report.persist_stats = persist_stats;
    report.persisted_assets = persisted_assets;

    Ok(report)
}
