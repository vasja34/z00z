mod test_recovery_common;

use tempfile::tempdir;
use z00z_aggregators::{
    AggregatorId, RecoveryBoundary, RecoveryIntent, RejectClass, SecondaryState,
};
use z00z_storage::settlement::{SettlementRecoveryState, SettlementRouteCtx, SettlementStateRoot};

use self::test_recovery_common::{
    batch_id, placement_table, recovery_record, route, route_bound_recovery_state,
};

#[path = "test_hjmt_topology_support.rs"]
mod hjmt_topology_support;

use hjmt_topology_support::{
    bind_previous_generation, load_cfg, placement_row, read_route_table, set_activation_checkpoint,
    staged_three_by_seven, staged_two_by_seven, write_home,
};

const FOV_T_001: &str = "FOV-T-001";
const FOV_T_002: &str = "FOV-T-002";
const ROUTE_MIGRATION_FIXTURE: &str = "Route migration fixture";

struct RejectCase {
    id: &'static str,
    kind: &'static str,
    requester: AggregatorId,
    intent: RecoveryIntent,
    want_class: RejectClass,
    want_detail: &'static str,
    table: z00z_aggregators::ShardPlacementTable,
    current: z00z_storage::settlement::SettlementRecoveryState,
}

#[test]
fn test_failover_reject_matrix() -> Result<(), Box<dyn std::error::Error>> {
    let live_route = route(5, 12);
    let recovery = route_bound_recovery_state(0x81, batch_id(FOV_T_001), live_route, [0x41; 32])?;
    let primary = AggregatorId::new(21);
    let secondary = AggregatorId::new(22);
    let ready = SecondaryState::ready(secondary);
    let record = recovery_record(
        FOV_T_001,
        live_route,
        primary,
        vec![ready],
        recovery.clone(),
    );
    let wrong_generation_table = placement_table(
        route(5, live_route.routing_generation + 1),
        primary,
        vec![ready],
        recovery.journal_lineage,
    );
    let pending_table = placement_table(
        live_route,
        primary,
        vec![SecondaryState::pending(secondary)],
        recovery.journal_lineage,
    );

    let mut wrong_lineage = recovery.clone();
    wrong_lineage.journal_lineage[0] ^= 0xff;
    let mut wrong_route_digest = recovery.clone();
    wrong_route_digest.route = Some(SettlementRouteCtx::new(
        batch_id(FOV_T_001).into_bytes(),
        live_route.shard_id.as_u32(),
        live_route.routing_generation,
        [0x99; 32],
    ));
    let mut wrong_shard = recovery.clone();
    wrong_shard.route = Some(SettlementRouteCtx::new(
        batch_id(FOV_T_001).into_bytes(),
        live_route.shard_id.as_u32() + 1,
        live_route.routing_generation,
        [0x41; 32],
    ));

    let stale_root = z00z_storage::settlement::SettlementRecoveryState::new(
        recovery.version,
        SettlementStateRoot::settlement_v1([0xAB; 32]),
        recovery.root_generation,
        recovery.proof_version,
        recovery.bucket_policy_generation,
        recovery.bucket_policy_id,
        recovery.journal_lineage,
    )
    .with_route(recovery.route.expect("route-bound recovery"));

    let stale_restart = z00z_storage::settlement::SettlementRecoveryState::new(
        recovery.version + 1,
        recovery.state_root,
        recovery.root_generation,
        recovery.proof_version,
        recovery.bucket_policy_generation,
        recovery.bucket_policy_id,
        recovery.journal_lineage,
    )
    .with_route(recovery.route.expect("route-bound recovery"));

    let cases = vec![
        RejectCase {
            id: FOV_T_001,
            kind: "wrong lineage",
            requester: secondary,
            intent: RecoveryIntent::TakeoverSecondary,
            want_class: RejectClass::PolicyReject,
            want_detail: "wrong lineage",
            table: placement_table(live_route, primary, vec![ready], recovery.journal_lineage),
            current: wrong_lineage,
        },
        RejectCase {
            id: FOV_T_001,
            kind: "wrong generation",
            requester: secondary,
            intent: RecoveryIntent::TakeoverSecondary,
            want_class: RejectClass::PolicyReject,
            want_detail: "wrong generation",
            table: wrong_generation_table.clone(),
            current: recovery.clone(),
        },
        RejectCase {
            id: FOV_T_001,
            kind: "wrong route digest",
            requester: secondary,
            intent: RecoveryIntent::TakeoverSecondary,
            want_class: RejectClass::PolicyReject,
            want_detail: "wrong route digest",
            table: placement_table(live_route, primary, vec![ready], recovery.journal_lineage),
            current: wrong_route_digest,
        },
        RejectCase {
            id: FOV_T_001,
            kind: "wrong shard",
            requester: secondary,
            intent: RecoveryIntent::TakeoverSecondary,
            want_class: RejectClass::PolicyReject,
            want_detail: "wrong shard",
            table: placement_table(live_route, primary, vec![ready], recovery.journal_lineage),
            current: wrong_shard,
        },
        RejectCase {
            id: FOV_T_001,
            kind: "stale local root",
            requester: secondary,
            intent: RecoveryIntent::TakeoverSecondary,
            want_class: RejectClass::PolicyReject,
            want_detail: "stale local root",
            table: placement_table(live_route, primary, vec![ready], recovery.journal_lineage),
            current: stale_root,
        },
        RejectCase {
            id: FOV_T_001,
            kind: "stale restart",
            requester: secondary,
            intent: RecoveryIntent::TakeoverSecondary,
            want_class: RejectClass::PolicyReject,
            want_detail: "stale restart",
            table: placement_table(live_route, primary, vec![ready], recovery.journal_lineage),
            current: stale_restart,
        },
        RejectCase {
            id: FOV_T_002,
            kind: "secondary aggregator down",
            requester: secondary,
            intent: RecoveryIntent::TakeoverSecondary,
            want_class: RejectClass::DeferredRetry,
            want_detail: "secondary aggregator down",
            table: pending_table,
            current: recovery.clone(),
        },
        RejectCase {
            id: FOV_T_002,
            kind: "split-brain",
            requester: primary,
            intent: RecoveryIntent::TakeoverSecondary,
            want_class: RejectClass::PolicyReject,
            want_detail: "split-brain",
            table: placement_table(live_route, primary, vec![ready], recovery.journal_lineage),
            current: recovery.clone(),
        },
        RejectCase {
            id: FOV_T_002,
            kind: "route migration during crash",
            requester: secondary,
            intent: RecoveryIntent::TakeoverSecondary,
            want_class: RejectClass::PolicyReject,
            want_detail: "wrong generation",
            table: wrong_generation_table,
            current: recovery.clone(),
        },
    ];

    assert_eq!(ROUTE_MIGRATION_FIXTURE, "Route migration fixture");

    let boundary = RecoveryBoundary;
    for case in cases {
        let err = boundary
            .resume(
                case.requester,
                &case.table,
                &record,
                &case.current,
                case.intent,
            )
            .expect_err(case.kind);
        assert_eq!(err.class, case.want_class, "{} {}", case.id, case.kind);
        assert!(
            err.detail.contains(case.want_detail),
            "{} {} -> {}",
            case.id,
            case.kind,
            err.detail
        );
    }

    Ok(())
}

