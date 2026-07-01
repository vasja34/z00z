use std::{
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
};

use jmt::{KeyHash, ValueHash};
use sha2::{Digest, Sha256};
use z00z_utils::codec::{BincodeCodec, Codec};

use super::{SettlementStore, SettlementStoreError};
use crate::settlement::keys::{definition_key, serial_key};
use crate::settlement::proof_batch_verify::path_ord_v1;
use crate::settlement::{
    BatchPathEntryV1, BatchProofBlobV1, BatchProofFamilyTagV1, BatchProofHeaderV1, DeletionFactV1,
    HjmtProofFamily, InclusionOpeningV1, LeafFamilyTagV1, NodeDomainTagV1, NonExistenceOpeningV1,
    OpeningEntryV1, PathWitnessRefV1, ProofBlob, SettlementLeafFamily, SettlementPath,
    SiblingSideTagV1, TerminalFamilyTagV1, WitnessNodeV1,
};

const LEAF_DOMAIN_SEPARATOR: &[u8] = b"JMT::LeafNode";
const INTERNAL_DOMAIN_SEPARATOR: &[u8] = b"JMT::IntrnalNode";
const JMT_PLACEHOLDER_HASH: [u8; 32] = *b"SPARSE_MERKLE_PLACEHOLDER_HASH__";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ReuseKey {
    root_generation: u8,
    proof_family: BatchProofFamilyTagV1,
    bucket_policy_digest: [u8; 32],
    routing_generation: Option<u64>,
    shard_id: Option<u32>,
    witness_bytes: Vec<u8>,
}

#[derive(Clone)]
struct BatchInput {
    blob: ProofBlob,
    opening: OpeningEntryV1,
    path: SettlementPath,
    leaf_family: LeafFamilyTagV1,
    terminal_family: TerminalFamilyTagV1,
}

#[derive(Clone, Debug, serde::Deserialize)]
enum BranchNodeWire {
    Null,
    Internal(BranchInternalWire),
    Leaf(BranchLeafWire),
}

