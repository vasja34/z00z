use tempfile::tempdir;

use z00z_aggregators::{AggregatorId, RecoveryBoundary, RecoveryIntent, RejectClass};

#[path = "test_hjmt_topology_support.rs"]
mod hjmt_topology_support;
#[path = "test_recovery_common.rs"]
mod test_recovery_common;

use hjmt_topology_support::{
    bind_previous_generation, canonical_five_by_seven, load_cfg, new_transfer_six_by_seven,
    placement_row, primary_id, read_route_table, remaining_transfer_five_by_seven,
    set_activation_checkpoint, shard_ids, staged_five_by_seven, staged_three_by_seven,
    staged_two_by_seven, write_home,
};
use test_recovery_common::{
    batch_id, live_failover_manifest, placement_table, recovery_record, route,
    route_bound_recovery_state, route_migration_publication_case,
};

#[test]
fn test_remaining_owner_generation() {
    let temp = tempdir().expect("tempdir");
    let old_home = temp.path().join("old_5a7s");
    let new_home = temp.path().join("new_5a7s_remaining");
    write_home(&old_home, 1, &canonical_five_by_seven(8100));
    write_home(&new_home, 2, &remaining_transfer_five_by_seven(8200));
    set_activation_checkpoint(&old_home, 11);
    bind_previous_generation(&new_home, &read_route_table(&old_home));
    set_activation_checkpoint(&new_home, 42);

    let old_cfg = load_cfg(&old_home);
    let new_cfg = load_cfg(&new_home);
    let old_table = read_route_table(&old_home);
    let new_table = read_route_table(&new_home);
    let new_row = placement_row(&new_cfg, 0, 2);

    new_table
        .validate_migration(&old_table)
        .expect("remaining-aggregator transfer must stay generation bound");

    assert_eq!(primary_id(&old_cfg, 0, 1), AggregatorId::new(0));
    assert_eq!(new_row.primary_id, AggregatorId::new(1));
    assert!(new_row
        .standby
        .iter()
        .any(|standby| standby.aggregator_id == AggregatorId::new(0)));
    assert_eq!(old_table.activation_checkpoint, 11);
    assert_eq!(new_table.activation_checkpoint, 42);
}

#[test]
fn test_new_owner_generation() {
    let temp = tempdir().expect("tempdir");
    let old_home = temp.path().join("old_5a7s");
    let new_home = temp.path().join("new_6a7s_transfer");
    write_home(&old_home, 1, &canonical_five_by_seven(8300));
    write_home(&new_home, 2, &new_transfer_six_by_seven(8400));
    set_activation_checkpoint(&old_home, 11);
    bind_previous_generation(&new_home, &read_route_table(&old_home));
    set_activation_checkpoint(&new_home, 42);

    let old_cfg = load_cfg(&old_home);
    let new_cfg = load_cfg(&new_home);
    let old_table = read_route_table(&old_home);
    let new_table = read_route_table(&new_home);
    let new_row = placement_row(&new_cfg, 0, 2);

    new_table
        .validate_migration(&old_table)
        .expect("new-aggregator transfer must stay generation bound");

    assert_eq!(primary_id(&old_cfg, 0, 1), AggregatorId::new(0));
    assert_eq!(new_row.primary_id, AggregatorId::new(5));
    assert!(new_row
        .standby
        .iter()
        .any(|standby| standby.aggregator_id == AggregatorId::new(0)));
    assert_eq!(new_cfg.node_stat().expect("new stat").agg_count, 6);
}

#[test]
fn test_removes_owner_refs() {
    let temp = tempdir().expect("tempdir");
    let old_home = temp.path().join("old_3a7s");
    let new_home = temp.path().join("new_2a7s");
    let removed = AggregatorId::new(5);
    write_home(&old_home, 1, &staged_three_by_seven(8500));
    write_home(&new_home, 2, &staged_two_by_seven(8600));
    set_activation_checkpoint(&old_home, 11);
    bind_previous_generation(&new_home, &read_route_table(&old_home));
    set_activation_checkpoint(&new_home, 42);

    let old_cfg = load_cfg(&old_home);
    let new_cfg = load_cfg(&new_home);
    let old_table = read_route_table(&old_home);
    let new_table = read_route_table(&new_home);

    new_table
        .validate_migration(&old_table)
        .expect("decommission must stay generation bound");

    assert_eq!(old_cfg.node_stat().expect("old stat").agg_count, 3);
    assert_eq!(new_cfg.node_stat().expect("new stat").agg_count, 2);
    assert_eq!(new_cfg.node_stat().expect("new stat").shard_count, 7);

    for shard_id in [5_u16, 6] {
        let old_row = placement_row(&old_cfg, shard_id, 1);
        let new_row = placement_row(&new_cfg, shard_id, 2);
        assert_eq!(old_row.primary_id, removed);
        assert_ne!(new_row.primary_id, removed);
        assert!(
            old_row
                .standby
                .iter()
                .any(|standby| standby.aggregator_id == new_row.primary_id),
            "shard {shard_id} must transfer to a lawful prior standby"
        );
    }

    for shard_id in shard_ids(&new_home) {
        let row = placement_row(&new_cfg, shard_id, 2);
        assert_ne!(
            row.primary_id, removed,
            "removed aggregator must not remain primary for shard {shard_id}"
        );
        assert!(
            row.standby
                .iter()
                .all(|standby| standby.aggregator_id != removed),
            "removed aggregator must not remain standby for shard {shard_id}"
        );
    }

    assert_eq!(old_table.activation_checkpoint, 11);
    assert_eq!(new_table.activation_checkpoint, 42);
}

