use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use z00z_crypto::{expert::hash_domain, hash_zk::hash_zk};
use z00z_utils::codec::{BincodeCodec, Codec};

use crate::backend::roots::HjmtBucketKey;
use crate::settlement::{
    BucketId, RootGeneration, SettlementRouteCtx, HJMT_PROOF_ENVELOPE_VERSION,
};

use super::{
    ClaimNullRec, DefinitionId, FeeReplayRec, SerialId, SettlementPath, SettlementStateRoot,
    SettlementStoreError,
};

hash_domain!(
    StorHjmtJournalDom,
    "z00z.storage.settlement.hjmt.journal.v1",
    1
);

pub(super) const HJMT_JOURNAL_ROOT_GENERATION: u8 = RootGeneration::SettlementV1.version();
pub(super) const HJMT_JOURNAL_PROOF_VERSION: u16 = HJMT_PROOF_ENVELOPE_VERSION as u16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum HjmtCommitStatus {
    Prepared,
    ChildrenCommitted,
    ParentsCommitted,
    RootPublished,
}

impl HjmtCommitStatus {
    pub(crate) fn rank(self) -> u8 {
        match self {
            Self::Prepared => 0,
            Self::ChildrenCommitted => 1,
            Self::ParentsCommitted => 2,
            Self::RootPublished => 3,
        }
    }

    fn from_rank(rank: u8) -> Result<Self, SettlementStoreError> {
        match rank {
            0 => Ok(Self::Prepared),
            1 => Ok(Self::ChildrenCommitted),
            2 => Ok(Self::ParentsCommitted),
            3 => Ok(Self::RootPublished),
            _ => Err(SettlementStoreError::Backend(
                "hjmt journal status byte is unsupported".to_string(),
            )),
        }
    }
}

// Storage-created scopes and semantic root transitions stay journaled on one
// durable path. Runtime may attach route context, but storage keeps journal and
// scope authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct HjmtCommitJournalEntry {
    pub(crate) version: u64,
    pub(crate) bucket_epoch: u64,
    pub(crate) bucket_policy_id: [u8; 32],
    pub(crate) root_generation: u8,
    pub(crate) proof_version: u16,
    pub(crate) previous_semantic_state_root: [u8; 32],
    pub(crate) next_semantic_state_root: [u8; 32],
    pub(crate) touched_definitions: Vec<DefinitionId>,
    pub(crate) touched_serials: Vec<(DefinitionId, SerialId)>,
    pub(crate) touched_buckets: Vec<HjmtBucketKey>,
    pub(crate) fee_replay_count: u64,
    pub(crate) fee_replay_digest: [u8; 32],
    pub(crate) fee_replay_digests: Vec<[u8; 32]>,
    pub(crate) child_commit_digest: [u8; 32],
    pub(crate) parent_commit_digest: [u8; 32],
    pub(crate) status: HjmtCommitStatus,
    pub(crate) route: Option<SettlementRouteCtx>,
}

impl HjmtCommitJournalEntry {
    pub(crate) fn new(
        version: u64,
        bucket_epoch: u64,
        bucket_policy_id: [u8; 32],
        previous_root: SettlementStateRoot,
        next_root: SettlementStateRoot,
        touched_buckets: &[HjmtBucketKey],
        child_digest: [u8; 32],
        parent_digest: [u8; 32],
    ) -> Self {
        let mut def_ids = BTreeSet::new();
        let mut serial_ids = BTreeSet::new();
        let mut bucket_ids = BTreeSet::new();

        for key in touched_buckets {
            def_ids.insert(key.0);
            serial_ids.insert((key.0, key.1));
            bucket_ids.insert(*key);
        }

        Self {
            version,
            bucket_epoch,
            bucket_policy_id,
            root_generation: HJMT_JOURNAL_ROOT_GENERATION,
            proof_version: HJMT_JOURNAL_PROOF_VERSION,
            previous_semantic_state_root: previous_root.into_bytes(),
            next_semantic_state_root: next_root.into_bytes(),
            touched_definitions: def_ids.into_iter().collect(),
            touched_serials: serial_ids.into_iter().collect(),
            touched_buckets: bucket_ids.into_iter().collect(),
            fee_replay_count: 0,
            fee_replay_digest: [0u8; 32],
            fee_replay_digests: Vec::new(),
            child_commit_digest: child_digest,
            parent_commit_digest: parent_digest,
            status: HjmtCommitStatus::Prepared,
            route: None,
        }
    }

