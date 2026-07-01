use std::collections::BTreeSet;

use super::{
    store::SettlementStore,
    tx_plan_types::{
        NextState, ObjectDeltaSetV1, SeenOps, SettlementActionV1, SettlementObjectDeltaV1,
        StoreSnap,
    },
    DefinitionId, FeeEnvelope, SettlementPath, SettlementStateRoot, SettlementStoreError,
    StoreItem, StoreOp,
};

type MaterializedObjectDelta = (
    Vec<SettlementObjectDeltaV1>,
    Vec<SettlementObjectDeltaV1>,
    Vec<SettlementObjectDeltaV1>,
);

impl SettlementStore {
    pub(super) fn chk_path_binding_next(
        &self,
        next: &NextState,
        path: SettlementPath,
    ) -> Result<(), SettlementStoreError> {
        if let Some(existing) = next.path_opt(self, &path.terminal_id()) {
            if existing != path {
                return Err(SettlementStoreError::PathTerminalMix);
            }
        }

        Ok(())
    }

    pub(super) fn require_exact_next(
        &self,
        next: &NextState,
        path: &SettlementPath,
    ) -> Result<(), SettlementStoreError> {
        let existing = next
            .path_opt(self, &path.terminal_id())
            .ok_or(SettlementStoreError::PathMiss)?;
        if existing == *path {
            return Ok(());
        }

        Err(SettlementStoreError::PathMiss)
    }

    pub(super) fn snap_store(&self) -> StoreSnap {
        StoreSnap {
            flat_inner: self.flat_store.snap(),
            flat_version: self.flat_version,
            flat_root: self.flat_root,
            model: self.model.clone(),
            tree_roots: self.tree_roots.clone(),
            path_by_terminal_id: self.path_by_terminal_id.clone(),
            nullifier: self.nullifier.clone(),
            claim_null_seq: self.claim_null_seq,
            fee_replays: self.fee_replays.clone(),
            fee_replay_seq: self.fee_replay_seq,
            settlement_root_by_ver: self.settlement_root_by_ver.clone(),
            model_by_ver: self.model_by_ver.clone(),
            hjmt_roots_by_ver: self.hjmt_roots_by_ver.clone(),
            last_object_delta: self.last_object_delta.clone(),
            object_deltas_by_ver: self.object_deltas_by_ver.clone(),
        }
    }

    pub(super) fn restore_store(&mut self, snap: StoreSnap) {
        self.flat_store.restore(snap.flat_inner);
        self.flat_version = snap.flat_version;
        self.flat_root = snap.flat_root;
        self.model = snap.model;
        self.tree_roots = snap.tree_roots;
        self.path_by_terminal_id = snap.path_by_terminal_id;
        self.nullifier = snap.nullifier;
        self.claim_null_seq = snap.claim_null_seq;
        self.fee_replays = snap.fee_replays;
        self.fee_replay_seq = snap.fee_replay_seq;
        self.settlement_root_by_ver = snap.settlement_root_by_ver;
        self.model_by_ver = snap.model_by_ver;
        self.hjmt_roots_by_ver = snap.hjmt_roots_by_ver;
        self.last_object_delta = snap.last_object_delta;
        self.object_deltas_by_ver = snap.object_deltas_by_ver;
    }

    pub(super) fn precheck_ops(
        &self,
        ops: &[StoreOp],
    ) -> Result<(Vec<SettlementPath>, NextState), SettlementStoreError> {
        let mut seen = SeenOps::default();

        for op in ops {
            let path = self.op_path(op)?;
            seen.touch(path)?;
        }

        let paths = seen.into_paths();
        let def_ids = def_ids_for_paths(&paths);
        let mut next = NextState::new(self, &def_ids, paths.len());

        for op in ops {
            self.apply_op(op, &mut next)?;
        }

        Ok((paths, next))
    }

    pub(crate) fn load_item(
        &self,
        path: &SettlementPath,
    ) -> Result<StoreItem, SettlementStoreError> {
        self.hjmt_get_settlement_item(path)?
            .ok_or(SettlementStoreError::PathMiss)
    }

    pub(super) fn compat_object_delta(
        &self,
        ops: &[StoreOp],
        next: &NextState,
        prior_root: SettlementStateRoot,
        expected_new_root: SettlementStateRoot,
        fee_envelope: Option<FeeEnvelope>,
    ) -> Result<ObjectDeltaSetV1, SettlementStoreError> {
        let (deleted_objects, created_objects, updated_objects) =
            self.materialize_object_delta(ops, next)?;
        let delta = ObjectDeltaSetV1::new(
            SettlementActionV1::CompatibilityStoreOps,
            [0u8; 32],
            None,
            deleted_objects,
            created_objects,
            updated_objects,
            fee_envelope,
            prior_root,
            expected_new_root,
        );
        delta.check_contract()?;
        Ok(delta)
    }

