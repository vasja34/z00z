use std::collections::BTreeMap;

use z00z_crypto::expert::hash_domain;
use z00z_crypto::expert::traits::DomainSeparation;
use z00z_crypto::poseidon2_hash;

use super::tree_id::TreeRootRef;
#[cfg(test)]
use super::ProofItem;
use super::SettlementLeaf;
use super::{
    DefinitionId, DefinitionRootLeaf, ModelErr, SerialId, SerialRootLeaf, SettlementPath,
    SettlementStateRoot, StoreItem, TerminalId,
};

hash_domain!(StorTerminalDom, "z00z.storage.settlement", 1);
hash_domain!(StorSerialDom, "z00z.storage.serial", 1);
hash_domain!(StorDefDom, "z00z.storage.definition", 1);
hash_domain!(StorStateDom, "z00z.storage.state", 1);

type TerminalMap = BTreeMap<TerminalId, SettlementLeaf>;
type SerialMap = BTreeMap<SerialId, TerminalMap>;
type DefMap = BTreeMap<DefinitionId, SerialMap>;

/// Reference hierarchy model for deterministic path and root semantics.
#[derive(Clone, Debug, Default)]
pub struct SettlementModel {
    defs: DefMap,
}

impl SettlementModel {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn scope_clone(&self, def_ids: &[DefinitionId]) -> Self {
        let mut defs = DefMap::new();

        for definition_id in def_ids {
            if let Some(serials) = self.defs.get(definition_id) {
                defs.insert(*definition_id, serials.clone());
            }
        }

        Self { defs }
    }

    pub(crate) fn merge_scope(&mut self, next: Self, def_ids: &[DefinitionId]) {
        for definition_id in def_ids {
            self.defs.remove(definition_id);
        }

        self.defs.extend(next.defs);
    }

    pub(crate) fn item_opt(&self, path: &SettlementPath) -> Result<Option<StoreItem>, ModelErr> {
        let Some(serials) = self.defs.get(&path.definition_id) else {
            return Ok(None);
        };
        let Some(terminals) = serials.get(&path.serial_id) else {
            return Ok(None);
        };
        let Some(leaf) = terminals.get(&path.terminal_id()) else {
            return Ok(None);
        };

        Ok(Some(StoreItem::new(*path, leaf.clone())?))
    }

    pub fn put_leaf(&mut self, item: StoreItem) -> Result<SettlementStateRoot, ModelErr> {
        item.check_path()?;

        let (path, leaf) = item.into_parts();
        self.defs
            .entry(path.definition_id)
            .or_default()
            .entry(path.serial_id)
            .or_default()
            .insert(path.terminal_id(), leaf);

        self.root()
    }

    pub fn root(&self) -> Result<SettlementStateRoot, ModelErr> {
        let mut parts = Vec::with_capacity(self.defs.len());
        for (definition_id, serials) in &self.defs {
            let leaf = self.def_leaf(*definition_id, serials)?;
            parts.push(leaf.encode());
        }

        Ok(SettlementStateRoot::settlement_v1(
            hash_many::<StorStateDom>(parts),
        ))
    }

    pub(crate) fn def_ids(&self) -> Vec<DefinitionId> {
        self.defs.keys().copied().collect()
    }

    pub(crate) fn has_def(&self, definition_id: DefinitionId) -> bool {
        self.defs.contains_key(&definition_id)
    }

    pub(crate) fn has_serial(&self, definition_id: DefinitionId, serial_id: SerialId) -> bool {
        self.defs
            .get(&definition_id)
            .is_some_and(|serials| serials.contains_key(&serial_id))
    }

    pub(crate) fn paths(&self) -> Vec<SettlementPath> {
        let mut paths = Vec::new();

        for (definition_id, serials) in &self.defs {
            for (serial_id, terminals) in serials {
                for terminal_id in terminals.keys() {
                    paths.push(SettlementPath::new(
                        *definition_id,
                        *serial_id,
                        *terminal_id,
                    ));
                }
            }
        }

        paths
    }

