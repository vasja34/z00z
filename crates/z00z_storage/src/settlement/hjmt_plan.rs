use std::collections::{BTreeMap, BTreeSet};

use jmt::KeyHash;

use crate::backend::{
    codec::{key_from_terminal_id, leaf_payload, path_payload},
    memory::KeyValueOp,
    roots::HjmtBucketKey,
};
use crate::settlement::tx_plan_types::{NextState, ObjectDeltaSetV1};

use super::{
    model::SettlementModel, tree_id::HjmtTreeId, DefinitionId, SerialId, SettlementPath,
    SettlementStateRoot, SettlementStore, SettlementStoreError, StoreItem, StoreOp, TerminalId,
};
use crate::settlement::BucketId;

#[derive(Clone, Debug)]
pub(super) struct HjmtBatch {
    pub(super) tree_id: HjmtTreeId,
    pub(super) ops: Vec<KeyValueOp>,
}

impl HjmtBatch {
    fn new(tree_id: HjmtTreeId) -> Self {
        Self {
            tree_id,
            ops: Vec::new(),
        }
    }

    pub(super) fn push(&mut self, key: KeyHash, value: Option<Vec<u8>>) {
        self.ops.push((key, value));
    }

    pub(super) fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
}

pub(crate) struct HjmtPlan {
    pub(super) ops: Vec<StoreOp>,
    pub(super) delta_set: ObjectDeltaSetV1,
    pub(super) next: NextState,
    pub(super) live_model: SettlementModel,
    pub(super) root: SettlementStateRoot,
    pub(super) has_live_definitions: bool,
    pub(super) terminal_batches: Vec<HjmtBatch>,
    pub(super) touched_buckets: Vec<HjmtBucketKey>,
    pub(super) path_batch: HjmtBatch,
}

impl HjmtPlan {
    pub(super) fn has_ops(&self) -> bool {
        !self.terminal_batches.is_empty() || !self.path_batch.is_empty()
    }
}

impl SettlementStore {
    pub(super) fn hjmt_plan_ops(&self, ops: &[StoreOp]) -> Result<HjmtPlan, SettlementStoreError> {
        self.hjmt_plan_ops_with_delta(ops, None, None)
    }

    pub(super) fn hjmt_plan_ops_with_delta(
        &self,
        ops: &[StoreOp],
        delta_set: Option<ObjectDeltaSetV1>,
        fee_envelope: Option<super::FeeEnvelope>,
    ) -> Result<HjmtPlan, SettlementStoreError> {
        let (paths, next) = self.precheck_ops(ops)?;
        let scope = PlanScope::from_paths(&paths);
        let mut semantic_model = self.model.clone();
        semantic_model.merge_scope(next.model.clone(), &next.def_ids);
        let root = semantic_model.root()?;
        let prior_root = self.hjmt_root()?;
        let delta_set = match delta_set {
            Some(delta_set) => {
                self.bind_object_delta(ops, &next, prior_root, root, delta_set, fee_envelope)?
            }
            None => self.compat_object_delta(ops, &next, prior_root, root, fee_envelope)?,
        };
        let has_live_definitions = !semantic_model.def_ids().is_empty();
        let mut terminal = BTreeMap::<HjmtBucketKey, HjmtBatch>::new();

        for shard in &scope.shards {
            for terminal_id in &shard.terminal_ids {
                let old_path = self.path_by_terminal_id.get(terminal_id).copied();
                let new_path = next.path_opt(self, terminal_id);
                let old_item = item_for_path(&self.model, old_path)?;
                let new_item = item_for_path(&next.model, new_path)?;
                if old_item == new_item {
                    continue;
                }

                if let Some(new_item) = new_item {
                    let path = new_item.path();
                    let bucket_id = self.bucket_policy().derive_bucket_id(path);
                    let batch = terminal
                        .entry(hjmt_bucket_key(path, bucket_id))
                        .or_insert_with(|| terminal_batch(path, bucket_id));
                    batch.push(
                        key_from_terminal_id(path.terminal_id()),
                        Some(leaf_payload(new_item.leaf())?),
                    );
                    continue;
                }

                if let Some(path) = old_path {
                    let bucket_id = self.bucket_policy().derive_bucket_id(path);
                    let batch = terminal
                        .entry(hjmt_bucket_key(path, bucket_id))
                        .or_insert_with(|| terminal_batch(path, bucket_id));
                    batch.push(key_from_terminal_id(path.terminal_id()), None);
                }
            }
        }

        let mut path_batch = HjmtBatch::new(HjmtTreeId::PathIndex);
        for terminal_id in &scope.terminal_path_ids {
            let old_path = self.path_by_terminal_id.get(terminal_id).copied();
            let new_path = next.path_opt(self, terminal_id);
            if old_path != new_path {
                path_batch.push(
                    key_from_terminal_id(*terminal_id),
                    new_path.map(path_payload).transpose()?,
                );
            }
        }

        let touched_buckets = terminal.keys().copied().collect();
        let terminal_batches = terminal.into_values().collect();

        Ok(HjmtPlan {
            ops: ops.to_vec(),
            delta_set,
            next,
            live_model: semantic_model,
            root,
            has_live_definitions,
            terminal_batches,
            touched_buckets,
            path_batch,
        })
    }
}

