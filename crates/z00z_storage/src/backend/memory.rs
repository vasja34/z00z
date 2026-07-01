use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap},
};

use jmt::{
    storage::{LeafNode, Node, NodeKey, StaleNodeIndex, TreeReader, TreeUpdateBatch, TreeWriter},
    KeyHash, Version,
};

use crate::backend::codec::map_jmt_err;
use crate::backend::types::SettlementStoreError;

pub(crate) type StoredValue = Option<Vec<u8>>;
pub(crate) type VersionedValue = (Version, StoredValue);
pub(crate) type KeyValueOp = (KeyHash, StoredValue);
pub(crate) type ValueHistory = HashMap<KeyHash, Vec<VersionedValue>>;

#[derive(Clone, Default)]
pub(crate) struct MemTreeInner {
    pub(crate) nodes: HashMap<NodeKey, Node>,
    pub(crate) stale_nodes: BTreeSet<StaleNodeIndex>,
    pub(crate) value_history: ValueHistory,
}

#[derive(Default)]
pub struct MemTreeStore {
    pub(super) inner: RefCell<MemTreeInner>,
}

impl TreeReader for MemTreeStore {
    fn get_node_option(&self, node_key: &NodeKey) -> anyhow::Result<Option<Node>> {
        Ok(self.inner.borrow().nodes.get(node_key).cloned())
    }

    fn get_value_option(
        &self,
        max_version: Version,
        key_hash: KeyHash,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        let inner = self.inner.borrow();
        let Some(history) = inner.value_history.get(&key_hash) else {
            return Ok(None);
        };

        for (version, value) in history.iter().rev() {
            if *version <= max_version {
                return Ok(value.clone());
            }
        }

        Ok(None)
    }

    fn get_rightmost_leaf(&self) -> anyhow::Result<Option<(NodeKey, LeafNode)>> {
        let inner = self.inner.borrow();
        let mut best: Option<(NodeKey, LeafNode)> = None;

        for (node_key, node) in &inner.nodes {
            let Node::Leaf(leaf_node) = node else {
                continue;
            };

            if best
                .as_ref()
                .map(|(_, current)| leaf_node.key_hash() > current.key_hash())
                .unwrap_or(true)
            {
                best = Some((node_key.clone(), leaf_node.clone()));
            }
        }

        Ok(best)
    }
}

impl TreeWriter for MemTreeStore {
    fn write_node_batch(&self, node_batch: &jmt::storage::NodeBatch) -> anyhow::Result<()> {
        let mut inner = self.inner.borrow_mut();

        for (node_key, node) in node_batch.nodes() {
            inner.nodes.insert(node_key.clone(), node.clone());
        }

        for ((version, key_hash), value) in node_batch.values() {
            put_value(&mut inner.value_history, *version, *key_hash, value.clone());
        }

        Ok(())
    }
}

impl MemTreeStore {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn snap(&self) -> MemTreeInner {
        self.inner.borrow().clone()
    }

    pub(crate) fn restore(&self, inner: MemTreeInner) {
        *self.inner.borrow_mut() = inner;
    }

    fn write_tree_update_batch(&self, batch: TreeUpdateBatch) -> Result<(), SettlementStoreError> {
        self.write_node_batch(&batch.node_batch)
            .map_err(map_jmt_err)?;

        let mut inner = self.inner.borrow_mut();
        for stale in batch.stale_node_index_batch {
            inner.stale_nodes.insert(stale);
        }

        Ok(())
    }
}

fn put_value(
    value_history: &mut ValueHistory,
    version: Version,
    key_hash: KeyHash,
    value: StoredValue,
) {
    let history = value_history.entry(key_hash).or_default();

    if let Some((last_version, last_value)) = history.last_mut() {
        match version.cmp(last_version) {
            std::cmp::Ordering::Less => return,
            std::cmp::Ordering::Equal => {
                *last_value = value;
                return;
            }
            std::cmp::Ordering::Greater => {}
        }
    }

    history.push((version, value));
}

pub(crate) fn apply_batch(
    store: &MemTreeStore,
    batch: TreeUpdateBatch,
) -> Result<(), SettlementStoreError> {
    store.write_tree_update_batch(batch)
}
