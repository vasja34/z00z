use std::collections::BTreeMap;

use crate::settlement::{
    CheckRoot, RightLeaf, SettlementPath, SettlementStateRoot, SettlementStore, StoreItem,
    TerminalId, TerminalLeaf,
};
use crate::snapshot::{PrepReplayEntry, PrepSnapshot};

use super::build::{InputResolver, MemberWit, ResolvedInput, SettlementState, StateError};
use super::CheckpointExecInput;

#[derive(Clone)]
struct BuildRow {
    def_id: crate::settlement::DefinitionId,
    leaf: TerminalLeaf,
}

#[derive(Clone)]
struct BuildRightRow {
    path: SettlementPath,
    leaf: RightLeaf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct BuildKey {
    root: CheckRoot,
    terminal_id: TerminalId,
}

#[derive(Debug)]
pub(super) struct BuildIdx {
    map: BTreeMap<BuildKey, ResolvedInput>,
}

pub(super) struct BuildState {
    root: CheckRoot,
    rows: BTreeMap<TerminalId, BuildRow>,
    right_rows: Vec<BuildRightRow>,
    out_defs: BTreeMap<TerminalId, crate::settlement::DefinitionId>,
}

pub(super) fn proof_root(prev_root: CheckRoot) -> SettlementStateRoot {
    SettlementStateRoot::settlement_v1(prev_root.into_bytes())
}

fn build_root(
    rows: &BTreeMap<TerminalId, BuildRow>,
    right_rows: &[BuildRightRow],
) -> Result<CheckRoot, StateError> {
    let mut store = SettlementStore::try_new().map_err(|err| StateError::State(err.to_string()))?;
    for row in rows.values() {
        let path = SettlementPath::new(
            row.def_id,
            crate::settlement::SerialId::new(row.leaf.serial_id),
            row.leaf.terminal_id(),
        );
        let item = StoreItem::new(path, row.leaf.clone())
            .map_err(|err| StateError::State(err.to_string()))?;
        store
            .put_settlement_item(item)
            .map_err(|err| StateError::State(err.to_string()))?;
    }
    for row in right_rows {
        let item =
            StoreItem::new(row.path, row.leaf).map_err(|err| StateError::State(err.to_string()))?;
        store
            .put_settlement_item(item)
            .map_err(|err| StateError::State(err.to_string()))?;
    }

    store
        .settlement_root()
        .map(CheckRoot::from)
        .map_err(|err| StateError::State(err.to_string()))
}

impl BuildState {
    pub(super) fn new(
        snapshot: &PrepSnapshot,
        exec: &CheckpointExecInput,
    ) -> Result<Self, StateError> {
        let mut rows = BTreeMap::new();
        let mut right_rows = Vec::new();
        for entry in &snapshot.entries {
            if let Ok(leaf) = entry.terminal_leaf() {
                let terminal_id = entry.path().terminal_id();
                rows.insert(
                    terminal_id,
                    BuildRow {
                        def_id: entry.path().definition_id,
                        leaf: leaf.clone(),
                    },
                );
                continue;
            }

            right_rows.push(BuildRightRow {
                path: entry.path(),
                leaf: *entry
                    .right_leaf()
                    .map_err(|err| StateError::State(err.to_string()))?,
            });
        }

        let mut out_defs = BTreeMap::new();
        for tx in exec.txs() {
            for out in tx.outputs() {
                let terminal_id = out.leaf().terminal_id();
                if out_defs.insert(terminal_id, out.definition_id()).is_some() {
                    return Err(StateError::DupOut);
                }
            }
        }

        let root = build_root(&rows, &right_rows)?;
        Ok(Self {
            root,
            rows,
            right_rows,
            out_defs,
        })
    }
}

impl SettlementState for BuildState {
    fn root(&self) -> CheckRoot {
        self.root
    }

    fn get_leaf(&self, id: &TerminalId) -> Result<Option<TerminalLeaf>, StateError> {
        Ok(self.rows.get(id).map(|row| row.leaf.clone()))
    }

    fn del_leaf(&mut self, id: &TerminalId) -> Result<(), StateError> {
        self.rows.remove(id);
        self.root = build_root(&self.rows, &self.right_rows)?;
        Ok(())
    }

    fn put_leaf(&mut self, leaf: TerminalLeaf) -> Result<(), StateError> {
        let terminal_id = leaf.terminal_id();
        let def_id = self
            .out_defs
            .get(&terminal_id)
            .copied()
            .ok_or(StateError::DupOut)?;
        self.rows.insert(terminal_id, BuildRow { def_id, leaf });
        self.root = build_root(&self.rows, &self.right_rows)?;
        Ok(())
    }

    fn leaf_hash(&self, leaf: &TerminalLeaf) -> Result<[u8; 32], StateError> {
        let terminal_id = leaf.terminal_id();
        let path = SettlementPath::new(
            self.out_defs
                .get(&terminal_id)
                .copied()
                .ok_or(StateError::DupOut)?,
            crate::settlement::SerialId::new(leaf.serial_id),
            terminal_id,
        );
        let item =
            StoreItem::new(path, leaf.clone()).map_err(|err| StateError::State(err.to_string()))?;
        let mut store =
            SettlementStore::try_new().map_err(|err| StateError::State(err.to_string()))?;
        store
            .put_settlement_item(item)
            .map_err(|err| StateError::State(err.to_string()))?;
        store
            .settlement_proof_blob(&path)
            .map(|blob| blob.terminal_leaf_hash())
            .map_err(|err| StateError::State(err.to_string()))
    }
}

impl BuildIdx {
    pub(super) fn new(
        prev_root: CheckRoot,
        replay: &[PrepReplayEntry],
    ) -> Result<Self, StateError> {
        let mut map = BTreeMap::new();
        for entry in replay {
            let item = entry.item();
            let Ok(leaf) = item.terminal_leaf() else {
                continue;
            };
            let path = item.path();
            let terminal_id = path.terminal_id();
            let resolved = ResolvedInput::new(
                path,
                leaf.clone(),
                MemberWit::new(item.wit().to_vec(), entry.proof_item().clone())?,
            )?;
            map.insert(
                BuildKey {
                    root: prev_root,
                    terminal_id,
                },
                resolved,
            );
        }
        Ok(Self { map })
    }
}

impl InputResolver for BuildIdx {
    fn resolve(
        &self,
        prev_root: CheckRoot,
        terminal_id: TerminalId,
        serial_id: u32,
    ) -> Result<ResolvedInput, StateError> {
        let key = BuildKey {
            root: prev_root,
            terminal_id,
        };
        let resolved = match self.map.get(&key).cloned() {
            Some(resolved) => resolved,
            None if self.map.keys().any(|item| item.terminal_id == terminal_id) => {
                return Err(StateError::PrevRoot)
            }
            None => return Err(StateError::MissingInput),
        };
        if resolved.serial_id() != serial_id {
            return Err(StateError::LeafMatch);
        }
        Ok(resolved)
    }
}