    pub(super) fn bind_object_delta(
        &self,
        ops: &[StoreOp],
        next: &NextState,
        prior_root: SettlementStateRoot,
        expected_new_root: SettlementStateRoot,
        mut delta: ObjectDeltaSetV1,
        fee_envelope: Option<FeeEnvelope>,
    ) -> Result<ObjectDeltaSetV1, SettlementStoreError> {
        let (expected_deleted, expected_created, expected_updated) =
            self.materialize_object_delta(ops, next)?;
        delta.prior_root = prior_root;
        delta.expected_new_root = expected_new_root;
        delta.fee_envelope = fee_envelope;
        check_delta_section("deleted", &expected_deleted, &delta.deleted_objects)?;
        check_delta_section("created", &expected_created, &delta.created_objects)?;
        check_delta_section("updated", &expected_updated, &delta.updated_objects)?;
        delta.check_contract()?;
        Ok(delta)
    }

    fn op_path(&self, op: &StoreOp) -> Result<SettlementPath, SettlementStoreError> {
        Ok(match op {
            StoreOp::Put(item) => {
                item.check_path()?;
                item.path()
            }
            StoreOp::Delete(path) => *path,
        })
    }

    fn apply_op(&self, op: &StoreOp, next: &mut NextState) -> Result<(), SettlementStoreError> {
        match op {
            StoreOp::Put(item) => self.apply_put(item, next),
            StoreOp::Delete(path) => self.apply_del(path, next),
        }
    }

    fn apply_put(
        &self,
        item: &StoreItem,
        next: &mut NextState,
    ) -> Result<(), SettlementStoreError> {
        let path = item.path();
        self.chk_path_binding_next(next, path)?;
        next.model.put_leaf(item.clone())?;
        next.terminal_path_ops
            .insert(path.terminal_id(), Some(path));
        Ok(())
    }

    fn apply_del(
        &self,
        path: &crate::settlement::SettlementPath,
        next: &mut NextState,
    ) -> Result<(), SettlementStoreError> {
        self.require_exact_next(next, path)?;
        let _ = next
            .model
            .item_opt(path)?
            .ok_or(SettlementStoreError::PathMiss)?;
        next.model.del_leaf(path)?;
        next.terminal_path_ops.insert(path.terminal_id(), None);
        Ok(())
    }

    fn materialize_object_delta(
        &self,
        ops: &[StoreOp],
        next: &NextState,
    ) -> Result<MaterializedObjectDelta, SettlementStoreError> {
        let mut deleted_objects = Vec::new();
        let mut created_objects = Vec::new();
        let mut updated_objects = Vec::new();

        for op in ops {
            match op {
                StoreOp::Put(item) => {
                    let path = item.path();
                    let prior = self.model.item_opt(&path)?;
                    let next_item = next.model.item_opt(&path)?.ok_or_else(|| {
                        SettlementStoreError::ObjectDelta(
                            "typed object delta next model is missing a put path".to_string(),
                        )
                    })?;
                    if let Some(prior) = prior {
                        updated_objects.push(SettlementObjectDeltaV1::updated(
                            path,
                            prior.leaf().clone(),
                            next_item.leaf().clone(),
                            None,
                        ));
                    } else {
                        created_objects.push(SettlementObjectDeltaV1::created(
                            path,
                            next_item.leaf().clone(),
                            None,
                        ));
                    }
                }
                StoreOp::Delete(path) => {
                    let prior = self
                        .model
                        .item_opt(path)?
                        .ok_or(SettlementStoreError::PathMiss)?;
                    deleted_objects.push(SettlementObjectDeltaV1::deleted(
                        *path,
                        prior.leaf().clone(),
                        None,
                    ));
                }
            }
        }

        Ok((deleted_objects, created_objects, updated_objects))
    }
}

fn check_delta_section(
    name: &str,
    expected: &[SettlementObjectDeltaV1],
    got: &[SettlementObjectDeltaV1],
) -> Result<(), SettlementStoreError> {
    if expected.len() != got.len() {
        return Err(SettlementStoreError::ObjectDelta(format!(
            "typed object delta {name} section length drifted from live store ops"
        )));
    }

    for (expect, got) in expected.iter().zip(got.iter()) {
        if expect.path != got.path
            || expect.object_kind != got.object_kind
            || expect.prior_leaf != got.prior_leaf
            || expect.next_leaf != got.next_leaf
        {
            return Err(SettlementStoreError::ObjectDelta(format!(
                "typed object delta {name} section drifted from live store ops"
            )));
        }
    }

    Ok(())
}

fn def_ids_for_paths(paths: &[SettlementPath]) -> Vec<DefinitionId> {
    paths
        .iter()
        .map(|path| path.definition_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
