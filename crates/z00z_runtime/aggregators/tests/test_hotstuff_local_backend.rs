#[path = "test_common.rs"]
mod test_common;
#[path = "test_recovery_common.rs"]
mod test_recovery_common;

use z00z_aggregators::{
    bind_publication_contract, membership_digest_for_voters, validator_decision_snapshot,
    AggregatorId, BatchPlanner, BatchRoute, CommitSubject, ConsensusValidatorDecision,
    HotstuffLeaderConflict, HotstuffLocal, HotstuffProposal, InMemoryVoteTransport,
    JournalCandidate, OrderedBatch, QuorumRule, ReplayVerifiedVoteService, RouteRangeRule,
    SecondaryReplayRequest, SecondaryState, ShardId, ShardPlacement, ShardPlacementTable,
    ShardQuorumCertificate, ShardRouteTable, ShardVote, ShardVoteKind, ShardVoteRole,
    VoteExchangeContext, VoteExchangeOutcome, VoteExchangeResult, VoteTransport,
    VoteTransportEnvelope, WorkItem,
};
use z00z_storage::{
    checkpoint::{
        derive_checkpoint_id, CheckpointDraft, CheckpointExecInputId, CheckpointVersion,
        CreatedEnt, SpentEnt,
    },
    settlement::{CheckRoot, SettlementRecoveryState, SettlementStateRoot},
    snapshot::PrepSnapshotId,
};

use self::{
    test_common::{batch_id, tx_item, HASH_MAX, HASH_MIN},
    test_recovery_common::{recovery_record, route_bound_recovery_state},
};

struct HotstuffFixture {
    route: BatchRoute,
    batch: OrderedBatch,
    candidate: JournalCandidate,
    items: Vec<WorkItem>,
    planner: BatchPlanner,
    placement: ShardPlacement,
    placement_table: ShardPlacementTable,
    record: z00z_aggregators::ShardRecoveryRecord,
    recovery: SettlementRecoveryState,
    publication: z00z_aggregators::PublicationBinding,
    theorem_digest: [u8; 32],
    primary: AggregatorId,
    ready_secondaries: Vec<SecondaryState>,
}

#[test]
fn test_leader_rounds() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = hotstuff_fixture(7)?;
    let backend = HotstuffLocal::from_placement(&fixture.placement).map_err(box_reject_record)?;

    assert_eq!(backend.view(), 0);
    assert_eq!(backend.leader(0), fixture.primary);
    assert_eq!(backend.leader(1), AggregatorId::new(41));
    assert_eq!(backend.leader(2), AggregatorId::new(42));
    assert_eq!(backend.leader(6), AggregatorId::new(46));
    assert_eq!(backend.leader(7), fixture.primary);

    Ok(())
}

#[test]
fn test_timeout_change() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = hotstuff_fixture(7)?;
    let mut backend =
        HotstuffLocal::from_placement(&fixture.placement).map_err(box_reject_record)?;

    let proposal0 = backend
        .propose(subject_for_view(0, &fixture).map_err(box_reject_record)?)
        .map_err(box_reject_record)?;
    assert_eq!(proposal0.leader_id, fixture.primary);

    let timeout = backend
        .timeout(AggregatorId::new(41), "leader offline")
        .map_err(box_reject_record)?;
    let change = backend.advance_view(&timeout).map_err(box_reject_record)?;
    assert_eq!(change.from_view, 0);
    assert_eq!(change.to_view, 1);
    assert_eq!(change.new_leader_id, AggregatorId::new(41));

    let proposal1 = backend
        .propose(subject_for_view(1, &fixture).map_err(box_reject_record)?)
        .map_err(box_reject_record)?;
    let votes = prepare_votes(&fixture, &proposal1.subject, 4)?;
    let commit = backend
        .commit(&proposal1, &votes)
        .map_err(box_reject_record)?;

    assert_eq!(
        commit.backend_qc.view_change_digest,
        Some(change.evidence_digest)
    );
    assert_eq!(
        commit.backend_qc.certificate.quorum_rule,
        QuorumRule::BftTwoFPlusOne
    );
    assert_eq!(commit.commit.subject, proposal1.subject);

    Ok(())
}

