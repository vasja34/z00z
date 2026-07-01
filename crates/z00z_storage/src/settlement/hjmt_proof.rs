use jmt::{KeyHash, Sha256Jmt};
use z00z_utils::codec::{BincodeCodec, Codec};

use super::{
    hjmt_config::env_value,
    hjmt_plan::bucket_key_for_path,
    model::SettlementModel,
    tree_id::{HjmtTreeId, TreeRootRef},
    SettlementPath, SettlementStore, SettlementStoreError,
};
use crate::backend::{
    codec::map_jmt_err,
    memory::{apply_batch, MemTreeStore},
    roots::HjmtRoots,
    types::terminal_value_hash,
};
use crate::settlement::keys::{definition_key, serial_key};
use crate::settlement::proof::{
    hjmt_checkpoint_digest, hjmt_default_value_commitment, HjmtPriorProofEnvelope,
    SettlementLeafFamily,
};
use crate::settlement::HjmtProofFamily;
use crate::settlement::{
    BucketRootLeaf, DefinitionRootLeaf, ProofBlob, ProofItem, ProofScanOut, SerialRootLeaf,
};

const PROOF_BATCH_MODE_ENV: &str = "Z00Z_STORAGE_PROOF_BATCH_MODE";
const PROOF_BATCH_MIN_ENV: &str = "Z00Z_STORAGE_PROOF_BATCH_MIN";

struct HjmtCurrentPathProofContext {
    backend_root: [u8; 32],
    def_leaf: DefinitionRootLeaf,
    ser_leaf: SerialRootLeaf,
    bucket_leaf: BucketRootLeaf,
    definition_proof: Vec<u8>,
    serial_proof: Vec<u8>,
    bucket_proof: Vec<u8>,
    terminal_proof: Vec<u8>,
}

struct ProofJob {
    store: SettlementStore,
    path: SettlementPath,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProofBatchMode {
    Serial,
    Parallel,
}

fn proof_batch_mode() -> ProofBatchMode {
    match env_value(PROOF_BATCH_MODE_ENV)
        .ok()
        .flatten()
        .as_deref()
        .map(str::trim)
    {
        Some("parallel") => ProofBatchMode::Parallel,
        _ => ProofBatchMode::Serial,
    }
}

fn proof_batch_min() -> usize {
    env_value(PROOF_BATCH_MIN_ENV)
        .ok()
        .flatten()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(4)
}

impl SettlementStore {
    pub(crate) fn hjmt_current_backend_root(&self) -> Result<[u8; 32], SettlementStoreError> {
        self.hjmt_backend_root_for_roots(&self.hjmt_roots)
    }

    pub(crate) fn hjmt_current_journal_digest(&self, backend_root: [u8; 32]) -> [u8; 32] {
        if let Some(digest) = self.cached_current_journal_digest(backend_root) {
            return digest;
        }
        let digest = self.current_journal_digest_uncached(backend_root);
        self.store_current_journal_digest(backend_root, digest);
        digest
    }

    pub(crate) fn hjmt_backend_root_for_roots(
        &self,
        roots: &HjmtRoots,
    ) -> Result<[u8; 32], SettlementStoreError> {
        if let Some(root) =
            self.cached_subtree_root(roots.version, HjmtTreeId::Definition, roots)?
        {
            return Ok(root);
        }
        self.hjmt_empty_tree_root(roots.version)
    }

    pub(crate) fn hjmt_settlement_proof_item(
        &self,
        path: &SettlementPath,
    ) -> Result<ProofItem, SettlementStoreError> {
        let item = self
            .hjmt_get_settlement_item(path)?
            .ok_or(SettlementStoreError::PathMiss)?;
        let def_leaf = self.hjmt_need_def_leaf(path)?;
        let ser_leaf = self.hjmt_need_ser_leaf(path)?;

        Ok(ProofItem::new_settlement(
            self.hjmt_roots.settlement_root(),
            *path,
            def_leaf,
            ser_leaf,
            item.leaf().clone(),
        )?)
    }

