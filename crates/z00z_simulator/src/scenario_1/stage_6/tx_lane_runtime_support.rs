use super::{
    actor_runtime_password, compute_leaf_ad, compute_tag16, decode_card_compact,
    decode_output_pack, deterministic_seed_phrase_24, encode_card_compact, path_exists,
    read_to_string, AssetWire, Codec, OutputBundle, PrepFile, ReceiverCard, ReceiverCardRecord,
    RpcTransport, SimContext, Stage4FeeSinkCfg, Stage4TxPrepareCfg, TxStorage, Z00ZScalar, ZkPack,
};

use std::path::{Path, PathBuf};

use super::tx_lane_impl::FeeParty;

const TEST_TAMPER_FILE: &str = "test_hooks/stage4_output_tamper.txt";
const TEST_ROOT_TAMPER_FILE: &str = "test_hooks/stage4_root_tamper.txt";

fn load_test_tamper(out_dir: &Path) -> Result<Option<String>, String> {
    let base = out_dir.parent().unwrap_or(out_dir);
    let path = base.join(TEST_TAMPER_FILE);
    if !path_exists(&path).map_err(|e| e.to_string())? {
        return Ok(None);
    }

    let mode = read_to_string(&path).map_err(|e| {
        format!(
            "stage4: failed to read test tamper file {}: {e}",
            path.display()
        )
    })?;
    let mode = mode.trim();
    if mode.is_empty() {
        return Err(format!(
            "stage4: test tamper file {} must not be empty",
            path.display()
        ));
    }
    Ok(Some(mode.to_string()))
}

fn load_root_tamper(out_dir: &Path) -> Result<Option<String>, String> {
    let base = out_dir.parent().unwrap_or(out_dir);
    let path = base.join(TEST_ROOT_TAMPER_FILE);
    if !path_exists(&path).map_err(|e| e.to_string())? {
        return Ok(None);
    }

    let mode = read_to_string(&path).map_err(|e| {
        format!(
            "stage4: failed to read root tamper file {}: {e}",
            path.display()
        )
    })?;
    let mode = mode.trim();
    if mode.is_empty() {
        return Err(format!(
            "stage4: root tamper file {} must not be empty",
            path.display()
        ));
    }
    Ok(Some(mode.to_string()))
}

pub(super) fn apply_root_tamper(prep: &mut PrepFile, out_dir: &Path) -> Result<(), String> {
    let Some(mode) = load_root_tamper(out_dir)? else {
        return Ok(());
    };

    match mode.as_str() {
        "prev_root_hex" => {
            let first = prep
                .prev_root_hex
                .chars()
                .next()
                .ok_or_else(|| "stage4: root tamper prev_root_hex is empty".to_string())?;
            let repl = if first == '0' { '1' } else { '0' };
            prep.prev_root_hex.replace_range(0..1, &repl.to_string());
            Ok(())
        }
        other => Err(format!("stage4: unsupported root tamper mode: {other}")),
    }
}

fn tamper_value(out: &mut OutputBundle) -> Result<(), String> {
    out.value = out
        .value
        .checked_add(1)
        .ok_or_else(|| "stage4: test tamper value overflow".to_string())?;
    Ok(())
}

fn tamper_commit(out: &mut OutputBundle) -> Result<(), String> {
    let pack = decode_output_pack(out)?;
    let mut other_blind = [0u8; 32];
    other_blind[0] = 9;
    let blind = Z00ZScalar::try_from_bytes(other_blind)
        .map_err(|e| format!("stage4: test tamper blinding decode failed: {e}"))?;
    let commit = z00z_crypto::create_commitment(pack.value, &blind)
        .map_err(|e| format!("stage4: test tamper commitment build failed: {e}"))?;
    out.leaf.c_amount = commit
        .as_bytes()
        .try_into()
        .map_err(|_| "stage4: test tamper commitment size mismatch".to_string())?;
    let leaf_ad = compute_leaf_ad(
        &out.leaf.asset_id,
        out.leaf.serial_id,
        &out.leaf.r_pub,
        &out.leaf.owner_tag,
        &out.leaf.c_amount,
    );
    out.leaf.tag16 = compute_tag16(&out.k_dh, &leaf_ad);
    out.leaf.enc_pack = ZkPack::encrypt(
        &out.k_dh,
        &leaf_ad,
        &out.leaf.r_pub,
        &out.leaf.asset_id,
        out.leaf.serial_id,
        &pack.to_bytes(),
    );
    Ok(())
}

fn tamper_range(out: &mut OutputBundle) -> Result<(), String> {
    let byte = out
        .leaf
        .range_proof
        .first_mut()
        .ok_or_else(|| "stage4: test tamper range proof is empty".to_string())?;
    *byte ^= 1;
    Ok(())
}

