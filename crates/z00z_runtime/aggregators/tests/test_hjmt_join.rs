use tempfile::tempdir;

use z00z_aggregators::AggregatorId;

#[path = "test_hjmt_topology_support.rs"]
mod hjmt_topology_support;

use hjmt_topology_support::{
    bind_previous_generation, canonical_five_by_seven, load_cfg, owner_join_six_by_seven,
    placement_row, primary_id, read_route_table, set_activation_checkpoint,
    standby_join_six_by_seven, write_home,
};

#[test]
fn test_join_keeps_route() {
    let temp = tempdir().expect("tempdir");
    let old_home = temp.path().join("old_5a7s");
    let new_home = temp.path().join("new_6a7s_standby");
    write_home(&old_home, 1, &canonical_five_by_seven(7700));
    write_home(&new_home, 1, &standby_join_six_by_seven(7800));
    set_activation_checkpoint(&old_home, 11);
    set_activation_checkpoint(&new_home, 11);

    let old_cfg = load_cfg(&old_home);
    let new_cfg = load_cfg(&new_home);
    let old_table = read_route_table(&old_home);
    let new_table = read_route_table(&new_home);
    let new_row = placement_row(&new_cfg, 0, 1);

    assert_eq!(old_cfg.node_stat().expect("old stat").agg_count, 5);
    assert_eq!(new_cfg.node_stat().expect("new stat").agg_count, 6);
    assert_eq!(new_cfg.node_stat().expect("new stat").shard_count, 7);
    assert_eq!(primary_id(&old_cfg, 0, 1), AggregatorId::new(0));
    assert_eq!(new_row.primary_id, AggregatorId::new(0));
    assert!(new_row
        .standby
        .iter()
        .any(|standby| standby.aggregator_id == AggregatorId::new(5)));
    assert_eq!(old_table.canonical_bytes(), new_table.canonical_bytes());
}

#[test]
fn test_join_advances_generation() {
    let temp = tempdir().expect("tempdir");
    let old_home = temp.path().join("old_5a7s");
    let new_home = temp.path().join("new_6a7s_owner");
    write_home(&old_home, 1, &canonical_five_by_seven(7900));
    write_home(&new_home, 2, &owner_join_six_by_seven(8000));
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
        .expect("owner activation must bind to generation advance");

    assert_eq!(primary_id(&old_cfg, 0, 1), AggregatorId::new(0));
    assert_eq!(new_row.primary_id, AggregatorId::new(5));
    assert!(new_row
        .standby
        .iter()
        .any(|standby| standby.aggregator_id == AggregatorId::new(0)));
    assert_eq!(old_cfg.node_stat().expect("old stat").routing_generation, 1);
    assert_eq!(new_cfg.node_stat().expect("new stat").routing_generation, 2);
    assert_eq!(old_table.activation_checkpoint, 11);
    assert_eq!(new_table.activation_checkpoint, 42);
}

#[test]
fn owner_join_rejects_checkpoint_rollback() {
    let temp = tempdir().expect("tempdir");
    let old_home = temp.path().join("old_5a7s");
    let new_home = temp.path().join("new_6a7s_owner");
    write_home(&old_home, 1, &canonical_five_by_seven(8100));
    write_home(&new_home, 2, &owner_join_six_by_seven(8200));
    set_activation_checkpoint(&old_home, 11);
    bind_previous_generation(&new_home, &read_route_table(&old_home));
    set_activation_checkpoint(&new_home, 10);

    let old_table = read_route_table(&old_home);
    let new_table = read_route_table(&new_home);

    let err = new_table
        .validate_migration(&old_table)
        .expect_err("owner activation must reject before the activation checkpoint advances");

    assert_eq!(format!("{err:?}"), "BadPrevGen");
}