    pub(crate) fn hjmt_settlement_proof_blob(
        &self,
        path: &SettlementPath,
    ) -> Result<ProofBlob, SettlementStoreError> {
        if self.hjmt_get_settlement_item(path)?.is_none() {
            return self.hjmt_settlement_deletion_proof_blob(path);
        }

        let proof = self.hjmt_settlement_proof_item(path)?;
        let bucket_leaf = self.hjmt_need_bucket_leaf(path)?;
        let terminal_leaf = self.cached_terminal_leaf_value(proof.leaf())?;
        let terminal_leaf_hash = terminal_leaf.leaf_hash;
        let backend_root = self.hjmt_need_backend_root()?.into_bytes();
        let definition_proof = self.encode_hjmt_def_proof(path)?;
        let serial_proof = self.encode_hjmt_ser_proof(path)?;
        let bucket_proof = self.encode_hjmt_bucket_proof(path, bucket_leaf)?;
        let terminal_proof = self.encode_hjmt_terminal_proof(path, bucket_leaf)?;
        let journal_digest = self.hjmt_current_journal_digest(backend_root);

        Ok(ProofBlob::new_forest(
            proof,
            terminal_leaf_hash,
            backend_root,
            self.bucket_policy(),
            bucket_leaf,
            definition_proof,
            serial_proof,
            bucket_proof,
            terminal_proof,
            HjmtProofFamily::Inclusion,
            Some(self.hjmt_roots.version),
            Some(journal_digest),
        ))
    }

    pub fn settlement_proof_blobs(
        &self,
        paths: &[SettlementPath],
    ) -> Result<Vec<ProofBlob>, SettlementStoreError> {
        self.require_hjmt_mode()?;
        if paths.is_empty() {
            return Ok(Vec::new());
        }
        if proof_batch_mode() == ProofBatchMode::Serial || paths.len() < proof_batch_min() {
            return self.sched_run_serial_batch("hjmt_proof_batch", paths.len(), || {
                paths
                    .iter()
                    .map(|path| self.hjmt_settlement_proof_blob(path))
                    .collect()
            });
        }
        let jobs = paths
            .iter()
            .copied()
            .map(|path| ProofJob {
                store: self.fork_sched_view(),
                path,
            })
            .collect::<Vec<_>>();
        self.sched_map("hjmt_proof_batch", jobs, |job| {
            job.store.hjmt_settlement_proof_blob(&job.path)
        })
    }

    pub(crate) fn hjmt_settlement_nonexistence_proof_blob(
        &self,
        path: &SettlementPath,
        leaf_family: SettlementLeafFamily,
    ) -> Result<ProofBlob, SettlementStoreError> {
        if self.hjmt_get_settlement_item(path)?.is_some() {
            return Err(SettlementStoreError::PathMiss);
        }
        if let Some(proof_blob) = self.cached_nonexistence_proof_blob(*path, leaf_family) {
            return Ok(proof_blob);
        }
        let proof_blob = self.hjmt_nonexistence_blob_uncached(path, leaf_family)?;
        self.store_nonexistence_proof_blob(*path, leaf_family, proof_blob.clone());
        Ok(proof_blob)
    }

    pub(super) fn hjmt_nonexistence_blob_uncached(
        &self,
        path: &SettlementPath,
        leaf_family: SettlementLeafFamily,
    ) -> Result<ProofBlob, SettlementStoreError> {
        if self.hjmt_get_settlement_item(path)?.is_some() {
            return Err(SettlementStoreError::PathMiss);
        }

        let current = self.hjmt_current_path_proof_ctx(path)?;
        let marker_leaf = leaf_family.marker_leaf(*path);
        let proof = ProofItem::new_settlement(
            self.hjmt_roots.settlement_root(),
            *path,
            current.def_leaf,
            current.ser_leaf,
            marker_leaf.clone(),
        )?;

        Ok(ProofBlob::new_forest(
            proof,
            terminal_value_hash(marker_leaf.clone())?.0,
            current.backend_root,
            self.bucket_policy(),
            current.bucket_leaf,
            current.definition_proof,
            current.serial_proof,
            current.bucket_proof,
            current.terminal_proof,
            HjmtProofFamily::NonExistence,
            Some(self.hjmt_roots.version),
            Some(self.hjmt_current_journal_digest(current.backend_root)),
        )
        .with_hjmt_default_commitment(Some(hjmt_default_value_commitment()))
        .with_hjmt_leaf_family(leaf_family))
    }

    pub(crate) fn hjmt_settlement_proof_scan(
        &self,
        path: &SettlementPath,
    ) -> Result<ProofScanOut, SettlementStoreError> {
        let proof_blob = self.hjmt_settlement_proof_blob(path)?;
        self.validate_settlement_proof_blob(&proof_blob)?;
        if proof_blob.hjmt_proof_family() != Some(HjmtProofFamily::Inclusion) {
            return Err(SettlementStoreError::Proof(
                crate::settlement::ProofChkErr::ProofFamilyMix,
            ));
        }
        Ok(ProofScanOut::from_blob(proof_blob))
    }

