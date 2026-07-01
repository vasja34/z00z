mod test_common;

use z00z_aggregators::{
    BatchPlanner, PlannerMode, RejectClass, RouteRangeRule, ShardId, ShardRouteTable,
};

use self::test_common::{
    batch_id, bridge_table, claim_item, planner_copies, span_table, split_items, tx_item, verdict,
    PlannerOut, HASH_MAX, HASH_MIN,
};

#[test]
fn test_modes_match_accepts() {
    // These labels name the mandatory workload families; planner truth is the same
    // because this phase only admits payload-bound route keys, not storage semantics.
    let cases = vec![
        (
            "broad",
            vec![
                tx_item("broad-a"),
                claim_item("broad-b"),
                tx_item("broad-c"),
                claim_item("broad-d"),
            ],
        ),
        (
            "hot-shard",
            vec![
                tx_item("hot-shard-a"),
                tx_item("hot-shard-b"),
                tx_item("hot-shard-c"),
                tx_item("hot-shard-d"),
                claim_item("hot-shard-e"),
            ],
        ),
        (
            "hot-serial",
            vec![
                claim_item("hot-serial-a"),
                claim_item("hot-serial-b"),
                claim_item("hot-serial-c"),
                tx_item("hot-serial-d"),
            ],
        ),
        (
            "delete-heavy",
            vec![
                claim_item("delete-heavy-a"),
                claim_item("delete-heavy-b"),
                tx_item("delete-heavy-c"),
            ],
        ),
        (
            "search-heavy",
            vec![
                tx_item("search-heavy-a"),
                tx_item("search-heavy-b"),
                tx_item("search-heavy-c"),
                claim_item("search-heavy-d"),
            ],
        ),
        (
            "proof-heavy",
            vec![
                claim_item("proof-heavy-a"),
                tx_item("proof-heavy-b"),
                claim_item("proof-heavy-c"),
                tx_item("proof-heavy-d"),
                claim_item("proof-heavy-e"),
            ],
        ),
        (
            "mixed present or absent",
            vec![
                tx_item("mixed-a"),
                claim_item("mixed-b"),
                tx_item("mixed-c"),
                claim_item("mixed-d"),
            ],
        ),
    ];

    for (label, items) in cases {
        let table = span_table(&items, ShardId::new(1));
        let batch_id = batch_id(label);
        let central = run_mode(PlannerMode::Central, &table, batch_id, &items);
        let per_agg = run_mode(PlannerMode::PerAgg, &table, batch_id, &items);

        assert_eq!(central, per_agg, "planner mode drift for {label}");

        match central {
            PlannerOut::Accept(planned) => {
                assert_eq!(
                    planned.route.shard_id,
                    ShardId::new(1),
                    "wrong target shard for {label}"
                );
                assert_eq!(planned.route.routing_generation, table.routing_generation);
                assert_eq!(planned.route_table_digest, table.digest());
                assert_eq!(planned.op_count, items.len());
            }
            PlannerOut::Reject { detail, .. } => {
                panic!("accepted profile {label} rejected: {detail}")
            }
        }
    }
}

#[test]
fn test_modes_match_rejects() {
    let left = tx_item("cross-shard-left");
    let right = claim_item("cross-shard-right");
    let cross_table = split_items(&left, &right);
    let cross_items = vec![left, right];

    let cross_central = run_mode(
        PlannerMode::Central,
        &cross_table,
        batch_id("cross-shard"),
        &cross_items,
    );
    let cross_per_agg = run_mode(
        PlannerMode::PerAgg,
        &cross_table,
        batch_id("cross-shard"),
        &cross_items,
    );
    assert_eq!(cross_central, cross_per_agg);
    assert!(matches!(
        cross_central,
        PlannerOut::Reject {
            class: RejectClass::PolicyReject,
            ..
        }
    ));

    let invalid_table = ShardRouteTable {
        routing_generation: 1,
        shard_set: vec![ShardId::new(1), ShardId::new(0)],
        rules: vec![RouteRangeRule::new(HASH_MIN, HASH_MAX, ShardId::new(0))],
        previous_generation_digest: Some(bridge_table().digest()),
        activation_checkpoint: 77,
    };
    let invalid_items = vec![tx_item("invalid-route-a"), claim_item("invalid-route-b")];
    let invalid_central = run_mode(
        PlannerMode::Central,
        &invalid_table,
        batch_id("invalid-route"),
        &invalid_items,
    );
    let invalid_per_agg = run_mode(
        PlannerMode::PerAgg,
        &invalid_table,
        batch_id("invalid-route"),
        &invalid_items,
    );
    assert_eq!(invalid_central, invalid_per_agg);
    match invalid_central {
        PlannerOut::Reject { class, detail } => {
            assert_eq!(class, RejectClass::PolicyReject);
            assert!(detail.contains("route table contract violation"));
        }
        PlannerOut::Accept(_) => panic!("invalid route table unexpectedly accepted"),
    }
}

fn run_mode(
    mode: PlannerMode,
    table: &ShardRouteTable,
    batch_id: z00z_aggregators::BatchId,
    items: &[z00z_aggregators::WorkItem],
) -> PlannerOut {
    match mode {
        PlannerMode::Central => {
            verdict(BatchPlanner::new(table.clone()).plan_batch(batch_id, items))
        }
        PlannerMode::PerAgg => {
            let mut verdicts = planner_copies(table, 5)
                .into_iter()
                .map(|planner| verdict(planner.plan_batch(batch_id, items)))
                .collect::<Vec<_>>();
            let first = verdicts.remove(0);
            assert!(
                verdicts.into_iter().all(|item| item == first),
                "per-aggregator drift"
            );
            first
        }
    }
}
