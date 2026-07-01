#![forbid(unsafe_code)]

use z00z_aggregators::{ShardExecState, ShardExecTicket, ShardPlacementView};
use z00z_storage::settlement::{
    inspect_object_package, ObjectPolicyRegistryV1, ObjectValidatorVerdict,
};

use crate::{
    checkpoint::CheckpointFlow,
    verdict::{reject_class, RejectClass, ResolvedBatch, Verdict, VerdictKind},
};

pub trait ValidatorService {
    fn validate(&mut self, batch: ResolvedBatch) -> Verdict;
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ValidatorBoundary;

impl ValidatorBoundary {
    #[must_use]
    fn is_incomplete_batch(&self, batch: &ResolvedBatch) -> bool {
        if batch.published.blob_ref.trim().is_empty() {
            return true;
        }

        matches!(
            batch.runtime_exec().map(|ticket| ticket.state),
            Some(ShardExecState::RetryPending | ShardExecState::RecoveryPending)
        )
    }

    #[must_use]
    pub fn placement_view<'a>(&self, batch: &'a ResolvedBatch) -> Option<&'a ShardPlacementView> {
        batch.runtime_placement()
    }

    #[must_use]
    pub fn exec_ticket<'a>(&self, batch: &'a ResolvedBatch) -> Option<&'a ShardExecTicket> {
        batch.runtime_exec()
    }

    pub fn checkpoint_flow(&self, batch: &ResolvedBatch) -> Result<CheckpointFlow, RejectClass> {
        CheckpointFlow::try_from_resolved(batch)
    }

    #[must_use]
    pub fn inspect_object_packages(
        &self,
        batch: &ResolvedBatch,
        registry: &ObjectPolicyRegistryV1,
    ) -> Vec<ObjectValidatorVerdict> {
        let prev_root = batch.published.pub_in.prev_settlement_root();
        let new_root = batch.published.pub_in.new_settlement_root();
        batch
            .object_packages()
            .map(|package| inspect_object_package(package, registry, prev_root, new_root))
            .collect()
    }

    #[must_use]
    pub fn verdict_for_batch(
        &self,
        batch: &ResolvedBatch,
        registry: &ObjectPolicyRegistryV1,
    ) -> Verdict {
        let flow = self.checkpoint_flow(batch);
        let incomplete = self.is_incomplete_batch(batch);
        let object_verdicts = if flow.is_ok() {
            self.inspect_object_packages(batch, registry)
        } else {
            Vec::new()
        };
        let object_reject = object_verdicts.iter().filter_map(|item| item.reject).next();
        let batch_reject = flow
            .as_ref()
            .err()
            .cloned()
            .or_else(|| object_reject.map(reject_class));

        Verdict {
            batch_id: batch.ordered.batch_id,
            checkpoint_id: Some(batch.published.checkpoint_id),
            publication: flow.ok().map(|item| item.publication),
            kind: if batch_reject.is_some() {
                VerdictKind::Rejected
            } else if incomplete {
                VerdictKind::Incomplete
            } else {
                VerdictKind::Accepted
            },
            reject: batch_reject,
            object_verdicts,
        }
    }
}
