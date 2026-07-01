use super::{
    compute_wallet_file_id, create_dir_all, decode_export_salt, decrypt_seed_phrase,
    encode_card_compact, extract_receiver_ids, from_hex, json, push_log, save_json, ActorRun,
    ActorSnap, ActorSpec, Arc, ChainId, HashMap, Hidden, Path, PersistWalletId, RpcTransport,
    SimActor, SimContext, SystemTimeProvider, TimeProvider, WalletId, WalletKernel, WalletRecord,
    WalletService, WalletSource, WalletSystemMetadata, WalletUserFields,
};
use std::{thread::sleep, time::Duration};
use z00z_utils::codec::Value;

const STAGE2_WAIT_RETRIES: u32 = 400;
const STAGE2_WAIT_MS: u64 = 50;

fn wait_for_wallet_file(wlt_path: &Path, actor_name: &str) -> Result<(), String> {
    for attempt in 0..=STAGE2_WAIT_RETRIES {
        if z00z_utils::io::path_exists(wlt_path)
            .map_err(|err| format!("verify_wlt stat {} failed: {err}", wlt_path.display()))?
        {
            return Ok(());
        }
        if attempt < STAGE2_WAIT_RETRIES {
            sleep(Duration::from_millis(STAGE2_WAIT_MS));
        }
    }

    Err(format!(
        "verify_wlt missing wallet source for {}: {}",
        actor_name,
        wlt_path.display()
    ))
}

async fn unlock_wallet_with_retry(
    transport: &impl RpcTransport,
    wallet_id: &str,
    password: &str,
    actor_name: &str,
    context: &str,
) -> Result<Value, String> {
    let mut last_err = String::new();

    for attempt in 0..=STAGE2_WAIT_RETRIES {
        match transport
            .call(
                "wallet.session.unlock_wallet",
                json!({
                    "wallet_id": wallet_id,
                    "password": password,
                }),
            )
            .await
        {
            Ok(session) => return Ok(session),
            Err(err) => {
                last_err = err.to_string();
                if attempt < STAGE2_WAIT_RETRIES {
                    sleep(Duration::from_millis(STAGE2_WAIT_MS));
                }
            }
        }
    }

    Err(format!("unlock_wallet({actor_name}) {context}: {last_err}"))
}

fn derive_secret_copy(wallet_id: &PersistWalletId, seed_phrase: &str) -> Result<[u8; 32], String> {
    const MAX_RETRY: u32 = 16;

    let seed_phrase = z00z_wallets::key::SeedPhrase24::parse_in(
        z00z_wallets::key::MnemonicLanguage::English,
        seed_phrase,
    )
    .map_err(|err| format!("parse seed phrase for {} failed: {err}", wallet_id.0))?;
    let seed_bip39 = seed_phrase
        .to_bip39_seed("")
        .map_err(|err| format!("derive bip39 seed for {} failed: {err}", wallet_id.0))?;

    let mut retry = 0u32;
    let mut bytes = z00z_crypto::hash::poseidon2_hash(
        b"z00z.wallet.receiver_secret.v1",
        &[wallet_id.0.as_bytes(), seed_bip39.reveal()],
    );

    loop {
        match z00z_wallets::key::ReceiverSecret::from_bytes(bytes) {
            Ok(secret) => return Ok(*secret.as_bytes()),
            Err(
                z00z_wallets::key::StealthKeyError::ZeroSecret
                | z00z_wallets::key::StealthKeyError::InvalidSecretKey
                | z00z_wallets::key::StealthKeyError::ZeroScalarRejected
                | z00z_wallets::key::StealthKeyError::IdentityPointRejected,
            ) if retry < MAX_RETRY => {
                retry += 1;
                let retry_bytes = retry.to_le_bytes();
                bytes = z00z_crypto::hash::poseidon2_hash(
                    b"z00z.wallet.receiver_secret.retry.v1",
                    &[
                        wallet_id.0.as_bytes(),
                        seed_bip39.reveal(),
                        &retry_bytes,
                        &bytes,
                    ],
                );
            }
            Err(err) => {
                return Err(format!(
                    "derive receiver secret for {} failed: {err}",
                    wallet_id.0
                ));
            }
        }
    }
}

