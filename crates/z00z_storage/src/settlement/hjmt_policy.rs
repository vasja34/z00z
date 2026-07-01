use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;
use z00z_crypto::{expert::hash_domain, hash_zk::hash_zk};

use super::{hjmt_config::SettlementBackendMode, SettlementStore, SettlementStoreError};
use crate::backend::{redb::StoragePlane, roots::HjmtBucketKey};
use crate::settlement::{
    AdaptiveBucket, BucketEpoch, BucketId, BucketOccupancyEvidence, BucketOccupancyMetric,
    BucketPolicy, DefinitionId, MergeProof, OccupancyClass, OccupancyScope, PolicyTransitionProof,
    SerialId, SettlementPath, SplitProof, StoreOp,
};

hash_domain!(
    StorAdaptivePolicyDom,
    "z00z.storage.settlement.hjmt.policy.v1",
    1
);

#[derive(Debug, Error)]
pub enum AdaptiveProofErr {
    #[error("adaptive policy store error: {0}")]
    Store(#[from] SettlementStoreError),
    #[error("adaptive bucket is missing")]
    BucketMiss,
    #[error("adaptive split is ineligible under the current bucket occupancy")]
    SplitIneligible,
    #[error("adaptive merge is ineligible under the current bucket occupancy")]
    MergeIneligible,
    #[error("adaptive proof epoch mismatch")]
    WrongEpoch,
    #[error("adaptive proof prior root mismatch")]
    WrongOldRoot,
    #[error("adaptive proof next root mismatch")]
    WrongNewRoot,
    #[error("adaptive proof carries a stale policy id")]
    StalePolicyId,
    #[error("adaptive proof next-policy drift")]
    NextPolicyDrift,
    #[error("adaptive policy transition batch is unordered")]
    UnorderedTransition,
    #[error("adaptive occupancy evidence drift")]
    OccupancyDrift,
}

struct BucketState {
    root: [u8; 32],
    count: usize,
}

impl SettlementStore {
    pub fn adaptive_bucket(
        &self,
        path: &SettlementPath,
    ) -> Result<AdaptiveBucket, AdaptiveProofErr> {
        self.require_hjmt_mode()?;
        let item = self
            .hjmt_get_settlement_item(path)?
            .ok_or(SettlementStoreError::PathMiss)?;
        self.adaptive_bucket_for_key(self.bucket_key_for_item(item.path()))
    }

    pub fn split_proof(&self, path: &SettlementPath) -> Result<SplitProof, AdaptiveProofErr> {
        self.require_hjmt_mode()?;
        let item = self
            .hjmt_get_settlement_item(path)?
            .ok_or(SettlementStoreError::PathMiss)?;
        self.split_proof_for_bucket(self.bucket_key_for_item(item.path()))
    }

    pub fn bucket_occupancy_metric(
        &self,
        path: &SettlementPath,
    ) -> Result<BucketOccupancyMetric, AdaptiveProofErr> {
        self.require_hjmt_mode()?;
        let item = self
            .hjmt_get_settlement_item(path)?
            .ok_or(SettlementStoreError::PathMiss)?;
        self.bucket_metric_for_key(self.bucket_key_for_item(item.path()))
    }

    pub fn merge_proof(
        &self,
        left: &SettlementPath,
        right: &SettlementPath,
    ) -> Result<MergeProof, AdaptiveProofErr> {
        self.require_hjmt_mode()?;
        let left_item = self
            .hjmt_get_settlement_item(left)?
            .ok_or(SettlementStoreError::PathMiss)?;
        let right_item = self
            .hjmt_get_settlement_item(right)?
            .ok_or(SettlementStoreError::PathMiss)?;
        self.merge_proof_for_pair(
            self.bucket_key_for_item(left_item.path()),
            self.bucket_key_for_item(right_item.path()),
        )
    }

    pub fn policy_transition_proof(
        &self,
        next_policy: BucketPolicy,
    ) -> Result<PolicyTransitionProof, AdaptiveProofErr> {
        self.require_hjmt_mode()?;
        self.policy_transition_proof_expected(next_policy)
    }

