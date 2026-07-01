use crate::settlement::{BucketId, DefinitionId, SerialId, SettlementPath, SettlementStateRoot};

/// Schema version for storage-owned JMT serialization artifacts.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct JmtSerVersion(u8);

impl JmtSerVersion {
    pub const CURRENT: Self = Self(1);

    #[must_use]
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self.0
    }

    #[must_use]
    pub const fn is_supported(self) -> bool {
        self.0 == Self::CURRENT.0
    }
}

/// External content-addressed identifier for one canonical serialization artifact.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct JmtSerArtifactId([u8; 32]);

impl JmtSerArtifactId {
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn into_bytes(self) -> [u8; 32] {
        self.0
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for JmtSerArtifactId {
    fn from(bytes: [u8; 32]) -> Self {
        Self::new(bytes)
    }
}

/// Storage-owned logical tree identifier derived from the asset-store topology.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum JmtSerTreeId {
    Definition,
    Serial(DefinitionId),
    Bucket {
        definition_id: DefinitionId,
        serial_id: SerialId,
    },
    Terminal {
        definition_id: DefinitionId,
        serial_id: SerialId,
        bucket_id: BucketId,
    },
    PathIndex,
}

/// Storage-owned serialized node kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum JmtSerNodeKind {
    Internal,
    Leaf,
    Null,
}

/// Typed root record for one logical tree inside the shared JMT-backed store.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JmtSerTreeRoot {
    pub tree_id: JmtSerTreeId,
    pub root: [u8; 32],
    pub jmt_root: [u8; 32],
}

impl JmtSerTreeRoot {
    #[must_use]
    pub const fn new(tree_id: JmtSerTreeId, root: [u8; 32], jmt_root: [u8; 32]) -> Self {
        Self {
            tree_id,
            root,
            jmt_root,
        }
    }
}

/// Root bundle carried by a deterministic serialization artifact.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JmtSerRoots {
    pub sem_root: SettlementStateRoot,
    pub trees: Vec<JmtSerTreeRoot>,
}

impl JmtSerRoots {
    #[must_use]
    pub fn new(sem_root: SettlementStateRoot, trees: Vec<JmtSerTreeRoot>) -> Self {
        Self { sem_root, trees }
    }
}

/// Artifact-level metadata needed for deterministic reconstruction.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JmtSerMeta {
    pub path_order: Vec<SettlementPath>,
    pub node_count: u32,
    pub edge_count: u32,
}

impl JmtSerMeta {
    #[must_use]
    pub fn new(path_order: Vec<SettlementPath>, node_count: u32, edge_count: u32) -> Self {
        Self {
            path_order,
            node_count,
            edge_count,
        }
    }
}

/// Storage-owned serialized node record.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JmtSerNode {
    pub id: [u8; 32],
    pub tree_id: JmtSerTreeId,
    pub kind: JmtSerNodeKind,
    pub key: Vec<u8>,
    pub node_hash: [u8; 32],
    pub key_hash: Option<[u8; 32]>,
    pub value_hash: Option<[u8; 32]>,
    pub payload: Vec<u8>,
}

impl JmtSerNode {
    #[must_use]
    pub fn new(
        id: [u8; 32],
        tree_id: JmtSerTreeId,
        kind: JmtSerNodeKind,
        key: Vec<u8>,
        node_hash: [u8; 32],
        key_hash: Option<[u8; 32]>,
        value_hash: Option<[u8; 32]>,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            id,
            tree_id,
            kind,
            key,
            node_hash,
            key_hash,
            value_hash,
            payload,
        }
    }
}

/// Storage-owned typed edge between two serialized nodes.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JmtSerEdge {
    pub tree_id: JmtSerTreeId,
    pub parent: [u8; 32],
    pub child: [u8; 32],
    pub slot: u8,
}

impl JmtSerEdge {
    #[must_use]
    pub const fn new(tree_id: JmtSerTreeId, parent: [u8; 32], child: [u8; 32], slot: u8) -> Self {
        Self {
            tree_id,
            parent,
            child,
            slot,
        }
    }
}

/// Deterministic storage-owned JMT serialization artifact.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JmtSerArtifact {
    pub version: JmtSerVersion,
    pub roots: JmtSerRoots,
    pub meta: JmtSerMeta,
    pub nodes: Vec<JmtSerNode>,
    pub edges: Vec<JmtSerEdge>,
}

impl JmtSerArtifact {
    #[must_use]
    pub fn new(
        version: JmtSerVersion,
        roots: JmtSerRoots,
        meta: JmtSerMeta,
        nodes: Vec<JmtSerNode>,
        edges: Vec<JmtSerEdge>,
    ) -> Self {
        Self {
            version,
            roots,
            meta,
            nodes,
            edges,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        JmtSerArtifact, JmtSerArtifactId, JmtSerEdge, JmtSerMeta, JmtSerNode, JmtSerNodeKind,
        JmtSerRoots, JmtSerTreeId, JmtSerTreeRoot, JmtSerVersion,
    };
    use crate::settlement::{
        BucketId, DefinitionId, SerialId, SettlementPath, SettlementStateRoot, TerminalId,
    };

    fn test_path() -> SettlementPath {
        SettlementPath::new(
            DefinitionId::new([1u8; 32]),
            SerialId::new(7),
            TerminalId::new([9u8; 32]),
        )
    }

    #[test]
    fn test_version_is_supported() {
        assert!(JmtSerVersion::CURRENT.is_supported());
        assert_eq!(JmtSerVersion::CURRENT.as_u8(), 1);
    }

    #[test]
    fn test_id_keeps_raw_bytes() {
        let artifact_id = JmtSerArtifactId::new([7u8; 32]);

        assert_eq!(artifact_id.as_bytes(), &[7u8; 32]);
    }

    #[test]
    fn test_artifact_keeps_storage_contract() {
        let tree_id = JmtSerTreeId::Terminal {
            definition_id: DefinitionId::new([2u8; 32]),
            serial_id: SerialId::new(11),
            bucket_id: BucketId::new([3u8; 32]),
        };
        let roots = JmtSerRoots::new(
            SettlementStateRoot::settlement_v1([4u8; 32]),
            vec![JmtSerTreeRoot::new(tree_id, [5u8; 32], [15u8; 32])],
        );
        let meta = JmtSerMeta::new(vec![test_path()], 1, 0);
        let nodes = vec![JmtSerNode::new(
            [6u8; 32],
            tree_id,
            JmtSerNodeKind::Leaf,
            vec![1, 2, 3],
            [7u8; 32],
            Some([8u8; 32]),
            Some([9u8; 32]),
            vec![9, 10],
        )];
        let artifact = JmtSerArtifact::new(JmtSerVersion::CURRENT, roots, meta, nodes, Vec::new());

        assert_eq!(artifact.version, JmtSerVersion::CURRENT);
        assert_eq!(artifact.meta.node_count, 1);
        assert_eq!(artifact.meta.edge_count, 0);
        assert_eq!(artifact.nodes.len(), 1);
        assert!(matches!(artifact.nodes[0].kind, JmtSerNodeKind::Leaf));
    }

    #[test]
    fn test_edge_keeps_tree_scope() {
        let edge = JmtSerEdge::new(JmtSerTreeId::Definition, [1u8; 32], [2u8; 32], 1);

        assert!(matches!(edge.tree_id, JmtSerTreeId::Definition));
        assert_eq!(edge.slot, 1);
    }
}