pub(super) fn apply_test_tamper(
    outputs: &mut [OutputBundle],
    out_dir: &Path,
) -> Result<(), String> {
    let Some(mode) = load_test_tamper(out_dir)? else {
        return Ok(());
    };
    let out = outputs
        .first_mut()
        .ok_or_else(|| "stage4: test tamper requires at least one output".to_string())?;

    match mode.as_str() {
        "witness" => Ok(()),
        "tag16" => {
            out.leaf.tag16 ^= 1;
            Ok(())
        }
        "value" => tamper_value(out),
        "commit" => tamper_commit(out),
        "range" => tamper_range(out),
        other => Err(format!("stage4: unsupported test tamper mode: {other}")),
    }
}

pub(super) fn apply_wit_tamper(selected: &mut [AssetWire], out_dir: &Path) -> Result<(), String> {
    let Some(mode) = load_test_tamper(out_dir)? else {
        return Ok(());
    };
    if mode == "witness" {
        let row = selected
            .first_mut()
            .ok_or_else(|| "stage4: test tamper witness requires at least one input".to_string())?;
        let pack = row
            .enc_pack
            .as_mut()
            .ok_or_else(|| "stage4: test tamper witness requires input enc_pack".to_string())?;
        pack.tag[0] ^= 1;
    }
    Ok(())
}

pub(crate) fn validate_fee_sink(ctx: &SimContext, cfg: &Stage4FeeSinkCfg) -> Result<(), String> {
    if cfg.wallet_id.trim().is_empty() {
        return Err("stage4: transaction.fee_sink.wallet_id must not be empty".to_string());
    }
    if let Some(card) = cfg.receiver_card_hex.as_deref() {
        if card.trim().is_empty() {
            return Err(
                "stage4: transaction.fee_sink.receiver_card_hex must not be empty".to_string(),
            );
        }
        decode_card_compact(card)
            .map_err(|e| format!("stage4: invalid fee sink receiver card: {e}"))?;
    }
    let is_actor = ctx
        .actors
        .iter()
        .any(|item| item.name.eq_ignore_ascii_case(&cfg.wallet_id));
    if !is_actor && cfg.password.as_deref().unwrap_or("").trim().is_empty() {
        return Err("stage4: external fee sink requires transaction.fee_sink.password".to_string());
    }
    Ok(())
}

pub(super) fn fee_capture<'a>(
    actors: &'a [crate::SimActor],
    fee: &'a FeeParty,
) -> Option<(&'a str, &'a str, &'a str)> {
    if actors.iter().any(|item| item.wallet_id == fee.wallet_id) {
        return None;
    }
    fee.password
        .as_deref()
        .map(|pass| (fee.actor.as_str(), fee.wallet_id.as_str(), pass))
}

pub(crate) fn validate_tx_mode(cfg: &Stage4TxPrepareCfg) -> Result<(), String> {
    match cfg.transaction.mode.as_str() {
        "fraction" => {
            let fraction = cfg.transaction.fraction.ok_or_else(|| {
                "stage4: transaction.fraction is required when transaction.mode=fraction"
                    .to_string()
            })?;
            if !(fraction > 0.0 && fraction <= 1.0) {
                return Err("stage4: transaction.fraction must be in range (0, 1]".to_string());
            }
            Ok(())
        }
        "amount" if cfg.transaction.amount.is_none() => {
            Err("stage4: transaction.amount is required when transaction.mode=amount".to_string())
        }
        "amount" => {
            let amount = cfg.transaction.amount.expect("checked amount presence");
            if amount == 0 {
                return Err("stage4: transaction.amount must be > 0".to_string());
            }
            Ok(())
        }
        _ => Err("stage4: transaction.mode must be 'fraction' or 'amount'".to_string()),
    }
}

pub(super) async fn resolve_fee_party(
    ctx: &SimContext,
    transport: &impl RpcTransport,
    cfg: &Stage4TxPrepareCfg,
) -> Result<FeeParty, String> {
    let fee_name = cfg.transaction.fee_sink.wallet_id.clone();
    if let Some(actor) = ctx
        .actors
        .iter()
        .find(|item| item.name.eq_ignore_ascii_case(&fee_name))
    {
        let actor_card = encode_card_compact(&actor.card);
        if let Some(expect) = cfg.transaction.fee_sink.receiver_card_hex.as_deref() {
            if expect != actor_card {
                return Err(
                    "stage4: fee sink receiver card does not match configured actor wallet"
                        .to_string(),
                );
            }
        }
        return Ok(FeeParty {
            actor: actor.name.clone(),
            wallet_id: actor.wallet_id.clone(),
            password: actor_runtime_password(actor),
            card: actor.card.clone(),
        });
    }

    let fee_pass = cfg.transaction.fee_sink.password.clone().ok_or_else(|| {
        "stage4: external fee sink requires transaction.fee_sink.password".to_string()
    })?;
    let wallet_id = match find_wallet_id(transport, &fee_name).await? {
        Some(wallet_id) => wallet_id,
        None => create_fee_wallet(transport, cfg, &fee_name, &fee_pass).await?,
    };
    let card = load_wallet_card(transport, cfg, &wallet_id, &fee_pass).await?;
    if let Some(expect) = cfg.transaction.fee_sink.receiver_card_hex.as_deref() {
        let actual = encode_card_compact(&card);
        if actual != expect {
            return Err(
                "stage4: fee sink receiver card does not match provisioned wallet".to_string(),
            );
        }
    }

    Ok(FeeParty {
        actor: fee_name,
        wallet_id,
        password: Some(fee_pass),
        card,
    })
}