    pub fn validate_split_proof(&self, proof: &SplitProof) -> Result<(), AdaptiveProofErr> {
        let historical = self.historical_subject_store(proof.prior_epoch)?;
        let subject = historical.as_ref().unwrap_or(self);
        let expected = subject.split_proof_expected_by_root(proof.prior_bucket_root)?;
        validate_split(expected, proof)
    }

    pub fn validate_merge_proof(&self, proof: &MergeProof) -> Result<(), AdaptiveProofErr> {
        let historical = self.historical_subject_store(proof.prior_epoch)?;
        let subject = historical.as_ref().unwrap_or(self);
        let expected = subject
            .merge_proof_expected_by_roots(proof.left_bucket_root, proof.right_bucket_root)?;
        validate_merge(expected, proof)
    }

    pub fn validate_policy_transition_proof(
        &self,
        proof: &PolicyTransitionProof,
        next_policy: BucketPolicy,
    ) -> Result<(), AdaptiveProofErr> {
        let historical = self.historical_subject_store(proof.prior_epoch)?;
        let subject = historical.as_ref().unwrap_or(self);
        let expected = subject.policy_transition_proof_expected(next_policy)?;
        validate_transition(expected, proof, next_policy)
    }

    fn adaptive_bucket_for_key(
        &self,
        key: HjmtBucketKey,
    ) -> Result<AdaptiveBucket, AdaptiveProofErr> {
        let state = self.bucket_state(key)?;
        let epoch = BucketEpoch::new(self.hjmt_roots.version);
        Ok(AdaptiveBucket {
            definition_id: key.0,
            serial_id: key.1,
            bucket_id: key.2,
            epoch,
            bucket_policy_id: self.bucket_policy().bucket_policy_id(),
            bucket_root: state.root,
            occupancy_evidence: bucket_evidence(
                key,
                self.bucket_policy(),
                epoch,
                state.root,
                state.count,
            ),
        })
    }

    fn split_proof_expected_by_root(
        &self,
        prior_bucket_root: [u8; 32],
    ) -> Result<SplitProof, AdaptiveProofErr> {
        let key = self.bucket_key_by_root(prior_bucket_root)?;
        self.split_proof_for_bucket(key)
    }

    fn merge_proof_expected_by_roots(
        &self,
        left_bucket_root: [u8; 32],
        right_bucket_root: [u8; 32],
    ) -> Result<MergeProof, AdaptiveProofErr> {
        let left = self.bucket_key_by_root(left_bucket_root)?;
        let right = self.bucket_key_by_root(right_bucket_root)?;
        self.merge_proof_for_pair(left, right)
    }

    fn split_proof_for_bucket(&self, key: HjmtBucketKey) -> Result<SplitProof, AdaptiveProofErr> {
        let current_policy = self.bucket_policy();
        if current_policy.bucket_bits() >= 32 {
            return Err(AdaptiveProofErr::SplitIneligible);
        }
        let members = self.bucket_paths(key)?;
        if members.len() < split_threshold(current_policy) {
            return Err(AdaptiveProofErr::SplitIneligible);
        }
        let next_policy = derived_next_policy(current_policy, current_policy.bucket_bits() + 1)?;
        if let Some(proof) = self.cached_policy_split(key, next_policy.bucket_policy_id()) {
            return Ok(proof);
        }
        let proof = self.sched_one("hjmt_split_proof", self.fork_sched_view(), move |store| {
            store.split_proof_uncached_for_bucket(key)
        })?;
        self.store_policy_split(key, next_policy.bucket_policy_id(), proof);
        Ok(proof)
    }