    fn hjmt_settlement_deletion_proof_blob(
        &self,
        path: &SettlementPath,
    ) -> Result<ProofBlob, SettlementStoreError> {
        let deleted_version = self.hjmt_last_version_for_path(path)?;
        let hist_store = self.hjmt_store_at(deleted_version)?;
        let prior_store = hist_store.as_ref().unwrap_or(self);
        let (deleted_model, deleted_roots) = if let Some(store) = hist_store.as_ref() {
            (store.model.clone(), store.hjmt_roots.clone())
        } else {
            let model = self
                .model_by_ver
                .get(&deleted_version)
                .cloned()
                .ok_or(SettlementStoreError::HistMiss)?;
            let roots = self
                .hjmt_roots_by_ver
                .get(&deleted_version)
                .cloned()
                .ok_or(SettlementStoreError::HistMiss)?;
            (model, roots)
        };
        let deleted_item = deleted_model
            .item_opt(path)?
            .ok_or(SettlementStoreError::PathMiss)?;
        let deleted_leaf = deleted_item.leaf().clone();

        let current = self.hjmt_current_path_proof_ctx(path)?;
        let proof = ProofItem::new_settlement(
            self.hjmt_roots.settlement_root(),
            *path,
            current.def_leaf,
            current.ser_leaf,
            deleted_leaf.clone(),
        )?;
        let prior_backend_root = prior_store
            .hjmt_need_backend_root_at(&deleted_roots)?
            .into_bytes();
        let prior_journal_digest = deleted_roots.journal_digest.unwrap_or_else(|| {
            hjmt_checkpoint_digest(
                deleted_roots.settlement_root(),
                prior_backend_root,
                deleted_version,
            )
        });
        let prior = HjmtPriorProofEnvelope::new(
            deleted_version,
            deleted_roots.settlement_root(),
            prior_backend_root,
            prior_journal_digest,
            prior_store.hjmt_need_def_leaf_at(&deleted_roots, &deleted_model, path)?,
            prior_store.hjmt_need_ser_leaf_at(&deleted_roots, &deleted_model, path)?,
            prior_store.hjmt_need_bucket_leaf_at(&deleted_roots, &deleted_model, path)?,
            prior_store.encode_hjmt_def_proof_at(path, deleted_version)?,
            prior_store.encode_hjmt_ser_proof_at(path, deleted_version)?,
            prior_store.encode_hjmt_bucket_proof_at(
                path,
                prior_store.hjmt_need_bucket_leaf_at(&deleted_roots, &deleted_model, path)?,
                deleted_version,
            )?,
            prior_store.encode_hjmt_terminal_proof_at(
                path,
                prior_store.hjmt_need_bucket_leaf_at(&deleted_roots, &deleted_model, path)?,
                deleted_version,
            )?,
        );

        Ok(ProofBlob::new_forest(
            proof,
            terminal_value_hash(deleted_leaf)?.0,
            current.backend_root,
            self.bucket_policy(),
            current.bucket_leaf,
            current.definition_proof,
            current.serial_proof,
            current.bucket_proof,
            current.terminal_proof,
            HjmtProofFamily::Deletion,
            Some(self.hjmt_roots.version),
            Some(self.hjmt_current_journal_digest(current.backend_root)),
        )
        .with_hjmt_prior(prior))
    }

