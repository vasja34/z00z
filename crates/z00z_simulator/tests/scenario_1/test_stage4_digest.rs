use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use z00z_storage::settlement::{CheckRoot, SettlementStore, StoreItem, StoreOp};
use z00z_storage::snapshot::{PrepFsStore, PrepSnapshot, PrepSnapshotId, PrepSnapshotStore};
use z00z_utils::{codec::Value, io::load_json};
use z00z_wallets::tx::TxPackage;

use z00z_simulator::scenario_1::stage_6::shared_cases;

#[derive(Debug, Clone, PartialEq, Eq)]
struct InputRef {
    asset_id: [u8; 32],
    serial_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunRep {
    digest: String,
    snap_digest: String,
    input_refs: Vec<InputRef>,
    has_spend_proof: bool,
    has_spend_auth: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PrepRefFile {
    snapshot_id_hex: String,
}

struct OutCase {
    out: PathBuf,
}

fn pkg_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/tx_alice_to_bob_pkg.json")
}

fn snap_file(out: &Path) -> std::path::PathBuf {
    out.join("stage_4_snapshot.json")
}

fn prep_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/checkpoint_prep.json")
}

fn shared_case(out: PathBuf) -> OutCase {
    assert!(pkg_file(&out).exists(), "cached stage4 tx package missing");
    assert!(snap_file(&out).exists(), "cached stage4 snapshot missing");
    assert!(prep_file(&out).exists(), "cached stage4 prep missing");
    OutCase { out }
}

fn read_rep(out: &Path) -> RunRep {
    let pkg: TxPackage = load_json(pkg_file(out)).expect("tx package");
    let snap: Value = load_json(snap_file(out)).expect("stage4 snapshot");
    let snap_digest = snap["tx_digest_hex"]
        .as_str()
        .expect("snapshot digest")
        .to_string();

    RunRep {
        digest: pkg.tx_digest_hex.clone(),
        snap_digest,
        input_refs: pkg
            .tx
            .inputs
            .iter()
            .map(|row| InputRef {
                asset_id: hex::decode(&row.asset_id_hex)
                    .expect("asset_id_hex decode")
                    .try_into()
                    .expect("asset_id_hex size"),
                serial_id: row.serial_id,
            })
            .collect(),
        has_spend_proof: pkg.tx.proof.spend.is_some(),
        has_spend_auth: pkg.tx.auth.spend.is_some(),
    }
}

fn seed_case_a() -> &'static OutCase {
    static CASE: std::sync::OnceLock<OutCase> = std::sync::OnceLock::new();
    CASE.get_or_init(|| shared_case(shared_cases::default_stage6_out()))
}

fn seed_case_b() -> &'static OutCase {
    static CASE: std::sync::OnceLock<OutCase> = std::sync::OnceLock::new();
    CASE.get_or_init(|| shared_case(shared_cases::default_stage6_rerun_out()))
}

fn fraction_case() -> &'static OutCase {
    static CASE: std::sync::OnceLock<OutCase> = std::sync::OnceLock::new();
    CASE.get_or_init(|| shared_case(shared_cases::fraction_02_stage6_out()))
}

fn load_prep(out: &Path) -> PrepSnapshot {
    let prep_ref: PrepRefFile = load_json(prep_file(out)).expect("prep ref");
    let raw = hex::decode(prep_ref.snapshot_id_hex).expect("snapshot id hex");
    let snap_id = PrepSnapshotId::new(raw.try_into().expect("snapshot id size"));
    let store = PrepFsStore::new(out.join("transactions"));
    store.load_snapshot(&snap_id).expect("prep snapshot")
}

fn assert_snapshot_digest_alignment(rep: &RunRep, label: &str) {
    assert_eq!(
        rep.digest, rep.snap_digest,
        "{label} snapshot digest mismatch"
    );
    assert!(
        rep.has_spend_proof && rep.has_spend_auth,
        "{label} must keep spend proof + auth surface"
    );
}

fn storage_root(snapshot: &PrepSnapshot) -> CheckRoot {
    let mut store = SettlementStore::new();
    let ops = snapshot
        .entries
        .iter()
        .map(|entry| {
            let item = StoreItem::new(entry.path(), entry.leaf().clone()).expect("store item");
            StoreOp::Put(Box::new(item))
        })
        .collect::<Vec<_>>();
    store.apply_settlement_ops(ops).expect("apply ops");
    CheckRoot::from(store.settlement_root().expect("storage root"))
}

fn lossy_root(snapshot: &PrepSnapshot) -> CheckRoot {
    let mut store = SettlementStore::new();
    let ops = snapshot
        .entries
        .iter()
        .map(|entry| {
            let path = z00z_storage::settlement::SettlementPath::new(
                z00z_storage::settlement::DefinitionId::new([0; 32]),
                entry.path().serial_id,
                entry.path().terminal_id(),
            );
            let item = StoreItem::new(path, entry.leaf().clone()).expect("lossy store item");
            StoreOp::Put(Box::new(item))
        })
        .collect::<Vec<_>>();
    store.apply_settlement_ops(ops).expect("apply ops");
    CheckRoot::from(store.settlement_root().expect("lossy root"))
}

#[test]
fn test_stage4_digest_snapshot_alignment() {
    let rep = read_rep(&seed_case_a().out);
    assert_snapshot_digest_alignment(&rep, "default stage4 seed");
}

#[test]
#[ignore = "heavy replay-contract suite; run explicitly or via full_verify max-safe"]
fn test_stage4_digest_replay_heavy() {
    // Replay claim is valid only when Stage 3 seed, Stage 4 seed, selection shape, and tx config stay fixed.
    // Fresh scenario runs recreate upstream wallet material, so canonical input asset IDs may differ
    // even when the selected serial set stays stable.
    let rep_a = read_rep(&seed_case_a().out);
    let rep_b = read_rep(&seed_case_b().out);
    let rep_c = read_rep(&fraction_case().out);

    assert_snapshot_digest_alignment(&rep_a, "run A");
    assert_snapshot_digest_alignment(&rep_b, "run B");
    assert_snapshot_digest_alignment(&rep_c, "run C");

    let serials_a: Vec<u32> = rep_a.input_refs.iter().map(|row| row.serial_id).collect();
    let serials_b: Vec<u32> = rep_b.input_refs.iter().map(|row| row.serial_id).collect();
    let serials_c: Vec<u32> = rep_c.input_refs.iter().map(|row| row.serial_id).collect();

    assert_eq!(serials_a, serials_b, "selected serials must stay fixed");
    assert_eq!(
        serials_a, serials_c,
        "semantic change must not change selected serials"
    );
    assert_ne!(
        rep_a.digest, rep_c.digest,
        "semantic tx change must change digest"
    );
}

#[test]
fn test_stage4_tracks_prior_root() {
    // This gate is intentionally scoped to the accepted Stage 4 row shape only.
    // A lossy root that drops the canonical definition binding must not match the
    // storage-owned root. Stage 4 persists the exact storage-produced settlement root.
    let prep = load_prep(&seed_case_a().out);
    let storage = storage_root(&prep);
    let prior = lossy_root(&prep);

    assert_eq!(storage, prep.prev_root);
    assert_ne!(
        storage, prior,
        "lossy definition-stripped root must not drive stage4 prep"
    );
}