    fn merge_proof_for_pair(
        &self,
        left: HjmtBucketKey,
        right: HjmtBucketKey,
    ) -> Result<MergeProof, AdaptiveProofErr> {
        let current_policy = self.bucket_policy();
        if current_policy.bucket_bits() <= 1 {
            return Err(AdaptiveProofErr::MergeIneligible);
        }
        let (left, right) = canonical_bucket_pair(left, right);
        if !are_merge_siblings(left.2, right.2, current_policy.bucket_bits()) {
            return Err(AdaptiveProofErr::MergeIneligible);
        }
        let left_count = self.bucket_paths(left)?.len();
        let right_count = self.bucket_paths(right)?.len();
        let combined_count = left_count.saturating_add(right_count);
        if left_count > merge_threshold(current_policy)
            || right_count > merge_threshold(current_policy)
            || combined_count >= split_threshold(current_policy)
        {
            return Err(AdaptiveProofErr::MergeIneligible);
        }
        let next_policy = derived_next_policy(current_policy, current_policy.bucket_bits() - 1)?;
        if let Some(proof) = self.cached_policy_merge(left, right, next_policy.bucket_policy_id()) {
            return Ok(proof);
        }
        let proof = self.sched_one("hjmt_merge_proof", self.fork_sched_view(), move |store| {
            store.merge_proof_uncached_for_pair(left, right)
        })?;
        self.store_policy_merge(left, right, next_policy.bucket_policy_id(), proof);
        Ok(proof)
    }

    fn policy_transition_proof_expected(
        &self,
        next_policy: BucketPolicy,
    ) -> Result<PolicyTransitionProof, AdaptiveProofErr> {
        let current_policy = self.bucket_policy();
        if next_policy.bucket_policy_id() == current_policy.bucket_policy_id() {
            return Err(AdaptiveProofErr::StalePolicyId);
        }
        if next_policy.compatibility_generation() <= current_policy.compatibility_generation() {
            return Err(AdaptiveProofErr::UnorderedTransition);
        }
        if let Some(proof) = self.cached_policy_transition(next_policy.bucket_policy_id()) {
            return Ok(proof);
        }
        let proof = self.sched_one(
            "hjmt_policy_transition",
            self.fork_sched_view(),
            move |store| store.policy_transition_proof_uncached(next_policy),
        )?;
        self.store_policy_transition(next_policy.bucket_policy_id(), proof);
        Ok(proof)
    }

    fn split_proof_uncached_for_bucket(
        &self,
        key: HjmtBucketKey,
    ) -> Result<SplitProof, AdaptiveProofErr> {
        let current_policy = self.bucket_policy();
        let members = self.bucket_paths(key)?;
        let next_policy = derived_next_policy(current_policy, current_policy.bucket_bits() + 1)?;
        let next_store = self.rebuild_under_policy(next_policy)?;
        let mut child_roots = child_bucket_roots(&next_store, next_policy, &members)?;
        if child_roots.len() != 2 {
            return Err(AdaptiveProofErr::SplitIneligible);
        }
        child_roots.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));

        let prior_epoch = BucketEpoch::new(self.hjmt_roots.version);
        let next_epoch = next_epoch(prior_epoch)?;
        let prior_root = self.settlement_root()?;
        let next_root = next_store.settlement_root()?;
        let prior_state = self.bucket_state(key)?;
        let prior_backend_root = self.hjmt_current_backend_root()?;
        let next_backend_root = next_store.hjmt_current_backend_root()?;
        let prior_digest = self.hjmt_current_journal_digest(prior_backend_root);
        let next_digest = next_store.hjmt_current_journal_digest(next_backend_root);

