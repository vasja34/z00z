mod test_recovery_common;

use z00z_aggregators::{
    AggregatorId, BatchId, DistSim, FrameStage, JournalFrame, RecoveryBoundary, RecoveryIntent,
    ShardExecState, ShardExecTicket, ShardPlacement, StandbyState,
};

use self::test_recovery_common::{
    batch_id, bind_recovery_route, durable_commit_publication_case, live_recovery_state,
    placement_table, recovery_record, route, route_bound_recovery_state,
};

fn record_from_recovery(
    recovery: z00z_storage::settlement::SettlementRecoveryState,
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

#[test]
fn test_handles_partition_replay() -> Result<(), Box<dyn std::error::Error>> {
    let route = route(3, 9);
    let primary = AggregatorId::new(7);
    let standby = AggregatorId::new(8);
    let ready = StandbyState::ready(standby);
    let recovery =
        route_bound_recovery_state(0x71, batch_id("dist-sim-primary"), route, [0x31; 32])?;
    let record = recovery_record(
        "dist-sim-primary",
        route,
        primary,
        vec![ready],
        recovery.clone(),
    );
    let table = placement_table(route, primary, vec![ready], recovery.journal_lineage);

    let mut sim =
        DistSim::new(route, [primary, standby]).expect("dist sim must build for lawful members");
    sim.seed(primary, record.clone())
        .expect("primary seed must accept the live recovery record");
    sim.partition(standby)
        .expect("partition must accept a known standby");
    sim.enqueue_delayed(JournalFrame::new(primary, standby, 1, record.clone()), 1);

    let deferred = sim.step();
    assert_eq!(deferred.len(), 1);
    assert_eq!(deferred[0].stage, FrameStage::Deferred);
    assert!(deferred[0].detail.contains("partitioned"));
    assert!(sim
        .resume(standby, &table, &record, RecoveryIntent::TakeoverStandby)
        .is_err());

    sim.heal(standby).expect("heal must accept a known standby");
    let applied = sim.step();
    assert_eq!(applied.len(), 1);
    assert_eq!(applied[0].stage, FrameStage::Applied);
    sim.resume(standby, &table, &record, RecoveryIntent::TakeoverStandby)
        .expect("healed standby must resume after replicated recovery applies");

    sim.enqueue_replay(JournalFrame::new(primary, standby, 1, record.clone()), 0);
    let replay = sim.step();
    assert_eq!(replay.len(), 1);
    assert_eq!(replay[0].stage, FrameStage::ReplayIgnored);
    assert_eq!(
        sim.node(standby).expect("standby node").applied_count(),
        1,
        "replay must stay idempotent on the standby state"
    );

    Ok(())
}

#[test]
fn test_catchup_fails_closed() -> Result<(), Box<dyn std::error::Error>> {
    let case = durable_commit_publication_case()?;
    let later_recovery = case.later_recovery;
    let route_ctx = later_recovery.route.expect("route-bound later recovery");
    let route = route(route_ctx.shard_id() as u16, route_ctx.routing_generation());
    let primary = AggregatorId::new(27);
    let standby = AggregatorId::new(28);
    let ready = StandbyState::ready(standby);
    let latest_record = record_from_recovery(later_recovery.clone(), route, primary, vec![ready]);
    let table = placement_table(route, primary, vec![ready], later_recovery.journal_lineage);

    let stale = bind_recovery_route(
        live_recovery_state(0x61)?,
        BatchId::from_bytes(route_ctx.batch_id()),
        route,
        route_ctx.route_table_digest(),
    );
    let stale_record = record_from_recovery(stale, route, primary, vec![ready]);

    let mut sim =
        DistSim::new(route, [primary, standby]).expect("dist sim must build for lawful members");
    sim.seed(primary, latest_record.clone())
        .expect("primary seed must accept the latest recovery record");
    sim.enqueue_delayed(
        JournalFrame::new(primary, standby, 3, latest_record.clone()),
        2,
    );
    sim.enqueue_front(JournalFrame::new(primary, standby, 2, stale_record));

    let first = sim.step();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].stage, FrameStage::Applied);
    let stale_err = sim
        .sync_verdict(standby, &latest_record)
        .expect_err("stale standby state must fail before the later frame lands");
    assert!(stale_err.detail.contains("wrong lineage"));

    let dropped = sim.drop_next().expect("queued latest frame");
    assert_eq!(dropped.record.batch_id, latest_record.batch_id);
    assert!(sim
        .resume(
            standby,
            &table,
            &latest_record,
            RecoveryIntent::TakeoverStandby
        )
        .is_err());

    sim.enqueue(JournalFrame::new(
        primary,
        standby,
        3,
        latest_record.clone(),
    ));
    let second = sim.step();
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].stage, FrameStage::Applied);
    sim.sync_verdict(standby, &latest_record)
        .expect("latest replicated recovery must replace the stale replay");
    sim.resume(
        standby,
        &table,
        &latest_record,
        RecoveryIntent::TakeoverStandby,
    )
    .expect("caught-up standby must resume after the latest state transfer");

    Ok(())
}