fn item_for_path(
    model: &SettlementModel,
    path: Option<SettlementPath>,
) -> Result<Option<StoreItem>, SettlementStoreError> {
    match path {
        Some(path) => Ok(model.item_opt(&path)?),
        None => Ok(None),
    }
}

fn hjmt_bucket_key(path: SettlementPath, bucket_id: BucketId) -> HjmtBucketKey {
    (path.definition_id, path.serial_id, bucket_id)
}

fn terminal_batch(path: SettlementPath, bucket_id: BucketId) -> HjmtBatch {
    HjmtBatch::new(HjmtTreeId::BucketTerminal(
        path.definition_id,
        path.serial_id,
        bucket_id,
    ))
}

pub(super) fn bucket_key_for_path(bucket_id: BucketId) -> KeyHash {
    KeyHash(bucket_id.into_bytes())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Touch {
    definition_id: DefinitionId,
    serial_id: SerialId,
    terminal_id: TerminalId,
}

impl Touch {
    const fn from_path(path: SettlementPath) -> Self {
        Self {
            definition_id: path.definition_id,
            serial_id: path.serial_id,
            terminal_id: path.terminal_id(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ShardKey {
    definition_id: DefinitionId,
    serial_id: SerialId,
}

impl ShardKey {
    const fn new(definition_id: DefinitionId, serial_id: SerialId) -> Self {
        Self {
            definition_id,
            serial_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ShardItem {
    terminal_ids: Vec<TerminalId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PlanScope {
    shards: Vec<ShardItem>,
    terminal_path_ids: Vec<TerminalId>,
}

impl PlanScope {
    fn from_paths(paths: &[SettlementPath]) -> Self {
        let touched = paths
            .iter()
            .copied()
            .map(Touch::from_path)
            .collect::<BTreeSet<_>>();
        let mut shard_map: BTreeMap<ShardKey, BTreeSet<TerminalId>> = BTreeMap::new();
        let mut terminal_path_ids = BTreeSet::new();

        for touch in touched {
            let shard_key = ShardKey::new(touch.definition_id, touch.serial_id);
            shard_map
                .entry(shard_key)
                .or_default()
                .insert(touch.terminal_id);
            terminal_path_ids.insert(touch.terminal_id);
        }

        let shards = shard_map
            .into_values()
            .map(|terminal_ids| ShardItem {
                terminal_ids: terminal_ids.into_iter().collect(),
            })
            .collect();

        Self {
            shards,
            terminal_path_ids: terminal_path_ids.into_iter().collect(),
        }
    }
}