        Ok(SplitProof {
            prior_root,
            next_root,
            prior_epoch,
            next_epoch,
            prior_bucket_root: prior_state.root,
            left_bucket_root: child_roots[0].1,
            right_bucket_root: child_roots[1].1,
            bucket_policy_id: next_policy.bucket_policy_id(),
            occupancy_evidence: bucket_evidence(
                key,
                current_policy,
                prior_epoch,
                prior_state.root,
                prior_state.count,
            ),
            key_range_commitment: split_range_commitment(
                key.0,
                key.1,
                key.2,
                child_roots[0].0,
                child_roots[1].0,
            ),
            journal_digest: transition_digest(
                "split_transition_v1",
                &[
                    prior_root.into_bytes().to_vec(),
                    next_root.into_bytes().to_vec(),
                    prior_digest.to_vec(),
                    next_digest.to_vec(),
                    key.2.into_bytes().to_vec(),
                    child_roots[0].0.into_bytes().to_vec(),
                    child_roots[1].0.into_bytes().to_vec(),
                ],
            ),
        })
    }

    fn merge_proof_uncached_for_pair(
        &self,
        left: HjmtBucketKey,
        right: HjmtBucketKey,
    ) -> Result<MergeProof, AdaptiveProofErr> {
        let current_policy = self.bucket_policy();
        let next_policy = derived_next_policy(current_policy, current_policy.bucket_bits() - 1)?;
        let next_store = self.rebuild_under_policy(next_policy)?;
        let mut merged_members = self.bucket_paths(left)?;
        merged_members.extend(self.bucket_paths(right)?);
        let merged_bucket_id = merged_members[0].bucket_id(next_policy);
        let merged_root = next_store
            .hjmt_roots
            .terminal_roots
            .get(&(left.0, left.1, merged_bucket_id))
            .copied()
            .ok_or(AdaptiveProofErr::BucketMiss)?
            .into_bytes();
        let prior_epoch = BucketEpoch::new(self.hjmt_roots.version);
        let next_epoch = next_epoch(prior_epoch)?;
        let prior_root = self.settlement_root()?;
        let next_root = next_store.settlement_root()?;
        let left_state = self.bucket_state(left)?;
        let right_state = self.bucket_state(right)?;
        let prior_backend_root = self.hjmt_current_backend_root()?;
        let next_backend_root = next_store.hjmt_current_backend_root()?;
        let prior_digest = self.hjmt_current_journal_digest(prior_backend_root);
        let next_digest = next_store.hjmt_current_journal_digest(next_backend_root);

        Ok(MergeProof {
            prior_root,
            next_root,
            prior_epoch,
            next_epoch,
            left_bucket_root: left_state.root,
            right_bucket_root: right_state.root,
            merged_bucket_root: merged_root,
            bucket_policy_id: next_policy.bucket_policy_id(),
            left_evidence: bucket_evidence(
                left,
                current_policy,
                prior_epoch,
                left_state.root,
                left_state.count,
            ),
            right_evidence: bucket_evidence(
                right,
                current_policy,
                prior_epoch,
                right_state.root,
                right_state.count,
            ),
            pair_evidence: pair_evidence(
                left,
                right,
                merged_bucket_id,
                current_policy,
                prior_epoch,
                left_state.root,
                right_state.root,
                left_state.count.saturating_add(right_state.count),
            ),
            key_range_commitment: merge_range_commitment(
                left.0,
                left.1,
                left.2,
                right.2,
                merged_bucket_id,
            ),
            journal_digest: transition_digest(
                "merge_transition_v1",
                &[
                    prior_root.into_bytes().to_vec(),
                    next_root.into_bytes().to_vec(),
                    prior_digest.to_vec(),
                    next_digest.to_vec(),
                    left.2.into_bytes().to_vec(),
                    right.2.into_bytes().to_vec(),
                    merged_bucket_id.into_bytes().to_vec(),
                ],
            ),
        })
    }

    fn policy_transition_proof_uncached(
        &self,
        next_policy: BucketPolicy,
    ) -> Result<PolicyTransitionProof, AdaptiveProofErr> {
        let current_policy = self.bucket_policy();
        let next_store = self.rebuild_under_policy(next_policy)?;
        let terminal_paths = self.sorted_paths().into_iter().collect::<Vec<_>>();
        let terminal_commitment = terminal_set_commitment(&terminal_paths);
        let prior_root = self.settlement_root()?;
        let next_root = next_store.settlement_root()?;
        let prior_epoch = BucketEpoch::new(self.hjmt_roots.version);
        let next_epoch = next_epoch(prior_epoch)?;
        let prior_backend_root = self.hjmt_current_backend_root()?;
        let next_backend_root = next_store.hjmt_current_backend_root()?;
        let prior_digest = self.hjmt_current_journal_digest(prior_backend_root);
        let next_digest = next_store.hjmt_current_journal_digest(next_backend_root);
        let occupancy_evidence = self.transition_evidence(
            current_policy,
            next_policy,
            prior_epoch,
            prior_root.into_bytes(),
            next_root.into_bytes(),
            terminal_commitment,
        )?;

        Ok(PolicyTransitionProof {
            prior_root,
            next_root,
            prior_epoch,
            next_epoch,
            prior_policy_id: current_policy.bucket_policy_id(),
            next_policy_id: next_policy.bucket_policy_id(),
            terminal_set_commitment: terminal_commitment,
            occupancy_evidence,
            replay_digest: transition_digest(
                "policy_transition_v1",
                &[
                    prior_root.into_bytes().to_vec(),
                    next_root.into_bytes().to_vec(),
                    current_policy.bucket_policy_id().to_vec(),
                    next_policy.bucket_policy_id().to_vec(),
                    prior_digest.to_vec(),
                    next_digest.to_vec(),
                    terminal_commitment.to_vec(),
                ],
            ),
        })
    }

    fn bucket_key_for_item(&self, path: SettlementPath) -> HjmtBucketKey {
        (
            path.definition_id,
            path.serial_id,
            self.cached_bucket_id(path),
        )
    }

    pub(super) fn bucket_key_by_root(
        &self,
        root: [u8; 32],
    ) -> Result<HjmtBucketKey, AdaptiveProofErr> {
        self.hjmt_roots
            .terminal_roots
            .iter()
            .find_map(|(key, value)| (value.into_bytes() == root).then_some(*key))
            .ok_or(AdaptiveProofErr::BucketMiss)
    }

    fn bucket_state(&self, key: HjmtBucketKey) -> Result<BucketState, AdaptiveProofErr> {
        let root = self
            .hjmt_roots
            .terminal_roots
            .get(&key)
            .copied()
            .ok_or(AdaptiveProofErr::BucketMiss)?
            .into_bytes();
        let count = self.bucket_paths(key)?.len();
        Ok(BucketState { root, count })
    }

    fn bucket_metric_for_key(
        &self,
        key: HjmtBucketKey,
    ) -> Result<BucketOccupancyMetric, AdaptiveProofErr> {
        let state = self.bucket_state(key)?;
        Ok(BucketOccupancyMetric {
            definition_id: key.0,
            serial_id: key.1,
            bucket_id: key.2,
            epoch: BucketEpoch::new(self.hjmt_roots.version),
            bucket_root: state.root,
            class: occupancy_class(state.count, self.bucket_policy()),
            exact_count: u64::try_from(state.count).unwrap_or(u64::MAX),
        })
    }

    fn current_bucket_keys(&self) -> Result<Vec<HjmtBucketKey>, AdaptiveProofErr> {
        let mut keys = BTreeSet::new();
        for settlement_path in self.sorted_paths() {
            if self.hjmt_get_settlement_item(&settlement_path)?.is_none() {
                continue;
            }
            keys.insert(self.bucket_key_for_item(settlement_path));
        }
        Ok(keys.into_iter().collect())
    }

    fn bucket_paths(&self, key: HjmtBucketKey) -> Result<Vec<SettlementPath>, AdaptiveProofErr> {
        let mut paths = Vec::new();
        for settlement_path in self.sorted_paths() {
            if self.hjmt_get_settlement_item(&settlement_path)?.is_none() {
                continue;
            }
            if self.bucket_key_for_item(settlement_path) == key {
                paths.push(settlement_path);
            }
        }
        Ok(paths)
    }

    fn rebuild_under_policy(
        &self,
        policy: BucketPolicy,
    ) -> Result<SettlementStore, AdaptiveProofErr> {
        let mut rebuilt = SettlementStore::build_with_policy(
            StoragePlane::off(),
            SettlementBackendMode::Hjmt,
            policy,
        );
        rebuilt.scheduler = self.scheduler.fork_view();
        let mut ops = Vec::new();
        for path in self.sorted_paths() {
            let item = self
                .hjmt_get_settlement_item(&path)?
                .ok_or(SettlementStoreError::PathMiss)?;
            ops.push(StoreOp::Put(Box::new(item)));
        }
        if !ops.is_empty() {
            let _ = rebuilt.hjmt_apply_ops(ops)?;
        }
        Ok(rebuilt)
    }

    fn historical_subject_store(
        &self,
        epoch: BucketEpoch,
    ) -> Result<Option<SettlementStore>, AdaptiveProofErr> {
        if epoch.get() == self.hjmt_roots.version {
            return Ok(None);
        }
        self.hjmt_store_at(epoch.get())?
            .ok_or(AdaptiveProofErr::WrongEpoch)
            .map(Some)
    }

    fn transition_evidence(
        &self,
        current_policy: BucketPolicy,
        next_policy: BucketPolicy,
        epoch: BucketEpoch,
        prior_root: [u8; 32],
        next_root: [u8; 32],
        terminal_commitment: [u8; 32],
    ) -> Result<BucketOccupancyEvidence, AdaptiveProofErr> {
        let mut fields = vec![
            vec![1],
            vec![OccupancyScope::Set.tag()],
            vec![OccupancyClass::SetCommit.tag()],
            current_policy.bucket_policy_id().to_vec(),
            next_policy.bucket_policy_id().to_vec(),
            epoch.get().to_be_bytes().to_vec(),
            prior_root.to_vec(),
            next_root.to_vec(),
            terminal_commitment.to_vec(),
        ];
        for key in self.current_bucket_keys()? {
            let state = self.bucket_state(key)?;
            let evidence = bucket_evidence(key, current_policy, epoch, state.root, state.count);
            fields.push(key.0.as_bytes().to_vec());
            fields.push(key.1.get().to_be_bytes().to_vec());
            fields.push(key.2.into_bytes().to_vec());
            fields.push(vec![evidence.class.tag()]);
            fields.push(evidence.bind.to_vec());
        }
        Ok(BucketOccupancyEvidence::new(
            OccupancyScope::Set,
            OccupancyClass::SetCommit,
            transition_digest("policy_transition_occupancy_v1", &fields),
        ))
    }
}

