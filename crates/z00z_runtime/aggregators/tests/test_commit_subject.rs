#[path = "test_common.rs"]
mod test_common;
#[path = "test_recovery_common.rs"]
mod test_recovery_common;

use z00z_aggregators::{
    bind_publication_contract, AggregatorId, BatchPlanner, BatchRoute, CommitSubject,
    ConsensusAdapter, JournalCandidate, RouteRangeRule, SecondaryState, ShardId, ShardPlacement,
    ShardRouteTable,
};
use z00z_storage::{
    checkpoint::{
        derive_checkpoint_id, CheckpointDraft, CheckpointExecInputId, CheckpointVersion,
        CreatedEnt, SpentEnt,
    },
    settlement::{CheckRoot, SettlementStateRoot},
    snapshot::PrepSnapshotId,
};

use self::{
    test_common::{batch_id, tx_item, HASH_MAX, HASH_MIN},
    test_recovery_common::{recovery_record, route_bound_recovery_state},
};

struct SubjectFixture {
    batch: z00z_aggregators::OrderedBatch,
    candidate: JournalCandidate,
    membership_digest: [u8; 32],
}

#[test]
fn test_commit_subject_drifts() -> Result<(), Box<dyn std::error::Error>> {
    let route = BatchRoute {
        shard_id: ShardId::new(5),
        routing_generation: 12,
    };
    let base = subject_fixture("commit-subject", "seed-a", route)?;
    let stable = build_subject(
        17,
        &base.batch,
        &base.candidate,
        base.membership_digest,
        publication_binding(
            &base.batch,
            SettlementStateRoot::settlement_v1([0x11; 32]),
            base.candidate.state_root,
        ),
        base.candidate.state_root.into_bytes(),
        None,
    )
    .expect("stable subject");

    let base_digest = stable.digest();
    assert_eq!(base_digest, stable.digest());
    assert_eq!(stable.encode(), stable.encode());

    let routed_fixture = subject_fixture(
        "commit-subject",
        "seed-a",
        BatchRoute {
            shard_id: ShardId::new(6),
            routing_generation: 12,
        },
    )?;
    let routed = build_subject(
        17,
        &routed_fixture.batch,
        &routed_fixture.candidate,
        routed_fixture.membership_digest,
        publication_binding(
            &routed_fixture.batch,
            SettlementStateRoot::settlement_v1([0x11; 32]),
            routed_fixture.candidate.state_root,
        ),
        routed_fixture.candidate.state_root.into_bytes(),
        None,
    )
    .expect("routed subject");
    assert_ne!(base_digest, routed.digest());

    let generation_fixture = subject_fixture(
        "commit-subject",
        "seed-a",
        BatchRoute {
            shard_id: ShardId::new(5),
            routing_generation: 13,
        },
    )?;
    let generation_subject = build_subject(
        17,
        &generation_fixture.batch,
        &generation_fixture.candidate,
        generation_fixture.membership_digest,
        publication_binding(
            &generation_fixture.batch,
            SettlementStateRoot::settlement_v1([0x11; 32]),
            generation_fixture.candidate.state_root,
        ),
        generation_fixture.candidate.state_root.into_bytes(),
        None,
    )
    .expect("generation subject");
    assert_ne!(base_digest, generation_subject.digest());

    let drifted_root = SettlementStateRoot::settlement_v1([0x77; 32]);
    let mut root_candidate = base.candidate.clone();
    root_candidate.state_root = drifted_root;
    let root_subject = build_subject(
        17,
        &base.batch,
        &root_candidate,
        base.membership_digest,
        publication_binding(
            &base.batch,
            SettlementStateRoot::settlement_v1([0x11; 32]),
            drifted_root,
        ),
        drifted_root.into_bytes(),
        None,
    )
    .expect("root subject");
    assert_ne!(base_digest, root_subject.digest());

    let mut proof_candidate = base.candidate.clone();
    proof_candidate.proof_version = proof_candidate.proof_version.saturating_add(1);
    let proof_subject = build_subject(
        17,
        &base.batch,
        &proof_candidate,
        base.membership_digest,
        publication_binding(
            &base.batch,
            SettlementStateRoot::settlement_v1([0x11; 32]),
            proof_candidate.state_root,
        ),
        proof_candidate.state_root.into_bytes(),
        None,
    )
    .expect("proof subject");
    assert_ne!(base_digest, proof_subject.digest());

    let mut policy_candidate = base.candidate.clone();
    policy_candidate.bucket_policy_generation =
        policy_candidate.bucket_policy_generation.saturating_add(1);
    policy_candidate.bucket_policy_id = [0x55; 32];
    let policy_subject = build_subject(
        17,
        &base.batch,
        &policy_candidate,
        base.membership_digest,
        publication_binding(
            &base.batch,
            SettlementStateRoot::settlement_v1([0x11; 32]),
            policy_candidate.state_root,
        ),
        policy_candidate.state_root.into_bytes(),
        None,
    )
    .expect("policy subject");
    assert_ne!(base_digest, policy_subject.digest());

    let publication_subject = build_subject(
        17,
        &base.batch,
        &base.candidate,
        base.membership_digest,
        publication_binding(
            &base.batch,
            SettlementStateRoot::settlement_v1([0x44; 32]),
            base.candidate.state_root,
        ),
        base.candidate.state_root.into_bytes(),
        None,
    )
    .expect("publication subject");
    assert_ne!(base_digest, publication_subject.digest());

    Ok(())
}