pub(crate) async fn create_actor_runtime(
    ctx: &SimContext,
    transport: &impl RpcTransport,
    wallet_svc: &Arc<WalletService>,
    spec: &ActorSpec,
    stage_id: u32,
    logs: &mut Vec<String>,
    wallets_dir: &Path,
    keys_dir: &Path,
    _wallet_net: &str,
    _wallet_chain: &str,
) -> Result<(SimActor, ActorSnap, ActorRun), String> {
    let seed_phrase_param = if ctx.config.simulation.use_mock_rng {
        Some(super::deterministic_seed_phrase_24(spec.rng_seed)?)
    } else {
        None
    };

    let resp = transport
        .call(
            "app.wallet.create_wallet",
            json!({
                "name":        spec.name,
                "password":    spec.password,
                "seed_phrase": seed_phrase_param,
            }),
        )
        .await
        .map_err(|e| format!("create_wallet({}) RPC: {e}", spec.name))?;

    let persist_id_str = resp["wallet_id"]
        .as_str()
        .ok_or_else(|| format!("no wallet_id in response for {}", spec.name))?
        .to_string();
    let seed_phrase = resp["seed_phrase"]
        .as_str()
        .ok_or_else(|| format!("no seed_phrase in response for {}", spec.name))?
        .to_string();

    push_log(
        logs,
        stage_id,
        "S2-2",
        "create_wallet",
        "ok",
        &format!("{} → {}", spec.name, &persist_id_str),
    )?;

    let persist_id = PersistWalletId(persist_id_str.clone());
    let session = unlock_wallet_with_retry(
        transport,
        &persist_id_str,
        &spec.password,
        &spec.name,
        "for key-derivation",
    )
    .await?;
    let keys = wallet_svc
        .receiver_keys(&persist_id)
        .await
        .map_err(|e| format!("receiver_keys({}): {e}", spec.name))?;
    let secret_copy = derive_secret_copy(&persist_id, &seed_phrase)?;
    transport
        .call("wallet.session.lock_wallet", json!({ "session": session }))
        .await
        .map_err(|e| format!("lock_wallet({}) after key-derivation: {e}", spec.name))?;
    let card = keys
        .export_receiver_card()
        .map_err(|e| format!("export_receiver_card for {}: {e}", spec.name))?;

    push_log(
        logs,
        stage_id,
        "S2-6",
        "derive_stealth_keys",
        "ok",
        &format!(
            "{} handle={}",
            spec.name,
            super::hex_str(&keys.owner_handle)
        ),
    )?;

    let hex64 = persist_id.0.trim_start_matches("wallet_");
    let id_bytes_vec = from_hex(hex64).map_err(|e| format!("from_hex for {}: {e}", spec.name))?;
    if id_bytes_vec.len() != 32 {
        return Err(format!(
            "wallet_id hex must decode to 32 bytes for {}",
            spec.name
        ));
    }
    let mut id_bytes = [0u8; 32];
    id_bytes.copy_from_slice(&id_bytes_vec);

    let now_ms = if ctx.config.simulation.use_mock_rng {
        ctx.config.simulation.mock_rng_seed.unwrap_or(0)
    } else {
        SystemTimeProvider.compat_unix_timestamp_millis()
    };

    let kernel = WalletKernel::new(WalletId(id_bytes), ChainId::DEVNET);
    let user = WalletUserFields {
        wallet_name: spec.name.to_string(),
        memo: None,
    };
    let system = WalletSystemMetadata {
        created_at: now_ms,
        updated_at: now_ms,
    };
    let record = WalletRecord::new(kernel, user, system);

    let keys_json = json!({
        "name":         spec.name,
        "wallet_id":    super::hex_str(&id_bytes),
        "owner_handle": super::hex_str(&keys.owner_handle),
        "view_pk":      super::hex_str(&card.view_pk),
        "identity_pk":  super::hex_str(&card.identity_pk),
        "card_compact": encode_card_compact(&card),
    });
    let keys_file = keys_dir.join(format!("{}_keys.json", spec.name));
    save_json(&keys_file, &keys_json).map_err(|e| e.to_string())?;
    push_log(
        logs,
        stage_id,
        "S2-8",
        "export_keys",
        "ok",
        &keys_file.to_string_lossy(),
    )?;

    let file_hash = compute_wallet_file_id(&persist_id.0);
    let wlt_name = format!("wallet_{}.wlt", super::hex_str(&file_hash[..8]));
    let wlt_path = wallets_dir.join(&wlt_name);
    wait_for_wallet_file(&wlt_path, &spec.name)?;
    let verify_wallet_svc = WalletService::with_output_dir(wallets_dir.to_path_buf());
    verify_wallet_svc
        .open_wallet_source(WalletSource::Path {
            path: wlt_path.to_string_lossy().to_string(),
        })
        .await
        .map_err(|e| format!("open_wallet_source for {}: {e}", spec.name))?;

    push_log(
        logs,
        stage_id,
        "S2-9",
        "verify_wlt",
        "ok",
        &wlt_path.to_string_lossy(),
    )?;

    let owner_handle = super::hex_str(&keys.owner_handle);
    let view_pk = super::hex_str(&card.view_pk);
    let identity_pk = super::hex_str(&card.identity_pk);

    Ok((
        SimActor {
            name: spec.name.to_string(),
            password: Some(spec.password.to_string()),
            wallet_id: persist_id.0.clone(),
            record,
            keys,
            card,
            balance: HashMap::new(),
            receiver_secret: Hidden::hide(secret_copy),
            session: None,
        },
        ActorSnap {
            name: spec.name.to_string(),
            wallet_id: super::hex_str(&id_bytes),
            owner_handle: owner_handle.clone(),
            view_pk: view_pk.clone(),
            identity_pk: identity_pk.clone(),
            wlt_path: wlt_path.to_string_lossy().to_string(),
            wlt_verified: true,
        },
        ActorRun {
            name: spec.name.to_string(),
            password: spec.password.to_string(),
            wallet_id: persist_id.0,
            session: z00z_utils::codec::Value::Null,
            seed_phrase,
            receiver_secret_hex: super::hex_str(&secret_copy),
            owner_handle,
            view_pk,
            identity_pk,
            receiver_ids: Vec::new(),
        },
    ))
}