fn validate_split(expected: SplitProof, got: &SplitProof) -> Result<(), AdaptiveProofErr> {
    if got.prior_epoch != expected.prior_epoch || got.next_epoch != expected.next_epoch {
        return Err(AdaptiveProofErr::WrongEpoch);
    }
    if got.prior_root != expected.prior_root || got.prior_bucket_root != expected.prior_bucket_root
    {
        return Err(AdaptiveProofErr::WrongOldRoot);
    }
    if got.bucket_policy_id != expected.bucket_policy_id {
        return Err(AdaptiveProofErr::StalePolicyId);
    }
    if got.occupancy_evidence != expected.occupancy_evidence {
        return Err(AdaptiveProofErr::OccupancyDrift);
    }
    if got.next_root != expected.next_root
        || got.left_bucket_root != expected.left_bucket_root
        || got.right_bucket_root != expected.right_bucket_root
        || got.key_range_commitment != expected.key_range_commitment
        || got.journal_digest != expected.journal_digest
    {
        return Err(AdaptiveProofErr::WrongNewRoot);
    }
    Ok(())
}

fn validate_merge(expected: MergeProof, got: &MergeProof) -> Result<(), AdaptiveProofErr> {
    if got.prior_epoch != expected.prior_epoch || got.next_epoch != expected.next_epoch {
        return Err(AdaptiveProofErr::WrongEpoch);
    }
    if got.prior_root != expected.prior_root
        || got.left_bucket_root != expected.left_bucket_root
        || got.right_bucket_root != expected.right_bucket_root
    {
        return Err(AdaptiveProofErr::WrongOldRoot);
    }
    if got.bucket_policy_id != expected.bucket_policy_id {
        return Err(AdaptiveProofErr::StalePolicyId);
    }
    if got.left_evidence != expected.left_evidence
        || got.right_evidence != expected.right_evidence
        || got.pair_evidence != expected.pair_evidence
    {
        return Err(AdaptiveProofErr::OccupancyDrift);
    }
    if got.next_root != expected.next_root
        || got.merged_bucket_root != expected.merged_bucket_root
        || got.key_range_commitment != expected.key_range_commitment
        || got.journal_digest != expected.journal_digest
    {
        return Err(AdaptiveProofErr::WrongNewRoot);
    }
    Ok(())
}

