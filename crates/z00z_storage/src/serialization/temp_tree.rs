use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
};

use jmt::{
    storage::{LeafNode, Node, NodeKey, TreeReader, TreeWriter},
    KeyHash, Version,
};

type TmpHist = HashMap<KeyHash, Vec<(Version, Option<Vec<u8>>)>>;

#[derive(Clone, Debug)]
pub(super) struct TreeLeaf {
    pub(super) key_hash: KeyHash,
    pub(super) raw_key: Vec<u8>,
    pub(super) payload: Vec<u8>,
}

#[derive(Clone, Default)]
struct TempTreeInner {
    nodes: BTreeMap<NodeKey, Node>,
    hist: TmpHist,
}

#[derive(Default)]
pub(super) struct TempTreeStore {
    inner: RefCell<TempTreeInner>,
}

impl TreeReader for TempTreeStore {
    fn get_node_option(&self, node_key: &NodeKey) -> anyhow::Result<Option<Node>> {
        Ok(self.inner.borrow().nodes.get(node_key).cloned())
    }

    fn get_value_option(
        &self,
        max_ver: Version,
        key_hash: KeyHash,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        let inner = self.inner.borrow();
        let Some(hist) = inner.hist.get(&key_hash) else {
            return Ok(None);
        };

        for (ver, value) in hist.iter().rev() {
            if *ver <= max_ver {
                return Ok(value.clone());
            }
        }

        Ok(None)
    }

    fn get_rightmost_leaf(&self) -> anyhow::Result<Option<(NodeKey, LeafNode)>> {
        let inner = self.inner.borrow();
        let mut best: Option<(NodeKey, LeafNode)> = None;

        for (node_key, node) in &inner.nodes {
            let Node::Leaf(leaf) = node else {
                continue;
            };

            if best
                .as_ref()
                .map(|(_, current)| leaf.key_hash() > current.key_hash())
                .unwrap_or(true)
            {
                best = Some((node_key.clone(), leaf.clone()));
            }
        }

        Ok(best)
    }
}

impl TreeWriter for TempTreeStore {
    fn write_node_batch(&self, node_batch: &jmt::storage::NodeBatch) -> anyhow::Result<()> {
        let mut inner = self.inner.borrow_mut();

        for (node_key, node) in node_batch.nodes() {
            inner.nodes.insert(node_key.clone(), node.clone());
        }

        for ((ver, key_hash), value) in node_batch.values() {
            push_hist(&mut inner.hist, *ver, *key_hash, value.clone());
        }

        Ok(())
    }
}

impl TempTreeStore {
    pub(super) fn root_key(&self) -> Option<NodeKey> {
        self.inner
            .borrow()
            .nodes
            .keys()
            .find(|key| key.nibble_path().is_empty())
            .cloned()
    }

    pub(super) fn node(&self, node_key: &NodeKey) -> Option<Node> {
        self.inner.borrow().nodes.get(node_key).cloned()
    }

    pub(super) fn leaf_bytes(&self, key_hash: KeyHash, ver: Version) -> Option<Vec<u8>> {
        self.get_value_option(ver, key_hash).ok().flatten()
    }
}

fn push_hist(hist: &mut TmpHist, ver: Version, key_hash: KeyHash, value: Option<Vec<u8>>) {
    let slot = hist.entry(key_hash).or_default();
    if let Some((last_ver, last_value)) = slot.last_mut() {
        match ver.cmp(last_ver) {
            std::cmp::Ordering::Less => return,
            std::cmp::Ordering::Equal => {
                *last_value = value;
                return;
            }
            std::cmp::Ordering::Greater => {}
        }
    }

    slot.push((ver, value));
}
