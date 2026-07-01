use std::collections::BTreeSet;

use jmt::{
    proof::{SparseMerkleProof, INTERNAL_DOMAIN_SEPARATOR, LEAF_DOMAIN_SEPARATOR},
    KeyHash, RootHash, ValueHash,
};
use sha2::{Digest, Sha256};
use z00z_crypto::{expert::hash_domain, hash_zk::hash_zk};
use z00z_utils::codec::{BincodeCodec, Codec};

use super::keys::{definition_key, serial_key, terminal_key};
use super::proof::{
    hjmt_default_child_commitment, hjmt_default_value_commitment, ProofChkErr,
    HJMT_DEFAULT_COMMITMENT_VERSION,
};
use super::proof_batch::{
    batch_proof_accept_transcript_v1, batch_proof_transcript_domain_v1, BatchPathEntryV1,
    BatchProofBlobV1, BatchProofHeaderV1, CheckpointPublicationProofV1, CheckpointPublicationV1,
    DeletionFactV1, LeafFamilyTagV1, NodeDomainTagV1, OpeningKindTagV1, PriorProofContextV1,
    PublicationHandoffRowV1, PublicationRouteSnapshotV1, RootGenerationTagV1, ShardRootLeafV1,
    SiblingSideTagV1, WitnessNodeV1,
};
use super::{
    BucketId, BucketPolicy, BucketRootLeaf, DefinitionId, DefinitionRootLeaf, SerialId,
    SerialRootLeaf, SettlementLeaf, SettlementLeafFamily, SettlementPath, SettlementStateRoot,
};

const JMT_PLACEHOLDER_HASH: [u8; 32] = *b"SPARSE_MERKLE_PLACEHOLDER_HASH__";
const ROOT_BIND_VER: u8 = 1;

hash_domain!(StorProofBindDom, "z00z.storage.proof.bind", 1);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct BatchPathOrdV1 {
    pub root_generation: RootGenerationTagV1,
    pub routing_generation: Option<u64>,
    pub shard_id: Option<u32>,
    pub definition_id: [u8; 32],
    pub serial_id: u32,
    pub terminal_family: u8,
    pub terminal_id: [u8; 32],
}

#[derive(Clone, Copy, Debug)]
struct RootWalkV1 {
    current_hash: [u8; 32],
    current_key: KeyHash,
    current_domain: NodeDomainTagV1,
    last_level: Option<u16>,
}

#[must_use]
pub(crate) fn path_ord_v1(header: &BatchProofHeaderV1, path: &BatchPathEntryV1) -> BatchPathOrdV1 {
    BatchPathOrdV1 {
        root_generation: header.root_generation,
        routing_generation: path.routing_generation,
        shard_id: path.shard_id,
        definition_id: path.path.definition_id.into_bytes(),
        serial_id: path.path.serial_id.get(),
        terminal_family: path.terminal_family as u8,
        terminal_id: path.path.terminal_id.into_bytes(),
    }
}

pub fn check_batch_contract_v1(batch: &BatchProofBlobV1) -> Result<(), ProofChkErr> {
    let policy = check_header(batch)?;
    check_paths(batch, policy)?;
    check_exact_usage(batch)?;
    check_openings(batch)?;
    check_witnesses(batch)?;
    bind_transcript(batch)?;
    check_atomic_roots(batch, policy)?;
    Ok(())
}

pub fn check_shard_root_leaf_v1(leaf: &ShardRootLeafV1) -> Result<(), ProofChkErr> {
    leaf.check_contract_v1()
}

pub fn check_checkpoint_publication_contract_v1(
    publication: &CheckpointPublicationV1,
) -> Result<(), ProofChkErr> {
    publication.check_contract_v1()
}

pub fn check_public_checkpoint_v1(proof: &CheckpointPublicationProofV1) -> Result<(), ProofChkErr> {
    proof.check_contract_v1()
}

