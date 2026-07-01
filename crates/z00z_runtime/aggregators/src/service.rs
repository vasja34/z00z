#![forbid(unsafe_code)]

use z00z_storage::{
    checkpoint::{CheckpointExecTx, CheckpointId, CheckpointPubIn},
    settlement::{SettlementExecHandoff, StoreOp},
};

use crate::types::{
    BatchId, IntakeId, OrderedBatch, PublicationBinding, PublicationRecord, PublicationRequest,
    PublishedBatch, RejectRecord, SoftConfirmation, WorkItem, WorkPayload,
};

pub trait AggregatorIngress {
    fn admit(&mut self, item: WorkPayload) -> Result<WorkItem, RejectRecord>;
}

pub trait AggregatorOrdering {
    fn order(&mut self, items: &[WorkItem]) -> Result<OrderedBatch, RejectRecord>;
}

pub trait AggregatorRecovery {
    fn build_publication(&mut self, batch: OrderedBatch) -> PublicationRequest;

    fn record_publication(&mut self, batch: PublishedBatch) -> PublicationRecord;
}

pub trait AggregatorService: AggregatorIngress + AggregatorOrdering + AggregatorRecovery {
    fn emit_soft_confirmation(&self, intake_id: &IntakeId, batch_id: &BatchId) -> SoftConfirmation;

    fn bind_exec_handoff(
        &self,
        batch: &OrderedBatch,
        ops: Vec<StoreOp>,
        txs: Vec<CheckpointExecTx>,
    ) -> SettlementExecHandoff {
        batch.exec_handoff(ops, txs)
    }
}

#[must_use]
pub fn bind_publication_contract(
    batch_id: BatchId,
    checkpoint_id: CheckpointId,
    route_table_digest: [u8; 32],
    pub_in: &CheckpointPubIn,
) -> PublicationBinding {
    PublicationBinding::new(batch_id, checkpoint_id, route_table_digest, pub_in)
}
