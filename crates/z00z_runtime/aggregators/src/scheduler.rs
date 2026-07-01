#![forbid(unsafe_code)]

use crate::{
    dist_scheduler::DistScheduler,
    placement::ShardPlacementTable,
    types::{RejectRecord, WorkItem},
    ShardRouteTable,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SchedulerBoundary;

impl SchedulerBoundary {
    pub fn plan_waves(
        &self,
        route_table: &ShardRouteTable,
        placement_table: &ShardPlacementTable,
        items: &[WorkItem],
    ) -> Result<Vec<Vec<WorkItem>>, RejectRecord> {
        DistScheduler::new(route_table.clone(), placement_table.clone())
            .plan_waves(items)
            .map(|waves| {
                waves
                    .into_iter()
                    .map(|wave| {
                        wave.batches
                            .into_iter()
                            .flat_map(|batch| batch.items)
                            .collect()
                    })
                    .collect()
            })
    }
}