fn validate_transition(
    expected: PolicyTransitionProof,
    got: &PolicyTransitionProof,
    next_policy: BucketPolicy,
) -> Result<(), AdaptiveProofErr> {
    if got.prior_epoch != expected.prior_epoch || got.next_epoch != expected.next_epoch {
        return Err(AdaptiveProofErr::WrongEpoch);
    }
    if got.prior_root != expected.prior_root {
        return Err(AdaptiveProofErr::WrongOldRoot);
    }
    if got.prior_policy_id != expected.prior_policy_id {
        return Err(AdaptiveProofErr::StalePolicyId);
    }
    if got.next_policy_id != next_policy.bucket_policy_id() {
        return Err(AdaptiveProofErr::NextPolicyDrift);
    }
    if got.occupancy_evidence != expected.occupancy_evidence {
        return Err(AdaptiveProofErr::OccupancyDrift);
    }
    if got.next_root != expected.next_root
        || got.terminal_set_commitment != expected.terminal_set_commitment
        || got.replay_digest != expected.replay_digest
    {
        return Err(AdaptiveProofErr::WrongNewRoot);
    }
    Ok(())
}

fn child_bucket_roots(
    store: &SettlementStore,
    policy: BucketPolicy,
    members: &[SettlementPath],
) -> Result<Vec<(BucketId, [u8; 32])>, AdaptiveProofErr> {
    let mut buckets = BTreeMap::<BucketId, [u8; 32]>::new();
    for path in members {
        let key = (path.definition_id, path.serial_id, path.bucket_id(policy));
        let root = store
            .hjmt_roots
            .terminal_roots
            .get(&key)
            .copied()
            .ok_or(AdaptiveProofErr::BucketMiss)?
            .into_bytes();
        buckets.insert(key.2, root);
    }
    Ok(buckets.into_iter().collect())
}

