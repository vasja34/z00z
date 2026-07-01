use z00z_utils::codec::Value;
use z00z_utils::io::atomic_write_file_private;
use z00z_utils::io::{read_to_string, write_file};
use z00z_wallets::security::WalletEncryption;

use super::{
    aead, format_system_time_local, from_hex, path_exists, Codec, HashMap, JsonCodec, Path,
    SafePassword, Serialize, SystemTimeProvider, TimeProvider, WalletService,
};

#[derive(Debug, Serialize)]
pub(crate) struct Stage2Snap {
    pub(crate) stage: u32,
    pub(crate) chain_id: String,
    pub(crate) wallet_count: usize,
    pub(crate) out_dir: String,
    pub(crate) actors_ready_for_stage3: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct ActorSnap {
    pub(crate) name: String,
    pub(crate) wallet_id: String,
    pub(crate) owner_handle: String,
    pub(crate) view_pk: String,
    pub(crate) identity_pk: String,
    pub(crate) wlt_path: String,
    pub(crate) wlt_verified: bool,
}

#[derive(Debug, Serialize)]
struct LogRow {
    timestamp: String,
    stage: u32,
    step: String,
    event: String,
    status: String,
    detail: String,
}

pub(crate) struct ActorRun {
    pub(crate) name: String,
    pub(crate) password: String,
    pub(crate) wallet_id: String,
    pub(crate) session: Value,
    pub(crate) seed_phrase: String,
    pub(crate) receiver_secret_hex: String,
    pub(crate) owner_handle: String,
    pub(crate) view_pk: String,
    pub(crate) identity_pk: String,
    pub(crate) receiver_ids: Vec<String>,
}

pub(crate) fn push_log(
    logs: &mut Vec<String>,
    stage: u32,
    step: &str,
    event: &str,
    status: &str,
    detail: &str,
) -> Result<(), String> {
    let row = LogRow {
        timestamp: format_system_time_local(SystemTimeProvider.now()),
        stage,
        step: step.to_string(),
        event: event.to_string(),
        status: status.to_string(),
        detail: detail.to_string(),
    };
    let codec = JsonCodec;
    let bytes = codec.serialize(&row).map_err(|e| e.to_string())?;
    let line = String::from_utf8(bytes).map_err(|e| e.to_string())?;
    logs.push(line);
    Ok(())
}

pub(crate) fn flush_logs(path: &Path, logs: &[String]) -> Result<(), String> {
    let mut body = String::new();
    if path_exists(path).map_err(|e| e.to_string())? {
        body = read_to_string(path).map_err(|e| e.to_string())?;
        if !body.is_empty() && !body.ends_with('\n') {
            body.push('\n');
        }
    }
    body.push_str(&logs.join("\n"));
    body.push('\n');
    write_file(path, body.as_bytes()).map_err(|e| e.to_string())
}

pub(crate) fn write_wlt_map_txt(path: &Path, actors: &[ActorSnap]) -> Result<(), String> {
    let mut body = String::from("# Wallet File Mapping (Stage 2)\n\n");
    body.push_str("name | wallet_id | wlt_path\n");
    body.push_str("-----|-----------|---------\n");
    for actor in actors {
        body.push_str(&format!(
            "{} | {} | {}\n",
            actor.name, actor.wallet_id, actor.wlt_path
        ));
    }
    write_file(path, body.as_bytes()).map_err(|e| e.to_string())
}

pub(crate) fn debug_write_wallet_secrets_md(
    path: &Path,
    actors: &[ActorRun],
) -> Result<(), String> {
    let mut body = String::from("# Wallet Secrets (Stage 2) [DEBUG]\n\n");
    body.push_str("⚠️ Sensitive simulator artifact. Never use these secrets in production.\n");
    body.push_str("🔑 Includes passwords, seed phrases, and key material.\n\n");
    body.push_str("name | wallet_id | password | seed_phrase | receiver_secret_hex | owner_handle | view_pk | identity_pk | receiver_ids\n");
    body.push_str("-----|-----------|----------|-------------|---------------------|-------------|--------|-------------|-------------\n");
    for actor in actors {
        let receiver_ids = actor.receiver_ids.join("<br>");
        body.push_str(&format!(
            "{} | {} | {} | {} | {} | {} | {} | {} | {}\n",
            actor.name,
            actor.wallet_id,
            actor.password,
            actor.seed_phrase,
            actor.receiver_secret_hex,
            actor.owner_handle,
            actor.view_pk,
            actor.identity_pk,
            receiver_ids
        ));
    }
    atomic_write_file_private(path, body.as_bytes()).map_err(|e| e.to_string())
}

#[cfg(feature = "wallet_debug_tools")]
pub(crate) fn norm_seed(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

#[cfg(feature = "wallet_debug_tools")]
pub(crate) fn read_seed_md(path: &Path, actor_name: &str) -> Result<String, String> {
    let text = read_to_string(path).map_err(|e| e.to_string())?;
    for line in text.lines() {
        if !line.contains('|') {
            continue;
        }
        let cols: Vec<&str> = line.split('|').map(|x| x.trim()).collect();
        if cols.len() < 5 {
            continue;
        }
        if cols[0] == "name" || cols[0].starts_with("-----") {
            continue;
        }
        if cols[0].eq_ignore_ascii_case(actor_name) {
            return Ok(cols[3].to_string());
        }
    }
    Err(format!(
        "seed_phrase row not found in markdown for {actor_name}"
    ))
}

pub(crate) fn extract_receiver_ids(resp: &Value) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(items) = resp["items"].as_array() {
        for item in items {
            if let Some(receiver_id) = item["receiver_id"].as_str() {
                out.push(receiver_id.to_string());
            } else if let Some(public_key) = item["public_key"].as_str() {
                out.push(public_key.to_string());
            }
        }
    }
    if let Some(items) = resp["receivers"].as_array() {
        for item in items {
            if let Some(receiver_id) = item["receiver_id"].as_str() {
                out.push(receiver_id.to_string());
            } else if let Some(public_key) = item["public_key"].as_str() {
                out.push(public_key.to_string());
            } else if let Some(receiver_id) = item.as_str() {
                out.push(receiver_id.to_string());
            }
        }
    }
    out
}

pub(crate) fn decrypt_seed_phrase(
    seed_resp: &Value,
    wallet_id: &str,
    password: &str,
    seed_salt: &[u8; 16],
) -> Result<String, String> {
    let ciphertext = seed_resp["encrypted_payload"]["ciphertext"]
        .as_str()
        .ok_or_else(|| format!("encrypted_payload.ciphertext missing, response: {seed_resp}"))?;
    let algorithm = seed_resp["encrypted_payload"]["metadata"]["algorithm"]
        .as_str()
        .ok_or_else(|| "encrypted_payload.metadata.algorithm missing".to_string())?;
    let nonce = seed_resp["encrypted_payload"]["metadata"]["nonce"]
        .as_str()
        .ok_or_else(|| "encrypted_payload.metadata.nonce missing".to_string())?;
    let key_derivation = seed_resp["encrypted_payload"]["metadata"]["key_derivation"]
        .as_str()
        .ok_or_else(|| "encrypted_payload.metadata.key_derivation missing".to_string())?;

    if algorithm != "xchacha20poly1305" {
        return Err(format!("unexpected encryption algorithm: {algorithm}"));
    }
    if !key_derivation.to_ascii_lowercase().contains("argon2id") {
        return Err(format!("unexpected key_derivation: {key_derivation}"));
    }

    let nonce_hex = nonce.strip_prefix("0x").unwrap_or(nonce);
    let nonce_bytes = from_hex(nonce_hex).map_err(|e| format!("nonce invalid hex: {e}"))?;
    if nonce_bytes.len() != aead::XCHACHA_NONCE_SIZE {
        return Err(format!("nonce len must be 24, got {}", nonce_bytes.len()));
    }
    let mut nonce_arr = [0u8; aead::XCHACHA_NONCE_SIZE];
    nonce_arr.copy_from_slice(&nonce_bytes);

    let cipher = from_hex(ciphertext).map_err(|e| format!("ciphertext is not valid hex: {e}"))?;
    let aad = aead::build_aad_multipart("wallet.seed_phrase_response", &[wallet_id.as_bytes()])
        .map_err(|e| e.to_string())?;
    let mut key = WalletEncryption::derive_key(&SafePassword::from(password), seed_salt)
        .map_err(|e| e.to_string())?;

    let mut envelope = Vec::with_capacity(1 + nonce_arr.len() + cipher.len());
    envelope.push(aead::XCHACHA20_POLY1305_ID);
    envelope.extend_from_slice(&nonce_arr);
    envelope.extend_from_slice(&cipher);
    let recovered = aead::open(&key, &aad, &envelope).map_err(|e| e.to_string())?;
    key.fill(0);
    String::from_utf8(recovered).map_err(|e| e.to_string())
}

pub(crate) fn decode_export_salt(export_resp: &Value, password: &str) -> Result<[u8; 16], String> {
    let export = JsonCodec
        .serialize(&export_resp["encrypted_payload"])
        .and_then(|bytes| {
            JsonCodec
                .deserialize::<z00z_wallets::rpc::types::common::RuntimeEncryptedResponse>(&bytes)
        })
        .map_err(|e| format!("invalid export payload: {e}"))?;

    WalletService::decode_export_seed_salt(&export, &SafePassword::from(password))
        .map_err(|e| e.to_string())
}

pub(crate) fn validate_rpc_log_privacy(path: &Path, actors: &[ActorRun]) -> Result<(), String> {
    let text = read_rpc_log_bundle(path)?;
    if text.trim().is_empty() {
        return Err("rpc logger file is empty".to_string());
    }

    for actor in actors {
        if text.contains(&actor.password) {
            return Err(format!("rpc log leaked password for {}", actor.name));
        }
        if text.contains(&actor.seed_phrase) {
            return Err(format!("rpc log leaked seed phrase for {}", actor.name));
        }
    }

    let mut risk_by_method: HashMap<String, String> = HashMap::new();
    for line in text.lines() {
        let value = parse_rpc_log_line(line)?;
        let method = value["method"].as_str().unwrap_or("");
        let risk = value["risk"].as_str().unwrap_or("");
        if !method.is_empty() && !risk.is_empty() {
            risk_by_method
                .entry(method.to_string())
                .or_insert(risk.to_string());
        }
    }

    let expected = [
        ("app.wallet.create_wallet", "high"),
        ("wallet.session.unlock_wallet", "high"),
        ("wallet.session.show_seed_phrase", "critical"),
        ("wallet.key.derive_receiver", "medium"),
        ("wallet.key.list_receivers", "medium"),
        ("wallet.backup.create_backup", "low"),
    ];
    for (method, risk) in expected {
        let got = risk_by_method
            .get(method)
            .ok_or_else(|| format!("rpc log missing method: {method}"))?;
        if got != risk {
            return Err(format!(
                "risk mismatch for {method}: expected {risk}, got {got}"
            ));
        }
    }

    Ok(())
}

fn read_rpc_log_bundle(path: &Path) -> Result<String, String> {
    let mut parts = Vec::new();
    parts.push(read_to_string(path).map_err(|e| e.to_string())?);

    for idx in 1..=16 {
        let rotated = std::path::PathBuf::from(format!("{}.{}", path.to_string_lossy(), idx));
        if !path_exists(&rotated).map_err(|e| e.to_string())? {
            continue;
        }
        parts.push(read_to_string(&rotated).map_err(|e| e.to_string())?);
    }

    Ok(parts.join("\n"))
}

fn parse_rpc_log_line(line: &str) -> Result<Value, String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Err("rpc logger line is empty".to_string());
    }

    if let Ok(value) = JsonCodec.deserialize(trimmed.as_bytes()) {
        return Ok(value);
    }

    let json_start = trimmed
        .find('{')
        .ok_or_else(|| format!("rpc logger line has no JSON payload: {trimmed}"))?;
    JsonCodec
        .deserialize(&trimmed.as_bytes()[json_start..])
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_rpc_log_line;

    #[test]
    fn test_parse_log_line_json() {
        let value = parse_rpc_log_line(r#"{"method":"wallet.key.list_receivers","risk":"medium"}"#)
            .expect("parse json-only line");
        assert_eq!(value["method"].as_str(), Some("wallet.key.list_receivers"));
        assert_eq!(value["risk"].as_str(), Some("medium"));
    }

    #[test]
    fn test_log_line_prefixed_log() {
        let value = parse_rpc_log_line(
            r#"[2026-03-29 16:39:59.680] [INFO] {"method":"wallet.session.show_seed_phrase","risk":"critical"}"#,
        )
        .expect("parse prefixed logger line");
        assert_eq!(
            value["method"].as_str(),
            Some("wallet.session.show_seed_phrase")
        );
        assert_eq!(value["risk"].as_str(), Some("critical"));
    }
}
