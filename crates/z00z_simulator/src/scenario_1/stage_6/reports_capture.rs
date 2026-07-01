use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use z00z_core::AssetWire;
use z00z_crypto::expert::encoding::to_hex;
use z00z_networks_rpc::RpcTransport;
use z00z_utils::{
    codec::{json, Codec, JsonCodec, Value},
    io::path_exists,
    time::{format_system_time_local, SystemTimeProvider, TimeProvider},
};
use z00z_wallets::{
    domains::hashing::compute_wallet_file_id, rpc::types::common::PersistWalletId, WalletService,
};

use crate::{config::Stage4TxPrepareCfg, SimActor, SimContext};

use super::{
    actor_runtime_password, extract_next_cursor, SerialDistRow, WalletItemRow, WalletStateDump,
    WalletStateRow,
};

pub(crate) async fn capture_wallet_states(
    transport: &dyn RpcTransport,
    cfg: &Stage4TxPrepareCfg,
    ctx: &SimContext,
    wallets_dir: &Path,
    phase: &str,
    fee_extra: Option<(&str, &str, &str)>,
) -> Result<WalletStateDump, String> {
    let mut wallets = Vec::new();
    for actor in &ctx.actors {
        wallets.push(
            capture_wallet_one(
                transport,
                cfg,
                ctx.wallet_service.as_ref(),
                actor,
                wallets_dir,
            )
            .await?,
        );
    }
    if let Some((name, wallet_id, password)) = fee_extra {
        if !wallets.iter().any(|row| row.wallet_id == wallet_id) {
            wallets.push(
                capture_wallet_named(
                    transport,
                    cfg,
                    ctx.wallet_service.as_ref(),
                    name,
                    wallet_id,
                    password,
                    wallets_dir,
                )
                .await?,
            );
        }
    }

    Ok(WalletStateDump {
        stage: 4,
        phase: phase.to_string(),
        generated_at: format_system_time_local(SystemTimeProvider.now()),
        wallets,
    })
}

pub(crate) async fn capture_wallet_actor(
    transport: &dyn RpcTransport,
    cfg: &Stage4TxPrepareCfg,
    wallet_service: Option<&Arc<WalletService>>,
    actor: &SimActor,
    wallets_dir: &Path,
    phase: &str,
) -> Result<WalletStateDump, String> {
    let wallet = capture_wallet_one(transport, cfg, wallet_service, actor, wallets_dir).await?;

    Ok(WalletStateDump {
        stage: 4,
        phase: phase.to_string(),
        generated_at: format_system_time_local(SystemTimeProvider.now()),
        wallets: vec![wallet],
    })
}

async fn capture_wallet_one(
    transport: &dyn RpcTransport,
    cfg: &Stage4TxPrepareCfg,
    wallet_service: Option<&Arc<WalletService>>,
    actor: &SimActor,
    wallets_dir: &Path,
) -> Result<WalletStateRow, String> {
    let password = actor_runtime_password(actor)
        .ok_or_else(|| format!("stage4: no password for actor {}", actor.name))?;
    let (rows, page_count) = with_live_wallet_rows(
        transport,
        cfg,
        wallet_service,
        &actor.name,
        &actor.wallet_id,
        &password,
    )
    .await?;

    let items = rows_to_items(&rows)?;
    let serial_dist = build_serial_dist(&items);
    let wlt_path = wallet_path_for_id(wallets_dir, &actor.wallet_id);
    let (wlt_exists, wlt_size_bytes) = wallet_file_state(&wlt_path)?;

    Ok(WalletStateRow {
        actor: actor.name.clone(),
        wallet_id: actor.wallet_id.clone(),
        wlt_path: wlt_path.to_string_lossy().to_string(),
        wlt_exists,
        wlt_size_bytes,
        page_count,
        item_count: items.len(),
        serial_dist,
        items,
    })
}

async fn capture_wallet_named(
    transport: &dyn RpcTransport,
    cfg: &Stage4TxPrepareCfg,
    wallet_service: Option<&Arc<WalletService>>,
    actor: &str,
    wallet_id: &str,
    password: &str,
    wallets_dir: &Path,
) -> Result<WalletStateRow, String> {
    let (rows, page_count) =
        with_live_wallet_rows(transport, cfg, wallet_service, actor, wallet_id, password).await?;

    let items = rows_to_items(&rows)?;
    let serial_dist = build_serial_dist(&items);
    let wlt_path = wallet_path_for_id(wallets_dir, wallet_id);
    let (wlt_exists, wlt_size_bytes) = wallet_file_state(&wlt_path)?;

    Ok(WalletStateRow {
        actor: actor.to_string(),
        wallet_id: wallet_id.to_string(),
        wlt_path: wlt_path.to_string_lossy().to_string(),
        wlt_exists,
        wlt_size_bytes,
        page_count,
        item_count: items.len(),
        serial_dist,
        items,
    })
}

fn wallet_file_state(path: &Path) -> Result<(bool, u64), String> {
    let wlt_exists = path_exists(path).map_err(|e| e.to_string())?;
    let wlt_size_bytes = if wlt_exists {
        z00z_utils::io::file_len(path).map_err(|e| e.to_string())?
    } else {
        0
    };
    Ok((wlt_exists, wlt_size_bytes))
}

