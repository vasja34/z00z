//! Sanitization and summary helpers.

use crate::domains::hashing::compute_fingerprint;
use crate::rpc::logging::config::RpcLoggingTruncationConfig;
use crate::rpc::methods::tx_runtime_state::tx_package_summary;
use crate::rpc::types::tx::PortableWalletTxPackage;
use crate::tx::{ThinWalletTxPackage, TxPackage};
use z00z_utils::codec::{Codec, JsonCodec, Map, Value};

fn bytes_array_to_hex(value: &Value) -> Option<String> {
    let arr = value.as_array()?;

    let mut bytes = Vec::with_capacity(arr.len());
    for v in arr {
        let n = v.as_u64()?;
        if n > 255 {
            return None;
        }
        bytes.push(n as u8);
    }

    Some(hex::encode(bytes))
}

fn summarize_asset_id(asset_id: &Value, trunc: &RpcLoggingTruncationConfig) -> Value {
    if let Some(s) = asset_id.as_str() {
        return Value::String(truncate_non_secret(s, trunc));
    }

    if let Some(hex) = bytes_array_to_hex(asset_id) {
        return Value::String(truncate_non_secret(&hex, trunc));
    }

    object_summary(asset_id)
}

fn summarize_tx_package_blob(tx_data: &str, trunc: &RpcLoggingTruncationConfig) -> Value {
    let codec = JsonCodec;
    if let Ok(package) = codec.deserialize::<TxPackage>(tx_data.as_bytes()) {
        return summarize_tx_package(&package, tx_data.as_bytes(), trunc);
    }

    if let Ok(thin) = codec.deserialize::<ThinWalletTxPackage>(tx_data.as_bytes()) {
        return summarize_thin_tx_package(&thin, trunc);
    }

    Value::String("<redacted>".to_string())
}

fn summarize_portable_tx_blob(tx_data: &str, trunc: &RpcLoggingTruncationConfig) -> Value {
    let codec = JsonCodec;
    let Ok(portable) = codec.deserialize::<PortableWalletTxPackage>(tx_data.as_bytes()) else {
        return Value::String("<redacted>".to_string());
    };

    let mut out = Map::new();
    out.insert(
        "package_version".to_string(),
        Value::Number((portable.package_version as u64).into()),
    );
    out.insert(
        "chain_id".to_string(),
        Value::String(truncate_non_secret(&portable.chain_id, trunc)),
    );
    out.insert(
        "tx_hash_hex".to_string(),
        Value::String(truncate_non_secret(&portable.tx_hash_hex, trunc)),
    );
    out.insert(
        "metadata_hash_hex".to_string(),
        Value::String(truncate_non_secret(&portable.metadata_hash_hex, trunc)),
    );

    let embedded = match codec.deserialize::<TxPackage>(&portable.tx_bytes) {
        Ok(package) => summarize_tx_package(&package, &portable.tx_bytes, trunc),
        Err(_) => Value::String("<redacted>".to_string()),
    };
    out.insert("tx_package".to_string(), embedded);
    Value::Object(out)
}

fn summarize_tx_package(
    package: &TxPackage,
    tx_bytes: &[u8],
    trunc: &RpcLoggingTruncationConfig,
) -> Value {
    let mut out = Map::new();
    out.insert("kind".to_string(), Value::String(package.kind.clone()));
    out.insert(
        "package_type".to_string(),
        Value::String(package.package_type.clone()),
    );
    out.insert(
        "version".to_string(),
        Value::Number((package.version as u64).into()),
    );
    out.insert(
        "chain_id".to_string(),
        Value::Number((package.chain_id as u64).into()),
    );
    out.insert("status".to_string(), Value::String(package.status.clone()));
    out.insert(
        "tx_digest_hex".to_string(),
        Value::String(truncate_non_secret(&package.tx_digest_hex, trunc)),
    );

    if let Some(summary) = tx_package_summary(tx_bytes) {
        out.insert("amount".to_string(), Value::Number(summary.amount.into()));
        out.insert("fee".to_string(), Value::Number(summary.fee.into()));
        out.insert(
            "inputs_count".to_string(),
            Value::Number((summary.inputs.len() as u64).into()),
        );
        out.insert(
            "outputs_count".to_string(),
            Value::Number((summary.outputs.len() as u64).into()),
        );
    }

    Value::Object(out)
}

