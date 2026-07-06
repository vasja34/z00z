#[path = "test_common.rs"]
mod test_common;
#[path = "test_recovery_common.rs"]
mod test_recovery_common;

use z00z_aggregators::{
    bind_publication_contract, membership_digest_for_voters, AggregatorId, BatchPlanner,
    BatchRoute, CommitSubject, RouteRangeRule, SecondaryReplayRequest, SecondaryState, ShardId,
    ShardPlacement, ShardPlacementTable, ShardRouteTable, ShardVote, ShardVoteKind, ShardVoteRole,
    VoteExchangeContext, WorkItem,
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

#[allow(dead_code)]
pub struct TransportFixture {
    pub subject: CommitSubject,
    pub items: Vec<WorkItem>,
    pub planner: BatchPlanner,
    pub placement: ShardPlacement,
    pub placement_table: ShardPlacementTable,
    pub record: z00z_aggregators::ShardRecoveryRecord,
    pub recovery: SettlementRecoveryState,
    pub publication: z00z_aggregators::PublicationBinding,
    pub theorem_digest: [u8; 32],
    pub primary: AggregatorId,
    pub ready_secondaries: [SecondaryState; 2],
}

pub fn transport_fixture() -> Result<TransportFixture, Box<dyn std::error::Error>> {
    let route = BatchRoute {
        shard_id: ShardId::new(6),
        routing_generation: 12,
    };
    let primary = AggregatorId::new(31);
    let ready_secondaries = [
        SecondaryState::ready(AggregatorId::new(32)),
        SecondaryState::ready(AggregatorId::new(33)),
    ];
    let planner = planner(route);
    let items = vec![tx_item("transport-adapter")];
    let batch = planner
        .make_batch(batch_id("transport-adapter"), &items)
        .expect("planned batch");
    let recovery = route_bound_recovery_state(
        0x91,
        batch.batch_id,
        route,
        batch.planned.route_table_digest.into_bytes(),
    )?;
    let placement = ShardPlacement::new(
        route,
        primary,
        vec![ready_secondaries[0], ready_secondaries[1]],
        recovery.journal_lineage,
    );
    let mut placement_table = ShardPlacementTable::default();
    placement_table.insert(placement.clone());
    let record = recovery_record(
        "transport-adapter",
        route,
        primary,
        placement.secondaries.clone(),
        recovery.clone(),
    );
    let candidate = z00z_aggregators::JournalCandidate::from_record(&record).expect("candidate");
    let publication = publication_binding(
        &batch,
        SettlementStateRoot::settlement_v1([0x11; 32]),
        candidate.state_root,
    );
    let theorem_digest = publication.pub_in_digest();
    let subject = CommitSubject::from_runtime(
        17,
        membership_digest_for_voters(
            route,
            primary,
            ready_secondaries
                .iter()
                .map(|secondary| secondary.aggregator_id),
        ),
        &batch,
        &candidate,
        &publication,
        theorem_digest,
        None,
    )
    .expect("commit subject");
    Ok(TransportFixture {
        subject,
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

pub fn vote_exchange_context<'a>(
    fixture: &'a TransportFixture,
    voter_id: AggregatorId,
    subject: &'a CommitSubject,
) -> VoteExchangeContext<'a> {
    vote_exchange_context_with_placement(fixture, &fixture.placement_table, voter_id, subject)
}

pub fn vote_exchange_context_with_placement<'a>(
    fixture: &'a TransportFixture,
    placement_table: &'a ShardPlacementTable,
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
            placement_table,
            recovery_record: &fixture.record,
            local_recovery: &fixture.recovery,
            publication_binding: &fixture.publication,
            theorem_or_settlement_digest: fixture.theorem_digest,
            da_availability_digest: None,
        },
    }
}

#[allow(dead_code)]
#[must_use]
pub fn local_vote(
    subject: &CommitSubject,
    voter_id: AggregatorId,
    voter_role: ShardVoteRole,
) -> ShardVote {
    ShardVote::new_local(
        voter_id,
        voter_role,
        subject.shard_id,
        subject.term,
        subject.membership_digest,
        subject.digest(),
        ShardVoteKind::LocalCommit,
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
