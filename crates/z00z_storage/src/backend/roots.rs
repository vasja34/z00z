use std::collections::{BTreeMap, HashMap};

use crate::settlement::{
    model::empty_state_root, tree_id::TreeRootRef, BucketId, DefinitionId, SerialId,
    SettlementStateRoot,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TreeRoots {
    pub(crate) sem_root: SettlementStateRoot,
    pub(crate) settlement_root: SettlementStateRoot,
    pub(crate) def_root: Option<TreeRootRef>,
    pub(crate) def_rows: BTreeMap<DefinitionId, Vec<u8>>,
    pub(crate) serial_roots: HashMap<DefinitionId, TreeRootRef>,
    pub(crate) terminal_roots: HashMap<(DefinitionId, SerialId), TreeRootRef>,
}

pub(crate) type HjmtSerialKey = (DefinitionId, SerialId);
pub(crate) type HjmtBucketKey = (DefinitionId, SerialId, BucketId);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HjmtRoots {
    pub(crate) version: u64,
    pub(crate) sem_root: SettlementStateRoot,
    pub(crate) settlement_root: SettlementStateRoot,
    pub(crate) journal_digest: Option<[u8; 32]>,
    pub(crate) def_root: Option<TreeRootRef>,
    pub(crate) serial_roots: HashMap<DefinitionId, TreeRootRef>,
    pub(crate) bucket_roots: HashMap<HjmtSerialKey, TreeRootRef>,
    pub(crate) terminal_roots: HashMap<HjmtBucketKey, TreeRootRef>,
}

impl HjmtRoots {
    pub(crate) fn new() -> Self {
        let sem_root = empty_state_root();
        Self {
            version: 0,
            sem_root,
            settlement_root: sem_root,
            journal_digest: None,
            def_root: None,
            serial_roots: HashMap::new(),
            bucket_roots: HashMap::new(),
            terminal_roots: HashMap::new(),
        }
    }

    pub(crate) const fn sem_root(&self) -> SettlementStateRoot {
        self.sem_root
    }

    pub(crate) const fn settlement_root(&self) -> SettlementStateRoot {
        self.settlement_root
    }
}

impl TreeRoots {
    pub(crate) fn new() -> Self {
        let sem_root = empty_state_root();
        Self {
            sem_root,
            settlement_root: sem_root,
            def_root: None,
            def_rows: BTreeMap::new(),
            serial_roots: HashMap::new(),
            terminal_roots: HashMap::new(),
        }
    }
}

impl Default for TreeRoots {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for HjmtRoots {
    fn default() -> Self {
        Self::new()
    }
}