#[test]
fn test_leader_conflict() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = hotstuff_fixture(7)?;
    let mut backend =
        HotstuffLocal::from_placement(&fixture.placement).map_err(box_reject_record)?;

    let first_subject = subject_for_view(0, &fixture).map_err(box_reject_record)?;
    let first = backend
        .propose(first_subject.clone())
        .map_err(box_reject_record)?;
    let mut second_subject = first_subject;
    second_subject.new_state_root = SettlementStateRoot::settlement_v1([0x77; 32]);

    let err = backend
        .propose(second_subject.clone())
        .expect_err("conflicting leader proposal must reject");
    assert!(err.detail.contains("leader conflict"));

    let second = HotstuffProposal::new(0, fixture.primary, second_subject, None);
    let evidence = HotstuffLeaderConflict::new(&first, &second).map_err(box_reject_record)?;

    assert_eq!(evidence.view, 0);
    assert_eq!(evidence.leader_id, fixture.primary);
    assert_ne!(
        evidence.first_subject_digest,
        evidence.second_subject_digest
    );

    Ok(())
}

#[test]
fn test_validator_bind() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = hotstuff_fixture(7)?;
    let mut backend =
        HotstuffLocal::from_placement(&fixture.placement).map_err(box_reject_record)?;

    let proposal = backend
        .propose(subject_for_view(0, &fixture).map_err(box_reject_record)?)
        .map_err(box_reject_record)?;
    let votes = prepare_votes(&fixture, &proposal.subject, 4)?;
    let commit = backend
        .commit(&proposal, &votes)
        .map_err(box_reject_record)?;

    let missing = validator_decision_snapshot(
        "accepted",
        None,
        proposal.subject.batch_id,
        &proposal.subject,
        &commit.commit.certificate,
        proposal.subject.theorem_or_settlement_digest,
        None,
        None,
    );
    let err = backend
        .bind_validator(&commit, &missing)
        .expect_err("backend QC without validator binding must reject");
    assert!(err.detail.contains("publication binding"));

    let wrong = validator_decision_snapshot(
        "accepted",
        None,
        proposal.subject.batch_id,
        &proposal.subject,
        &commit.commit.certificate,
        [0x66; 32],
        Some(fixture.publication.checkpoint_id()),
        Some(&fixture.publication),
    );
    let err = backend
        .bind_validator(&commit, &wrong)
        .expect_err("wrong theorem binding must reject");
    assert!(err.detail.contains("theorem"));

    let valid = validator_decision_snapshot(
        "accepted",
        None,
        proposal.subject.batch_id,
        &proposal.subject,
        &commit.commit.certificate,
        proposal.subject.theorem_or_settlement_digest,
        Some(fixture.publication.checkpoint_id()),
        Some(&fixture.publication),
    );
    backend
        .bind_validator(&commit, &valid)
        .map_err(box_reject_record)?;

    Ok(())
}

#[test]
fn test_subject_guard() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = hotstuff_fixture(7)?;
    let mut backend =
        HotstuffLocal::from_placement(&fixture.placement).map_err(box_reject_record)?;

    let err = backend
        .propose(subject_for_view(1, &fixture).map_err(box_reject_record)?)
        .expect_err("view jump without timeout must reject");
    assert!(err.detail.contains("wrong view"));

    let mut wrong_membership = subject_for_view(0, &fixture).map_err(box_reject_record)?;
    wrong_membership.membership_digest = [0xAA; 32];
    let err = backend
        .propose(wrong_membership)
        .expect_err("membership drift must reject");
    assert!(err.detail.contains("membership drift"));

    let mut wrong_route = subject_for_view(0, &fixture).map_err(box_reject_record)?;
    wrong_route.routing_generation += 1;
    let err = backend
        .propose(wrong_route)
        .expect_err("route drift must reject");
    assert!(err.detail.contains("wrong generation"));

    Ok(())
}