#[test]
fn test_faildown_keeps_generation() {
    let temp = tempdir().expect("tempdir");
    let old_home = temp.path().join("old_3a7s");
    let mid_home = temp.path().join("mid_2a7s");
    let new_home = temp.path().join("new_5a7s");
    let removed = AggregatorId::new(5);
    write_home(&old_home, 1, &staged_three_by_seven(9100));
    write_home(&mid_home, 2, &staged_two_by_seven(9200));
    write_home(&new_home, 3, &staged_five_by_seven(9300));
    set_activation_checkpoint(&old_home, 11);
    bind_previous_generation(&mid_home, &read_route_table(&old_home));
    set_activation_checkpoint(&mid_home, 42);
    bind_previous_generation(&new_home, &read_route_table(&mid_home));
    set_activation_checkpoint(&new_home, 101);

    let old_cfg = load_cfg(&old_home);
    let mid_cfg = load_cfg(&mid_home);
    let new_cfg = load_cfg(&new_home);
    let old_table = read_route_table(&old_home);
    let mid_table = read_route_table(&mid_home);
    let new_table = read_route_table(&new_home);

    mid_table
        .validate_migration(&old_table)
        .expect("fail-down must stay generation bound");
    new_table
        .validate_migration(&mid_table)
        .expect("fail-up must stay generation bound");

    assert_eq!(old_cfg.node_stat().expect("old stat").agg_count, 3);
    assert_eq!(mid_cfg.node_stat().expect("mid stat").agg_count, 2);
    assert_eq!(new_cfg.node_stat().expect("new stat").agg_count, 5);
    assert_eq!(new_cfg.node_stat().expect("new stat").routing_generation, 3);
    assert_eq!(shard_ids(&mid_home), shard_ids(&new_home));

    for shard_id in shard_ids(&mid_home) {
        let row = placement_row(&mid_cfg, shard_id, 2);
        assert_ne!(row.primary_id, removed);
        assert!(row
            .standby
            .iter()
            .all(|standby| standby.aggregator_id != removed));
    }
    for shard_id in shard_ids(&new_home) {
        let row = placement_row(&new_cfg, shard_id, 3);
        assert_ne!(row.primary_id, removed);
        assert!(row
            .standby
            .iter()
            .all(|standby| standby.aggregator_id != removed));
    }

    assert_eq!(old_table.activation_checkpoint, 11);
    assert_eq!(mid_table.activation_checkpoint, 42);
    assert_eq!(new_table.activation_checkpoint, 101);
}

#[test]
fn test_rejects_route_migration() {
    let old_route = route(0, 1);
    let recovery = route_bound_recovery_state(
        0x91,
        batch_id("route-migration-drift"),
        old_route,
        [0x51; 32],
    )
    .expect("recovery state");
    let new_route = route(0, 2);
    let record = recovery_record(
        "route-migration-drift",
        old_route,
        AggregatorId::new(0),
        vec![z00z_aggregators::StandbyState::ready(AggregatorId::new(1))],
        recovery.clone(),
    );
    let live_table = placement_table(
        new_route,
        AggregatorId::new(0),
        vec![z00z_aggregators::StandbyState::ready(AggregatorId::new(1))],
        recovery.journal_lineage,
    );

    let err = RecoveryBoundary
        .resume(
            AggregatorId::new(1),
            &live_table,
            &record,
            &recovery,
            RecoveryIntent::TakeoverStandby,
        )
        .expect_err("route migration during crash must reject");

    assert_eq!(err.class, RejectClass::PolicyReject);
    assert!(err.detail.contains("wrong generation"));
}

#[test]
fn test_fov_g004_prior_root() -> Result<(), Box<dyn std::error::Error>> {
    let case = route_migration_publication_case()?;
    let err = RecoveryBoundary
        .resume(
            AggregatorId::new(22),
            &case.live_table,
            &case.record,
            &case.recovery,
            RecoveryIntent::TakeoverStandby,
        )
        .expect_err("route migration during crash must reject before a new public root is visible");

    assert_eq!(err.class, RejectClass::PolicyReject);
    assert!(err.detail.contains("wrong generation"));

    let prior_root = hex::encode(case.prior.public_root_v1()?.into_bytes());
    let migration_candidate_root =
        hex::encode(case.migration_candidate.public_root_v1()?.into_bytes());
    assert_ne!(
        prior_root, migration_candidate_root,
        "the migration candidate must not collapse onto the prior visible public root"
    );

    let manifest = live_failover_manifest()?;
    let row = manifest
        .cases
        .iter()
        .find(|case| {
            case.fixture_id == "FOV-G-004"
                && case.kind == "crash during route migration resolves lawfully"
        })
        .expect("FOV-G-004 manifest row");
    assert_eq!(row.expected_public_root_hexes, vec![prior_root]);

    Ok(())
}
