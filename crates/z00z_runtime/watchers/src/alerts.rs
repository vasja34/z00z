#![forbid(unsafe_code)]

use z00z_aggregators::{BatchId, IntakeId, PublishedBatch};
use z00z_validators::ObjectRejectCode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatcherAlert {
    pub kind: AlertKind,
    pub severity: AlertSeverity,
    pub subject: AlertSubject,
}

impl WatcherAlert {
    #[must_use]
    pub const fn batch(kind: AlertKind, severity: AlertSeverity, batch_id: BatchId) -> Self {
        Self {
            kind,
            severity,
            subject: AlertSubject::Batch(batch_id),
        }
    }

    #[must_use]
    pub const fn intake(kind: AlertKind, severity: AlertSeverity, intake_id: IntakeId) -> Self {
        Self {
            kind,
            severity,
            subject: AlertSubject::Intake(intake_id),
        }
    }

    #[must_use]
    pub fn published(kind: AlertKind, severity: AlertSeverity, published: PublishedBatch) -> Self {
        Self {
            kind,
            severity,
            subject: AlertSubject::Published(Box::new(published)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertKind {
    PublicationLag,
    MissingBlob,
    CensorshipSuspect,
    ProviderDivergence,
    RetryStagnation,
    InvalidBatch,
    ValidatorIncomplete,
    RouteRollout,
    SchedulerWave,
    ShardStall,
    ShardFreeze,
    DispatchDispute,
    RouteDrift,
    FailoverState,
    StorageLockHazard,
    ObjectReject(ObjectRejectCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertSeverity {
    Info,
    Warn,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertSubject {
    Intake(IntakeId),
    Batch(BatchId),
    Published(Box<PublishedBatch>),
}
