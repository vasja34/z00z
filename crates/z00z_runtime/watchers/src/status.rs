#![forbid(unsafe_code)]

use z00z_aggregators::{
    AggregatorId, BatchId, DistNote, PublicationState, ShardExecState, ShardExecTicket, ShardId,
    ShardPlacementView,
};
use z00z_validators::VerdictKind;

use crate::{
    alerts::AlertSeverity,
    da_health::{ProviderOutcome, ProviderStage},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationSnapshot {
    pub batch_id: BatchId,
    pub publication_state: PublicationState,
    pub shard_id: Option<ShardId>,
    pub aggregator_id: Option<AggregatorId>,
    pub routing_generation: Option<u64>,
    pub exec_state: Option<ShardExecState>,
    pub binding_digest: Option<[u8; 32]>,
    pub route_table_digest: Option<[u8; 32]>,
    pub verdict_kind: Option<VerdictKind>,
    pub provider_stage: Option<ProviderStage>,
    pub provider_outcome: Option<ProviderOutcome>,
    pub runtime_notes: Vec<DistNote>,
    pub runtime_truth: bool,
    pub alert_counts: AlertCounts,
}

impl ObservationSnapshot {
    #[must_use]
    pub fn from_runtime(
        batch_id: BatchId,
        publication_state: PublicationState,
        placement: Option<&ShardPlacementView>,
        exec_ticket: Option<&ShardExecTicket>,
        verdict_kind: Option<VerdictKind>,
        provider_stage: Option<ProviderStage>,
        provider_outcome: Option<ProviderOutcome>,
        alert_counts: AlertCounts,
    ) -> Self {
        let placement = exec_ticket.map(|ticket| &ticket.placement).or(placement);
        Self {
            batch_id,
            publication_state,
            shard_id: placement.map(|item| item.route.shard_id),
            aggregator_id: placement.map(|item| item.primary_id),
            routing_generation: placement.map(|item| item.route.routing_generation),
            exec_state: exec_ticket.map(|ticket| ticket.state),
            binding_digest: None,
            route_table_digest: None,
            verdict_kind,
            provider_stage,
            provider_outcome,
            runtime_notes: Vec::new(),
            runtime_truth: false,
            alert_counts,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AlertCounts {
    pub info: u64,
    pub warn: u64,
    pub critical: u64,
}

impl AlertCounts {
    #[must_use]
    pub const fn with_increment(mut self, severity: AlertSeverity) -> Self {
        match severity {
            AlertSeverity::Info => self.info += 1,
            AlertSeverity::Warn => self.warn += 1,
            AlertSeverity::Critical => self.critical += 1,
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefers_exec_ticket_placement() {
        let snapshot = ObservationSnapshot::from_runtime(
            BatchId::from_bytes([1u8; 32]),
            PublicationState::HandedOff,
            Some(&placement_view(1, 3, 7)),
            Some(&exec_ticket(2, 4, 8, ShardExecState::Running)),
            None,
            Some(ProviderStage::Resolve),
            Some(ProviderOutcome::Success),
            AlertCounts::default(),
        );

        assert_eq!(snapshot.shard_id, Some(ShardId::new(2)));
        assert_eq!(snapshot.aggregator_id, Some(AggregatorId::new(8)));
        assert_eq!(snapshot.routing_generation, Some(4));
        assert_eq!(snapshot.exec_state, Some(ShardExecState::Running));
    }

    fn placement_view(shard: u16, generation: u64, aggregator: u16) -> ShardPlacementView {
        ShardPlacementView {
            route: z00z_aggregators::BatchRoute {
                shard_id: ShardId::new(shard),
                routing_generation: generation,
            },
            primary_id: AggregatorId::new(aggregator),
            standby: Vec::new(),
            expected_journal_lineage: [0u8; 32],
        }
    }

    fn exec_ticket(
        shard: u16,
        generation: u64,
        aggregator: u16,
        state: ShardExecState,
    ) -> ShardExecTicket {
        ShardExecTicket {
            batch_id: BatchId::from_bytes([2u8; 32]),
            placement: placement_view(shard, generation, aggregator),
            state,
        }
    }
}
