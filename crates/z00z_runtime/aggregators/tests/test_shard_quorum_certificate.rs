#[path = "test_common.rs"]
mod test_common;
#[path = "test_recovery_common.rs"]
mod test_recovery_common;

use z00z_aggregators::{
    bind_publication_contract, membership_digest_for_voters, AggregatorId, BatchPlanner,
    BatchRoute, CommitSubject, JournalCandidate, RouteRangeRule, SecondaryState, ShardId,
    ShardQuorumCertificate, ShardRouteTable, ShardVote, ShardVoteKind, ShardVoteRole,
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

struct VoteFixture {
    subject: CommitSubject,
    primary: AggregatorId,
    secondaries: Vec<SecondaryState>,
}

#[test]
fn test_shard_qc_sorting() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = vote_fixture()?;
    let primary_vote = ShardVote::new_local(
        fixture.primary,
        ShardVoteRole::Primary,
        fixture.subject.shard_id,
        fixture.subject.term,
        fixture.subject.membership_digest,
        fixture.subject.digest(),
        ShardVoteKind::LocalCommit,
    );
    let secondary_vote = ShardVote::new_local(
        fixture.secondaries[0].aggregator_id,
        ShardVoteRole::Secondary,
        fixture.subject.shard_id,
        fixture.subject.term,
        fixture.subject.membership_digest,
        fixture.subject.digest(),
        ShardVoteKind::LocalCommit,
    );

    let certificate = ShardQuorumCertificate::new(
        &fixture.subject,
        fixture.primary,
        fixture
            .secondaries
            .iter()
            .map(|secondary| secondary.aggregator_id),
        &[secondary_vote.clone(), primary_vote.clone()],
    )
    .expect("certificate");
    let stable = ShardQuorumCertificate::new(
        &fixture.subject,
        fixture.primary,
        fixture
            .secondaries
            .iter()
            .map(|secondary| secondary.aggregator_id),
        &[primary_vote, secondary_vote],
    )
    .expect("stable certificate");

    assert_eq!(certificate.digest(), stable.digest());
    assert!(certificate.votes[0].voter_id < certificate.votes[1].voter_id);

    Ok(())
}

#[test]
fn test_shard_qc_member_rejects() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = vote_fixture()?;
    let subject_digest = fixture.subject.digest();
    let primary_vote = ShardVote::new_local(
        fixture.primary,
        ShardVoteRole::Primary,
        fixture.subject.shard_id,
        fixture.subject.term,
        fixture.subject.membership_digest,
        subject_digest,
        ShardVoteKind::LocalCommit,
    );

    let duplicate = ShardQuorumCertificate::new(
        &fixture.subject,
        fixture.primary,
        fixture
            .secondaries
            .iter()
            .map(|secondary| secondary.aggregator_id),
        &[primary_vote.clone(), primary_vote.clone()],
    )
    .expect_err("duplicate voters must reject");
    assert!(duplicate.detail.contains("duplicate voter ids"));

    let inactive_vote = ShardVote::new_local(
        AggregatorId::new(99),
        ShardVoteRole::Secondary,
        fixture.subject.shard_id,
        fixture.subject.term,
        fixture.subject.membership_digest,
        subject_digest,
        ShardVoteKind::LocalCommit,
    );
    let inactive = ShardQuorumCertificate::new(
        &fixture.subject,
        fixture.primary,
        fixture
            .secondaries
            .iter()
            .map(|secondary| secondary.aggregator_id),
        &[primary_vote.clone(), inactive_vote],
    )
    .expect_err("inactive voter must reject");
    assert!(inactive.detail.contains("inactive voter ids"));

    let wrong_role_vote = ShardVote::new_local(
        fixture.secondaries[0].aggregator_id,
        ShardVoteRole::Primary,
        fixture.subject.shard_id,
        fixture.subject.term,
        fixture.subject.membership_digest,
        subject_digest,
        ShardVoteKind::LocalCommit,
    );
    let wrong_role = ShardQuorumCertificate::new(
        &fixture.subject,
        fixture.primary,
        fixture
            .secondaries
            .iter()
            .map(|secondary| secondary.aggregator_id),
        &[primary_vote, wrong_role_vote],
    )
    .expect_err("wrong role must reject");
    assert!(wrong_role.detail.contains("wrong voter role"));

    Ok(())
}

