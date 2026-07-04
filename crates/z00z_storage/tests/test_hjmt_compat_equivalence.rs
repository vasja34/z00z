use std::{fs, path::PathBuf};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use z00z_crypto::expert::encoding::to_hex;
use z00z_storage::fixture_support::settlement_corpus::{
    self, assert_store_matches_oracle, asset_item, fee_actor, fee_del_ops, fee_envelope,
    fee_put_ops, list_items, load_fixture, redb_store, right_ctx, right_leaf, right_path,
    transferred_right_leaf, AssetSeed, FixtureRightClass, HjmtEnvGuard, OracleState, RightSeed,
};
use z00z_storage::settlement::{RightActionCtx, SettlementLeaf, SettlementStore};
use z00z_utils::codec::{Codec, JsonCodec};

const COMPAT_EQ_REGEN_CMD: &str = "Z00Z_REGEN_DUMP=1 cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_compat_equivalence test_manifest_matches_contract -- --exact --nocapture";
const COMPAT_EQ_EVIDENCE_PTR: &str =
    "crates/z00z_storage/tests/test_hjmt_compat_equivalence.rs::test_manifest_matches_contract";
const COMPAT_SEEDS: [u64; 8] = [0, 7, 13, 29, 41, 53, 61, 63];

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CompatManifest {
    version: u8,
    cases: Vec<CompatCase>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CompatCase {
    id: String,
    seed: u64,
    expected_verdict: String,
    bucket_bits: u8,
    operation_kinds: Vec<String>,
    terminal_branch: String,
    final_root_hex: String,
    expected_row_count: u32,
    reload_roundtrip: bool,
    regen_command: String,
    evidence_pointer: String,
}

fn seeded_asset(seed: u64) -> AssetSeed {
    let mark = 101u8.wrapping_add(seed as u8);
    AssetSeed {
        label: format!("compat_asset_{mark}"),
        definition_mark: 70u8.wrapping_add((seed as u8) % 11),
        serial_id: 10 + ((seed % 5) as u32),
        terminal_mark: mark,
        value: 5_000 + (seed % 1_000),
    }
}

fn seeded_right(seed: u64) -> RightSeed {
    let mark = 151u8.wrapping_add(seed as u8);
    let right_class = match seed % 5 {
        0 => FixtureRightClass::MachineCapability,
        1 => FixtureRightClass::DataAccess,
        2 => FixtureRightClass::ServiceEntitlement,
        3 => FixtureRightClass::ValidatorMandate,
        _ => FixtureRightClass::OneTimeUse,
    };
    RightSeed {
        label: format!("compat_right_{mark}"),
        definition_mark: 90u8.wrapping_add((seed as u8) % 11),
        serial_id: 20 + ((seed % 5) as u32),
        terminal_mark: mark,
        right_class,
    }
}

#[test]
fn test_manifest_matches_contract() {
    let manifest = load_manifest();
    let live = build_manifest();
    if std::env::var_os("Z00Z_REGEN_DUMP").is_some() {
        let rendered = JsonCodec
            .serialize_pretty(&live)
            .expect("encode compat manifest");
        println!("{}", String::from_utf8(rendered).expect("utf8 manifest"));
    }
    assert_eq!(manifest, live);
}

#[test]
fn test_seeded_ops_match_oracle() -> Result<(), Box<dyn std::error::Error>> {
    for seed in COMPAT_SEEDS {
        let case = run_case(seed)?;
        assert_eq!(case.expected_verdict, "accept");
        assert!(case.reload_roundtrip);
    }
    Ok(())
}

fn load_manifest() -> CompatManifest {
    load_json(
        fixture_root()
            .join("compat_equivalence_random_ops")
            .join("manifest.json"),
    )
}

fn build_manifest() -> CompatManifest {
    CompatManifest {
        version: 1,
        cases: COMPAT_SEEDS
            .into_iter()
            .enumerate()
            .map(|(index, seed)| render_case(index, seed))
            .collect(),
    }
}

fn render_case(index: usize, seed: u64) -> CompatCase {
    run_case(seed)
        .unwrap_or_else(|err| panic!("compat case {} seed {} failed: {err}", index + 1, seed))
}

fn run_case(seed: u64) -> Result<CompatCase, Box<dyn std::error::Error>> {
    let (_env, temp, mut store) = isolated_store()?;
    let mut oracle = OracleState::default();
    let fixture = load_fixture();
    for item in settlement_corpus::load_fixture_items(&fixture) {
        store.put_settlement_item(item.clone())?;
        oracle.put(item)?;
    }

    let asset = seeded_asset(seed);
    let right = seeded_right(seed);
    let asset_item = asset_item(&asset);
    let created_path = right_path(&right);
    let created_leaf = right_leaf(&right);
    let transferred = transferred_right_leaf(created_leaf, right.terminal_mark);

    store.put_settlement_item(asset_item.clone())?;
    oracle.put(asset_item)?;
    assert_store_matches_oracle(&store, &oracle);

    store.create_right_with_fee(
        created_path,
        created_leaf,
        right_ctx(&created_leaf, 15),
        fee_envelope(
            right.terminal_mark,
            store.fee_support_ctx(&fee_put_ops(
                created_path,
                SettlementLeaf::Right(created_leaf),
            )?)?,
        ),
        fee_actor(right.terminal_mark, 15),
    )?;
    oracle.create_right(created_path, created_leaf)?;
    assert_store_matches_oracle(&store, &oracle);

    store.transfer_right_with_fee(
        created_path,
        transferred,
        right_ctx(&transferred, 15),
        fee_envelope(
            right.terminal_mark.wrapping_add(1),
            store.fee_support_ctx(&fee_put_ops(
                created_path,
                SettlementLeaf::Right(transferred),
            )?)?,
        ),
        fee_actor(right.terminal_mark.wrapping_add(1), 15),
    )?;
    oracle.transfer_right(created_path, transferred, 15)?;
    assert_store_matches_oracle(&store, &oracle);

    let terminal_branch = match seed % 3 {
        0 => {
            store.consume_right_with_fee(
                created_path,
                right_ctx(&transferred, 15),
                fee_envelope(
                    right.terminal_mark.wrapping_add(2),
                    store.fee_support_ctx(&fee_del_ops(created_path))?,
                ),
                fee_actor(right.terminal_mark.wrapping_add(2), 15),
            )?;
            oracle.consume_right(created_path, 15)?;
            "consume_right"
        }
        1 => {
            store.revoke_right_with_fee(
                created_path,
                right_ctx(&transferred, 15),
                fee_envelope(
                    right.terminal_mark.wrapping_add(2),
                    store.fee_support_ctx(&fee_del_ops(created_path))?,
                ),
                fee_actor(right.terminal_mark.wrapping_add(2), 15),
            )?;
            oracle.revoke_right(created_path, 15)?;
            "revoke_right"
        }
        _ => {
            store.expire_right(
                created_path,
                RightActionCtx {
                    now: 25,
                    ..RightActionCtx::default()
                },
            )?;
            oracle.expire_right(created_path, 25)?;
            "expire_right"
        }
    };
    assert_store_matches_oracle(&store, &oracle);

    let final_root = store.settlement_root()?;
    let row_count = list_items(&store)?.len() as u32;
    drop(store);

    let reloaded = SettlementStore::load(temp.path())?;
    assert_eq!(reloaded.settlement_root()?, final_root);
    assert_eq!(list_items(&reloaded)?.len() as u32, row_count);
    assert_store_matches_oracle(&reloaded, &oracle);

    Ok(CompatCase {
        id: format!(
            "CEQ-G-{:03}",
            COMPAT_SEEDS
                .iter()
                .position(|candidate| *candidate == seed)
                .expect("seed position")
                + 1
        ),
        seed,
        expected_verdict: "accept".to_string(),
        bucket_bits: reloaded.bucket_policy().bucket_bits(),
        operation_kinds: vec![
            "put_asset".to_string(),
            "create_right".to_string(),
            "transfer_right".to_string(),
            terminal_branch.to_string(),
        ],
        terminal_branch: terminal_branch.to_string(),
        final_root_hex: to_hex(final_root.as_bytes()),
        expected_row_count: row_count,
        reload_roundtrip: true,
        regen_command: COMPAT_EQ_REGEN_CMD.to_string(),
        evidence_pointer: COMPAT_EQ_EVIDENCE_PTR.to_string(),
    })
}

fn isolated_store(
) -> Result<(HjmtEnvGuard, tempfile::TempDir, SettlementStore), Box<dyn std::error::Error>> {
    redb_store()
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
