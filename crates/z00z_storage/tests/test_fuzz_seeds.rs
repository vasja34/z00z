use std::fs;

use z00z_core::assets::AssetLeaf;
use z00z_storage::settlement::{
    BucketOccupancyEvidence, BucketPolicy, DefinitionId, FeeEnvelope, MergeProof,
    PolicyTransitionProof, ProofBlob, SerialId, SettlementLeaf, SettlementLeafFamily,
    SettlementPath, SettlementStore, SplitProof, TerminalId, TerminalLeaf,
};
use z00z_utils::codec::{BincodeCodec, Codec};

const README: &str = include_str!("../fuzz/seeds/settlement_proofs/README.md");

fn bytes(value: u8) -> [u8; 32] {
    [value; 32]
}

fn seed_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fuzz/seeds/settlement_proofs")
}

fn item_for_path(path: SettlementPath) -> z00z_storage::settlement::StoreItem {
    let mut core = AssetLeaf::dummy_for_scan(path.serial_id.get());
    core.asset_id = path.terminal_id().into_bytes();
    let leaf = TerminalLeaf::from(core);
    z00z_storage::settlement::StoreItem::new(path, leaf).expect("seed item")
}

fn seeded_store() -> SettlementStore {
    let mut store = SettlementStore::new();
    for mark in [41u8, 42, 43] {
        let path = SettlementPath::new(
            DefinitionId::new(bytes(31)),
            SerialId::new(u32::from(mark - 40)),
            TerminalId::new(bytes(mark)),
        );
        let _ = store.put_settlement_item(item_for_path(path));
    }
    store
}

fn next_policy(store: &SettlementStore) -> BucketPolicy {
    BucketPolicy::new(
        store.bucket_policy().bucket_bits() + 1,
        store.bucket_policy().min_bucket_count(),
        store.bucket_policy().max_target_leaf_count(),
        store.bucket_policy().compatibility_generation() + 1,
    )
    .expect("next policy")
}

fn dispatch_seed(seed: &[u8]) {
    let mode = seed.first().copied().unwrap_or(0) % 8;
    let bytes = seed.get(1..).unwrap_or(&[]);
    let codec = BincodeCodec;

    match mode {
        0 => {
            let _ = SettlementLeaf::decode(bytes);
        }
        1 => {
            let _: Result<SettlementPath, _> = codec.deserialize(bytes);
        }
        2 => {
            let _: Result<FeeEnvelope, _> = codec.deserialize(bytes);
        }
        3 => {
            if let Ok(blob) = codec.deserialize::<ProofBlob>(bytes) {
                let store = seeded_store();
                let _ = store.validate_settlement_proof_blob(&blob);
                let _ = store.validate_settlement_nonexistence_proof_blob(
                    &blob,
                    SettlementLeafFamily::Terminal,
                );
                let _ = store.validate_settlement_nonexistence_proof_blob(
                    &blob,
                    SettlementLeafFamily::Right,
                );
                let _ = store.validate_settlement_nonexistence_proof_blob(
                    &blob,
                    SettlementLeafFamily::Voucher,
                );
            }
        }
        4 => {
            let _: Result<BucketOccupancyEvidence, _> = codec.deserialize(bytes);
        }
        5 => {
            if let Ok(proof) = codec.deserialize::<PolicyTransitionProof>(bytes) {
                let store = seeded_store();
                let _ = store.validate_policy_transition_proof(&proof, next_policy(&store));
            }
        }
        6 => {
            if let Ok(proof) = codec.deserialize::<SplitProof>(bytes) {
                let store = seeded_store();
                let _ = store.validate_split_proof(&proof);
            }
        }
        _ => {
            if let Ok(proof) = codec.deserialize::<MergeProof>(bytes) {
                let store = seeded_store();
                let _ = store.validate_merge_proof(&proof);
            }
        }
    }
}

#[test]
fn test_fuzz_docs_cover_surfaces() {
    for needle in [
        "SettlementLeaf::decode",
        "SettlementPath",
        "FeeEnvelope",
        "ProofBlob",
        "validate_settlement_proof_blob",
        "voucher-leaf family drift",
        "validate_split_proof",
        "validate_merge_proof",
        "validate_policy_transition_proof",
        "epoch drift",
        "root drift",
        "policy-transition replay",
    ] {
        assert!(README.contains(needle), "README missing {needle}");
    }
}

#[test]
fn test_fuzz_files_cover_dispatch() {
    let mut seen = [false; 8];
    let entries = fs::read_dir(seed_dir()).expect("seed dir");
    let mut seed_count = 0usize;
    for entry in entries {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("seed") {
            continue;
        }
        let data = fs::read(&path).expect("seed bytes");
        let mode = usize::from(data.first().copied().unwrap_or(0) % 8);
        seen[mode] = true;
        seed_count += 1;
        dispatch_seed(&data);
    }

    assert!(seed_count >= 8, "expected at least 8 seed files");
    assert!(seen.into_iter().all(std::convert::identity));
}
