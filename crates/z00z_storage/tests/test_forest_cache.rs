use std::sync::{Mutex, OnceLock};

use z00z_core::assets::{AssetLeaf, AssetPackPlain};
use z00z_crypto::ZkPackEncrypted;
use z00z_storage::settlement::{
    DefinitionId, ProofBlob, SettlementLeafFamily, SettlementPath, SettlementStore, StoreItem,
    TerminalId, TerminalLeaf,
};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvGuard {
    bucket_bits: Option<String>,
    redb_root: Option<String>,
}

impl EnvGuard {
    fn with_bucket_bits(bits: &str) -> Self {
        let guard = Self {
            bucket_bits: std::env::var("Z00Z_SETTLEMENT_BUCKET_BITS").ok(),
            redb_root: std::env::var("Z00Z_STORAGE_REDB_ROOT").ok(),
        };
        std::env::set_var("Z00Z_SETTLEMENT_BUCKET_BITS", bits);
        std::env::remove_var("Z00Z_STORAGE_REDB_ROOT");
        guard
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(bits) = &self.bucket_bits {
            std::env::set_var("Z00Z_SETTLEMENT_BUCKET_BITS", bits);
        } else {
            std::env::remove_var("Z00Z_SETTLEMENT_BUCKET_BITS");
        }
        if let Some(root) = &self.redb_root {
            std::env::set_var("Z00Z_STORAGE_REDB_ROOT", root);
        } else {
            std::env::remove_var("Z00Z_STORAGE_REDB_ROOT");
        }
    }
}

fn path(definition: u8, serial: u32, asset: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new([definition; 32]),
        z00z_storage::settlement::SerialId::new(serial),
        TerminalId::new([asset; 32]),
    )
}

fn leaf(path: SettlementPath, value: u64) -> TerminalLeaf {
    let payload = AssetPackPlain {
        value,
        blinding: [3u8; 32],
        s_out: [4u8; 32],
    }
    .to_bytes();

    AssetLeaf {
        asset_id: path.terminal_id().into_bytes(),
        serial_id: path.serial_id.get(),
        r_pub: [1u8; 32],
        owner_tag: [2u8; 32],
        c_amount: [5u8; 32],
        enc_pack: ZkPackEncrypted {
            version: 1,
            ciphertext: payload,
            tag: [0u8; 16],
        },
        range_proof: vec![9u8; 4],
        tag16: 11,
    }
    .into()
}

fn item(path: SettlementPath, value: u64) -> StoreItem {
    StoreItem::new(path, leaf(path, value)).expect("store item")
}

fn put(store: &mut SettlementStore, path: SettlementPath, value: u64) {
    store
        .put_settlement_item(item(path, value))
        .expect("put settlement item");
}

fn below_split_count(store: &SettlementStore) -> usize {
    usize::try_from(store.bucket_policy().min_bucket_count()).expect("usize")
}

fn proof(store: &SettlementStore, path: SettlementPath) -> ProofBlob {
    store
        .settlement_proof_blob(&path)
        .expect("settlement proof blob")
}

fn same_bucket_paths(store: &mut SettlementStore, need: usize) -> Vec<SettlementPath> {
    let policy = store.bucket_policy();
    let first = path(41, 9, 1);
    let bucket_id = first.bucket_id(policy);
    let mut out = vec![(1u8, first)];
    for seed in 2..=255 {
        let next = path(41, 9, seed);
        if next.bucket_id(policy) == bucket_id {
            out.push((seed, next));
            if out.len() == need {
                break;
            }
        }
    }

    assert_eq!(out.len(), need, "failed same bucket group");
    for (seed, next) in &out {
        put(store, *next, 4_100 + u64::from(*seed));
    }
    out.into_iter().map(|(_, path)| path).collect()
}

