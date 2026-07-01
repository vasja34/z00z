#![forbid(unsafe_code)]

use z00z_aggregators::{
    DistLevel, DistNote, DistNoteKind, PublicationRecord, PublicationState, PublishedBatch,
    ShardExecTicket, ShardPlacementView, SoftConfirmation,
};
use z00z_validators::{ObjectRejectCode, Verdict, VerdictKind};

use crate::{
    alerts::WatcherAlert,
    da_health::{ProviderOutcome, ProviderSignal, ProviderStage},
    evidence_export::{EvidenceKey, EvidenceRecord},
    publication::{PublicationWatch, PublicationWatchErr},
    status::{AlertCounts, ObservationSnapshot},
};

#[derive(Debug, Clone, PartialEq)]
pub struct WatcherInput {
    pub published: PublishedBatch,
    pub publication: PublicationRecord,
    pub soft_confirmation: Option<SoftConfirmation>,
    pub placement: Option<ShardPlacementView>,
    pub exec_ticket: Option<ShardExecTicket>,
    pub verdict: Option<Verdict>,
    pub provider_signal: Option<ProviderSignal>,
    pub runtime_notes: Vec<DistNote>,
}

pub trait WatcherService {
    fn observe(&mut self, input: WatcherInput) -> ObservationSnapshot;

    fn alerts(&self) -> &[WatcherAlert];
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WatcherBoundary;

impl WatcherBoundary {
    fn apply_runtime_notes(&self, snapshot: &mut ObservationSnapshot, runtime_notes: &[DistNote]) {
        snapshot.runtime_notes = runtime_notes.to_vec();
        snapshot.runtime_truth = false;
        for note in runtime_notes {
            snapshot.alert_counts = snapshot
                .alert_counts
                .with_increment(note_level_severity(note.level));
        }
    }

    fn has_publication_gap(&self, input: &WatcherInput) -> bool {
        if matches!(
            input.publication.state,
            PublicationState::Accepted | PublicationState::Rejected | PublicationState::Finalized
        ) {
            return false;
        }

        matches!(
            input.provider_signal.as_ref().map(|signal| signal.outcome),
            Some(
                ProviderOutcome::Pending | ProviderOutcome::RetryPending | ProviderOutcome::Missing
            )
        )
    }

    fn validator_state_alert_inner(&self, input: &WatcherInput) -> Option<WatcherAlert> {
        match self.publication_watch(input) {
            Ok(_) => {
                if matches!(
                    input.verdict.as_ref().map(|verdict| verdict.kind.clone()),
                    Some(VerdictKind::Incomplete)
                ) || matches!(input.publication.state, PublicationState::RetryPending)
                    || self.has_publication_gap(input)
                {
                    Some(WatcherAlert::batch(
                        crate::alerts::AlertKind::ValidatorIncomplete,
                        crate::alerts::AlertSeverity::Warn,
                        input.published.batch_id,
                    ))
                } else {
                    None
                }
            }
            Err(err) if err.is_validator_incomplete() => Some(WatcherAlert::batch(
                crate::alerts::AlertKind::ValidatorIncomplete,
                crate::alerts::AlertSeverity::Warn,
                input.published.batch_id,
            )),
            Err(_) => Some(WatcherAlert::batch(
                crate::alerts::AlertKind::InvalidBatch,
                crate::alerts::AlertSeverity::Critical,
                input.published.batch_id,
            )),
        }
    }