    fn hjmt_current_path_proof_ctx(
        &self,
        path: &SettlementPath,
    ) -> Result<HjmtCurrentPathProofContext, SettlementStoreError> {
        let version = self.hjmt_roots.version;
        let bucket_id = self.cached_bucket_id(*path);
        let bucket_policy_id = self.bucket_policy().bucket_policy_id();
        let empty_root = self.hjmt_empty_tree_root(version)?;
        let definition_proof = self.encode_hjmt_proof_allow_empty(
            HjmtTreeId::Definition,
            definition_key(path.definition_id),
            version,
        )?;
        let backend_root = self
            .hjmt_roots
            .def_root
            .map(TreeRootRef::into_bytes)
            .unwrap_or(empty_root);

        let synthetic_bucket_leaf = |terminal_jmt_root| BucketRootLeaf {
            definition_id: path.definition_id,
            serial_id: path.serial_id,
            bucket_id,
            terminal_jmt_root,
            bucket_policy_id,
        };

        let Some(def_leaf) =
            self.definition_leaf_from_roots(&self.hjmt_roots, &self.model, path.definition_id)?
        else {
            return Ok(HjmtCurrentPathProofContext {
                backend_root,
                def_leaf: DefinitionRootLeaf {
                    definition_id: path.definition_id,
                    definition_root: empty_root,
                },
                ser_leaf: SerialRootLeaf {
                    definition_id: path.definition_id,
                    serial_id: path.serial_id,
                    serial_root: empty_root,
                },
                bucket_leaf: synthetic_bucket_leaf(empty_root),
                definition_proof,
                serial_proof: Vec::new(),
                bucket_proof: Vec::new(),
                terminal_proof: Vec::new(),
            });
        };

        let serial_proof = self.encode_hjmt_proof_allow_empty(
            HjmtTreeId::Serial(path.definition_id),
            serial_key(path.definition_id, path.serial_id),
            version,
        )?;

        let Some(ser_leaf) = self.serial_leaf_from_roots(
            &self.hjmt_roots,
            &self.model,
            (path.definition_id, path.serial_id),
        )?
        else {
            return Ok(HjmtCurrentPathProofContext {
                backend_root,
                def_leaf,
                ser_leaf: SerialRootLeaf {
                    definition_id: path.definition_id,
                    serial_id: path.serial_id,
                    serial_root: empty_root,
                },
                bucket_leaf: synthetic_bucket_leaf(empty_root),
                definition_proof,
                serial_proof,
                bucket_proof: Vec::new(),
                terminal_proof: Vec::new(),
            });
        };

        let bucket_proof = self.encode_hjmt_proof_allow_empty(
            HjmtTreeId::Bucket(path.definition_id, path.serial_id),
            bucket_key_for_path(bucket_id),
            version,
        )?;

        let Some(bucket_leaf) = self.bucket_leaf_from_roots(
            &self.hjmt_roots,
            &self.model,
            (path.definition_id, path.serial_id, bucket_id),
        )?
        else {
            return Ok(HjmtCurrentPathProofContext {
                backend_root,
                def_leaf,
                ser_leaf,
                bucket_leaf: synthetic_bucket_leaf(empty_root),
                definition_proof,
                serial_proof,
                bucket_proof,
                terminal_proof: Vec::new(),
            });
        };

        let terminal_proof = self.encode_hjmt_proof_allow_empty(
            HjmtTreeId::BucketTerminal(path.definition_id, path.serial_id, bucket_id),
            crate::settlement::keys::terminal_key(path.terminal_id()),
            version,
        )?;

        Ok(HjmtCurrentPathProofContext {
            backend_root,
            def_leaf,
            ser_leaf,
            bucket_leaf,
            definition_proof,
            serial_proof,
            bucket_proof,
            terminal_proof,
        })
    }

    fn hjmt_need_backend_root(&self) -> Result<TreeRootRef, SettlementStoreError> {
        self.hjmt_need_backend_root_at(&self.hjmt_roots)
    }

    fn hjmt_need_backend_root_at(
        &self,
        roots: &HjmtRoots,
    ) -> Result<TreeRootRef, SettlementStoreError> {
        let root = self
            .cached_subtree_root(roots.version, HjmtTreeId::Definition, roots)?
            .ok_or_else(|| {
                SettlementStoreError::Jmt("missing hjmt definition proof root".to_string())
            })?;
        Ok(TreeRootRef::new(root))
    }

    pub(crate) fn hjmt_last_version_for_path(
        &self,
        path: &SettlementPath,
    ) -> Result<u64, SettlementStoreError> {
        let mut versions: Vec<_> = self.model_by_ver.keys().copied().collect();
        versions.sort_unstable();
        for version in versions.into_iter().rev() {
            let model = self
                .model_by_ver
                .get(&version)
                .ok_or(SettlementStoreError::HistMiss)?;
            if model.item_opt(path)?.is_some() {
                return Ok(version);
            }
        }

        if self.backend.is_on() {
            return self
                .backend
                .hjmt_last_version_for_path(*path)
                .map_err(|err| SettlementStoreError::Backend(err.to_string()))?
                .ok_or(SettlementStoreError::PathMiss);
        }

        Err(SettlementStoreError::PathMiss)
    }