#[test]
fn test_shard_qc_field_rejects() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = vote_fixture()?;
    let subject_digest = fixture.subject.digest();
    let primary_vote = ShardVote::new_local(
        fixture.primary,
        ShardVoteRole::Primary,
        fixture.subject.shard_id,
        fixture.subject.term,
        fixture.subject.membership_digest,
        subject_digest,
        ShardVoteKind::LocalCommit,
    );
    let secondary_vote = ShardVote::new_local(
        fixture.secondaries[0].aggregator_id,
        ShardVoteRole::Secondary,
        fixture.subject.shard_id,
        fixture.subject.term,
        fixture.subject.membership_digest,
        subject_digest,
        ShardVoteKind::LocalCommit,
    );

    let mixed_term = ShardVote::new_local(
        fixture.secondaries[0].aggregator_id,
        ShardVoteRole::Secondary,
        fixture.subject.shard_id,
        fixture.subject.term + 1,
        fixture.subject.membership_digest,
        subject_digest,
        ShardVoteKind::LocalCommit,
    );
    let err = ShardQuorumCertificate::new(
        &fixture.subject,
        fixture.primary,
        fixture
            .secondaries
            .iter()
            .map(|secondary| secondary.aggregator_id),
        &[primary_vote.clone(), mixed_term],
    )
    .expect_err("mixed terms must reject");
    assert!(err.detail.contains("mixed terms"));

    let mixed_membership = ShardVote::new_local(
        fixture.secondaries[0].aggregator_id,
        ShardVoteRole::Secondary,
        fixture.subject.shard_id,
        fixture.subject.term,
        membership_digest_for_voters(
            fixture.subject.route(),
            fixture.primary,
            [fixture.secondaries[1].aggregator_id],
        ),
        subject_digest,
        ShardVoteKind::LocalCommit,
    );
    let err = ShardQuorumCertificate::new(
        &fixture.subject,
        fixture.primary,
        fixture
            .secondaries
            .iter()
            .map(|secondary| secondary.aggregator_id),
        &[primary_vote.clone(), mixed_membership],
    )
    .expect_err("mixed membership digests must reject");
    assert!(err.detail.contains("mixed membership digests"));

    let mixed_subject = ShardVote::new_local(
        fixture.secondaries[0].aggregator_id,
        ShardVoteRole::Secondary,
        fixture.subject.shard_id,
        fixture.subject.term,
        fixture.subject.membership_digest,
        [0x99; 32],
        ShardVoteKind::LocalCommit,
    );
    let err = ShardQuorumCertificate::new(
        &fixture.subject,
        fixture.primary,
        fixture
            .secondaries
            .iter()
            .map(|secondary| secondary.aggregator_id),
        &[primary_vote.clone(), mixed_subject],
    )
    .expect_err("mixed subject digests must reject");
    assert!(err.detail.contains("mixed subject digests"));

    let certificate = ShardQuorumCertificate::new(
        &fixture.subject,
        fixture.primary,
        fixture
            .secondaries
            .iter()
            .map(|secondary| secondary.aggregator_id),
        &[primary_vote, secondary_vote.clone()],
    )
    .expect("certificate");
    assert_eq!(certificate.votes.len(), 2);

    let below_quorum = ShardQuorumCertificate::new(
        &fixture.subject,
        fixture.primary,
        fixture
            .secondaries
            .iter()
            .map(|secondary| secondary.aggregator_id),
        &[secondary_vote],
    )
    .expect_err("below quorum must reject");
    assert!(below_quorum.detail.contains("below quorum"));

    Ok(())
}

fn vote_fixture() -> Result<VoteFixture, Box<dyn std::error::Error>> {
    let route = BatchRoute {
        shard_id: ShardId::new(5),
        routing_generation: 12,
    };
    let primary = AggregatorId::new(21);
    let secondaries = vec![
        SecondaryState::ready(AggregatorId::new(22)),
        SecondaryState::ready(AggregatorId::new(23)),
    ];
    let batch = planner(route)
        .make_batch(batch_id("qc-fixture"), &[tx_item("qc")])
        .expect("planned batch");
    let recovery = route_bound_recovery_state(
        0x81,
        batch.batch_id,
        route,
        batch.planned.route_table_digest.into_bytes(),
    )?;
    let record = recovery_record("qc-fixture", route, primary, secondaries.clone(), recovery);
    let candidate = JournalCandidate::from_record(&record).expect("recovery candidate");
    let publication = publication_binding(
        &batch,
        SettlementStateRoot::settlement_v1([0x11; 32]),
        candidate.state_root,
    );
    let subject = CommitSubject::from_runtime(
        17,
        membership_digest_for_voters(
            route,
            primary,
            secondaries.iter().map(|secondary| secondary.aggregator_id),
        ),
        &batch,
        &candidate,
        &publication,
        publication.pub_in_digest(),
        None,
    )
    .expect("commit subject");
    Ok(VoteFixture {
        subject,
        primary,
        secondaries,
    })
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
