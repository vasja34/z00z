use std::collections::BTreeMap;

use jmt::{proof::SparseMerkleProof, KeyHash, RootHash, Sha256Jmt, Version};
use sha2::Sha256;

use super::{tree_id::HjmtTreeId, SettlementStoreError};
use crate::backend::{
    codec::map_jmt_err,
    memory::{apply_batch, KeyValueOp, MemTreeInner, MemTreeStore},
};

#[derive(Clone)]
pub(super) struct HjmtStoreSnap {
    trees: BTreeMap<HjmtTreeId, MemTreeInner>,
    latest_versions: BTreeMap<HjmtTreeId, Version>,
}

#[derive(Clone, Default)]
pub(super) struct HjmtTreeSnap {
    inner: MemTreeInner,
    latest_version: Version,
}

#[derive(Default)]
pub(super) struct HjmtStore {
    trees: BTreeMap<HjmtTreeId, MemTreeStore>,
    latest_versions: BTreeMap<HjmtTreeId, Version>,
}

impl HjmtStore {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn snap(&self) -> HjmtStoreSnap {
        let trees = self
            .trees
            .iter()
            .map(|(tree_id, store)| (*tree_id, store.snap()))
            .collect();
        HjmtStoreSnap {
            trees,
            latest_versions: self.latest_versions.clone(),
        }
    }

    pub(super) fn restore(&mut self, snap: HjmtStoreSnap) {
        self.trees.clear();
        for (tree_id, inner) in snap.trees {
            let store = MemTreeStore::new();
            store.restore(inner);
            self.trees.insert(tree_id, store);
        }
        self.latest_versions = snap.latest_versions;
    }

    pub(super) fn tree_snap(&self, tree_id: HjmtTreeId) -> HjmtTreeSnap {
        HjmtTreeSnap {
            inner: self
                .trees
                .get(&tree_id)
                .map(MemTreeStore::snap)
                .unwrap_or_default(),
            latest_version: self.latest_versions.get(&tree_id).copied().unwrap_or(0),
        }
    }

    pub(super) fn restore_tree(&mut self, tree_id: HjmtTreeId, snap: HjmtTreeSnap) {
        let store = self.trees.entry(tree_id).or_default();
        store.restore(snap.inner);
        self.latest_versions.insert(tree_id, snap.latest_version);
    }

    pub(super) fn commit_snap(
        snap: HjmtTreeSnap,
        ops: Vec<KeyValueOp>,
        version: Version,
    ) -> Result<(RootHash, HjmtTreeSnap), SettlementStoreError> {
        let store = MemTreeStore::new();
        store.restore(snap.inner);
        if version > 0 {
            let _ = ensure_store_version(&store, snap.latest_version, version - 1)?;
        }
        let jmt = Sha256Jmt::new(&store);
        let (root, batch) = jmt.put_value_set(ops, version).map_err(map_jmt_err)?;
        apply_batch(&store, batch)?;
        Ok((
            root,
            HjmtTreeSnap {
                inner: store.snap(),
                latest_version: version,
            },
        ))
    }

    pub(super) fn ensure_snap(
        snap: HjmtTreeSnap,
        version: Version,
    ) -> Result<HjmtTreeSnap, SettlementStoreError> {
        let store = MemTreeStore::new();
        store.restore(snap.inner);
        let latest_version = ensure_store_version(&store, snap.latest_version, version)?;
        Ok(HjmtTreeSnap {
            inner: store.snap(),
            latest_version,
        })
    }

    pub(super) fn get_proof(
        &self,
        tree_id: HjmtTreeId,
        key: KeyHash,
        version: Version,
    ) -> Result<SparseMerkleProof<Sha256>, SettlementStoreError> {
        let store = self
            .trees
            .get(&tree_id)
            .ok_or(SettlementStoreError::EmptyTree)?;
        let jmt = Sha256Jmt::new(store);
        let (_value, proof) = jmt.get_with_proof(key, version).map_err(map_jmt_err)?;
        Ok(proof)
    }
}

fn ensure_store_version(
    store: &MemTreeStore,
    latest: Version,
    version: Version,
) -> Result<Version, SettlementStoreError> {
    if version == 0 {
        return Ok(latest.max(version));
    }
    let jmt = Sha256Jmt::new(store);
    let mut next_latest = latest;
    for next_version in latest.saturating_add(1)..=version {
        let (_root, batch) = jmt
            .put_value_set(Vec::new(), next_version)
            .map_err(map_jmt_err)?;
        apply_batch(store, batch)?;
        next_latest = next_version;
    }
    Ok(next_latest.max(version))
}