    #[must_use]
    pub fn placement_view<'a>(&self, input: &'a WatcherInput) -> Option<&'a ShardPlacementView> {
        input
            .exec_ticket
            .as_ref()
            .map(|ticket| &ticket.placement)
            .or(input.placement.as_ref())
    }

    #[must_use]
    pub fn exec_ticket<'a>(&self, input: &'a WatcherInput) -> Option<&'a ShardExecTicket> {
        input.exec_ticket.as_ref()
    }

    pub fn publication_watch(
        &self,
        input: &WatcherInput,
    ) -> Result<PublicationWatch, PublicationWatchErr> {
        PublicationWatch::try_from_runtime(
            &input.published,
            &input.publication,
            input.verdict.as_ref(),
            self.placement_view(input),
            self.exec_ticket(input),
        )
    }

    pub fn checked_snapshot(
        &self,
        input: &WatcherInput,
        alert_counts: AlertCounts,
    ) -> Result<ObservationSnapshot, PublicationWatchErr> {
        let publication = self.publication_watch(input)?;
        let mut snapshot = ObservationSnapshot::from_runtime(
            input.published.batch_id,
            input.publication.state.clone(),
            self.placement_view(input),
            self.exec_ticket(input),
            input.verdict.as_ref().map(VerdictKind::clone_from_ref),
            input.provider_signal.as_ref().map(stage_of),
            input.provider_signal.as_ref().map(outcome_of),
            alert_counts,
        );
        snapshot.binding_digest = Some(publication.publication.binding_digest());
        snapshot.route_table_digest = Some(publication.publication.route_table_digest());
        snapshot.routing_generation = publication
            .runtime_route
            .map(|route| route.routing_generation)
            .or(snapshot.routing_generation);
        self.apply_runtime_notes(&mut snapshot, &input.runtime_notes);
        Ok(snapshot)
    }

    #[must_use]
    pub fn project_snapshot(
        &self,
        input: &WatcherInput,
        alert_counts: AlertCounts,
    ) -> ObservationSnapshot {
        let alert = self.validator_state_alert_inner(input);
        let mut snapshot = self
            .checked_snapshot(input, alert_counts)
            .unwrap_or_else(|_| {
                let mut snapshot = ObservationSnapshot::from_runtime(
                    input.published.batch_id,
                    input.publication.state.clone(),
                    self.placement_view(input),
                    self.exec_ticket(input),
                    input.verdict.as_ref().map(VerdictKind::clone_from_ref),
                    input.provider_signal.as_ref().map(stage_of),
                    input.provider_signal.as_ref().map(outcome_of),
                    alert_counts,
                );
                self.apply_runtime_notes(&mut snapshot, &input.runtime_notes);
                snapshot
            });

        if matches!(
            alert.as_ref().map(|item| &item.kind),
            Some(crate::alerts::AlertKind::ValidatorIncomplete)
        ) {
            snapshot.verdict_kind = Some(VerdictKind::Incomplete);
        }
        if let Some(alert) = alert {
            snapshot.alert_counts = snapshot.alert_counts.with_increment(alert.severity);
        }

        snapshot
    }

    #[must_use]
    pub fn validator_state_alerts(&self, input: &WatcherInput) -> Vec<WatcherAlert> {
        self.validator_state_alert_inner(input)
            .into_iter()
            .collect()
    }

    #[must_use]
    pub fn runtime_note_alerts(&self, input: &WatcherInput) -> Vec<WatcherAlert> {
        input
            .runtime_notes
            .iter()
            .map(|note| {
                WatcherAlert::batch(
                    note_kind(note.kind),
                    note_level_severity(note.level),
                    input.published.batch_id,
                )
            })
            .collect()
    }

    #[must_use]
    pub fn validator_state_evidence(
        &self,
        input: &WatcherInput,
        sequence: u64,
    ) -> Option<EvidenceRecord> {
        let alert = self.validator_state_alert_inner(input)?;
        Some(EvidenceRecord {
            evidence_key: EvidenceKey {
                batch_id: input.published.batch_id,
                sequence,
            },
            kind: alert.kind,
            severity: alert.severity,
            subject: alert.subject,
            publication: Some(input.publication.clone()),
            published: Some(input.published.clone()),
            soft_confirmation: input.soft_confirmation.clone(),
            placement: input.placement.clone(),
            exec_ticket: input.exec_ticket.clone(),
            verdict: input.verdict.clone(),
            provider_signal: input.provider_signal.clone(),
        })
    }

    #[must_use]
    pub fn object_alerts(&self, verdict: &Verdict) -> Vec<WatcherAlert> {
        verdict
            .object_verdicts
            .iter()
            .filter_map(|item| {
                item.reject.map(|reject| {
                    WatcherAlert::batch(
                        crate::alerts::AlertKind::ObjectReject(reject),
                        reject_severity(reject),
                        verdict.batch_id,
                    )
                })
            })
            .collect()
    }
}

fn stage_of(signal: &ProviderSignal) -> ProviderStage {
    signal.stage
}

fn outcome_of(signal: &ProviderSignal) -> ProviderOutcome {
    signal.outcome
}

trait VerdictKindClone {
    fn clone_from_ref(item: &Verdict) -> VerdictKind;
}

impl VerdictKindClone for VerdictKind {
    fn clone_from_ref(item: &Verdict) -> VerdictKind {
        item.kind.clone()
    }
}

fn reject_severity(code: ObjectRejectCode) -> crate::alerts::AlertSeverity {
    match code {
        ObjectRejectCode::UnknownPolicy
        | ObjectRejectCode::InvalidBacking
        | ObjectRejectCode::WrongFamilyProof
        | ObjectRejectCode::VoucherUsedAsCash
        | ObjectRejectCode::RightUsedAsValue
        | ObjectRejectCode::Replay
        | ObjectRejectCode::DoubleRedeem
        | ObjectRejectCode::StaleRoot
        | ObjectRejectCode::FeeBoundary => crate::alerts::AlertSeverity::Critical,
        ObjectRejectCode::UnknownAction
        | ObjectRejectCode::MissingRight
        | ObjectRejectCode::RightOutOfScope
        | ObjectRejectCode::RightExpired
        | ObjectRejectCode::RightRevoked
        | ObjectRejectCode::RightConsumed
        | ObjectRejectCode::ResidualMismatch
        | ObjectRejectCode::ForcedAcceptance
        | ObjectRejectCode::MissingSignature
        | ObjectRejectCode::MissingAttestation
        | ObjectRejectCode::ExpiredVoucherUse => crate::alerts::AlertSeverity::Warn,
    }
}

fn note_kind(kind: DistNoteKind) -> crate::alerts::AlertKind {
    match kind {
        DistNoteKind::RouteRollout => crate::alerts::AlertKind::RouteRollout,
        DistNoteKind::SchedulerWave => crate::alerts::AlertKind::SchedulerWave,
        DistNoteKind::ShardStall => crate::alerts::AlertKind::ShardStall,
        DistNoteKind::ShardFreeze => crate::alerts::AlertKind::ShardFreeze,
        DistNoteKind::DispatchDispute => crate::alerts::AlertKind::DispatchDispute,
        DistNoteKind::RouteDrift => crate::alerts::AlertKind::RouteDrift,
        DistNoteKind::FailoverState => crate::alerts::AlertKind::FailoverState,
        DistNoteKind::StorageLockHazard => crate::alerts::AlertKind::StorageLockHazard,
    }
}

fn note_level_severity(level: DistLevel) -> crate::alerts::AlertSeverity {
    match level {
        DistLevel::Info => crate::alerts::AlertSeverity::Info,
        DistLevel::Warn => crate::alerts::AlertSeverity::Warn,
        DistLevel::Critical => crate::alerts::AlertSeverity::Critical,
    }
}
