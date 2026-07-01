use std::{collections::BTreeSet, path::Path};

use hex::encode;
use z00z_utils::{
    codec::{Codec, JsonCodec, Value},
    io::{load_json, read_to_string},
};
use z00z_wallets::tx::{TxOutRole, TxPackage};

fn parse_log_row(line: &str) -> Value {
    JsonCodec
        .deserialize(line.as_bytes())
        .expect("valid stage 8 log row")
}

fn obj_keys(value: &Value) -> BTreeSet<String> {
    value
        .as_object()
        .expect("json object")
        .keys()
        .cloned()
        .collect()
}

pub fn read_events(out: &Path) -> BTreeSet<String> {
    read_to_string(out.join("logs").join("logger.json"))
        .expect("stage 8 logger")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            parse_log_row(line)["event"]
                .as_str()
                .expect("event string")
                .to_string()
        })
        .collect()
}

pub fn log_detail(out: &Path, event: &str) -> String {
    read_to_string(out.join("logs").join("logger.json"))
        .expect("stage 8 logger")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(parse_log_row)
        .find(|row| row["event"].as_str() == Some(event))
        .and_then(|row| row["detail"].as_str().map(str::to_string))
        .unwrap_or_else(|| panic!("missing log detail for event {event}"))
}

pub fn parse_count(detail: &str, key: &str) -> u64 {
    detail
        .split_whitespace()
        .find_map(|part| part.strip_prefix(&format!("{key}=")))
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or_else(|| panic!("missing count {key} in detail: {detail}"))
}

pub fn assert_snap(snap: &Value) {
    assert_eq!(
        obj_keys(snap),
        BTreeSet::from([
            "canonical_status".to_string(),
            "claimed_count_after_route".to_string(),
            "input_tx_digest_hex".to_string(),
            "recipient_output_index".to_string(),
            "rpc_status".to_string(),
            "runtime_status".to_string(),
            "stage".to_string(),
            "status".to_string(),
            "transfer_count".to_string(),
        ])
    );
    assert_eq!(snap["stage"].as_u64(), Some(8));
    assert_eq!(snap["transfer_count"].as_u64(), Some(1));
    assert_eq!(snap["recipient_output_index"].as_u64(), Some(0));
    assert_eq!(snap["input_tx_digest_hex"].as_str().map(str::len), Some(64));
    assert_eq!(snap["rpc_status"].as_str(), Some("RECEIVE_DETECTED"));
    assert_eq!(snap["canonical_status"].as_str(), Some("RECEIVE_DETECTED"));
    assert_eq!(snap["runtime_status"].as_str(), Some("RECEIVE_DETECTED"));
    assert!(snap["claimed_count_after_route"].as_u64().unwrap_or(0) > 0);
}

pub fn assert_tx(tx: &Value) {
    assert_eq!(
        obj_keys(tx),
        BTreeSet::from([
            "amount".to_string(),
            "asset_id_hex".to_string(),
            "c_amount".to_string(),
            "ciphertext_len".to_string(),
            "owner_tag".to_string(),
            "r_pub".to_string(),
            "recipient_output_index".to_string(),
            "serial_id".to_string(),
            "source_tx_digest_hex".to_string(),
            "stage".to_string(),
            "status".to_string(),
            "tag16".to_string(),
        ])
    );
    assert_eq!(tx["stage"].as_u64(), Some(8));
    assert_eq!(tx["recipient_output_index"].as_u64(), Some(0));
    assert_eq!(tx["source_tx_digest_hex"].as_str().map(str::len), Some(64));
    assert_eq!(tx["status"].as_str(), Some("ok"));
    assert_eq!(tx["asset_id_hex"].as_str().map(str::len), Some(64));
    assert_eq!(tx["r_pub"].as_str().map(str::len), Some(64));
    assert_eq!(tx["owner_tag"].as_str().map(str::len), Some(64));
    assert_eq!(tx["c_amount"].as_str().map(str::len), Some(64));
    assert!(tx["ciphertext_len"].as_u64().unwrap_or(0) > 0);
}

pub fn assert_selected_from_stage4(out: &Path, tx: &Value) {
    let pkg: TxPackage = load_json(out.join("transactions").join("tx_alice_to_bob_pkg.json"))
        .expect("stage 6 tx package");
    let recv_rows: Vec<_> = pkg
        .tx
        .outputs
        .iter()
        .filter(|row| row.role == TxOutRole::Recipient)
        .collect();
    let out_idx = tx["recipient_output_index"]
        .as_u64()
        .expect("recipient output index") as usize;
    let selected = recv_rows.get(out_idx).expect("selected recipient output");
    let asset = selected
        .asset_wire
        .clone()
        .to_asset()
        .expect("selected asset");

    assert_eq!(
        pkg.tx_digest_hex,
        tx["source_tx_digest_hex"]
            .as_str()
            .expect("source tx digest")
    );
    assert_eq!(
        encode(asset.asset_id()),
        tx["asset_id_hex"].as_str().expect("asset id hex")
    );
    assert_eq!(
        u64::from(asset.serial_id),
        tx["serial_id"].as_u64().expect("serial id")
    );
    assert_eq!(asset.amount, tx["amount"].as_u64().expect("amount"));
}

pub fn assert_safe(snap: &Value, tx: &Value) {
    for field in [
        "k_dh",
        "s_out",
        "blinding",
        "receiver_secret",
        "password",
        "seed_phrase",
    ] {
        assert!(
            tx.get(field).is_none(),
            "unexpected secret field in stage8 tx artifact: {field}"
        );
        assert!(
            snap.get(field).is_none(),
            "unexpected secret field in stage8 snapshot: {field}"
        );
    }
}
