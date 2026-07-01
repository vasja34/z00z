use tempfile::tempdir;
use z00z_core::assets::AssetLeaf;
use z00z_storage::fixture_support::settlement_corpus::{
    next_policy, sibling_bucket_pair, split_ready_paths, HjmtEnvGuard,
};
use z00z_storage::settlement::{
    AdaptiveProofErr, BucketEpoch, BucketPolicy, MergeProof, PolicyTransitionProof, SettlementPath,
    SettlementStateRoot, SettlementStore, SplitProof, StoreItem, TerminalLeaf,
};

fn item(path: SettlementPath, mark: u8) -> StoreItem {
    let mut core = AssetLeaf::dummy_for_scan(path.serial_id.get());
    core.asset_id = path.terminal_id().into_bytes();
    core.r_pub = [mark; 32];
    core.owner_tag = [mark.wrapping_add(1); 32];
    core.c_amount = [mark.wrapping_add(2); 32];
    core.range_proof = vec![mark; 8];
    StoreItem::new(path, TerminalLeaf::from(core)).expect("store item")
}

fn path(def_mark: u8, serial: u32, term_mark: u8) -> SettlementPath {
    SettlementPath::new(
        z00z_storage::settlement::DefinitionId::new([def_mark; 32]),
        z00z_storage::settlement::SerialId::new(serial),
        z00z_storage::settlement::TerminalId::new([term_mark; 32]),
    )
}

#[test]
fn test_split_transition_reload_bound() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = HjmtEnvGuard::with_bits("1");
    let temp = tempdir()?;

    let mut store = SettlementStore::load(temp.path())?;
    let paths = split_ready_paths(&mut store, 41, 9);
    let anchor = paths[0];
    let split = store.split_proof(&anchor)?;
    let policy = next_policy(&store);
    let transition = store.policy_transition_proof(policy)?;
    drop(store);

    let mut reloaded = SettlementStore::load(temp.path())?;
    reloaded.validate_split_proof(&split)?;
    reloaded.validate_policy_transition_proof(&transition, policy)?;
    reloaded.put_settlement_item(item(path(98, 1, 201), 0x91))?;
    reloaded.validate_split_proof(&split)?;
    reloaded.validate_policy_transition_proof(&transition, policy)?;

    let split_err = reloaded
        .validate_split_proof(&SplitProof {
            prior_root: SettlementStateRoot::settlement_v1([0x42; 32]),
            ..split
        })
        .expect_err("wrong split root must reject");
    assert!(matches!(split_err, AdaptiveProofErr::WrongOldRoot));

    let transition_err = reloaded
        .validate_policy_transition_proof(
            &PolicyTransitionProof {
                prior_epoch: BucketEpoch::new(transition.prior_epoch.get() + 1),
                ..transition
            },
            policy,
        )
        .expect_err("wrong transition epoch must reject");
    assert!(matches!(transition_err, AdaptiveProofErr::WrongEpoch));

    Ok(())
}

#[test]
fn test_merge_transition_rejects_drift() {
    let _guard = HjmtEnvGuard::with_bits("2");
    let mut store = SettlementStore::new();
    let (left, right) = sibling_bucket_pair(&mut store, 33, 11);

    let merge = store.merge_proof(&left, &right).expect("merge proof");
    store
        .validate_merge_proof(&merge)
        .expect("merge proof validation");

    let next = BucketPolicy::new(
        store.bucket_policy().bucket_bits(),
        store.bucket_policy().min_bucket_count(),
        store.bucket_policy().max_target_leaf_count(),
        store.bucket_policy().compatibility_generation() + 1,
    )
    .expect("next bucket policy");
    let transition = store
        .policy_transition_proof(next)
        .expect("policy transition proof");
    store
        .validate_policy_transition_proof(&transition, next)
        .expect("policy transition validation");

    let nonadjacent = path(33, 11, 127);
    store
        .put_settlement_item(item(nonadjacent, 0x7f))
        .expect("seed nonadjacent path");
    let merge_err = store
        .merge_proof(&left, &nonadjacent)
        .expect_err("non-sibling pair must reject");
    assert!(matches!(merge_err, AdaptiveProofErr::MergeIneligible));

    let tampered_merge = MergeProof {
        merged_bucket_root: [0x5A; 32],
        ..merge
    };
    let merge_err = store
        .validate_merge_proof(&tampered_merge)
        .expect_err("wrong merged root must reject");
    assert!(matches!(merge_err, AdaptiveProofErr::WrongNewRoot));

    let stale = PolicyTransitionProof {
        prior_policy_id: [0x11; 32],
        ..transition
    };
    let stale_err = store
        .validate_policy_transition_proof(&stale, next)
        .expect_err("stale prior policy must reject");
    assert!(matches!(stale_err, AdaptiveProofErr::StalePolicyId));

    let drift = BucketPolicy::new(
        next.bucket_bits() + 1,
        next.min_bucket_count(),
        next.max_target_leaf_count(),
        next.compatibility_generation() + 1,
    )
    .expect("drifted bucket policy");
    let drift_err = store
        .validate_policy_transition_proof(&transition, drift)
        .expect_err("drifted next policy must reject");
    assert!(matches!(
        drift_err,
        AdaptiveProofErr::NextPolicyDrift | AdaptiveProofErr::WrongNewRoot
    ));
}