    pub(crate) fn with_status(mut self, status: HjmtCommitStatus) -> Self {
        self.status = status;
        self
    }

    pub(crate) fn with_route(mut self, route: SettlementRouteCtx) -> Self {
        self.route = Some(route);
        self
    }

    pub(crate) fn seal_fee_replay_state(&mut self, rows: &[FeeReplayRec]) {
        self.fee_replay_count = hjmt_fee_replay_count(rows);
        self.fee_replay_digest = hjmt_fee_replay_digest(rows);
        self.fee_replay_digests = hjmt_fee_replay_digests(rows);
    }

    pub(crate) fn require_root_published(
        &self,
        version: u64,
        state_root: SettlementStateRoot,
    ) -> Result<(), SettlementStoreError> {
        if self.version != version {
            return Err(SettlementStoreError::Backend(
                "hjmt journal version does not match active metadata".to_string(),
            ));
        }
        if self.bucket_epoch != version {
            return Err(SettlementStoreError::Backend(
                "hjmt journal bucket epoch does not match active metadata".to_string(),
            ));
        }
        if self.root_generation != HJMT_JOURNAL_ROOT_GENERATION {
            return Err(SettlementStoreError::Backend(
                "hjmt journal root generation is unsupported".to_string(),
            ));
        }
        if self.proof_version != HJMT_JOURNAL_PROOF_VERSION {
            return Err(SettlementStoreError::Backend(
                "hjmt journal proof version is unsupported".to_string(),
            ));
        }
        if self.status != HjmtCommitStatus::RootPublished {
            return Err(SettlementStoreError::Backend(
                "hjmt journal active metadata is not root-published".to_string(),
            ));
        }
        if self.next_semantic_state_root != state_root.into_bytes() {
            return Err(SettlementStoreError::Backend(
                "hjmt journal state root does not match active metadata".to_string(),
            ));
        }
        Ok(())
    }
}

pub(crate) fn hjmt_child_digest(
    terminal_rows: &[(SettlementPath, BucketId, Vec<u8>)],
    settlement_path_rows: &[(SettlementPath, Vec<u8>)],
    claim_rows: &[ClaimNullRec],
    fee_rows: &[FeeReplayRec],
    child_root_rows: &[(Vec<u8>, [u8; 32])],
) -> Result<[u8; 32], SettlementStoreError> {
    let mut terminal_rows = terminal_rows.to_vec();
    let mut settlement_path_rows = settlement_path_rows.to_vec();
    let mut claim_rows = claim_rows.to_vec();
    let mut fee_rows = fee_rows.to_vec();
    let mut child_root_rows = child_root_rows.to_vec();
    terminal_rows.sort_by_key(|(path, bucket_id, _)| (*path, *bucket_id));
    settlement_path_rows.sort_by_key(|(path, _)| *path);
    claim_rows.sort_by_key(|row| row.nullifier);
    fee_rows.sort_by_key(|row| row.replay_key);
    child_root_rows.sort_by(|left, right| left.0.cmp(&right.0));
    digest(
        "children",
        &(
            terminal_rows,
            settlement_path_rows,
            claim_rows,
            fee_rows,
            child_root_rows,
        ),
    )
}

pub(crate) fn hjmt_parent_digest(
    root_rows: &[(Vec<u8>, [u8; 32])],
) -> Result<[u8; 32], SettlementStoreError> {
    let mut rows = root_rows.to_vec();
    rows.sort_by(|left, right| left.0.cmp(&right.0));
    digest("parents", &rows)
}

