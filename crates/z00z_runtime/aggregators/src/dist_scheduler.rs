#![forbid(unsafe_code)]

use std::collections::{BTreeMap, VecDeque};

use sha2::{Digest, Sha256};

use crate::{
    batch_planner::{BatchPlanner, RouteErr, ShardRouteTable},
    dist_dispatch::{DistLevel, DistNote, DistNoteKind},
    placement::{AggregatorId, ShardPlacementTable},
    types::{BatchId, BatchPlanned, BatchRoute, RejectClass, RejectRecord, WorkItem},
};

const SCHEDULE_BATCH_LABEL: &[u8] = b"z00z.runtime.dist-scheduler.batch.v1";

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBatch {
    pub owner_id: AggregatorId,
    pub planned: BatchPlanned,
    pub items: Vec<WorkItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchedulerWave {
    pub index: usize,
    pub notes: Vec<DistNote>,
    pub batches: Vec<BatchWave>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchWave {
    pub owner_id: AggregatorId,
    pub planned: BatchPlanned,
    pub items: Vec<WorkItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistScheduler {
    route_table: ShardRouteTable,
    placement_table: ShardPlacementTable,
}

impl DistScheduler {
    #[must_use]
    pub fn new(route_table: ShardRouteTable, placement_table: ShardPlacementTable) -> Self {
        Self {
            route_table,
            placement_table,
        }
    }

    pub fn plan_waves(&self, items: &[WorkItem]) -> Result<Vec<SchedulerWave>, RejectRecord> {
        if items.is_empty() {
            return Ok(Vec::new());
        }
        self.route_table.validate().map_err(route_reject)?;

        let mut groups = BTreeMap::<(AggregatorId, u16), Vec<WorkItem>>::new();
        for item in items {
            let shard_id = self
                .route_table
                .lookup(item.route_key())
                .map_err(|err| route_lookup_reject(item, err))?;
            let route = BatchRoute {
                shard_id,
                routing_generation: self.route_table.routing_generation,
            };
            let Some(placement) = self.placement_table.placement(route) else {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "placement route missing for scheduler",
                ));
            };
            groups
                .entry((placement.primary_id, shard_id.as_u16()))
                .or_default()
                .push(item.clone());
        }

        let planner = BatchPlanner::new(self.route_table.clone());
        let mut owner_batches = BTreeMap::<AggregatorId, VecDeque<ScheduledBatch>>::new();
        for ((owner_id, shard_num), grouped) in groups {
            let shard_id = crate::ShardId::new(shard_num);
            let route = BatchRoute {
                shard_id,
                routing_generation: self.route_table.routing_generation,
            };
            let ordered = planner.make_batch(schedule_batch_id(route, &grouped), &grouped)?;
            owner_batches
                .entry(owner_id)
                .or_default()
                .push_back(ScheduledBatch {
                    owner_id,
                    planned: ordered.planned,
                    items: ordered.items,
                });
        }

        let mut waves = Vec::new();
        let mut wave_index = 0usize;
        while owner_batches.values().any(|queue| !queue.is_empty()) {
            wave_index += 1;
            let mut batches = Vec::new();
            for queue in owner_batches.values_mut() {
                if let Some(batch) = queue.pop_front() {
                    batches.push(BatchWave {
                        owner_id: batch.owner_id,
                        planned: batch.planned,
                        items: batch.items,
                    });
                }
            }

            let detail = format!(
                "scheduler wave {} carries {} shard-owned dispatch batches; throughput claims stay publication-root scoped",
                wave_index,
                batches.len()
            );
            waves.push(SchedulerWave {
                index: wave_index,
                notes: vec![DistNote::new(
                    DistNoteKind::SchedulerWave,
                    DistLevel::Info,
                    detail,
                )],
                batches,
            });
        }

        Ok(waves)
    }
}

fn schedule_batch_id(route: BatchRoute, items: &[WorkItem]) -> BatchId {
    let mut hasher = Sha256::new();
    hasher.update(SCHEDULE_BATCH_LABEL);
    hasher.update(route.shard_id.as_u16().to_be_bytes());
    hasher.update(route.routing_generation.to_be_bytes());
    hasher.update((items.len() as u32).to_be_bytes());
    for item in items {
        hasher.update([item.kind_tag()]);
        hasher.update(item.route_key());
        hasher.update(item.digest_hex().as_bytes());
    }
    BatchId::from_bytes(hasher.finalize().into())
}

fn route_reject(err: RouteErr) -> RejectRecord {
    reject(
        RejectClass::PolicyReject,
        format!("route table contract violation: {err}"),
    )
}

fn route_lookup_reject(item: &WorkItem, err: RouteErr) -> RejectRecord {
    RejectRecord {
        intake_id: Some(item.intake_id().clone()),
        class: RejectClass::PolicyReject,
        detail: format!("route lookup rejected intake {}: {err}", item.digest_hex()),
    }
}

fn reject(class: RejectClass, detail: impl Into<String>) -> RejectRecord {
    RejectRecord {
        intake_id: None,
        class,
        detail: detail.into(),
    }
}
