#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::types::{BatchPlanned, BatchRoute, ShardId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AggregatorId(u16);

impl AggregatorId {
    #[must_use]
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn as_u16(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StandbyState {
    pub aggregator_id: AggregatorId,
    pub is_ready: bool,
}

impl StandbyState {
    #[must_use]
    pub const fn ready(aggregator_id: AggregatorId) -> Self {
        Self {
            aggregator_id,
            is_ready: true,
        }
    }

    #[must_use]
    pub const fn pending(aggregator_id: AggregatorId) -> Self {
        Self {
            aggregator_id,
            is_ready: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShardPlacement {
    pub route: BatchRoute,
    pub primary_id: AggregatorId,
    pub standby: Vec<StandbyState>,
    pub expected_journal_lineage: [u8; 32],
}

impl ShardPlacement {
    #[must_use]
    pub fn new(
        route: BatchRoute,
        primary_id: AggregatorId,
        standby: Vec<StandbyState>,
        expected_journal_lineage: [u8; 32],
    ) -> Self {
        Self {
            route,
            primary_id,
            standby,
            expected_journal_lineage,
        }
    }

    #[must_use]
    pub fn view(&self) -> ShardPlacementView {
        ShardPlacementView {
            route: self.route,
            primary_id: self.primary_id,
            standby: self.standby.clone(),
            expected_journal_lineage: self.expected_journal_lineage,
        }
    }

    #[must_use]
    pub fn standby(&self, aggregator_id: AggregatorId) -> Option<&StandbyState> {
        self.standby
            .iter()
            .find(|standby| standby.aggregator_id == aggregator_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShardPlacementView {
    pub route: BatchRoute,
    pub primary_id: AggregatorId,
    pub standby: Vec<StandbyState>,
    pub expected_journal_lineage: [u8; 32],
}

impl ShardPlacementView {
    #[must_use]
    pub fn activate(&self, primary_id: AggregatorId) -> Self {
        let mut view = self.clone();
        view.primary_id = primary_id;
        view
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ShardPlacementTable {
    placements: BTreeMap<ShardId, ShardPlacement>,
}

impl ShardPlacementTable {
    #[must_use]
    pub fn with_primary(
        mut self,
        route: BatchRoute,
        primary_id: AggregatorId,
        expected_journal_lineage: [u8; 32],
    ) -> Self {
        self.insert(ShardPlacement::new(
            route,
            primary_id,
            Vec::new(),
            expected_journal_lineage,
        ));
        self
    }

    pub fn insert(&mut self, placement: ShardPlacement) -> Option<ShardPlacement> {
        self.placements.insert(placement.route.shard_id, placement)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.placements.is_empty()
    }

    #[must_use]
    pub fn placement(&self, route: BatchRoute) -> Option<&ShardPlacement> {
        self.placements
            .get(&route.shard_id)
            .filter(|placement| placement.route.routing_generation == route.routing_generation)
    }

    #[must_use]
    pub fn placement_for_shard(&self, shard_id: ShardId) -> Option<&ShardPlacement> {
        self.placements.get(&shard_id)
    }

    pub fn placements(&self) -> impl Iterator<Item = &ShardPlacement> {
        self.placements.values()
    }

    #[must_use]
    pub fn view(&self, planned: &BatchPlanned) -> Option<ShardPlacementView> {
        self.placement(planned.route).map(ShardPlacement::view)
    }
}

#[cfg(test)]
mod tests {
    use z00z_storage::checkpoint::CheckpointDraftId;

    use super::*;
    use crate::types::{BatchId, BatchPlanned, PlanDigest, ShardId};

    #[test]
    fn test_for_matches_route_gen() {
        let route = BatchRoute {
            shard_id: ShardId::new(2),
            routing_generation: 7,
        };
        let placement = ShardPlacement::new(
            route,
            AggregatorId::new(11),
            vec![StandbyState::ready(AggregatorId::new(12))],
            [0x11; 32],
        );
        let mut table = ShardPlacementTable::default();
        table.insert(placement);

        let planned = planned_batch(route);
        let view = table.view(&planned).expect("matching placement");

        assert_eq!(view.route, route);
        assert_eq!(view.primary_id, AggregatorId::new(11));
        assert_eq!(view.standby.len(), 1);
        assert!(view.standby[0].is_ready);
        assert_eq!(view.expected_journal_lineage, [0x11; 32]);
    }

    #[test]
    fn test_for_rejects_gen_drift() {
        let route = BatchRoute {
            shard_id: ShardId::new(2),
            routing_generation: 7,
        };
        let mut table = ShardPlacementTable::default();
        table.insert(ShardPlacement::new(
            route,
            AggregatorId::new(11),
            Vec::new(),
            [0x22; 32],
        ));

        let drifted = planned_batch(BatchRoute {
            shard_id: ShardId::new(2),
            routing_generation: 8,
        });

        assert!(table.view(&drifted).is_none());
        assert!(table.placement_for_shard(route.shard_id).is_some());
    }

    fn planned_batch(route: BatchRoute) -> BatchPlanned {
        BatchPlanned {
            batch_id: BatchId::new(CheckpointDraftId::new([3u8; 32])),
            route,
            route_table_digest: PlanDigest::new([4u8; 32]),
            intake_ids: Vec::new(),
            op_count: 1,
            plan_digest: PlanDigest::new([5u8; 32]),
        }
    }
}
