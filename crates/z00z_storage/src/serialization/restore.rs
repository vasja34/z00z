use std::collections::{BTreeMap, BTreeSet};

use crate::{
    error::SerializationError,
    serialization::{JmtSerArtifact, JmtSerEdge, JmtSerNode, JmtSerTreeId},
};

type TreeGroupKey = (u8, [u8; 32], u32, [u8; 32]);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JmtTreeState {
    pub tree_id: JmtSerTreeId,
    pub root: [u8; 32],
    pub jmt_root: [u8; 32],
    pub node_ids: Vec<[u8; 32]>,
    pub edge_ids: Vec<([u8; 32], [u8; 32])>,
    pub is_root_bound: bool,
}

impl JmtTreeState {
    fn new(
        tree_id: JmtSerTreeId,
        root: [u8; 32],
        jmt_root: [u8; 32],
        node_ids: Vec<[u8; 32]>,
        edge_ids: Vec<([u8; 32], [u8; 32])>,
        is_root_bound: bool,
    ) -> Self {
        Self {
            tree_id,
            root,
            jmt_root,
            node_ids,
            edge_ids,
            is_root_bound,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JmtRestore {
    artifact: JmtSerArtifact,
    trees: Vec<JmtTreeState>,
}

impl JmtRestore {
    #[must_use]
    pub fn trees(&self) -> &[JmtTreeState] {
        &self.trees
    }

    #[must_use]
    pub fn artifact(&self) -> &JmtSerArtifact {
        &self.artifact
    }

    #[must_use]
    pub fn node_count(&self) -> usize {
        self.artifact.nodes.len()
    }

    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.artifact.edges.len()
    }

    #[must_use]
    pub fn tree_count(&self) -> usize {
        self.trees.len()
    }
}

pub fn restore_artifact(artifact: &JmtSerArtifact) -> Result<JmtRestore, SerializationError> {
    if artifact.meta.node_count as usize != artifact.nodes.len() {
        return Err(SerializationError::RebuildMix);
    }
    if artifact.meta.edge_count as usize != artifact.edges.len() {
        return Err(SerializationError::RebuildMix);
    }

    let mut node_map = BTreeMap::new();
    for node in &artifact.nodes {
        if node_map.insert(node.id, node).is_some() {
            return Err(SerializationError::RebuildMix);
        }
    }

    let mut seen_paths = BTreeSet::new();
    for path in &artifact.meta.path_order {
        if !seen_paths.insert(*path) {
            return Err(SerializationError::RebuildMix);
        }
    }

    let root_keys: BTreeSet<_> = artifact
        .roots
        .trees
        .iter()
        .map(|tree_root| tree_key(&tree_root.tree_id))
        .collect();
    if root_keys.len() != artifact.roots.trees.len() {
        return Err(SerializationError::RebuildMix);
    }

    let mut edges_by_tree: BTreeMap<TreeGroupKey, Vec<&JmtSerEdge>> = BTreeMap::new();
    for edge in &artifact.edges {
        let Some(parent) = node_map.get(&edge.parent) else {
            return Err(SerializationError::RebuildMix);
        };
        let Some(child) = node_map.get(&edge.child) else {
            return Err(SerializationError::RebuildMix);
        };
        if parent.tree_id != edge.tree_id || child.tree_id != edge.tree_id {
            return Err(SerializationError::RebuildMix);
        }

        edges_by_tree
            .entry(tree_key(&edge.tree_id))
            .or_default()
            .push(edge);
    }

    let mut nodes_by_tree: BTreeMap<TreeGroupKey, Vec<&JmtSerNode>> = BTreeMap::new();
    for node in &artifact.nodes {
        nodes_by_tree
            .entry(tree_key(&node.tree_id))
            .or_default()
            .push(node);
    }

    if nodes_by_tree.keys().any(|key| !root_keys.contains(key)) {
        return Err(SerializationError::RebuildMix);
    }
    if edges_by_tree.keys().any(|key| !root_keys.contains(key)) {
        return Err(SerializationError::RebuildMix);
    }

    let mut trees = Vec::with_capacity(artifact.roots.trees.len());
    for tree_root in &artifact.roots.trees {
        let key = tree_key(&tree_root.tree_id);
        let tree_nodes = nodes_by_tree
            .get(&key)
            .ok_or(SerializationError::RebuildMix)?;
        let tree_edges = edges_by_tree.get(&key).cloned().unwrap_or_default();

        let mut indegree = BTreeMap::new();
        for node in tree_nodes {
            indegree.insert(node.id, 0usize);
        }

        let mut children = BTreeMap::<[u8; 32], Vec<[u8; 32]>>::new();
        let mut seen_edges = BTreeSet::new();
        for edge in &tree_edges {
            if !seen_edges.insert((edge.parent, edge.child, edge.slot)) {
                return Err(SerializationError::RebuildMix);
            }
            let Some(child_degree) = indegree.get_mut(&edge.child) else {
                return Err(SerializationError::RebuildMix);
            };
            *child_degree += 1;
            children.entry(edge.parent).or_default().push(edge.child);
        }

        let root_ids: Vec<[u8; 32]> = indegree
            .iter()
            .filter_map(|(node_id, degree)| (*degree == 0).then_some(*node_id))
            .collect();
        if root_ids.len() != 1 {
            return Err(SerializationError::RootMix);
        }

        let root_id = root_ids[0];
        let root_node = node_map
            .get(&root_id)
            .ok_or(SerializationError::RebuildMix)?;
        if tree_root.root != tree_root.jmt_root {
            return Err(SerializationError::RootMix);
        }
        if root_node.node_hash != tree_root.jmt_root {
            return Err(SerializationError::RootMix);
        }
        if tree_edges.len() + 1 != tree_nodes.len() {
            return Err(SerializationError::RebuildMix);
        }

        let mut stack = vec![root_id];
        let mut visited = BTreeSet::new();
        while let Some(node_id) = stack.pop() {
            if !visited.insert(node_id) {
                continue;
            }
            if let Some(next_ids) = children.get(&node_id) {
                stack.extend(next_ids.iter().copied());
            }
        }
        if visited.len() != tree_nodes.len() {
            return Err(SerializationError::RebuildMix);
        }

        let mut node_ids: Vec<[u8; 32]> = nodes_by_tree
            .get(&key)
            .into_iter()
            .flat_map(|nodes| nodes.iter().map(|node| node.id))
            .collect();
        let mut edge_ids: Vec<([u8; 32], [u8; 32])> = edges_by_tree
            .get(&key)
            .into_iter()
            .flat_map(|edges| edges.iter().map(|edge| (edge.parent, edge.child)))
            .collect();

        node_ids.sort();
        edge_ids.sort();

        trees.push(JmtTreeState::new(
            tree_root.tree_id,
            tree_root.root,
            tree_root.jmt_root,
            node_ids,
            edge_ids,
            true,
        ));
    }

    Ok(JmtRestore {
        artifact: artifact.clone(),
        trees,
    })
}

fn tree_key(tree_id: &JmtSerTreeId) -> (u8, [u8; 32], u32, [u8; 32]) {
    match tree_id {
        JmtSerTreeId::Definition => (0, [0u8; 32], 0, [0u8; 32]),
        JmtSerTreeId::Serial(definition_id) => (1, definition_id.into_bytes(), 0, [0u8; 32]),
        JmtSerTreeId::Bucket {
            definition_id,
            serial_id,
        } => (2, definition_id.into_bytes(), serial_id.get(), [0u8; 32]),
        JmtSerTreeId::Terminal {
            definition_id,
            serial_id,
            bucket_id,
        } => (
            3,
            definition_id.into_bytes(),
            serial_id.get(),
            bucket_id.into_bytes(),
        ),
        JmtSerTreeId::PathIndex => (4, [0u8; 32], 0, [0u8; 32]),
    }
}
