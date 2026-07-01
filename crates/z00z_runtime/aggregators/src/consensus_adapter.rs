#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use z00z_storage::settlement::SettlementStateRoot;

use crate::{
    placement::{AggregatorId, ShardPlacementTable},
    recovery::ShardRecoveryRecord,
    types::{BatchId, BatchRoute, RejectClass, RejectRecord},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalCandidate {
    pub batch_id: BatchId,
    pub route: BatchRoute,
    pub state_root: SettlementStateRoot,
    pub journal_lineage: [u8; 32],
    pub version: u64,
    pub root_generation: u8,
    pub proof_version: u16,
    pub bucket_policy_generation: u32,
    pub bucket_policy_id: [u8; 32],
}

impl JournalCandidate {
    pub fn from_record(record: &ShardRecoveryRecord) -> Result<Self, RejectRecord> {
        let recovery = &record.recovery;
        if recovery.version != 0 {
            let Some(route) = recovery.route else {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "candidate recovery route is missing",
                ));
            };
            if route.shard_id() != record.placement.route.shard_id.as_u32()
                || route.routing_generation() != record.placement.route.routing_generation
            {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "candidate recovery route drifted from shard placement",
                ));
            }
            if route.batch_id() != record.batch_id.into_bytes() {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "candidate recovery batch id drifted from the recovery record",
                ));
            }
        }

        Ok(Self {
            batch_id: record.batch_id,
            route: record.placement.route,
            state_root: recovery.state_root,
            journal_lineage: recovery.journal_lineage,
            version: recovery.version,
            root_generation: recovery.root_generation,
            proof_version: recovery.proof_version,
            bucket_policy_generation: recovery.bucket_policy_generation,
            bucket_policy_id: recovery.bucket_policy_id,
        })
    }

    #[must_use]
    pub fn conflicts_with(&self, other: &Self) -> bool {
        self.route == other.route
            && (self.batch_id != other.batch_id
                || self.state_root != other.state_root
                || self.journal_lineage != other.journal_lineage
                || self.version != other.version
                || self.root_generation != other.root_generation
                || self.proof_version != other.proof_version
                || self.bucket_policy_generation != other.bucket_policy_generation
                || self.bucket_policy_id != other.bucket_policy_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsensusCommit {
    pub term: u64,
    pub batch_id: BatchId,
    pub route: BatchRoute,
    pub state_root: SettlementStateRoot,
    pub journal_lineage: [u8; 32],
    pub voter_ids: Vec<AggregatorId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipChange {
    Join,
    Leave,
    Decommission,
    Rejoin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsensusAdapter {
    route: BatchRoute,
    term: u64,
    active_ids: BTreeSet<AggregatorId>,
    retired_ids: BTreeSet<AggregatorId>,
    frozen_term: Option<u64>,
    committed: Option<ConsensusCommit>,
}

impl ConsensusAdapter {
    pub fn new(
        route: BatchRoute,
        active_ids: impl IntoIterator<Item = AggregatorId>,
    ) -> Result<Self, RejectRecord> {
        let active_ids = active_ids.into_iter().collect::<BTreeSet<_>>();
        if active_ids.is_empty() {
            return Err(reject(
                RejectClass::PolicyReject,
                "consensus member set must not be empty",
            ));
        }

        Ok(Self {
            route,
            term: 0,
            active_ids,
            retired_ids: BTreeSet::new(),
            frozen_term: None,
            committed: None,
        })
    }

    #[must_use]
    pub const fn route(&self) -> BatchRoute {
        self.route
    }

    #[must_use]
    pub fn active_ids(&self) -> Vec<AggregatorId> {
        self.active_ids.iter().copied().collect()
    }

    #[must_use]
    pub fn committed(&self) -> Option<&ConsensusCommit> {
        self.committed.as_ref()
    }

    pub fn bind_placement(
        &self,
        placement_table: &ShardPlacementTable,
    ) -> Result<(), RejectRecord> {
        let Some(placement) = placement_table.placement(self.route) else {
            return Err(reject(
                RejectClass::PolicyReject,
                "placement route missing for consensus",
            ));
        };

        let mut expected = BTreeSet::new();
        expected.insert(placement.primary_id);
        expected.extend(
            placement
                .standby
                .iter()
                .map(|standby| standby.aggregator_id),
        );
        if expected != self.active_ids {
            return Err(reject(
                RejectClass::PolicyReject,
                "membership drift: consensus member set does not match shard placement",
            ));
        }
        Ok(())
    }

    pub fn commit(
        &mut self,
        term: u64,
        candidate: &JournalCandidate,
        votes: &[AggregatorId],
    ) -> Result<ConsensusCommit, RejectRecord> {
        if candidate.route != self.route {
            return Err(reject(
                RejectClass::PolicyReject,
                "wrong generation: consensus candidate route drifted",
            ));
        }
        if term < self.term {
            return Err(reject(
                RejectClass::PolicyReject,
                "stale term: consensus term regressed",
            ));
        }
        if self.frozen_term == Some(term) {
            return Err(reject(
                RejectClass::PolicyReject,
                "split-brain: same-term quorum is frozen after a divergent root",
            ));
        }

        let voter_ids = self.unique_votes(votes)?;
        if voter_ids.len() < self.quorum_count() {
            return Err(reject(
                RejectClass::DeferredRetry,
                "no quorum: same-shard root does not have a majority term",
            ));
        }

        if let Some(committed) = &self.committed {
            if committed.term == term {
                let current = JournalCandidate {
                    batch_id: committed.batch_id,
                    route: committed.route,
                    state_root: committed.state_root,
                    journal_lineage: committed.journal_lineage,
                    version: candidate.version,
                    root_generation: candidate.root_generation,
                    proof_version: candidate.proof_version,
                    bucket_policy_generation: candidate.bucket_policy_generation,
                    bucket_policy_id: candidate.bucket_policy_id,
                };
                if current.conflicts_with(candidate) {
                    self.frozen_term = Some(term);
                    return Err(reject(
                        RejectClass::PolicyReject,
                        "split-brain: divergent root reached the same quorum term",
                    ));
                }
                return Ok(committed.clone());
            }
        }

        self.term = term;
        self.frozen_term = None;
        let commit = ConsensusCommit {
            term,
            batch_id: candidate.batch_id,
            route: candidate.route,
            state_root: candidate.state_root,
            journal_lineage: candidate.journal_lineage,
            voter_ids,
        };
        self.committed = Some(commit.clone());
        Ok(commit)
    }

    pub fn apply_change(
        &mut self,
        change: MembershipChange,
        member: AggregatorId,
        routing_generation: u64,
    ) -> Result<(), RejectRecord> {
        if routing_generation < self.route.routing_generation {
            return Err(reject(
                RejectClass::PolicyReject,
                "stale member: membership change routing_generation regressed",
            ));
        }

        match change {
            MembershipChange::Join => {
                if self.active_ids.contains(&member) {
                    return Err(reject(
                        RejectClass::PolicyReject,
                        "member already active in the consensus set",
                    ));
                }
                if self.retired_ids.contains(&member) {
                    return Err(reject(
                        RejectClass::PolicyReject,
                        "generation bound: retired member must use rejoin on a newer routing_generation",
                    ));
                }
                self.bump_generation(routing_generation);
                self.active_ids.insert(member);
            }
            MembershipChange::Leave => {
                if !self.active_ids.contains(&member) {
                    return Err(reject(
                        RejectClass::PolicyReject,
                        "stale member: cannot remove a non-member from the consensus set",
                    ));
                }
                if self.active_ids.len() == 1 {
                    return Err(reject(
                        RejectClass::PolicyReject,
                        "membership change would remove the last active consensus member",
                    ));
                }
                self.bump_generation(routing_generation);
                self.active_ids.remove(&member);
                self.retired_ids.remove(&member);
            }
            MembershipChange::Decommission => {
                if !self.active_ids.contains(&member) {
                    return Err(reject(
                        RejectClass::PolicyReject,
                        "stale member: cannot decommission a non-member",
                    ));
                }
                if self.active_ids.len() == 1 {
                    return Err(reject(
                        RejectClass::PolicyReject,
                        "membership change would remove the last active consensus member",
                    ));
                }
                self.bump_generation(routing_generation);
                self.active_ids.remove(&member);
                self.retired_ids.insert(member);
            }
            MembershipChange::Rejoin => {
                if self.active_ids.contains(&member) {
                    return Err(reject(
                        RejectClass::PolicyReject,
                        "member already active in the consensus set",
                    ));
                }
                if !self.retired_ids.contains(&member) {
                    return Err(reject(
                        RejectClass::PolicyReject,
                        "stale member: only a retired member can rejoin",
                    ));
                }
                if routing_generation <= self.route.routing_generation {
                    return Err(reject(
                        RejectClass::PolicyReject,
                        "generation bound: retired member must rejoin on a newer routing_generation",
                    ));
                }
                self.bump_generation(routing_generation);
                self.retired_ids.remove(&member);
                self.active_ids.insert(member);
            }
        }
        Ok(())
    }

    fn bump_generation(&mut self, routing_generation: u64) {
        if routing_generation > self.route.routing_generation {
            self.route.routing_generation = routing_generation;
            self.term = 0;
            self.frozen_term = None;
            self.committed = None;
        }
    }

    fn quorum_count(&self) -> usize {
        (self.active_ids.len() / 2) + 1
    }

    fn unique_votes(&self, votes: &[AggregatorId]) -> Result<Vec<AggregatorId>, RejectRecord> {
        let mut unique = BTreeSet::new();
        for voter in votes {
            if !self.active_ids.contains(voter) {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "stale member: quorum vote referenced a non-member",
                ));
            }
            unique.insert(*voter);
        }
        Ok(unique.into_iter().collect())
    }
}

fn reject(class: RejectClass, detail: &str) -> RejectRecord {
    RejectRecord {
        intake_id: None,
        class,
        detail: detail.to_string(),
    }
}