pub fn check_publication_route_v1(
    publication: &CheckpointPublicationV1,
    route: &PublicationRouteSnapshotV1,
) -> Result<(), ProofChkErr> {
    // Validators and watchers must reuse this storage-owned route binding
    // contract instead of inventing a second route-table acceptance path.
    publication.check_contract_v1()?;
    route.check_contract_v1()?;
    if publication.shard_leaves.len() != route.shard_ids.len() {
        return Err(ProofChkErr::PublicationCountMix);
    }
    for (leaf, shard_id) in publication.shard_leaves.iter().zip(route.shard_ids.iter()) {
        check_route_binding_v1(
            route,
            leaf.route_table_digest,
            Some(publication.publication_checkpoint),
            Some((leaf.shard_id, leaf.routing_generation)),
        )?;
        if leaf.shard_id != *shard_id {
            return Err(ProofChkErr::PublicationCountMix);
        }
    }
    Ok(())
}

pub fn check_public_checkpoint_route_v1(
    proof: &CheckpointPublicationProofV1,
    route: &PublicationRouteSnapshotV1,
) -> Result<(), ProofChkErr> {
    proof.check_contract_v1()?;
    check_publication_route_v1(&proof.publication, route)
}

pub fn check_handoff_route_v1(
    handoff: &[PublicationHandoffRowV1],
    route: &PublicationRouteSnapshotV1,
) -> Result<(), ProofChkErr> {
    route.check_contract_v1()?;
    if handoff.len() != route.shard_ids.len() {
        return Err(ProofChkErr::PublicationCountMix);
    }
    let mut prev = None;
    let mut seen = BTreeSet::new();
    let mut checkpoints = BTreeSet::new();
    for row in handoff {
        if let Some(last) = prev {
            if row.shard_id <= last {
                return if row.shard_id == last {
                    Err(ProofChkErr::PublicationDupShard)
                } else {
                    Err(ProofChkErr::PublicationOrderMix)
                };
            }
        }
        check_route_binding_v1(
            route,
            row.route_table_digest,
            None,
            Some((row.shard_id, row.routing_generation)),
        )?;
        if !seen.insert(row.shard_id) {
            return Err(ProofChkErr::PublicationDupShard);
        }
        if !checkpoints.insert(row.checkpoint_id) {
            return Err(ProofChkErr::PublicationCheckpointMix);
        }
        prev = Some(row.shard_id);
    }
    let want = route.shard_ids.iter().copied().collect::<BTreeSet<_>>();
    if seen != want {
        return Err(ProofChkErr::PublicationCountMix);
    }
    Ok(())
}

pub fn check_route_binding_v1(
    route: &PublicationRouteSnapshotV1,
    route_table_digest: [u8; 32],
    publication_checkpoint: Option<u64>,
    shard_ctx: Option<(u32, u64)>,
) -> Result<(), ProofChkErr> {
    route.check_contract_v1()?;
    if route.route_table_digest != route_table_digest {
        return Err(ProofChkErr::PublicationRouteMix);
    }
    if publication_checkpoint.is_some_and(|checkpoint| checkpoint < route.activation_checkpoint) {
        return Err(ProofChkErr::PublicationCheckpointMix);
    }
    if let Some((shard_id, routing_generation)) = shard_ctx {
        if routing_generation != route.routing_generation {
            return Err(ProofChkErr::PublicationRouteMix);
        }
        if route.shard_ids.binary_search(&shard_id).is_err() {
            return Err(ProofChkErr::PublicationCountMix);
        }
    }
    Ok(())
}

