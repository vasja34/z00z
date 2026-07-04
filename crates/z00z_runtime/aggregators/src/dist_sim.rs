#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use z00z_storage::settlement::SettlementRecoveryState;

use crate::{
    placement::{AggregatorId, ShardPlacementTable},
    recovery::{RecoveryBoundary, RecoveryIntent, ShardRecoveryRecord},
    shard_exec::ShardExecTicket,
    types::{BatchRoute, RejectClass, RejectRecord},
    JournalCandidate,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalFrame {
    pub from_id: AggregatorId,
    pub to_id: AggregatorId,
    pub term: u64,
    pub record: ShardRecoveryRecord,
}

impl JournalFrame {
    #[must_use]
    pub fn new(
        from_id: AggregatorId,
        to_id: AggregatorId,
        term: u64,
        record: ShardRecoveryRecord,
    ) -> Self {
        Self {
            from_id,
            to_id,
            term,
            record,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameStage {
    Applied,
    Deferred,
    ReplayIgnored,
    Dropped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameVerdict {
    pub from_id: AggregatorId,
    pub to_id: AggregatorId,
    pub term: u64,
    pub stage: FrameStage,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistNode {
    aggregator_id: AggregatorId,
    is_online: bool,
    current: Option<SettlementRecoveryState>,
    record: Option<ShardRecoveryRecord>,
    seen_keys: BTreeSet<FrameKey>,
    applied_count: usize,
}

impl DistNode {
    #[must_use]
    pub const fn aggregator_id(&self) -> AggregatorId {
        self.aggregator_id
    }

    #[must_use]
    pub const fn is_online(&self) -> bool {
        self.is_online
    }

    #[must_use]
    pub fn current(&self) -> Option<&SettlementRecoveryState> {
        self.current.as_ref()
    }

    #[must_use]
    pub fn record(&self) -> Option<&ShardRecoveryRecord> {
        self.record.as_ref()
    }

    #[must_use]
    pub const fn applied_count(&self) -> usize {
        self.applied_count
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistSim {
    route: BatchRoute,
    member_ids: BTreeSet<AggregatorId>,
    nodes: BTreeMap<AggregatorId, DistNode>,
    queue: VecDeque<QueuedFrame>,
    tick: u64,
}

impl DistSim {
    pub fn new(
        route: BatchRoute,
        member_ids: impl IntoIterator<Item = AggregatorId>,
    ) -> Result<Self, RejectRecord> {
        let member_ids = member_ids.into_iter().collect::<BTreeSet<_>>();
        if member_ids.is_empty() {
            return Err(reject(
                RejectClass::PolicyReject,
                "distributed simulator member set must not be empty",
            ));
        }

        let nodes = member_ids
            .iter()
            .copied()
            .map(|aggregator_id| {
                (
                    aggregator_id,
                    DistNode {
                        aggregator_id,
                        is_online: true,
                        current: None,
                        record: None,
                        seen_keys: BTreeSet::new(),
                        applied_count: 0,
                    },
                )
            })
            .collect();

        Ok(Self {
            route,
            member_ids,
            nodes,
            queue: VecDeque::new(),
            tick: 0,
        })
    }

    pub fn seed(
        &mut self,
        aggregator_id: AggregatorId,
        record: ShardRecoveryRecord,
    ) -> Result<(), RejectRecord> {
        JournalCandidate::from_record(&record)?;
        if record.placement.route != self.route {
            return Err(reject(
                RejectClass::PolicyReject,
                "wrong generation: seeded recovery record drifted from the simulator route",
            ));
        }
        let Some(node) = self.nodes.get_mut(&aggregator_id) else {
            return Err(reject(
                RejectClass::PolicyReject,
                "seed aggregator is not part of the distributed simulator",
            ));
        };
        let key = FrameKey::from_record(&record);
        node.current = Some(record.recovery.clone());
        node.record = Some(record);
        node.seen_keys.insert(key);
        node.applied_count = 1;
        Ok(())
    }

    #[must_use]
    pub fn node(&self, aggregator_id: AggregatorId) -> Option<&DistNode> {
        self.nodes.get(&aggregator_id)
    }

    pub fn partition(&mut self, aggregator_id: AggregatorId) -> Result<(), RejectRecord> {
        let Some(node) = self.nodes.get_mut(&aggregator_id) else {
            return Err(reject(
                RejectClass::PolicyReject,
                "partition target is not part of the distributed simulator",
            ));
        };
        node.is_online = false;
        Ok(())
    }

    pub fn heal(&mut self, aggregator_id: AggregatorId) -> Result<(), RejectRecord> {
        let Some(node) = self.nodes.get_mut(&aggregator_id) else {
            return Err(reject(
                RejectClass::PolicyReject,
                "heal target is not part of the distributed simulator",
            ));
        };
        node.is_online = true;
        Ok(())
    }

    pub fn enqueue(&mut self, frame: JournalFrame) {
        self.queue.push_back(QueuedFrame::now(frame, self.tick));
    }

    pub fn enqueue_front(&mut self, frame: JournalFrame) {
        self.queue.push_front(QueuedFrame::now(frame, self.tick));
    }

    pub fn enqueue_delayed(&mut self, frame: JournalFrame, delay: u64) {
        self.queue
            .push_back(QueuedFrame::delayed(frame, self.tick.saturating_add(delay)));
    }

    pub fn enqueue_replay(&mut self, frame: JournalFrame, delay: u64) {
        self.queue.push_back(QueuedFrame {
            frame,
            deliver_at: self.tick.saturating_add(delay),
            replay_once: true,
        });
    }

    #[must_use]
    pub fn drop_next(&mut self) -> Option<JournalFrame> {
        self.queue.pop_front().map(|queued| queued.frame)
    }

    pub fn step(&mut self) -> Vec<FrameVerdict> {
        self.tick = self.tick.saturating_add(1);
        let mut verdicts = Vec::new();
        let mut ready = 0usize;
        for queued in &self.queue {
            if queued.deliver_at <= self.tick {
                ready += 1;
            } else {
                break;
            }
        }

        for _ in 0..ready {
            let queued = self.queue.pop_front().expect("ready frame");
            match self.apply_frame(&queued.frame) {
                Ok(verdict) => {
                    if queued.replay_once && verdict.stage == FrameStage::Applied {
                        self.queue.push_back(QueuedFrame::delayed(
                            queued.frame.clone(),
                            self.tick.saturating_add(1),
                        ));
                    }
                    verdicts.push(verdict);
                }
                Err(err) if err.class == RejectClass::DeferredRetry => {
                    self.queue.push_back(QueuedFrame::delayed(
                        queued.frame.clone(),
                        self.tick.saturating_add(1),
                    ));
                    verdicts.push(FrameVerdict {
                        from_id: queued.frame.from_id,
                        to_id: queued.frame.to_id,
                        term: queued.frame.term,
                        stage: FrameStage::Deferred,
                        detail: err.detail,
                    });
                }
                Err(err) => verdicts.push(FrameVerdict {
                    from_id: queued.frame.from_id,
                    to_id: queued.frame.to_id,
                    term: queued.frame.term,
                    stage: FrameStage::Dropped,
                    detail: err.detail,
                }),
            }
        }

        verdicts
    }

    pub fn sync_verdict(
        &self,
        aggregator_id: AggregatorId,
        latest: &ShardRecoveryRecord,
    ) -> Result<(), RejectRecord> {
        let Some(node) = self.nodes.get(&aggregator_id) else {
            return Err(reject(
                RejectClass::PolicyReject,
                "secondary is not part of the distributed simulator",
            ));
        };
        let Some(record) = &node.record else {
            return Err(reject(
                RejectClass::DeferredRetry,
                "secondary unavailable: missing replicated journal state",
            ));
        };
        let Some(current) = &node.current else {
            return Err(reject(
                RejectClass::DeferredRetry,
                "secondary unavailable: missing replicated recovery state",
            ));
        };
        if record.batch_id != latest.batch_id {
            return Err(reject(
                RejectClass::PolicyReject,
                "partial replay: replicated batch id does not match the latest journal state",
            ));
        }
        if current.journal_lineage != latest.recovery.journal_lineage {
            return Err(reject(
                RejectClass::PolicyReject,
                "wrong lineage: replicated journal lineage drifted",
            ));
        }
        if current.version != latest.recovery.version {
            return Err(reject(
                RejectClass::PolicyReject,
                "stale replay: replicated recovery version drifted",
            ));
        }
        if current.state_root != latest.recovery.state_root {
            return Err(reject(
                RejectClass::PolicyReject,
                "stale local root: replicated state root drifted",
            ));
        }
        if current.root_generation != latest.recovery.root_generation
            || current.proof_version != latest.recovery.proof_version
            || current.bucket_policy_generation != latest.recovery.bucket_policy_generation
            || current.bucket_policy_id != latest.recovery.bucket_policy_id
        {
            return Err(reject(
                RejectClass::PolicyReject,
                "stale replay: replicated backend generation metadata drifted",
            ));
        }
        if current.route != latest.recovery.route {
            return Err(reject(
                RejectClass::PolicyReject,
                "wrong shard: replicated route context drifted",
            ));
        }
        Ok(())
    }

    pub fn resume(
        &self,
        requester: AggregatorId,
        placement_table: &ShardPlacementTable,
        latest: &ShardRecoveryRecord,
        intent: RecoveryIntent,
    ) -> Result<ShardExecTicket, RejectRecord> {
        self.sync_verdict(requester, latest)?;
        let current = self
            .nodes
            .get(&requester)
            .and_then(DistNode::current)
            .ok_or_else(|| {
                reject(
                    RejectClass::DeferredRetry,
                    "secondary unavailable: missing replicated recovery state",
                )
            })?;
        RecoveryBoundary.resume(requester, placement_table, latest, current, intent)
    }

    fn apply_frame(&mut self, frame: &JournalFrame) -> Result<FrameVerdict, RejectRecord> {
        if !self.member_ids.contains(&frame.from_id) || !self.member_ids.contains(&frame.to_id) {
            return Err(reject(
                RejectClass::PolicyReject,
                "membership drift: journal frame referenced a non-member",
            ));
        }

        JournalCandidate::from_record(&frame.record)?;
        if frame.record.placement.route != self.route {
            return Err(reject(
                RejectClass::PolicyReject,
                "wrong generation: journal frame route drifted from the simulator route",
            ));
        }

        let Some(node) = self.nodes.get_mut(&frame.to_id) else {
            return Err(reject(
                RejectClass::PolicyReject,
                "journal frame target is not part of the distributed simulator",
            ));
        };
        if !node.is_online {
            return Err(reject(
                RejectClass::DeferredRetry,
                "partitioned: target aggregator is offline",
            ));
        }

        let key = FrameKey::from_record(&frame.record);
        if !node.seen_keys.insert(key) {
            return Ok(FrameVerdict {
                from_id: frame.from_id,
                to_id: frame.to_id,
                term: frame.term,
                stage: FrameStage::ReplayIgnored,
                detail: "replay ignored: replicated journal state was already applied".to_string(),
            });
        }

        node.current = Some(frame.record.recovery.clone());
        node.record = Some(frame.record.clone());
        node.applied_count = node.applied_count.saturating_add(1);
        Ok(FrameVerdict {
            from_id: frame.from_id,
            to_id: frame.to_id,
            term: frame.term,
            stage: FrameStage::Applied,
            detail: "replicated recovery state applied".to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QueuedFrame {
    frame: JournalFrame,
    deliver_at: u64,
    replay_once: bool,
}

impl QueuedFrame {
    fn now(frame: JournalFrame, tick: u64) -> Self {
        Self::delayed(frame, tick)
    }

    fn delayed(frame: JournalFrame, deliver_at: u64) -> Self {
        Self {
            frame,
            deliver_at,
            replay_once: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FrameKey {
    batch_id: [u8; 32],
    version: u64,
    state_root: [u8; 32],
    journal_lineage: [u8; 32],
}

impl FrameKey {
    fn from_record(record: &ShardRecoveryRecord) -> Self {
        Self {
            batch_id: record.batch_id.into_bytes(),
            version: record.recovery.version,
            state_root: record.recovery.state_root.into_bytes(),
            journal_lineage: record.recovery.journal_lineage,
        }
    }
}

fn reject(class: RejectClass, detail: &str) -> RejectRecord {
    RejectRecord {
        intake_id: None,
        class,
        detail: detail.to_string(),
    }
}