    #[cfg(test)]
    pub fn proof_case(&self, path: &SettlementPath) -> Result<ProofItem, ModelErr> {
        let serials = self.defs.get(&path.definition_id).ok_or(ModelErr::NoDef)?;
        let terminals = serials.get(&path.serial_id).ok_or(ModelErr::NoSerial)?;
        let leaf = terminals
            .get(&path.terminal_id())
            .cloned()
            .ok_or(ModelErr::NoTerminal)?;
        let serial_root_leaf = self.serial_leaf(path.definition_id, path.serial_id, terminals)?;
        let definition_root_leaf = self.def_leaf(path.definition_id, serials)?;

        ProofItem::new_settlement(
            self.root()?,
            *path,
            definition_root_leaf,
            serial_root_leaf,
            leaf,
        )
    }

    pub fn del_leaf(&mut self, path: &SettlementPath) -> Result<SettlementStateRoot, ModelErr> {
        let serials = self
            .defs
            .get_mut(&path.definition_id)
            .ok_or(ModelErr::NoDef)?;
        let terminals = serials.get_mut(&path.serial_id).ok_or(ModelErr::NoSerial)?;
        if terminals.remove(&path.terminal_id()).is_none() {
            return Err(ModelErr::NoTerminal);
        }
        if terminals.is_empty() {
            serials.remove(&path.serial_id);
        }
        if serials.is_empty() {
            self.defs.remove(&path.definition_id);
        }

        self.root()
    }

    fn def_leaf(
        &self,
        definition_id: DefinitionId,
        serials: &SerialMap,
    ) -> Result<DefinitionRootLeaf, ModelErr> {
        let mut parts = Vec::with_capacity(serials.len());
        for (serial_id, terminals) in serials {
            let leaf = self.serial_leaf(definition_id, *serial_id, terminals)?;
            parts.push(leaf.encode());
        }

        let def_root = TreeRootRef::new(hash_many::<StorDefDom>(parts));

        Ok(DefinitionRootLeaf {
            definition_id,
            definition_root: def_root.into_bytes(),
        })
    }

    fn serial_leaf(
        &self,
        definition_id: DefinitionId,
        serial_id: SerialId,
        terminals: &TerminalMap,
    ) -> Result<SerialRootLeaf, ModelErr> {
        let mut parts = Vec::with_capacity(terminals.len());
        for leaf in terminals.values() {
            parts.push(terminal_leaf_hash(leaf)?);
        }

        let ser_root = TreeRootRef::new(hash_many::<StorSerialDom>(parts));

        Ok(SerialRootLeaf {
            definition_id,
            serial_id,
            serial_root: ser_root.into_bytes(),
        })
    }
}

pub(crate) fn empty_state_root() -> SettlementStateRoot {
    SettlementStateRoot::settlement_v1(hash_many::<StorStateDom>(Vec::new()))
}

pub(crate) fn terminal_leaf_hash(leaf: &SettlementLeaf) -> Result<Vec<u8>, ModelErr> {
    let payload = leaf.encode()?;
    Ok(poseidon2_hash(
        domain_tag::<StorTerminalDom>().as_slice(),
        &[payload.as_slice()],
    )
    .to_vec())
}

fn hash_many<M>(parts: Vec<Vec<u8>>) -> [u8; 32]
where
    M: DomainSeparation,
{
    let domain = domain_tag::<M>();
    poseidon2_hash(domain.as_slice(), &as_refs(&parts))
}

fn domain_tag<M>() -> Vec<u8>
where
    M: DomainSeparation,
{
    M::domain_separation_tag("").into_bytes()
}

fn as_refs(parts: &[Vec<u8>]) -> Vec<&[u8]> {
    parts.iter().map(Vec::as_slice).collect()
}