fn check_header(batch: &BatchProofBlobV1) -> Result<BucketPolicy, ProofChkErr> {
    if batch.header.encoding_version != super::proof_batch::BATCH_PROOF_ENCODING_VERSION {
        return Err(ProofChkErr::UnsupportedBatchProofVersion);
    }
    if batch.header.transcript_domain != batch_proof_transcript_domain_v1() {
        return Err(ProofChkErr::BatchTranscriptMix);
    }
    if batch.header.root_generation != RootGenerationTagV1::RootGeneration1 {
        return Err(ProofChkErr::BatchRootGenerationMix);
    }
    if batch.header.root_generation.into_version()
        != batch.header.settlement_root.generation_version()
    {
        return Err(ProofChkErr::BatchRootGenerationMix);
    }
    if batch.header.root_bind_version != ROOT_BIND_VER {
        return Err(ProofChkErr::BatchBindVerMix);
    }
    if batch.header.root_bind != root_bind(batch.header.settlement_root, batch.header.backend_root)
    {
        return Err(ProofChkErr::BatchRootBindMix);
    }
    if !super::proof_batch::BatchProofLimits::v1().contains(batch.header.batch_limits) {
        return Err(ProofChkErr::BatchLimitMix);
    }
    if batch.header.leaf_family_set.is_empty() {
        return Err(ProofChkErr::BatchLeafFamilyMix);
    }
    let mut sorted = batch.header.leaf_family_set.clone();
    sorted.sort_unstable();
    sorted.dedup();
    if sorted != batch.header.leaf_family_set {
        return Err(ProofChkErr::BatchLeafFamilyMix);
    }
    let checkpoint = batch
        .header
        .journal_checkpoint
        .ok_or(ProofChkErr::BatchCheckpointMix)?;
    if batch.header.checkpoint_bind
        != checkpoint_bind(
            batch.header.settlement_root,
            batch.header.backend_root,
            checkpoint,
            batch.header.journal_digest,
        )
    {
        return Err(ProofChkErr::BatchCheckpointMix);
    }
    live_policy_v1(&batch.header)
}

fn live_policy_v1(header: &BatchProofHeaderV1) -> Result<BucketPolicy, ProofChkErr> {
    let policy = BucketPolicy::with_width(
        header.bucket_bits,
        header.bucket_id_width,
        header.min_bucket_count,
        header.max_target_leaf_count,
        u32::try_from(header.policy_generation).map_err(|_| ProofChkErr::BatchPolicyMix)?,
    )
    .map_err(|_| ProofChkErr::BatchPolicyMix)?;
    if header.policy_generation != u64::from(policy.compatibility_generation()) {
        return Err(ProofChkErr::BatchPolicyMix);
    }
    if header.bucket_policy_digest != policy.bucket_policy_id() {
        return Err(ProofChkErr::BatchPolicyMix);
    }
    Ok(policy)
}

fn check_paths(batch: &BatchProofBlobV1, policy: BucketPolicy) -> Result<(), ProofChkErr> {
    let mut prev_ord = None;
    let mut seen_paths = BTreeSet::new();
    for entry in &batch.path_table {
        let ord = path_ord_v1(&batch.header, entry);
        if let Some(prev) = prev_ord {
            if ord < prev {
                return Err(ProofChkErr::BatchOrderMix);
            }
        }
        prev_ord = Some(ord);
        if !seen_paths.insert(entry.path) {
            return Err(ProofChkErr::BatchDupPath);
        }
        entry.path.check().map_err(|_| ProofChkErr::BatchPathMix)?;
        if entry.terminal_family.into_live() != entry.leaf_family.into_live() {
            return Err(ProofChkErr::BatchLeafFamilyMix);
        }
        if !batch.header.leaf_family_set.contains(&entry.leaf_family) {
            return Err(ProofChkErr::BatchLeafFamilyMix);
        }
        let has_shard = entry.shard_id.is_some();
        let has_route = entry.routing_generation.is_some();
        if has_shard != has_route {
            return Err(ProofChkErr::BatchShardCtxMix);
        }
        if has_shard || has_route {
            return Err(ProofChkErr::BatchShardCtxMix);
        }
        let opening = batch
            .opening_table
            .get(entry.opening_index as usize)
            .ok_or(ProofChkErr::BatchIndexMix)?;
        let refs = batch
            .reference_table
            .get(entry.reference_index as usize)
            .ok_or(ProofChkErr::BatchIndexMix)?;
        check_path_opening(batch, entry, opening, policy)?;
        for witness_index in &refs.witness_indexes {
            let witness = batch
                .witness_dag
                .get(*witness_index as usize)
                .ok_or(ProofChkErr::BatchIndexMix)?;
            if witness.node_domain == NodeDomainTagV1::Shard
                || witness.node_domain == NodeDomainTagV1::Global
            {
                return Err(ProofChkErr::BatchWitnessDomainMix);
            }
        }
    }
    Ok(())
}

