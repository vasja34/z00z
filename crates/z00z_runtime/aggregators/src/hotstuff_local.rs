#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use z00z_crypto::domains::ShardEvidenceDomain;

use crate::{
    bft_committee::BftCommittee,
    bft_engine::BftCommit,
    commit_subject::{
        digest_bytes, push_bytes32, push_len_prefixed, push_u64, push_u8, CommitSubject,
        COMMIT_SUBJECT_VERSION,
    },
    consensus_store::ConsensusValidatorDecision,
    placement::{AggregatorId, ShardPlacement},
    shard_quorum_certificate::{QuorumRule, ShardQuorumCertificate},
    shard_vote::ShardVote,
    types::{BatchRoute, RejectClass, RejectRecord},
};

const HOTSTUFF_LOCAL_TAG: &[u8] = b"z00z.hotstuff_local";

/// One deterministic local proposal for one HotStuff-like view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotstuffProposal {
    pub version: u8,
    pub view: u64,
    pub leader_id: AggregatorId,
    pub subject: CommitSubject,
    pub justify_qc_digest: Option<[u8; 32]>,
    pub proposal_digest: [u8; 32],
}

impl HotstuffProposal {
    /// Build one proposal that binds one full commit subject.
    #[must_use]
    pub fn new(
        view: u64,
        leader_id: AggregatorId,
        subject: CommitSubject,
        justify_qc_digest: Option<[u8; 32]>,
    ) -> Self {
        let mut proposal = Self {
            version: COMMIT_SUBJECT_VERSION,
            view,
            leader_id,
            subject,
            justify_qc_digest,
            proposal_digest: [0u8; 32],
        };
        proposal.proposal_digest = digest_bytes::<ShardEvidenceDomain>(
            "hotstuff_proposal",
            &proposal.encode_without_digest(),
        );
        proposal
    }

    /// Return the bound subject digest.
    #[must_use]
    pub fn subject_digest(&self) -> [u8; 32] {
        self.subject.digest()
    }

    fn encode_without_digest(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(512);
        out.extend_from_slice(HOTSTUFF_LOCAL_TAG);
        push_u8(&mut out, self.version);
        push_u8(&mut out, 1);
        push_u64(&mut out, self.view);
        push_u64(&mut out, u64::from(self.leader_id.as_u16()));
        push_len_prefixed(&mut out, &self.subject.encode());
        push_opt_bytes32(&mut out, self.justify_qc_digest);
        out
    }
}

/// Structured timeout evidence for one view change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotstuffTimeout {
    pub version: u8,
    pub view: u64,
    pub leader_id: AggregatorId,
    pub reporter_id: AggregatorId,
    pub membership_digest: [u8; 32],
    pub proposal_digest: Option<[u8; 32]>,
    pub detail: String,
    pub evidence_digest: [u8; 32],
}

impl HotstuffTimeout {
    /// Build timeout evidence for one stalled view.
    #[must_use]
    pub fn new(
        view: u64,
        leader_id: AggregatorId,
        reporter_id: AggregatorId,
        membership_digest: [u8; 32],
        proposal_digest: Option<[u8; 32]>,
        detail: impl Into<String>,
    ) -> Self {
        let mut timeout = Self {
            version: COMMIT_SUBJECT_VERSION,
            view,
            leader_id,
            reporter_id,
            membership_digest,
            proposal_digest,
            detail: detail.into(),
            evidence_digest: [0u8; 32],
        };
        timeout.evidence_digest = digest_bytes::<ShardEvidenceDomain>(
            "hotstuff_timeout",
            &timeout.encode_without_digest(),
        );
        timeout
    }

    fn encode_without_digest(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(192);
        out.extend_from_slice(HOTSTUFF_LOCAL_TAG);
        push_u8(&mut out, self.version);
        push_u8(&mut out, 2);
        push_u64(&mut out, self.view);
        push_u64(&mut out, u64::from(self.leader_id.as_u16()));
        push_u64(&mut out, u64::from(self.reporter_id.as_u16()));
        push_bytes32(&mut out, self.membership_digest);
        push_opt_bytes32(&mut out, self.proposal_digest);
        push_len_prefixed(&mut out, self.detail.as_bytes());
        out
    }
}

