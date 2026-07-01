use std::{collections::HashSet, path::Path};

use z00z_core::{Asset, AssetClass, AssetWire};
use z00z_networks_rpc::RpcTransport;
use z00z_utils::codec::{json, Codec, JsonCodec, Value};
use z00z_wallets::{
    domains::hashing::compute_wallet_file_id,
    rpc::types::{common::PersistWalletId, wallet::WalletSource},
};

use crate::SimContext;

use super::{hex_str, ActorPersistStat};

pub(super) fn verify_restart(
    ctx: &SimContext,
    wallets_dir: &Path,
    actor_idxs: &[usize],
    _wallet_ids: &[String],
    per_actor_assets: &[Vec<Asset>],
    rpc_log: &Path,
) -> Result<(Vec<ActorPersistStat>, Vec<Vec<Asset>>), String> {
    if let Some(wallet_svc) = &ctx.wallet_service {
        let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
        rt.block_on(async {
            for actor_idx in actor_idxs {
                let actor = ctx
                    .actors
                    .get(*actor_idx)
                    .ok_or_else(|| format!("actor index {actor_idx} missing"))?;
                let wallet_id = PersistWalletId(actor.wallet_id.clone());
                wallet_svc
                    .lock_wallet(&wallet_id)
                    .await
                    .map_err(|e| format!("restart pre-lock({}): {e}", actor.name))?;
            }
            Ok::<(), String>(())
        })?;
    }

    let (wallet_svc, transport) =
        crate::scenario_1::stage_2::build_logged_transport(ctx, wallets_dir, rpc_log)?;
    let wallets_dir = wallets_dir.to_path_buf();

    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    rt.block_on(async move {
        fn class_code(class: AssetClass) -> u8 {
            match class {
                AssetClass::Coin => 1,
                AssetClass::Token => 2,
                AssetClass::Nft => 3,
                AssetClass::Void => 4,
            }
        }

        fn asset_key_triplet(items: &[Asset]) -> HashSet<([u8; 32], u8, u64)> {
            let mut out = HashSet::with_capacity(items.len());
            for item in items {
                out.insert((item.asset_id(), class_code(item.definition.class), item.amount));
            }
            out
        }

        async fn fetch_all_wires(
            transport: &impl RpcTransport,
            wallet_id: &str,
        ) -> Result<Vec<AssetWire>, String> {
            let mut out = Vec::new();
            let mut cursor: Option<String> = None;

            loop {
                let list = transport
                    .call(
                        "wallet.asset.list_assets",
                        json!({
                            "wallet_id": wallet_id,
                            "limit": 50,
                            "cursor": cursor,
                            "filter": null,
                        }),
                    )
                    .await
                    .map_err(|e| format!("restart list_assets({wallet_id}): {e}"))?;

                let items = list
                    .get("items")
                    .cloned()
                    .or_else(|| list.get("assets").cloned())
                    .unwrap_or_else(|| Value::Array(Vec::new()));
                let mut page: Vec<AssetWire> = JsonCodec
                    .serialize(&items)
                    .and_then(|bytes| JsonCodec.deserialize(&bytes))
                    .map_err(|e| format!("restart decode({wallet_id}): {e}"))?;
                out.append(&mut page);

                let has_more = list
                    .get("has_more")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if !has_more {
                    break;
                }

                cursor = list
                    .get("next_cursor")
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string);
                if cursor.is_none() {
                    break;
                }
            }

            Ok(out)
        }

        let mut persist_stats = Vec::with_capacity(actor_idxs.len());
        let mut persisted_assets = Vec::with_capacity(actor_idxs.len());

        for (slot, assets) in per_actor_assets.iter().enumerate() {
            let actor_idx = *actor_idxs
                .get(slot)
                .ok_or_else(|| format!("actor slot {slot} missing"))?;
            let actor = ctx
                .actors
                .get(actor_idx)
                .ok_or_else(|| format!("actor index {actor_idx} missing"))?;

            let wallet_id = actor.wallet_id.clone();

            let pass = crate::scenario_1::stage_2::actor_runtime_password(actor)
                .ok_or_else(|| format!("password mapping missing for actor {}", actor.name))?;

            let source_hash = compute_wallet_file_id(&wallet_id);
            let source_wlt =
                wallets_dir.join(format!("wallet_{}.wlt", hex_str(&source_hash[..8])));
            wallet_svc
                .open_wallet_source(WalletSource::Path {
                    path: source_wlt.to_string_lossy().to_string(),
                })
                .await
                .map_err(|e| format!("restart open_wallet_source({}): {e}", actor.name))?;

            let wallet_id_obj = PersistWalletId(wallet_id.clone());
            let safe_pass = z00z_crypto::expert::encoding::SafePassword::from(pass.as_str());
            wallet_svc
                .unlock_wallet_in_memory(&wallet_id_obj, &safe_pass)
                .await
                .map_err(|e| format!("restart unlock failed for {}: {e}", actor.name))?;

            let listed_wires = fetch_all_wires(&transport, &wallet_id).await?;
            let listed_assets: Vec<Asset> = listed_wires
                .into_iter()
                .map(|wire| wire.to_asset())
                .collect::<Result<_, _>>()
                .map_err(|e| format!("restart to_asset({}): {e}", actor.name))?;

            let expect_count = assets.len();
            let listed_count = listed_assets.len();
            let expect_sum: u128 = assets.iter().map(|item| u128::from(item.amount)).sum();
            let listed_sum: u128 = listed_assets.iter().map(|item| u128::from(item.amount)).sum();
            let expect_set = asset_key_triplet(assets);
            let listed_set = asset_key_triplet(&listed_assets);

            let is_ok = listed_count == expect_count
                && listed_sum == expect_sum
                && listed_set == expect_set;
            if !is_ok {
                return Err(format!(
                    "restart persistence mismatch for {}: expected_count={} listed_count={} expected_sum={} listed_sum={} expected_set={} listed_set={}",
                    actor.name,
                    expect_count,
                    listed_count,
                    expect_sum,
                    listed_sum,
                    expect_set.len(),
                    listed_set.len(),
                ));
            }

            persist_stats.push(ActorPersistStat {
                actor: actor.name.clone(),
                is_ok,
                expected_count: expect_count,
                listed_count,
            });
            persisted_assets.push(listed_assets);
        }

        Ok((persist_stats, persisted_assets))
    })
}