fn summarize_thin_tx_package(
    thin: &ThinWalletTxPackage,
    trunc: &RpcLoggingTruncationConfig,
) -> Value {
    let mut out = Map::new();
    out.insert(
        "transport_mode".to_string(),
        Value::String("thin".to_string()),
    );
    out.insert(
        "package_version".to_string(),
        Value::Number((thin.package_version as u64).into()),
    );
    out.insert(
        "chain_id".to_string(),
        Value::String(truncate_non_secret(&thin.chain_id, trunc)),
    );
    out.insert(
        "package_kind".to_string(),
        Value::String(thin.package_kind.clone()),
    );
    out.insert(
        "package_type".to_string(),
        Value::String(thin.package_type.clone()),
    );
    out.insert(
        "tx_hash_hex".to_string(),
        Value::String(truncate_non_secret(&thin.tx_hash_hex, trunc)),
    );
    out.insert(
        "input_refs_count".to_string(),
        Value::Number((thin.input_refs.len() as u64).into()),
    );
    Value::Object(out)
}

pub fn fingerprint(value: &str) -> String {
    let hash = compute_fingerprint(value);
    format!("<fingerprint:{}>", hex::encode(&hash[..4]))
}

pub fn truncate_non_secret(value: &str, trunc: &RpcLoggingTruncationConfig) -> String {
    if value.len() <= trunc.non_secret_min_bytes {
        return value.to_string();
    }

    let head = value.chars().take(trunc.head_chars).collect::<String>();
    let tail = value
        .chars()
        .rev()
        .take(trunc.tail_chars)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();

    format!("<len={} {}...{}>", value.chars().count(), head, tail)
}

pub fn object_summary(value: &Value) -> Value {
    match value {
        Value::Array(arr) => Value::String(format!("<array len={}>", arr.len())),
        Value::Object(map) => Value::String(format!("<object keys={}>", map.len())),
        _ => Value::Null,
    }
}

