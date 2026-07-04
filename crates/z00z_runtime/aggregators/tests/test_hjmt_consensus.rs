#[path = "test_common.rs"]
mod test_common;
mod test_recovery_common;

use tempfile::tempdir;
use z00z_aggregators::{
    bind_publication_contract, AggregatorId, BatchId, BatchPlanner, BatchRoute, CommitSubject,
    ConsensusAdapter, JournalCandidate, MembershipChange, RecoveryBoundary, RouteRangeRule,
    SecondaryState, ShardExecState, ShardExecTicket, ShardPlacement, ShardRouteTable, ShardVote,
    ShardVoteKind, ShardVoteRole,
};
use z00z_storage::{
    checkpoint::{
        derive_checkpoint_id, CheckpointDraft, CheckpointExecInputId, CheckpointVersion,
        CreatedEnt, SpentEnt,
    },
    settlement::{CheckRoot, SettlementRecoveryState, SettlementStateRoot},
    snapshot::PrepSnapshotId,
};

use self::test_common::{tx_item, HASH_MAX, HASH_MIN};
use self::test_recovery_common::{
    batch_id, bind_recovery_route, live_recovery_state, recovery_record, route,
    route_bound_recovery_state,
};

#[path = "test_hjmt_topology_support.rs"]
mod hjmt_topology_support;

use hjmt_topology_support::{
    bind_previous_generation, canonical_five_by_seven, load_cfg, owner_join_six_by_seven,
    placement_row, read_route_table, set_activation_checkpoint, write_home,
};

fn record_from_recovery(
    recovery: SettlementRecoveryState,
    route: z00z_aggregators::BatchRoute,
    primary: AggregatorId,
    secondary: Vec<SecondaryState>,
) -> z00z_aggregators::ShardRecoveryRecord {
    let batch_id = BatchId::from_bytes(recovery.route.expect("route-bound recovery").batch_id());
    let placement = ShardPlacement::new(route, primary, secondary, recovery.journal_lineage);
    let ticket = ShardExecTicket {
        batch_id,
        placement: placement.view(),
        state: ShardExecState::Routed,
    };
    let boundary = RecoveryBoundary;
    let publication = boundary.mark_handed_off(batch_id);
    boundary
        .capture(&ticket, &publication, recovery)
        .expect("recovery record")
}

#[test]
fn test_quorum_freezes_term_roots() -> Result<(), Box<dyn std::error::Error>> {
    let route = route(5, 12);
    let primary = AggregatorId::new(21);
    let secondary_a = SecondaryState::ready(AggregatorId::new(22));
    let secondary_b = SecondaryState::ready(AggregatorId::new(23));
    let batch = planner(route)
        .make_batch(
            batch_id("consensus-majority"),
            &[tx_item("consensus-majority")],
        )
        .expect("planned batch");
    let recovery = route_bound_recovery_state(
        0x81,
        batch.batch_id,
        route,
        batch.planned.route_table_digest.into_bytes(),
    )?;
    let latest_record = recovery_record(
        "consensus-majority",
        route,
        primary,
        vec![secondary_a, secondary_b],
        recovery.clone(),
    );
    let latest_candidate =
        JournalCandidate::from_record(&latest_record).expect("latest recovery record");
    let latest_subject = subject_for_candidate(
        7,
        &batch,
        &latest_candidate,
        primary,
        &[secondary_a, secondary_b],
        SettlementStateRoot::settlement_v1([0x11; 32]),
    )
    .expect("latest subject");

    let conflicting = bind_recovery_route(
        live_recovery_state(0x91)?,
        batch.batch_id,
        route,
        batch.planned.route_table_digest.into_bytes(),
    );
    let conflicting_record =
        record_from_recovery(conflicting, route, primary, vec![secondary_a, secondary_b]);
    let conflicting_candidate =
        JournalCandidate::from_record(&conflicting_record).expect("conflicting recovery record");
    let conflicting_subject = subject_for_candidate(
        7,
        &batch,
        &conflicting_candidate,
        primary,
        &[secondary_a, secondary_b],
        SettlementStateRoot::settlement_v1([0x11; 32]),
    )
    .expect("conflicting subject");

    let mut adapter = ConsensusAdapter::new(
        route,
        primary,
        [secondary_a.aggregator_id, secondary_b.aggregator_id],
    )
    .expect("consensus adapter must build");
    let commit_votes = votes_for_subject(
        &latest_subject,
        &[
            (primary, ShardVoteRole::Primary),
            (secondary_a.aggregator_id, ShardVoteRole::Secondary),
        ],
    );
    let commit = adapter
        .commit(&latest_subject, &commit_votes)
        .expect("majority quorum must commit");
    assert_eq!(commit.term, 7);
    assert_eq!(commit.route, route);
    assert_eq!(commit.certificate.subject_digest, latest_subject.digest());

    let split_votes = votes_for_subject(
        &conflicting_subject,
        &[
            (primary, ShardVoteRole::Primary),
            (secondary_b.aggregator_id, ShardVoteRole::Secondary),
        ],
    );
    let split_err = adapter
        .commit(&conflicting_subject, &split_votes)
        .expect_err("divergent root must freeze the same quorum term");
    assert!(split_err.detail.contains("split-brain"));

    let frozen_votes = votes_for_subject(
        &latest_subject,
        &[
            (primary, ShardVoteRole::Primary),
            (secondary_a.aggregator_id, ShardVoteRole::Secondary),
        ],
    );
    let frozen_err = adapter
        .commit(&latest_subject, &frozen_votes)
        .expect_err("same term must stay frozen after divergence");
    assert!(frozen_err.detail.contains("frozen"));

    let latest_subject_next_term = subject_for_candidate(
        8,
        &batch,
        &latest_candidate,
        primary,
        &[secondary_a, secondary_b],
        SettlementStateRoot::settlement_v1([0x11; 32]),
    )
    .expect("next-term subject");
    let next_term_votes = votes_for_subject(
        &latest_subject_next_term,
        &[
            (primary, ShardVoteRole::Primary),
            (secondary_b.aggregator_id, ShardVoteRole::Secondary),
        ],
    );
    let commit = adapter
        .commit(&latest_subject_next_term, &next_term_votes)
        .expect("new term must clear the frozen quorum");
    assert_eq!(commit.term, 8);

    Ok(())
}