async fn with_live_wallet_rows(
    transport: &dyn RpcTransport,
    cfg: &Stage4TxPrepareCfg,
    wallet_service: Option<&Arc<WalletService>>,
    actor: &str,
    wallet_id: &str,
    password: &str,
) -> Result<(Vec<Value>, usize), String> {
    if let Some(service) = wallet_service {
        let wallet_id_obj = PersistWalletId(wallet_id.to_string());
        let safe_password = z00z_crypto::expert::encoding::SafePassword::from(password);
        service
            .ensure_wallet_session(&wallet_id_obj, &safe_password)
            .await
            .map_err(|e| format!("unlock wallet {actor} failed: {e}"))?;

        let rows_res = list_wallet_rows_all(transport, cfg, wallet_id).await;
        let lock_res = service
            .lock_wallet(&wallet_id_obj)
            .await
            .map_err(|e| format!("lock wallet {actor} failed: {e}"));
        return resolve_rows(actor, wallet_id, rows_res, lock_res);
    }

    let session = unlock_named(transport, cfg, actor, wallet_id, password).await?;
    let rows_res = list_wallet_rows_all(transport, cfg, wallet_id).await;
    let lock_res = lock_wallet(transport, cfg, &session).await;
    resolve_rows(actor, wallet_id, rows_res, lock_res)
}

async fn unlock_named(
    transport: &dyn RpcTransport,
    cfg: &Stage4TxPrepareCfg,
    actor: &str,
    wallet_id: &str,
    password: &str,
) -> Result<Value, String> {
    transport
        .call(
            &cfg.rpc.unlock_method,
            json!({
                "wallet_id": wallet_id,
                "password": password,
            }),
        )
        .await
        .map_err(|e| format!("unlock wallet {actor} failed: {e}"))
}

async fn lock_wallet(
    transport: &dyn RpcTransport,
    cfg: &Stage4TxPrepareCfg,
    session: &Value,
) -> Result<(), String> {
    transport
        .call(&cfg.rpc.lock_method, json!({"session": session}))
        .await
        .map(|_| ())
        .map_err(|e| format!("lock wallet failed: {e}"))
}

fn resolve_rows(
    _actor: &str,
    wallet_id: &str,
    rows_res: Result<(Vec<Value>, usize), String>,
    lock_res: Result<(), String>,
) -> Result<(Vec<Value>, usize), String> {
    match rows_res {
        Ok(rows) => {
            lock_res?;
            Ok(rows)
        }
        Err(err) => {
            if let Err(lock_err) = lock_res {
                return Err(format!(
                    "{err}; lock wallet {wallet_id} on failure failed: {lock_err}",
                ));
            }
            Err(err)
        }
    }
}

async fn list_wallet_rows_all(
    transport: &dyn RpcTransport,
    cfg: &Stage4TxPrepareCfg,
    wallet_id: &str,
) -> Result<(Vec<Value>, usize), String> {
    let mut cursor = Value::Null;
    let mut rows = Vec::<Value>::new();
    let mut page_count = 0usize;

    while page_count < 128 {
        page_count += 1;
        let listed = list_wallet_page(transport, cfg, wallet_id, &cursor).await?;
        rows.extend(extract_list_rows(&listed)?);

        match extract_next_cursor(&listed) {
            Some(next) => cursor = next,
            None => break,
        }
    }

    Ok((rows, page_count))
}

async fn list_wallet_page(
    transport: &dyn RpcTransport,
    cfg: &Stage4TxPrepareCfg,
    wallet_id: &str,
    cursor: &Value,
) -> Result<Value, String> {
    transport
        .call(
            &cfg.rpc.list_assets_method,
            json!({
                "wallet_id": wallet_id,
                "limit": cfg.rpc.list_limit,
                "cursor": cursor,
                "filter": Value::Null,
            }),
        )
        .await
        .map_err(|e| format!("list_assets failed for wallet_id={wallet_id}: {e}"))
}

fn extract_list_rows(listed: &Value) -> Result<Vec<Value>, String> {
    listed
        .get("items")
        .or_else(|| listed.get("assets"))
        .ok_or_else(|| "stage4: list_assets response has no items/assets".to_string())?
        .as_array()
        .ok_or_else(|| "stage4: list_assets items/assets is not an array".to_string())
        .cloned()
}

fn rows_to_items(rows: &[Value]) -> Result<Vec<WalletItemRow>, String> {
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let wire: AssetWire = JsonCodec
            .serialize(row)
            .and_then(|bytes| JsonCodec.deserialize(&bytes))
            .map_err(|e| format!("stage4: invalid asset wire in wallet dump: {e}"))?;
        let leaf = z00z_wallets::tx::asset_wire_to_leaf(&wire)?;

        let status = row
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("present")
            .to_string();

        out.push(WalletItemRow {
            asset_id_hex: to_hex(&leaf.asset_id),
            serial_id: wire.serial_id,
            class: format!("{:?}", wire.definition.class),
            amount: wire.amount,
            status,
        });
    }
    Ok(out)
}

fn build_serial_dist(items: &[WalletItemRow]) -> Vec<SerialDistRow> {
    let mut by_serial = BTreeMap::<u32, (usize, u64)>::new();
    for item in items {
        let e = by_serial.entry(item.serial_id).or_insert((0usize, 0u64));
        e.0 += 1;
        e.1 = e.1.saturating_add(item.amount);
    }

    by_serial
        .into_iter()
        .map(|(serial_id, (row_count, total_amount))| SerialDistRow {
            serial_id,
            row_count,
            total_amount,
        })
        .collect()
}

fn wallet_path_for_id(wallets_dir: &Path, wallet_id: &str) -> PathBuf {
    let file_id = compute_wallet_file_id(wallet_id);
    let file_hex = to_hex(&file_id[..8]);
    wallets_dir.join(format!("wallet_{file_hex}.wlt"))
}