#[test]
fn test_cache_keeps_unchanged_roots() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let _env = EnvGuard::with_bucket_bits("2");
    let mut store = SettlementStore::new();
    let target = path(10, 1, 1);
    put(&mut store, target, 1_001);

    let warm = proof(&store, target);
    let hot = proof(&store, target);
    assert_eq!(warm, hot);

    let metrics_before = store.forest_cache_metrics();
    let bucket_root_before = warm
        .hjmt_bucket_root_leaf()
        .expect("bucket leaf")
        .terminal_jmt_root;
    put(&mut store, path(10, 2, 2), 2_002);
    let after_bucket = proof(&store, target);
    assert_eq!(
        after_bucket
            .hjmt_bucket_root_leaf()
            .expect("bucket leaf")
            .terminal_jmt_root,
        bucket_root_before
    );

    let serial_root_before = after_bucket.item().ser_leaf().serial_root;
    put(&mut store, path(10, 3, 3), 2_103);
    let after_serial = proof(&store, target);
    assert_eq!(
        after_serial.item().ser_leaf().serial_root,
        serial_root_before
    );

    let definition_root_before = after_serial.item().def_leaf().definition_root;
    put(&mut store, path(11, 1, 4), 3_004);
    let after_definition = proof(&store, target);
    assert_eq!(
        after_definition.item().def_leaf().definition_root,
        definition_root_before
    );

    let metrics_after = store.forest_cache_metrics();
    assert!(metrics_after.subtree_root.hits > metrics_before.subtree_root.hits);
    assert!(metrics_after.parent_leaf.hits > metrics_before.parent_leaf.hits);
    store.verify_forest_cache().expect("verify forest cache");
}

#[test]
fn test_cache_reuses_shared_proofs() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let _env = EnvGuard::with_bucket_bits("1");
    let mut store = SettlementStore::new();
    let steady_count = below_split_count(&store);
    let group = same_bucket_paths(&mut store, steady_count);

    let first = proof(&store, group[0]);
    let metrics_before = store.forest_cache_metrics();
    let second = proof(&store, group[1]);
    let metrics_after = store.forest_cache_metrics();

    assert_eq!(first.definition_proof(), second.definition_proof());
    assert_eq!(first.serial_proof(), second.serial_proof());
    assert_eq!(first.hjmt_bucket_proof(), second.hjmt_bucket_proof());
    assert!(metrics_after.proof_segment.hits > metrics_before.proof_segment.hits);
    store.verify_forest_cache().expect("verify forest cache");
}

#[test]
fn test_cache_invalidates_absence() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let _env = EnvGuard::with_bucket_bits("2");
    let mut store = SettlementStore::new();
    let missing = path(21, 1, 9);

    let absence = store
        .settlement_nonexistence_proof_blob(&missing, SettlementLeafFamily::Terminal)
        .expect("absence proof");
    store
        .validate_settlement_nonexistence_proof_blob(&absence, SettlementLeafFamily::Terminal)
        .expect("validate absence proof");
    let metrics_before = store.forest_cache_metrics();
    assert!(metrics_before.nonexistence.entries > 0);

    put(&mut store, missing, 2_109);
    let err = store
        .settlement_nonexistence_proof_blob(&missing, SettlementLeafFamily::Terminal)
        .expect_err("matching insert must invalidate absence cache");
    assert_eq!(err.to_string(), "settlement path is missing");

    let metrics_after = store.forest_cache_metrics();
    assert!(metrics_after.nonexistence.entries < metrics_before.nonexistence.entries);
    store.verify_forest_cache().expect("verify forest cache");
}

#[cfg(debug_assertions)]
#[test]
fn test_cache_eviction_keeps_proofs() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let _env = EnvGuard::with_bucket_bits("2");
    let mut store = SettlementStore::new();
    let target = path(31, 1, 1);
    let mut paths = vec![target];
    put(&mut store, target, 3_101);
    for seed in 2..=8 {
        let next = path(31, 1 + u32::from(seed % 3), seed);
        put(&mut store, next, 3_100 + u64::from(seed));
        paths.push(next);
    }

    let baseline = proof(&store, target);
    store.clear_forest_cache();
    store.set_forest_cache_test_limit(1);
    for next in paths {
        let blob = proof(&store, next);
        store
            .validate_settlement_proof_blob(&blob)
            .expect("validate hot proof");
    }

    let after = proof(&store, target);
    let metrics = store.forest_cache_metrics();
    assert!(metrics.subtree_root.evictions > 0 || metrics.proof_segment.evictions > 0);
    assert_eq!(after, baseline);
    store.verify_forest_cache().expect("verify forest cache");
}
