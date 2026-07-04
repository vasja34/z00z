#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod batch_planner;
mod commit_subject;
mod consensus_adapter;
mod dist_dispatch;
mod dist_scheduler;
mod dist_sim;
mod ingress;
mod ordering;
mod placement;
mod recovery;
mod scheduler;
mod secondary_replay;
mod service;
mod shard_exec;
mod shard_quorum_certificate;
mod shard_vote;
mod types;

pub use batch_planner::{BatchPlanner, RouteErr, RouteRangeRule, ShardRouteTable};
pub use commit_subject::{CommitSubject, JournalCandidate};
pub use consensus_adapter::{ConsensusAdapter, ConsensusCommit, MembershipChange};
pub use dist_dispatch::{
    DispatchStage, DispatchVerdict, DistDispatch, DistLevel, DistNote, DistNoteKind, RouteRollout,
};
pub use dist_scheduler::{BatchWave, DistScheduler, ScheduledBatch, SchedulerWave};
pub use dist_sim::{DistNode, DistSim, FrameStage, FrameVerdict, JournalFrame};
pub use ingress::IngressBoundary;
pub use ordering::OrderingBoundary;
pub use placement::{
    AggregatorId, SecondaryState, ShardPlacement, ShardPlacementTable, ShardPlacementView,
};
pub use recovery::{RecoveryBoundary, RecoveryIntent, ShardRecoveryRecord};
pub use scheduler::SchedulerBoundary;
pub use secondary_replay::{
    SecondaryReplayAccept, SecondaryReplayReject, SecondaryReplayRejectCode,
    SecondaryReplayRequest, SecondaryReplayVerdict, SecondaryReplayVerifier,
};
pub use service::{
    bind_publication_contract, AggregatorIngress, AggregatorOrdering, AggregatorRecovery,
    AggregatorService,
};
pub use shard_exec::{ShardExecState, ShardExecTicket, ShardExecutor};
pub use shard_quorum_certificate::{
    membership_digest_for_voters, QuorumRule, ShardQuorumCertificate,
};
pub use shard_vote::{ShardVote, ShardVoteKind, ShardVoteRole};
pub use types::{
    BatchId, BatchPlanned, BatchRoute, IntakeId, ObjectWitnessBundleV1, OrderedBatch, PlanDigest,
    PlannerMode, PublicationBinding, PublicationRecord, PublicationRequest, PublicationState,
    PublishedBatch, RejectClass, RejectRecord, RightWitnessRefV1, RightWitnessStateV1,
    RuntimeObjectPackageV1, ShardId, SoftConfirmation, WorkItem, WorkPayload,
};
