use std::path::{Path, PathBuf};

use z00z_core::assets::Asset;
use z00z_utils::io::{load_bincode, load_json};

use z00z_simulator::scenario_1::stage_13::shared_cases;

type RowKey = (String, String, String, u64, String, u64);
type LifeMap = std::collections::BTreeMap<RowKey, String>;

fn cross_out() -> PathBuf {
    shared_cases::full_stage13_out()
}

fn load_snap(out: &Path, name: &str) -> serde_json::Value {
    load_json(out.join(name)).expect("snapshot json")
}

fn load_tx_pkg(out: &Path) -> serde_json::Value {
    load_json(out.join("transactions").join("tx_alice_to_bob_pkg.json")).expect("tx package json")
}

fn load_rows(out: &Path, name: &str) -> serde_json::Value {
    load_json(out.join("transactions").join(name)).unwrap_or_else(|_| panic!("load rows: {name}"))
}

fn req_str<'a>(row: &'a serde_json::Value, key: &str) -> &'a str {
    row[key]
        .as_str()
        .unwrap_or_else(|| panic!("{key} missing or not string"))
}

fn req_u64(row: &serde_json::Value, key: &str) -> u64 {
    row[key]
        .as_u64()
        .unwrap_or_else(|| panic!("{key} missing or not u64"))
}

fn check_row_shape(row: &serde_json::Value, digest: &str) {
    assert!(!req_str(row, "actor").is_empty(), "actor must not be empty");
    assert!(
        !req_str(row, "wallet_id").is_empty(),
        "wallet_id must not be empty"
    );
    assert!(
        !req_str(row, "asset_id_hex").is_empty(),
        "asset_id_hex must not be empty"
    );
    let _ = req_u64(row, "serial_id");
    assert!(!req_str(row, "class").is_empty(), "class must not be empty");
    assert!(req_u64(row, "amount") > 0, "amount must be positive");
    assert!(
        !req_str(row, "lifecycle_status").is_empty(),
        "lifecycle_status must not be empty"
    );
    assert_eq!(
        req_str(row, "tx_digest_hex"),
        digest,
        "row tx_digest_hex must match tx package digest"
    );
}

fn row_key(row: &serde_json::Value) -> RowKey {
    (
        req_str(row, "actor").to_string(),
        req_str(row, "wallet_id").to_string(),
        req_str(row, "asset_id_hex").to_string(),
        req_u64(row, "serial_id"),
        req_str(row, "class").to_string(),
        req_u64(row, "amount"),
    )
}

fn check_snap(s4: &serde_json::Value, digest: &str, out_len: usize) {
    assert_eq!(
        s4["stage"].as_u64().expect("s4.stage missing"),
        6,
        "stage_4_snapshot.json stage mismatch"
    );
    assert_eq!(
        s4["status"].as_str().expect("s4.status missing"),
        "ok",
        "stage_4_snapshot.json status mismatch"
    );
    assert_eq!(
        s4["tx_digest_hex"]
            .as_str()
            .expect("s4.tx_digest_hex missing"),
        digest,
        "snapshot tx_digest_hex mismatch"
    );
    assert_eq!(
        s4["tx_count"].as_u64().expect("s4.tx_count missing"),
        1,
        "snapshot tx_count mismatch"
    );
    assert_eq!(
        s4["output_count"]
            .as_u64()
            .expect("s4.output_count missing"),
        out_len as u64,
        "snapshot output_count mismatch"
    );
}