fn canonical_bucket_pair(
    left: HjmtBucketKey,
    right: HjmtBucketKey,
) -> (HjmtBucketKey, HjmtBucketKey) {
    if left <= right {
        (left, right)
    } else {
        (right, left)
    }
}

fn derived_next_policy(
    current: BucketPolicy,
    bucket_bits: u8,
) -> Result<BucketPolicy, AdaptiveProofErr> {
    BucketPolicy::new(
        bucket_bits,
        current.min_bucket_count(),
        current.max_target_leaf_count(),
        current.compatibility_generation().saturating_add(1),
    )
    .map_err(|err| SettlementStoreError::Backend(err.to_string()).into())
}

fn split_threshold(policy: BucketPolicy) -> usize {
    usize::try_from(policy.min_bucket_count())
        .unwrap_or(usize::MAX.saturating_sub(1))
        .saturating_add(1)
}

fn merge_threshold(policy: BucketPolicy) -> usize {
    usize::try_from(policy.min_bucket_count().saturating_sub(1)).unwrap_or(0)
}

fn next_epoch(epoch: BucketEpoch) -> Result<BucketEpoch, AdaptiveProofErr> {
    epoch
        .get()
        .checked_add(1)
        .map(BucketEpoch::new)
        .ok_or(AdaptiveProofErr::WrongEpoch)
}

fn are_merge_siblings(left: BucketId, right: BucketId, bucket_bits: u8) -> bool {
    if bucket_bits <= 1 {
        return false;
    }
    sibling_bucket_id(left, bucket_bits) == right
}

fn sibling_bucket_id(bucket_id: BucketId, bucket_bits: u8) -> BucketId {
    let mut bytes = bucket_id.into_bytes();
    let bit_index = bucket_bits - 1;
    let byte_index = usize::from(bit_index / 8);
    let bit_mask = 1u8 << (7 - (bit_index % 8));
    bytes[byte_index] ^= bit_mask;
    BucketId::new(bytes)
}

