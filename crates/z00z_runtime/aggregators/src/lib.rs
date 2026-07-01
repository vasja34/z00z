#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod batch_planner;
mod consensus_adapter;
mod dist_dispatch;
mod dist_scheduler;
mod dist_sim;
mod ingress;
mod ordering;
mod placement;
mod recovery;
mod scheduler;
mod service;
mod shard_exec;
mod types;

pub use batch_planner::{BatchPlanner, RouteErr, RouteRangeRule, ShardRouteTable};
pub use consensus_adapter::{
    ConsensusAdapter, ConsensusCommit, JournalCandidate, MembershipChange,
};
pub use dist_dispatch::{
    DispatchStage, DispatchVerdict, DistDispatch, DistLevel, DistNote, DistNoteKind, RouteRollout,
};
pub use dist_scheduler::{BatchWave, DistScheduler, ScheduledBatch, SchedulerWave};
pub use dist_sim::{DistNode, DistSim, FrameStage, FrameVerdict, JournalFrame};
pub use ingress::IngressBoundary;
pub use ordering::OrderingBoundary;
pub use placement::{
    AggregatorId, ShardPlacement, ShardPlacementTable, ShardPlacementView, StandbyState,
};
pub use recovery::{RecoveryBoundary, RecoveryIntent, ShardRecoveryRecord};
pub use scheduler::SchedulerBoundary;
pub use service::{
    bind_publication_contract, AggregatorIngress, AggregatorOrdering, AggregatorRecovery,
    AggregatorService,
};
pub use shard_exec::{ShardExecState, ShardExecTicket, ShardExecutor};
pub use types::{
    BatchId, BatchPlanned, BatchRoute, IntakeId, ObjectWitnessBundleV1, OrderedBatch, PlanDigest,
    PlannerMode, PublicationBinding, PublicationRecord, PublicationRequest, PublicationState,
    PublishedBatch, RejectClass, RejectRecord, RightWitnessRefV1, RightWitnessStateV1,
    RuntimeObjectPackageV1, ShardId, SoftConfirmation, WorkItem, WorkPayload,
};