fn subject_fixture(
    label: &str,
    seed: &str,
    route: BatchRoute,
) -> Result<SubjectFixture, Box<dyn std::error::Error>> {
    let primary = AggregatorId::new(21);
    let secondaries = vec![
        SecondaryState::ready(AggregatorId::new(22)),
        SecondaryState::ready(AggregatorId::new(23)),
    ];
    let batch = planner(route)
        .make_batch(batch_id(label), &[tx_item(seed)])
        .expect("planned batch");
    let recovery = route_bound_recovery_state(
        0x81,
        batch.batch_id,
        route,
        batch.planned.route_table_digest.into_bytes(),
    )?;
    let record = recovery_record(label, route, primary, secondaries.clone(), recovery.clone());
    let candidate = JournalCandidate::from_record(&record).expect("recovery candidate");
    let placement = ShardPlacement::new(route, primary, secondaries, recovery.journal_lineage);
    let adapter = ConsensusAdapter::from_placement(&placement).expect("consensus adapter");
    Ok(SubjectFixture {
        batch,
        candidate,
        membership_digest: adapter.membership_digest(),
    })
}

fn build_subject(
    term: u64,
    batch: &z00z_aggregators::OrderedBatch,
    candidate: &JournalCandidate,
    membership_digest: [u8; 32],
    publication_binding: z00z_aggregators::PublicationBinding,
    theorem_or_settlement_digest: [u8; 32],
    da_availability_digest: Option<[u8; 32]>,
) -> Result<CommitSubject, z00z_aggregators::RejectRecord> {
    CommitSubject::from_runtime(
        term,
        membership_digest,
        batch,
        candidate,
        &publication_binding,
        theorem_or_settlement_digest,
        da_availability_digest,
    )
}

fn publication_binding(
    batch: &z00z_aggregators::OrderedBatch,
    prev_root: SettlementStateRoot,
    new_root: SettlementStateRoot,
) -> z00z_aggregators::PublicationBinding {
    let draft = CheckpointDraft::new(
        CheckpointVersion::CURRENT,
        52,
        CheckRoot::new(prev_root.into_bytes()),
        CheckRoot::new(new_root.into_bytes()),
        vec![SpentEnt::new([0x31; 32]), SpentEnt::new([0x32; 32])],
        vec![CreatedEnt::new([0x41; 32], [0x51; 32])],
    );
    let proof = draft
        .attest_proof(
            PrepSnapshotId::new([0x61; 32]),
            CheckpointExecInputId::new([0x71; 32]),
        )
        .expect("checkpoint proof");
    let artifact = draft.finalize(proof).expect("checkpoint artifact");
    let checkpoint_id = derive_checkpoint_id(&artifact).expect("checkpoint id");
    bind_publication_contract(
        batch.batch_id,
        checkpoint_id,
        batch.planned.route_table_digest.into_bytes(),
        &artifact.pub_in(),
    )
}

fn planner(route: BatchRoute) -> BatchPlanner {
    BatchPlanner::new(ShardRouteTable {
        routing_generation: route.routing_generation,
        shard_set: vec![route.shard_id],
        rules: vec![RouteRangeRule::new(HASH_MIN, HASH_MAX, route.shard_id)],
        previous_generation_digest: (route.routing_generation > 0)
            .then_some(ShardRouteTable::default().digest()),
        activation_checkpoint: 11,
    })
}
