use tempfile::tempdir;
use z00z_aggregators::{AggregatorId, DistNoteKind, RouteRollout, ShardPlacementTable};

#[path = "test_hjmt_topology_support.rs"]
mod hjmt_topology_support;

use hjmt_topology_support::{
    bind_previous_generation, canonical_five_by_seven, load_cfg, owner_join_six_by_seven,
    read_route_table, set_activation_checkpoint, write_home,
};

#[test]
fn test_requires_checkpoint_ack() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let old_home = temp.path().join("old_5a7s");
    let new_home = temp.path().join("new_6a7s_owner");
    write_home(&old_home, 1, &canonical_five_by_seven(9800));
    write_home(&new_home, 2, &owner_join_six_by_seven(9900));
    set_activation_checkpoint(&old_home, 11);
    bind_previous_generation(&new_home, &read_route_table(&old_home));
    set_activation_checkpoint(&new_home, 42);

    let old_table = read_route_table(&old_home);
    let new_table = read_route_table(&new_home);
    let new_cfg = load_cfg(&new_home);
    let mut rollout = RouteRollout::new(old_table);
    let members = member_ids(&new_cfg.placement_table().expect("placement table"));

    runtime(rollout.stage(new_table.clone(), members.clone()))?;
    let route_digest = new_table.digest();
    for member in members.iter().take(members.len().saturating_sub(1)) {
        runtime(rollout.ack(*member, route_digest, new_table.routing_generation))?;
    }

    let ack_err = rollout
        .activate(new_table.activation_checkpoint)
        .expect_err("missing ack must fail closed");
    assert!(ack_err.detail.contains("missing ack"));

    runtime(rollout.ack(
        *members.last().expect("last rollout member"),
        route_digest,
        new_table.routing_generation,
    ))?;

    let checkpoint_err = rollout
        .activate(new_table.activation_checkpoint - 1)
        .expect_err("stale checkpoint must reject");
    assert!(checkpoint_err.detail.contains("stale checkpoint"));

    let digest = runtime(rollout.activate(new_table.activation_checkpoint))?;
    assert_eq!(digest, route_digest);
    assert_eq!(rollout.active().digest(), route_digest);

    let notes = rollout.take_notes();
    assert!(notes
        .iter()
        .any(|note| note.kind == DistNoteKind::RouteRollout));
    assert!(notes.iter().all(|note| !note.proof_truth));

    Ok(())
}

#[test]
fn test_rejects_rollout_drift() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let old_home = temp.path().join("old_5a7s");
    let new_home = temp.path().join("new_6a7s_owner");
    write_home(&old_home, 1, &canonical_five_by_seven(9810));
    write_home(&new_home, 2, &owner_join_six_by_seven(9910));
    bind_previous_generation(&new_home, &read_route_table(&old_home));

    let old_table = read_route_table(&old_home);
    let new_table = read_route_table(&new_home);
    let new_cfg = load_cfg(&new_home);
    let mut rollout = RouteRollout::new(old_table.clone());
    let members = member_ids(&new_cfg.placement_table().expect("placement table"));
    runtime(rollout.stage(new_table.clone(), members.clone()))?;

    let stale_digest = rollout
        .ack(members[0], old_table.digest(), new_table.routing_generation)
        .expect_err("stale digest must reject");
    assert!(stale_digest.detail.contains("stale digest"));

    let wrong_generation = rollout
        .ack(members[0], new_table.digest(), old_table.routing_generation)
        .expect_err("mixed generation must reject");
    assert!(wrong_generation.detail.contains("wrong generation"));

    let late_joiner = rollout
        .ack(
            AggregatorId::new(99),
            new_table.digest(),
            new_table.routing_generation,
        )
        .expect_err("late joiner must reject");
    assert!(late_joiner.detail.contains("late joiner"));

    Ok(())
}

fn member_ids(table: &ShardPlacementTable) -> Vec<AggregatorId> {
    let mut members = table
        .placements()
        .flat_map(|placement| {
            let mut out = vec![placement.primary_id];
            out.extend(
                placement
                    .secondaries
                    .iter()
                    .map(|secondary| secondary.aggregator_id),
            );
            out
        })
        .collect::<Vec<_>>();
    members.sort_unstable();
    members.dedup();
    members
}

fn runtime<T>(
    result: Result<T, z00z_aggregators::RejectRecord>,
) -> Result<T, Box<dyn std::error::Error>> {
    result.map_err(|err| std::io::Error::other(err.detail).into())
}