    fn hjmt_need_def_leaf_at(
        &self,
        roots: &HjmtRoots,
        model: &SettlementModel,
        path: &SettlementPath,
    ) -> Result<DefinitionRootLeaf, SettlementStoreError> {
        if !model
            .paths()
            .into_iter()
            .any(|next| next.definition_id == path.definition_id)
        {
            return Err(SettlementStoreError::Jmt(
                "missing hjmt definition root leaf".to_string(),
            ));
        }
        let root = self
            .cached_subtree_root(roots.version, HjmtTreeId::Serial(path.definition_id), roots)?
            .ok_or_else(|| {
                SettlementStoreError::Jmt("missing hjmt definition root leaf".to_string())
            })?;
        self.cached_definition_leaf(roots.version, path.definition_id, root)
    }

    fn hjmt_need_ser_leaf_at(
        &self,
        roots: &HjmtRoots,
        model: &SettlementModel,
        path: &SettlementPath,
    ) -> Result<SerialRootLeaf, SettlementStoreError> {
        if !model.paths().into_iter().any(|next| {
            next.definition_id == path.definition_id && next.serial_id == path.serial_id
        }) {
            return Err(SettlementStoreError::Jmt(
                "missing hjmt serial root leaf".to_string(),
            ));
        }
        let root = self
            .cached_subtree_root(
                roots.version,
                HjmtTreeId::Bucket(path.definition_id, path.serial_id),
                roots,
            )?
            .ok_or_else(|| {
                SettlementStoreError::Jmt("missing hjmt serial root leaf".to_string())
            })?;
        self.cached_serial_leaf(roots.version, (path.definition_id, path.serial_id), root)
    }

    fn hjmt_need_bucket_leaf_at(
        &self,
        roots: &HjmtRoots,
        model: &SettlementModel,
        path: &SettlementPath,
    ) -> Result<BucketRootLeaf, SettlementStoreError> {
        let bucket_id = self.bucket_policy().derive_bucket_id(*path);
        if model.item_opt(path)?.is_none() {
            return Err(SettlementStoreError::Jmt(
                "missing hjmt bucket root leaf".to_string(),
            ));
        }
        let root = self
            .cached_subtree_root(
                roots.version,
                HjmtTreeId::BucketTerminal(path.definition_id, path.serial_id, bucket_id),
                roots,
            )?
            .ok_or_else(|| {
                SettlementStoreError::Jmt("missing hjmt bucket root leaf".to_string())
            })?;
        self.cached_bucket_leaf(
            roots.version,
            (path.definition_id, path.serial_id, bucket_id),
            root,
        )
    }

    fn hjmt_need_def_leaf(
        &self,
        path: &SettlementPath,
    ) -> Result<DefinitionRootLeaf, SettlementStoreError> {
        self.hjmt_need_def_leaf_at(&self.hjmt_roots, &self.model, path)
    }

    fn hjmt_need_ser_leaf(
        &self,
        path: &SettlementPath,
    ) -> Result<SerialRootLeaf, SettlementStoreError> {
        self.hjmt_need_ser_leaf_at(&self.hjmt_roots, &self.model, path)
    }

    fn hjmt_need_bucket_leaf(
        &self,
        path: &SettlementPath,
    ) -> Result<BucketRootLeaf, SettlementStoreError> {
        self.hjmt_need_bucket_leaf_at(&self.hjmt_roots, &self.model, path)
    }

    fn encode_hjmt_def_proof(
        &self,
        path: &SettlementPath,
    ) -> Result<Vec<u8>, SettlementStoreError> {
        self.encode_hjmt_def_proof_at(path, self.hjmt_roots.version)
    }

    fn encode_hjmt_ser_proof(
        &self,
        path: &SettlementPath,
    ) -> Result<Vec<u8>, SettlementStoreError> {
        self.encode_hjmt_ser_proof_at(path, self.hjmt_roots.version)
    }

    fn encode_hjmt_bucket_proof(
        &self,
        path: &SettlementPath,
        bucket_leaf: BucketRootLeaf,
    ) -> Result<Vec<u8>, SettlementStoreError> {
        self.encode_hjmt_bucket_proof_at(path, bucket_leaf, self.hjmt_roots.version)
    }

    fn encode_hjmt_terminal_proof(
        &self,
        path: &SettlementPath,
        bucket_leaf: BucketRootLeaf,
    ) -> Result<Vec<u8>, SettlementStoreError> {
        self.encode_hjmt_terminal_proof_at(path, bucket_leaf, self.hjmt_roots.version)
    }