fn check_openings(batch: &BatchProofBlobV1) -> Result<(), ProofChkErr> {
    for opening in &batch.opening_table {
        match (batch.header.proof_family.into_live(), opening.opening_kind) {
            (super::proof::HjmtProofFamily::Inclusion, OpeningKindTagV1::InclusionLeaf) => {
                let payload = opening.decode_inclusion()?;
                if payload.version != 1 {
                    return Err(ProofChkErr::UnsupportedBatchProofVersion);
                }
                let leaf = payload.decode_leaf()?;
                let family =
                    LeafFamilyTagV1::from_live(super::SettlementLeafFamily::from_leaf(&leaf));
                if !batch.header.leaf_family_set.contains(&family) {
                    return Err(ProofChkErr::BatchLeafFamilyMix);
                }
            }
            (super::proof::HjmtProofFamily::NonExistence, OpeningKindTagV1::AbsenceOpening) => {
                let payload = opening.decode_nonexistence()?;
                if payload.version != 1 {
                    return Err(ProofChkErr::UnsupportedBatchProofVersion);
                }
                if payload.default_commitment_version != HJMT_DEFAULT_COMMITMENT_VERSION {
                    return Err(ProofChkErr::UnsupportedBatchProofVersion);
                }
                if payload.default_commitment != hjmt_default_value_commitment()
                    || payload.default_child_commitment != hjmt_default_child_commitment()
                {
                    return Err(ProofChkErr::BatchDefaultCommitmentMix);
                }
            }
            (super::proof::HjmtProofFamily::Deletion, OpeningKindTagV1::DeletionFact) => {
                let payload = opening.decode_deletion()?;
                if payload.version != 1
                    || payload.default_commitment_version != HJMT_DEFAULT_COMMITMENT_VERSION
                {
                    return Err(ProofChkErr::UnsupportedBatchProofVersion);
                }
                if payload.default_child_commitment != hjmt_default_child_commitment() {
                    return Err(ProofChkErr::BatchDefaultCommitmentMix);
                }
                if payload.prior_context.version != 1 {
                    return Err(ProofChkErr::UnsupportedBatchProofVersion);
                }
                if payload.prior_context.prior_settlement_root == batch.header.settlement_root {
                    return Err(ProofChkErr::PriorRootMix);
                }
            }
            _ => return Err(ProofChkErr::BatchOpeningKindMix),
        }
    }
    Ok(())
}

fn check_exact_usage(batch: &BatchProofBlobV1) -> Result<(), ProofChkErr> {
    let mut used_openings = vec![false; batch.opening_table.len()];
    let mut used_refs = vec![false; batch.reference_table.len()];
    let mut used_witnesses = vec![false; batch.witness_dag.len()];

    for entry in &batch.path_table {
        let opening_index = entry.opening_index as usize;
        if used_openings
            .get(opening_index)
            .copied()
            .ok_or(ProofChkErr::BatchIndexMix)?
        {
            return Err(ProofChkErr::BatchIndexMix);
        }
        used_openings[opening_index] = true;

        let reference_index = entry.reference_index as usize;
        *used_refs
            .get_mut(reference_index)
            .ok_or(ProofChkErr::BatchIndexMix)? = true;

        for witness_index in &batch.reference_table[reference_index].witness_indexes {
            *used_witnesses
                .get_mut(*witness_index as usize)
                .ok_or(ProofChkErr::BatchIndexMix)? = true;
        }
    }

    if used_openings.iter().any(|used| !used)
        || used_refs.iter().any(|used| !used)
        || used_witnesses.iter().any(|used| !used)
    {
        return Err(ProofChkErr::BatchIndexMix);
    }

    Ok(())
}