fn pkg_info(tx_pkg: &serde_json::Value) -> (String, usize) {
    assert_eq!(
        tx_pkg["kind"].as_str().expect("tx.kind missing"),
        "TxPackage",
        "tx package kind mismatch"
    );
    assert_eq!(
        tx_pkg["version"].as_u64().expect("tx.version missing"),
        1,
        "tx package version mismatch"
    );
    assert_eq!(
        tx_pkg["status"].as_str().expect("tx.status missing"),
        "prepared",
        "tx package status mismatch"
    );

    let digest = tx_pkg["tx_digest_hex"]
        .as_str()
        .expect("tx.tx_digest_hex missing");
    assert!(!digest.is_empty(), "tx_digest_hex is empty");

    let outputs = tx_pkg["tx"]["outputs"]
        .as_array()
        .expect("tx.outputs missing or not array");
    assert!(!outputs.is_empty(), "tx.outputs is empty");

    let fee = tx_pkg["tx"]["fee"].as_u64().expect("tx.fee missing");
    assert!(fee > 0, "tx.fee must be positive");

    (digest.to_string(), outputs.len())
}

fn bob_rows(rows: &[serde_json::Value], life: &str) -> std::collections::BTreeSet<RowKey> {
    rows.iter()
        .filter(|row| {
            req_str(row, "actor").eq_ignore_ascii_case("bob")
                && req_str(row, "lifecycle_status") == life
        })
        .map(row_key)
        .collect::<std::collections::BTreeSet<_>>()
}

fn life_map(rows: &[serde_json::Value]) -> LifeMap {
    rows.iter()
        .map(|row| (row_key(row), req_str(row, "lifecycle_status").to_string()))
        .collect::<LifeMap>()
}

fn next_life(pending: &str) -> String {
    pending.replacen("pending_", "confirmed_", 1)
}

fn life_pair_ok(pending: &LifeMap, confirm: &LifeMap) {
    assert_eq!(
        pending.len(),
        confirm.len(),
        "pending/confirm row count must match"
    );
    for (key, pending_life) in pending {
        let confirm_life = confirm
            .get(key)
            .unwrap_or_else(|| panic!("confirmed row missing for continuity key"));
        assert_eq!(
            confirm_life,
            &next_life(pending_life),
            "lifecycle transition mismatch for continuity key"
        );
    }
}

fn check_rows(pending: &serde_json::Value, confirm: &serde_json::Value, digest: &str) {
    let pending_rows = pending
        .as_array()
        .expect("wallets_pending.json must be array");
    let confirm_rows = confirm
        .as_array()
        .expect("wallets_confirmed.json must be array");
    assert!(!pending_rows.is_empty(), "pending rows must not be empty");
    assert!(!confirm_rows.is_empty(), "confirm rows must not be empty");

    for row in pending_rows {
        check_row_shape(row, digest);
    }
    for row in confirm_rows {
        check_row_shape(row, digest);
    }

    let pending_map = life_map(pending_rows);
    let confirm_map = life_map(confirm_rows);
    life_pair_ok(&pending_map, &confirm_map);

    let bob_pending = bob_rows(pending_rows, "pending_receive");
    let bob_confirm = bob_rows(confirm_rows, "confirmed_receive");
    assert!(
        !bob_pending.is_empty(),
        "bob pending_receive rows must not be empty"
    );
    assert_eq!(
        bob_pending, bob_confirm,
        "bob pending/confirm rows must keep the same schema and keys"
    );
}

fn sum_genesis(out: &Path) -> u64 {
    let mut sum = 0u64;
    for path in list_gen_bins(out) {
        let part: Vec<Asset> = load_bincode(&path).expect("genesis bin");
        for item in part {
            sum = sum.checked_add(item.amount).expect("genesis sum overflow");
        }
    }
    sum
}

fn list_gen_bins(out: &Path) -> Vec<PathBuf> {
    let base = out.join("genesis");
    let mut files = Vec::new();
    let entries = std::fs::read_dir(&base).expect("read genesis dir");

    for entry in entries {
        let path = entry.expect("read dir entry").path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|item| item.to_str()) else {
            continue;
        };
        if name.starts_with("genesis_") && name.ends_with(".bin") {
            files.push(path);
        }
    }

    files.sort();
    assert!(!files.is_empty(), "no genesis_*.bin files found");
    files
}

