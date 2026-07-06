#![forbid(unsafe_code)]

use std::collections::{BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};
use z00z_crypto::domains::ShardTransportEnvelopeDomain;

use crate::{
    commit_subject::{digest_bytes, push_bytes32, push_len_prefixed, push_u64, push_u8},
    evidence::{TransportFaultEvidence, TransportFaultEvidenceKind},
    placement::AggregatorId,
    shard_vote::ShardVoteKind,
    CommitSubject,
};

const SHARD_TRANSPORT_TAG: &[u8] = b"z00z.shard_transport";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportPayloadStatus {
    Available,
    Missing { detail: String },
}

impl TransportPayloadStatus {
    fn code(&self) -> u8 {
        match self {
            Self::Available => 1,
            Self::Missing { .. } => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoteTransportEnvelope {
    pub message_id: [u8; 32],
    pub from_id: AggregatorId,
    pub to_id: AggregatorId,
    pub subject: CommitSubject,
    pub vote_kind: ShardVoteKind,
    pub payload_status: TransportPayloadStatus,
}

impl VoteTransportEnvelope {
    #[must_use]
    pub fn available(
        from_id: AggregatorId,
        to_id: AggregatorId,
        subject: CommitSubject,
        vote_kind: ShardVoteKind,
    ) -> Self {
        Self::new(
            from_id,
            to_id,
            subject,
            vote_kind,
            TransportPayloadStatus::Available,
        )
    }

    #[must_use]
    pub fn missing_payload(
        from_id: AggregatorId,
        to_id: AggregatorId,
        subject: CommitSubject,
        vote_kind: ShardVoteKind,
        detail: impl Into<String>,
    ) -> Self {
        Self::new(
            from_id,
            to_id,
            subject,
            vote_kind,
            TransportPayloadStatus::Missing {
                detail: detail.into(),
            },
        )
    }

    #[must_use]
    pub fn new(
        from_id: AggregatorId,
        to_id: AggregatorId,
        subject: CommitSubject,
        vote_kind: ShardVoteKind,
        payload_status: TransportPayloadStatus,
    ) -> Self {
        let message_id = message_id_for(from_id, to_id, &subject, vote_kind, &payload_status);
        Self {
            message_id,
            from_id,
            to_id,
            subject,
            vote_kind,
            payload_status,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TransportDeliveryPlan {
    pub delay: u64,
    pub duplicate_after: Option<u64>,
    pub replay_after: Option<u64>,
    pub drop_on_delivery: bool,
    pub enqueue_front: bool,
}

impl TransportDeliveryPlan {
    #[must_use]
    pub const fn delayed(delay: u64) -> Self {
        Self {
            delay,
            duplicate_after: None,
            replay_after: None,
            drop_on_delivery: false,
            enqueue_front: false,
        }
    }

    #[must_use]
    pub fn with_duplicate_after(mut self, delay: u64) -> Self {
        self.duplicate_after = Some(delay);
        self
    }

    #[must_use]
    pub fn with_replay_after(mut self, delay: u64) -> Self {
        self.replay_after = Some(delay);
        self
    }

    #[must_use]
    pub fn drop_on_delivery(mut self) -> Self {
        self.drop_on_delivery = true;
        self
    }

    #[must_use]
    pub fn front_of_queue(mut self) -> Self {
        self.enqueue_front = true;
        self
    }
}

pub trait VoteTransport {
    fn enqueue(&mut self, envelope: VoteTransportEnvelope);

    fn enqueue_front(&mut self, envelope: VoteTransportEnvelope);

    fn enqueue_delayed(&mut self, envelope: VoteTransportEnvelope, delay: u64);

    fn requeue(&mut self, envelope: VoteTransportEnvelope, delay: u64);

    fn step(&mut self) -> Vec<VoteTransportEnvelope>;

    fn drop_next(&mut self) -> Option<VoteTransportEnvelope>;
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct InMemoryVoteTransport {
    queue: VecDeque<QueuedEnvelope>,
    tick: u64,
    blocked_ids: BTreeSet<AggregatorId>,
    fault_records: Vec<TransportFaultEvidence>,
}

impl InMemoryVoteTransport {
    #[must_use]
    pub const fn tick(&self) -> u64 {
        self.tick
    }

    #[must_use]
    pub fn fault_records(&self) -> &[TransportFaultEvidence] {
        &self.fault_records
    }

    pub fn partition_peer(&mut self, aggregator_id: AggregatorId) {
        if self.blocked_ids.insert(aggregator_id) {
            self.fault_records.push(TransportFaultEvidence::for_peer(
                TransportFaultEvidenceKind::PartitionDeferred,
                self.tick,
                aggregator_id,
                "transport partitioned the peer before delivery",
            ));
        }
    }

    pub fn heal_peer(&mut self, aggregator_id: AggregatorId) {
        self.blocked_ids.remove(&aggregator_id);
        self.fault_records.push(TransportFaultEvidence::for_peer(
            TransportFaultEvidenceKind::Heal,
            self.tick,
            aggregator_id,
            "transport healed the peer and restored delivery eligibility",
        ));
    }

    pub fn restart_peer(&mut self, aggregator_id: AggregatorId) {
        self.fault_records.push(TransportFaultEvidence::for_peer(
            TransportFaultEvidenceKind::Restart,
            self.tick,
            aggregator_id,
            "transport recorded a peer restart event",
        ));
    }

    pub fn reconnect_peer(&mut self, aggregator_id: AggregatorId) {
        self.blocked_ids.remove(&aggregator_id);
        self.fault_records.push(TransportFaultEvidence::for_peer(
            TransportFaultEvidenceKind::Reconnect,
            self.tick,
            aggregator_id,
            "transport recorded a peer reconnect event",
        ));
    }

    pub fn enqueue_planned(
        &mut self,
        envelope: VoteTransportEnvelope,
        plan: TransportDeliveryPlan,
    ) {
        if plan.delay > 0 {
            self.fault_records
                .push(TransportFaultEvidence::for_envelope(
                    TransportFaultEvidenceKind::Delay,
                    self.tick,
                    &envelope,
                    format!("transport delayed delivery by {} ticks", plan.delay),
                ));
        }
        if plan.enqueue_front {
            self.fault_records
                .push(TransportFaultEvidence::for_envelope(
                    TransportFaultEvidenceKind::Reorder,
                    self.tick,
                    &envelope,
                    "transport reordered the envelope to the front of the queue",
                ));
        }

        let queued = QueuedEnvelope::planned(envelope, self.tick.saturating_add(plan.delay), plan);
        if plan.enqueue_front {
            self.queue.push_front(queued);
        } else {
            self.queue.push_back(queued);
        }
    }
}

impl VoteTransport for InMemoryVoteTransport {
    fn enqueue(&mut self, envelope: VoteTransportEnvelope) {
        self.queue
            .push_back(QueuedEnvelope::delayed(envelope, self.tick));
    }

    fn enqueue_front(&mut self, envelope: VoteTransportEnvelope) {
        self.queue
            .push_front(QueuedEnvelope::delayed(envelope, self.tick));
    }

    fn enqueue_delayed(&mut self, envelope: VoteTransportEnvelope, delay: u64) {
        self.queue.push_back(QueuedEnvelope::delayed(
            envelope,
            self.tick.saturating_add(delay),
        ));
    }

    fn requeue(&mut self, envelope: VoteTransportEnvelope, delay: u64) {
        self.enqueue_delayed(envelope, delay);
    }

    fn step(&mut self) -> Vec<VoteTransportEnvelope> {
        self.tick = self.tick.saturating_add(1);
        let mut ready = Vec::new();
        for _ in 0..self.queue.len() {
            let mut queued = self.queue.pop_front().expect("queued transport envelope");
            if queued.deliver_at > self.tick {
                self.queue.push_back(queued);
                continue;
            }

            if queued.plan.drop_on_delivery {
                self.fault_records
                    .push(TransportFaultEvidence::for_envelope(
                        TransportFaultEvidenceKind::Drop,
                        self.tick,
                        &queued.envelope,
                        "transport dropped the envelope before delivery",
                    ));
                continue;
            }

            if self.blocked_ids.contains(&queued.envelope.from_id)
                || self.blocked_ids.contains(&queued.envelope.to_id)
            {
                self.fault_records
                    .push(TransportFaultEvidence::for_envelope(
                        TransportFaultEvidenceKind::PartitionDeferred,
                        self.tick,
                        &queued.envelope,
                        "partitioned peer prevented delivery and forced deterministic retry",
                    ));
                queued.deliver_at = self.tick.saturating_add(1);
                self.queue.push_back(queued);
                continue;
            }

            if let Some(delay) = queued.plan.duplicate_after.take() {
                self.queue.push_back(QueuedEnvelope::delayed(
                    queued.envelope.clone(),
                    self.tick.saturating_add(delay),
                ));
                self.fault_records
                    .push(TransportFaultEvidence::for_envelope(
                        TransportFaultEvidenceKind::Duplicate,
                        self.tick,
                        &queued.envelope,
                        format!(
                        "transport scheduled a duplicate delivery {} ticks after the first send",
                        delay
                    ),
                    ));
            }

            if let Some(delay) = queued.plan.replay_after.take() {
                self.queue.push_back(QueuedEnvelope::delayed(
                    queued.envelope.clone(),
                    self.tick.saturating_add(delay),
                ));
                self.fault_records
                    .push(TransportFaultEvidence::for_envelope(
                        TransportFaultEvidenceKind::Replay,
                        self.tick,
                        &queued.envelope,
                        format!(
                            "transport replayed the envelope {} ticks after the first send",
                            delay
                        ),
                    ));
            }

            ready.push(queued.envelope);
        }
        ready
    }

    fn drop_next(&mut self) -> Option<VoteTransportEnvelope> {
        self.queue.pop_front().map(|queued| {
            self.fault_records
                .push(TransportFaultEvidence::for_envelope(
                    TransportFaultEvidenceKind::Drop,
                    self.tick,
                    &queued.envelope,
                    "transport manually dropped the next queued envelope",
                ));
            queued.envelope
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QueuedEnvelope {
    envelope: VoteTransportEnvelope,
    deliver_at: u64,
    plan: TransportDeliveryPlan,
}

impl QueuedEnvelope {
    fn delayed(envelope: VoteTransportEnvelope, deliver_at: u64) -> Self {
        Self::planned(envelope, deliver_at, TransportDeliveryPlan::default())
    }

    fn planned(
        envelope: VoteTransportEnvelope,
        deliver_at: u64,
        plan: TransportDeliveryPlan,
    ) -> Self {
        Self {
            envelope,
            deliver_at,
            plan,
        }
    }
}

fn message_id_for(
    from_id: AggregatorId,
    to_id: AggregatorId,
    subject: &CommitSubject,
    vote_kind: ShardVoteKind,
    payload_status: &TransportPayloadStatus,
) -> [u8; 32] {
    let mut out = Vec::with_capacity(192);
    out.extend_from_slice(SHARD_TRANSPORT_TAG);
    push_u64(&mut out, u64::from(from_id.as_u16()));
    push_u64(&mut out, u64::from(to_id.as_u16()));
    push_bytes32(&mut out, subject.digest());
    push_u8(
        &mut out,
        match vote_kind {
            ShardVoteKind::Prepare => 1,
            ShardVoteKind::Commit => 2,
            ShardVoteKind::LocalCommit => 3,
        },
    );
    push_u8(&mut out, payload_status.code());
    match payload_status {
        TransportPayloadStatus::Available => {}
        TransportPayloadStatus::Missing { detail } => {
            push_bytes32(&mut out, subject.payload_digest);
            push_len_prefixed(&mut out, detail.as_bytes());
        }
    }
    digest_bytes::<ShardTransportEnvelopeDomain>("message_id", &out)
}