fn check_witnesses(batch: &BatchProofBlobV1) -> Result<(), ProofChkErr> {
    for node in &batch.witness_dag {
        if node.hash_material.len() != 1 {
            return Err(ProofChkErr::BatchHashCountMix);
        }
        if node.node_domain == NodeDomainTagV1::Shard || node.node_domain == NodeDomainTagV1::Global
        {
            return Err(ProofChkErr::BatchWitnessDomainMix);
        }
        if node.child_index > 1 {
            return Err(ProofChkErr::BatchWitnessStepMix);
        }
        if node.subtree_marker {
            return Err(ProofChkErr::BatchSubtreeMix);
        }
    }
    for refs in &batch.reference_table {
        let mut seen = BTreeSet::new();
        for index in &refs.witness_indexes {
            if !seen.insert(*index) {
                return Err(ProofChkErr::BatchIndexMix);
            }
        }
    }
    Ok(())
}

fn bind_transcript(batch: &BatchProofBlobV1) -> Result<[u8; 32], ProofChkErr> {
    let bytes = batch.encode()?;
    Ok(batch_proof_accept_transcript_v1(&bytes))
}

fn check_atomic_roots(batch: &BatchProofBlobV1, policy: BucketPolicy) -> Result<(), ProofChkErr> {
    for entry in &batch.path_table {
        let opening = batch
            .opening_table
            .get(entry.opening_index as usize)
            .ok_or(ProofChkErr::BatchIndexMix)?;
        let refs = batch
            .reference_table
            .get(entry.reference_index as usize)
            .ok_or(ProofChkErr::BatchIndexMix)?;
        let mut walk = open_root_walk(batch, entry, opening)?;
        for witness_index in &refs.witness_indexes {
            let witness = batch
                .witness_dag
                .get(*witness_index as usize)
                .ok_or(ProofChkErr::BatchIndexMix)?;
            consume_witness_step(&mut walk, witness, entry, policy)?;
        }
        while walk.current_domain < NodeDomainTagV1::Definition {
            promote_domain(&mut walk, entry, policy)?;
        }
        if walk.current_hash != batch.header.backend_root {
            return Err(ProofChkErr::BatchRootMix);
        }
    }
    Ok(())
}

fn open_root_walk(
    batch: &BatchProofBlobV1,
    entry: &BatchPathEntryV1,
    opening: &super::proof_batch::OpeningEntryV1,
) -> Result<RootWalkV1, ProofChkErr> {
    let current_hash = match batch.header.proof_family.into_live() {
        super::proof::HjmtProofFamily::Inclusion => {
            let payload = opening.decode_inclusion()?;
            let leaf = payload.decode_leaf()?;
            leaf_node_hash(terminal_key(entry.path.terminal_id()), leaf.encode()?)
        }
        super::proof::HjmtProofFamily::NonExistence | super::proof::HjmtProofFamily::Deletion => {
            JMT_PLACEHOLDER_HASH
        }
    };
    Ok(RootWalkV1 {
        current_hash,
        current_key: terminal_key(entry.path.terminal_id()),
        current_domain: NodeDomainTagV1::Terminal,
        last_level: None,
    })
}

fn consume_witness_step(
    walk: &mut RootWalkV1,
    witness: &WitnessNodeV1,
    entry: &BatchPathEntryV1,
    policy: BucketPolicy,
) -> Result<(), ProofChkErr> {
    while walk.current_domain < witness.node_domain {
        promote_domain(walk, entry, policy)?;
    }
    if walk.current_domain != witness.node_domain {
        return Err(ProofChkErr::BatchWitnessDomainMix);
    }
    if witness.tree_level > 255 {
        return Err(ProofChkErr::BatchWitnessStepMix);
    }
    if let Some(prev) = walk.last_level {
        if witness.tree_level <= prev {
            return Err(ProofChkErr::BatchWitnessStepMix);
        }
    }
    let expected_child = key_bit_from_leaf(walk.current_key, witness.tree_level);
    if witness.child_index != expected_child {
        return Err(ProofChkErr::BatchWitnessStepMix);
    }
    let expected_side = if expected_child == 0 {
        SiblingSideTagV1::RightSibling
    } else {
        SiblingSideTagV1::LeftSibling
    };
    if witness.sibling_side != expected_side {
        return Err(ProofChkErr::BatchWitnessStepMix);
    }
    walk.current_hash = if expected_child == 0 {
        internal_node_hash(walk.current_hash, witness.hash_material[0])
    } else {
        internal_node_hash(witness.hash_material[0], walk.current_hash)
    };
    walk.last_level = Some(witness.tree_level);
    Ok(())
}