fn hotstuff_fixture(member_count: usize) -> Result<HotstuffFixture, Box<dyn std::error::Error>> {
    let route = BatchRoute {
        shard_id: ShardId::new(6),
        routing_generation: 21,
    };
    let primary = AggregatorId::new(40);
    let planner = planner(route);
    let items = vec![tx_item("hotstuff-local")];
    let batch = planner
        .make_batch(batch_id("hotstuff-local"), &items)
        .expect("planned batch");
    let recovery = route_bound_recovery_state(
        0xA1,
        batch.batch_id,
        route,
        batch.planned.route_table_digest.into_bytes(),
    )?;
    let ready_secondaries = (1..member_count)
        .map(|index| SecondaryState::ready(AggregatorId::new(40 + index as u16)))
        .collect::<Vec<_>>();
    let placement = ShardPlacement::new(
        route,
        primary,
        ready_secondaries.clone(),
        recovery.journal_lineage,
    );
    let mut placement_table = ShardPlacementTable::default();
    placement_table.insert(placement.clone());
    let record = recovery_record(
        "hotstuff-local",
        route,
        placement.primary_id,
        placement.secondaries.clone(),
        recovery.clone(),
    );
    let candidate = JournalCandidate::from_record(&record).expect("recovery candidate");
    let publication = publication_binding(
        &batch,
        SettlementStateRoot::settlement_v1([0x51; 32]),
        candidate.state_root,
    );
    let theorem_digest = publication.pub_in_digest();

    Ok(HotstuffFixture {
        route,
        batch,
        candidate,
        items,
        planner,
        placement,
        placement_table,
        record,
        recovery,
        publication,
        theorem_digest,
        primary,
        ready_secondaries,
    })
}

fn subject_for_view(
    view: u64,
    fixture: &HotstuffFixture,
) -> Result<CommitSubject, z00z_aggregators::RejectRecord> {
    CommitSubject::from_runtime(
        view,
        membership_digest_for_voters(
            fixture.route,
            fixture.primary,
            fixture
                .ready_secondaries
                .iter()
                .map(|secondary| secondary.aggregator_id),
        ),
        &fixture.batch,
        &fixture.candidate,
        &fixture.publication,
        fixture.theorem_digest,
        None,
    )
}

fn prepare_votes(
    fixture: &HotstuffFixture,
    subject: &CommitSubject,
    secondary_count: usize,
) -> Result<Vec<ShardVote>, Box<dyn std::error::Error>> {
    let secondary_ids = fixture
        .ready_secondaries
        .iter()
        .take(secondary_count)
        .map(|secondary| secondary.aggregator_id)
        .collect::<Vec<_>>();
    let mut transport = InMemoryVoteTransport::default();
    for secondary_id in &secondary_ids {
        transport.enqueue(VoteTransportEnvelope::available(
            fixture.primary,
            *secondary_id,
            subject.clone(),
            ShardVoteKind::Prepare,
        ));
    }

    let mut service = ReplayVerifiedVoteService::local();
    let mut votes = vec![primary_vote(subject, fixture.primary)];
    for envelope in transport.step() {
        let context = vote_context(fixture, envelope.to_id, subject);
        let result = service.process_envelope(&envelope, context);
        match result {
            VoteExchangeResult {
                outcome: VoteExchangeOutcome::Vote(vote),
                ..
            } => votes.push(vote),
            other => return Err(format!("expected vote, got {other:?}").into()),
        }
    }

    Ok(votes)
}

fn vote_context<'a>(
    fixture: &'a HotstuffFixture,
    voter_id: AggregatorId,
    subject: &'a CommitSubject,
) -> VoteExchangeContext<'a> {
    VoteExchangeContext {
        voter_role: ShardVoteRole::Secondary,
        replay_request: SecondaryReplayRequest {
            voter_id,
            term: subject.term,
            items: &fixture.items,
            planner: &fixture.planner,
            placement_table: &fixture.placement_table,
            recovery_record: &fixture.record,
            local_recovery: &fixture.recovery,
            publication_binding: &fixture.publication,
            theorem_or_settlement_digest: fixture.theorem_digest,
            da_availability_digest: None,
        },
    }
}

fn primary_vote(subject: &CommitSubject, voter_id: AggregatorId) -> ShardVote {
    ShardVote::new_local(
        voter_id,
        ShardVoteRole::Primary,
        subject.shard_id,
        subject.term,
        subject.membership_digest,
        subject.digest(),
        ShardVoteKind::Prepare,
    )
}

fn publication_binding(
    batch: &OrderedBatch,
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

fn box_reject_record(err: z00z_aggregators::RejectRecord) -> Box<dyn std::error::Error> {
    err.detail.into()
}

#[allow(dead_code)]
fn _assert_validator_type(_: &ConsensusValidatorDecision, _: &ShardQuorumCertificate) {}