#[test]
fn test_cross_stage_supply_conservation() {
    let out = cross_out();
    let _s1 = load_snap(&out, "stage_1_snapshot.json");
    let s3 = load_snap(&out, "stage_3_snapshot.json");

    let s1_total = sum_genesis(&out);
    let s3_total: u64 = s3["actor_claims"]
        .as_array()
        .expect("s3.actor_claims missing")
        .iter()
        .map(|row| {
            row["total_amount"]
                .as_u64()
                .expect("s3 total_amount missing")
        })
        .sum();

    assert_eq!(
        s1_total, s3_total,
        "supply conservation violated: s1={s1_total} s3={s3_total}"
    );
}

#[test]
fn test_cross_stage_count_pipeline() {
    let out = cross_out();
    let s1 = load_snap(&out, "stage_1_snapshot.json");
    let s3 = load_snap(&out, "stage_3_snapshot.json");

    let s1_count = s1["assets_count"]
        .as_u64()
        .expect("s1.assets_count missing");
    let s3_count = s3["input_assets_count"]
        .as_u64()
        .expect("s3.input_assets_count missing");

    assert_eq!(
        s1_count, s3_count,
        "asset count: stage_1={s1_count} stage_3_input={s3_count}"
    );
}

const EXPECTED_ACTORS: &[&str] = &["alice", "bob", "charlie"];

#[test]
fn test_cross_stage_actors_claimed() {
    let out = cross_out();
    let snap = load_snap(&out, "stage_3_snapshot.json");
    let claims = snap["actor_claims"]
        .as_array()
        .expect("actor_claims field missing or not array");

    assert_eq!(
        claims.len(),
        EXPECTED_ACTORS.len(),
        "actor_claims count mismatch: got {}",
        claims.len()
    );

    for expected in EXPECTED_ACTORS {
        let entry = claims.iter().find(|row| {
            row["name"]
                .as_str()
                .map(|name| name.to_lowercase() == *expected)
                .unwrap_or(false)
        });
        let entry = entry.unwrap_or_else(|| panic!("{expected} missing from actor_claims"));
        let count = entry["assets_count"]
            .as_u64()
            .expect("assets_count field missing");
        assert!(count > 0, "{expected} has zero assets in stage_3 claims");
    }
}

#[test]
fn test_s4_tx_pkg_ok() {
    let out = cross_out();
    let s4 = load_snap(&out, "stage_4_snapshot.json");
    let tx_pkg = load_tx_pkg(&out);
    let pending = load_rows(&out, "wallets_pending.json");
    let confirm = load_rows(&out, "wallets_confirmed.json");
    let (digest, out_len) = pkg_info(&tx_pkg);

    check_snap(&s4, &digest, out_len);
    check_rows(&pending, &confirm, &digest);
}

#[test]
fn test_s4_bob_pending_ok() {
    let out = cross_out();

    let selected = load_rows(&out, "wallets_selected_inputs.json");
    let pending = load_rows(&out, "wallets_pending.json");

    let selected_rows = selected
        .as_array()
        .expect("wallets_selected_inputs.json must be array");
    let pending_rows = pending
        .as_array()
        .expect("wallets_pending.json must be array");

    let mut selected_serials = std::collections::BTreeSet::<u32>::new();
    for row in selected_rows {
        let serial = row["serial_id"]
            .as_u64()
            .expect("selected serial_id missing") as u32;
        selected_serials.insert(serial);
    }
    assert!(
        !selected_serials.is_empty(),
        "selected inputs serial set must not be empty"
    );

    let mut bob_pending_serials = std::collections::BTreeSet::<u32>::new();
    for row in pending_rows {
        let actor = row["actor"].as_str().unwrap_or_default();
        let lifecycle = row["lifecycle_status"].as_str().unwrap_or_default();
        if actor.eq_ignore_ascii_case("bob") && lifecycle == "pending_receive" {
            let serial = row["serial_id"]
                .as_u64()
                .expect("pending serial_id missing") as u32;
            bob_pending_serials.insert(serial);
        }
    }

    for serial in &selected_serials {
        assert!(
            bob_pending_serials.contains(serial),
            "selected input serial_id {} is missing from Bob pending_receive rows",
            serial
        );
    }
}