fn promote_domain(
    walk: &mut RootWalkV1,
    entry: &BatchPathEntryV1,
    policy: BucketPolicy,
) -> Result<(), ProofChkErr> {
    match walk.current_domain {
        NodeDomainTagV1::Terminal => {
            let bucket_id = policy.derive_bucket_id(entry.path);
            let leaf = BucketRootLeaf {
                definition_id: entry.path.definition_id,
                serial_id: entry.path.serial_id,
                bucket_id,
                terminal_jmt_root: walk.current_hash,
                bucket_policy_id: policy.bucket_policy_id(),
            };
            walk.current_key = bucket_key(bucket_id);
            walk.current_hash = leaf_node_hash(walk.current_key, leaf.encode());
            walk.current_domain = NodeDomainTagV1::Bucket;
            walk.last_level = None;
            Ok(())
        }
        NodeDomainTagV1::Bucket => {
            let leaf = SerialRootLeaf {
                definition_id: entry.path.definition_id,
                serial_id: entry.path.serial_id,
                serial_root: walk.current_hash,
            };
            walk.current_key = serial_key(entry.path.definition_id, entry.path.serial_id);
            walk.current_hash = leaf_node_hash(walk.current_key, leaf.encode());
            walk.current_domain = NodeDomainTagV1::Serial;
            walk.last_level = None;
            Ok(())
        }
        NodeDomainTagV1::Serial => {
            let leaf = DefinitionRootLeaf {
                definition_id: entry.path.definition_id,
                definition_root: walk.current_hash,
            };
            walk.current_key = definition_key(entry.path.definition_id);
            walk.current_hash = leaf_node_hash(walk.current_key, leaf.encode());
            walk.current_domain = NodeDomainTagV1::Definition;
            walk.last_level = None;
            Ok(())
        }
        NodeDomainTagV1::Definition | NodeDomainTagV1::Shard | NodeDomainTagV1::Global => {
            Err(ProofChkErr::BatchWitnessDomainMix)
        }
    }
}

fn check_path_opening(
    batch: &BatchProofBlobV1,
    entry: &BatchPathEntryV1,
    opening: &super::proof_batch::OpeningEntryV1,
    policy: BucketPolicy,
) -> Result<(), ProofChkErr> {
    match batch.header.proof_family.into_live() {
        super::proof::HjmtProofFamily::Inclusion => {
            let payload = opening.decode_inclusion()?;
            let leaf = payload.decode_leaf()?;
            check_leaf_binding(entry, &leaf)
        }
        super::proof::HjmtProofFamily::Deletion => {
            let payload = opening.decode_deletion()?;
            let leaf = payload.decode_deleted_leaf()?;
            check_leaf_binding(entry, &leaf)?;
            check_deletion_prior_context(entry.path, &payload, policy)
        }
        super::proof::HjmtProofFamily::NonExistence => {
            let payload = opening.decode_nonexistence()?;
            let leaf = payload.decode_marker_leaf()?;
            if SettlementLeafFamily::from_leaf(&leaf) != entry.leaf_family.into_live() {
                return Err(ProofChkErr::BatchLeafFamilyMix);
            }
            if leaf != entry.leaf_family.into_live().marker_leaf(entry.path) {
                return Err(ProofChkErr::BatchPathMix);
            }
            Ok(())
        }
    }
}

