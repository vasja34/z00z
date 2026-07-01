use std::collections::{BTreeMap, HashMap};

use jmt::{
    storage::{Node, NodeKey, TreeWriter},
    KeyHash, Sha256Jmt,
};
use sha2::{Digest, Sha256};
use z00z_utils::codec::{BincodeCodec, Codec};

use super::temp_tree::{TempTreeStore, TreeLeaf};
use crate::{
    error::SerializationError,
    serialization::{
        JmtSerArtifact, JmtSerEdge, JmtSerMeta, JmtSerNode, JmtSerNodeKind, JmtSerRoots,
        JmtSerTreeId, JmtSerTreeRoot, JmtSerVersion,
    },
    settlement::{
        keys::{definition_key, serial_key},
        BucketId, SettlementStateRoot, SettlementStore, TerminalId,
    },
};

pub fn build_artifact(store: &SettlementStore) -> Result<JmtSerArtifact, SerializationError> {
    let sem_root = store
        .settlement_root()
        .map_err(|err| SerializationError::Backend(err.to_string()))?;
    build_artifact_from_forest(store, sem_root)
}

fn build_artifact_from_forest(
    store: &SettlementStore,
    sem_root: SettlementStateRoot,
) -> Result<JmtSerArtifact, SerializationError> {
    let def_root = store.ser_hjmt_def_root();
    let serial_roots = store.ser_hjmt_serial_roots();
    let bucket_roots = store.ser_hjmt_bucket_roots();
    let terminal_roots = store.ser_hjmt_terminal_roots();
    let def_rows = store
        .ser_hjmt_def_rows()
        .map_err(|err| SerializationError::Backend(err.to_string()))?;
    let serial_rows = store
        .ser_hjmt_serial_rows()
        .map_err(|err| SerializationError::Backend(err.to_string()))?;
    let bucket_rows = store
        .ser_hjmt_bucket_rows()
        .map_err(|err| SerializationError::Backend(err.to_string()))?;
    let terminal_rows = store
        .ser_hjmt_terminal_rows()
        .map_err(|err| SerializationError::Backend(err.to_string()))?;
    let path_rows = store
        .ser_hjmt_settlement_path_rows()
        .map_err(|err| SerializationError::Backend(err.to_string()))?;

    let path_order = store.ser_hjmt_path_order();

    let mut roots = Vec::new();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    if !def_rows.is_empty() {
        let tree_id = JmtSerTreeId::Definition;
        let expect = def_root.ok_or(SerializationError::RootMix)?;
        let leaves: Vec<_> = def_rows
            .iter()
            .map(|(def_id, payload)| TreeLeaf {
                key_hash: definition_key(*def_id),
                raw_key: def_id.as_bytes().to_vec(),
                payload: payload.clone(),
            })
            .collect();
        roots.push(build_tree(
            tree_id, expect, &leaves, &mut nodes, &mut edges,
        )?);
    }

    for (definition_id, root) in &serial_roots {
        let tree_id = JmtSerTreeId::Serial(*definition_id);
        let leaves: Vec<_> = serial_rows
            .iter()
            .filter(|(row_def_id, _, _)| row_def_id == definition_id)
            .map(|(_, serial_id, payload)| TreeLeaf {
                key_hash: serial_key(*definition_id, *serial_id),
                raw_key: serial_id.get().to_le_bytes().to_vec(),
                payload: payload.clone(),
            })
            .collect();
        roots.push(build_tree(tree_id, *root, &leaves, &mut nodes, &mut edges)?);
    }

    for ((definition_id, serial_id), root) in &bucket_roots {
        let tree_id = JmtSerTreeId::Bucket {
            definition_id: *definition_id,
            serial_id: *serial_id,
        };
        let leaves: Vec<_> = bucket_rows
            .iter()
            .filter(|(row_def_id, row_serial_id, _, _)| {
                row_def_id == definition_id && row_serial_id == serial_id
            })
            .map(|(_, _, bucket_id, payload)| TreeLeaf {
                key_hash: bucket_key(*bucket_id),
                raw_key: bucket_id.as_bytes().to_vec(),
                payload: payload.clone(),
            })
            .collect();
        roots.push(build_tree(tree_id, *root, &leaves, &mut nodes, &mut edges)?);
    }

    for ((definition_id, serial_id, bucket_id), root) in &terminal_roots {
        let tree_id = JmtSerTreeId::Terminal {
            definition_id: *definition_id,
            serial_id: *serial_id,
            bucket_id: *bucket_id,
        };
        let leaves: Vec<_> = terminal_rows
            .iter()
            .filter(|(path, row_bucket_id, _)| {
                path.definition_id == *definition_id
                    && path.serial_id == *serial_id
                    && row_bucket_id == bucket_id
            })
            .map(|(path, _, payload)| TreeLeaf {
                key_hash: terminal_key(path.terminal_id),
                raw_key: path.terminal_id.as_bytes().to_vec(),
                payload: payload.clone(),
            })
            .collect();
        roots.push(build_tree(tree_id, *root, &leaves, &mut nodes, &mut edges)?);
    }

    if !path_rows.is_empty() {
        let tree_id = JmtSerTreeId::PathIndex;
        let leaves: Vec<_> = path_rows
            .iter()
            .map(|(path, payload)| TreeLeaf {
                key_hash: terminal_key(path.terminal_id),
                raw_key: path.terminal_id.as_bytes().to_vec(),
                payload: payload.clone(),
            })
            .collect();
        let path_root = calc_root(&leaves)?;
        roots.push(build_tree(
            tree_id, path_root, &leaves, &mut nodes, &mut edges,
        )?);
    }

    roots.sort_by_key(root_sort_key);
    nodes.sort_by_key(node_sort_key);
    edges.sort_by_key(edge_sort_key);

    let meta = JmtSerMeta::new(
        path_order,
        u32::try_from(nodes.len()).map_err(|_| SerializationError::RebuildMix)?,
        u32::try_from(edges.len()).map_err(|_| SerializationError::RebuildMix)?,
    );

    Ok(JmtSerArtifact::new(
        JmtSerVersion::CURRENT,
        JmtSerRoots::new(sem_root, roots),
        meta,
        nodes,
        edges,
    ))
}