#[test]
fn test_blocks_failover_reentry() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let old_home = temp.path().join("old_3a7s");
    let new_home = temp.path().join("new_2a7s");
    write_home(&old_home, 1, &staged_three_by_seven(8900));
    write_home(&new_home, 2, &staged_two_by_seven(9000));
    set_activation_checkpoint(&old_home, 11);
    bind_previous_generation(&new_home, &read_route_table(&old_home));
    set_activation_checkpoint(&new_home, 42);

    let old_table = read_route_table(&old_home);
    let new_table = read_route_table(&new_home);
    new_table
        .validate_migration(&old_table)
        .expect("decommissioned topology must stay generation bound");

    let new_cfg = load_cfg(&new_home);
    let row = placement_row(&new_cfg, 5, 2);
    let recovery = route_bound_recovery_state(
        0x84,
        batch_id("decommissioned-aggregator"),
        row.route,
        [0x84; 32],
    )?;
    let recovery = SettlementRecoveryState::new(
        recovery.version,
        recovery.state_root,
        recovery.root_generation,
        recovery.proof_version,
        recovery.bucket_policy_generation,
        recovery.bucket_policy_id,
        row.expected_journal_lineage,
    )
    .with_route(recovery.route.expect("route-bound recovery"));
    let record = recovery_record(
        "decommissioned-aggregator",
        row.route,
        row.primary_id,
        row.secondaries.clone(),
        recovery.clone(),
    );

    let err = RecoveryBoundary
        .resume(
            AggregatorId::new(5),
            &new_cfg.placement_table().expect("placement table"),
            &record,
            &recovery,
            RecoveryIntent::TakeoverSecondary,
        )
        .expect_err("removed aggregator must not re-enter failover");

    assert_eq!(err.class, RejectClass::PolicyReject);
    assert!(err.detail.contains("not a lawful secondary aggregator"));

    Ok(())
}

