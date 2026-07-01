#![forbid(unsafe_code)]

use z00z_aggregators::{bind_publication_contract, BatchRoute, PublicationBinding};
use z00z_storage::{
    checkpoint::derive_checkpoint_id,
    settlement::{check_route_binding_v1, PublicationRouteSnapshotV1},
};

use crate::{RejectClass, ResolvedBatch};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointFlow {
    pub publication: PublicationBinding,
    pub publication_route: PublicationRouteSnapshotV1,
    pub ordered_route: BatchRoute,
    pub runtime_route: Option<BatchRoute>,
}

impl CheckpointFlow {
    pub fn try_from_resolved(batch: &ResolvedBatch) -> Result<Self, RejectClass> {
        if batch.published.batch_id != batch.ordered.batch_id
            || batch.ordered.batch_id != batch.ordered.planned.batch_id
        {
            return Err(RejectClass::ReconcileInvalid);
        }

        let checkpoint_id =
            derive_checkpoint_id(batch.artifact()).map_err(|_| RejectClass::ArtifactVersion)?;
        if batch.published.checkpoint_id != checkpoint_id {
            return Err(RejectClass::ReconcileInvalid);
        }
        if batch.link().checkpoint_id() != batch.published.checkpoint_id {
            return Err(RejectClass::ReconcileInvalid);
        }

        if batch.published.pub_in != batch.artifact().pub_in() {
            return Err(RejectClass::StateRootMismatch);
        }

        if let Some(exec_ticket) = batch.runtime_exec() {
            if exec_ticket.batch_id != batch.published.batch_id {
                return Err(RejectClass::ReconcileInvalid);
            }
        }

        let runtime_route = batch.runtime_placement().map(|placement| placement.route);
        if let Some(route) = runtime_route {
            if route != batch.ordered.planned.route {
                return Err(RejectClass::ReconcileInvalid);
            }
        }
        check_route_binding_v1(
            &batch.published.publication_route,
            batch.ordered.planned.route_table_digest.into_bytes(),
            Some(batch.published.publication_checkpoint),
            Some((
                batch.ordered.planned.route.shard_id.as_u32(),
                batch.ordered.planned.route.routing_generation,
            )),
        )
        .map_err(|_| RejectClass::ReconcileInvalid)?;

        Ok(Self {
            publication: bind_publication_contract(
                batch.published.batch_id,
                batch.published.checkpoint_id,
                batch.ordered.planned.route_table_digest.into_bytes(),
                &batch.published.pub_in,
            ),
            publication_route: batch.published.publication_route.clone(),
            ordered_route: batch.ordered.planned.route,
            runtime_route,
        })
    }

    #[must_use]
    pub const fn binding_digest(&self) -> [u8; 32] {
        self.publication.binding_digest()
    }
}