fn build_tree(
    tree_id: JmtSerTreeId,
    expect: [u8; 32],
    leaves: &[TreeLeaf],
    nodes: &mut Vec<JmtSerNode>,
    edges: &mut Vec<JmtSerEdge>,
) -> Result<JmtSerTreeRoot, SerializationError> {
    let temp = TempTreeStore::default();
    let mut ops: Vec<_> = leaves
        .iter()
        .map(|leaf| (leaf.key_hash, Some(leaf.payload.clone())))
        .collect();
    ops.sort_by_key(|(key_hash, _)| key_hash.0);

    let tree = Sha256Jmt::new(&temp);
    let (root, batch) = tree
        .put_value_set(ops, 0)
        .map_err(|err| SerializationError::Backend(err.to_string()))?;
    if root.0 != expect {
        return Err(SerializationError::RootMix);
    }
    temp.write_node_batch(&batch.node_batch)
        .map_err(|err| SerializationError::Backend(err.to_string()))?;

    let root_key = temp.root_key().ok_or(SerializationError::RootMix)?;
    let mut seen = BTreeMap::new();
    let leaf_map: HashMap<_, _> = leaves
        .iter()
        .map(|leaf| (leaf.key_hash, (leaf.raw_key.clone(), leaf.payload.clone())))
        .collect();
    push_node(
        &temp, tree_id, &root_key, &leaf_map, &mut seen, nodes, edges,
    )?;

    Ok(JmtSerTreeRoot::new(tree_id, expect, root.0))
}