pub fn extract_wallet_id(params: &Value) -> Option<String> {
    params
        .get("session")
        .and_then(|v| v.as_object())
        .and_then(|session| session.get("wallet_id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            params
                .get("wallet_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
}

pub fn summarize_params(
    method: &str,
    params: &Value,
    trunc: &RpcLoggingTruncationConfig,
) -> Option<Value> {
    let obj = params.as_object()?;

    match method {
        "app.wallet.create_wallet" => {
            let mut out = Map::new();
            if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
                out.insert(
                    "name".to_string(),
                    Value::String(truncate_non_secret(name, trunc)),
                );
            }
            out.insert(
                "password".to_string(),
                Value::String("<redacted>".to_string()),
            );
            if obj.get("seed_phrase").is_some() {
                out.insert(
                    "seed_phrase".to_string(),
                    Value::String("<redacted>".to_string()),
                );
            }
            Some(Value::Object(out))
        }
        "wallet.session.unlock_wallet" => {
            let mut out = Map::new();
            if let Some(id) = extract_wallet_id(params) {
                out.insert(
                    "wallet_id".to_string(),
                    Value::String(truncate_non_secret(&id, trunc)),
                );
            }
            out.insert(
                "password".to_string(),
                Value::String("<redacted>".to_string()),
            );
            Some(Value::Object(out))
        }
        "wallet.session.show_seed_phrase" => {
            let mut out = Map::new();

            if let Some(session) = obj.get("session").and_then(|v| v.as_object()) {
                if let Some(wallet_id) = session.get("wallet_id").and_then(|v| v.as_str()) {
                    out.insert(
                        "wallet_id".to_string(),
                        Value::String(truncate_non_secret(wallet_id, trunc)),
                    );
                }
                if session.get("token").is_some() {
                    out.insert(
                        "session_token".to_string(),
                        Value::String("<redacted>".to_string()),
                    );
                }
            }
            Some(Value::Object(out))
        }
        "wallet.key.rotate_master_key" => {
            let mut out = Map::new();
            if let Some(wallet_id) = obj.get("wallet_id").and_then(|v| v.as_str()) {
                out.insert(
                    "wallet_id".to_string(),
                    Value::String(truncate_non_secret(wallet_id, trunc)),
                );
            }
            if let Some(session) = obj.get("session").and_then(|v| v.as_object()) {
                if let Some(wallet_id) = session.get("wallet_id").and_then(|v| v.as_str()) {
                    out.insert(
                        "wallet_id".to_string(),
                        Value::String(truncate_non_secret(wallet_id, trunc)),
                    );
                }
                if session.get("token").is_some() {
                    out.insert(
                        "session_token".to_string(),
                        Value::String("<redacted>".to_string()),
                    );
                }
            }
            if obj.get("password").is_some() {
                out.insert(
                    "password".to_string(),
                    Value::String("<redacted>".to_string()),
                );
            }
            if obj.get("confirmation").is_some() {
                out.insert(
                    "confirmation".to_string(),
                    Value::String("<redacted>".to_string()),
                );
            }
            Some(Value::Object(out))
        }
        "app.wallet.delete_wallet" => {
            let mut out = Map::new();
            if let Some(id) = obj.get("wallet_id").and_then(|v| v.as_str()) {
                out.insert(
                    "wallet_id".to_string(),
                    Value::String(truncate_non_secret(id, trunc)),
                );
            }
            out.insert(
                "password".to_string(),
                Value::String("<redacted>".to_string()),
            );
            Some(Value::Object(out))
        }
        "wallet.tx.send_transaction" => {
            let mut out = Map::new();
            if let Some(session) = obj.get("session").and_then(|v| v.as_object()) {
                if let Some(wallet_id) = session.get("wallet_id").and_then(|v| v.as_str()) {
                    out.insert(
                        "wallet_id".to_string(),
                        Value::String(truncate_non_secret(wallet_id, trunc)),
                    );
                }
                if session.get("token").is_some() {
                    out.insert(
                        "session_token".to_string(),
                        Value::String("<redacted>".to_string()),
                    );
                }
            } else if let Some(wallet_id) = obj.get("wallet_id").and_then(|v| v.as_str()) {
                out.insert(
                    "wallet_id".to_string(),
                    Value::String(truncate_non_secret(wallet_id, trunc)),
                );
            }
            if let Some(amount) = obj.get("amount") {
                out.insert("amount".to_string(), amount.clone());
            }
            if obj.get("memo").is_some() {
                out.insert("memo".to_string(), Value::String("<redacted>".to_string()));
            }
            if obj.get("tx_data").is_some() {
                out.insert(
                    "tx_data".to_string(),
                    Value::String("<redacted>".to_string()),
                );
            }
            if let Some(key) = obj.get("idempotency_key").and_then(|v| v.as_str()) {
                out.insert(
                    "idempotency_key".to_string(),
                    Value::String(fingerprint(key)),
                );
            }
            if let Some(asset_id) = obj.get("asset_id") {
                out.insert("asset_id".to_string(), summarize_asset_id(asset_id, trunc));
            }
            Some(Value::Object(out))
        }
        "wallet.tx.broadcast_transaction" | "wallet.tx.verify_transaction_package" => {
            let mut out = Map::new();
            if let Some(session) = obj.get("session").and_then(|v| v.as_object()) {
                if let Some(wallet_id) = session.get("wallet_id").and_then(|v| v.as_str()) {
                    out.insert(
                        "wallet_id".to_string(),
                        Value::String(truncate_non_secret(wallet_id, trunc)),
                    );
                }
                if session.get("token").is_some() {
                    out.insert(
                        "session_token".to_string(),
                        Value::String("<redacted>".to_string()),
                    );
                }
            }
            if let Some(tx_data) = obj.get("tx_data").and_then(|v| v.as_str()) {
                out.insert(
                    "tx_data_summary".to_string(),
                    summarize_tx_package_blob(tx_data, trunc),
                );
            }
            Some(Value::Object(out))
        }
        "wallet.tx.import_transaction" => {
            let mut out = Map::new();
            if let Some(session) = obj.get("session").and_then(|v| v.as_object()) {
                if let Some(wallet_id) = session.get("wallet_id").and_then(|v| v.as_str()) {
                    out.insert(
                        "wallet_id".to_string(),
                        Value::String(truncate_non_secret(wallet_id, trunc)),
                    );
                }
                if session.get("token").is_some() {
                    out.insert(
                        "session_token".to_string(),
                        Value::String("<redacted>".to_string()),
                    );
                }
            }
            if let Some(tx_data) = obj.get("tx_data").and_then(|v| v.as_str()) {
                out.insert(
                    "tx_data_summary".to_string(),
                    summarize_portable_tx_blob(tx_data, trunc),
                );
            }
            Some(Value::Object(out))
        }
        "wallet.tx.export_transaction" => {
            let mut out = Map::new();
            if let Some(session) = obj.get("session").and_then(|v| v.as_object()) {
                if let Some(wallet_id) = session.get("wallet_id").and_then(|v| v.as_str()) {
                    out.insert(
                        "wallet_id".to_string(),
                        Value::String(truncate_non_secret(wallet_id, trunc)),
                    );
                }
                if session.get("token").is_some() {
                    out.insert(
                        "session_token".to_string(),
                        Value::String("<redacted>".to_string()),
                    );
                }
            }
            if let Some(tx_id) = obj.get("tx_id").and_then(|v| v.as_str()) {
                out.insert(
                    "tx_id".to_string(),
                    Value::String(truncate_non_secret(tx_id, trunc)),
                );
            }
            Some(Value::Object(out))
        }
        "app.wallet.list_wallets" => Some(Value::Object(Map::new())),
        "wallet.asset.list_assets" => {
            let mut out = Map::new();
            if let Some(wallet_id) = obj.get("wallet_id").and_then(|v| v.as_str()) {
                out.insert(
                    "wallet_id".to_string(),
                    Value::String(truncate_non_secret(wallet_id, trunc)),
                );
            }
            out.insert(
                "pagination".to_string(),
                Value::String("<present>".to_string()),
            );
            Some(Value::Object(out))
        }
        "wallet.tx.get_transaction_history" => {
            let mut out = Map::new();
            if let Some(session) = obj.get("session").and_then(|v| v.as_object()) {
                if let Some(wallet_id) = session.get("wallet_id").and_then(|v| v.as_str()) {
                    out.insert(
                        "wallet_id".to_string(),
                        Value::String(truncate_non_secret(wallet_id, trunc)),
                    );
                }
                if session.get("token").is_some() {
                    out.insert(
                        "session_token".to_string(),
                        Value::String("<redacted>".to_string()),
                    );
                }
            } else if let Some(wallet_id) = obj.get("wallet_id").and_then(|v| v.as_str()) {
                out.insert(
                    "wallet_id".to_string(),
                    Value::String(truncate_non_secret(wallet_id, trunc)),
                );
            }
            out.insert(
                "pagination".to_string(),
                Value::String("<present>".to_string()),
            );
            Some(Value::Object(out))
        }
        _ => None,
    }
}

pub fn summarize_response(
    method: &str,
    result: &Value,
    trunc: &RpcLoggingTruncationConfig,
) -> Option<Value> {
    match method {
        "app.wallet.create_wallet" => {
            let obj = result.as_object()?;
            let mut out = Map::new();
            if let Some(wallet_id) = obj.get("wallet_id").and_then(|v| v.as_str()) {
                out.insert(
                    "wallet_id".to_string(),
                    Value::String(truncate_non_secret(wallet_id, trunc)),
                );
            }
            if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
                out.insert(
                    "name".to_string(),
                    Value::String(truncate_non_secret(name, trunc)),
                );
            }
            if obj.get("seed_phrase").is_some() {
                out.insert(
                    "seed_phrase".to_string(),
                    Value::String("<redacted>".to_string()),
                );
            }
            Some(Value::Object(out))
        }
        "wallet.session.show_seed_phrase" => {
            let obj = result.as_object()?;
            let mut out = Map::new();

            if obj.get("seed_phrase").is_some() {
                out.insert(
                    "seed_phrase".to_string(),
                    Value::String("<redacted>".to_string()),
                );
            }

            if obj.get("encrypted_payload").is_some() {
                out.insert(
                    "encrypted_payload".to_string(),
                    Value::String("<present>".to_string()),
                );
            }

            Some(Value::Object(out))
        }
        "wallet.session.unlock_wallet" => {
            let obj = result.as_object()?;
            let mut out = Map::new();
            if let Some(wallet_id) = obj.get("wallet_id").and_then(|v| v.as_str()) {
                out.insert(
                    "wallet_id".to_string(),
                    Value::String(truncate_non_secret(wallet_id, trunc)),
                );
            }
            if let Some(token) = obj.get("token").and_then(|v| v.as_str()) {
                out.insert(
                    "session_token".to_string(),
                    Value::String(fingerprint(token)),
                );
            }
            Some(Value::Object(out))
        }
        "app.wallet.list_wallets" => {
            if let Some(arr) = result.as_array() {
                let mut out = Map::new();
                out.insert(
                    "count".to_string(),
                    Value::Number((arr.len() as u64).into()),
                );
                return Some(Value::Object(out));
            }
            None
        }
        "wallet.asset.list_assets" => {
            let obj = result.as_object()?;
            let mut out = Map::new();
            if let Some(assets) = obj.get("assets").and_then(|v| v.as_array()) {
                out.insert(
                    "assets_count".to_string(),
                    Value::Number((assets.len() as u64).into()),
                );
            }
            Some(Value::Object(out))
        }
        "wallet.tx.get_transaction_history" => {
            let obj = result.as_object()?;
            let mut out = Map::new();
            if let Some(items) = obj.get("items").and_then(|v| v.as_array()) {
                out.insert(
                    "items_count".to_string(),
                    Value::Number((items.len() as u64).into()),
                );
            }
            Some(Value::Object(out))
        }
        "wallet.tx.verify_transaction_package" => {
            let obj = result.as_object()?;
            let mut out = Map::new();
            if let Some(tx_digest_hex) = obj.get("tx_digest_hex").and_then(|v| v.as_str()) {
                out.insert(
                    "tx_digest_hex".to_string(),
                    Value::String(truncate_non_secret(tx_digest_hex, trunc)),
                );
            }
            for key in [
                "package_status",
                "is_valid",
                "lifecycle",
                "import_ready",
                "all_owned_spendable",
            ] {
                if let Some(value) = obj.get(key) {
                    out.insert(key.to_string(), value.clone());
                }
            }
            if let Some(owned_outputs) = obj.get("owned_outputs").and_then(|v| v.as_array()) {
                out.insert(
                    "owned_outputs_count".to_string(),
                    Value::Number((owned_outputs.len() as u64).into()),
                );
            }
            if let Some(errors) = obj.get("errors").and_then(|v| v.as_array()) {
                out.insert(
                    "errors_count".to_string(),
                    Value::Number((errors.len() as u64).into()),
                );
            }
            if let Some(error_codes) = obj.get("error_codes").and_then(|v| v.as_array()) {
                out.insert(
                    "error_codes_count".to_string(),
                    Value::Number((error_codes.len() as u64).into()),
                );
            }
            Some(Value::Object(out))
        }
        "wallet.tx.import_transaction" => {
            let obj = result.as_object()?;
            let mut out = Map::new();
            if let Some(tx_id) = obj.get("tx_id").and_then(|v| v.as_str()) {
                out.insert(
                    "tx_id".to_string(),
                    Value::String(truncate_non_secret(tx_id, trunc)),
                );
            }
            for key in ["status", "lifecycle"] {
                if let Some(value) = obj.get(key) {
                    out.insert(key.to_string(), value.clone());
                }
            }
            if let Some(outputs) = obj.get("imported_outputs").and_then(|v| v.as_array()) {
                out.insert(
                    "imported_outputs_count".to_string(),
                    Value::Number((outputs.len() as u64).into()),
                );
            }
            if let Some(error_codes) = obj.get("error_codes").and_then(|v| v.as_array()) {
                out.insert(
                    "error_codes_count".to_string(),
                    Value::Number((error_codes.len() as u64).into()),
                );
            }
            Some(Value::Object(out))
        }
        "wallet.tx.export_transaction" => {
            let obj = result.as_object()?;
            let mut out = Map::new();
            if let Some(success) = obj.get("success") {
                out.insert("success".to_string(), success.clone());
            }
            if obj.get("export_path").is_some() {
                out.insert(
                    "export_path".to_string(),
                    Value::String("<present>".to_string()),
                );
            }
            Some(Value::Object(out))
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "test_logging_summary.rs"]
mod tests;