/// Structured view-change evidence derived from timeout evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotstuffViewChange {
    pub version: u8,
    pub from_view: u64,
    pub to_view: u64,
    pub new_leader_id: AggregatorId,
    pub membership_digest: [u8; 32],
    pub timeout_digests: Vec<[u8; 32]>,
    pub evidence_digest: [u8; 32],
}

impl HotstuffViewChange {
    /// Build one view change from one timeout witness.
    pub fn new(
        timeout: &HotstuffTimeout,
        new_leader_id: AggregatorId,
    ) -> Result<Self, RejectRecord> {
        let Some(to_view) = timeout.view.checked_add(1) else {
            return Err(reject(
                RejectClass::PolicyReject,
                "view overflow: HotStuff-local timeout cannot advance past u64::MAX",
            ));
        };

        let mut change = Self {
            version: COMMIT_SUBJECT_VERSION,
            from_view: timeout.view,
            to_view,
            new_leader_id,
            membership_digest: timeout.membership_digest,
            timeout_digests: vec![timeout.evidence_digest],
            evidence_digest: [0u8; 32],
        };
        change.evidence_digest = digest_bytes::<ShardEvidenceDomain>(
            "hotstuff_view_change",
            &change.encode_without_digest(),
        );
        Ok(change)
    }

    fn encode_without_digest(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(192);
        out.extend_from_slice(HOTSTUFF_LOCAL_TAG);
        push_u8(&mut out, self.version);
        push_u8(&mut out, 3);
        push_u64(&mut out, self.from_view);
        push_u64(&mut out, self.to_view);
        push_u64(&mut out, u64::from(self.new_leader_id.as_u16()));
        push_bytes32(&mut out, self.membership_digest);
        push_u64(&mut out, self.timeout_digests.len() as u64);
        for digest in &self.timeout_digests {
            push_bytes32(&mut out, *digest);
        }
        out
    }
}

/// Structured conflicting-proposal evidence for one leader and one view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotstuffLeaderConflict {
    pub version: u8,
    pub view: u64,
    pub leader_id: AggregatorId,
    pub membership_digest: [u8; 32],
    pub first_proposal_digest: [u8; 32],
    pub second_proposal_digest: [u8; 32],
    pub first_subject_digest: [u8; 32],
    pub second_subject_digest: [u8; 32],
    pub evidence_digest: [u8; 32],
}

impl HotstuffLeaderConflict {
    /// Build one leader-conflict artifact from two proposals.
    pub fn new(first: &HotstuffProposal, second: &HotstuffProposal) -> Result<Self, RejectRecord> {
        if first.view != second.view {
            return Err(reject(
                RejectClass::PolicyReject,
                "leader conflict requires one shared view",
            ));
        }
        if first.leader_id != second.leader_id {
            return Err(reject(
                RejectClass::PolicyReject,
                "leader conflict requires one shared leader",
            ));
        }
        if first.subject.membership_digest != second.subject.membership_digest {
            return Err(reject(
                RejectClass::PolicyReject,
                "leader conflict requires one shared membership digest",
            ));
        }
        if first.subject_digest() == second.subject_digest() {
            return Err(reject(
                RejectClass::PolicyReject,
                "leader conflict requires different subject digests",
            ));
        }

        let mut conflict = Self {
            version: COMMIT_SUBJECT_VERSION,
            view: first.view,
            leader_id: first.leader_id,
            membership_digest: first.subject.membership_digest,
            first_proposal_digest: first.proposal_digest,
            second_proposal_digest: second.proposal_digest,
            first_subject_digest: first.subject_digest(),
            second_subject_digest: second.subject_digest(),
            evidence_digest: [0u8; 32],
        };
        conflict.evidence_digest = digest_bytes::<ShardEvidenceDomain>(
            "hotstuff_leader_conflict",
            &conflict.encode_without_digest(),
        );
        Ok(conflict)
    }