async fn find_wallet_id(
    transport: &impl RpcTransport,
    wallet_name: &str,
) -> Result<Option<String>, String> {
    let listed = transport
        .call("app.wallet.list_wallets", z00z_utils::codec::json!({}))
        .await
        .map_err(|e| format!("stage4: list_wallets RPC failed: {e}"))?;
    let rows = listed
        .as_array()
        .ok_or_else(|| "stage4: list_wallets response must be an array".to_string())?;
    Ok(rows.iter().find_map(|row| {
        if row["name"]
            .as_str()
            .is_some_and(|name| name.eq_ignore_ascii_case(wallet_name))
        {
            row.get("wallet_id")
                .and_then(|item| item.as_str())
                .map(|item| item.to_string())
        } else {
            None
        }
    }))
}

async fn create_fee_wallet(
    transport: &impl RpcTransport,
    cfg: &Stage4TxPrepareCfg,
    wallet_name: &str,
    wallet_pass: &str,
) -> Result<String, String> {
    let seed_phrase = cfg
        .transaction
        .fee_sink
        .rng_seed
        .map(deterministic_seed_phrase_24)
        .transpose()?;
    let created = transport
        .call(
            "app.wallet.create_wallet",
            z00z_utils::codec::json!({
                "name": wallet_name,
                "password": wallet_pass,
                "seed_phrase": seed_phrase,
            }),
        )
        .await
        .map_err(|e| format!("stage4: create fee wallet RPC failed: {e}"))?;
    created
        .get("wallet_id")
        .and_then(|item| item.as_str())
        .map(|item| item.to_string())
        .ok_or_else(|| "stage4: fee wallet create response missing wallet_id".to_string())
}

async fn load_wallet_card(
    transport: &impl RpcTransport,
    cfg: &Stage4TxPrepareCfg,
    wallet_id: &str,
    wallet_pass: &str,
) -> Result<ReceiverCard, String> {
    let session = transport
        .call(
            &cfg.rpc.unlock_method,
            z00z_utils::codec::json!({
                "wallet_id": wallet_id,
                "password": wallet_pass,
            }),
        )
        .await
        .map_err(|e| format!("stage4: unlock fee wallet for card RPC failed: {e}"))?;
    let card_res = transport
        .call(
            "wallet.key.get_receiver_card",
            z00z_utils::codec::json!({"session": session}),
        )
        .await;
    let lock_res = transport
        .call(
            &cfg.rpc.lock_method,
            z00z_utils::codec::json!({"session": session}),
        )
        .await;

    let card_compact = match card_res {
        Ok(card) => {
            lock_res.map_err(|e| format!("stage4: lock fee wallet for card RPC failed: {e}"))?;
            card["card_compact"]
                .as_str()
                .map(|item| item.to_string())
                .ok_or_else(|| {
                    "stage4: get_receiver_card response missing card_compact".to_string()
                })?
        }
        Err(err) => {
            if let Err(lock_err) = lock_res {
                return Err(format!(
                    "stage4: get_receiver_card for fee wallet failed: {err}; stage4: fee wallet lock on failure RPC failed: {lock_err}"
                ));
            }
            return Err(format!(
                "stage4: get_receiver_card for fee wallet failed: {err}"
            ));
        }
    };

    ReceiverCardRecord::from_compact(&card_compact, None)
        .and_then(|record| record.decode_card())
        .map_err(|e| format!("stage4: invalid fee wallet receiver card: {e}"))
}

pub(crate) struct Stage4ResolvedPaths {
    pub(crate) outputs_dir: PathBuf,
    pub(crate) logs_dir: PathBuf,
    pub(crate) transactions_dir: PathBuf,
    pub(crate) wallets_dir: PathBuf,
    pub(crate) tx_pkg_file: PathBuf,
    pub(crate) snapshot_file: PathBuf,
    pub(crate) logger_file: PathBuf,
    pub(crate) rpc_logger_file: PathBuf,
    pub(crate) alice_keys_file: PathBuf,
    pub(crate) bob_keys_file: PathBuf,
    pub(crate) wallets_state_before_file: Option<PathBuf>,
    pub(crate) wallets_state_after_file: Option<PathBuf>,
    pub(crate) wallets_state_diff_file: Option<PathBuf>,
    pub(crate) wallets_state_report_md_file: Option<PathBuf>,
    pub(crate) wallets_state_report_xlsx_file: Option<PathBuf>,
}