pub(crate) fn encode_journal(entry: &HjmtCommitJournalEntry) -> Vec<u8> {
    let mut out = Vec::with_capacity(220 + 16);
    out.extend_from_slice(&entry.version.to_be_bytes());
    out.extend_from_slice(&entry.bucket_epoch.to_be_bytes());
    out.extend_from_slice(&entry.bucket_policy_id);
    out.push(entry.root_generation);
    out.extend_from_slice(&entry.proof_version.to_be_bytes());
    out.extend_from_slice(&entry.previous_semantic_state_root);
    out.extend_from_slice(&entry.next_semantic_state_root);
    out.extend_from_slice(&entry.child_commit_digest);
    out.extend_from_slice(&entry.parent_commit_digest);
    out.extend_from_slice(&entry.fee_replay_count.to_be_bytes());
    out.extend_from_slice(&entry.fee_replay_digest);
    out.push(entry.status.rank());
    put_u32(&mut out, entry.touched_definitions.len());
    for definition_id in &entry.touched_definitions {
        out.extend_from_slice(definition_id.as_bytes());
    }
    put_u32(&mut out, entry.touched_serials.len());
    for (definition_id, serial_id) in &entry.touched_serials {
        out.extend_from_slice(definition_id.as_bytes());
        out.extend_from_slice(&serial_id.get().to_be_bytes());
    }
    put_u32(&mut out, entry.touched_buckets.len());
    for (definition_id, serial_id, bucket_id) in &entry.touched_buckets {
        out.extend_from_slice(definition_id.as_bytes());
        out.extend_from_slice(&serial_id.get().to_be_bytes());
        out.extend_from_slice(bucket_id.as_bytes());
    }
    put_u32(&mut out, entry.fee_replay_digests.len());
    for digest in &entry.fee_replay_digests {
        out.extend_from_slice(digest);
    }
    if let Some(route) = entry.route {
        out.extend_from_slice(&route.batch_id());
        out.extend_from_slice(&route.shard_id().to_be_bytes());
        out.extend_from_slice(&route.routing_generation().to_be_bytes());
        out.extend_from_slice(&route.route_table_digest());
    }
    out
}

pub(crate) fn hjmt_journal_digest(entry: &HjmtCommitJournalEntry) -> [u8; 32] {
    let bytes = encode_journal(entry);
    hash_zk::<StorHjmtJournalDom>("journal_entry_digest_v1", &[bytes.as_slice()])
}

pub(crate) fn hjmt_fee_replay_digests(rows: &[FeeReplayRec]) -> Vec<[u8; 32]> {
    let mut digests: Vec<_> = rows.iter().map(|row| row.replay_digest).collect();
    digests.sort();
    digests.dedup();
    digests
}

pub(super) fn hjmt_fee_replay_count(rows: &[FeeReplayRec]) -> u64 {
    u64::try_from(rows.len()).unwrap_or(u64::MAX)
}

pub(super) fn hjmt_fee_replay_digest(rows: &[FeeReplayRec]) -> [u8; 32] {
    let mut rows = rows.to_vec();
    rows.sort_by_key(|row| row.replay_key);

    let mut hasher = Sha256::new();
    hasher.update(hjmt_fee_replay_count(&rows).to_be_bytes());
    for row in rows {
        hasher.update(row.replay_key.as_bytes());
        hasher.update(row.replay_digest);
        hasher.update(row.nonce);
        hasher.update(row.transition_id);
        hasher.update(row.domain_id);
        hasher.update(row.payer_commitment);
        hasher.update(row.sponsor_commitment);
        hasher.update(row.budget_units.to_be_bytes());
        hasher.update(row.budget_commitment);
        hasher.update([u8::from(row.support_ref.is_some())]);
        hasher.update(row.support_ref.unwrap_or([0u8; 32]));
        hasher.update(row.failure_policy_id);
        hasher.update(row.expires_at.to_be_bytes());
        hasher.update(row.accepted_at_seq.to_be_bytes());
    }

    hasher.finalize().into()
}

