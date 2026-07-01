use super::{BucketId, DefinitionId, SerialId, SettlementPath};

const DEF_TREE_TAG: u8 = 0x01;
const SER_TREE_TAG: u8 = 0x02;
const TERM_TREE_TAG: u8 = 0x03;

/// Private logical tree identity for the nested topology design.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TreeId {
    Definition,
    Serial(DefinitionId),
    Terminal(DefinitionId, SerialId),
}

impl TreeId {
    #[must_use]
    pub(crate) fn ns_bytes(self) -> Vec<u8> {
        let mut out = Vec::with_capacity(37);
        match self {
            Self::Definition => out.push(DEF_TREE_TAG),
            Self::Serial(def_id) => {
                out.push(SER_TREE_TAG);
                out.extend_from_slice(def_id.as_bytes());
            }
            Self::Terminal(def_id, serial_id) => {
                out.push(TERM_TREE_TAG);
                out.extend_from_slice(def_id.as_bytes());
                out.extend_from_slice(&serial_id.get().to_le_bytes());
            }
        }
        out
    }
}

/// Private physical hjmt tree identity for the bucketed HJMT backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum HjmtTreeId {
    Definition,
    Serial(DefinitionId),
    Bucket(DefinitionId, SerialId),
    BucketTerminal(DefinitionId, SerialId, BucketId),
    PathIndex,
}

/// Private wrapper for child-tree root bytes before they are embedded in public parent leaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TreeRootRef([u8; 32]);

impl TreeRootRef {
    #[must_use]
    pub(crate) const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub(crate) const fn into_bytes(self) -> [u8; 32] {
        self.0
    }
}

/// Storage-internal path index payload.
///
/// This record is not consensus-visible and must not change public root semantics.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PathIndexRec {
    path: SettlementPath,
}

impl PathIndexRec {
    #[must_use]
    pub(crate) const fn new(path: SettlementPath) -> Self {
        Self { path }
    }
}

#[cfg(test)]
mod tests {
    use super::{TreeId, DEF_TREE_TAG, SER_TREE_TAG, TERM_TREE_TAG};
    use crate::settlement::{DefinitionId, SerialId};

    #[test]
    fn test_tree_bytes_are_unique() {
        let def_id = DefinitionId::new([7u8; 32]);
        let ser_id = SerialId::new(9);

        let def_ns = TreeId::Definition.ns_bytes();
        let ser_ns = TreeId::Serial(def_id).ns_bytes();
        let term_ns = TreeId::Terminal(def_id, ser_id).ns_bytes();

        assert_eq!(def_ns[0], DEF_TREE_TAG);
        assert_eq!(ser_ns[0], SER_TREE_TAG);
        assert_eq!(term_ns[0], TERM_TREE_TAG);
        assert_ne!(def_ns, ser_ns);
        assert_ne!(ser_ns, term_ns);
    }
}
