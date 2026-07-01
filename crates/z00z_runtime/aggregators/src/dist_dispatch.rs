#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    batch_planner::{BatchPlanner, RouteErr, ShardRouteTable},
    placement::{AggregatorId, ShardPlacementTable},
    types::{BatchId, BatchPlanned, BatchRoute, PlanDigest, RejectClass, RejectRecord, WorkItem},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistLevel {
    Info,
    Warn,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistNoteKind {
    RouteRollout,
    SchedulerWave,
    ShardStall,
    ShardFreeze,
    DispatchDispute,
    RouteDrift,
    FailoverState,
    StorageLockHazard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistNote {
    pub kind: DistNoteKind,
    pub level: DistLevel,
    pub batch_id: Option<BatchId>,
    pub route: Option<BatchRoute>,
    pub aggregator_id: Option<AggregatorId>,
    pub detail: String,
    pub proof_truth: bool,
}

impl DistNote {
    #[must_use]
    pub fn new(kind: DistNoteKind, level: DistLevel, detail: impl Into<String>) -> Self {
        Self {
            kind,
            level,
            batch_id: None,
            route: None,
            aggregator_id: None,
            detail: detail.into(),
            proof_truth: false,
        }
    }

    #[must_use]
    pub fn with_batch(mut self, batch_id: BatchId) -> Self {
        self.batch_id = Some(batch_id);
        self
    }

    #[must_use]
    pub fn with_route(mut self, route: BatchRoute) -> Self {
        self.route = Some(route);
        self
    }

    #[must_use]
    pub fn with_owner(mut self, aggregator_id: AggregatorId) -> Self {
        self.aggregator_id = Some(aggregator_id);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchStage {
    Delivered,
    Deferred,
    Duplicate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchVerdict {
    pub batch_id: BatchId,
    pub route: BatchRoute,
    pub owner_id: AggregatorId,
    pub stage: DispatchStage,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteRollout {
    active: ShardRouteTable,
    pending: Option<PendingRollout>,
    notes: Vec<DistNote>,
}

impl RouteRollout {
    #[must_use]
    pub fn new(active: ShardRouteTable) -> Self {
        Self {
            active,
            pending: None,
            notes: Vec::new(),
        }
    }

    #[must_use]
    pub fn active(&self) -> &ShardRouteTable {
        &self.active
    }

    #[must_use]
    pub fn pending_digest(&self) -> Option<PlanDigest> {
        self.pending.as_ref().map(|pending| pending.next.digest())
    }

    #[must_use]
    pub fn notes(&self) -> &[DistNote] {
        &self.notes
    }

    #[must_use]
    pub fn take_notes(&mut self) -> Vec<DistNote> {
        std::mem::take(&mut self.notes)
    }

    pub fn stage(
        &mut self,
        next: ShardRouteTable,
        member_ids: impl IntoIterator<Item = AggregatorId>,
    ) -> Result<(), RejectRecord> {
        if self.pending.is_some() {
            return Err(reject(
                RejectClass::DeferredRetry,
                "route rollout already has a pending staged generation",
            ));
        }
        next.validate_migration(&self.active)
            .map_err(route_reject)?;

        let required_ids = member_ids.into_iter().collect::<BTreeSet<_>>();
        if required_ids.is_empty() {
            return Err(reject(
                RejectClass::PolicyReject,
                "route rollout requires at least one process acknowledgement target",
            ));
        }

        self.notes.push(
            DistNote::new(
                DistNoteKind::RouteRollout,
                DistLevel::Info,
                format!(
                    "staged route rollout generation {} at checkpoint {}",
                    next.routing_generation, next.activation_checkpoint
                ),
            )
            .with_route(BatchRoute {
                shard_id: next.shard_set[0],
                routing_generation: next.routing_generation,
            }),
        );

        self.pending = Some(PendingRollout {
            next,
            required_ids,
            acked_ids: BTreeSet::new(),
        });
        Ok(())
    }

    pub fn ack(
        &mut self,
        aggregator_id: AggregatorId,
        route_digest: PlanDigest,
        routing_generation: u64,
    ) -> Result<(), RejectRecord> {
        let Some(pending) = self.pending.as_mut() else {
            return Err(reject(
                RejectClass::PolicyReject,
                "stale ack: no staged route rollout exists",
            ));
        };
        if !pending.required_ids.contains(&aggregator_id) {
            return Err(reject(
                RejectClass::PolicyReject,
                "late joiner: process is outside the staged rollout set",
            ));
        }
        if routing_generation != pending.next.routing_generation {
            return Err(reject(
                RejectClass::PolicyReject,
                "wrong generation: route rollout ack drifted from the staged table",
            ));
        }
        if route_digest != pending.next.digest() {
            return Err(reject(
                RejectClass::PolicyReject,
                "stale digest: route rollout ack does not match the staged table digest",
            ));
        }

        pending.acked_ids.insert(aggregator_id);
        self.notes.push(
            DistNote::new(
                DistNoteKind::RouteRollout,
                DistLevel::Info,
                "route rollout process acknowledged the staged generation",
            )
            .with_owner(aggregator_id)
            .with_route(BatchRoute {
                shard_id: pending.next.shard_set[0],
                routing_generation: pending.next.routing_generation,
            }),
        );
        Ok(())
    }

    pub fn activate(&mut self, checkpoint: u64) -> Result<PlanDigest, RejectRecord> {
        let Some(pending) = self.pending.take() else {
            return Err(reject(
                RejectClass::PolicyReject,
                "route activation requires a staged route table",
            ));
        };
        if checkpoint < pending.next.activation_checkpoint {
            self.pending = Some(pending);
            return Err(reject(
                RejectClass::PolicyReject,
                "stale checkpoint: route activation checkpoint has not been reached",
            ));
        }
        if pending.acked_ids != pending.required_ids {
            self.pending = Some(pending);
            return Err(reject(
                RejectClass::PolicyReject,
                "missing ack: route activation requires checkpoint and process acknowledgement evidence",
            ));
        }

        let digest = pending.next.digest();
        self.notes.push(
            DistNote::new(
                DistNoteKind::RouteRollout,
                DistLevel::Info,
                format!(
                    "activated route rollout generation {} at checkpoint {}",
                    pending.next.routing_generation, checkpoint
                ),
            )
            .with_route(BatchRoute {
                shard_id: pending.next.shard_set[0],
                routing_generation: pending.next.routing_generation,
            }),
        );
        self.active = pending.next;
        Ok(digest)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistDispatch {
    route_table: ShardRouteTable,
    placement_table: ShardPlacementTable,
    planner: BatchPlanner,
    online_ids: BTreeSet<AggregatorId>,
    owner_epoch: BTreeMap<AggregatorId, u64>,
    owner_seq: BTreeMap<AggregatorId, u64>,
    locks: LockTable,
    notes: Vec<DistNote>,
}

impl DistDispatch {
    pub fn new(
        route_table: ShardRouteTable,
        placement_table: ShardPlacementTable,
    ) -> Result<Self, RejectRecord> {
        route_table.validate().map_err(route_reject)?;
        let mut owner_ids = BTreeSet::new();
        let mut owner_epoch = BTreeMap::new();
        let mut owner_seq = BTreeMap::new();
        for placement in placement_table.placements() {
            if placement.route.routing_generation != route_table.routing_generation {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "placement route missing for dispatch generation",
                ));
            }
            owner_ids.insert(placement.primary_id);
            owner_epoch.entry(placement.primary_id).or_insert(1);
            owner_seq.entry(placement.primary_id).or_insert(1);
        }
        if owner_ids.is_empty() {
            return Err(reject(
                RejectClass::PolicyReject,
                "distributed dispatch requires shard-owned primary workers",
            ));
        }

        Ok(Self {
            planner: BatchPlanner::new(route_table.clone()),
            route_table,
            placement_table,
            online_ids: owner_ids,
            owner_epoch,
            owner_seq,
            locks: LockTable::default(),
            notes: Vec::new(),
        })
    }

    #[must_use]
    pub fn planner(&self) -> &BatchPlanner {
        &self.planner
    }

    #[must_use]
    pub fn notes(&self) -> &[DistNote] {
        &self.notes
    }

    #[must_use]
    pub fn take_notes(&mut self) -> Vec<DistNote> {
        std::mem::take(&mut self.notes)
    }

    pub fn partition(&mut self, owner_id: AggregatorId) -> Result<(), RejectRecord> {
        if !self.owner_epoch.contains_key(&owner_id) {
            return Err(reject(
                RejectClass::PolicyReject,
                "dispatch owner is not part of the shard placement",
            ));
        }
        self.online_ids.remove(&owner_id);
        self.notes.push(
            DistNote::new(
                DistNoteKind::ShardStall,
                DistLevel::Warn,
                "owner was partitioned from the local dispatch network",
            )
            .with_owner(owner_id),
        );
        Ok(())
    }

    pub fn heal(&mut self, owner_id: AggregatorId) -> Result<(), RejectRecord> {
        if !self.owner_epoch.contains_key(&owner_id) {
            return Err(reject(
                RejectClass::PolicyReject,
                "dispatch owner is not part of the shard placement",
            ));
        }
        self.online_ids.insert(owner_id);
        self.notes.push(
            DistNote::new(
                DistNoteKind::FailoverState,
                DistLevel::Info,
                "owner healed back into the local dispatch network",
            )
            .with_owner(owner_id),
        );
        Ok(())
    }

    pub fn restart(
        &mut self,
        owner_id: AggregatorId,
        process_epoch: u64,
    ) -> Result<(), RejectRecord> {
        let Some(current_epoch) = self.owner_epoch.get_mut(&owner_id) else {
            return Err(reject(
                RejectClass::PolicyReject,
                "dispatch owner is not part of the shard placement",
            ));
        };
        if process_epoch <= *current_epoch {
            return Err(reject(
                RejectClass::PolicyReject,
                "stale owner: restart epoch must advance monotonically",
            ));
        }

        *current_epoch = process_epoch;
        self.owner_seq.insert(owner_id, 1);
        self.locks.release_owner(owner_id);
        self.notes.push(
            DistNote::new(
                DistNoteKind::FailoverState,
                DistLevel::Info,
                "owner restart registered and sequence state reset",
            )
            .with_owner(owner_id),
        );
        Ok(())
    }

    pub fn dispatch_batch(
        &mut self,
        batch_id: BatchId,
        items: &[WorkItem],
        owner_id: AggregatorId,
        delivery_seq: u64,
        process_epoch: u64,
        lock_path: impl Into<String>,
    ) -> Result<DispatchVerdict, RejectRecord> {
        let planned = self.planner.plan_batch(batch_id, items)?;
        self.dispatch_planned(planned, owner_id, delivery_seq, process_epoch, lock_path)
    }

    pub fn dispatch_planned(
        &mut self,
        planned: BatchPlanned,
        owner_id: AggregatorId,
        delivery_seq: u64,
        process_epoch: u64,
        lock_path: impl Into<String>,
    ) -> Result<DispatchVerdict, RejectRecord> {
        let Some(placement) = self.placement_table.placement(planned.route) else {
            return Err(reject(
                RejectClass::PolicyReject,
                "placement route missing for dispatch",
            ));
        };
        if placement.primary_id != owner_id {
            let detail =
                "wrong owner: planner decisions must be delivered to the owning aggregator";
            self.notes.push(
                DistNote::new(DistNoteKind::DispatchDispute, DistLevel::Critical, detail)
                    .with_batch(planned.batch_id)
                    .with_route(planned.route)
                    .with_owner(owner_id),
            );
            return Err(reject(RejectClass::PolicyReject, detail));
        }
        if planned.route.routing_generation != self.route_table.routing_generation {
            let detail = "wrong generation: dispatch route drifted from the live table";
            self.notes.push(
                DistNote::new(DistNoteKind::RouteDrift, DistLevel::Critical, detail)
                    .with_batch(planned.batch_id)
                    .with_route(planned.route)
                    .with_owner(owner_id),
            );
            return Err(reject(RejectClass::PolicyReject, detail));
        }
        if planned.route_table_digest != self.route_table.digest() {
            let detail = "wrong route digest: dispatch route drifted from the live table digest";
            self.notes.push(
                DistNote::new(DistNoteKind::RouteDrift, DistLevel::Critical, detail)
                    .with_batch(planned.batch_id)
                    .with_route(planned.route)
                    .with_owner(owner_id),
            );
            return Err(reject(RejectClass::PolicyReject, detail));
        }
        if !self.online_ids.contains(&owner_id) {
            let detail = "owner unavailable: dispatch waits until the shard owner is reachable";
            self.notes.push(
                DistNote::new(DistNoteKind::ShardStall, DistLevel::Warn, detail)
                    .with_batch(planned.batch_id)
                    .with_route(planned.route)
                    .with_owner(owner_id),
            );
            return Ok(DispatchVerdict {
                batch_id: planned.batch_id,
                route: planned.route,
                owner_id,
                stage: DispatchStage::Deferred,
                detail: detail.to_string(),
            });
        }

        let current_epoch = self.owner_epoch.get(&owner_id).copied().ok_or_else(|| {
            reject(
                RejectClass::PolicyReject,
                "dispatch owner is not registered",
            )
        })?;
        if process_epoch < current_epoch {
            let detail = "stale owner: dispatch referenced an older process epoch";
            self.notes.push(
                DistNote::new(DistNoteKind::FailoverState, DistLevel::Critical, detail)
                    .with_batch(planned.batch_id)
                    .with_route(planned.route)
                    .with_owner(owner_id),
            );
            return Err(reject(RejectClass::PolicyReject, detail));
        }
        if process_epoch > current_epoch {
            let detail = "duplicate process: restart must be registered before dispatch resumes";
            self.notes.push(
                DistNote::new(DistNoteKind::FailoverState, DistLevel::Critical, detail)
                    .with_batch(planned.batch_id)
                    .with_route(planned.route)
                    .with_owner(owner_id),
            );
            return Err(reject(RejectClass::PolicyReject, detail));
        }

        let expected_seq = self.owner_seq.get(&owner_id).copied().unwrap_or(1);
        if delivery_seq < expected_seq {
            let detail = "duplicate delivery: dispatch frame was already applied";
            self.notes.push(
                DistNote::new(DistNoteKind::DispatchDispute, DistLevel::Warn, detail)
                    .with_batch(planned.batch_id)
                    .with_route(planned.route)
                    .with_owner(owner_id),
            );
            return Ok(DispatchVerdict {
                batch_id: planned.batch_id,
                route: planned.route,
                owner_id,
                stage: DispatchStage::Duplicate,
                detail: detail.to_string(),
            });
        }
        if delivery_seq > expected_seq {
            let detail = "reorder: dispatch frame arrived ahead of the expected owner sequence";
            self.notes.push(
                DistNote::new(DistNoteKind::DispatchDispute, DistLevel::Warn, detail)
                    .with_batch(planned.batch_id)
                    .with_route(planned.route)
                    .with_owner(owner_id),
            );
            return Ok(DispatchVerdict {
                batch_id: planned.batch_id,
                route: planned.route,
                owner_id,
                stage: DispatchStage::Deferred,
                detail: detail.to_string(),
            });
        }

        let lock_path = lock_path.into();
        if let Err(err) = self
            .locks
            .acquire(&lock_path, planned.route, owner_id, process_epoch)
        {
            self.notes.push(
                DistNote::new(
                    DistNoteKind::StorageLockHazard,
                    DistLevel::Critical,
                    &err.detail,
                )
                .with_batch(planned.batch_id)
                .with_route(planned.route)
                .with_owner(owner_id),
            );
            return Err(err);
        }

        self.owner_seq
            .insert(owner_id, expected_seq.saturating_add(1));
        self.notes.push(
            DistNote::new(
                DistNoteKind::FailoverState,
                DistLevel::Info,
                "dispatch frame applied to the owning shard worker",
            )
            .with_batch(planned.batch_id)
            .with_route(planned.route)
            .with_owner(owner_id),
        );

        Ok(DispatchVerdict {
            batch_id: planned.batch_id,
            route: planned.route,
            owner_id,
            stage: DispatchStage::Delivered,
            detail: "dispatch frame delivered to the owning aggregator".to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingRollout {
    next: ShardRouteTable,
    required_ids: BTreeSet<AggregatorId>,
    acked_ids: BTreeSet<AggregatorId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LockLease {
    route: BatchRoute,
    owner_id: AggregatorId,
    process_epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct LockTable {
    leases: BTreeMap<String, LockLease>,
}

impl LockTable {
    fn acquire(
        &mut self,
        lock_path: &str,
        route: BatchRoute,
        owner_id: AggregatorId,
        process_epoch: u64,
    ) -> Result<(), RejectRecord> {
        let Some(current) = self.leases.get(lock_path) else {
            self.leases.insert(
                lock_path.to_string(),
                LockLease {
                    route,
                    owner_id,
                    process_epoch,
                },
            );
            return Ok(());
        };

        if current.owner_id == owner_id
            && current.process_epoch == process_epoch
            && current.route == route
        {
            return Ok(());
        }
        if current.process_epoch > process_epoch {
            return Err(reject(
                RejectClass::PolicyReject,
                "stale owner: storage lock already belongs to a newer process epoch",
            ));
        }
        if current.owner_id != owner_id {
            return Err(reject(
                RejectClass::PolicyReject,
                "concurrent writer: storage lock already belongs to another shard owner",
            ));
        }
        if current.route.shard_id != route.shard_id {
            return Err(reject(
                RejectClass::PolicyReject,
                "shared root hazard: one storage lock path must not multiplex multiple shards",
            ));
        }
        if current.process_epoch != process_epoch {
            return Err(reject(
                RejectClass::PolicyReject,
                "duplicate process: storage lock still belongs to a different process epoch",
            ));
        }

        Err(reject(
            RejectClass::PolicyReject,
            "storage lock hazard: unexpected competing writer state",
        ))
    }

    fn release_owner(&mut self, owner_id: AggregatorId) {
        self.leases.retain(|_, lease| lease.owner_id != owner_id);
    }
}

fn route_reject(err: RouteErr) -> RejectRecord {
    reject(
        RejectClass::PolicyReject,
        format!("route table contract violation: {err}"),
    )
}

fn reject(class: RejectClass, detail: impl Into<String>) -> RejectRecord {
    RejectRecord {
        intake_id: None,
        class,
        detail: detail.into(),
    }
}