    fn encode_without_digest(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(224);
        out.extend_from_slice(HOTSTUFF_LOCAL_TAG);
        push_u8(&mut out, self.version);
        push_u8(&mut out, 4);
        push_u64(&mut out, self.view);
        push_u64(&mut out, u64::from(self.leader_id.as_u16()));
        push_bytes32(&mut out, self.membership_digest);
        push_bytes32(&mut out, self.first_proposal_digest);
        push_bytes32(&mut out, self.second_proposal_digest);
        push_bytes32(&mut out, self.first_subject_digest);
        push_bytes32(&mut out, self.second_subject_digest);
        out
    }
}

/// Local backend QC that wraps the canonical shard quorum certificate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotstuffQc {
    pub version: u8,
    pub view: u64,
    pub leader_id: AggregatorId,
    pub proposal_digest: [u8; 32],
    pub subject_digest: [u8; 32],
    pub membership_digest: [u8; 32],
    pub certificate: ShardQuorumCertificate,
    pub view_change_digest: Option<[u8; 32]>,
    pub qc_digest: [u8; 32],
}

impl HotstuffQc {
    /// Build one backend QC over one canonical shard quorum certificate.
    pub fn new(
        proposal: &HotstuffProposal,
        certificate: ShardQuorumCertificate,
        view_change_digest: Option<[u8; 32]>,
    ) -> Result<Self, RejectRecord> {
        if certificate.quorum_rule != QuorumRule::BftTwoFPlusOne {
            return Err(reject(
                RejectClass::PolicyReject,
                "wrong quorum rule: HotStuff-local backend QC requires 2f+1 certificate math",
            ));
        }
        certificate.verify_subject(&proposal.subject)?;

        let mut qc = Self {
            version: COMMIT_SUBJECT_VERSION,
            view: proposal.view,
            leader_id: proposal.leader_id,
            proposal_digest: proposal.proposal_digest,
            subject_digest: proposal.subject_digest(),
            membership_digest: proposal.subject.membership_digest,
            certificate,
            view_change_digest,
            qc_digest: [0u8; 32],
        };
        qc.qc_digest =
            digest_bytes::<ShardEvidenceDomain>("hotstuff_backend_qc", &qc.encode_without_digest());
        Ok(qc)
    }

    fn encode_without_digest(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(512);
        out.extend_from_slice(HOTSTUFF_LOCAL_TAG);
        push_u8(&mut out, self.version);
        push_u8(&mut out, 5);
        push_u64(&mut out, self.view);
        push_u64(&mut out, u64::from(self.leader_id.as_u16()));
        push_bytes32(&mut out, self.proposal_digest);
        push_bytes32(&mut out, self.subject_digest);
        push_bytes32(&mut out, self.membership_digest);
        push_len_prefixed(&mut out, &self.certificate.encode());
        push_opt_bytes32(&mut out, self.view_change_digest);
        out
    }
}

/// Full local commit bundle with backend QC plus canonical shard certificate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotstuffCommit {
    pub proposal: HotstuffProposal,
    pub backend_qc: HotstuffQc,
    pub commit: BftCommit,
}

/// Local deterministic HotStuff-like backend behind the live subject seam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotstuffLocal {
    route: BatchRoute,
    committee: BftCommittee,
    current_view: u64,
    last_qc: Option<HotstuffQc>,
    last_change: Option<HotstuffViewChange>,
    proposals: BTreeMap<(u64, AggregatorId), HotstuffProposal>,
}

impl HotstuffLocal {
    /// Build the backend from one live shard placement.
    pub fn from_placement(placement: &ShardPlacement) -> Result<Self, RejectRecord> {
        Ok(Self {
            route: placement.route,
            committee: BftCommittee::from_placement(placement)?,
            current_view: 0,
            last_qc: None,
            last_change: None,
            proposals: BTreeMap::new(),
        })
    }

    /// Return the active committee.
    #[must_use]
    pub const fn committee(&self) -> &BftCommittee {
        &self.committee
    }

    /// Return the active route.
    #[must_use]
    pub const fn route(&self) -> BatchRoute {
        self.route
    }

    /// Return the current view.
    #[must_use]
    pub const fn view(&self) -> u64 {
        self.current_view
    }

    /// Return the active membership digest.
    #[must_use]
    pub fn membership_digest(&self) -> [u8; 32] {
        self.committee.membership_digest(self.route)
    }