pub(crate) fn validate_fee_replay_state(
    rows: &[FeeReplayRec],
    expect_count: u64,
    expect_digest: [u8; 32],
) -> Result<(), SettlementStoreError> {
    if hjmt_fee_replay_count(rows) != expect_count {
        return Err(SettlementStoreError::Backend(
            "hjmt fee replay count mismatch".to_string(),
        ));
    }
    if hjmt_fee_replay_digest(rows) != expect_digest {
        return Err(SettlementStoreError::Backend(
            "hjmt fee replay digest mismatch".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn decode_journal(bytes: &[u8]) -> Result<HjmtCommitJournalEntry, SettlementStoreError> {
    if bytes.len() < 220 {
        return Err(SettlementStoreError::Backend(
            "hjmt journal entry has invalid length".to_string(),
        ));
    }

    let mut version = [0u8; 8];
    version.copy_from_slice(&bytes[..8]);
    let mut bucket_epoch = [0u8; 8];
    bucket_epoch.copy_from_slice(&bytes[8..16]);
    let mut bucket_policy_id = [0u8; 32];
    bucket_policy_id.copy_from_slice(&bytes[16..48]);
    let root_generation = bytes[48];
    let mut proof_version = [0u8; 2];
    proof_version.copy_from_slice(&bytes[49..51]);
    let mut previous_root = [0u8; 32];
    previous_root.copy_from_slice(&bytes[51..83]);
    let mut next_root = [0u8; 32];
    next_root.copy_from_slice(&bytes[83..115]);
    let mut child_digest = [0u8; 32];
    child_digest.copy_from_slice(&bytes[115..147]);
    let mut parent_digest = [0u8; 32];
    parent_digest.copy_from_slice(&bytes[147..179]);
    let mut fee_replay_count = [0u8; 8];
    fee_replay_count.copy_from_slice(&bytes[179..187]);
    let mut fee_replay_digest = [0u8; 32];
    fee_replay_digest.copy_from_slice(&bytes[187..219]);

    let mut pos = 220;
    let touched_definitions = take_definitions(bytes, &mut pos)?;
    let touched_serials = take_serials(bytes, &mut pos)?;
    let touched_buckets = take_buckets(bytes, &mut pos)?;
    let fee_replay_digests = take_fee_replay_digests(bytes, &mut pos)?;
    let route = match bytes.len().saturating_sub(pos) {
        0 => None,
        76 => Some(take_route(bytes, &mut pos)?),
        _ => {
            return Err(SettlementStoreError::Backend(
                "hjmt journal entry has trailing bytes".to_string(),
            ))
        }
    };
    if pos != bytes.len() {
        return Err(SettlementStoreError::Backend(
            "hjmt journal entry has trailing bytes".to_string(),
        ));
    }

    Ok(HjmtCommitJournalEntry {
        version: u64::from_be_bytes(version),
        bucket_epoch: u64::from_be_bytes(bucket_epoch),
        bucket_policy_id,
        root_generation,
        proof_version: u16::from_be_bytes(proof_version),
        previous_semantic_state_root: previous_root,
        next_semantic_state_root: next_root,
        touched_definitions,
        touched_serials,
        touched_buckets,
        fee_replay_count: u64::from_be_bytes(fee_replay_count),
        fee_replay_digest,
        fee_replay_digests,
        child_commit_digest: child_digest,
        parent_commit_digest: parent_digest,
        status: HjmtCommitStatus::from_rank(bytes[219])?,
        route,
    })
}

fn put_u32(out: &mut Vec<u8>, len: usize) {
    debug_assert!(u32::try_from(len).is_ok());
    let len = len as u32;
    out.extend_from_slice(&len.to_be_bytes());
}

fn take_u32(bytes: &[u8], pos: &mut usize) -> Result<usize, SettlementStoreError> {
    let end = pos.saturating_add(4);
    if end > bytes.len() {
        return Err(SettlementStoreError::Backend(
            "hjmt journal entry is truncated".to_string(),
        ));
    }
    let mut raw = [0u8; 4];
    raw.copy_from_slice(&bytes[*pos..end]);
    *pos = end;
    Ok(u32::from_be_bytes(raw) as usize)
}

fn take_32(bytes: &[u8], pos: &mut usize) -> Result<[u8; 32], SettlementStoreError> {
    let end = pos.saturating_add(32);
    if end > bytes.len() {
        return Err(SettlementStoreError::Backend(
            "hjmt journal entry is truncated".to_string(),
        ));
    }
    let mut raw = [0u8; 32];
    raw.copy_from_slice(&bytes[*pos..end]);
    *pos = end;
    Ok(raw)
}

fn take_serial(bytes: &[u8], pos: &mut usize) -> Result<SerialId, SettlementStoreError> {
    let end = pos.saturating_add(4);
    if end > bytes.len() {
        return Err(SettlementStoreError::Backend(
            "hjmt journal entry is truncated".to_string(),
        ));
    }
    let mut raw = [0u8; 4];
    raw.copy_from_slice(&bytes[*pos..end]);
    *pos = end;
    Ok(SerialId::new(u32::from_be_bytes(raw)))
}

fn take_u64(bytes: &[u8], pos: &mut usize) -> Result<u64, SettlementStoreError> {
    let end = pos.saturating_add(8);
    if end > bytes.len() {
        return Err(SettlementStoreError::Backend(
            "hjmt journal entry is truncated".to_string(),
        ));
    }
    let mut raw = [0u8; 8];
    raw.copy_from_slice(&bytes[*pos..end]);
    *pos = end;
    Ok(u64::from_be_bytes(raw))
}

fn take_route(bytes: &[u8], pos: &mut usize) -> Result<SettlementRouteCtx, SettlementStoreError> {
    let batch_id = take_32(bytes, pos)?;
    let shard_id = take_u32(bytes, pos)?;
    let routing_generation = take_u64(bytes, pos)?;
    let route_table_digest = take_32(bytes, pos)?;
    Ok(SettlementRouteCtx::new(
        batch_id,
        u32::try_from(shard_id).map_err(|_| {
            SettlementStoreError::Backend("hjmt journal route shard id exceeds u32".to_string())
        })?,
        routing_generation,
        route_table_digest,
    ))
}

fn take_definitions(
    bytes: &[u8],
    pos: &mut usize,
) -> Result<Vec<DefinitionId>, SettlementStoreError> {
    let count = take_u32(bytes, pos)?;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        out.push(DefinitionId::new(take_32(bytes, pos)?));
    }
    Ok(out)
}

fn take_serials(
    bytes: &[u8],
    pos: &mut usize,
) -> Result<Vec<(DefinitionId, SerialId)>, SettlementStoreError> {
    let count = take_u32(bytes, pos)?;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        out.push((
            DefinitionId::new(take_32(bytes, pos)?),
            take_serial(bytes, pos)?,
        ));
    }
    Ok(out)
}

fn take_buckets(bytes: &[u8], pos: &mut usize) -> Result<Vec<HjmtBucketKey>, SettlementStoreError> {
    let count = take_u32(bytes, pos)?;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        out.push((
            DefinitionId::new(take_32(bytes, pos)?),
            take_serial(bytes, pos)?,
            BucketId::new(take_32(bytes, pos)?),
        ));
    }
    Ok(out)
}

fn take_fee_replay_digests(
    bytes: &[u8],
    pos: &mut usize,
) -> Result<Vec<[u8; 32]>, SettlementStoreError> {
    let count = take_u32(bytes, pos)?;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        out.push(take_32(bytes, pos)?);
    }
    Ok(out)
}

fn digest<T: Serialize>(label: &'static str, value: &T) -> Result<[u8; 32], SettlementStoreError> {
    let codec = BincodeCodec;
    let payload = codec.serialize(value)?;
    Ok(hash_zk::<StorHjmtJournalDom>(label, &[payload.as_slice()]))
}
