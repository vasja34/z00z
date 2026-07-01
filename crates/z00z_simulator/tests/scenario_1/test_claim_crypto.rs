#![cfg(feature = "wallet_debug_tools")]

use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use z00z_core::{Asset, AssetClass};
use z00z_crypto::{verify_range_proof, AGGREGATION_FACTOR, MIN_VALUE_PROMISE, RANGE_PROOF_BITS};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{load_bincode, load_json, load_json_bounded, read_to_string},
};

#[derive(Clone, serde::Deserialize)]
struct ClaimRow {
    amount: u64,
}

use z00z_simulator::scenario_1::support::claim_shared_cases;

static CLAIM_OUT: OnceLock<PathBuf> = OnceLock::new();

fn claim_out() -> &'static PathBuf {
    CLAIM_OUT.get_or_init(claim_shared_cases::default_stage3_out)
}

fn load_actor_assets(out: &Path, name: &str) -> Vec<Asset> {
    let p = out
        .join("claim")
        .join(format!("export_wallet_debug_{name}.json"));
    let v: serde_json::Value = load_json_bounded(&p, 64 * 1024 * 1024).expect("load debug dump");
    serde_json::from_value(
        v.get("imported_assets_full")
            .cloned()
            .expect("imported_assets_full present"),
    )
    .expect("decode imported assets")
}

fn sum_amt(items: &[Asset]) -> u128 {
    items.iter().map(|a| u128::from(a.amount)).sum()
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
fn test_stage3_crypto_ok() {
    let out = claim_out();

    let mut all = Vec::<Asset>::new();
    for name in ["alice", "bob", "charlie"] {
        let items = load_actor_assets(out, name);
        assert!(!items.is_empty(), "{name} has no imported assets");

        for item in &items {
            assert!(item.r_pub.is_none(), "r_pub must be None after claim");
            assert!(
                item.owner_tag.is_none(),
                "owner_tag must be None after claim"
            );
            assert!(item.enc_pack.is_none(), "enc_pack must be None after claim");
            assert!(item.secret.is_none(), "secret must be None after claim");
            assert!(item.tag16.is_none(), "tag16 must be None after claim");

            match item.definition.class {
                AssetClass::Coin | AssetClass::Token => {
                    assert!(item.amount > 0, "coin/token amount must be > 0")
                }
                AssetClass::Nft | AssetClass::Void => {}
            }

            let proof = item.range_proof.as_ref().expect("range proof exists");
            assert!(!proof.is_empty(), "range proof must be non-empty");
            verify_range_proof(
                proof,
                &item.commitment,
                RANGE_PROOF_BITS,
                AGGREGATION_FACTOR,
                MIN_VALUE_PROMISE,
            )
            .expect("range proof verify");

            item.verify_complete().expect("asset full verify");
        }

        all.extend(items);
    }

    let snap_path = out.join("stage_3_snapshot.json");
    let snap_text = read_to_string(&snap_path).expect("read stage_3_snapshot");
    let snap: serde_json::Value = JsonCodec
        .deserialize(snap_text.as_bytes())
        .expect("decode stage_3_snapshot");

    let in_cnt = snap
        .get("input_assets_count")
        .and_then(|v| v.as_u64())
        .expect("input_assets_count") as usize;
    let dist_cnt = snap
        .get("distributed_assets_count")
        .and_then(|v| v.as_u64())
        .expect("distributed_assets_count") as usize;
    assert_eq!(in_cnt, dist_cnt);
    assert_eq!(dist_cnt, all.len());

    let amt_claim: u128 = ["alice", "bob", "charlie"]
        .iter()
        .map(|name| {
            let p = out.join("claim").join(format!("claim_rows_{name}.json"));
            let rows: Vec<ClaimRow> = load_json(&p).expect("load claimed rows");
            rows.iter().map(|row| u128::from(row.amount)).sum::<u128>()
        })
        .sum();

    let amt_import = sum_amt(&all);
    assert_eq!(amt_claim, amt_import, "sum(amount) mismatch");

    let claimed = all;

    let mut src = Vec::<Asset>::new();
    for p in list_gen_bins(out) {
        let mut part: Vec<Asset> = load_bincode(&p).expect("load genesis bin");
        src.append(&mut part);
    }

    assert_eq!(src.len(), claimed.len(), "asset count mismatch");
    assert_eq!(sum_amt(&src), sum_amt(&claimed), "amount total mismatch");
}
