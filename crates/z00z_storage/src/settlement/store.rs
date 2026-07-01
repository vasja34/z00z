use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

use crate::backend::{
    memory::MemTreeStore,
    redb::StoragePlane,
    roots::{HjmtRoots, TreeRoots},
};
use sha2::{Digest, Sha256};

#[cfg(test)]
use std::sync::{Mutex, OnceLock};

use jmt::{RootHash, Version};

#[allow(unused_imports)]
use super::tree_id::{PathIndexRec, TreeId, TreeRootRef};
use super::{
    hjmt_cache::ForestCache,
    hjmt_config::{bucket_policy_from_env, env_opt, SettlementBackendMode},
    hjmt_journal,
    hjmt_scheduler::ForestScheduler,
    hjmt_store::HjmtStore,
};
#[allow(unused_imports)]
use super::{
    model::SettlementModel, tx_plan_types::ObjectDeltaSetV1, BucketPolicy, CheckRoot,
    ClaimSourceRoot, DefinitionId, DefinitionRootLeaf, FeeActorCtx, FeeEnvelope, FeeReplayKey,
    FeeReplayRec, PolicySetCommitmentV1, PolicySetMemberV1, ProofBlob, ProofItem, ProofScanOut,
    RightActionCtx, RightLeaf, SerialId, SerialRootLeaf, SettlementListReq, SettlementLookup,
    SettlementPage, SettlementPageTok, SettlementPath, SettlementStateRoot, SnapItem, StoreItem,
    TerminalId,
};
use crate::backend::error::StoreBackendError;
pub use crate::backend::types::{
    ClaimNullRec, ClaimNullStatus, ClaimNullTx, ClaimNullifier, SettlementStoreError, StoreOp,
};
use z00z_crypto::expert::encoding::to_hex;

#[cfg(test)]
pub(crate) const TEST_HJMT_INJ_STAGE_ENV: &str = "Z00Z_STORAGE_HJMT_INJ_STAGE";