fn push_node(
    temp: &TempTreeStore,
    tree_id: JmtSerTreeId,
    node_key: &NodeKey,
    leaf_map: &HashMap<KeyHash, (Vec<u8>, Vec<u8>)>,
    seen: &mut BTreeMap<Vec<u8>, [u8; 32]>,
    nodes: &mut Vec<JmtSerNode>,
    edges: &mut Vec<JmtSerEdge>,
) -> Result<[u8; 32], SerializationError> {
    let key_bytes = BincodeCodec.serialize(node_key)?;
    if let Some(node_id) = seen.get(&key_bytes) {
        return Ok(*node_id);
    }

    let node = temp.node(node_key).ok_or(SerializationError::RebuildMix)?;
    let node_id = hash_many(
        b"node-id",
        [tree_tag(tree_id).as_slice(), key_bytes.as_slice()],
    );

    match node {
        Node::Null => {
            return Err(SerializationError::NodeKindMix);
        }
        Node::Leaf(leaf) => {
            let payload = temp
                .leaf_bytes(leaf.key_hash(), node_key.version())
                .ok_or(SerializationError::RebuildMix)?;
            let (raw_key, _) = leaf_map
                .get(&leaf.key_hash())
                .ok_or(SerializationError::RebuildMix)?;
            nodes.push(JmtSerNode::new(
                node_id,
                tree_id,
                JmtSerNodeKind::Leaf,
                raw_key.clone(),
                leaf.hash::<Sha256>(),
                Some(leaf.key_hash().0),
                Some(sha256(&payload)),
                payload,
            ));
        }
        Node::Internal(internal) => {
            let payload = BincodeCodec.serialize(&Node::Internal(internal.clone()))?;
            nodes.push(JmtSerNode::new(
                node_id,
                tree_id,
                JmtSerNodeKind::Internal,
                key_bytes.clone(),
                internal.hash::<Sha256>(),
                None,
                None,
                payload,
            ));

            seen.insert(key_bytes.clone(), node_id);

            for (nibble, child) in internal.children_sorted() {
                let child_key = NodeKey::new(
                    child.version,
                    node_key
                        .nibble_path()
                        .nibbles()
                        .chain(std::iter::once(nibble))
                        .collect(),
                );
                let child_id = push_node(temp, tree_id, &child_key, leaf_map, seen, nodes, edges)?;
                edges.push(JmtSerEdge::new(
                    tree_id,
                    node_id,
                    child_id,
                    u8::from(nibble),
                ));
            }

            return Ok(node_id);
        }
    }

    seen.insert(key_bytes, node_id);
    Ok(node_id)
}

fn calc_root(leaves: &[TreeLeaf]) -> Result<[u8; 32], SerializationError> {
    let temp = TempTreeStore::default();
    let mut ops: Vec<_> = leaves
        .iter()
        .map(|leaf| (leaf.key_hash, Some(leaf.payload.clone())))
        .collect();
    ops.sort_by_key(|(key_hash, _)| key_hash.0);
    let tree = Sha256Jmt::new(&temp);
    let (root, _batch) = tree
        .put_value_set(ops, 0)
        .map_err(|err| SerializationError::Backend(err.to_string()))?;
    Ok(root.0)
}

fn bucket_key(bucket_id: BucketId) -> KeyHash {
    KeyHash(bucket_id.into_bytes())
}

fn terminal_key(terminal_id: TerminalId) -> KeyHash {
    KeyHash(terminal_id.into_bytes())
}

fn tree_tag(tree_id: JmtSerTreeId) -> Vec<u8> {
    BincodeCodec
        .serialize(&tree_id)
        .expect("serialize tree id for deterministic artifact build")
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn hash_many<'a>(tag: &'static [u8], parts: impl IntoIterator<Item = &'a [u8]>) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(tag);
    for part in parts {
        hasher.update((part.len() as u64).to_le_bytes());
        hasher.update(part);
    }
    hasher.finalize().into()
}

type TreeRankKey = (u8, [u8; 32], u32, [u8; 32]);
type NodeSortKey = (u8, [u8; 32], u32, [u8; 32], bool, Vec<u8>, [u8; 32]);
type EdgeSortKey = (u8, [u8; 32], u32, [u8; 32], [u8; 32], u8, [u8; 32]);

fn tree_rank(tree_id: &JmtSerTreeId) -> TreeRankKey {
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

fn root_sort_key(root: &JmtSerTreeRoot) -> (u8, [u8; 32], u32, [u8; 32], [u8; 32]) {
    let (rank, def_id, serial_id, bucket_id) = tree_rank(&root.tree_id);
    (rank, def_id, serial_id, bucket_id, root.root)
}

fn node_sort_key(node: &JmtSerNode) -> NodeSortKey {
    let (rank, def_id, serial_id, bucket_id) = tree_rank(&node.tree_id);
    (
        rank,
        def_id,
        serial_id,
        bucket_id,
        matches!(node.kind, JmtSerNodeKind::Leaf),
        node.key.clone(),
        node.id,
    )
}

fn edge_sort_key(edge: &JmtSerEdge) -> EdgeSortKey {
    let (rank, def_id, serial_id, bucket_id) = tree_rank(&edge.tree_id);
    (
        rank,
        def_id,
        serial_id,
        bucket_id,
        edge.parent,
        edge.slot,
        edge.child,
    )
}
