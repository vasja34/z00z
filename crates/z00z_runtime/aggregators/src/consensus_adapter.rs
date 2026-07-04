#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use z00z_storage::settlement::SettlementStateRoot;

use crate::{
    commit_subject::CommitSubject,
    placement::{AggregatorId, ShardPlacement, ShardPlacementTable},
    shard_quorum_certificate::{membership_digest_for_voters, ShardQuorumCertificate},
    shard_vote::ShardVote,
    types::{BatchId, BatchRoute, RejectClass, RejectRecord},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsensusCommit {
    pub term: u64,
    pub batch_id: BatchId,
    pub route: BatchRoute,
    pub state_root: SettlementStateRoot,
    pub journal_lineage: [u8; 32],
    pub subject: CommitSubject,
    pub certificate: ShardQuorumCertificate,
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
    primary_id: AggregatorId,
    active_secondary_ids: BTreeSet<AggregatorId>,
    retired_ids: BTreeSet<AggregatorId>,
    frozen_term: Option<u64>,
    committed: Option<ConsensusCommit>,
}

impl ConsensusAdapter {
    pub fn new(
        route: BatchRoute,
        primary_id: AggregatorId,
        active_secondary_ids: impl IntoIterator<Item = AggregatorId>,
    ) -> Result<Self, RejectRecord> {
        let active_secondary_ids = active_secondary_ids.into_iter().collect::<BTreeSet<_>>();
        if active_secondary_ids.contains(&primary_id) {
            return Err(reject(
                RejectClass::PolicyReject,
                "consensus membership cannot list the primary as a secondary member",
            ));
        }

        Ok(Self {
            route,
            term: 0,
            primary_id,
            active_secondary_ids,
            retired_ids: BTreeSet::new(),
            frozen_term: None,
            committed: None,
        })
    }

    pub fn from_placement(placement: &ShardPlacement) -> Result<Self, RejectRecord> {
        Self::new(
            placement.route,
            placement.primary_id,
            placement
                .secondaries
                .iter()
                .filter(|secondary| secondary.is_ready)
                .map(|secondary| secondary.aggregator_id),
        )
    }

    #[must_use]
    pub const fn route(&self) -> BatchRoute {
        self.route
    }

    #[must_use]
    pub fn active_ids(&self) -> Vec<AggregatorId> {
        std::iter::once(self.primary_id)
            .chain(self.active_secondary_ids.iter().copied())
            .collect()
    }

    #[must_use]
    pub fn membership_digest(&self) -> [u8; 32] {
        membership_digest_for_voters(
            self.route,
            self.primary_id,
            self.active_secondary_ids.iter().copied(),
        )
    }

    #[must_use]
    pub fn committed(&self) -> Option<&ConsensusCommit> {
        self.committed.as_ref()
    }

    pub fn bind_placement(
        &mut self,
        placement_table: &ShardPlacementTable,
    ) -> Result<(), RejectRecord> {
        let Some(placement) = placement_table.placement(self.route) else {
            return Err(reject(
                RejectClass::PolicyReject,
                "placement route missing for consensus",
            ));
        };

        let expected_secondaries = placement
            .secondaries
            .iter()
            .filter(|secondary| secondary.is_ready)
            .map(|secondary| secondary.aggregator_id)
            .collect::<BTreeSet<_>>();
        let current_members = self.active_ids().into_iter().collect::<BTreeSet<_>>();
        let expected_members = std::iter::once(placement.primary_id)
            .chain(expected_secondaries.iter().copied())
            .collect::<BTreeSet<_>>();
        if current_members != expected_members {
            return Err(reject(
                RejectClass::PolicyReject,
                "membership drift: consensus member set does not match ready shard placement members",
            ));
        }
        self.primary_id = placement.primary_id;
        self.active_secondary_ids = expected_secondaries;
        Ok(())
    }

    pub fn commit(
        &mut self,
        subject: &CommitSubject,
        votes: &[ShardVote],
    ) -> Result<ConsensusCommit, RejectRecord> {
        if subject.route() != self.route {
            return Err(reject(
                RejectClass::PolicyReject,
                "wrong generation: consensus subject route drifted",
            ));
        }
        if subject.term < self.term {
            return Err(reject(
                RejectClass::PolicyReject,
                "stale term: consensus term regressed",
            ));
        }
        if subject.membership_digest != self.membership_digest() {
            return Err(reject(
                RejectClass::PolicyReject,
                "membership drift: consensus subject membership digest drifted from active placement members",
            ));
        }
        if self.frozen_term == Some(subject.term) {
            return Err(reject(
                RejectClass::PolicyReject,
                "split-brain: same-term quorum is frozen after a divergent root",
            ));
        }

        let subject_digest = subject.digest();
        if let Some(committed) = &self.committed {
            if committed.term == subject.term {
                if committed.subject.digest() != subject_digest {
                    self.frozen_term = Some(subject.term);
                    return Err(reject(
                        RejectClass::PolicyReject,
                        "split-brain: divergent root reached the same quorum term",
                    ));
                }
                return Ok(committed.clone());
            }
        }

        let certificate = ShardQuorumCertificate::new(
            subject,
            self.primary_id,
            self.active_secondary_ids.iter().copied(),
            votes,
        )?;
        self.term = subject.term;
        self.frozen_term = None;

        let commit = ConsensusCommit {
            term: subject.term,
            batch_id: subject.batch_id,
            route: subject.route(),
            state_root: subject.new_state_root,
            journal_lineage: subject.journal_lineage,
            subject: subject.clone(),
            certificate,
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
        if member == self.primary_id {
            return Err(reject(
                RejectClass::PolicyReject,
                "membership change cannot rewrite the primary role through the secondary membership seam",
            ));
        }

        match change {
            MembershipChange::Join => {
                if self.active_secondary_ids.contains(&member) {
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
                self.active_secondary_ids.insert(member);
            }
            MembershipChange::Leave => {
                if !self.active_secondary_ids.contains(&member) {
                    return Err(reject(
                        RejectClass::PolicyReject,
                        "stale member: cannot remove a non-member from the consensus set",
                    ));
                }
                self.bump_generation(routing_generation);
                self.active_secondary_ids.remove(&member);
                self.retired_ids.remove(&member);
            }
            MembershipChange::Decommission => {
                if !self.active_secondary_ids.contains(&member) {
                    return Err(reject(
                        RejectClass::PolicyReject,
                        "stale member: cannot decommission a non-member",
                    ));
                }
                self.bump_generation(routing_generation);
                self.active_secondary_ids.remove(&member);
                self.retired_ids.insert(member);
            }
            MembershipChange::Rejoin => {
                if self.active_secondary_ids.contains(&member) {
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
                self.active_secondary_ids.insert(member);
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
}

fn reject(class: RejectClass, detail: &str) -> RejectRecord {
    RejectRecord {
        intake_id: None,
        class,
        detail: detail.to_string(),
    }
}
