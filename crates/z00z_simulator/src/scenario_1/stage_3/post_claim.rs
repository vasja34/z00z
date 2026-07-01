use super::{
    create_dir_all, hex_str, path_exists, read_file, remove_file, rename_file, save_json,
    write_file, Codec, JsonCodec, Path, RpcTransport, SimContext,
};

#[cfg(feature = "wallet_debug_tools")]
use std::path::PathBuf;

#[cfg(feature = "wallet_debug_tools")]
use z00z_core::Asset;
use z00z_wallets::domains::hashing::compute_wallet_file_id;
use z00z_wallets::rpc::types::wallet::WalletSource;

#[cfg(feature = "wallet_debug_tools")]
use z00z_crypto::expert::encoding::SafePassword;
#[cfg(feature = "wallet_debug_tools")]
use z00z_utils::io::load_json_bounded;
#[cfg(feature = "wallet_debug_tools")]
use z00z_wallets::{
    db::WalletIdentity, internal_debug_tools::debug_export_wallet,
    rpc::types::common::PersistWalletId,
};

pub(crate) fn run_post_claim_export(
    ctx: &SimContext,
    actor_idxs: &[usize],
    wallets_dir: &Path,
) -> Result<(), String> {
    let logs_dir = ctx.outputs_dir.join("logs");
    create_dir_all(&logs_dir).map_err(|e| e.to_string())?;
    let rpc_log = logs_dir.join("rpc_logger.json");

    let export_dir = ctx.outputs_dir.join("wallets_export_import");
    create_dir_all(&export_dir).map_err(|e| e.to_string())?;

    let bob_idx = actor_idxs
        .iter()
        .copied()
        .find(|idx| {
            ctx.actors
                .get(*idx)
                .map(|actor| actor.name.eq_ignore_ascii_case("bob"))
                .unwrap_or(false)
        })
        .or_else(|| actor_idxs.first().copied())
        .ok_or_else(|| "post-claim export: no actor indexes".to_string())?;

    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let payload_file = export_dir.join("export_wallet_encrypted_payload_post_claim.json");
    let mut picks = Vec::new();
    picks.push(bob_idx);
    for idx in actor_idxs.iter().copied() {
        if idx != bob_idx {
            picks.push(idx);
        }
    }

    let mut errs = Vec::<String>::new();

    for idx in picks {
        let actor = ctx
            .actors
            .get(idx)
            .ok_or_else(|| format!("post-claim export: actor index {idx} missing"))?;
        let actor_name = actor.name.to_ascii_lowercase();
        let pass = crate::scenario_1::stage_2::actor_runtime_password(actor)
            .ok_or_else(|| format!("post-claim export: no password mapping for {actor_name}"))?;

        let export_resp = rt.block_on(async {
            let (svc, transport) =
                crate::scenario_1::stage_2::build_logged_transport(ctx, wallets_dir, &rpc_log)?;

            let source_hash = compute_wallet_file_id(&actor.wallet_id);
            let source_wlt = wallets_dir.join(format!("wallet_{}.wlt", hex_str(&source_hash[..8])));
            svc.open_wallet_source(WalletSource::Path {
                path: source_wlt.to_string_lossy().to_string(),
            })
            .await
            .map_err(|e| format!("post-claim open_wallet_source({actor_name}) failed: {e}"))?;

            transport
                .call(
                    "wallet.session.unlock_wallet",
                    z00z_utils::codec::json!({
                        "wallet_id": actor.wallet_id,
                        "password": pass,
                    }),
                )
                .await
                .map_err(|e| format!("post-claim unlock_wallet({actor_name}) RPC: {e}"))?;

            transport
                .call(
                    "app.wallet.export_wallet",
                    z00z_utils::codec::json!({
                        "wallet_id": actor.wallet_id,
                        "password": pass,
                    }),
                )
                .await
                .map_err(|e| format!("post-claim export_wallet({actor_name}) RPC: {e}"))
        });

        let export_resp = match export_resp {
            Ok(value) => value,
            Err(err) => {
                errs.push(err);
                continue;
            }
        };

        let payload = export_resp["encrypted_payload"].clone();
        if payload.is_null() {
            errs.push(format!(
                "post-claim export_wallet({actor_name}) RPC: missing encrypted_payload"
            ));
            continue;
        }
        save_json(&payload_file, &payload).map_err(|e| e.to_string())?;

        let payload_data =
            String::from_utf8(JsonCodec.serialize(&payload).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;

        let import_resp = rt.block_on(async {
            let (_svc, transport) =
                crate::scenario_1::stage_2::build_logged_transport(ctx, &export_dir, &rpc_log)?;
            transport
                .call(
                    "app.wallet.import_wallet",
                    z00z_utils::codec::json!({
                        "data": payload_data,
                        "password": pass,
                        "name": format!("PostClaim {}", actor.name),
                    }),
                )
                .await
                .map_err(|e| format!("post-claim import_wallet({actor_name}) RPC: {e}"))
        });

        let import_resp = match import_resp {
            Ok(value) => value,
            Err(err) => {
                errs.push(err);
                continue;
            }
        };

        let imported_id = import_resp["wallet_id"]
            .as_str()
            .ok_or_else(|| "post-claim import_wallet response missing wallet_id".to_string())?;
        if imported_id != actor.wallet_id {
            errs.push(format!(
                "post-claim export/import wallet_id mismatch: {} != {}",
                imported_id, actor.wallet_id
            ));
            continue;
        }

        #[cfg(feature = "wallet_debug_tools")]
        {
            let identity = WalletIdentity {
                network: "p2p".into(),
                chain: "devnet".into(),
            };
            let imported_hash = compute_wallet_file_id(imported_id);
            let imported_wlt =
                export_dir.join(format!("wallet_{}.wlt", hex_str(&imported_hash[..8])));
            if !path_exists(&imported_wlt).map_err(|e| e.to_string())? {
                return Err(format!(
                    "post-claim imported .wlt file is missing: {}",
                    imported_wlt.display()
                ));
            }

            let persist_id = PersistWalletId(imported_id.to_string());
            let pw = SafePassword::from(pass);
            let debug_path = export_dir.join("export_wallet_debug_post_claim.json");
            debug_export_wallet(&imported_wlt, &persist_id, &pw, &identity, &debug_path)
                .map_err(|e| format!("post-claim debug_export_wallet failed: {e}"))?;
            redact_debug_dump_secrets(&debug_path)?;
        }

        let imported_hash = compute_wallet_file_id(imported_id);
        let file_hex8 = hex_str(&imported_hash[..8]);
        let source_wlt = wallets_dir.join(format!("wallet_{file_hex8}.wlt"));
        let imported_wlt = export_dir.join(format!("wallet_{file_hex8}.wlt"));
        let post_claim_wlt = export_dir.join(format!("wallet_{file_hex8}_post_claim.wlt"));

        if !path_exists(&source_wlt).map_err(|e| e.to_string())? {
            return Err(format!(
                "post-claim source .wlt file is missing: {}",
                source_wlt.display()
            ));
        }
        if !path_exists(&imported_wlt).map_err(|e| e.to_string())? {
            return Err(format!(
                "post-claim imported .wlt file is missing: {}",
                imported_wlt.display()
            ));
        }

        let source_bytes = read_file(&source_wlt).map_err(|e| e.to_string())?;
        let imported_bytes = read_file(&imported_wlt).map_err(|e| e.to_string())?;

        if imported_bytes != source_bytes {
            write_file(&imported_wlt, &source_bytes).map_err(|e| e.to_string())?;
            let synced = read_file(&imported_wlt).map_err(|e| e.to_string())?;
            if synced != source_bytes {
                return Err("post-claim imported .wlt is not byte-identical after sync".to_string());
            }
        }

        if path_exists(&post_claim_wlt).map_err(|e| e.to_string())? {
            remove_file(&post_claim_wlt).map_err(|e| e.to_string())?;
        }
        rename_file(&imported_wlt, &post_claim_wlt).map_err(|e| e.to_string())?;

        return Ok(());
    }

    let actor = ctx
        .actors
        .get(bob_idx)
        .ok_or_else(|| format!("post-claim export: actor index {bob_idx} missing"))?;

    let stale_hash = compute_wallet_file_id(&actor.wallet_id);
    let stale_wlt = export_dir.join(format!("wallet_{}.wlt", hex_str(&stale_hash[..8])));
    if path_exists(&stale_wlt).map_err(|e| e.to_string())? {
        remove_file(&stale_wlt).map_err(|e| e.to_string())?;
    }

    let reason = if errs.is_empty() {
        "post-claim export/import failed with unknown reason".to_string()
    } else {
        errs.join(" | ")
    };

    let fallback = z00z_utils::codec::json!({
        "status": "post_claim_export_skipped",
        "wallet_id": actor.wallet_id,
        "reason": reason,
    });
    save_json(&payload_file, &fallback).map_err(|e| e.to_string())?;

    #[cfg(feature = "wallet_debug_tools")]
    {
        let actor_name = actor.name.to_ascii_lowercase();
        let pass = crate::scenario_1::stage_2::actor_runtime_password(actor)
            .ok_or_else(|| format!("post-claim export: no password mapping for {actor_name}"))?;
        let identity = WalletIdentity {
            network: "p2p".into(),
            chain: "devnet".into(),
        };
        let source_hash = compute_wallet_file_id(&actor.wallet_id);
        let source_wlt = wallets_dir.join(format!("wallet_{}.wlt", hex_str(&source_hash[..8])));
        if path_exists(&source_wlt).map_err(|e| e.to_string())? {
            let persist_id = PersistWalletId(actor.wallet_id.clone());
            let pw = SafePassword::from(pass);
            let debug_path = export_dir.join("export_wallet_debug_post_claim.json");
            debug_export_wallet(&source_wlt, &persist_id, &pw, &identity, &debug_path)
                .map_err(|e| format!("post-claim fallback debug_export_wallet failed: {e}"))?;
            redact_debug_dump_secrets(&debug_path)?;
        }
    }

    Ok(())
}

#[cfg(feature = "wallet_debug_tools")]
fn redact_debug_dump_secrets(path: &Path) -> Result<(), String> {
    let mut root: z00z_utils::codec::Value = load_json_bounded(path, 64 * 1024 * 1024)
        .map_err(|e| format!("debug dump read failed: {e}"))?;
    let obj = root
        .as_object_mut()
        .ok_or_else(|| "debug dump invalid shape: object expected".to_string())?;

    obj.insert("secrets".to_string(), z00z_utils::codec::json!([]));
    obj.insert(
        "secrets_redacted".to_string(),
        z00z_utils::codec::json!(true),
    );

    save_json(path, &root).map_err(|e| format!("debug dump redact failed: {e}"))
}

#[cfg(feature = "wallet_debug_tools")]
fn require_wlt_path(wallets_dir: &Path, wallet_id: &str) -> Result<PathBuf, String> {
    let file_id = compute_wallet_file_id(wallet_id);
    let hex8 = hex_str(&file_id[..8]);
    let path = wallets_dir.join(format!("wallet_{hex8}.wlt"));
    if !path_exists(&path).map_err(|e| e.to_string())? {
        return Err(format!("wlt not found: {}", path.display()));
    }
    Ok(path)
}

#[cfg(feature = "wallet_debug_tools")]
#[derive(serde::Serialize)]
struct DumpAssetRow {
    asset_id_hex: String,
    #[serde(flatten)]
    asset: Asset,
}

#[cfg(feature = "wallet_debug_tools")]
pub(crate) fn export_debug_dumps(
    ctx: &SimContext,
    actor_idxs: &[usize],
    persisted_assets: &[Vec<Asset>],
    out_claim: &Path,
) -> Result<(), String> {
    let wallets_dir = ctx.outputs_dir.join("wallets");
    let identity = WalletIdentity {
        network: "p2p".into(),
        chain: "devnet".into(),
    };

    for (slot, &actor_idx) in actor_idxs.iter().enumerate() {
        let actor = ctx
            .actors
            .get(actor_idx)
            .ok_or_else(|| format!("actor index {actor_idx} missing in debug dump"))?;

        let name = actor.name.to_ascii_lowercase();
        let pw = crate::scenario_1::stage_2::actor_runtime_password(actor)
            .map(SafePassword::from)
            .ok_or_else(|| format!("no password mapping for actor '{}'", actor.name))?;

        let persist_id = PersistWalletId(actor.wallet_id.clone());
        let wlt_path = require_wlt_path(&wallets_dir, &actor.wallet_id)?;
        let debug_path = out_claim.join(format!("export_wallet_debug_{name}.json"));

        debug_export_wallet(&wlt_path, &persist_id, &pw, &identity, &debug_path)
            .map_err(|e| format!("debug_export_wallet({name}): {e}"))?;
        redact_debug_dump_secrets(&debug_path)?;

        let assets = persisted_assets
            .get(slot)
            .ok_or_else(|| format!("missing persisted_assets for slot {slot}"))?;
        enrich_debug_dump_with_assets(&debug_path, assets)?;

        ctx.logger.info(&format!(
            "stage3.debug_dump: actor={name} path={}",
            debug_path.display()
        ));
    }

    Ok(())
}

#[cfg(feature = "wallet_debug_tools")]
fn enrich_debug_dump_with_assets(path: &Path, assets: &[Asset]) -> Result<(), String> {
    let dump_assets: Vec<DumpAssetRow> = assets
        .iter()
        .cloned()
        .map(|asset| DumpAssetRow {
            asset_id_hex: hex_str(&asset.asset_id()),
            asset: sanitize_dump_asset(asset),
        })
        .collect();
    let mut root: z00z_utils::codec::Value = load_json_bounded(path, 64 * 1024 * 1024)
        .map_err(|e| format!("debug dump read failed: {e}"))?;
    if !root.is_object() {
        return Err("debug dump invalid shape: object expected".to_string());
    }

    let obj = root
        .as_object_mut()
        .ok_or_else(|| "debug dump object expected".to_string())?;
    obj.insert(
        "imported_assets_full".to_string(),
        z00z_utils::codec::json!(dump_assets),
    );
    obj.insert(
        "imported_assets_count".to_string(),
        z00z_utils::codec::json!(assets.len() as u64),
    );

    save_json(path, &root).map_err(|e| format!("debug dump write failed: {e}"))
}

#[cfg(feature = "wallet_debug_tools")]
fn sanitize_dump_asset(mut asset: Asset) -> Asset {
    asset.r_pub = None;
    asset.owner_tag = None;
    asset.enc_pack = None;
    asset.secret = None;
    asset.tag16 = None;
    asset.leaf_ad_id = None;
    asset
}