pub(crate) async fn enrich_actor_runtime(
    transport: &impl RpcTransport,
    stage_id: u32,
    logs: &mut Vec<String>,
    wallets_dir: &Path,
    actor: &mut ActorRun,
    sim_actor: &mut SimActor,
    idx: usize,
) -> Result<(), String> {
    let session = unlock_wallet_with_retry(
        transport,
        &actor.wallet_id,
        &actor.password,
        &actor.name,
        "RPC",
    )
    .await?;
    actor.session = session.clone();
    sim_actor.session = Some(session.clone());

    let seed_resp = transport
        .call(
            "wallet.session.show_seed_phrase",
            json!({
                "session": session,
                "password": actor.password,
                "confirmation": "I understand",
            }),
        )
        .await
        .map_err(|e| format!("show_seed_phrase({}) RPC: {e}", actor.name))?;

    let export_resp = transport
        .call(
            "app.wallet.export_wallet",
            json!({
                "wallet_id": actor.wallet_id,
                "password": actor.password,
            }),
        )
        .await
        .map_err(|e| format!("export_wallet({}) RPC: {e}", actor.name))?;
    let seed_salt = decode_export_salt(&export_resp, &actor.password)?;

    let decrypted_seed =
        decrypt_seed_phrase(&seed_resp, &actor.wallet_id, &actor.password, &seed_salt)?;
    if decrypted_seed != actor.seed_phrase {
        return Err(format!("show_seed_phrase mismatch for {}", actor.name));
    }
    push_log(
        logs,
        stage_id,
        "S2-12",
        "show_seed_integrity",
        "ok",
        &format!("{} encrypted_seed_verified=true", actor.name),
    )?;

    let path = format!("m/44'/1337'/0'/0/{}", idx);
    let derive_resp = transport
        .call(
            "wallet.key.derive_receiver",
            json!({
                "session": session,
                "path": path,
            }),
        )
        .await
        .map_err(|e| format!("derive_receiver({}) RPC: {e}", actor.name))?;

    let _derived_public_key = derive_resp["public_key"]
        .as_str()
        .ok_or_else(|| format!("derive_receiver missing public_key for {}", actor.name))?;

    let list_receivers_resp = transport
        .call(
            "wallet.key.list_receivers",
            json!({
                "session": session,
                "limit": 50,
                "cursor": z00z_utils::codec::Value::Null,
                "filter": z00z_utils::codec::Value::Null,
            }),
        )
        .await
        .map_err(|e| format!("list_receivers({}) RPC: {e}", actor.name))?;

    let mut receiver_ids = extract_receiver_ids(&list_receivers_resp);
    if receiver_ids.is_empty() {
        return Err(format!(
            "list_receivers returned zero receiver ids for {}",
            actor.name
        ));
    }
    receiver_ids.sort();
    receiver_ids.dedup();
    actor.receiver_ids = receiver_ids;

    push_log(
        logs,
        stage_id,
        "S2-13",
        "derive_list_receivers",
        "ok",
        &format!("{} receiver_ids={}", actor.name, actor.receiver_ids.len()),
    )?;

    let backup_target_dir = wallets_dir.join("backups").join(actor.name.as_str());
    create_dir_all(&backup_target_dir).map_err(|e| e.to_string())?;

    let backup_cfg_resp = transport
        .call(
            "wallet.backup.configure_backup",
            json!({
                "session": session,
                "settings": {
                    "auto_backup_enabled": false,
                    "backup_interval_hours": 24,
                    "backup_location": backup_target_dir.to_string_lossy().to_string(),
                    "encrypt_backups": true
                }
            }),
        )
        .await
        .map_err(|e| format!("configure_backup({}) RPC: {e}", actor.name))?;

    if backup_cfg_resp["settings"]["backup_location"]
        .as_str()
        .is_none()
    {
        return Err(format!(
            "configure_backup missing settings for {}",
            actor.name
        ));
    }

    let backup_resp = transport
        .call(
            "wallet.backup.create_backup",
            json!({
                "session": session,
                "password": actor.password,
                "destination": z00z_utils::codec::Value::Null,
            }),
        )
        .await
        .map_err(|e| format!("create_backup({}) RPC: {e}", actor.name))?;

    let backup_success = backup_resp["success"].as_bool().unwrap_or(false);
    if !backup_success {
        let msg = backup_resp["message"].as_str().unwrap_or("unknown error");
        return Err(format!("backup failed for {}: {}", actor.name, msg));
    }

    let list_backup_resp = transport
        .call(
            "wallet.backup.list_backups",
            json!({
                "session": session,
                "cursor": z00z_utils::codec::Value::Null,
                "limit": 50,
            }),
        )
        .await
        .map_err(|e| format!("list_backups({}) RPC: {e}", actor.name))?;

    let backup_count = list_backup_resp["items"]
        .as_array()
        .or_else(|| list_backup_resp["backups"].as_array())
        .map(|items| items.len())
        .unwrap_or(0);
    if backup_count == 0 {
        return Err(format!(
            "list_backups returned zero backups for {}",
            actor.name
        ));
    }

    let wallet_file_id = compute_wallet_file_id(&actor.wallet_id);
    let wallet_stem = hex::encode(&wallet_file_id[..8]);
    let live_history_path = wallets_dir.join(format!("wallet_{wallet_stem}_tx_history.jsonl"));
    let noncanonical_history_dir = wallets_dir.join(format!("wallet_{wallet_stem}_tx_history"));
    // The live tx-history JSONL is the canonical post-backup path. Under broad
    // release-load the file can appear shortly after the backup RPC returns, so
    // poll briefly before treating the path as missing.
    let mut live_history_ready = false;
    for attempt in 0..=STAGE2_WAIT_RETRIES {
        if z00z_utils::io::path_exists(&live_history_path)
            .map_err(|e| format!("backup_roundtrip stat live tx-history JSONL failed: {e}"))?
        {
            live_history_ready = true;
            break;
        }
        if attempt < STAGE2_WAIT_RETRIES {
            sleep(Duration::from_millis(STAGE2_WAIT_MS));
        }
    }
    if !live_history_ready {
        return Err(format!(
            "backup_roundtrip missing live tx-history JSONL for {}",
            actor.name
        ));
    }
    if z00z_utils::io::path_exists(&noncanonical_history_dir)
        .map_err(|e| format!("backup_roundtrip stat noncanonical tx-history dir failed: {e}"))?
    {
        return Err(format!(
            "backup_roundtrip found noncanonical tx-history directory for {}",
            actor.name
        ));
    }
    let history_bytes = z00z_utils::io::read_file(&live_history_path)
        .map_err(|e| format!("backup_roundtrip read live tx-history JSONL failed: {e}"))?;
    let history_rows = z00z_wallets::backup::decode_tx_history_rows(&history_bytes)
        .map_err(|e| format!("backup_roundtrip decode live tx-history JSONL failed: {e}"))?;
    if history_rows
        .iter()
        .any(|row| row.wallet_stem != wallet_stem)
    {
        return Err(format!(
            "backup_roundtrip live tx-history wallet stem mismatch for {}",
            actor.name
        ));
    }

    push_log(
        logs,
        stage_id,
        "S2-10",
        "backup_roundtrip",
        "ok",
        &format!(
            "{} backups={} live_jsonl={} rows={}",
            actor.name,
            backup_count,
            live_history_path.to_string_lossy(),
            history_rows.len()
        ),
    )?;

    Ok(())
}