#[cfg(test)]
pub(crate) fn test_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub struct SettlementStore {
    pub(super) backend_mode: SettlementBackendMode,
    pub(super) bucket_policy: BucketPolicy,
    pub(crate) backend: StoragePlane,
    pub(crate) forest_cache: ForestCache,
    pub(super) scheduler: ForestScheduler,
    pub(super) hjmt_store: HjmtStore,
    pub(crate) hjmt_roots: HjmtRoots,
    pub(crate) flat_store: MemTreeStore,
    pub(crate) flat_version: Version,
    pub(crate) flat_root: Option<RootHash>,
    pub(crate) model: SettlementModel,
    pub(crate) tree_roots: TreeRoots,
    pub(crate) path_by_terminal_id: HashMap<TerminalId, SettlementPath>,
    pub(crate) nullifier: BTreeMap<ClaimNullifier, ClaimNullRec>,
    pub(crate) claim_null_seq: u64,
    pub(crate) fee_replays: BTreeMap<FeeReplayKey, FeeReplayRec>,
    pub(crate) fee_replay_seq: u64,
    pub(crate) settlement_root_by_ver: HashMap<Version, SettlementStateRoot>,
    pub(crate) model_by_ver: HashMap<Version, SettlementModel>,
    pub(crate) hjmt_roots_by_ver: HashMap<Version, HjmtRoots>,
    pub(crate) last_object_delta: Option<ObjectDeltaSetV1>,
    pub(crate) object_deltas_by_ver: HashMap<Version, ObjectDeltaSetV1>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettlementRouteCtx {
    batch_id: [u8; 32],
    shard_id: u32,
    routing_generation: u64,
    route_table_digest: [u8; 32],
}

impl SettlementRouteCtx {
    #[must_use]
    pub const fn new(
        batch_id: [u8; 32],
        shard_id: u32,
        routing_generation: u64,
        route_table_digest: [u8; 32],
    ) -> Self {
        Self {
            batch_id,
            shard_id,
            routing_generation,
            route_table_digest,
        }
    }

    #[must_use]
    pub const fn batch_id(self) -> [u8; 32] {
        self.batch_id
    }

    #[must_use]
    pub const fn shard_id(self) -> u32 {
        self.shard_id
    }

    #[must_use]
    pub const fn routing_generation(self) -> u64 {
        self.routing_generation
    }

    #[must_use]
    pub const fn route_table_digest(self) -> [u8; 32] {
        self.route_table_digest
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettlementExecHandoff {
    route: SettlementRouteCtx,
    ops: Vec<StoreOp>,
    txs: Vec<crate::checkpoint::CheckpointExecTx>,
}

impl SettlementExecHandoff {
    #[must_use]
    pub fn new(
        route: SettlementRouteCtx,
        ops: Vec<StoreOp>,
        txs: Vec<crate::checkpoint::CheckpointExecTx>,
    ) -> Self {
        Self { route, ops, txs }
    }

    #[must_use]
    pub const fn route(&self) -> SettlementRouteCtx {
        self.route
    }

    #[must_use]
    pub fn ops(&self) -> &[StoreOp] {
        &self.ops
    }

    #[must_use]
    pub fn txs(&self) -> &[crate::checkpoint::CheckpointExecTx] {
        &self.txs
    }

    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        SettlementRouteCtx,
        Vec<StoreOp>,
        Vec<crate::checkpoint::CheckpointExecTx>,
    ) {
        (self.route, self.ops, self.txs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeOpKind {
    Put,
    Delete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeLeafKind {
    Terminal,
    Right,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeSeen {
    pub definition: bool,
    pub serial: bool,
    pub object: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeFlowItem {
    pub tx_id: String,
    pub op_kind: ScopeOpKind,
    pub definition_id: String,
    pub serial_id: u32,
    pub terminal_id: String,
    pub leaf_family: ScopeLeafKind,
    pub first_seen: ScopeSeen,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeRootFlow {
    pub prev_root: String,
    pub post_root: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeFlow {
    pub batch_id: String,
    pub shard_id: u32,
    pub routing_generation: u64,
    pub route_table_digest: String,
    pub items: Vec<ScopeFlowItem>,
    pub root_flow: ScopeRootFlow,
}

impl ScopeFlow {
    #[must_use]
    pub fn new(
        route: SettlementRouteCtx,
        items: Vec<ScopeFlowItem>,
        prev_root: SettlementStateRoot,
        post_root: SettlementStateRoot,
    ) -> Self {
        Self {
            batch_id: to_hex(&route.batch_id()),
            shard_id: route.shard_id(),
            routing_generation: route.routing_generation(),
            route_table_digest: to_hex(&route.route_table_digest()),
            items,
            root_flow: ScopeRootFlow {
                prev_root: to_hex(prev_root.as_bytes()),
                post_root: to_hex(post_root.as_bytes()),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettlementRecoveryState {
    pub version: u64,
    pub state_root: SettlementStateRoot,
    pub root_generation: u8,
    pub proof_version: u16,
    pub bucket_policy_generation: u32,
    pub bucket_policy_id: [u8; 32],
    pub journal_lineage: [u8; 32],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route: Option<SettlementRouteCtx>,
}

impl SettlementRecoveryState {
    #[must_use]
    pub fn new(
        version: u64,
        state_root: SettlementStateRoot,
        root_generation: u8,
        proof_version: u16,
        bucket_policy_generation: u32,
        bucket_policy_id: [u8; 32],
        journal_lineage: [u8; 32],
    ) -> Self {
        Self {
            version,
            state_root,
            root_generation,
            proof_version,
            bucket_policy_generation,
            bucket_policy_id,
            journal_lineage,
            route: None,
        }
    }

    #[must_use]
    pub fn with_route(mut self, route: SettlementRouteCtx) -> Self {
        self.route = Some(route);
        self
    }

    #[must_use]
    pub fn live_policy_member_v1(&self, activation_checkpoint: u64) -> PolicySetMemberV1 {
        PolicySetMemberV1::new(
            u64::from(self.bucket_policy_generation),
            self.bucket_policy_id,
            activation_checkpoint,
            None,
        )
    }

    #[must_use]
    pub fn live_policy_set_v1(&self, activation_checkpoint: u64) -> PolicySetCommitmentV1 {
        PolicySetCommitmentV1::new(vec![self.live_policy_member_v1(activation_checkpoint)])
    }
}

pub(crate) fn scope_tx_id(index: usize, proof: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"z00z.scope.tx-id.v1");
    hasher.update((index as u64).to_be_bytes());
    hasher.update((proof.len() as u64).to_be_bytes());
    hasher.update(proof);
    to_hex(&hasher.finalize())
}

/// Storage-owned semantic facade for generalized settlement operations.
///
/// This trait is the only public semantic boundary above the raw backend layer.
/// Physical backend roots and table layout stay private to storage internals.
pub trait SettlementTreeBackend {
    fn settlement_root(&self) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn settlement_root_for_version(
        &self,
        version: Version,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn get_settlement_item(
        &self,
        path: &SettlementPath,
    ) -> Result<Option<StoreItem>, SettlementStoreError>;

    fn lookup_settlement(
        &self,
        lookup: SettlementLookup,
    ) -> Result<Option<StoreItem>, SettlementStoreError>;

    fn list_settlement(
        &self,
        req: SettlementListReq,
    ) -> Result<SettlementPage, SettlementStoreError>;

    fn put_settlement_item(
        &mut self,
        item: StoreItem,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn del_settlement_item(
        &mut self,
        path: &SettlementPath,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn apply_settlement_ops(
        &mut self,
        ops: Vec<StoreOp>,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn apply_exec_handoff(
        &mut self,
        handoff: SettlementExecHandoff,
    ) -> Result<ScopeFlow, SettlementStoreError>;

    fn recovery_state(&self) -> Result<SettlementRecoveryState, SettlementStoreError>;

    fn settlement_proof_item(
        &self,
        path: &SettlementPath,
    ) -> Result<ProofItem, SettlementStoreError>;

    fn settlement_proof_blob(
        &self,
        path: &SettlementPath,
    ) -> Result<ProofBlob, SettlementStoreError>;

    fn settlement_proof_scan(
        &self,
        path: &SettlementPath,
    ) -> Result<ProofScanOut, SettlementStoreError>;

    fn settlement_inclusion_batch_v1(
        &self,
        paths: &[SettlementPath],
    ) -> Result<crate::settlement::BatchProofBlobV1, SettlementStoreError>;

    fn settlement_nonexistence_batch_v1(
        &self,
        paths: &[SettlementPath],
        leaf_family: crate::settlement::SettlementLeafFamily,
    ) -> Result<crate::settlement::BatchProofBlobV1, SettlementStoreError>;

    fn settlement_deletion_batch_v1(
        &self,
        paths: &[SettlementPath],
    ) -> Result<crate::settlement::BatchProofBlobV1, SettlementStoreError>;

    fn create_right(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn create_right_with_fee(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn transfer_right(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn transfer_right_with_fee(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn consume_right(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn consume_right_with_fee(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn revoke_right(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn revoke_right_with_fee(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn expire_right(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn challenge_right(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;

    fn challenge_right_with_fee(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError>;
}

impl SettlementStore {
    /// Create a managed local store for tests, simulations, and local benches.
    ///
    /// This constructor ignores env-driven durable root discovery, uses a
    /// managed local backend outside `#[cfg(test)]`, and never panics on
    /// startup drift. Use `try_new()` or `load()` for fallible operator-bound
    /// open or reload boundaries.
    pub fn new() -> Self {
        let mode = SettlementBackendMode::from_env_or_default().unwrap_or_default();
        let bucket_policy =
            bucket_policy_from_env().unwrap_or_else(|_| BucketPolicy::default_fixed());
        Self::build_with_policy(StoragePlane::managed_default(), mode, bucket_policy)
    }

    /// Create a settlement store using the canonical fallible startup path.
    pub fn try_new() -> Result<Self, SettlementStoreError> {
        let mode = SettlementBackendMode::from_env_or_default()?;
        let bucket_policy = bucket_policy_from_env()?;
        if let Some(root) = env_opt("Z00Z_STORAGE_REDB_ROOT") {
            return Self::open_with_policy(PathBuf::from(root), mode, bucket_policy);
        }

        Ok(Self::build_with_policy(
            StoragePlane::default(),
            mode,
            bucket_policy,
        ))
    }

    #[cfg(not(test))]
    pub(crate) fn transient_hjmt() -> Result<Self, SettlementStoreError> {
        Ok(Self::build_with_policy(
            StoragePlane::off(),
            SettlementBackendMode::Hjmt,
            bucket_policy_from_env()?,
        ))
    }

    #[cfg(test)]
    pub(crate) fn test_hjmt_store() -> Self {
        Self::build_with_policy(
            StoragePlane::off(),
            SettlementBackendMode::Hjmt,
            BucketPolicy::default_fixed(),
        )
    }

    pub fn load(root: impl Into<PathBuf>) -> Result<Self, SettlementStoreError> {
        let mode = SettlementBackendMode::from_env_or_default()?;
        let bucket_policy = bucket_policy_from_env()?;
        Self::open_with_policy(root, mode, bucket_policy)
    }

    #[cfg(all(test, feature = "test-params-fast"))]
    pub(crate) fn load_with_backend_mode(
        root: impl Into<PathBuf>,
        mode: SettlementBackendMode,
    ) -> Result<Self, SettlementStoreError> {
        Self::open_with_policy(root, mode, BucketPolicy::default_fixed())
    }

    fn open_with_policy(
        root: impl Into<PathBuf>,
        mode: SettlementBackendMode,
        bucket_policy: BucketPolicy,
    ) -> Result<Self, SettlementStoreError> {
        let backend = StoragePlane::new(root.into());
        let mut store = Self::build_with_policy(StoragePlane::off(), mode, bucket_policy);
        crate::backend::JournalBackend::recover_journal(&backend)?;
        if let Some(state) = backend.load_state()? {
            store.hjmt_rehydrate(state)?;
        }
        store.backend = backend;
        Ok(store)
    }

    pub(super) fn build_with_policy(
        backend: StoragePlane,
        backend_mode: SettlementBackendMode,
        bucket_policy: BucketPolicy,
    ) -> Self {
        Self {
            backend_mode,
            bucket_policy,
            backend,
            forest_cache: ForestCache::new(),
            scheduler: ForestScheduler::new(),
            hjmt_store: HjmtStore::new(),
            hjmt_roots: HjmtRoots::new(),
            flat_store: MemTreeStore::new(),
            flat_version: 0,
            flat_root: None,
            model: SettlementModel::new(),
            tree_roots: TreeRoots::default(),
            path_by_terminal_id: HashMap::new(),
            nullifier: BTreeMap::new(),
            claim_null_seq: 0,
            fee_replays: BTreeMap::new(),
            fee_replay_seq: 0,
            settlement_root_by_ver: HashMap::new(),
            model_by_ver: HashMap::new(),
            hjmt_roots_by_ver: HashMap::new(),
            last_object_delta: None,
            object_deltas_by_ver: HashMap::new(),
        }
    }

    #[must_use]
    pub fn backend_name(&self) -> &'static str {
        self.backend_mode.name()
    }

    #[must_use]
    pub const fn bucket_policy(&self) -> BucketPolicy {
        self.bucket_policy
    }

    pub(crate) fn require_hjmt_mode(&self) -> Result<(), SettlementStoreError> {
        let _ = self;
        Ok(())
    }

    pub fn settlement_root(&self) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        Ok(self.hjmt_roots.settlement_root())
    }

    pub fn settlement_root_for_version(
        &self,
        version: Version,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        if let Some(root) = self.settlement_root_by_ver.get(&version).copied() {
            return Ok(root);
        }

        let Some((_, roots)) = self.hjmt_history_at(version)? else {
            return Err(SettlementStoreError::Backend(
                "missing settlement root version".to_string(),
            ));
        };
        Ok(roots.settlement_root())
    }

    pub(crate) fn hjmt_history_at(
        &self,
        version: Version,
    ) -> Result<Option<(SettlementModel, HjmtRoots)>, SettlementStoreError> {
        if let (Some(model), Some(roots)) = (
            self.model_by_ver.get(&version),
            self.hjmt_roots_by_ver.get(&version),
        ) {
            return Ok(Some((model.clone(), roots.clone())));
        }
        let Some(store) = self.hjmt_store_at(version)? else {
            return Ok(None);
        };
        Ok(Some((store.model, store.hjmt_roots)))
    }

    pub(crate) fn hjmt_store_at(
        &self,
        version: Version,
    ) -> Result<Option<Self>, SettlementStoreError> {
        let Some(state) = self
            .backend
            .load_hjmt_state_at(version)
            .map_err(|err| SettlementStoreError::Backend(err.to_string()))?
        else {
            return Ok(None);
        };

        let mut store = Self::build_with_policy(
            StoragePlane::off(),
            SettlementBackendMode::Hjmt,
            self.bucket_policy(),
        );
        store.hjmt_rehydrate(state)?;
        Ok(Some(store))
    }

    pub fn apply_settlement_ops(
        &mut self,
        ops: Vec<StoreOp>,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_apply_settlement_ops(ops)
    }

    pub fn apply_exec_handoff(
        &mut self,
        handoff: SettlementExecHandoff,
    ) -> Result<ScopeFlow, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_apply_exec_handoff(handoff)
    }

    pub fn recovery_state(&self) -> Result<SettlementRecoveryState, SettlementStoreError> {
        self.require_hjmt_mode()?;

        let version = self.hjmt_roots.version;
        let state_root = self.settlement_root()?;
        if version == 0 {
            return Ok(SettlementRecoveryState::new(
                0,
                state_root,
                hjmt_journal::HJMT_JOURNAL_ROOT_GENERATION,
                hjmt_journal::HJMT_JOURNAL_PROOF_VERSION,
                self.bucket_policy().compatibility_generation(),
                self.bucket_policy().bucket_policy_id(),
                [0u8; 32],
            ));
        }

        let state = self.backend.load_hjmt_state_at(version)?.ok_or_else(|| {
            SettlementStoreError::Backend(
                "missing persisted hjmt recovery state for the active version".to_string(),
            )
        })?;
        let journal = state.hjmt_journal.ok_or_else(|| {
            SettlementStoreError::Backend(
                "missing persisted hjmt journal for the active version".to_string(),
            )
        })?;

        let mut recovery = SettlementRecoveryState::new(
            version,
            state_root,
            journal.root_generation,
            journal.proof_version,
            self.bucket_policy().compatibility_generation(),
            journal.bucket_policy_id,
            hjmt_journal::hjmt_journal_digest(&journal),
        );
        if let Some(route) = journal.route {
            recovery = recovery.with_route(route);
        }

        Ok(recovery)
    }
}

impl SettlementTreeBackend for SettlementStore {
    fn settlement_root(&self) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::settlement_root(self)
    }

    fn settlement_root_for_version(
        &self,
        version: Version,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::settlement_root_for_version(self, version)
    }

    fn get_settlement_item(
        &self,
        path: &SettlementPath,
    ) -> Result<Option<StoreItem>, SettlementStoreError> {
        SettlementStore::get_settlement_item(self, path)
    }

    fn lookup_settlement(
        &self,
        lookup: SettlementLookup,
    ) -> Result<Option<StoreItem>, SettlementStoreError> {
        SettlementStore::lookup_settlement(self, lookup)
    }

    fn list_settlement(
        &self,
        req: SettlementListReq,
    ) -> Result<SettlementPage, SettlementStoreError> {
        SettlementStore::list_settlement(self, req)
    }

    fn put_settlement_item(
        &mut self,
        item: StoreItem,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::put_settlement_item(self, item)
    }

    fn del_settlement_item(
        &mut self,
        path: &SettlementPath,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::del_settlement_item(self, path)
    }

    fn apply_settlement_ops(
        &mut self,
        ops: Vec<StoreOp>,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::apply_settlement_ops(self, ops)
    }

    fn apply_exec_handoff(
        &mut self,
        handoff: SettlementExecHandoff,
    ) -> Result<ScopeFlow, SettlementStoreError> {
        SettlementStore::apply_exec_handoff(self, handoff)
    }

    fn recovery_state(&self) -> Result<SettlementRecoveryState, SettlementStoreError> {
        SettlementStore::recovery_state(self)
    }

    fn settlement_proof_item(
        &self,
        path: &SettlementPath,
    ) -> Result<ProofItem, SettlementStoreError> {
        SettlementStore::settlement_proof_item(self, path)
    }

    fn settlement_proof_blob(
        &self,
        path: &SettlementPath,
    ) -> Result<ProofBlob, SettlementStoreError> {
        SettlementStore::settlement_proof_blob(self, path)
    }

    fn settlement_proof_scan(
        &self,
        path: &SettlementPath,
    ) -> Result<ProofScanOut, SettlementStoreError> {
        SettlementStore::settlement_proof_scan(self, path)
    }

    fn settlement_inclusion_batch_v1(
        &self,
        paths: &[SettlementPath],
    ) -> Result<crate::settlement::BatchProofBlobV1, SettlementStoreError> {
        SettlementStore::settlement_inclusion_batch_v1(self, paths)
    }

    fn settlement_nonexistence_batch_v1(
        &self,
        paths: &[SettlementPath],
        leaf_family: crate::settlement::SettlementLeafFamily,
    ) -> Result<crate::settlement::BatchProofBlobV1, SettlementStoreError> {
        SettlementStore::settlement_nonexistence_batch_v1(self, paths, leaf_family)
    }

    fn settlement_deletion_batch_v1(
        &self,
        paths: &[SettlementPath],
    ) -> Result<crate::settlement::BatchProofBlobV1, SettlementStoreError> {
        SettlementStore::settlement_deletion_batch_v1(self, paths)
    }

    fn create_right(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::create_right(self, path, leaf, ctx)
    }

    fn create_right_with_fee(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::create_right_with_fee(self, path, leaf, ctx, envelope, actor)
    }

    fn transfer_right(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::transfer_right(self, path, leaf, ctx)
    }

    fn transfer_right_with_fee(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::transfer_right_with_fee(self, path, leaf, ctx, envelope, actor)
    }

    fn consume_right(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::consume_right(self, path, ctx)
    }

    fn consume_right_with_fee(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::consume_right_with_fee(self, path, ctx, envelope, actor)
    }

    fn revoke_right(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::revoke_right(self, path, ctx)
    }

    fn revoke_right_with_fee(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::revoke_right_with_fee(self, path, ctx, envelope, actor)
    }

    fn expire_right(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::expire_right(self, path, ctx)
    }

    fn challenge_right(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::challenge_right(self, path, leaf, ctx)
    }

    fn challenge_right_with_fee(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        SettlementStore::challenge_right_with_fee(self, path, leaf, ctx, envelope, actor)
    }
}

impl Default for SettlementStore {
    fn default() -> Self {
        Self::new()
    }
}

impl From<StoreBackendError> for SettlementStoreError {
    fn from(err: StoreBackendError) -> Self {
        match err {
            StoreBackendError::UnsupportedGeneration(message) => {
                Self::UnsupportedGeneration(message)
            }
            other => Self::Backend(other.to_string()),
        }
    }
}

pub(super) fn next_ver(version: Version) -> Version {
    version.saturating_add(1)
}
