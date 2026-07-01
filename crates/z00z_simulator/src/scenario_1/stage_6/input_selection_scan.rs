use std::collections::BTreeSet;

use z00z_core::AssetWire;
use z00z_networks_rpc::RpcTransport;
use z00z_utils::codec::{json, Codec, JsonCodec, Value};
use z00z_wallets::tx::{pick_input_rows, AssetSelCfg};

use crate::config::{Stage4SelectionCfg, Stage4TxPrepareCfg};

use super::parse_asset_class;

pub(crate) async fn list_sender_inputs_distinct_serials(
    transport: &dyn RpcTransport,
    cfg: &Stage4TxPrepareCfg,
    wallet_id: &str,
    recv_sec: [u8; 32],
) -> Result<Vec<AssetWire>, String> {
    let target = distinct_serial_target(&cfg.transaction.input_assets_selection)?;
    let mut cursor = Value::Null;
    let mut pages = 0usize;
    let mut rows = Vec::<AssetWire>::new();

    while pages < 64 {
        pages += 1;
        let listed = transport
            .call(
                &cfg.rpc.list_assets_method,
                json!({
                    "wallet_id": wallet_id,
                    "limit": cfg.rpc.list_limit,
                    "cursor": cursor,
                    "filter": {
                        "asset_class": cfg.rpc.list_filter.asset_class,
                        "min_balance": cfg.rpc.list_filter.min_balance,
                    },
                }),
            )
            .await
            .map_err(|e| format!("stage4: list_assets sender RPC failed: {e}"))?;
        let mut page_rows = parse_list_settlement_rows(&listed)?;
        rows.append(&mut page_rows);

        if distinct_serial_count_rows(&rows, cfg) >= target {
            break;
        }

        match extract_next_cursor(&listed) {
            Some(next) => cursor = next,
            None => break,
        }
    }

    pick_sender_rows(rows, recv_sec, cfg)
}

pub(crate) fn pick_sender_rows(
    rows: Vec<AssetWire>,
    recv_sec: [u8; 32],
    cfg: &Stage4TxPrepareCfg,
) -> Result<Vec<AssetWire>, String> {
    let mut seen = BTreeSet::new();
    let mut rows: Vec<([u8; 32], AssetWire)> = rows
        .into_iter()
        .filter_map(|row| match canonical_input_asset_id(recv_sec, &row) {
            Ok(Some(asset_id)) if seen.insert(asset_id) => Some(Ok((asset_id, row))),
            Ok(Some(_)) => None,
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        })
        .collect::<Result<Vec<_>, String>>()?;

    rows.sort_by(|(left_id, left), (right_id, right)| {
        left.serial_id
            .cmp(&right.serial_id)
            .then_with(|| left_id.cmp(right_id))
            .then_with(|| left.amount.cmp(&right.amount))
    });
    let rows: Vec<AssetWire> = rows.into_iter().map(|(_, row)| row).collect();

    let sel = AssetSelCfg {
        distinct_serial_ids_min: cfg
            .transaction
            .input_assets_selection
            .distinct_serial_ids_min,
        distinct_serial_ids_target: cfg
            .transaction
            .input_assets_selection
            .distinct_serial_ids_target,
        distinct_serial_ids_max: cfg
            .transaction
            .input_assets_selection
            .distinct_serial_ids_max,
    };

    pick_input_rows(
        rows,
        parse_asset_class(&cfg.transaction.class)?,
        &cfg.transaction.symbol,
        sel,
    )
    .map_err(|e| format!("stage4: {e}"))
}

pub(crate) fn canonical_input_asset_id(
    recv_sec: [u8; 32],
    row: &AssetWire,
) -> Result<Option<[u8; 32]>, String> {
    if row.r_pub.is_none()
        || row.owner_tag.is_none()
        || row.enc_pack.is_none()
        || row.tag16.is_none()
    {
        return Ok(None);
    }

    let s_in = match z00z_wallets::tx::resolve_input_secret(recv_sec, row) {
        Ok(secret) => secret,
        Err(err) if err.contains("not decryptable for sender secret") => return Ok(None),
        Err(err) => return Err(err),
    };
    let _ = s_in;
    z00z_wallets::tx::asset_wire_to_leaf(row).map(|leaf| Some(leaf.asset_id))
}

pub(crate) fn distinct_serial_count_rows(rows: &[AssetWire], cfg: &Stage4TxPrepareCfg) -> usize {
    let class = parse_asset_class(&cfg.transaction.class).ok();
    let symbol = cfg.transaction.symbol.to_ascii_lowercase();
    rows.iter()
        .filter(|w| match &class {
            Some(cls) => &w.definition.class == cls,
            None => false,
        })
        .filter(|w| w.definition.symbol.to_ascii_lowercase() == symbol)
        .map(|w| w.serial_id)
        .collect::<std::collections::BTreeSet<u32>>()
        .len()
}

pub(crate) fn extract_next_cursor(listed: &Value) -> Option<Value> {
    for key in ["next_cursor", "nextCursor", "cursor_next", "next"] {
        if let Some(value) = listed.get(key) {
            if !value.is_null() {
                return Some(value.clone());
            }
        }
    }

    if let Some(nested) = listed.get("pagination") {
        for key in ["next_cursor", "nextCursor", "next"] {
            if let Some(value) = nested.get(key) {
                if !value.is_null() {
                    return Some(value.clone());
                }
            }
        }
    }
    None
}

pub(crate) fn parse_list_settlement_rows(listed: &Value) -> Result<Vec<AssetWire>, String> {
    let items_val = listed
        .get("items")
        .or_else(|| listed.get("assets"))
        .ok_or_else(|| "stage4: list_assets response has no items/assets".to_string())?;
    let rows = items_val
        .as_array()
        .ok_or_else(|| "stage4: list_assets items/assets is not an array".to_string())?;

    let mut all_rows: Vec<AssetWire> = Vec::with_capacity(rows.len());
    for row in rows {
        let wire: AssetWire = JsonCodec
            .serialize(row)
            .and_then(|bytes| JsonCodec.deserialize(&bytes))
            .map_err(|e| format!("stage4: invalid asset wire in list_assets: {e}"))?;
        all_rows.push(wire);
    }

    Ok(all_rows)
}

pub(crate) fn distinct_serial_target(selection: &Stage4SelectionCfg) -> Result<usize, String> {
    let target_u32 = if selection.distinct_serial_ids_target == 0 {
        selection.distinct_serial_ids_min
    } else {
        selection.distinct_serial_ids_target
    };
    if target_u32 == 0 {
        return Err("stage4: distinct serial target must be > 0".to_string());
    }
    if selection.distinct_serial_ids_max == 0 || selection.distinct_serial_ids_max > 10 {
        return Err("stage4: distinct_serial_ids_max must be in range 1..=10".to_string());
    }
    if target_u32 > selection.distinct_serial_ids_max {
        return Err(format!(
            "stage4: target distinct serials {} exceeds configured max {}",
            target_u32, selection.distinct_serial_ids_max
        ));
    }
    Ok(target_u32 as usize)
}

pub(crate) fn unique_serial_count(rows: &[AssetWire]) -> usize {
    rows.iter()
        .map(|row| row.serial_id)
        .collect::<std::collections::BTreeSet<u32>>()
        .len()
}