fn check_deletion_prior_context(
    path: SettlementPath,
    payload: &DeletionFactV1,
    policy: BucketPolicy,
) -> Result<(), ProofChkErr> {
    let prior = &payload.prior_context;
    check_prior_root_bind(prior)?;

    let definition_leaf = decode_definition_root_leaf(&prior.definition_root_leaf_bytes)?;
    let serial_leaf = decode_serial_root_leaf(&prior.serial_root_leaf_bytes)?;
    let bucket_leaf = decode_bucket_root_leaf(&prior.bucket_root_leaf_bytes)?;
    if definition_leaf.definition_id != path.definition_id {
        return Err(ProofChkErr::PriorDefMix);
    }
    if serial_leaf.definition_id != path.definition_id || serial_leaf.serial_id != path.serial_id {
        return Err(ProofChkErr::PriorSerMix);
    }
    let expected_bucket = policy.derive_bucket_id(path);
    if bucket_leaf.definition_id != path.definition_id
        || bucket_leaf.serial_id != path.serial_id
        || bucket_leaf.bucket_id != expected_bucket
    {
        return Err(ProofChkErr::PriorBucketMix);
    }
    if bucket_leaf.bucket_policy_id != policy.bucket_policy_id() {
        return Err(ProofChkErr::PriorBucketMix);
    }

    let definition_proof = decode_branch(&prior.definition_proof_bytes)?;
    definition_proof
        .verify_existence(
            RootHash::from(prior.prior_backend_root),
            definition_key(path.definition_id),
            prior.definition_root_leaf_bytes.clone(),
        )
        .map_err(|_| ProofChkErr::PriorDefProofMix)?;

    let serial_proof = decode_branch(&prior.serial_proof_bytes)?;
    serial_proof
        .verify_existence(
            RootHash::from(definition_leaf.definition_root),
            serial_key(path.definition_id, path.serial_id),
            prior.serial_root_leaf_bytes.clone(),
        )
        .map_err(|_| ProofChkErr::PriorSerProofMix)?;

    let bucket_proof = decode_branch(&prior.bucket_proof_bytes)?;
    bucket_proof
        .verify_existence(
            RootHash::from(serial_leaf.serial_root),
            bucket_key(bucket_leaf.bucket_id),
            prior.bucket_root_leaf_bytes.clone(),
        )
        .map_err(|_| ProofChkErr::PriorBucketProofMix)?;

    let terminal_proof = decode_branch(&prior.prior_terminal_proof_bytes)?;
    terminal_proof
        .verify_existence(
            RootHash::from(bucket_leaf.terminal_jmt_root),
            terminal_key(path.terminal_id()),
            payload.deleted_leaf_bytes.clone(),
        )
        .map_err(|_| ProofChkErr::PriorTerminalProofMix)?;

    Ok(())
}

fn check_leaf_binding(entry: &BatchPathEntryV1, leaf: &SettlementLeaf) -> Result<(), ProofChkErr> {
    if SettlementLeafFamily::from_leaf(leaf) != entry.leaf_family.into_live() {
        return Err(ProofChkErr::BatchLeafFamilyMix);
    }
    leaf.check_path(entry.path)
        .map_err(|_| ProofChkErr::BatchPathMix)?;
    if let Some(serial_id) = leaf.serial_id() {
        if serial_id != entry.path.serial_id {
            return Err(ProofChkErr::BatchPathMix);
        }
    }
    Ok(())
}

fn bucket_key(bucket_id: super::BucketId) -> KeyHash {
    KeyHash(bucket_id.into_bytes())
}

fn key_bit_from_leaf(key: KeyHash, level: u16) -> u8 {
    let bit_index = 255usize.saturating_sub(level as usize);
    let byte = key.0[bit_index / 8];
    let shift = 7 - (bit_index % 8);
    u8::from(((byte >> shift) & 1) != 0)
}

fn leaf_node_hash(key: KeyHash, value_bytes: Vec<u8>) -> [u8; 32] {
    leaf_node_hash_bytes(key, &value_bytes)
}

fn leaf_node_hash_bytes(key: KeyHash, value_bytes: &[u8]) -> [u8; 32] {
    let value_hash = ValueHash::with::<Sha256>(value_bytes);
    let mut hasher = Sha256::new();
    hasher.update(LEAF_DOMAIN_SEPARATOR);
    hasher.update(key.0);
    hasher.update(value_hash.0);
    hasher.finalize().into()
}