    /// Return the deterministic leader for one view.
    #[must_use]
    pub fn leader(&self, view: u64) -> AggregatorId {
        let members = self.member_ids();
        members[(view as usize) % members.len()]
    }

    /// Register one proposal for the active view.
    pub fn propose(&mut self, subject: CommitSubject) -> Result<HotstuffProposal, RejectRecord> {
        if subject.route() != self.route {
            return Err(reject(
                RejectClass::PolicyReject,
                "wrong generation: HotStuff-local subject route drifted from the active route",
            ));
        }
        if subject.term < self.current_view {
            return Err(reject(
                RejectClass::PolicyReject,
                "stale view: HotStuff-local view regressed",
            ));
        }
        if subject.term != self.current_view {
            return Err(reject(
                RejectClass::PolicyReject,
                "wrong view: HotStuff-local proposal term must equal the active view",
            ));
        }
        if subject.membership_digest != self.membership_digest() {
            return Err(reject(
                RejectClass::PolicyReject,
                "membership drift: HotStuff-local proposal drifted from the active committee",
            ));
        }

        let leader_id = self.leader(subject.term);
        let proposal = HotstuffProposal::new(
            subject.term,
            leader_id,
            subject,
            self.last_qc.as_ref().map(|qc| qc.qc_digest),
        );
        let key = (proposal.view, proposal.leader_id);
        if let Some(current) = self.proposals.get(&key) {
            if current.subject_digest() != proposal.subject_digest() {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "leader conflict: HotStuff-local leader proposed a different subject in the same view",
                ));
            }
            return Ok(current.clone());
        }
        self.proposals.insert(key, proposal.clone());
        Ok(proposal)
    }

    /// Build timeout evidence for the current view.
    pub fn timeout(
        &self,
        reporter_id: AggregatorId,
        detail: impl Into<String>,
    ) -> Result<HotstuffTimeout, RejectRecord> {
        if !self.is_member(reporter_id) {
            return Err(reject(
                RejectClass::PolicyReject,
                "inactive reporter: HotStuff-local timeout reporter is not in the active committee",
            ));
        }
        let leader_id = self.leader(self.current_view);
        let proposal_digest = self
            .proposals
            .get(&(self.current_view, leader_id))
            .map(|proposal| proposal.proposal_digest);
        Ok(HotstuffTimeout::new(
            self.current_view,
            leader_id,
            reporter_id,
            self.membership_digest(),
            proposal_digest,
            detail,
        ))
    }

    /// Advance one view after timeout evidence.
    pub fn advance_view(
        &mut self,
        timeout: &HotstuffTimeout,
    ) -> Result<HotstuffViewChange, RejectRecord> {
        if timeout.view != self.current_view {
            return Err(reject(
                RejectClass::PolicyReject,
                "stale timeout: HotStuff-local timeout does not match the active view",
            ));
        }
        if timeout.membership_digest != self.membership_digest() {
            return Err(reject(
                RejectClass::PolicyReject,
                "membership drift: HotStuff-local timeout drifted from the active committee",
            ));
        }

        let Some(next_view) = self.current_view.checked_add(1) else {
            return Err(reject(
                RejectClass::PolicyReject,
                "view overflow: HotStuff-local backend cannot advance past u64::MAX",
            ));
        };
        let change = HotstuffViewChange::new(timeout, self.leader(next_view))?;
        self.current_view = next_view;
        self.last_change = Some(change.clone());
        Ok(change)
    }

    /// Collect one backend QC from canonical BFT votes.
    pub fn collect_qc(
        &mut self,
        proposal: &HotstuffProposal,
        votes: &[ShardVote],
    ) -> Result<HotstuffQc, RejectRecord> {
        self.check_proposal(proposal)?;
        let certificate = ShardQuorumCertificate::new_bft(
            &proposal.subject,
            self.committee.primary_id(),
            self.committee.ready_secondary_ids().iter().copied(),
            votes,
        )?;
        let view_change_digest = self
            .last_change
            .as_ref()
            .filter(|change| change.to_view == proposal.view)
            .map(|change| change.evidence_digest);
        let qc = HotstuffQc::new(proposal, certificate, view_change_digest)?;
        self.last_qc = Some(qc.clone());
        Ok(qc)
    }

    /// Commit one proposal after collecting one backend QC.
    pub fn commit(
        &mut self,
        proposal: &HotstuffProposal,
        votes: &[ShardVote],
    ) -> Result<HotstuffCommit, RejectRecord> {
        let backend_qc = self.collect_qc(proposal, votes)?;
        let commit = BftCommit::new(proposal.subject.clone(), backend_qc.certificate.clone());
        Ok(HotstuffCommit {
            proposal: proposal.clone(),
            backend_qc,
            commit,
        })
    }

    /// Verify that one backend commit is still bound to the validator decision.
    pub fn bind_validator(
        &self,
        commit: &HotstuffCommit,
        decision: &ConsensusValidatorDecision,
    ) -> Result<(), RejectRecord> {
        let subject = &commit.proposal.subject;
        commit.backend_qc.certificate.verify_subject(subject)?;

        if decision.batch_id != subject.batch_id {
            return Err(reject(
                RejectClass::PolicyReject,
                "validator drift: HotStuff-local validator batch id drifted from the proposal subject",
            ));
        }
        if decision.subject_digest != subject.digest() {
            return Err(reject(
                RejectClass::PolicyReject,
                "validator drift: HotStuff-local validator subject digest drifted from the proposal subject",
            ));
        }
        if decision.certificate_digest != commit.commit.certificate.digest() {
            return Err(reject(
                RejectClass::PolicyReject,
                "validator drift: HotStuff-local validator certificate digest drifted from the backend QC",
            ));
        }
        if decision.theorem_digest != subject.theorem_or_settlement_digest {
            return Err(reject(
                RejectClass::PolicyReject,
                "validator drift: HotStuff-local validator theorem digest drifted from the proposal subject",
            ));
        }
        if decision.publication_binding_digest != Some(subject.publication_binding_digest) {
            return Err(reject(
                RejectClass::PolicyReject,
                "validator binding missing: backend QC still requires the live publication binding digest",
            ));
        }
        if decision.checkpoint_id.is_none() {
            return Err(reject(
                RejectClass::PolicyReject,
                "validator checkpoint missing: backend QC still requires validator checkpoint binding",
            ));
        }
        Ok(())
    }

    fn check_proposal(&self, proposal: &HotstuffProposal) -> Result<(), RejectRecord> {
        if proposal.subject.route() != self.route {
            return Err(reject(
                RejectClass::PolicyReject,
                "wrong generation: HotStuff-local proposal route drifted from the active route",
            ));
        }
        if proposal.subject.membership_digest != self.membership_digest() {
            return Err(reject(
                RejectClass::PolicyReject,
                "membership drift: HotStuff-local proposal drifted from the active committee",
            ));
        }
        if proposal.view != self.current_view || proposal.subject.term != self.current_view {
            return Err(reject(
                RejectClass::PolicyReject,
                "wrong view: HotStuff-local proposal does not match the active view",
            ));
        }
        if proposal.leader_id != self.leader(proposal.view) {
            return Err(reject(
                RejectClass::PolicyReject,
                "wrong leader: HotStuff-local proposal leader drifted from the deterministic leader schedule",
            ));
        }
        Ok(())
    }

    fn is_member(&self, aggregator_id: AggregatorId) -> bool {
        aggregator_id == self.committee.primary_id()
            || self
                .committee
                .ready_secondary_ids()
                .contains(&aggregator_id)
    }

    fn member_ids(&self) -> Vec<AggregatorId> {
        std::iter::once(self.committee.primary_id())
            .chain(self.committee.ready_secondary_ids().iter().copied())
            .collect()
    }
}

fn push_opt_bytes32(out: &mut Vec<u8>, value: Option<[u8; 32]>) {
    match value {
        Some(value) => {
            push_u8(out, 1);
            push_bytes32(out, value);
        }
        None => push_u8(out, 0),
    }
}

fn reject(class: RejectClass, detail: &str) -> RejectRecord {
    RejectRecord {
        intake_id: None,
        class,
        detail: detail.to_string(),
    }
}
