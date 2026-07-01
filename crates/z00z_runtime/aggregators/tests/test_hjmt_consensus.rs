mod test_recovery_common;

use tempfile::tempdir;
use z00z_aggregators::{
    AggregatorId, BatchId, ConsensusAdapter, JournalCandidate, MembershipChange, RecoveryBoundary,
    ShardExecState, ShardExecTicket, ShardPlacement, StandbyState,
};
use z00z_storage::settlement::SettlementRecoveryState;

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
    standby: Vec<StandbyState>,
) -> z00z_aggregators::ShardRecoveryRecord {
    let batch_id = BatchId::from_bytes(recovery.route.expect("route-bound recovery").batch_id());
    let placement = ShardPlacement::new(route, primary, standby, recovery.journal_lineage);
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

fn member_ids(row: &z00z_aggregators::ShardPlacement) -> Vec<AggregatorId> {
    let mut members = vec![row.primary_id];
    members.extend(row.standby.iter().map(|standby| standby.aggregator_id));
    members
}

#[test]
fn test_quorum_freezes_term_roots() -> Result<(), Box<dyn std::error::Error>> {
    let route = route(5, 12);
    let primary = AggregatorId::new(21);
    let standby_a = StandbyState::ready(AggregatorId::new(22));
    let standby_b = StandbyState::ready(AggregatorId::new(23));
    let recovery =
        route_bound_recovery_state(0x81, batch_id("consensus-majority"), route, [0x41; 32])?;
    let latest_record = recovery_record(
        "consensus-majority",
        route,
        primary,
        vec![standby_a, standby_b],
        recovery.clone(),
    );
    let latest_candidate =
        JournalCandidate::from_record(&latest_record).expect("latest recovery record");

    let conflicting = bind_recovery_route(
        live_recovery_state(0x91)?,
        BatchId::from_bytes(recovery.route.expect("route-bound recovery").batch_id()),
        route,
        [0x41; 32],
    );
    let conflicting_record =
        record_from_recovery(conflicting, route, primary, vec![standby_a, standby_b]);
    let conflicting_candidate =
        JournalCandidate::from_record(&conflicting_record).expect("conflicting recovery record");

    let mut adapter = ConsensusAdapter::new(
        route,
        [primary, standby_a.aggregator_id, standby_b.aggregator_id],
    )
    .expect("consensus adapter must build");
    let commit = adapter
        .commit(7, &latest_candidate, &[primary, standby_a.aggregator_id])
        .expect("majority quorum must commit");
    assert_eq!(commit.term, 7);
    assert_eq!(commit.route, route);

    let split_err = adapter
        .commit(
            7,
            &conflicting_candidate,
            &[primary, standby_b.aggregator_id],
        )
        .expect_err("divergent root must freeze the same quorum term");
    assert!(split_err.detail.contains("split-brain"));

    let frozen_err = adapter
        .commit(7, &latest_candidate, &[primary, standby_a.aggregator_id])
        .expect_err("same term must stay frozen after divergence");
    assert!(frozen_err.detail.contains("frozen"));

    let commit = adapter
        .commit(8, &latest_candidate, &[primary, standby_b.aggregator_id])
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

    let mut adapter =
        ConsensusAdapter::new(old_row.route, member_ids(&old_row)).expect("old adapter");
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
