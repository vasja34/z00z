use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use z00z_storage::snapshot::{PrepFsStore, PrepSnapshot, PrepSnapshotId, PrepSnapshotStore};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::read_file,
};
use z00z_wallets::tx::{verify_full_tx_package, TxOutRole, TxOutputWire, TxPackage};

use z00z_simulator::scenario_1::support::checkpoint_shared_cases;

struct CpRunCase {
    out: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
struct Stage9BridgeFile {
    bridge_outputs: Vec<TxOutputWire>,
}

fn cp_file(out: &Path) -> PathBuf {
    out.join("transactions/checkpoint_s7.json")
}

fn bridge_file(out: &Path) -> PathBuf {
    out.join("transactions/checkpoint_bridge_s6.json")
}

fn frag_file(out: &Path, idx: u32) -> PathBuf {
    out.join(format!("transactions/leaf_alice_to_charlie_frag{idx}.json"))
}

fn prep_file(out: &Path) -> PathBuf {
    out.join("transactions/checkpoint_prep.json")
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PrepRefFile {
    snapshot_id_hex: String,
}

fn tx_file(out: &Path) -> PathBuf {
    out.join("transactions/tx_alice_to_bob_pkg.json")
}

fn load_json(path: &Path) -> Value {
    serde_json::from_slice(&read_file(path).expect("read json")).expect("decode json")
}

fn load_tx(path: &Path) -> TxPackage {
    JsonCodec
        .deserialize(&read_file(path).expect("read tx json"))
        .expect("decode tx package")
}

fn load_bridge(out: &Path) -> Stage9BridgeFile {
    serde_json::from_slice(&read_file(bridge_file(out)).expect("read bridge"))
        .expect("decode bridge")
}

fn load_prep(out: &Path) -> PrepSnapshot {
    let prep_ref: PrepRefFile =
        serde_json::from_slice(&read_file(prep_file(out)).expect("read prep ref"))
            .expect("decode prep ref");
    let raw = hex::decode(prep_ref.snapshot_id_hex).expect("snapshot id hex");
    let snap_id = PrepSnapshotId::new(raw.try_into().expect("snapshot id size"));
    let store = PrepFsStore::new(out.join("transactions"));
    store.load_snapshot(&snap_id).expect("prep snapshot")
}

fn load_prep_ref(out: &Path) -> PrepRefFile {
    serde_json::from_slice(&read_file(prep_file(out)).expect("read prep ref"))
        .expect("decode prep ref")
}

fn cp_run_case() -> &'static CpRunCase {
    static CASE: OnceLock<CpRunCase> = OnceLock::new();
    CASE.get_or_init(|| CpRunCase {
        out: checkpoint_shared_cases::default_stage11_out(),
    })
}

fn made_rows(cp: &Value) -> Vec<Value> {
    cp["created_delta"]
        .as_array()
        .expect("created delta")
        .clone()
}

fn made_ids(cp: &Value) -> Vec<String> {
    made_rows(cp)
        .into_iter()
        .map(|row| {
            row["asset_id_hex"]
                .as_str()
                .map(|item| item.to_string())
                .expect("created asset id")
        })
        .collect()
}

fn out_asset_id(row: &TxOutputWire) -> String {
    let wire = row.asset_wire.clone().to_wire().expect("asset wire");
    hex::encode(wire.to_asset().expect("output asset").asset_id())
}

fn role_ids(tx: &TxPackage, role: &str) -> Vec<String> {
    let want = match role {
        "recipient" => TxOutRole::Recipient,
        "change" => TxOutRole::Change,
        "fee" => TxOutRole::Fee,
        other => panic!("unexpected role {other}"),
    };
    tx.tx
        .outputs
        .iter()
        .filter(|row| row.role == want)
        .map(out_asset_id)
        .collect()
}

fn bridge_ids(bridge: &Stage9BridgeFile) -> Vec<String> {
    bridge.bridge_outputs.iter().map(out_asset_id).collect()
}

fn bridge_role_ids(bridge: &Stage9BridgeFile, role: TxOutRole) -> Vec<String> {
    bridge
        .bridge_outputs
        .iter()
        .filter(|row| row.role == role)
        .map(out_asset_id)
        .collect()
}

fn fee_out_id(tx: &TxPackage) -> String {
    tx.tx
        .outputs
        .iter()
        .find(|row| row.role == TxOutRole::Fee)
        .map(out_asset_id)
        .expect("fee output id")
}

fn load_cp_run() -> (Value, PrepSnapshot, Value, Value) {
    let out = &cp_run_case().out;

    (
        load_json(&cp_file(out)),
        load_prep(out),
        load_json(&frag_file(out, 1)),
        load_json(&frag_file(out, 2)),
    )
}

fn assert_frag_row(frag: &Value, prev_root: &str, spent: &[Value], made: &[Value]) {
    let inputs = frag["inputs"].as_array().expect("frag inputs");
    let outputs = frag["outputs"].as_array().expect("frag outputs");

    assert_eq!(inputs.len(), 1, "each fragment must carry one typed input");
    assert_eq!(
        outputs.len(),
        1,
        "each fragment must carry one typed output"
    );
    assert_eq!(
        inputs[0]["prev_root_hex"]
            .as_str()
            .expect("input prev root"),
        prev_root,
        "fragment input must keep prep-derived prev_root_hex"
    );

    let spent_id = inputs[0]["asset_id_hex"].as_str().expect("spent id");
    let made_id = outputs[0]["asset_id_hex"].as_str().expect("made id");
    let made_hash = outputs[0]["leaf_hash_hex"].as_str().expect("made hash");

    assert!(
        spent.iter().any(|row| row.as_str() == Some(spent_id)),
        "fragment input asset id must appear in spent_delta"
    );
    assert!(
        made.iter().any(|row| {
            row["asset_id_hex"].as_str() == Some(made_id)
                && row["leaf_hash_hex"].as_str() == Some(made_hash)
        }),
        "fragment output typed row must appear in created_delta"
    );
}

#[test]
fn test_fee_out_in_cp() {
    let out = &cp_run_case().out;
    let cp = load_json(&cp_file(out));
    let tx = load_tx(&tx_file(out));
    let bridge = load_bridge(out);
    let fee_id = fee_out_id(&tx);
    let made = made_ids(&cp);
    let bridge_ids = bridge_ids(&bridge);

    assert!(
        !made.iter().any(|row| row == fee_id.as_str()),
        "checkpoint created delta must exclude the original stage4 fee output asset id"
    );
    assert_eq!(
        made.len(),
        bridge_ids.len(),
        "checkpoint created delta must carry every persisted bridge output as typed asset_id/leaf_hash entries"
    );
    assert!(
        bridge_ids.iter().all(|id| made.contains(id)),
        "checkpoint created delta must match persisted bridge output ids"
    );
}

#[test]
fn test_spent_id_only() {
    let out = &cp_run_case().out;
    let cp = load_json(&cp_file(out));
    let spent = cp["spent_delta"].as_array().expect("spent delta");
    let made = made_rows(&cp);

    assert!(!spent.is_empty(), "spent delta must not be empty");
    assert!(
        spent.iter().all(|row| {
            row.as_str()
                .is_some_and(|item| item.len() == 64 && !item.contains(':'))
        }),
        "spent delta must contain canonical asset_id hex only"
    );
    assert!(
        made.iter().all(|row| {
            row["asset_id_hex"]
                .as_str()
                .is_some_and(|item| item.len() == 64 && !item.contains(':'))
                && row["leaf_hash_hex"]
                    .as_str()
                    .is_some_and(|item| item.len() == 64 && !item.contains(':'))
        }),
        "created delta must keep typed asset_id/leaf_hash pairs only"
    );
}

#[test]
fn test_made_pairs_typed() {
    let out = &cp_run_case().out;
    let cp = load_json(&cp_file(out));
    let made = made_rows(&cp);

    assert!(!made.is_empty(), "created delta must not be empty");
    assert!(
        made.iter()
            .all(|row| row.get("asset_id_hex").is_some() && row.get("leaf_hash_hex").is_some()),
        "created delta rows must expose typed fields"
    );
}

#[test]
fn test_bridge_roles_in_cp() {
    let out = &cp_run_case().out;
    let cp = load_json(&cp_file(out));
    let tx = load_tx(&tx_file(out));
    let bridge = load_bridge(out);
    let made = made_ids(&cp);
    let recipient_ids = role_ids(&tx, "recipient");
    let change_ids = role_ids(&tx, "change");
    let fee_ids = role_ids(&tx, "fee");
    let bridge_recipient_ids = bridge_role_ids(&bridge, TxOutRole::Recipient);
    let bridge_change_ids = bridge_role_ids(&bridge, TxOutRole::Change);
    let bridge_fee_ids = bridge_role_ids(&bridge, TxOutRole::Fee);

    assert!(
        !recipient_ids.is_empty(),
        "tx must contain recipient outputs"
    );
    assert!(!change_ids.is_empty(), "tx must contain change outputs");
    assert!(!fee_ids.is_empty(), "tx must contain fee outputs");
    assert!(
        !bridge_recipient_ids.is_empty(),
        "persisted bridge outputs must contain recipient rows"
    );
    assert!(
        bridge_change_ids.is_empty(),
        "persisted bridge outputs must not carry stage4 change roles"
    );
    assert!(
        bridge_fee_ids.is_empty(),
        "persisted bridge outputs must not carry stage4 fee roles"
    );
    assert!(
        bridge_recipient_ids.iter().all(|id| made.contains(id)),
        "checkpoint created delta must preserve persisted recipient bridge outputs"
    );
    assert!(
        change_ids.iter().all(|id| !made.contains(id)),
        "checkpoint created delta must exclude original stage4 change outputs"
    );
    assert!(
        fee_ids.iter().all(|id| !made.contains(id)),
        "checkpoint created delta must exclude original stage4 fee outputs"
    );
}

#[test]
fn test_cp_prev_root() {
    let out = &cp_run_case().out;
    let cp = load_json(&cp_file(out));
    let prep = load_prep(out);

    assert_eq!(
        cp["prev_root_hex"].as_str().expect("checkpoint prev root"),
        hex::encode(prep.prev_root.as_bytes()),
        "stage6 must consume the exact prev_root emitted by stage4 prep"
    );
}

#[test]
fn test_cp_root_delta() {
    let out = &cp_run_case().out;
    let cp = load_json(&cp_file(out));
    let prev_root = cp["prev_root_hex"].as_str().expect("prev root");
    let new_root = cp["new_root_hex"].as_str().expect("new root");
    let spent = cp["spent_delta"].as_array().expect("spent delta");
    let made = cp["created_delta"].as_array().expect("created delta");

    assert_eq!(
        prev_root.len(),
        64,
        "prev_root_hex must be typed 32-byte hex"
    );
    assert_eq!(new_root.len(), 64, "new_root_hex must be typed 32-byte hex");
    assert_ne!(
        prev_root, new_root,
        "checkpoint transition must emit a new typed root"
    );
    assert!(
        spent.iter().all(|row| row
            .as_str()
            .is_some_and(|item| item.len() == 64 && !item.contains(':'))),
        "spent_delta must stay canonical asset_id hex only"
    );
    assert!(
        made.iter().all(|row| {
            row["asset_id_hex"]
                .as_str()
                .is_some_and(|item| item.len() == 64 && !item.contains(':'))
                && row["leaf_hash_hex"]
                    .as_str()
                    .is_some_and(|item| item.len() == 64 && !item.contains(':'))
        }),
        "created_delta must stay typed asset_id/leaf_hash rows"
    );
}

#[test]
fn test_cp_frag_links() {
    let (cp, prep, frag_a, frag_b) = load_cp_run();
    let prev_root = cp["prev_root_hex"].as_str().expect("checkpoint prev root");
    let new_root = cp["new_root_hex"].as_str().expect("checkpoint new root");
    let spent = cp["spent_delta"].as_array().expect("spent delta");
    let made = cp["created_delta"].as_array().expect("created delta");
    let frag_ids = cp["fragment_ids"].as_array().expect("fragment ids");

    assert_eq!(prev_root, hex::encode(prep.prev_root.as_bytes()));
    assert_eq!(
        frag_ids.len(),
        2,
        "checkpoint must reference both saved fragments"
    );
    assert_eq!(
        frag_ids[0].as_str().expect("frag id 1"),
        frag_a["id"].as_str().expect("frag a id")
    );
    assert_eq!(
        frag_ids[1].as_str().expect("frag id 2"),
        frag_b["id"].as_str().expect("frag b id")
    );
    assert_eq!(
        frag_a["prev_root_hex"].as_str().expect("frag a prev"),
        prev_root
    );
    assert_eq!(
        frag_b["prev_root_hex"].as_str().expect("frag b prev"),
        prev_root
    );
    assert_eq!(
        new_root.len(),
        64,
        "checkpoint new_root_hex must stay typed"
    );

    assert_frag_row(&frag_a, prev_root, spent, made);
    assert_frag_row(&frag_b, prev_root, spent, made);
}

#[test]
fn test_stage4_prep_order_kept() {
    let out = &cp_run_case().out;
    let prep = load_prep(out);
    let tx = load_tx(&tx_file(out));
    let prep_refs: Vec<(String, u32)> = prep
        .entries
        .iter()
        .map(|entry| {
            (
                hex::encode(entry.path().terminal_id().into_bytes()),
                entry.path().serial_id.get(),
            )
        })
        .collect();
    let tx_refs: Vec<(String, u32)> = tx
        .tx
        .inputs
        .iter()
        .map(|row| (row.asset_id_hex.clone(), row.serial_id))
        .collect();

    assert!(
        tx_refs
            .iter()
            .all(|input_ref| prep_refs.contains(input_ref)),
        "prep snapshot must contain every Stage 4 execution input"
    );
    assert!(
        prep_refs.len() >= tx_refs.len(),
        "canonical prep snapshot may materialize the full claim-backed store"
    );
}

#[test]
fn test_prep_file_ref_only() {
    let out = &cp_run_case().out;
    let prep_ref = load_prep_ref(out);
    let prep_json = load_json(&prep_file(out));
    let prep = load_prep(out);

    assert_eq!(
        prep_json.as_object().map(|item| item.len()),
        Some(1),
        "checkpoint_prep.json must stay a thin snapshot reference"
    );
    assert_eq!(
        prep_json["snapshot_id_hex"].as_str(),
        Some(prep_ref.snapshot_id_hex.as_str()),
        "prep ref must expose the persisted snapshot id only"
    );
    assert_eq!(
        hex::encode(prep.prev_root.as_bytes()),
        load_json(&cp_file(out))["prev_root_hex"]
            .as_str()
            .expect("cp prev root"),
        "Stage 6 must consume the canonical snapshot behind the prep reference"
    );
}

#[test]
fn test_stage6_needs_tx_pkg() {
    let verdict = verify_full_tx_package(br#"{"broken":true}"#)
        .expect("broken tx package should produce an invalid verdict, not panic");
    let msg = verdict.errors.join("; ");

    assert!(
        msg.contains("decode tx package failed")
            || msg.contains("stage4 tx package verification failed")
            || msg.contains("invalid structure"),
        "stage6 must still require execution input beyond the stored snapshot: {msg}"
    );
}