#[test]
fn test_membership_keeps_generation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let old_home = temp.path().join("old_5a7s");
    let new_home = temp.path().join("new_6a7s_owner");
    write_home(&old_home, 1, &canonical_five_by_seven(9400));
    write_home(&new_home, 2, &owner_join_six_by_seven(9500));
    set_activation_checkpoint(&old_home, 11);
    bind_previous_generation(&new_home, &read_route_table(&old_home));
    set_activation_checkpoint(&new_home, 42);

    let old_cfg = load_cfg(&old_home);
    let new_cfg = load_cfg(&new_home);
    let old_row = placement_row(&old_cfg, 0, 1);
    let new_row = placement_row(&new_cfg, 0, 2);

    let mut adapter = ConsensusAdapter::from_placement(&old_row).expect("old adapter");
    adapter
        .bind_placement(&old_cfg.placement_table().expect("old placement table"))
        .expect("old placement must bind");

    adapter
        .apply_change(MembershipChange::Decommission, AggregatorId::new(2), 2)
        .expect("decommission must be generation-bound and lawful");
    adapter
        .apply_change(MembershipChange::Join, AggregatorId::new(5), 2)
        .expect("join must be generation-bound and lawful");
    adapter
        .bind_placement(&new_cfg.placement_table().expect("new placement table"))
        .expect("new placement must bind after membership updates");
    assert_eq!(
        adapter.route().routing_generation,
        new_row.route.routing_generation
    );

    let stale_rejoin = adapter
        .apply_change(MembershipChange::Rejoin, AggregatorId::new(2), 2)
        .expect_err("retired member must not rejoin without a newer routing_generation");
    assert!(stale_rejoin.detail.contains("generation bound"));

    adapter
        .apply_change(MembershipChange::Rejoin, AggregatorId::new(2), 3)
        .expect("retired member must rejoin on a newer routing_generation");
    adapter
        .apply_change(MembershipChange::Leave, AggregatorId::new(2), 3)
        .expect("active member must be able to leave");

    let stale_leave = adapter
        .apply_change(MembershipChange::Leave, AggregatorId::new(99), 3)
        .expect_err("non-member leave must reject");
    assert!(stale_leave.detail.contains("stale member"));

    Ok(())
}

fn subject_for_candidate(
    term: u64,
    batch: &z00z_aggregators::OrderedBatch,
    candidate: &JournalCandidate,
    primary: AggregatorId,
    secondaries: &[SecondaryState],
    prev_root: SettlementStateRoot,
) -> Result<CommitSubject, z00z_aggregators::RejectRecord> {
    let publication = publication_binding(batch, prev_root, candidate.state_root);
    CommitSubject::from_runtime(
        term,
        z00z_aggregators::membership_digest_for_voters(
            batch.planned.route,
            primary,
            secondaries.iter().map(|secondary| secondary.aggregator_id),
        ),
        batch,
        candidate,
        &publication,
        publication.pub_in_digest(),
        None,
    )
}

fn votes_for_subject(
    subject: &CommitSubject,
    voters: &[(AggregatorId, ShardVoteRole)],
) -> Vec<ShardVote> {
    let subject_digest = subject.digest();
    voters
        .iter()
        .map(|(voter_id, voter_role)| {
            ShardVote::new_local(
                *voter_id,
                *voter_role,
                subject.shard_id,
                subject.term,
                subject.membership_digest,
                subject_digest,
                ShardVoteKind::LocalCommit,
            )
        })
        .collect()
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
