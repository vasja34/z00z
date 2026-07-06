#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use z00z_storage::settlement::SettlementRecoveryState;

use crate::{
    batch_planner::BatchPlanner,
    commit_subject::{CommitSubject, JournalCandidate},
    ingress::IngressBoundary,
    placement::{AggregatorId, ShardPlacementTable},
    recovery::{RecoveryBoundary, RecoveryIntent, ShardRecoveryRecord},
    shard_quorum_certificate::membership_digest_for_voters,
    types::{PublicationBinding, RejectClass, RejectRecord, WorkItem},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecondaryReplayRejectCode {
    ShapeInvalid,
    WrongRoute,
    WrongPlanDigest,
    WrongRoot,
    WrongLineage,
    WrongProofVersion,
    WrongPolicyGeneration,
    WrongPublicationBinding,
    WrongTheoremDigest,
    WrongDaAvailability,
    MembershipDrift,
    StaleSecondaryState,
    WrongTerm,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecondaryReplayReject {
    pub code: SecondaryReplayRejectCode,
    pub class: RejectClass,
    pub detail: String,
}

impl SecondaryReplayReject {
    fn new(code: SecondaryReplayRejectCode, class: RejectClass, detail: impl Into<String>) -> Self {
        Self {
            code,
            class,
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecondaryReplayAccept {
    pub subject: CommitSubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecondaryReplayVerdict {
    Accept(Box<SecondaryReplayAccept>),
    Reject(SecondaryReplayReject),
}

#[derive(Debug, Clone, Copy)]
pub struct SecondaryReplayRequest<'a> {
    pub voter_id: AggregatorId,
    pub term: u64,
    pub items: &'a [WorkItem],
    pub planner: &'a BatchPlanner,
    pub placement_table: &'a ShardPlacementTable,
    pub recovery_record: &'a ShardRecoveryRecord,
    pub local_recovery: &'a SettlementRecoveryState,
    pub publication_binding: &'a PublicationBinding,
    pub theorem_or_settlement_digest: [u8; 32],
    pub da_availability_digest: Option<[u8; 32]>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SecondaryReplayVerifier;

impl SecondaryReplayVerifier {
    pub fn replay_subject(
        &self,
        request: &SecondaryReplayRequest<'_>,
    ) -> Result<CommitSubject, SecondaryReplayReject> {
        let items = rebuild_items(request.items)?;
        let batch = request
            .planner
            .make_batch(request.recovery_record.batch_id, &items)
            .map_err(classify_planner_reject)?;

        let live_placement = request
            .placement_table
            .placement(request.recovery_record.placement.route)
            .ok_or_else(|| {
                SecondaryReplayReject::new(
                    SecondaryReplayRejectCode::WrongRoute,
                    RejectClass::PolicyReject,
                    "wrong route: live placement table does not own the recovery route",
                )
            })?;

        RecoveryBoundary
            .resume(
                request.voter_id,
                request.placement_table,
                request.recovery_record,
                request.local_recovery,
                RecoveryIntent::TakeoverSecondary,
            )
            .map_err(classify_resume_reject)?;

        let candidate = JournalCandidate::from_record(request.recovery_record)
            .map_err(classify_candidate_reject)?;
        let membership_digest = membership_digest_for_voters(
            live_placement.route,
            live_placement.primary_id,
            live_placement
                .secondaries
                .iter()
                .filter(|secondary| secondary.is_ready)
                .map(|secondary| secondary.aggregator_id),
        );

        CommitSubject::from_runtime(
            request.term,
            membership_digest,
            &batch,
            &candidate,
            request.publication_binding,
            request.theorem_or_settlement_digest,
            request.da_availability_digest,
        )
        .map_err(classify_subject_reject)
    }

    #[must_use]
    pub fn verify(
        &self,
        claimed_subject: &CommitSubject,
        request: &SecondaryReplayRequest<'_>,
    ) -> SecondaryReplayVerdict {
        match self.replay_subject(request) {
            Ok(subject) => {
                if &subject == claimed_subject {
                    SecondaryReplayVerdict::Accept(Box::new(SecondaryReplayAccept { subject }))
                } else {
                    SecondaryReplayVerdict::Reject(compare_subjects(&subject, claimed_subject))
                }
            }
            Err(err) => SecondaryReplayVerdict::Reject(err),
        }
    }
}

fn rebuild_items(items: &[WorkItem]) -> Result<Vec<WorkItem>, SecondaryReplayReject> {
    let ingress = IngressBoundary;
    let mut rebuilt = Vec::with_capacity(items.len());
    for item in items {
        let mut rebound = ingress.normalize(item.payload().clone()).map_err(|err| {
            SecondaryReplayReject::new(
                SecondaryReplayRejectCode::ShapeInvalid,
                err.class,
                err.detail,
            )
        })?;
        if let Some(object_package) = item.object_package() {
            rebound = rebound.with_object_package(object_package.clone());
        }
        if rebound.route_key() != item.route_key()
            || rebound.admission_digest_bytes() != item.admission_digest_bytes()
            || rebound.kind_tag() != item.kind_tag()
        {
            return Err(SecondaryReplayReject::new(
                SecondaryReplayRejectCode::WrongPlanDigest,
                RejectClass::PolicyReject,
                "planner digest drift: ingress-normalized work item no longer matches the local admission digest",
            ));
        }
        rebuilt.push(rebound);
    }
    Ok(rebuilt)
}

fn compare_subjects(expected: &CommitSubject, claimed: &CommitSubject) -> SecondaryReplayReject {
    if expected.term != claimed.term {
        return SecondaryReplayReject::new(
            SecondaryReplayRejectCode::WrongTerm,
            RejectClass::PolicyReject,
            "wrong term: replayed subject term drifted from the claimed primary subject",
        );
    }
    if expected.shard_id != claimed.shard_id
        || expected.routing_generation != claimed.routing_generation
        || expected.route_table_digest != claimed.route_table_digest
    {
        return SecondaryReplayReject::new(
            SecondaryReplayRejectCode::WrongRoute,
            RejectClass::PolicyReject,
            "wrong route: replayed route metadata drifted from the claimed primary subject",
        );
    }
    if expected.membership_digest != claimed.membership_digest {
        return SecondaryReplayReject::new(
            SecondaryReplayRejectCode::MembershipDrift,
            RejectClass::PolicyReject,
            "membership drift: replayed membership digest drifted from the claimed primary subject",
        );
    }
    if expected.batch_id != claimed.batch_id
        || expected.plan_digest != claimed.plan_digest
        || expected.ordered_batch_digest != claimed.ordered_batch_digest
        || expected.payload_digest != claimed.payload_digest
    {
        return SecondaryReplayReject::new(
            SecondaryReplayRejectCode::WrongPlanDigest,
            RejectClass::PolicyReject,
            "planner digest drift: replayed batch digest set drifted from the claimed primary subject",
        );
    }
    if expected.previous_state_root != claimed.previous_state_root
        || expected.new_state_root != claimed.new_state_root
    {
        return SecondaryReplayReject::new(
            SecondaryReplayRejectCode::WrongRoot,
            RejectClass::PolicyReject,
            "wrong root: replayed settlement roots drifted from the claimed primary subject",
        );
    }
    if expected.journal_lineage != claimed.journal_lineage {
        return SecondaryReplayReject::new(
            SecondaryReplayRejectCode::WrongLineage,
            RejectClass::PolicyReject,
            "wrong lineage: replayed journal lineage drifted from the claimed primary subject",
        );
    }
    if expected.root_generation != claimed.root_generation
        || expected.proof_version != claimed.proof_version
    {
        return SecondaryReplayReject::new(
            SecondaryReplayRejectCode::WrongProofVersion,
            RejectClass::PolicyReject,
            "wrong proof version: replayed proof metadata drifted from the claimed primary subject",
        );
    }
    if expected.bucket_policy_generation != claimed.bucket_policy_generation
        || expected.bucket_policy_id != claimed.bucket_policy_id
    {
        return SecondaryReplayReject::new(
            SecondaryReplayRejectCode::WrongPolicyGeneration,
            RejectClass::PolicyReject,
            "wrong policy generation: replayed policy metadata drifted from the claimed primary subject",
        );
    }
    if expected.publication_binding_digest != claimed.publication_binding_digest {
        return SecondaryReplayReject::new(
            SecondaryReplayRejectCode::WrongPublicationBinding,
            RejectClass::PolicyReject,
            "wrong publication binding: replayed publication binding drifted from the claimed primary subject",
        );
    }
    if expected.theorem_or_settlement_digest != claimed.theorem_or_settlement_digest {
        return SecondaryReplayReject::new(
            SecondaryReplayRejectCode::WrongTheoremDigest,
            RejectClass::PolicyReject,
            "wrong theorem digest: replayed theorem digest drifted from the claimed primary subject",
        );
    }
    if expected.da_availability_digest != claimed.da_availability_digest {
        return SecondaryReplayReject::new(
            SecondaryReplayRejectCode::WrongDaAvailability,
            RejectClass::PolicyReject,
            "wrong data-availability digest: replayed availability binding drifted from the claimed primary subject",
        );
    }
    SecondaryReplayReject::new(
        SecondaryReplayRejectCode::WrongPlanDigest,
        RejectClass::PolicyReject,
        "replayed subject bytes drifted from the claimed primary subject",
    )
}

fn classify_planner_reject(err: RejectRecord) -> SecondaryReplayReject {
    let code = if err.class == RejectClass::ShapeInvalid {
        SecondaryReplayRejectCode::ShapeInvalid
    } else if err.detail.contains("route table")
        || err.detail.contains("multi-shard")
        || err.detail.contains("does not own")
    {
        SecondaryReplayRejectCode::WrongRoute
    } else {
        SecondaryReplayRejectCode::WrongPlanDigest
    };
    SecondaryReplayReject::new(code, err.class, err.detail)
}

fn classify_candidate_reject(err: RejectRecord) -> SecondaryReplayReject {
    let code = if err.detail.contains("route") {
        SecondaryReplayRejectCode::WrongRoute
    } else {
        SecondaryReplayRejectCode::WrongPlanDigest
    };
    SecondaryReplayReject::new(code, err.class, err.detail)
}

fn classify_resume_reject(err: RejectRecord) -> SecondaryReplayReject {
    let code = if err.detail.contains("wrong generation")
        || err.detail.contains("wrong shard")
        || err.detail.contains("wrong route digest")
    {
        SecondaryReplayRejectCode::WrongRoute
    } else if err.detail.contains("split-brain")
        || err.detail.contains("lawful secondary")
        || err.detail.contains("secondary aggregator down")
    {
        SecondaryReplayRejectCode::MembershipDrift
    } else {
        SecondaryReplayRejectCode::StaleSecondaryState
    };
    SecondaryReplayReject::new(code, err.class, err.detail)
}

fn classify_subject_reject(err: RejectRecord) -> SecondaryReplayReject {
    let code = if err.detail.contains("route drifted") {
        SecondaryReplayRejectCode::WrongRoute
    } else if err.detail.contains("publication route-table")
        || err.detail.contains("publication binding batch id")
    {
        SecondaryReplayRejectCode::WrongPublicationBinding
    } else if err.detail.contains("new settlement root") {
        SecondaryReplayRejectCode::WrongRoot
    } else {
        SecondaryReplayRejectCode::WrongPlanDigest
    };
    SecondaryReplayReject::new(code, err.class, err.detail)
}
