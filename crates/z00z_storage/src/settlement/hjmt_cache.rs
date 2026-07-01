use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    hash::Hash,
};

use super::{
    hjmt_journal::HjmtCommitStatus, model::SettlementModel, tree_id::HjmtTreeId, BucketPolicy,
    DefinitionId, DefinitionRootLeaf, SerialId, SerialRootLeaf, SettlementPath, SettlementStore,
    SettlementStoreError,
};
use crate::backend::{
    roots::{HjmtBucketKey, HjmtRoots, HjmtSerialKey},
    types::terminal_value_hash,
};
use crate::settlement::{
    keys::{definition_key, serial_key},
    BucketId, BucketRootLeaf, MergeProof, PolicyTransitionProof, ProofBlob, RootGeneration,
    SettlementLeaf, SettlementLeafFamily, SettlementStateRoot, SplitProof, TerminalId,
    HJMT_PROOF_ENVELOPE_VERSION,
};

const ROOT_GENERATION: u8 = RootGeneration::SettlementV1.version();
const DEFAULT_LAYER_LIMIT: usize = 512;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CacheLayerMetrics {
    pub hits: u64,
    pub misses: u64,
    pub inserts: u64,
    pub evictions: u64,
    pub invalidations: u64,
    pub entries: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ForestCacheMetrics {
    pub subtree_root: CacheLayerMetrics,
    pub parent_leaf: CacheLayerMetrics,
    pub terminal_leaf: CacheLayerMetrics,
    pub bucket_derivation: CacheLayerMetrics,
    pub proof_segment: CacheLayerMetrics,
    pub nonexistence: CacheLayerMetrics,
    pub policy_proof: CacheLayerMetrics,
    pub journal_digest: CacheLayerMetrics,
    pub path_index: CacheLayerMetrics,
}

#[derive(Clone)]
struct CacheCell<V> {
    seq: u64,
    value: V,
}

#[derive(Clone)]
struct BoundedCache<K, V> {
    limit: usize,
    next_seq: u64,
    map: HashMap<K, CacheCell<V>>,
    order: VecDeque<(u64, K)>,
    hits: u64,
    misses: u64,
    inserts: u64,
    evictions: u64,
    invalidations: u64,
}

impl<K, V> BoundedCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn new(limit: usize) -> Self {
        Self {
            limit,
            next_seq: 0,
            map: HashMap::new(),
            order: VecDeque::new(),
            hits: 0,
            misses: 0,
            inserts: 0,
            invalidations: 0,
            evictions: 0,
        }
    }

    fn get_cloned(&mut self, key: &K) -> Option<V> {
        let Some(cell) = self.map.get(key) else {
            self.misses = self.misses.saturating_add(1);
            return None;
        };
        self.hits = self.hits.saturating_add(1);
        Some(cell.value.clone())
    }

    fn insert(&mut self, key: K, value: V) {
        self.next_seq = self.next_seq.saturating_add(1);
        let seq = self.next_seq;
        self.map.insert(key.clone(), CacheCell { seq, value });
        self.order.push_back((seq, key));
        self.inserts = self.inserts.saturating_add(1);
        self.evict();
    }

    fn evict(&mut self) {
        while self.map.len() > self.limit {
            let Some((seq, key)) = self.order.pop_front() else {
                break;
            };
            let should_remove = self.map.get(&key).is_some_and(|cell| cell.seq == seq);
            if should_remove {
                self.map.remove(&key);
                self.evictions = self.evictions.saturating_add(1);
            }
        }
    }

    fn invalidate_where<F>(&mut self, predicate: F)
    where
        F: Fn(&K, &V) -> bool,
    {
        let keys = self
            .map
            .iter()
            .filter_map(|(key, cell)| predicate(key, &cell.value).then_some(key.clone()))
            .collect::<Vec<_>>();
        for key in keys {
            self.map.remove(&key);
            self.invalidations = self.invalidations.saturating_add(1);
        }
    }

    fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
    }

    #[cfg(debug_assertions)]
    fn set_limit(&mut self, limit: usize) {
        self.limit = limit.max(1);
        self.evict();
    }

    fn metrics(&self) -> CacheLayerMetrics {
        CacheLayerMetrics {
            hits: self.hits,
            misses: self.misses,
            inserts: self.inserts,
            invalidations: self.invalidations,
            evictions: self.evictions,
            entries: self.map.len(),
        }
    }

    fn sample(&self, limit: usize) -> Vec<(K, V)> {
        self.map
            .iter()
            .take(limit)
            .map(|(key, cell)| (key.clone(), cell.value.clone()))
            .collect()
    }

    #[cfg(debug_assertions)]
    fn corrupt_first<F>(&mut self, mutate: F) -> bool
    where
        F: FnOnce(&mut V),
    {
        let Some(key) = self.map.keys().next().cloned() else {
            return false;
        };
        let Some(cell) = self.map.get_mut(&key) else {
            return false;
        };
        mutate(&mut cell.value);
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct SubtreeRootKey {
    root_generation: u8,
    version: u64,
    policy_id: [u8; 32],
    tree_id: HjmtTreeId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ParentLeafKey {
    Definition {
        root_generation: u8,
        version: u64,
        policy_id: [u8; 32],
        definition_id: DefinitionId,
        child_root: [u8; 32],
    },
    Serial {
        root_generation: u8,
        version: u64,
        policy_id: [u8; 32],
        key: HjmtSerialKey,
        child_root: [u8; 32],
    },
    Bucket {
        root_generation: u8,
        version: u64,
        policy_id: [u8; 32],
        key: HjmtBucketKey,
        child_root: [u8; 32],
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct TerminalLeafKey {
    root_generation: u8,
    terminal_id: TerminalId,
    leaf_family: u8,
    codec_version: u16,
    payload_hash: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TerminalLeafValue {
    pub(super) encoded: Vec<u8>,
    pub(super) leaf_hash: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct BucketDerivationKey {
    root_generation: u8,
    epoch: u64,
    path: SettlementPath,
    policy_id: [u8; 32],
    bucket_bits: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ProofSegmentKey {
    root_generation: u8,
    version: u64,
    tree_id: HjmtTreeId,
    key_hash: [u8; 32],
    proof_version: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct NonExistenceKey {
    root_generation: u8,
    root: SettlementStateRoot,
    path: SettlementPath,
    epoch: u64,
    proof_version: u16,
    leaf_family: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PolicyProofKey {
    Split {
        root_generation: u8,
        prior_root: SettlementStateRoot,
        prior_epoch: u64,
        prior_policy_id: [u8; 32],
        next_policy_id: [u8; 32],
        bucket: HjmtBucketKey,
    },
    Merge {
        root_generation: u8,
        prior_root: SettlementStateRoot,
        prior_epoch: u64,
        prior_policy_id: [u8; 32],
        next_policy_id: [u8; 32],
        left: HjmtBucketKey,
        right: HjmtBucketKey,
    },
    Transition {
        root_generation: u8,
        prior_root: SettlementStateRoot,
        prior_epoch: u64,
        prior_policy_id: [u8; 32],
        next_policy_id: [u8; 32],
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PolicyProofValue {
    Split(SplitProof),
    Merge(MergeProof),
    Transition(PolicyTransitionProof),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct JournalDigestKey {
    root_generation: u8,
    version: u64,
    status: u8,
    backend_root: [u8; 32],
    settlement_root: SettlementStateRoot,
    policy_id: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PathIndexKey {
    terminal_id: TerminalId,
}

#[derive(Clone)]
pub(super) struct ForestCacheState {
    proof_version: u16,
    subtree_root: BoundedCache<SubtreeRootKey, [u8; 32]>,
    parent_leaf: BoundedCache<ParentLeafKey, Vec<u8>>,
    terminal_leaf: BoundedCache<TerminalLeafKey, TerminalLeafValue>,
    bucket_derivation: BoundedCache<BucketDerivationKey, BucketId>,
    proof_segment: BoundedCache<ProofSegmentKey, Vec<u8>>,
    nonexistence: BoundedCache<NonExistenceKey, ProofBlob>,
    policy_proof: BoundedCache<PolicyProofKey, PolicyProofValue>,
    journal_digest: BoundedCache<JournalDigestKey, [u8; 32]>,
    path_index: BoundedCache<PathIndexKey, SettlementPath>,
}

impl ForestCacheState {
    fn new() -> Self {
        Self {
            proof_version: HJMT_PROOF_ENVELOPE_VERSION as u16,
            subtree_root: BoundedCache::new(DEFAULT_LAYER_LIMIT),
            parent_leaf: BoundedCache::new(DEFAULT_LAYER_LIMIT),
            terminal_leaf: BoundedCache::new(DEFAULT_LAYER_LIMIT),
            bucket_derivation: BoundedCache::new(DEFAULT_LAYER_LIMIT),
            proof_segment: BoundedCache::new(DEFAULT_LAYER_LIMIT),
            nonexistence: BoundedCache::new(DEFAULT_LAYER_LIMIT),
            policy_proof: BoundedCache::new(DEFAULT_LAYER_LIMIT),
            journal_digest: BoundedCache::new(DEFAULT_LAYER_LIMIT),
            path_index: BoundedCache::new(DEFAULT_LAYER_LIMIT),
        }
    }

    #[cfg(debug_assertions)]
    fn set_limit(&mut self, limit: usize) {
        self.subtree_root.set_limit(limit);
        self.parent_leaf.set_limit(limit);
        self.terminal_leaf.set_limit(limit);
        self.bucket_derivation.set_limit(limit);
        self.proof_segment.set_limit(limit);
        self.nonexistence.set_limit(limit);
        self.policy_proof.set_limit(limit);
        self.journal_digest.set_limit(limit);
        self.path_index.set_limit(limit);
    }
}

pub(crate) struct ForestCache {
    inner: RefCell<ForestCacheState>,
    verifying: Cell<bool>,
}

impl ForestCache {
    pub(super) fn new() -> Self {
        Self {
            inner: RefCell::new(ForestCacheState::new()),
            verifying: Cell::new(false),
        }
    }

    pub(super) fn snapshot(&self) -> ForestCacheState {
        self.inner.borrow().clone()
    }

    pub(super) fn restore(&self, snap: ForestCacheState) {
        *self.inner.borrow_mut() = snap;
    }

    pub(super) fn metrics(&self) -> ForestCacheMetrics {
        let inner = self.inner.borrow();
        ForestCacheMetrics {
            subtree_root: inner.subtree_root.metrics(),
            parent_leaf: inner.parent_leaf.metrics(),
            terminal_leaf: inner.terminal_leaf.metrics(),
            bucket_derivation: inner.bucket_derivation.metrics(),
            proof_segment: inner.proof_segment.metrics(),
            nonexistence: inner.nonexistence.metrics(),
            policy_proof: inner.policy_proof.metrics(),
            journal_digest: inner.journal_digest.metrics(),
            path_index: inner.path_index.metrics(),
        }
    }

    pub(super) fn sync_proof_version(&self) {
        let mut inner = self.inner.borrow_mut();
        let want = HJMT_PROOF_ENVELOPE_VERSION as u16;
        if inner.proof_version == want {
            return;
        }
        inner.proof_version = want;
        inner.proof_segment.clear();
        inner.nonexistence.clear();
        inner.policy_proof.clear();
    }

    pub(crate) fn clear_all(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.subtree_root.clear();
        inner.parent_leaf.clear();
        inner.terminal_leaf.clear();
        inner.bucket_derivation.clear();
        inner.proof_segment.clear();
        inner.nonexistence.clear();
        inner.policy_proof.clear();
        inner.journal_digest.clear();
        inner.path_index.clear();
    }

    #[cfg(debug_assertions)]
    pub(super) fn set_limit_for_test(&self, limit: usize) {
        self.inner.borrow_mut().set_limit(limit);
    }

    #[cfg(debug_assertions)]
    pub(super) fn corrupt_for_test(&self) -> bool {
        let mut inner = self.inner.borrow_mut();
        if inner
            .subtree_root
            .corrupt_first(|value| *value = [0xA5; 32])
        {
            return true;
        }
        if inner
            .proof_segment
            .corrupt_first(|value| value.iter_mut().for_each(|byte| *byte = 0x5A))
        {
            return true;
        }
        inner.path_index.corrupt_first(|path| {
            *path = SettlementPath::new(
                path.definition_id,
                path.serial_id,
                TerminalId::new([0x55; 32]),
            )
        })
    }

    #[cfg(debug_assertions)]
    pub(super) fn corrupt_journal_key_for_test(&self) -> bool {
        let mut inner = self.inner.borrow_mut();
        let Some((key, cell)) = inner
            .journal_digest
            .map
            .iter()
            .next()
            .map(|(key, cell)| (*key, cell.clone()))
        else {
            return false;
        };
        inner.journal_digest.map.remove(&key);
        inner.journal_digest.map.insert(
            JournalDigestKey {
                backend_root: [0xC3; 32],
                ..key
            },
            cell,
        );
        true
    }

    fn metrics_and_samples(&self, sample_limit: usize) -> ForestCacheSamples {
        let inner = self.inner.borrow();
        ForestCacheSamples {
            subtree_root: inner.subtree_root.sample(sample_limit),
            parent_leaf: inner.parent_leaf.sample(sample_limit),
            terminal_leaf: inner.terminal_leaf.sample(sample_limit),
            bucket_derivation: inner.bucket_derivation.sample(sample_limit),
            proof_segment: inner.proof_segment.sample(sample_limit),
            nonexistence: inner.nonexistence.sample(sample_limit),
            policy_proof: inner.policy_proof.sample(sample_limit),
            journal_digest: inner.journal_digest.sample(sample_limit),
            path_index: inner.path_index.sample(sample_limit),
        }
    }

    fn enter_verify(&self) -> bool {
        if self.verifying.get() {
            return false;
        }
        self.verifying.set(true);
        true
    }

    fn leave_verify(&self) {
        self.verifying.set(false);
    }
}

#[derive(Clone)]
struct ForestCacheSamples {
    subtree_root: Vec<(SubtreeRootKey, [u8; 32])>,
    parent_leaf: Vec<(ParentLeafKey, Vec<u8>)>,
    terminal_leaf: Vec<(TerminalLeafKey, TerminalLeafValue)>,
    bucket_derivation: Vec<(BucketDerivationKey, BucketId)>,
    proof_segment: Vec<(ProofSegmentKey, Vec<u8>)>,
    nonexistence: Vec<(NonExistenceKey, ProofBlob)>,
    policy_proof: Vec<(PolicyProofKey, PolicyProofValue)>,
    journal_digest: Vec<(JournalDigestKey, [u8; 32])>,
    path_index: Vec<(PathIndexKey, SettlementPath)>,
}

impl SettlementStore {
    pub fn forest_cache_metrics(&self) -> ForestCacheMetrics {
        self.forest_cache.metrics()
    }

    pub fn verify_forest_cache(&self) -> Result<(), SettlementStoreError> {
        self.verify_forest_cache_with_limit(usize::MAX)
    }

    pub fn clear_forest_cache(&self) {
        self.forest_cache.clear_all();
    }

    #[cfg(debug_assertions)]
    pub fn set_forest_cache_test_limit(&self, limit: usize) {
        self.forest_cache.set_limit_for_test(limit);
    }

    #[cfg(debug_assertions)]
    pub fn corrupt_forest_cache_for_test(&self) -> bool {
        self.forest_cache.corrupt_for_test()
    }

    #[cfg(debug_assertions)]
    pub fn corrupt_journal_key_for_test(&self) -> bool {
        self.forest_cache.corrupt_journal_key_for_test()
    }

    pub(crate) fn verify_forest_cache_sample(&self) -> Result<(), SettlementStoreError> {
        self.verify_forest_cache_with_limit(1)
    }

    fn verify_forest_cache_with_limit(
        &self,
        sample_limit: usize,
    ) -> Result<(), SettlementStoreError> {
        self.forest_cache.sync_proof_version();
        if !self.forest_cache.enter_verify() {
            return Ok(());
        }
        let samples = self.forest_cache.metrics_and_samples(sample_limit);
        let result = self.verify_forest_cache_samples(samples);
        self.forest_cache.leave_verify();
        result
    }

    fn verify_forest_cache_samples(
        &self,
        samples: ForestCacheSamples,
    ) -> Result<(), SettlementStoreError> {
        let policy_id = self.bucket_policy().bucket_policy_id();
        let proof_version = HJMT_PROOF_ENVELOPE_VERSION as u16;
        let current_backend_root = self
            .hjmt_roots
            .def_root
            .map(|root| root.into_bytes())
            .unwrap_or(self.hjmt_empty_tree_root(self.hjmt_roots.version)?);

        for (key, root) in samples.subtree_root {
            if key.root_generation != ROOT_GENERATION || key.policy_id != policy_id {
                return Err(SettlementStoreError::Backend(
                    "forest cache subtree root key drift".to_string(),
                ));
            }
            let got = self.expected_subtree_root(key)?;
            if got != Some(root) {
                return Err(SettlementStoreError::Backend(
                    "forest cache subtree root drift".to_string(),
                ));
            }
        }

        for (key, encoded) in samples.parent_leaf {
            let (root_generation, key_policy_id) = match key {
                ParentLeafKey::Definition {
                    root_generation,
                    policy_id,
                    ..
                }
                | ParentLeafKey::Serial {
                    root_generation,
                    policy_id,
                    ..
                }
                | ParentLeafKey::Bucket {
                    root_generation,
                    policy_id,
                    ..
                } => (root_generation, policy_id),
            };
            if root_generation != ROOT_GENERATION || key_policy_id != policy_id {
                return Err(SettlementStoreError::Backend(
                    "forest cache parent leaf key drift".to_string(),
                ));
            }
            let got = self.expected_parent_leaf_payload(key)?;
            if got != encoded {
                return Err(SettlementStoreError::Backend(
                    "forest cache parent leaf drift".to_string(),
                ));
            }
        }

        for (key, value) in samples.terminal_leaf {
            let got = self.expected_terminal_leaf_value(key)?;
            if got != value {
                return Err(SettlementStoreError::Backend(
                    "forest cache terminal leaf drift".to_string(),
                ));
            }
        }

        for (key, bucket_id) in samples.bucket_derivation {
            let expected = key.path.bucket_id(self.bucket_policy());
            if key.root_generation != ROOT_GENERATION
                || key.policy_id != policy_id
                || key.bucket_bits != self.bucket_policy().bucket_bits()
                || expected != bucket_id
            {
                return Err(SettlementStoreError::Backend(
                    "forest cache bucket derivation drift".to_string(),
                ));
            }
        }

        for (key, bytes) in samples.proof_segment {
            if key.root_generation != ROOT_GENERATION || key.proof_version != proof_version {
                return Err(SettlementStoreError::Backend(
                    "forest cache proof segment key drift".to_string(),
                ));
            }
            let expected = self.encode_hjmt_maybe_empty_uncached(
                key.tree_id,
                jmt::KeyHash(key.key_hash),
                key.version,
            )?;
            if expected != bytes {
                return Err(SettlementStoreError::Backend(
                    "forest cache proof segment drift".to_string(),
                ));
            }
        }

        for (key, proof_blob) in samples.nonexistence {
            if key.root_generation != ROOT_GENERATION || key.proof_version != proof_version {
                return Err(SettlementStoreError::Backend(
                    "forest cache nonexistence key drift".to_string(),
                ));
            }
            let leaf_family = decode_leaf_family(key.leaf_family)?;
            if key.root != self.hjmt_roots.settlement_root()
                || key.epoch != self.hjmt_roots.version
                || proof_blob != self.hjmt_nonexistence_blob_uncached(&key.path, leaf_family)?
            {
                return Err(SettlementStoreError::Backend(
                    "forest cache nonexistence drift".to_string(),
                ));
            }
            self.validate_settlement_nonexistence_proof_blob(&proof_blob, leaf_family)?;
        }

        for (key, proof) in samples.policy_proof {
            match (key, proof) {
                (
                    PolicyProofKey::Split {
                        root_generation,
                        prior_root,
                        prior_epoch,
                        prior_policy_id,
                        next_policy_id,
                        bucket,
                    },
                    PolicyProofValue::Split(proof),
                ) => {
                    if root_generation != ROOT_GENERATION
                        || prior_root != proof.prior_root
                        || prior_epoch != proof.prior_epoch.get()
                        || prior_policy_id != self.bucket_policy().bucket_policy_id()
                        || next_policy_id != proof.bucket_policy_id
                        || self
                            .bucket_key_by_root(proof.prior_bucket_root)
                            .map_err(|err| SettlementStoreError::Backend(err.to_string()))?
                            != bucket
                    {
                        return Err(SettlementStoreError::Backend(
                            "forest cache split proof key drift".to_string(),
                        ));
                    }
                    self.validate_split_proof(&proof)
                        .map_err(|err| SettlementStoreError::Backend(err.to_string()))?;
                }
                (
                    PolicyProofKey::Merge {
                        root_generation,
                        prior_root,
                        prior_epoch,
                        prior_policy_id,
                        next_policy_id,
                        left,
                        right,
                    },
                    PolicyProofValue::Merge(proof),
                ) => {
                    if root_generation != ROOT_GENERATION
                        || prior_root != proof.prior_root
                        || prior_epoch != proof.prior_epoch.get()
                        || prior_policy_id != self.bucket_policy().bucket_policy_id()
                        || next_policy_id != proof.bucket_policy_id
                    {
                        return Err(SettlementStoreError::Backend(
                            "forest cache merge proof key drift".to_string(),
                        ));
                    }
                    let expected_left = self
                        .bucket_key_by_root(proof.left_bucket_root)
                        .map_err(|err| SettlementStoreError::Backend(err.to_string()))?;
                    let expected_right = self
                        .bucket_key_by_root(proof.right_bucket_root)
                        .map_err(|err| SettlementStoreError::Backend(err.to_string()))?;
                    if canonical_pair(expected_left, expected_right) != canonical_pair(left, right)
                    {
                        return Err(SettlementStoreError::Backend(
                            "forest cache merge scope drift".to_string(),
                        ));
                    }
                    self.validate_merge_proof(&proof)
                        .map_err(|err| SettlementStoreError::Backend(err.to_string()))?;
                }
                (
                    PolicyProofKey::Transition {
                        root_generation,
                        prior_root,
                        prior_epoch,
                        prior_policy_id,
                        next_policy_id,
                    },
                    PolicyProofValue::Transition(proof),
                ) => {
                    if root_generation != ROOT_GENERATION
                        || prior_root != proof.prior_root
                        || prior_epoch != proof.prior_epoch.get()
                        || prior_policy_id != proof.prior_policy_id
                        || next_policy_id != proof.next_policy_id
                    {
                        return Err(SettlementStoreError::Backend(
                            "forest cache transition proof key drift".to_string(),
                        ));
                    }
                    let next_policy = self.policy_from_id(next_policy_id)?;
                    self.validate_policy_transition_proof(&proof, next_policy)
                        .map_err(|err| SettlementStoreError::Backend(err.to_string()))?;
                }
                _ => {
                    return Err(SettlementStoreError::Backend(
                        "forest cache policy proof variant drift".to_string(),
                    ));
                }
            }
        }

        for (key, digest) in samples.journal_digest {
            if key.root_generation != ROOT_GENERATION || key.policy_id != policy_id {
                return Err(SettlementStoreError::Backend(
                    "forest cache journal key drift".to_string(),
                ));
            }
            let expected = self.current_journal_digest_uncached(key.backend_root);
            if key.version != self.hjmt_roots.version
                || key.status != HjmtCommitStatus::RootPublished.rank()
                || key.backend_root != current_backend_root
                || key.settlement_root != self.hjmt_roots.settlement_root()
                || expected != digest
            {
                return Err(SettlementStoreError::Backend(
                    "forest cache journal digest drift".to_string(),
                ));
            }
        }

        for (key, path) in samples.path_index {
            let got = self
                .path_by_terminal_id
                .get(&path.terminal_id())
                .copied()
                .ok_or_else(|| {
                    SettlementStoreError::Backend(
                        "forest cache path index missing current item".to_string(),
                    )
                })?;
            if key.terminal_id != path.terminal_id || got != path {
                return Err(SettlementStoreError::Backend(
                    "forest cache path index drift".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub(crate) fn warm_forest_cache_current(&self) -> Result<(), SettlementStoreError> {
        self.forest_cache.sync_proof_version();
        let version = self.hjmt_roots.version;
        let policy = self.bucket_policy();
        let policy_id = policy.bucket_policy_id();
        let terminal_entries = self
            .sorted_paths()
            .into_iter()
            .map(|path| {
                let item = self.hjmt_get_settlement_item(&path)?.ok_or_else(|| {
                    SettlementStoreError::Backend(
                        "forest cache warmup missing current item".to_string(),
                    )
                })?;
                let leaf_value = terminal_leaf_value(item.leaf())?;
                Ok((path, path.bucket_id(policy), leaf_value))
            })
            .collect::<Result<Vec<_>, SettlementStoreError>>()?;
        let journal_entry = self
            .hjmt_roots
            .journal_digest
            .map(|journal_digest| {
                let backend_root = self.hjmt_current_backend_root()?;
                Ok::<(JournalDigestKey, [u8; 32]), SettlementStoreError>((
                    JournalDigestKey {
                        root_generation: ROOT_GENERATION,
                        version,
                        status: HjmtCommitStatus::RootPublished.rank(),
                        backend_root,
                        settlement_root: self.hjmt_roots.settlement_root(),
                        policy_id,
                    },
                    journal_digest,
                ))
            })
            .transpose()?;

        {
            let mut inner = self.forest_cache.inner.borrow_mut();

            if let Some(root) = self.hjmt_roots.def_root {
                inner.subtree_root.insert(
                    SubtreeRootKey {
                        root_generation: ROOT_GENERATION,
                        version,
                        policy_id,
                        tree_id: HjmtTreeId::Definition,
                    },
                    root.into_bytes(),
                );
            }

            for (definition_id, root) in &self.hjmt_roots.serial_roots {
                inner.subtree_root.insert(
                    SubtreeRootKey {
                        root_generation: ROOT_GENERATION,
                        version,
                        policy_id,
                        tree_id: HjmtTreeId::Serial(*definition_id),
                    },
                    root.into_bytes(),
                );
                inner.parent_leaf.insert(
                    ParentLeafKey::Definition {
                        root_generation: ROOT_GENERATION,
                        version,
                        policy_id,
                        definition_id: *definition_id,
                        child_root: root.into_bytes(),
                    },
                    DefinitionRootLeaf {
                        definition_id: *definition_id,
                        definition_root: root.into_bytes(),
                    }
                    .encode(),
                );
            }

            for (key, root) in &self.hjmt_roots.bucket_roots {
                inner.subtree_root.insert(
                    SubtreeRootKey {
                        root_generation: ROOT_GENERATION,
                        version,
                        policy_id,
                        tree_id: HjmtTreeId::Bucket(key.0, key.1),
                    },
                    root.into_bytes(),
                );
                inner.parent_leaf.insert(
                    ParentLeafKey::Serial {
                        root_generation: ROOT_GENERATION,
                        version,
                        policy_id,
                        key: *key,
                        child_root: root.into_bytes(),
                    },
                    SerialRootLeaf {
                        definition_id: key.0,
                        serial_id: key.1,
                        serial_root: root.into_bytes(),
                    }
                    .encode(),
                );
            }

            for (key, root) in &self.hjmt_roots.terminal_roots {
                inner.subtree_root.insert(
                    SubtreeRootKey {
                        root_generation: ROOT_GENERATION,
                        version,
                        policy_id,
                        tree_id: HjmtTreeId::BucketTerminal(key.0, key.1, key.2),
                    },
                    root.into_bytes(),
                );
                inner.parent_leaf.insert(
                    ParentLeafKey::Bucket {
                        root_generation: ROOT_GENERATION,
                        version,
                        policy_id,
                        key: *key,
                        child_root: root.into_bytes(),
                    },
                    BucketRootLeaf {
                        definition_id: key.0,
                        serial_id: key.1,
                        bucket_id: key.2,
                        terminal_jmt_root: root.into_bytes(),
                        bucket_policy_id: policy_id,
                    }
                    .encode(),
                );
            }

            for (path, bucket_id, leaf_value) in terminal_entries {
                inner.path_index.insert(
                    PathIndexKey {
                        terminal_id: path.terminal_id,
                    },
                    path,
                );
                inner.bucket_derivation.insert(
                    BucketDerivationKey {
                        root_generation: ROOT_GENERATION,
                        epoch: version,
                        path,
                        policy_id,
                        bucket_bits: policy.bucket_bits(),
                    },
                    bucket_id,
                );
                inner.terminal_leaf.insert(leaf_value.0, leaf_value.1);
            }

            if let Some((key, digest)) = journal_entry {
                inner.journal_digest.insert(key, digest);
            }
        }

        Ok(())
    }

    pub(super) fn invalidate_forest_cache_for_ops(
        &self,
        ops: &[super::StoreOp],
        touched_buckets: &[HjmtBucketKey],
    ) {
        let mut terminal_ids = Vec::new();
        let mut defs = Vec::new();
        let mut serials = Vec::new();
        let mut buckets = Vec::new();

        for op in ops {
            match op {
                super::StoreOp::Put(item) => {
                    let path = item.path();
                    terminal_ids.push(path.terminal_id);
                    defs.push(path.definition_id);
                    serials.push((path.definition_id, path.serial_id));
                    buckets.push((
                        path.definition_id,
                        path.serial_id,
                        path.bucket_id(self.bucket_policy()),
                    ));
                }
                super::StoreOp::Delete(path) => {
                    terminal_ids.push(path.terminal_id);
                    defs.push(path.definition_id);
                    serials.push((path.definition_id, path.serial_id));
                    buckets.push((
                        path.definition_id,
                        path.serial_id,
                        path.bucket_id(self.bucket_policy()),
                    ));
                }
            }
        }
        buckets.extend_from_slice(touched_buckets);

        let terminal_ids = terminal_ids;
        let defs = defs;
        let serials = serials;
        let buckets = buckets;

        let mut inner = self.forest_cache.inner.borrow_mut();
        inner
            .path_index
            .invalidate_where(|key, _| terminal_ids.contains(&key.terminal_id));
        inner
            .terminal_leaf
            .invalidate_where(|key, _| terminal_ids.contains(&key.terminal_id));
        inner
            .bucket_derivation
            .invalidate_where(|key, _| terminal_ids.contains(&key.path.terminal_id));
        inner.nonexistence.clear();
        inner.journal_digest.clear();
        inner.policy_proof.invalidate_where(|key, _| match key {
            PolicyProofKey::Split { bucket, .. } => buckets.contains(bucket),
            PolicyProofKey::Merge { left, right, .. } => {
                buckets.contains(left) || buckets.contains(right)
            }
            PolicyProofKey::Transition { .. } => true,
        });
        inner
            .proof_segment
            .invalidate_where(|key, _| match key.tree_id {
                HjmtTreeId::Definition => defs
                    .iter()
                    .any(|definition_id| key.key_hash == definition_key(*definition_id).0),
                HjmtTreeId::Serial(definition_id) => serials.iter().any(|(def_id, serial_id)| {
                    definition_id == *def_id && key.key_hash == serial_key(*def_id, *serial_id).0
                }),
                HjmtTreeId::Bucket(definition_id, serial_id) => {
                    buckets.iter().any(|(def_id, ser_id, bucket_id)| {
                        definition_id == *def_id
                            && serial_id == *ser_id
                            && key.key_hash == super::hjmt_plan::bucket_key_for_path(*bucket_id).0
                    })
                }
                HjmtTreeId::BucketTerminal(definition_id, serial_id, bucket_id) => {
                    buckets.iter().any(|(def_id, ser_id, touched_bucket)| {
                        definition_id == *def_id
                            && serial_id == *ser_id
                            && bucket_id == *touched_bucket
                    })
                }
                HjmtTreeId::PathIndex => terminal_ids.iter().any(|terminal_id| {
                    key.key_hash == crate::settlement::keys::terminal_key(*terminal_id).0
                }),
            });
        inner
            .subtree_root
            .invalidate_where(|key, _| match key.tree_id {
                HjmtTreeId::Definition => {
                    defs.iter()
                        .any(|definition_id| key.tree_id == HjmtTreeId::Serial(*definition_id))
                        || !defs.is_empty()
                }
                HjmtTreeId::Serial(definition_id) => defs.contains(&definition_id),
                HjmtTreeId::Bucket(definition_id, serial_id) => {
                    serials.contains(&(definition_id, serial_id))
                }
                HjmtTreeId::BucketTerminal(definition_id, serial_id, bucket_id) => {
                    buckets.contains(&(definition_id, serial_id, bucket_id))
                }
                HjmtTreeId::PathIndex => terminal_ids.iter().any(|terminal_id| {
                    self.path_by_terminal_id
                        .get(terminal_id)
                        .is_some_and(|path| path.terminal_id == *terminal_id)
                }),
            });
        inner.parent_leaf.invalidate_where(|key, _| match key {
            ParentLeafKey::Definition { definition_id, .. } => defs.contains(definition_id),
            ParentLeafKey::Serial { key, .. } => serials.contains(key),
            ParentLeafKey::Bucket { key, .. } => buckets.contains(key),
        });
    }

    pub(crate) fn cached_path_for_terminal(
        &self,
        terminal_id: TerminalId,
    ) -> Option<SettlementPath> {
        let key = PathIndexKey { terminal_id };
        if let Some(path) = self
            .forest_cache
            .inner
            .borrow_mut()
            .path_index
            .get_cloned(&key)
        {
            return Some(path);
        }
        let path = self.path_by_terminal_id.get(&terminal_id).copied()?;
        self.forest_cache
            .inner
            .borrow_mut()
            .path_index
            .insert(key, path);
        Some(path)
    }

    pub(super) fn cached_bucket_id(&self, path: SettlementPath) -> BucketId {
        let key = BucketDerivationKey {
            root_generation: ROOT_GENERATION,
            epoch: self.hjmt_roots.version,
            path,
            policy_id: self.bucket_policy().bucket_policy_id(),
            bucket_bits: self.bucket_policy().bucket_bits(),
        };
        if let Some(bucket_id) = self
            .forest_cache
            .inner
            .borrow_mut()
            .bucket_derivation
            .get_cloned(&key)
        {
            return bucket_id;
        }
        let bucket_id = path.bucket_id(self.bucket_policy());
        self.forest_cache
            .inner
            .borrow_mut()
            .bucket_derivation
            .insert(key, bucket_id);
        bucket_id
    }

    pub(super) fn cached_terminal_leaf_value(
        &self,
        leaf: &SettlementLeaf,
    ) -> Result<TerminalLeafValue, SettlementStoreError> {
        let (key, value) = terminal_leaf_value(leaf)?;
        if let Some(value) = self
            .forest_cache
            .inner
            .borrow_mut()
            .terminal_leaf
            .get_cloned(&key)
        {
            return Ok(value);
        }
        self.forest_cache
            .inner
            .borrow_mut()
            .terminal_leaf
            .insert(key, value.clone());
        Ok(value)
    }

    pub(super) fn cached_definition_leaf(
        &self,
        version: u64,
        definition_id: DefinitionId,
        child_root: [u8; 32],
    ) -> Result<DefinitionRootLeaf, SettlementStoreError> {
        let _encoded = self.cached_parent_leaf_payload(ParentLeafKey::Definition {
            root_generation: ROOT_GENERATION,
            version,
            policy_id: self.bucket_policy().bucket_policy_id(),
            definition_id,
            child_root,
        })?;
        Ok(DefinitionRootLeaf {
            definition_id,
            definition_root: child_root,
        })
    }

    pub(super) fn cached_serial_leaf(
        &self,
        version: u64,
        key: HjmtSerialKey,
        child_root: [u8; 32],
    ) -> Result<SerialRootLeaf, SettlementStoreError> {
        let _encoded = self.cached_parent_leaf_payload(ParentLeafKey::Serial {
            root_generation: ROOT_GENERATION,
            version,
            policy_id: self.bucket_policy().bucket_policy_id(),
            key,
            child_root,
        })?;
        Ok(SerialRootLeaf {
            definition_id: key.0,
            serial_id: key.1,
            serial_root: child_root,
        })
    }

    pub(super) fn cached_bucket_leaf(
        &self,
        version: u64,
        key: HjmtBucketKey,
        child_root: [u8; 32],
    ) -> Result<BucketRootLeaf, SettlementStoreError> {
        let _encoded = self.cached_parent_leaf_payload(ParentLeafKey::Bucket {
            root_generation: ROOT_GENERATION,
            version,
            policy_id: self.bucket_policy().bucket_policy_id(),
            key,
            child_root,
        })?;
        Ok(BucketRootLeaf {
            definition_id: key.0,
            serial_id: key.1,
            bucket_id: key.2,
            terminal_jmt_root: child_root,
            bucket_policy_id: self.bucket_policy().bucket_policy_id(),
        })
    }

    pub(super) fn cached_subtree_root(
        &self,
        version: u64,
        tree_id: HjmtTreeId,
        _roots: &HjmtRoots,
    ) -> Result<Option<[u8; 32]>, SettlementStoreError> {
        let key = SubtreeRootKey {
            root_generation: ROOT_GENERATION,
            version,
            policy_id: self.bucket_policy().bucket_policy_id(),
            tree_id,
        };
        if let Some(root) = self
            .forest_cache
            .inner
            .borrow_mut()
            .subtree_root
            .get_cloned(&key)
        {
            return Ok(Some(root));
        }
        let root = self.expected_subtree_root(key)?;
        if let Some(root) = root {
            self.forest_cache
                .inner
                .borrow_mut()
                .subtree_root
                .insert(key, root);
        }
        Ok(root)
    }

    fn cached_parent_leaf_payload(
        &self,
        key: ParentLeafKey,
    ) -> Result<Vec<u8>, SettlementStoreError> {
        if let Some(encoded) = self
            .forest_cache
            .inner
            .borrow_mut()
            .parent_leaf
            .get_cloned(&key)
        {
            return Ok(encoded);
        }
        let encoded = self.expected_parent_leaf_payload(key)?;
        self.forest_cache
            .inner
            .borrow_mut()
            .parent_leaf
            .insert(key, encoded.clone());
        Ok(encoded)
    }

    pub(super) fn cached_proof_segment(
        &self,
        tree_id: HjmtTreeId,
        key_hash: [u8; 32],
        version: u64,
    ) -> Result<Option<Vec<u8>>, SettlementStoreError> {
        let key = ProofSegmentKey {
            root_generation: ROOT_GENERATION,
            version,
            tree_id,
            key_hash,
            proof_version: HJMT_PROOF_ENVELOPE_VERSION as u16,
        };
        Ok(self
            .forest_cache
            .inner
            .borrow_mut()
            .proof_segment
            .get_cloned(&key))
    }

    pub(super) fn store_proof_segment(
        &self,
        tree_id: HjmtTreeId,
        key_hash: [u8; 32],
        version: u64,
        bytes: Vec<u8>,
    ) {
        let key = ProofSegmentKey {
            root_generation: ROOT_GENERATION,
            version,
            tree_id,
            key_hash,
            proof_version: HJMT_PROOF_ENVELOPE_VERSION as u16,
        };
        self.forest_cache
            .inner
            .borrow_mut()
            .proof_segment
            .insert(key, bytes);
    }

    pub(super) fn cached_nonexistence_proof_blob(
        &self,
        path: SettlementPath,
        leaf_family: SettlementLeafFamily,
    ) -> Option<ProofBlob> {
        let key = NonExistenceKey {
            root_generation: ROOT_GENERATION,
            root: self.hjmt_roots.settlement_root(),
            path,
            epoch: self.hjmt_roots.version,
            proof_version: HJMT_PROOF_ENVELOPE_VERSION as u16,
            leaf_family: encode_leaf_family(leaf_family),
        };
        self.forest_cache
            .inner
            .borrow_mut()
            .nonexistence
            .get_cloned(&key)
    }

    pub(super) fn store_nonexistence_proof_blob(
        &self,
        path: SettlementPath,
        leaf_family: SettlementLeafFamily,
        proof_blob: ProofBlob,
    ) {
        let key = NonExistenceKey {
            root_generation: ROOT_GENERATION,
            root: self.hjmt_roots.settlement_root(),
            path,
            epoch: self.hjmt_roots.version,
            proof_version: HJMT_PROOF_ENVELOPE_VERSION as u16,
            leaf_family: encode_leaf_family(leaf_family),
        };
        self.forest_cache
            .inner
            .borrow_mut()
            .nonexistence
            .insert(key, proof_blob);
    }

    pub(super) fn cached_policy_split(
        &self,
        key: HjmtBucketKey,
        next_policy_id: [u8; 32],
    ) -> Option<SplitProof> {
        let key = PolicyProofKey::Split {
            root_generation: ROOT_GENERATION,
            prior_root: self.hjmt_roots.settlement_root(),
            prior_epoch: self.hjmt_roots.version,
            prior_policy_id: self.bucket_policy().bucket_policy_id(),
            next_policy_id,
            bucket: key,
        };
        match self
            .forest_cache
            .inner
            .borrow_mut()
            .policy_proof
            .get_cloned(&key)
        {
            Some(PolicyProofValue::Split(proof)) => Some(proof),
            _ => None,
        }
    }

    pub(super) fn store_policy_split(
        &self,
        key: HjmtBucketKey,
        next_policy_id: [u8; 32],
        proof: SplitProof,
    ) {
        self.forest_cache.inner.borrow_mut().policy_proof.insert(
            PolicyProofKey::Split {
                root_generation: ROOT_GENERATION,
                prior_root: proof.prior_root,
                prior_epoch: proof.prior_epoch.get(),
                prior_policy_id: self.bucket_policy().bucket_policy_id(),
                next_policy_id,
                bucket: key,
            },
            PolicyProofValue::Split(proof),
        );
    }

    pub(super) fn cached_policy_merge(
        &self,
        left: HjmtBucketKey,
        right: HjmtBucketKey,
        next_policy_id: [u8; 32],
    ) -> Option<MergeProof> {
        let (left, right) = canonical_pair(left, right);
        let key = PolicyProofKey::Merge {
            root_generation: ROOT_GENERATION,
            prior_root: self.hjmt_roots.settlement_root(),
            prior_epoch: self.hjmt_roots.version,
            prior_policy_id: self.bucket_policy().bucket_policy_id(),
            next_policy_id,
            left,
            right,
        };
        match self
            .forest_cache
            .inner
            .borrow_mut()
            .policy_proof
            .get_cloned(&key)
        {
            Some(PolicyProofValue::Merge(proof)) => Some(proof),
            _ => None,
        }
    }

    pub(super) fn store_policy_merge(
        &self,
        left: HjmtBucketKey,
        right: HjmtBucketKey,
        next_policy_id: [u8; 32],
        proof: MergeProof,
    ) {
        let (left, right) = canonical_pair(left, right);
        self.forest_cache.inner.borrow_mut().policy_proof.insert(
            PolicyProofKey::Merge {
                root_generation: ROOT_GENERATION,
                prior_root: proof.prior_root,
                prior_epoch: proof.prior_epoch.get(),
                prior_policy_id: self.bucket_policy().bucket_policy_id(),
                next_policy_id,
                left,
                right,
            },
            PolicyProofValue::Merge(proof),
        );
    }

    pub(super) fn cached_policy_transition(
        &self,
        next_policy_id: [u8; 32],
    ) -> Option<PolicyTransitionProof> {
        let key = PolicyProofKey::Transition {
            root_generation: ROOT_GENERATION,
            prior_root: self.hjmt_roots.settlement_root(),
            prior_epoch: self.hjmt_roots.version,
            prior_policy_id: self.bucket_policy().bucket_policy_id(),
            next_policy_id,
        };
        match self
            .forest_cache
            .inner
            .borrow_mut()
            .policy_proof
            .get_cloned(&key)
        {
            Some(PolicyProofValue::Transition(proof)) => Some(proof),
            _ => None,
        }
    }

    pub(super) fn store_policy_transition(
        &self,
        next_policy_id: [u8; 32],
        proof: PolicyTransitionProof,
    ) {
        self.forest_cache.inner.borrow_mut().policy_proof.insert(
            PolicyProofKey::Transition {
                root_generation: ROOT_GENERATION,
                prior_root: proof.prior_root,
                prior_epoch: proof.prior_epoch.get(),
                prior_policy_id: proof.prior_policy_id,
                next_policy_id,
            },
            PolicyProofValue::Transition(proof),
        );
    }

    pub(super) fn cached_current_journal_digest(&self, backend_root: [u8; 32]) -> Option<[u8; 32]> {
        let key = JournalDigestKey {
            root_generation: ROOT_GENERATION,
            version: self.hjmt_roots.version,
            status: HjmtCommitStatus::RootPublished.rank(),
            backend_root,
            settlement_root: self.hjmt_roots.settlement_root(),
            policy_id: self.bucket_policy().bucket_policy_id(),
        };
        self.forest_cache
            .inner
            .borrow_mut()
            .journal_digest
            .get_cloned(&key)
    }

    pub(super) fn store_current_journal_digest(&self, backend_root: [u8; 32], digest: [u8; 32]) {
        let key = JournalDigestKey {
            root_generation: ROOT_GENERATION,
            version: self.hjmt_roots.version,
            status: HjmtCommitStatus::RootPublished.rank(),
            backend_root,
            settlement_root: self.hjmt_roots.settlement_root(),
            policy_id: self.bucket_policy().bucket_policy_id(),
        };
        self.forest_cache
            .inner
            .borrow_mut()
            .journal_digest
            .insert(key, digest);
    }

    pub(super) fn current_journal_digest_uncached(&self, backend_root: [u8; 32]) -> [u8; 32] {
        self.hjmt_roots.journal_digest.unwrap_or_else(|| {
            crate::settlement::proof::hjmt_checkpoint_digest(
                self.hjmt_roots.settlement_root(),
                backend_root,
                self.hjmt_roots.version,
            )
        })
    }

    fn expected_subtree_root(
        &self,
        key: SubtreeRootKey,
    ) -> Result<Option<[u8; 32]>, SettlementStoreError> {
        let roots = self.roots_for_version(key.version)?;
        Ok(match key.tree_id {
            HjmtTreeId::Definition => roots.def_root.map(|root| root.into_bytes()),
            HjmtTreeId::Serial(definition_id) => roots
                .serial_roots
                .get(&definition_id)
                .copied()
                .map(|root| root.into_bytes()),
            HjmtTreeId::Bucket(definition_id, serial_id) => roots
                .bucket_roots
                .get(&(definition_id, serial_id))
                .copied()
                .map(|root| root.into_bytes()),
            HjmtTreeId::BucketTerminal(definition_id, serial_id, bucket_id) => roots
                .terminal_roots
                .get(&(definition_id, serial_id, bucket_id))
                .copied()
                .map(|root| root.into_bytes()),
            HjmtTreeId::PathIndex => None,
        })
    }

    fn expected_parent_leaf_payload(
        &self,
        key: ParentLeafKey,
    ) -> Result<Vec<u8>, SettlementStoreError> {
        match key {
            ParentLeafKey::Definition {
                version,
                definition_id,
                child_root,
                ..
            } => {
                let model = self.model_for_version(version)?;
                if !model_has_definition(&model, definition_id) {
                    return Err(SettlementStoreError::Backend(
                        "forest cache definition leaf missing".to_string(),
                    ));
                }
                Ok(DefinitionRootLeaf {
                    definition_id,
                    definition_root: child_root,
                }
                .encode())
            }
            ParentLeafKey::Serial {
                version,
                key,
                child_root,
                ..
            } => {
                let model = self.model_for_version(version)?;
                if !model_has_serial(&model, key.0, key.1) {
                    return Err(SettlementStoreError::Backend(
                        "forest cache serial leaf missing".to_string(),
                    ));
                }
                Ok(SerialRootLeaf {
                    definition_id: key.0,
                    serial_id: key.1,
                    serial_root: child_root,
                }
                .encode())
            }
            ParentLeafKey::Bucket {
                version,
                key,
                child_root,
                ..
            } => {
                let model = self.model_for_version(version)?;
                if !model_has_bucket(&model, self.bucket_policy(), key) {
                    return Err(SettlementStoreError::Backend(
                        "forest cache bucket leaf missing".to_string(),
                    ));
                }
                Ok(BucketRootLeaf {
                    definition_id: key.0,
                    serial_id: key.1,
                    bucket_id: key.2,
                    terminal_jmt_root: child_root,
                    bucket_policy_id: self.bucket_policy().bucket_policy_id(),
                }
                .encode())
            }
        }
    }

    fn expected_terminal_leaf_value(
        &self,
        key: TerminalLeafKey,
    ) -> Result<TerminalLeafValue, SettlementStoreError> {
        let path = self
            .sorted_paths()
            .into_iter()
            .find(|path| path.terminal_id == key.terminal_id)
            .ok_or_else(|| {
                SettlementStoreError::Backend("forest cache terminal path missing".to_string())
            })?;
        let item = self.hjmt_get_settlement_item(&path)?.ok_or_else(|| {
            SettlementStoreError::Backend("forest cache terminal item missing".to_string())
        })?;
        let (got_key, got_value) = terminal_leaf_value(item.leaf())?;
        if got_key != key {
            return Err(SettlementStoreError::Backend(
                "forest cache terminal leaf key drift".to_string(),
            ));
        }
        Ok(got_value)
    }

    fn roots_for_version(&self, version: u64) -> Result<HjmtRoots, SettlementStoreError> {
        if version == self.hjmt_roots.version {
            return Ok(self.hjmt_roots.clone());
        }
        let Some((_, roots)) = self.hjmt_history_at(version)? else {
            return Err(SettlementStoreError::HistMiss);
        };
        Ok(roots)
    }

    fn model_for_version(&self, version: u64) -> Result<SettlementModel, SettlementStoreError> {
        if version == self.hjmt_roots.version {
            return Ok(self.model.clone());
        }
        let Some((model, _)) = self.hjmt_history_at(version)? else {
            return Err(SettlementStoreError::HistMiss);
        };
        Ok(model)
    }

    fn policy_from_id(
        &self,
        next_policy_id: [u8; 32],
    ) -> Result<BucketPolicy, SettlementStoreError> {
        let current = self.bucket_policy();
        for bits in 1..=32 {
            for generation in current.compatibility_generation()
                ..=current.compatibility_generation().saturating_add(4)
            {
                let policy = BucketPolicy::new(
                    bits,
                    current.min_bucket_count(),
                    current.max_target_leaf_count(),
                    generation,
                )
                .map_err(|err| SettlementStoreError::Backend(err.to_string()))?;
                if policy.bucket_policy_id() == next_policy_id {
                    return Ok(policy);
                }
            }
        }
        Err(SettlementStoreError::Backend(
            "forest cache could not recover next bucket policy".to_string(),
        ))
    }
}

fn terminal_leaf_value(
    leaf: &SettlementLeaf,
) -> Result<(TerminalLeafKey, TerminalLeafValue), SettlementStoreError> {
    let encoded = leaf.encode()?;
    let leaf_hash = terminal_value_hash(leaf.clone())?.0;
    let family = match leaf {
        SettlementLeaf::Terminal(asset) => TerminalLeafKey {
            root_generation: ROOT_GENERATION,
            terminal_id: asset.terminal_id(),
            leaf_family: encode_leaf_family(SettlementLeafFamily::Terminal),
            codec_version: 1,
            payload_hash: leaf_hash,
        },
        SettlementLeaf::Right(right) => TerminalLeafKey {
            root_generation: ROOT_GENERATION,
            terminal_id: right.terminal_id,
            leaf_family: encode_leaf_family(SettlementLeafFamily::Right),
            codec_version: 1,
            payload_hash: leaf_hash,
        },
        SettlementLeaf::Voucher(voucher) => TerminalLeafKey {
            root_generation: ROOT_GENERATION,
            terminal_id: voucher.terminal_id,
            leaf_family: encode_leaf_family(SettlementLeafFamily::Voucher),
            codec_version: 1,
            payload_hash: leaf_hash,
        },
    };
    Ok((family, TerminalLeafValue { encoded, leaf_hash }))
}

fn encode_leaf_family(family: SettlementLeafFamily) -> u8 {
    match family {
        SettlementLeafFamily::Terminal => 1,
        SettlementLeafFamily::Right => 2,
        SettlementLeafFamily::Voucher => 3,
    }
}

fn decode_leaf_family(family: u8) -> Result<SettlementLeafFamily, SettlementStoreError> {
    match family {
        1 => Ok(SettlementLeafFamily::Terminal),
        2 => Ok(SettlementLeafFamily::Right),
        3 => Ok(SettlementLeafFamily::Voucher),
        _ => Err(SettlementStoreError::Backend(
            "forest cache leaf family tag drift".to_string(),
        )),
    }
}

fn canonical_pair(left: HjmtBucketKey, right: HjmtBucketKey) -> (HjmtBucketKey, HjmtBucketKey) {
    if left <= right {
        (left, right)
    } else {
        (right, left)
    }
}

fn model_has_bucket(model: &SettlementModel, policy: BucketPolicy, key: HjmtBucketKey) -> bool {
    model.paths().into_iter().any(|path| {
        path.definition_id == key.0
            && path.serial_id == key.1
            && policy.derive_bucket_id(path) == key.2
    })
}

fn model_has_serial(
    model: &SettlementModel,
    definition_id: DefinitionId,
    serial_id: SerialId,
) -> bool {
    model
        .paths()
        .into_iter()
        .any(|path| path.definition_id == definition_id && path.serial_id == serial_id)
}

fn model_has_definition(model: &SettlementModel, definition_id: DefinitionId) -> bool {
    model
        .paths()
        .into_iter()
        .any(|path| path.definition_id == definition_id)
}
