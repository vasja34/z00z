use z00z_storage::fixture_support::settlement_corpus::{
    next_policy, sibling_bucket_pair, split_ready_paths, HjmtEnvGuard,
};
use z00z_storage::settlement::{
    AdaptiveProofErr, BucketOccupancyEvidence, MergeProof, OccupancyClass, OccupancyScope,
    PolicyTransitionProof, SettlementStore, SplitProof,
};
use z00z_utils::codec::{Codec, JsonCodec};

fn proof_json<T: serde::Serialize>(value: &T) -> String {
    String::from_utf8(JsonCodec.serialize(value).expect("json bytes")).expect("utf8")
}

fn assert_hidden(surface: &str) {
    for needle in [
        "leaf_count",
        "exact_count",
        "raw_delta",
        "timing",
        "timestamp",
    ] {
        assert!(
            !surface.contains(needle),
            "proof-visible payload must not contain {needle}"
        );
    }
}

#[test]
fn test_privacy_hides_exact_metrics() {
    let _guard = HjmtEnvGuard::with_bits("1");
    let mut store = SettlementStore::new();
    let paths = split_ready_paths(&mut store, 41, 9);
    let target = paths[0];

    let metric = store
        .bucket_occupancy_metric(&target)
        .expect("occupancy metric");
    let bucket = store.adaptive_bucket(&target).expect("adaptive bucket");
    let split = store.split_proof(&target).expect("split proof");
    let transition = store
        .policy_transition_proof(next_policy(&store))
        .expect("transition proof");

    assert!(proof_json(&metric).contains("exact_count"));
    assert_hidden(&proof_json(&bucket));
    assert_hidden(&proof_json(&split));
    assert_hidden(&proof_json(&transition));
    assert_eq!(split.occupancy_evidence.class, OccupancyClass::SplitReady);
    assert_eq!(transition.occupancy_evidence.scope, OccupancyScope::Set);
}

#[test]
fn test_privacy_rejects_tampered_bind() {
    let _guard = HjmtEnvGuard::with_bits("1");
    let mut store = SettlementStore::new();
    let paths = split_ready_paths(&mut store, 41, 9);
    let target = paths[0];

    let split = store.split_proof(&target).expect("split proof");
    let split_err = store
        .validate_split_proof(&SplitProof {
            occupancy_evidence: BucketOccupancyEvidence::new(
                split.occupancy_evidence.scope,
                split.occupancy_evidence.class,
                [0xAA; 32],
            ),
            ..split
        })
        .expect_err("tampered split evidence must reject");
    assert!(matches!(split_err, AdaptiveProofErr::OccupancyDrift));

    let policy = next_policy(&store);
    let transition = store
        .policy_transition_proof(policy)
        .expect("transition proof");
    let transition_err = store
        .validate_policy_transition_proof(
            &PolicyTransitionProof {
                occupancy_evidence: BucketOccupancyEvidence::new(
                    OccupancyScope::Set,
                    OccupancyClass::SetCommit,
                    [0xCC; 32],
                ),
                ..transition
            },
            policy,
        )
        .expect_err("tampered transition evidence must reject");
    assert!(matches!(transition_err, AdaptiveProofErr::OccupancyDrift));
}

#[test]
fn test_privacy_rejects_tampered_pair() {
    let _guard = HjmtEnvGuard::with_bits("2");
    let mut store = SettlementStore::new();
    let (left, right) = sibling_bucket_pair(&mut store, 33, 11);
    let merge = store.merge_proof(&left, &right).expect("merge proof");

    let err = store
        .validate_merge_proof(&MergeProof {
            pair_evidence: BucketOccupancyEvidence::new(
                OccupancyScope::Pair,
                merge.pair_evidence.class,
                [0xBB; 32],
            ),
            ..merge
        })
        .expect_err("tampered merge evidence must reject");
    assert!(matches!(err, AdaptiveProofErr::OccupancyDrift));
}