fn bucket_evidence(
    key: HjmtBucketKey,
    policy: BucketPolicy,
    epoch: BucketEpoch,
    bucket_root: [u8; 32],
    occupancy: usize,
) -> BucketOccupancyEvidence {
    let class = occupancy_class(occupancy, policy);
    BucketOccupancyEvidence::new(
        OccupancyScope::Bucket,
        class,
        hash_zk::<StorAdaptivePolicyDom>(
            "adaptive_bucket_occupancy_evidence_v1",
            &[
                &[1],
                &[OccupancyScope::Bucket.tag()],
                &[class.tag()],
                key.0.as_bytes(),
                &key.1.get().to_be_bytes(),
                key.2.as_bytes(),
                &policy.bucket_policy_id(),
                &epoch.get().to_be_bytes(),
                &bucket_root,
            ],
        ),
    )
}

fn pair_evidence(
    left: HjmtBucketKey,
    right: HjmtBucketKey,
    merged_bucket_id: BucketId,
    policy: BucketPolicy,
    epoch: BucketEpoch,
    left_root: [u8; 32],
    right_root: [u8; 32],
    combined_count: usize,
) -> BucketOccupancyEvidence {
    let class = occupancy_class(combined_count, policy);
    BucketOccupancyEvidence::new(
        OccupancyScope::Pair,
        class,
        hash_zk::<StorAdaptivePolicyDom>(
            "adaptive_pair_occupancy_evidence_v1",
            &[
                &[1],
                &[OccupancyScope::Pair.tag()],
                &[class.tag()],
                left.0.as_bytes(),
                &left.1.get().to_be_bytes(),
                left.2.as_bytes(),
                right.2.as_bytes(),
                merged_bucket_id.as_bytes(),
                &policy.bucket_policy_id(),
                &epoch.get().to_be_bytes(),
                &left_root,
                &right_root,
            ],
        ),
    )
}

fn occupancy_class(occupancy: usize, policy: BucketPolicy) -> OccupancyClass {
    if occupancy == 0 {
        return OccupancyClass::Empty;
    }
    if occupancy <= merge_threshold(policy) {
        return OccupancyClass::MergeLow;
    }
    if occupancy < split_threshold(policy) {
        return OccupancyClass::Steady;
    }
    OccupancyClass::SplitReady
}

fn split_range_commitment(
    definition_id: DefinitionId,
    serial_id: SerialId,
    prior_bucket_id: BucketId,
    left_bucket_id: BucketId,
    right_bucket_id: BucketId,
) -> [u8; 32] {
    transition_digest(
        "split_range_commitment_v1",
        &[
            definition_id.into_bytes().to_vec(),
            serial_id.get().to_be_bytes().to_vec(),
            prior_bucket_id.into_bytes().to_vec(),
            left_bucket_id.into_bytes().to_vec(),
            right_bucket_id.into_bytes().to_vec(),
        ],
    )
}

fn merge_range_commitment(
    definition_id: DefinitionId,
    serial_id: SerialId,
    left_bucket_id: BucketId,
    right_bucket_id: BucketId,
    merged_bucket_id: BucketId,
) -> [u8; 32] {
    transition_digest(
        "merge_range_commitment_v1",
        &[
            definition_id.into_bytes().to_vec(),
            serial_id.get().to_be_bytes().to_vec(),
            left_bucket_id.into_bytes().to_vec(),
            right_bucket_id.into_bytes().to_vec(),
            merged_bucket_id.into_bytes().to_vec(),
        ],
    )
}

fn terminal_set_commitment(paths: &[SettlementPath]) -> [u8; 32] {
    let mut sorted = paths.to_vec();
    sorted.sort();
    let mut buffers = Vec::with_capacity(sorted.len());
    for path in sorted {
        let mut bytes = Vec::with_capacity(68);
        bytes.extend_from_slice(path.definition_id.as_bytes());
        bytes.extend_from_slice(&path.serial_id.get().to_be_bytes());
        bytes.extend_from_slice(path.terminal_id().as_bytes());
        buffers.push(bytes);
    }
    transition_digest("policy_terminal_commitment_v1", &buffers)
}

fn transition_digest(label: &'static str, parts: &[Vec<u8>]) -> [u8; 32] {
    let refs = parts.iter().map(Vec::as_slice).collect::<Vec<_>>();
    hash_zk::<StorAdaptivePolicyDom>(label, &refs)
}