    fn encode_hjmt_def_proof_at(
        &self,
        path: &SettlementPath,
        version: u64,
    ) -> Result<Vec<u8>, SettlementStoreError> {
        self.encode_hjmt_proof(
            HjmtTreeId::Definition,
            definition_key(path.definition_id),
            version,
        )
    }

    fn encode_hjmt_ser_proof_at(
        &self,
        path: &SettlementPath,
        version: u64,
    ) -> Result<Vec<u8>, SettlementStoreError> {
        self.encode_hjmt_proof(
            HjmtTreeId::Serial(path.definition_id),
            serial_key(path.definition_id, path.serial_id),
            version,
        )
    }

    fn encode_hjmt_bucket_proof_at(
        &self,
        path: &SettlementPath,
        bucket_leaf: BucketRootLeaf,
        version: u64,
    ) -> Result<Vec<u8>, SettlementStoreError> {
        self.encode_hjmt_proof(
            HjmtTreeId::Bucket(path.definition_id, path.serial_id),
            bucket_key_for_path(bucket_leaf.bucket_id),
            version,
        )
    }

    fn encode_hjmt_terminal_proof_at(
        &self,
        path: &SettlementPath,
        bucket_leaf: BucketRootLeaf,
        version: u64,
    ) -> Result<Vec<u8>, SettlementStoreError> {
        self.encode_hjmt_proof(
            HjmtTreeId::BucketTerminal(path.definition_id, path.serial_id, bucket_leaf.bucket_id),
            crate::settlement::keys::terminal_key(path.terminal_id()),
            version,
        )
    }

    fn encode_hjmt_proof(
        &self,
        tree_id: HjmtTreeId,
        key: jmt::KeyHash,
        version: u64,
    ) -> Result<Vec<u8>, SettlementStoreError> {
        self.encode_hjmt_proof_allow_empty(tree_id, key, version)
    }

    fn encode_hjmt_proof_allow_empty(
        &self,
        tree_id: HjmtTreeId,
        key: KeyHash,
        version: u64,
    ) -> Result<Vec<u8>, SettlementStoreError> {
        if let Some(bytes) = self.cached_proof_segment(tree_id, key.0, version)? {
            return Ok(bytes);
        }
        let bytes = self.encode_hjmt_maybe_empty_uncached(tree_id, key, version)?;
        self.store_proof_segment(tree_id, key.0, version, bytes.clone());
        Ok(bytes)
    }

    pub(super) fn encode_hjmt_maybe_empty_uncached(
        &self,
        tree_id: HjmtTreeId,
        key: KeyHash,
        version: u64,
    ) -> Result<Vec<u8>, SettlementStoreError> {
        let codec = BincodeCodec;
        let proof = match self.hjmt_store.get_proof(tree_id, key, version) {
            Ok(proof) => proof,
            Err(SettlementStoreError::EmptyTree) => self.encode_empty_hjmt_proof(key, version)?,
            Err(err) => return Err(err),
        };
        Ok(codec.serialize(&proof)?)
    }

    fn encode_empty_hjmt_proof(
        &self,
        key: KeyHash,
        version: u64,
    ) -> Result<jmt::proof::SparseMerkleProof<sha2::Sha256>, SettlementStoreError> {
        let store = MemTreeStore::new();
        let jmt = Sha256Jmt::new(&store);
        for next_version in 0..=version {
            let (_root, batch) = jmt
                .put_value_set(Vec::new(), next_version)
                .map_err(map_jmt_err)?;
            apply_batch(&store, batch)?;
        }
        let (_value, proof) = jmt.get_with_proof(key, version).map_err(map_jmt_err)?;
        Ok(proof)
    }

    pub(super) fn hjmt_empty_tree_root(
        &self,
        version: u64,
    ) -> Result<[u8; 32], SettlementStoreError> {
        let store = MemTreeStore::new();
        let jmt = Sha256Jmt::new(&store);
        let mut root = None;
        for next_version in 0..=version {
            let (next_root, batch) = jmt
                .put_value_set(Vec::new(), next_version)
                .map_err(map_jmt_err)?;
            apply_batch(&store, batch)?;
            root = Some(next_root.0);
        }
        root.ok_or_else(|| SettlementStoreError::Jmt("missing empty-tree root".to_string()))
    }
}
