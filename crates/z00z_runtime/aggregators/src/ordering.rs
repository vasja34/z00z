#![forbid(unsafe_code)]

use crate::{
    batch_planner::BatchPlanner,
    types::{BatchId, BatchPlanned, OrderedBatch, RejectRecord, WorkItem},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct OrderingBoundary {
    planner: BatchPlanner,
}

impl OrderingBoundary {
    #[must_use]
    pub fn new(planner: BatchPlanner) -> Self {
        Self { planner }
    }

    pub fn plan_batch(
        &self,
        batch_id: BatchId,
        items: &[WorkItem],
    ) -> Result<BatchPlanned, RejectRecord> {
        self.planner.plan_batch(batch_id, items)
    }

    pub fn make_batch(
        &self,
        batch_id: BatchId,
        items: &[WorkItem],
    ) -> Result<OrderedBatch, RejectRecord> {
        self.planner.make_batch(batch_id, items)
    }
}