impl BranchNodeWire {
    fn hash(&self) -> [u8; 32] {
        match self {
            Self::Null => JMT_PLACEHOLDER_HASH,
            Self::Internal(node) => node.hash(),
            Self::Leaf(node) => node.hash(),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
struct BranchInternalWire {
    left_child: [u8; 32],
    right_child: [u8; 32],
}

impl BranchInternalWire {
    fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(INTERNAL_DOMAIN_SEPARATOR);
        hasher.update(self.left_child);
        hasher.update(self.right_child);
        hasher.finalize().into()
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
struct BranchLeafWire {
    key_hash: KeyHash,
    value_hash: ValueHash,
}

impl BranchLeafWire {
    fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(LEAF_DOMAIN_SEPARATOR);
        hasher.update(self.key_hash.0);
        hasher.update(self.value_hash.0);
        hasher.finalize().into()
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
struct BranchProofWire {
    leaf: Option<BranchLeafWire>,
    siblings: Vec<BranchNodeWire>,
    phantom_hasher: PhantomData<()>,
}

impl SettlementStore {
    pub fn settlement_inclusion_batch_v1(
        &self,
        paths: &[SettlementPath],
    ) -> Result<BatchProofBlobV1, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.build_batch_v1(HjmtProofFamily::Inclusion, paths, None)
    }

    pub fn settlement_nonexistence_batch_v1(
        &self,
        paths: &[SettlementPath],
        leaf_family: SettlementLeafFamily,
    ) -> Result<BatchProofBlobV1, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.build_batch_v1(HjmtProofFamily::NonExistence, paths, Some(leaf_family))
    }

    pub fn settlement_deletion_batch_v1(
        &self,
        paths: &[SettlementPath],
    ) -> Result<BatchProofBlobV1, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.build_batch_v1(HjmtProofFamily::Deletion, paths, None)
    }

    fn build_batch_v1(
        &self,
        family: HjmtProofFamily,
        paths: &[SettlementPath],
        nonexistence_family: Option<SettlementLeafFamily>,
    ) -> Result<BatchProofBlobV1, SettlementStoreError> {
        if paths.is_empty() {
            return Err(SettlementStoreError::Jmt(
                "batch proof v1 requires at least one path".to_string(),
            ));
        }

        let settlement_root = self.hjmt_roots.settlement_root();
        let policy = self.bucket_policy();
        let checkpoint = Some(self.hjmt_roots.version);
        let family_tag = BatchProofFamilyTagV1::from_live(family);

        let mut inputs = paths
            .iter()
            .copied()
            .map(|path| self.load_batch_input(family, path, nonexistence_family))
            .collect::<Result<Vec<_>, _>>()?;

        let mut leaf_family_set = inputs
            .iter()
            .map(|input| input.leaf_family)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        leaf_family_set.sort_unstable();
        let backend_root = inputs[0].blob.backend_root();
        let journal_digest = inputs[0].blob.hjmt_journal_digest().ok_or({
            SettlementStoreError::Proof(crate::settlement::ProofChkErr::BatchCheckpointMix)
        })?;

        let header = BatchProofHeaderV1::from_policy(
            family_tag,
            settlement_root,
            backend_root,
            leaf_family_set,
            policy,
            checkpoint,
            journal_digest,
        );

        inputs.sort_by_key(|input| {
            let entry = BatchPathEntryV1 {
                path: input.path,
                terminal_family: input.terminal_family,
                leaf_family: input.leaf_family,
                shard_id: None,
                routing_generation: None,
                opening_index: 0,
                reference_index: 0,
            };
            path_ord_v1(&header, &entry)
        });

        let mut seen_paths = BTreeSet::new();
        let mut path_table = Vec::with_capacity(inputs.len());
        let mut witness_dag = Vec::new();
        let mut opening_table = Vec::with_capacity(inputs.len());
        let mut reference_table = Vec::with_capacity(inputs.len());
        let mut reuse_map = BTreeMap::<ReuseKey, u32>::new();

        for input in inputs {
            if !seen_paths.insert(input.path) {
                return Err(SettlementStoreError::Proof(
                    crate::settlement::ProofChkErr::BatchDupPath,
                ));
            }
            self.check_batch_header(
                &input.blob,
                settlement_root,
                backend_root,
                policy,
                checkpoint,
                journal_digest,
            )?;

            let opening_index = opening_table.len() as u32;
            opening_table.push(input.opening.clone());

            let witness_indexes = self.collect_witnesses(
                &input.blob,
                family_tag,
                settlement_root.generation_version(),
                policy.bucket_policy_id(),
                &mut witness_dag,
                &mut reuse_map,
            )?;
            let reference_index = reference_table.len() as u32;
            reference_table.push(PathWitnessRefV1 { witness_indexes });

            path_table.push(BatchPathEntryV1 {
                path: input.path,
                terminal_family: input.terminal_family,
                leaf_family: input.leaf_family,
                shard_id: None,
                routing_generation: None,
                opening_index,
                reference_index,
            });
        }

        let batch = BatchProofBlobV1::new(
            header,
            path_table,
            witness_dag,
            opening_table,
            reference_table,
        );
        batch
            .check_contract_v1()
            .map_err(SettlementStoreError::Proof)?;
        Ok(batch)
    }

    fn load_batch_input(
        &self,
        family: HjmtProofFamily,
        path: SettlementPath,
        nonexistence_family: Option<SettlementLeafFamily>,
    ) -> Result<BatchInput, SettlementStoreError> {
        let blob = match family {
            HjmtProofFamily::Inclusion | HjmtProofFamily::Deletion => {
                self.settlement_proof_blob(&path)?
            }
            HjmtProofFamily::NonExistence => self.settlement_nonexistence_proof_blob(
                &path,
                nonexistence_family.ok_or_else(|| {
                    SettlementStoreError::Jmt(
                        "batch nonexistence builder needs one leaf family".to_string(),
                    )
                })?,
            )?,
        };

        match family {
            HjmtProofFamily::Inclusion | HjmtProofFamily::Deletion => {
                self.validate_settlement_proof_blob(&blob)?
            }
            HjmtProofFamily::NonExistence => self.validate_settlement_nonexistence_proof_blob(
                &blob,
                nonexistence_family.expect("leaf family checked above"),
            )?,
        }

        if blob.hjmt_proof_family() != Some(family) {
            return Err(SettlementStoreError::Proof(
                crate::settlement::ProofChkErr::ProofFamilyMix,
            ));
        }

        let live_family = blob
            .hjmt_leaf_family()
            .unwrap_or_else(|| SettlementLeafFamily::from_leaf(blob.item().leaf()));
        let leaf_family = LeafFamilyTagV1::from_live(live_family);
        let terminal_family = TerminalFamilyTagV1::from_live(live_family);
        let opening = match family {
            HjmtProofFamily::Inclusion => {
                OpeningEntryV1::from_inclusion(InclusionOpeningV1::new(blob.item().leaf())?)
            }
            HjmtProofFamily::NonExistence => {
                OpeningEntryV1::from_nonexistence(NonExistenceOpeningV1::new(blob.item().leaf())?)
            }
            HjmtProofFamily::Deletion => {
                let prior = blob.hjmt_prior().ok_or_else(|| {
                    SettlementStoreError::Jmt(
                        "deletion batch proof blob missing prior proof envelope".to_string(),
                    )
                })?;
                OpeningEntryV1::from_deletion(DeletionFactV1::new(
                    blob.item().leaf(),
                    prior.to_prior_context_v1(),
                )?)
            }
        };

        Ok(BatchInput {
            blob,
            opening,
            path,
            leaf_family,
            terminal_family,
        })
    }

    fn check_batch_header(
        &self,
        blob: &ProofBlob,
        settlement_root: crate::settlement::SettlementStateRoot,
        backend_root: [u8; 32],
        policy: crate::settlement::BucketPolicy,
        checkpoint: Option<u64>,
        journal_digest: [u8; 32],
    ) -> Result<(), SettlementStoreError> {
        if blob.item().settlement_root() != settlement_root {
            return Err(SettlementStoreError::Proof(
                crate::settlement::ProofChkErr::BatchRootMix,
            ));
        }
        if blob.backend_root() != backend_root {
            return Err(SettlementStoreError::Proof(
                crate::settlement::ProofChkErr::BatchRootMix,
            ));
        }
        if blob.root_bind_ver() != 1 {
            return Err(SettlementStoreError::Proof(
                crate::settlement::ProofChkErr::BatchBindVerMix,
            ));
        }
        if blob.hjmt_bucket_policy() != Some(policy) {
            return Err(SettlementStoreError::Proof(
                crate::settlement::ProofChkErr::BatchPolicyMix,
            ));
        }
        if blob.hjmt_journal_checkpoint() != checkpoint {
            return Err(SettlementStoreError::Proof(
                crate::settlement::ProofChkErr::BatchCheckpointMix,
            ));
        }
        if blob.hjmt_journal_digest() != Some(journal_digest) {
            return Err(SettlementStoreError::Proof(
                crate::settlement::ProofChkErr::BatchCheckpointMix,
            ));
        }
        Ok(())
    }

    fn collect_witnesses(
        &self,
        blob: &ProofBlob,
        family: BatchProofFamilyTagV1,
        root_generation: u8,
        policy_digest: [u8; 32],
        witness_dag: &mut Vec<WitnessNodeV1>,
        reuse_map: &mut BTreeMap<ReuseKey, u32>,
    ) -> Result<Vec<u32>, SettlementStoreError> {
        let path = blob.item().path();
        let bucket_id = self.bucket_policy().derive_bucket_id(path);
        let mut witness_indexes = Vec::new();

        self.collect_branch_refs(
            blob.terminal_proof(),
            crate::settlement::keys::terminal_key(path.terminal_id()),
            NodeDomainTagV1::Terminal,
            family,
            root_generation,
            policy_digest,
            witness_dag,
            reuse_map,
            &mut witness_indexes,
        )?;
        if let Some(bucket_proof) = blob.hjmt_bucket_proof() {
            self.collect_branch_refs(
                bucket_proof,
                KeyHash(bucket_id.into_bytes()),
                NodeDomainTagV1::Bucket,
                family,
                root_generation,
                policy_digest,
                witness_dag,
                reuse_map,
                &mut witness_indexes,
            )?;
        }
        self.collect_branch_refs(
            blob.serial_proof(),
            serial_key(path.definition_id, path.serial_id),
            NodeDomainTagV1::Serial,
            family,
            root_generation,
            policy_digest,
            witness_dag,
            reuse_map,
            &mut witness_indexes,
        )?;
        self.collect_branch_refs(
            blob.definition_proof(),
            definition_key(path.definition_id),
            NodeDomainTagV1::Definition,
            family,
            root_generation,
            policy_digest,
            witness_dag,
            reuse_map,
            &mut witness_indexes,
        )?;

        Ok(witness_indexes)
    }

    #[allow(clippy::too_many_arguments)]
    fn collect_branch_refs(
        &self,
        proof_bytes: &[u8],
        queried_key: KeyHash,
        domain: NodeDomainTagV1,
        family: BatchProofFamilyTagV1,
        root_generation: u8,
        policy_digest: [u8; 32],
        witness_dag: &mut Vec<WitnessNodeV1>,
        reuse_map: &mut BTreeMap<ReuseKey, u32>,
        witness_indexes: &mut Vec<u32>,
    ) -> Result<(), SettlementStoreError> {
        if proof_bytes.is_empty() {
            return Ok(());
        }
        let branch: BranchProofWire = BincodeCodec.deserialize(proof_bytes)?;
        let _ = branch.phantom_hasher;
        let branch_nodes = if let Some(leaf) = branch.leaf {
            if leaf.key_hash == queried_key {
                siblings_to_nodes(&branch.siblings, queried_key, domain)?
            } else {
                leaf_absent_nodes(queried_key, leaf, &branch.siblings, domain)?
            }
        } else {
            siblings_to_nodes(&branch.siblings, queried_key, domain)?
        };

        for node in branch_nodes {
            let mut witness_bytes = Vec::new();
            node.encode(&mut witness_bytes);
            let key = ReuseKey {
                root_generation,
                proof_family: family,
                bucket_policy_digest: policy_digest,
                routing_generation: None,
                shard_id: None,
                witness_bytes,
            };
            let index = if let Some(index) = reuse_map.get(&key) {
                *index
            } else {
                let next = witness_dag.len() as u32;
                witness_dag.push(node);
                reuse_map.insert(key, next);
                next
            };
            witness_indexes.push(index);
        }

        Ok(())
    }
}

fn leaf_absent_nodes(
    queried_key: KeyHash,
    leaf: BranchLeafWire,
    siblings: &[BranchNodeWire],
    domain: NodeDomainTagV1,
) -> Result<Vec<WitnessNodeV1>, SettlementStoreError> {
    if siblings.is_empty() {
        return Err(SettlementStoreError::Jmt(
            "batch proof v1 cannot encode lone-leaf nonexistence".to_string(),
        ));
    }

    if common_prefix_bits(queried_key.0, leaf.key_hash.0) < siblings.len() {
        return Err(SettlementStoreError::Jmt(
            "batch proof v1 leaf-backed nonexistence prefix mismatch".to_string(),
        ));
    }
    let mut nodes = Vec::with_capacity(siblings.len() + 1);
    nodes.push(witness_node(
        (255 - siblings.len()) as u16,
        domain,
        queried_key,
        leaf.hash(),
    ));
    nodes.extend(siblings_to_nodes(siblings, queried_key, domain)?);
    Ok(nodes)
}

fn siblings_to_nodes(
    siblings: &[BranchNodeWire],
    queried_key: KeyHash,
    domain: NodeDomainTagV1,
) -> Result<Vec<WitnessNodeV1>, SettlementStoreError> {
    let level_start = (256usize.saturating_sub(siblings.len())) as u16;
    siblings
        .iter()
        .enumerate()
        .map(|(offset, sibling)| {
            let level = level_start.checked_add(offset as u16).ok_or_else(|| {
                SettlementStoreError::Jmt("batch proof v1 witness level overflow".to_string())
            })?;
            Ok(witness_node(level, domain, queried_key, sibling.hash()))
        })
        .collect()
}

fn witness_node(
    tree_level: u16,
    node_domain: NodeDomainTagV1,
    queried_key: KeyHash,
    sibling_hash: [u8; 32],
) -> WitnessNodeV1 {
    let child_index = key_bit(queried_key, tree_level);
    let sibling_side = if child_index == 0 {
        SiblingSideTagV1::RightSibling
    } else {
        SiblingSideTagV1::LeftSibling
    };
    WitnessNodeV1 {
        tree_level,
        node_domain,
        child_index,
        sibling_side,
        subtree_marker: false,
        hash_material: vec![sibling_hash],
    }
}

fn common_prefix_bits(left: [u8; 32], right: [u8; 32]) -> usize {
    for (index, (left_byte, right_byte)) in left.iter().zip(right.iter()).enumerate() {
        let diff = left_byte ^ right_byte;
        if diff != 0 {
            return index * 8 + diff.leading_zeros() as usize;
        }
    }
    256
}

fn key_bit(key: KeyHash, level: u16) -> u8 {
    let bit_index = 255usize.saturating_sub(level as usize);
    let byte = key.0[bit_index / 8];
    let shift = 7 - (bit_index % 8);
    u8::from(((byte >> shift) & 1) != 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settlement::{
        keys::{definition_key, serial_key},
        BucketRootLeaf, DefinitionId, DefinitionRootLeaf, NodeDomainTagV1, SerialId,
        SerialRootLeaf, SettlementLeaf, StoreItem, TerminalId,
    };

    use std::collections::{BTreeMap, BTreeSet};

    fn compare_path_seed(
        definition_id: DefinitionId,
        serial_id: SerialId,
        seed: u32,
    ) -> SettlementPath {
        let mut terminal = [0u8; 32];
        terminal[0] = (seed >> 8) as u8;
        terminal[1] = seed as u8;
        terminal[2] = definition_id.into_bytes()[0];
        terminal[3] = serial_id.get() as u8;
        terminal[4] = (seed >> 24) as u8;
        terminal[5] = (seed >> 16) as u8;
        SettlementPath::new(definition_id, serial_id, TerminalId::new(terminal))
    }

    fn compare_asset_item(path: SettlementPath) -> StoreItem {
        let mut core = z00z_core::assets::AssetLeaf::dummy_for_scan(path.serial_id.get());
        core.asset_id = path.terminal_id().into_bytes();
        let leaf = SettlementLeaf::Terminal(crate::settlement::TerminalLeaf::from(core));
        StoreItem::new(path, leaf).expect("comparison item")
    }

    fn compare_seed_paths(store: &mut SettlementStore, paths: &[SettlementPath]) {
        for path in paths {
            store
                .put_settlement_item(compare_asset_item(*path))
                .expect("seed comparison path");
        }
    }

    fn compare_scattered_paths(
        definition_mark: u8,
        serial_base: u32,
        needed: usize,
        start_seed: u32,
    ) -> Vec<SettlementPath> {
        let mut paths = (0..needed)
            .map(|idx| {
                let mark = definition_mark.wrapping_add(u8::try_from(idx % 29).expect("u8"));
                let definition_id = DefinitionId::new([mark; 32]);
                let serial_id = SerialId::new(serial_base + u32::try_from(idx).expect("u32"));
                compare_path_seed(
                    definition_id,
                    serial_id,
                    start_seed + u32::try_from(idx).expect("u32"),
                )
            })
            .collect::<Vec<_>>();
        paths.sort_unstable();
        paths
    }

    fn compare_same_bucket_companions(
        policy: crate::settlement::BucketPolicy,
        base_paths: &[SettlementPath],
        extra_per_path: usize,
        start_seed: u32,
    ) -> Vec<SettlementPath> {
        let mut seen = base_paths.iter().copied().collect::<BTreeSet<_>>();
        let mut paths = Vec::with_capacity(base_paths.len() * extra_per_path);
        let mut seed = start_seed;
        for base in base_paths {
            let target_bucket = base.bucket_id(policy);
            for _ in 0..extra_per_path {
                let selected = loop {
                    let candidate = compare_path_seed(base.definition_id, base.serial_id, seed);
                    let exhausted = seed == u32::MAX;
                    if !exhausted {
                        seed += 1;
                    }
                    if candidate == *base || seen.contains(&candidate) {
                        if exhausted {
                            break None;
                        }
                        continue;
                    }
                    if candidate.bucket_id(policy) != target_bucket {
                        if exhausted {
                            break None;
                        }
                        continue;
                    }
                    break Some(candidate);
                };
                let selected = selected.expect("missing same-bucket companion path");
                seen.insert(selected);
                paths.push(selected);
            }
        }
        paths.sort_unstable();
        paths
    }

    fn compare_missing_paths(
        store: &SettlementStore,
        present_paths: &[SettlementPath],
        start_seed: u32,
    ) -> Vec<SettlementPath> {
        let policy = store.bucket_policy();
        let mut seen = present_paths.iter().copied().collect::<BTreeSet<_>>();
        let mut paths = Vec::with_capacity(present_paths.len());
        let mut seed = start_seed;
        for (idx, base) in present_paths.iter().copied().enumerate() {
            let target_bucket = base.bucket_id(policy);
            let mut selected = None;
            loop {
                let path = compare_path_seed(
                    base.definition_id,
                    base.serial_id,
                    seed.saturating_add(u32::try_from(idx).expect("u32")),
                );
                let exhausted = seed == u32::MAX;
                if !exhausted {
                    seed += 1;
                }
                if path == base || seen.contains(&path) {
                    if exhausted {
                        break;
                    }
                    continue;
                }
                if path.bucket_id(policy) != target_bucket {
                    if exhausted {
                        break;
                    }
                    continue;
                }
                if store
                    .settlement_nonexistence_batch_v1(&[path], SettlementLeafFamily::Terminal)
                    .is_err()
                {
                    if exhausted {
                        break;
                    }
                    continue;
                }
                selected = Some(path);
                break;
            }
            let selected = selected.expect("missing comparison nonexistence path");
            seen.insert(selected);
            paths.push(selected);
        }
        paths
    }

    fn scattered_nonexistence_fixture(path_count: usize) -> (SettlementStore, Vec<SettlementPath>) {
        let mut store = SettlementStore::new();
        let present = compare_scattered_paths(0xB1, 141, path_count, 10_000);
        compare_seed_paths(&mut store, &present);
        let companions = compare_same_bucket_companions(store.bucket_policy(), &present, 2, 20_000);
        compare_seed_paths(&mut store, &companions);
        let missing = compare_missing_paths(&store, &present, 30_000);
        (store, missing)
    }

    fn internal_node_hash(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(INTERNAL_DOMAIN_SEPARATOR);
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().into()
    }

    fn bucket_key(bucket_id: crate::settlement::BucketId) -> KeyHash {
        KeyHash(bucket_id.into_bytes())
    }

    fn leaf_node_hash(key: KeyHash, value_bytes: Vec<u8>) -> [u8; 32] {
        let value_hash = ValueHash::with::<Sha256>(&value_bytes);
        let mut hasher = Sha256::new();
        hasher.update(LEAF_DOMAIN_SEPARATOR);
        hasher.update(key.0);
        hasher.update(value_hash.0);
        hasher.finalize().into()
    }

    fn promote_domain(
        current_hash: [u8; 32],
        current_domain: NodeDomainTagV1,
        path: SettlementPath,
        policy: crate::settlement::BucketPolicy,
    ) -> ([u8; 32], NodeDomainTagV1) {
        match current_domain {
            NodeDomainTagV1::Terminal => {
                let bucket_id = policy.derive_bucket_id(path);
                let leaf = BucketRootLeaf {
                    definition_id: path.definition_id,
                    serial_id: path.serial_id,
                    bucket_id,
                    terminal_jmt_root: current_hash,
                    bucket_policy_id: policy.bucket_policy_id(),
                };
                (
                    leaf_node_hash(bucket_key(bucket_id), leaf.encode()),
                    NodeDomainTagV1::Bucket,
                )
            }
            NodeDomainTagV1::Bucket => {
                let leaf = SerialRootLeaf {
                    definition_id: path.definition_id,
                    serial_id: path.serial_id,
                    serial_root: current_hash,
                };
                (
                    leaf_node_hash(
                        serial_key(path.definition_id, path.serial_id),
                        leaf.encode(),
                    ),
                    NodeDomainTagV1::Serial,
                )
            }
            NodeDomainTagV1::Serial => {
                let leaf = DefinitionRootLeaf {
                    definition_id: path.definition_id,
                    definition_root: current_hash,
                };
                (
                    leaf_node_hash(definition_key(path.definition_id), leaf.encode()),
                    NodeDomainTagV1::Definition,
                )
            }
            NodeDomainTagV1::Definition | NodeDomainTagV1::Shard | NodeDomainTagV1::Global => {
                unreachable!(
                    "promotion only runs from terminal->bucket->serial->definition, got {current_domain:?}"
                )
            }
        }
    }

    fn reconstruct_backend_root(
        path: SettlementPath,
        witness_indexes: &[u32],
        witness_dag: &[WitnessNodeV1],
        policy: crate::settlement::BucketPolicy,
    ) -> [u8; 32] {
        let mut current_hash = JMT_PLACEHOLDER_HASH;
        let mut current_key = crate::settlement::keys::terminal_key(path.terminal_id());
        let mut current_domain = NodeDomainTagV1::Terminal;
        let mut last_level = None;
        for witness_index in witness_indexes {
            let witness = &witness_dag[*witness_index as usize];
            while current_domain < witness.node_domain {
                let promoted = promote_domain(current_hash, current_domain, path, policy);
                current_hash = promoted.0;
                current_domain = promoted.1;
                current_key = match current_domain {
                    NodeDomainTagV1::Bucket => bucket_key(policy.derive_bucket_id(path)),
                    NodeDomainTagV1::Serial => serial_key(path.definition_id, path.serial_id),
                    NodeDomainTagV1::Definition => definition_key(path.definition_id),
                    NodeDomainTagV1::Terminal
                    | NodeDomainTagV1::Shard
                    | NodeDomainTagV1::Global => current_key,
                };
                last_level = None;
            }
            assert_eq!(current_domain, witness.node_domain);
            if let Some(prev) = last_level {
                assert!(witness.tree_level > prev);
            }
            let expected_child = key_bit(current_key, witness.tree_level);
            assert_eq!(expected_child, witness.child_index);
            assert_eq!(witness.hash_material.len(), 1);
            current_hash = if expected_child == 0 {
                internal_node_hash(current_hash, witness.hash_material[0])
            } else {
                internal_node_hash(witness.hash_material[0], current_hash)
            };
            last_level = Some(witness.tree_level);
        }
        while current_domain < NodeDomainTagV1::Definition {
            let promoted = promote_domain(current_hash, current_domain, path, policy);
            current_hash = promoted.0;
            current_domain = promoted.1;
        }
        current_hash
    }

    fn reconstruct_terminal_root(path: SettlementPath, witness_dag: &[WitnessNodeV1]) -> [u8; 32] {
        let mut current_hash = JMT_PLACEHOLDER_HASH;
        let current_key = crate::settlement::keys::terminal_key(path.terminal_id());
        let mut last_level = None;
        for witness in witness_dag {
            assert_eq!(witness.node_domain, NodeDomainTagV1::Terminal);
            if let Some(prev) = last_level {
                assert!(witness.tree_level > prev);
            }
            let expected_child = key_bit(current_key, witness.tree_level);
            assert_eq!(expected_child, witness.child_index);
            assert_eq!(witness.hash_material.len(), 1);
            current_hash = if expected_child == 0 {
                internal_node_hash(current_hash, witness.hash_material[0])
            } else {
                internal_node_hash(witness.hash_material[0], current_hash)
            };
            last_level = Some(witness.tree_level);
        }
        current_hash
    }

    #[test]
    fn test_scattered_nonexistence_batch_root() {
        let (store, missing) = scattered_nonexistence_fixture(2);
        let policy = store.bucket_policy();
        for path in missing {
            let blob = store
                .settlement_nonexistence_proof_blob(&path, SettlementLeafFamily::Terminal)
                .expect("single nonexistence proof");
            store
                .validate_settlement_nonexistence_proof_blob(&blob, SettlementLeafFamily::Terminal)
                .expect("single nonexistence verify");

            let mut witness_dag = Vec::new();
            let mut reuse_map = BTreeMap::new();
            let witness_indexes = store
                .collect_witnesses(
                    &blob,
                    BatchProofFamilyTagV1::NonExistence,
                    blob.item().settlement_root().generation_version(),
                    policy.bucket_policy_id(),
                    &mut witness_dag,
                    &mut reuse_map,
                )
                .expect("collect single witnesses");
            let terminal_nodes = witness_indexes
                .iter()
                .map(|index| witness_dag[*index as usize].clone())
                .take_while(|node| node.node_domain == NodeDomainTagV1::Terminal)
                .collect::<Vec<_>>();
            let terminal_root = reconstruct_terminal_root(path, &terminal_nodes);
            let branch: BranchProofWire = BincodeCodec
                .deserialize(blob.terminal_proof())
                .expect("decode terminal proof");
            let prefix_bits = branch
                .leaf
                .as_ref()
                .map(|leaf| {
                    common_prefix_bits(
                        crate::settlement::keys::terminal_key(path.terminal_id()).0,
                        leaf.key_hash.0,
                    )
                })
                .unwrap_or(256);
            assert_eq!(
                terminal_root,
                blob.hjmt_bucket_root_leaf()
                    .expect("bucket root leaf")
                    .terminal_jmt_root,
                "terminal_root path={path:?} prefix_bits={prefix_bits} siblings={} nodes={terminal_nodes:?}",
                branch.siblings.len()
            );
            let reconstructed =
                reconstruct_backend_root(path, &witness_indexes, &witness_dag, policy);
            assert_eq!(
                reconstructed,
                blob.backend_root(),
                "path={path:?} bucket={:?} witness_indexes={witness_indexes:?}",
                path.bucket_id(policy)
            );
        }
    }
}