fn internal_node_hash(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(INTERNAL_DOMAIN_SEPARATOR);
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

fn check_prior_root_bind(prior: &PriorProofContextV1) -> Result<(), ProofChkErr> {
    if prior.prior_root_bind_version != ROOT_BIND_VER {
        return Err(ProofChkErr::BindVerMix);
    }
    if prior.prior_root_bind != root_bind(prior.prior_settlement_root, prior.prior_backend_root) {
        return Err(ProofChkErr::PriorRootMix);
    }
    if prior.prior_checkpoint_bind
        != checkpoint_bind(
            prior.prior_settlement_root,
            prior.prior_backend_root,
            prior.prior_hjmt_version,
            prior.prior_journal_digest,
        )
    {
        return Err(ProofChkErr::JournalCheckpointMix);
    }
    Ok(())
}

fn root_bind(root: SettlementStateRoot, backend_root: [u8; 32]) -> [u8; 32] {
    let generation = [root.generation_version()];
    let root_bytes = root.into_bytes();
    hash_zk::<StorProofBindDom>(
        "proof_root_bind_v1",
        &[&generation, &root_bytes, &backend_root],
    )
}

fn checkpoint_bind(
    root: SettlementStateRoot,
    backend_root: [u8; 32],
    prior_hjmt_version: u64,
    journal_digest: [u8; 32],
) -> [u8; 32] {
    let generation = [root.generation_version()];
    let root_bytes = root.into_bytes();
    let version = prior_hjmt_version.to_be_bytes();
    hash_zk::<StorProofBindDom>(
        "proof_hjmt_checkpoint_bind_v1",
        &[
            &generation,
            &root_bytes,
            &backend_root,
            &version,
            &journal_digest,
        ],
    )
}

fn decode_branch(bytes: &[u8]) -> Result<SparseMerkleProof<Sha256>, ProofChkErr> {
    BincodeCodec.deserialize(bytes).map_err(ProofChkErr::Codec)
}

fn decode_definition_root_leaf(bytes: &[u8]) -> Result<DefinitionRootLeaf, ProofChkErr> {
    if bytes.len() != 64 {
        return Err(ProofChkErr::PriorDefMix);
    }
    let mut definition_id = [0u8; 32];
    definition_id.copy_from_slice(&bytes[..32]);
    let mut definition_root = [0u8; 32];
    definition_root.copy_from_slice(&bytes[32..64]);
    Ok(DefinitionRootLeaf {
        definition_id: DefinitionId::new(definition_id),
        definition_root,
    })
}

fn decode_serial_root_leaf(bytes: &[u8]) -> Result<SerialRootLeaf, ProofChkErr> {
    if bytes.len() != 68 {
        return Err(ProofChkErr::PriorSerMix);
    }
    let mut definition_id = [0u8; 32];
    definition_id.copy_from_slice(&bytes[..32]);
    let mut serial_id = [0u8; 4];
    serial_id.copy_from_slice(&bytes[32..36]);
    let mut serial_root = [0u8; 32];
    serial_root.copy_from_slice(&bytes[36..68]);
    Ok(SerialRootLeaf {
        definition_id: DefinitionId::new(definition_id),
        serial_id: SerialId::new(u32::from_le_bytes(serial_id)),
        serial_root,
    })
}

fn decode_bucket_root_leaf(bytes: &[u8]) -> Result<BucketRootLeaf, ProofChkErr> {
    if bytes.len() != 132 {
        return Err(ProofChkErr::PriorBucketMix);
    }
    let mut definition_id = [0u8; 32];
    definition_id.copy_from_slice(&bytes[..32]);
    let mut serial_id = [0u8; 4];
    serial_id.copy_from_slice(&bytes[32..36]);
    let mut bucket_id = [0u8; 32];
    bucket_id.copy_from_slice(&bytes[36..68]);
    let mut terminal_jmt_root = [0u8; 32];
    terminal_jmt_root.copy_from_slice(&bytes[68..100]);
    let mut bucket_policy_id = [0u8; 32];
    bucket_policy_id.copy_from_slice(&bytes[100..132]);
    Ok(BucketRootLeaf {
        definition_id: DefinitionId::new(definition_id),
        serial_id: SerialId::new(u32::from_le_bytes(serial_id)),
        bucket_id: BucketId::new(bucket_id),
        terminal_jmt_root,
        bucket_policy_id,
    })
}
