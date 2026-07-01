#![forbid(unsafe_code)]

use crate::{
    placement::{ShardPlacementTable, ShardPlacementView},
    types::{BatchPlanned, RejectClass, RejectRecord},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShardExecState {
    Routed,
    Running,
    RetryPending,
    RecoveryPending,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShardExecTicket {
    pub batch_id: crate::types::BatchId,
    pub placement: ShardPlacementView,
    pub state: ShardExecState,
}

impl ShardExecTicket {
    #[must_use]
    pub fn with_state(&self, state: ShardExecState) -> Self {
        Self {
            batch_id: self.batch_id,
            placement: self.placement.clone(),
            state,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ShardExecutor {
    placement_table: ShardPlacementTable,
}

impl ShardExecutor {
    #[must_use]
    pub fn new(placement_table: ShardPlacementTable) -> Self {
        Self { placement_table }
    }

    #[must_use]
    pub fn placement_table(&self) -> &ShardPlacementTable {
        &self.placement_table
    }

    pub fn route(&self, planned: &BatchPlanned) -> Result<ShardExecTicket, RejectRecord> {
        let placement = self
            .placement_table
            .view(planned)
            .ok_or_else(|| RejectRecord {
                intake_id: None,
                class: RejectClass::PolicyReject,
                detail: "placement table does not own the planned shard route".to_string(),
            })?;

        Ok(ShardExecTicket {
            batch_id: planned.batch_id,
            placement,
            state: ShardExecState::Routed,
        })
    }

    #[must_use]
    pub fn mark_running(&self, ticket: &ShardExecTicket) -> ShardExecTicket {
        ticket.with_state(ShardExecState::Running)
    }

    #[must_use]
    pub fn mark_retry(&self, ticket: &ShardExecTicket) -> ShardExecTicket {
        ticket.with_state(ShardExecState::RetryPending)
    }

    #[must_use]
    pub fn mark_recovery(&self, ticket: &ShardExecTicket) -> ShardExecTicket {
        ticket.with_state(ShardExecState::RecoveryPending)
    }

    #[must_use]
    pub fn mark_complete(&self, ticket: &ShardExecTicket) -> ShardExecTicket {
        ticket.with_state(ShardExecState::Completed)
    }
}

#[cfg(test)]
mod tests {
    use z00z_storage::checkpoint::CheckpointDraftId;

    use super::*;
    use crate::{
        placement::{AggregatorId, ShardPlacement, StandbyState},
        types::{BatchId, BatchPlanned, BatchRoute, PlanDigest, ShardId},
    };

    #[test]
    fn test_route_emits_runtime_ticket() {
        let route = BatchRoute {
            shard_id: ShardId::new(1),
            routing_generation: 3,
        };
        let mut table = ShardPlacementTable::default();
        table.insert(ShardPlacement::new(
            route,
            AggregatorId::new(7),
            vec![StandbyState::pending(AggregatorId::new(8))],
            [0x31; 32],
        ));
        let executor = ShardExecutor::new(table);

        let ticket = executor
            .route(&planned_batch(route))
            .expect("runtime route");

        assert_eq!(ticket.state, ShardExecState::Routed);
        assert_eq!(ticket.placement.primary_id, AggregatorId::new(7));
        assert_eq!(ticket.placement.route, route);
    }

    #[test]
    fn test_route_rejects_unowned_route() {
        let route = BatchRoute {
            shard_id: ShardId::new(1),
            routing_generation: 3,
        };
        let executor = ShardExecutor::default();
        let err = executor
            .route(&planned_batch(route))
            .expect_err("missing placement must reject");

        assert_eq!(err.class, RejectClass::PolicyReject);
        assert!(err.detail.contains("placement table"));
    }

    #[test]
    fn test_state_transitions_operational() {
        let route = BatchRoute {
            shard_id: ShardId::new(1),
            routing_generation: 3,
        };
        let mut table = ShardPlacementTable::default();
        table.insert(ShardPlacement::new(
            route,
            AggregatorId::new(7),
            Vec::new(),
            [0x41; 32],
        ));
        let executor = ShardExecutor::new(table);
        let ticket = executor
            .route(&planned_batch(route))
            .expect("runtime route");

        let running = executor.mark_running(&ticket);
        let recovered = executor.mark_recovery(&running);
        let completed = executor.mark_complete(&recovered);

        assert_eq!(running.state, ShardExecState::Running);
        assert_eq!(recovered.state, ShardExecState::RecoveryPending);
        assert_eq!(completed.state, ShardExecState::Completed);
        assert_eq!(completed.placement, ticket.placement);
    }

    fn planned_batch(route: BatchRoute) -> BatchPlanned {
        BatchPlanned {
            batch_id: BatchId::new(CheckpointDraftId::new([6u8; 32])),
            route,
            route_table_digest: PlanDigest::new([7u8; 32]),
            intake_ids: Vec::new(),
            op_count: 1,
            plan_digest: PlanDigest::new([8u8; 32]),
        }
    }
}
