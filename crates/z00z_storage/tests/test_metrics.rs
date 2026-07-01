use z00z_storage::settlement::{ForestCacheMetrics, SettlementLookup, SettlementStore};

use z00z_storage::fixture_support::settlement_corpus;

use z00z_storage::fixture_support::settlement_corpus::{
    load_fixture, load_fixture_items, redb_store_with_bits,
};

fn total_hits(metrics: &ForestCacheMetrics) -> u64 {
    metrics.subtree_root.hits
        + metrics.parent_leaf.hits
        + metrics.terminal_leaf.hits
        + metrics.bucket_derivation.hits
        + metrics.proof_segment.hits
        + metrics.nonexistence.hits
        + metrics.policy_proof.hits
        + metrics.journal_digest.hits
        + metrics.path_index.hits
}

#[test]
fn test_not_change_proof_authority() {
    let fixture = load_fixture();
    let sample_path = settlement_corpus::asset_path(&fixture.assets[0]);
    let items = load_fixture_items(&fixture);
    let _guard = settlement_corpus::HjmtEnvGuard::with_bits("2");
    let mut store = SettlementStore::new();
    store
        .apply_settlement_ops(
            items
                .iter()
                .cloned()
                .map(|item| z00z_storage::settlement::StoreOp::Put(Box::new(item)))
                .collect(),
        )
        .expect("seed fixture");

    let cold = store
        .settlement_proof_blob(&sample_path)
        .expect("cold proof")
        .encode()
        .expect("cold bytes");

    let batch_paths = items
        .iter()
        .take(8)
        .map(|item| item.path())
        .collect::<Vec<_>>();
    let _ = store
        .settlement_proof_blobs(&batch_paths)
        .expect("proof batch through scheduler");
    let _ = store
        .lookup_settlement(SettlementLookup::Terminal(sample_path.terminal_id))
        .expect("terminal lookup");
    let _ = store
        .lookup_settlement(SettlementLookup::Path(sample_path))
        .expect("path lookup");

    let warm = store
        .settlement_proof_blob(&sample_path)
        .expect("warm proof")
        .encode()
        .expect("warm bytes");
    assert_eq!(warm, cold);

    let cache = store.forest_cache_metrics();
    let sched = store.forest_scheduler_metrics();
    assert!(total_hits(&cache) > 0);
    assert!(sched.last_batch >= batch_paths.len());
}

#[test]
fn test_bounded_reload_safe() {
    let fixture = load_fixture();
    let items = load_fixture_items(&fixture);
    let sample_path = settlement_corpus::asset_path(&fixture.assets[0]);
    let (guard, temp, mut store) = redb_store_with_bits(Some("2")).expect("redb store");
    let _guard = guard;
    store
        .apply_settlement_ops(
            items
                .iter()
                .cloned()
                .map(|item| z00z_storage::settlement::StoreOp::Put(Box::new(item)))
                .collect(),
        )
        .expect("seed fixture");

    let _ = store
        .settlement_proof_blob(&sample_path)
        .expect("warm proof");
    let _ = store
        .settlement_proof_blobs(
            &items
                .iter()
                .take(6)
                .map(|item| item.path())
                .collect::<Vec<_>>(),
        )
        .expect("proof batch");
    let before_cache = store.forest_cache_metrics();
    let before_sched = store.forest_scheduler_metrics();
    let root_denom = (before_cache.subtree_root.hits + before_cache.subtree_root.misses).max(1);
    let proof_denom = (before_cache.proof_segment.hits + before_cache.proof_segment.misses).max(1);
    let root_ratio = before_cache.subtree_root.hits as f64 / root_denom as f64;
    let proof_ratio = before_cache.proof_segment.hits as f64 / proof_denom as f64;
    assert!((0.0..=1.0).contains(&root_ratio));
    assert!((0.0..=1.0).contains(&proof_ratio));
    assert_eq!(before_sched.reject_count, 0);

    let before_bytes = store
        .settlement_proof_blob(&sample_path)
        .expect("proof before reload")
        .encode()
        .expect("proof bytes before reload");

    drop(store);
    let reloaded = SettlementStore::load(temp.path()).expect("reload");
    let reloaded_bytes = reloaded
        .settlement_proof_blob(&sample_path)
        .expect("proof after reload")
        .encode()
        .expect("proof bytes after reload");
    assert_eq!(before_bytes, reloaded_bytes);

    let after_cache = reloaded.forest_cache_metrics();
    let after_sched = reloaded.forest_scheduler_metrics();
    let after_root_denom = (after_cache.subtree_root.hits + after_cache.subtree_root.misses).max(1);
    let after_proof_denom =
        (after_cache.proof_segment.hits + after_cache.proof_segment.misses).max(1);
    let after_root_ratio = after_cache.subtree_root.hits as f64 / after_root_denom as f64;
    let after_proof_ratio = after_cache.proof_segment.hits as f64 / after_proof_denom as f64;
    assert!((0.0..=1.0).contains(&after_root_ratio));
    assert!((0.0..=1.0).contains(&after_proof_ratio));
    assert_eq!(after_sched.reject_count, 0);
}
