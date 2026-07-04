use std::{fs, path::PathBuf};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tempfile::tempdir;
use z00z_core::assets::AssetLeaf;
use z00z_crypto::expert::encoding::to_hex;
use z00z_storage::fixture_support::settlement_corpus::{
    assert_store_matches_oracle, same_bucket_paths_with_count, HjmtEnvGuard, OracleState,
};
use z00z_storage::settlement::{SettlementPath, SettlementStore, StoreItem, StoreOp, TerminalLeaf};
use z00z_utils::codec::{Codec, JsonCodec};

const BUCKET_COMMIT_REGEN_CMD: &str = "Z00Z_REGEN_DUMP=1 cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_commit test_bucket_manifest_matches -- --exact --nocapture";
const BUCKET_COMMIT_EVIDENCE_PTR: &str =
    "crates/z00z_storage/tests/test_hjmt_batch_commit.rs::test_bucket_manifest_matches";

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct BucketCommitManifest {
    version: u8,
    cases: Vec<BucketCommitCase>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct BucketCommitCase {
    id: String,
    kind: String,
    expected_verdict: String,
    bucket_bits: u8,
    seed: Vec<BucketCommitSeed>,
    ops: Vec<BucketCommitOp>,
    old_root_hex: String,
    expected_root_hex: String,
    touched_bucket_ids_hex: Vec<String>,
    expected_row_count: u32,
    reload_roundtrip: bool,
    regen_command: String,
    evidence_pointer: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct BucketCommitSeed {
    definition_mark: u8,
    serial_id: u32,
    terminal_mark: u8,
    item_mark: u8,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct BucketCommitOp {
    kind: String,
    definition_mark: u8,
    serial_id: u32,
    terminal_mark: u8,
    item_mark: Option<u8>,
}

fn item(path: SettlementPath, mark: u8) -> StoreItem {
    let mut core = AssetLeaf::dummy_for_scan(path.serial_id.get());
    core.asset_id = path.terminal_id().into_bytes();
    core.r_pub = [mark; 32];
    core.owner_tag = [mark.wrapping_add(1); 32];
    core.c_amount = [mark.wrapping_add(2); 32];
    core.range_proof = vec![mark; 8];
    StoreItem::new(path, TerminalLeaf::from(core)).expect("store item")
}

#[test]
fn test_bucket_manifest_matches() {
    let manifest = load_bucket_commit_manifest();
    let live = build_bucket_commit_manifest();
    if std::env::var_os("Z00Z_REGEN_DUMP").is_some() {
        let rendered = JsonCodec
            .serialize_pretty(&live)
            .expect("encode bucket commit manifest");
        println!("{}", String::from_utf8(rendered).expect("utf8 manifest"));
    }
    assert_eq!(manifest, live);
}

#[test]
fn test_batch_commit_matches_oracle() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = HjmtEnvGuard::with_bits("1");
    let mut selector = SettlementStore::new();
    let paths = same_bucket_paths_with_count(&mut selector, 41, 9, 4);
    drop(selector);

    let temp = tempdir()?;
    let mut store = SettlementStore::load(temp.path())?;
    let mut oracle = OracleState::default();
    let target_bucket = paths[0].bucket_id(store.bucket_policy());

    let mut ops = Vec::new();
    for (index, path) in paths.iter().copied().enumerate() {
        assert_eq!(path.bucket_id(store.bucket_policy()), target_bucket);
        let item = item(path, 0x40u8.wrapping_add(index as u8));
        oracle.put(item.clone())?;
        ops.push(StoreOp::Put(Box::new(item)));
    }

    let root = store.apply_settlement_ops(ops)?;
    assert_eq!(root, oracle.root()?);
    assert_store_matches_oracle(&store, &oracle);
    drop(store);

    let reloaded = SettlementStore::load(temp.path())?;
    assert_eq!(reloaded.settlement_root()?, root);
    assert_store_matches_oracle(&reloaded, &oracle);

    Ok(())
}

fn load_bucket_commit_manifest() -> BucketCommitManifest {
    load_json(
        fixture_root()
            .join("bucket_commit_equivalence")
            .join("manifest.json"),
    )
}

fn build_bucket_commit_manifest() -> BucketCommitManifest {
    BucketCommitManifest {
        version: 1,
        cases: vec![
            render_bucket_commit_case("BCM-G-001"),
            render_bucket_commit_case("BCM-G-002"),
        ],
    }
}

fn render_bucket_commit_case(case_id: &str) -> BucketCommitCase {
    match case_id {
        "BCM-G-001" => {
            let _guard = HjmtEnvGuard::with_bits("1");
            let mut selector = SettlementStore::new();
            let paths = same_bucket_paths_with_count(&mut selector, 41, 9, 4);
            drop(selector);

            let temp = tempdir().expect("tempdir");
            let mut store = SettlementStore::load(temp.path()).expect("load store");
            let mut oracle = OracleState::default();
            let old_root = store.settlement_root().expect("old root");
            let target_bucket = paths[0].bucket_id(store.bucket_policy());

            let ops = paths
                .iter()
                .copied()
                .enumerate()
                .map(|(index, path)| {
                    assert_eq!(path.bucket_id(store.bucket_policy()), target_bucket);
                    let mark = 0x40u8.wrapping_add(index as u8);
                    let item = item(path, mark);
                    oracle.put(item.clone()).expect("oracle put");
                    StoreOp::Put(Box::new(item))
                })
                .collect::<Vec<_>>();
            let expected_root = store
                .apply_settlement_ops(ops.clone())
                .expect("apply settlement ops");
            assert_eq!(expected_root, oracle.root().expect("oracle root"));
            assert_store_matches_oracle(&store, &oracle);
            drop(store);

            let reloaded = SettlementStore::load(temp.path()).expect("reload");
            assert_eq!(
                reloaded.settlement_root().expect("reloaded root"),
                expected_root
            );
            assert_store_matches_oracle(&reloaded, &oracle);

            BucketCommitCase {
                id: case_id.to_string(),
                kind: "same_bucket_insert_batch".to_string(),
                expected_verdict: "accept".to_string(),
                bucket_bits: 1,
                seed: Vec::new(),
                ops: paths
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|(index, path)| BucketCommitOp {
                        kind: "put".to_string(),
                        definition_mark: path.definition_id.into_bytes()[0],
                        serial_id: path.serial_id.get(),
                        terminal_mark: path.terminal_id().into_bytes()[1],
                        item_mark: Some(0x40u8.wrapping_add(index as u8)),
                    })
                    .collect(),
                old_root_hex: to_hex(old_root.as_bytes()),
                expected_root_hex: to_hex(expected_root.as_bytes()),
                touched_bucket_ids_hex: vec![to_hex(&target_bucket.into_bytes())],
                expected_row_count: 4,
                reload_roundtrip: true,
                regen_command: BUCKET_COMMIT_REGEN_CMD.to_string(),
                evidence_pointer: BUCKET_COMMIT_EVIDENCE_PTR.to_string(),
            }
        }
        "BCM-G-002" => {
            let _guard = HjmtEnvGuard::with_bits("1");
            let mut selector = SettlementStore::new();
            let paths = same_bucket_paths_with_count(&mut selector, 52, 17, 5);
            drop(selector);

            let temp = tempdir().expect("tempdir");
            let mut store = SettlementStore::load(temp.path()).expect("load store");
            let mut oracle = OracleState::default();
            let target_bucket = paths[0].bucket_id(store.bucket_policy());

            let seed_specs = paths[..3]
                .iter()
                .copied()
                .enumerate()
                .map(|(index, path)| {
                    let mark = 0x50u8.wrapping_add(index as u8);
                    let item = item(path, mark);
                    oracle.put(item.clone()).expect("oracle seed put");
                    store
                        .put_settlement_item(item)
                        .expect("seed settlement item");
                    BucketCommitSeed {
                        definition_mark: path.definition_id.into_bytes()[0],
                        serial_id: path.serial_id.get(),
                        terminal_mark: path.terminal_id().into_bytes()[1],
                        item_mark: mark,
                    }
                })
                .collect::<Vec<_>>();

            let deleted = paths[0];
            let replacement_a = item(paths[3], 0x63);
            let replacement_b = item(paths[4], 0x64);
            let old_root = store.settlement_root().expect("old root");
            oracle.delete(deleted).expect("oracle delete");
            oracle
                .put(replacement_a.clone())
                .expect("oracle replacement a");
            oracle
                .put(replacement_b.clone())
                .expect("oracle replacement b");

            let expected_root = store
                .apply_settlement_ops(vec![
                    StoreOp::Delete(deleted),
                    StoreOp::Put(Box::new(replacement_a)),
                    StoreOp::Put(Box::new(replacement_b)),
                ])
                .expect("apply replacement ops");
            assert_eq!(expected_root, oracle.root().expect("oracle root"));
            assert_store_matches_oracle(&store, &oracle);
            drop(store);

            let reloaded = SettlementStore::load(temp.path()).expect("reload");
            assert_eq!(
                reloaded.settlement_root().expect("reloaded root"),
                expected_root
            );
            assert_store_matches_oracle(&reloaded, &oracle);

            BucketCommitCase {
                id: case_id.to_string(),
                kind: "delete_and_replace_reload".to_string(),
                expected_verdict: "accept".to_string(),
                bucket_bits: 1,
                seed: seed_specs,
                ops: vec![
                    BucketCommitOp {
                        kind: "delete".to_string(),
                        definition_mark: deleted.definition_id.into_bytes()[0],
                        serial_id: deleted.serial_id.get(),
                        terminal_mark: deleted.terminal_id().into_bytes()[1],
                        item_mark: None,
                    },
                    BucketCommitOp {
                        kind: "put".to_string(),
                        definition_mark: paths[3].definition_id.into_bytes()[0],
                        serial_id: paths[3].serial_id.get(),
                        terminal_mark: paths[3].terminal_id().into_bytes()[1],
                        item_mark: Some(0x63),
                    },
                    BucketCommitOp {
                        kind: "put".to_string(),
                        definition_mark: paths[4].definition_id.into_bytes()[0],
                        serial_id: paths[4].serial_id.get(),
                        terminal_mark: paths[4].terminal_id().into_bytes()[1],
                        item_mark: Some(0x64),
                    },
                ],
                old_root_hex: to_hex(old_root.as_bytes()),
                expected_root_hex: to_hex(expected_root.as_bytes()),
                touched_bucket_ids_hex: vec![to_hex(&target_bucket.into_bytes())],
                expected_row_count: 4,
                reload_roundtrip: true,
                regen_command: BUCKET_COMMIT_REGEN_CMD.to_string(),
                evidence_pointer: BUCKET_COMMIT_EVIDENCE_PTR.to_string(),
            }
        }
        other => panic!("unknown bucket commit case {other}"),
    }
}

fn load_json<T>(path: PathBuf) -> T
where
    T: DeserializeOwned,
{
    let bytes =
        fs::read(&path).unwrap_or_else(|err| panic!("read {} failed: {err}", path.display()));
    JsonCodec
        .deserialize(&bytes)
        .unwrap_or_else(|err| panic!("decode {} failed: {err}", path.display()))
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hjmt_upgrade")
}

#[test]
fn test_batch_reload_matches_oracle() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = HjmtEnvGuard::with_bits("1");
    let mut selector = SettlementStore::new();
    let paths = same_bucket_paths_with_count(&mut selector, 52, 17, 5);
    drop(selector);

    let temp = tempdir()?;
    let mut store = SettlementStore::load(temp.path())?;
    let mut oracle = OracleState::default();

    let mut seed_ops = Vec::new();
    for (index, path) in paths[..3].iter().copied().enumerate() {
        let item = item(path, 0x50u8.wrapping_add(index as u8));
        oracle.put(item.clone())?;
        seed_ops.push(StoreOp::Put(Box::new(item)));
    }
    store.apply_settlement_ops(seed_ops)?;

    let deleted = paths[0];
    oracle.delete(deleted)?;
    let replacement_a = item(paths[3], 0x63);
    let replacement_b = item(paths[4], 0x64);
    oracle.put(replacement_a.clone())?;
    oracle.put(replacement_b.clone())?;

    let root = store.apply_settlement_ops(vec![
        StoreOp::Delete(deleted),
        StoreOp::Put(Box::new(replacement_a)),
        StoreOp::Put(Box::new(replacement_b)),
    ])?;
    assert_eq!(root, oracle.root()?);
    assert_store_matches_oracle(&store, &oracle);
    drop(store);

    let reloaded = SettlementStore::load(temp.path())?;
    assert_eq!(reloaded.settlement_root()?, root);
    assert_store_matches_oracle(&reloaded, &oracle);

    Ok(())
}