#[test]
fn test_old_primary_failback_rejects() -> Result<(), Box<dyn std::error::Error>> {
    let live_route = route(6, 13);
    let recovery = route_bound_recovery_state(
        0x85,
        batch_id("old-primary-failback"),
        live_route,
        [0x45; 32],
    )?;
    let old_primary = AggregatorId::new(31);
    let takeover = AggregatorId::new(32);
    let witness = AggregatorId::new(33);
    let record = recovery_record(
        "old-primary-failback",
        live_route,
        old_primary,
        vec![
            SecondaryState::ready(takeover),
            SecondaryState::ready(witness),
        ],
        recovery.clone(),
    );
    let taken_over = placement_table(
        live_route,
        takeover,
        vec![
            SecondaryState::ready(old_primary),
            SecondaryState::ready(witness),
        ],
        recovery.journal_lineage,
    );

    let err = RecoveryBoundary
        .resume(
            old_primary,
            &taken_over,
            &record,
            &recovery,
            RecoveryIntent::RestartPrimary,
        )
        .expect_err("old primary restart must reject after takeover");

    assert_eq!(err.class, RejectClass::PolicyReject);
    assert!(err.detail.contains("live primary owner"));

    Ok(())
}

#[test]
fn test_rotated_primary_reentry_rejects() -> Result<(), Box<dyn std::error::Error>> {
    let live_route = route(7, 14);
    let recovery = route_bound_recovery_state(
        0x86,
        batch_id("rotated-primary-reentry"),
        live_route,
        [0x46; 32],
    )?;
    let old_primary = AggregatorId::new(41);
    let takeover = AggregatorId::new(42);
    let rotated = AggregatorId::new(43);
    let record = recovery_record(
        "rotated-primary-reentry",
        live_route,
        old_primary,
        vec![
            SecondaryState::ready(takeover),
            SecondaryState::ready(rotated),
        ],
        recovery.clone(),
    );
    let rotated_live = placement_table(
        live_route,
        rotated,
        vec![
            SecondaryState::ready(old_primary),
            SecondaryState::ready(takeover),
        ],
        recovery.journal_lineage,
    );

    let err = RecoveryBoundary
        .resume(
            old_primary,
            &rotated_live,
            &record,
            &recovery,
            RecoveryIntent::RestartPrimary,
        )
        .expect_err("old primary restart must reject after planned rotation");

    assert_eq!(err.class, RejectClass::PolicyReject);
    assert!(err.detail.contains("live primary owner"));

    Ok(())
}
