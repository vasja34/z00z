use crate::settlement::proof::hjmt_checkpoint_digest;
use crate::settlement::SettlementStore;
use crate::{
    checkpoint::CheckpointExecTx,
    settlement::{
        chk_blob_settlement, hjmt_default_child_commitment, hjmt_default_value_commitment,
        CheckRoot, ClaimSourceRoot, FeeActorCtx, FeeEnvelope, FeeErr, FeeReplayKey, FeeReplayRec,
        FeeSupportCtx, HjmtProofFamily, ModelErr, ObjectDeltaSetV1, ProofBlob, ProofChkErr,
        ProofItem, ProofScanOut, RightAction, RightActionCtx, RightLeaf, SettlementActionV1,
        SettlementLeaf, SettlementLeafFamily, SettlementListReq, SettlementLookup,
        SettlementObjectDeltaV1, SettlementPage, SettlementPath, SettlementStateRoot, StoreItem,
        VoucherAction, VoucherActionCtx, VoucherLeaf, HJMT_DEFAULT_COMMITMENT_VERSION,
    },
};
use z00z_crypto::{expert::hash_domain, hash_zk::hash_zk};
use z00z_crypto::{ClaimProofVer, ClaimSourceProof, CLAIM_ROOT_VERSION};
use z00z_utils::codec::{BincodeCodec, Codec};

use super::types::{
    terminal_value_hash, ClaimNullRec, ClaimNullTx, ClaimNullifier, SettlementStoreError, StoreOp,
};

hash_domain!(StorFeeOpsDom, "z00z.storage.settlement.fee.ops.v1", 1);
hash_domain!(StorFeeExecDom, "z00z.storage.settlement.fee.exec.v1", 1);

impl SettlementStore {
    /// Derive the live storage-backed claim-source contract for one canonical settlement item.
    /// The item must already match persisted membership in this store.
    /// Missing or drifted items are rejected instead of being re-derived through a helper-owned tree.
    pub fn claim_source_contract_for_item(
        &self,
        item: &StoreItem,
    ) -> Result<(ClaimSourceRoot, ClaimSourceProof), SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_claim_contract(item)
    }

    pub fn get_settlement_item(
        &self,
        path: &SettlementPath,
    ) -> Result<Option<StoreItem>, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_get_settlement_item(path)
    }

    pub fn lookup_settlement(
        &self,
        lookup: SettlementLookup,
    ) -> Result<Option<StoreItem>, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_lookup_settlement(lookup)
    }

    pub fn list_settlement(
        &self,
        req: SettlementListReq,
    ) -> Result<SettlementPage, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_list_settlement(req)
    }

    pub fn check_root(&self) -> Result<CheckRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_check_root()
    }

    pub fn put_settlement_item(
        &mut self,
        item: StoreItem,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_put_settlement_item(item)
    }

    pub fn del_settlement_item(
        &mut self,
        path: &SettlementPath,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_del_settlement_item(path)
    }

    pub fn apply_attested_settlement_ops(
        &mut self,
        ops: Vec<StoreOp>,
        txs: Vec<CheckpointExecTx>,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_apply_attest_exec(ops, txs)
    }

    pub fn apply_object_delta_set(
        &mut self,
        ops: Vec<StoreOp>,
        delta_set: ObjectDeltaSetV1,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_apply_ops_with_delta(ops, Some(delta_set))
    }

    pub fn apply_attested_object_delta_set(
        &mut self,
        ops: Vec<StoreOp>,
        txs: Vec<CheckpointExecTx>,
        delta_set: ObjectDeltaSetV1,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_apply_attest_delta(ops, txs, Some(delta_set))
    }

    pub fn apply_fee_object_delta_set(
        &mut self,
        ops: Vec<StoreOp>,
        delta_set: ObjectDeltaSetV1,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_apply_fee_delta(ops, envelope, actor, Some(delta_set))
    }

    pub fn apply_attested_fee_delta(
        &mut self,
        ops: Vec<StoreOp>,
        txs: Vec<CheckpointExecTx>,
        delta_set: ObjectDeltaSetV1,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_apply_fee_attest_delta(ops, txs, envelope, actor, Some(delta_set))
    }

    #[must_use]
    pub fn latest_object_delta(&self) -> Option<&ObjectDeltaSetV1> {
        self.last_object_delta.as_ref()
    }

    pub fn object_delta_for_version(
        &self,
        version: jmt::Version,
    ) -> Result<Option<ObjectDeltaSetV1>, SettlementStoreError> {
        self.require_hjmt_mode()?;
        if let Some(delta) = self.object_deltas_by_ver.get(&version) {
            return Ok(Some(delta.clone()));
        }
        let Some(store) = self.hjmt_store_at(version)? else {
            return Ok(None);
        };
        Ok(store.object_deltas_by_ver.get(&version).cloned())
    }

    pub fn settlement_proof_item(
        &self,
        path: &SettlementPath,
    ) -> Result<ProofItem, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_settlement_proof_item(path)
    }

    pub fn settlement_proof_blob(
        &self,
        path: &SettlementPath,
    ) -> Result<ProofBlob, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_settlement_proof_blob(path)
    }

    pub fn settlement_nonexistence_proof_blob(
        &self,
        path: &SettlementPath,
        leaf_family: SettlementLeafFamily,
    ) -> Result<ProofBlob, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_settlement_nonexistence_proof_blob(path, leaf_family)
    }

    pub fn settlement_proof_scan(
        &self,
        path: &SettlementPath,
    ) -> Result<ProofScanOut, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_settlement_proof_scan(path)
    }

    pub fn apply_settlement_claim_ops(
        &mut self,
        ops: Vec<StoreOp>,
        claims: &[ClaimNullTx],
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_apply_claim_ops(ops, claims)
    }

    pub fn settlement_claim_null_rec(
        &self,
        nullifier: &ClaimNullifier,
    ) -> Result<Option<ClaimNullRec>, SettlementStoreError> {
        self.require_hjmt_mode()?;
        Ok(self.nullifier.get(nullifier).cloned())
    }

    pub fn claim_source_root(&self) -> Result<ClaimSourceRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_claim_source_root()
    }

    pub fn claim_source_proof(
        &self,
        path: &SettlementPath,
    ) -> Result<ClaimSourceProof, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_claim_source_proof_settlement(path)
    }

    pub fn validate_reload(&self) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_validate_reload()
    }

    pub fn create_right(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        leaf.check_path(path).map_err(ModelErr::from)?;
        leaf.validate_action(RightAction::Create, ctx, None)
            .map_err(ModelErr::from)?;
        Err(SettlementStoreError::Fee(FeeErr::SupportRequired))
    }

    pub fn create_right_with_fee(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        leaf.check_path(path).map_err(ModelErr::from)?;
        leaf.validate_action(RightAction::Create, ctx, None)
            .map_err(ModelErr::from)?;
        self.apply_fee_ops(
            vec![StoreOp::Put(Box::new(StoreItem::new(path, leaf)?))],
            envelope,
            actor,
        )
    }

    pub fn transfer_right(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        let prior = self.current_right_leaf(&path)?;
        leaf.check_path(path).map_err(ModelErr::from)?;
        leaf.validate_action(RightAction::Transfer, ctx, Some(&prior))
            .map_err(ModelErr::from)?;
        Err(SettlementStoreError::Fee(FeeErr::SupportRequired))
    }

    pub fn transfer_right_with_fee(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        let prior = self.current_right_leaf(&path)?;
        leaf.check_path(path).map_err(ModelErr::from)?;
        leaf.validate_action(RightAction::Transfer, ctx, Some(&prior))
            .map_err(ModelErr::from)?;
        self.apply_fee_ops(
            vec![StoreOp::Put(Box::new(StoreItem::new(path, leaf)?))],
            envelope,
            actor,
        )
    }

    pub fn consume_right(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        let current = self.current_right_leaf(&path)?;
        current
            .validate_action(RightAction::Consume, ctx, Some(&current))
            .map_err(ModelErr::from)?;
        Err(SettlementStoreError::Fee(FeeErr::SupportRequired))
    }

    pub fn consume_right_with_fee(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        let current = self.current_right_leaf(&path)?;
        current
            .validate_action(RightAction::Consume, ctx, Some(&current))
            .map_err(ModelErr::from)?;
        self.apply_fee_ops(vec![StoreOp::Delete(path)], envelope, actor)
    }

    pub fn revoke_right(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        let current = self.current_right_leaf(&path)?;
        current
            .validate_action(RightAction::Revoke, ctx, None)
            .map_err(ModelErr::from)?;
        Err(SettlementStoreError::Fee(FeeErr::SupportRequired))
    }

    pub fn revoke_right_with_fee(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        let current = self.current_right_leaf(&path)?;
        current
            .validate_action(RightAction::Revoke, ctx, None)
            .map_err(ModelErr::from)?;
        self.apply_fee_ops(vec![StoreOp::Delete(path)], envelope, actor)
    }

    pub fn expire_right(
        &mut self,
        path: SettlementPath,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        let current = self.current_right_leaf(&path)?;
        current
            .validate_action(RightAction::Expire, ctx, None)
            .map_err(ModelErr::from)?;
        self.del_settlement_item(&path)
    }

    pub fn challenge_right(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        let prior = self.current_right_leaf(&path)?;
        leaf.check_path(path).map_err(ModelErr::from)?;
        leaf.validate_action(RightAction::Challenge, ctx, Some(&prior))
            .map_err(ModelErr::from)?;
        Err(SettlementStoreError::Fee(FeeErr::SupportRequired))
    }

    pub fn challenge_right_with_fee(
        &mut self,
        path: SettlementPath,
        leaf: RightLeaf,
        ctx: RightActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        let prior = self.current_right_leaf(&path)?;
        leaf.check_path(path).map_err(ModelErr::from)?;
        leaf.validate_action(RightAction::Challenge, ctx, Some(&prior))
            .map_err(ModelErr::from)?;
        self.apply_fee_ops(
            vec![StoreOp::Put(Box::new(StoreItem::new(path, leaf)?))],
            envelope,
            actor,
        )
    }

    pub fn issue_voucher_with_fee(
        &mut self,
        source_asset: Option<(SettlementPath, u64)>,
        path: SettlementPath,
        leaf: VoucherLeaf,
        ctx: VoucherActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        validate_voucher_leaf_contract(&leaf)?;
        leaf.check_path(path).map_err(ModelErr::from)?;

        let mut ops = Vec::new();
        let mut deleted = Vec::new();
        if let Some((source_path, source_units)) = source_asset {
            let source_item = self.current_terminal_item(&source_path)?;
            ops.push(StoreOp::Delete(source_path));
            deleted.push(SettlementObjectDeltaV1::deleted(
                source_path,
                source_item.leaf().clone(),
                Some(source_units),
            ));
        }

        let voucher_item = StoreItem::new(path, SettlementLeaf::Voucher(leaf.clone()))?;
        ops.push(StoreOp::Put(Box::new(voucher_item.clone())));
        let delta = ObjectDeltaSetV1::new(
            SettlementActionV1::Voucher(VoucherAction::Issue),
            leaf.policy_id,
            Some(ctx),
            deleted,
            vec![SettlementObjectDeltaV1::created(
                path,
                SettlementLeaf::Voucher(leaf),
                None,
            )],
            Vec::new(),
            Some(envelope),
            SettlementStateRoot::settlement_v1([0u8; 32]),
            SettlementStateRoot::settlement_v1([0u8; 32]),
        );
        self.apply_fee_object_delta_set(ops, delta, envelope, actor)
    }

    pub fn accept_voucher_with_fee(
        &mut self,
        path: SettlementPath,
        leaf: VoucherLeaf,
        ctx: VoucherActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        validate_voucher_leaf_contract(&leaf)?;
        leaf.check_path(path).map_err(ModelErr::from)?;
        let prior = self.current_voucher_leaf(&path)?;
        let ops = vec![StoreOp::Put(Box::new(StoreItem::new(
            path,
            SettlementLeaf::Voucher(leaf.clone()),
        )?))];
        let delta = ObjectDeltaSetV1::new(
            SettlementActionV1::Voucher(VoucherAction::Accept),
            leaf.policy_id,
            Some(ctx),
            Vec::new(),
            Vec::new(),
            vec![SettlementObjectDeltaV1::updated(
                path,
                SettlementLeaf::Voucher(prior),
                SettlementLeaf::Voucher(leaf),
                None,
            )],
            Some(envelope),
            SettlementStateRoot::settlement_v1([0u8; 32]),
            SettlementStateRoot::settlement_v1([0u8; 32]),
        );
        self.apply_fee_object_delta_set(ops, delta, envelope, actor)
    }

    pub fn reject_voucher_with_fee(
        &mut self,
        path: SettlementPath,
        refund_output: StoreItem,
        refund_units: u64,
        ctx: VoucherActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        let prior = self.current_voucher_leaf(&path)?;
        let _ = refund_output.terminal_leaf()?;
        let ops = vec![
            StoreOp::Delete(path),
            StoreOp::Put(Box::new(refund_output.clone())),
        ];
        let delta = ObjectDeltaSetV1::new(
            SettlementActionV1::Voucher(VoucherAction::Reject),
            prior.policy_id,
            Some(ctx),
            vec![SettlementObjectDeltaV1::deleted(
                path,
                SettlementLeaf::Voucher(prior),
                None,
            )],
            vec![SettlementObjectDeltaV1::created(
                refund_output.path(),
                refund_output.leaf().clone(),
                Some(refund_units),
            )],
            Vec::new(),
            Some(envelope),
            SettlementStateRoot::settlement_v1([0u8; 32]),
            SettlementStateRoot::settlement_v1([0u8; 32]),
        );
        self.apply_fee_object_delta_set(ops, delta, envelope, actor)
    }

    pub fn transfer_voucher_with_fee(
        &mut self,
        path: SettlementPath,
        leaf: VoucherLeaf,
        ctx: VoucherActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        validate_voucher_leaf_contract(&leaf)?;
        leaf.check_path(path).map_err(ModelErr::from)?;
        let prior = self.current_voucher_leaf(&path)?;
        let ops = vec![StoreOp::Put(Box::new(StoreItem::new(
            path,
            SettlementLeaf::Voucher(leaf.clone()),
        )?))];
        let delta = ObjectDeltaSetV1::new(
            SettlementActionV1::Voucher(VoucherAction::Transfer),
            leaf.policy_id,
            Some(ctx),
            Vec::new(),
            Vec::new(),
            vec![SettlementObjectDeltaV1::updated(
                path,
                SettlementLeaf::Voucher(prior),
                SettlementLeaf::Voucher(leaf),
                None,
            )],
            Some(envelope),
            SettlementStateRoot::settlement_v1([0u8; 32]),
            SettlementStateRoot::settlement_v1([0u8; 32]),
        );
        self.apply_fee_object_delta_set(ops, delta, envelope, actor)
    }

    pub fn redeem_voucher_full_with_fee(
        &mut self,
        path: SettlementPath,
        asset_output: StoreItem,
        asset_units: u64,
        ctx: VoucherActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        let prior = self.current_voucher_leaf(&path)?;
        let _ = asset_output.terminal_leaf()?;
        let ops = vec![
            StoreOp::Delete(path),
            StoreOp::Put(Box::new(asset_output.clone())),
        ];
        let delta = ObjectDeltaSetV1::new(
            SettlementActionV1::Voucher(VoucherAction::RedeemFull),
            prior.policy_id,
            Some(ctx),
            vec![SettlementObjectDeltaV1::deleted(
                path,
                SettlementLeaf::Voucher(prior),
                None,
            )],
            vec![SettlementObjectDeltaV1::created(
                asset_output.path(),
                asset_output.leaf().clone(),
                Some(asset_units),
            )],
            Vec::new(),
            Some(envelope),
            SettlementStateRoot::settlement_v1([0u8; 32]),
            SettlementStateRoot::settlement_v1([0u8; 32]),
        );
        self.apply_fee_object_delta_set(ops, delta, envelope, actor)
    }

    pub fn redeem_voucher_partial_with_fee(
        &mut self,
        path: SettlementPath,
        residual_leaf: VoucherLeaf,
        asset_output: StoreItem,
        asset_units: u64,
        ctx: VoucherActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        validate_voucher_leaf_contract(&residual_leaf)?;
        residual_leaf.check_path(path).map_err(ModelErr::from)?;
        let prior = self.current_voucher_leaf(&path)?;
        let _ = asset_output.terminal_leaf()?;
        let ops = vec![
            StoreOp::Put(Box::new(StoreItem::new(
                path,
                SettlementLeaf::Voucher(residual_leaf.clone()),
            )?)),
            StoreOp::Put(Box::new(asset_output.clone())),
        ];
        let delta = ObjectDeltaSetV1::new(
            SettlementActionV1::Voucher(VoucherAction::RedeemPartial),
            residual_leaf.policy_id,
            Some(ctx),
            Vec::new(),
            vec![SettlementObjectDeltaV1::created(
                asset_output.path(),
                asset_output.leaf().clone(),
                Some(asset_units),
            )],
            vec![SettlementObjectDeltaV1::updated(
                path,
                SettlementLeaf::Voucher(prior),
                SettlementLeaf::Voucher(residual_leaf),
                None,
            )],
            Some(envelope),
            SettlementStateRoot::settlement_v1([0u8; 32]),
            SettlementStateRoot::settlement_v1([0u8; 32]),
        );
        self.apply_fee_object_delta_set(ops, delta, envelope, actor)
    }

    pub fn refund_voucher_with_fee(
        &mut self,
        path: SettlementPath,
        refund_output: StoreItem,
        refund_units: u64,
        ctx: VoucherActionCtx,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        let prior = self.current_voucher_leaf(&path)?;
        let _ = refund_output.terminal_leaf()?;
        let ops = vec![
            StoreOp::Delete(path),
            StoreOp::Put(Box::new(refund_output.clone())),
        ];
        let delta = ObjectDeltaSetV1::new(
            SettlementActionV1::Voucher(VoucherAction::Refund),
            prior.policy_id,
            Some(ctx),
            vec![SettlementObjectDeltaV1::deleted(
                path,
                SettlementLeaf::Voucher(prior),
                None,
            )],
            vec![SettlementObjectDeltaV1::created(
                refund_output.path(),
                refund_output.leaf().clone(),
                Some(refund_units),
            )],
            Vec::new(),
            Some(envelope),
            SettlementStateRoot::settlement_v1([0u8; 32]),
            SettlementStateRoot::settlement_v1([0u8; 32]),
        );
        self.apply_fee_object_delta_set(ops, delta, envelope, actor)
    }

    pub fn expire_voucher(
        &mut self,
        path: SettlementPath,
        leaf: VoucherLeaf,
        ctx: VoucherActionCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        validate_voucher_leaf_contract(&leaf)?;
        leaf.check_path(path).map_err(ModelErr::from)?;
        let prior = self.current_voucher_leaf(&path)?;
        let ops = vec![StoreOp::Put(Box::new(StoreItem::new(
            path,
            SettlementLeaf::Voucher(leaf.clone()),
        )?))];
        let delta = ObjectDeltaSetV1::new(
            SettlementActionV1::Voucher(VoucherAction::Expire),
            leaf.policy_id,
            Some(ctx),
            Vec::new(),
            Vec::new(),
            vec![SettlementObjectDeltaV1::updated(
                path,
                SettlementLeaf::Voucher(prior),
                SettlementLeaf::Voucher(leaf),
                None,
            )],
            None,
            SettlementStateRoot::settlement_v1([0u8; 32]),
            SettlementStateRoot::settlement_v1([0u8; 32]),
        );
        self.apply_object_delta_set(ops, delta)
    }

    pub(super) fn hjmt_claim_contract(
        &self,
        item: &StoreItem,
    ) -> Result<(ClaimSourceRoot, ClaimSourceProof), SettlementStoreError> {
        let live_item = self
            .hjmt_get_settlement_item(&item.path())?
            .ok_or(SettlementStoreError::PathMiss)?;
        if live_item != *item {
            return Err(SettlementStoreError::PathMiss);
        }

        let claim_root = self.hjmt_claim_source_root()?;
        let claim_proof = self.hjmt_claim_source_proof_settlement(&item.path())?;
        Ok((claim_root, claim_proof))
    }

    pub(super) fn validate_proof_blob(proof_blob: &ProofBlob) -> Result<(), SettlementStoreError> {
        let proof_item = proof_blob.item().clone();
        let path = proof_item.path();
        chk_blob_settlement(
            &proof_blob.encode()?,
            proof_item.settlement_root(),
            &path,
            proof_item.def_leaf(),
            proof_item.ser_leaf(),
            proof_item.leaf(),
        )
        .map_err(SettlementStoreError::Proof)?;
        Ok(())
    }

    pub fn validate_settlement_proof_blob(
        &self,
        proof_blob: &ProofBlob,
    ) -> Result<(), SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.validate_settlement_blob_with_family(proof_blob, None)
    }

    pub fn validate_settlement_nonexistence_proof_blob(
        &self,
        proof_blob: &ProofBlob,
        leaf_family: SettlementLeafFamily,
    ) -> Result<(), SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.validate_settlement_blob_with_family(proof_blob, Some(leaf_family))
    }

    fn validate_settlement_blob_with_family(
        &self,
        proof_blob: &ProofBlob,
        nonexistence_family: Option<SettlementLeafFamily>,
    ) -> Result<(), SettlementStoreError> {
        let Some(family) = proof_blob.hjmt_proof_family() else {
            return Err(SettlementStoreError::Proof(ProofChkErr::ProofFamilyMix));
        };
        if nonexistence_family.is_some() && family != HjmtProofFamily::NonExistence {
            return Err(SettlementStoreError::Proof(ProofChkErr::ProofFamilyMix));
        }
        Self::validate_proof_blob(proof_blob)?;
        if proof_blob.item().settlement_root() != self.hjmt_roots.settlement_root() {
            return Err(SettlementStoreError::Proof(ProofChkErr::RootGenerationMix));
        }
        if proof_blob.hjmt_journal_checkpoint() != Some(self.hjmt_roots.version) {
            return Err(SettlementStoreError::Proof(
                ProofChkErr::JournalCheckpointMix,
            ));
        }
        let current_backend_root = self.hjmt_current_backend_root()?;
        if proof_blob.backend_root() != current_backend_root {
            return Err(SettlementStoreError::Proof(ProofChkErr::RootBindMix));
        }
        if proof_blob.hjmt_journal_digest()
            != Some(self.hjmt_current_journal_digest(current_backend_root))
        {
            return Err(SettlementStoreError::Proof(
                ProofChkErr::JournalCheckpointMix,
            ));
        }
        if proof_blob.hjmt_default_commitment_version() != Some(HJMT_DEFAULT_COMMITMENT_VERSION)
            || proof_blob.hjmt_default_child_commitment() != Some(hjmt_default_child_commitment())
        {
            return Err(SettlementStoreError::Proof(
                ProofChkErr::DefaultCommitmentMix,
            ));
        }

        match family {
            HjmtProofFamily::Inclusion => {
                if self
                    .hjmt_get_settlement_item(&proof_blob.item().path())?
                    .is_none()
                {
                    return Err(SettlementStoreError::PathMiss);
                }
            }
            HjmtProofFamily::NonExistence => {
                let Some(expected_leaf_family) = nonexistence_family else {
                    return Err(SettlementStoreError::Proof(ProofChkErr::ProofFamilyMix));
                };
                if proof_blob.hjmt_leaf_family() != Some(expected_leaf_family) {
                    return Err(SettlementStoreError::Proof(ProofChkErr::LeafMix));
                }
                let marker_leaf = expected_leaf_family.marker_leaf(proof_blob.item().path());
                if proof_blob.item().leaf() != &marker_leaf {
                    return Err(SettlementStoreError::Proof(ProofChkErr::LeafMix));
                }
                if proof_blob.terminal_leaf_hash() != terminal_value_hash(marker_leaf)?.0 {
                    return Err(SettlementStoreError::Proof(ProofChkErr::LeafHashMix));
                }
                if self
                    .hjmt_get_settlement_item(&proof_blob.item().path())?
                    .is_some()
                {
                    return Err(SettlementStoreError::Proof(ProofChkErr::TerminalProofMix));
                }
                if proof_blob.hjmt_default_commitment() != Some(hjmt_default_value_commitment()) {
                    return Err(SettlementStoreError::Proof(
                        ProofChkErr::DefaultCommitmentMix,
                    ));
                }
            }
            HjmtProofFamily::Deletion => {
                if self
                    .hjmt_get_settlement_item(&proof_blob.item().path())?
                    .is_some()
                {
                    return Err(SettlementStoreError::Proof(ProofChkErr::TerminalProofMix));
                }
                let prior = proof_blob
                    .hjmt_prior()
                    .ok_or(SettlementStoreError::Proof(ProofChkErr::PriorRootMix))?;
                if prior.version() >= self.hjmt_roots.version {
                    return Err(SettlementStoreError::Proof(ProofChkErr::PriorRootMix));
                }
                if self.settlement_root_for_version(prior.version())? != prior.settlement_root() {
                    return Err(SettlementStoreError::Proof(ProofChkErr::PriorRootMix));
                }
                let (_, prior_roots) = self
                    .hjmt_history_at(prior.version())?
                    .ok_or(SettlementStoreError::Proof(ProofChkErr::PriorRootMix))?;
                if prior.backend_root() != self.hjmt_backend_root_for_roots(&prior_roots)? {
                    return Err(SettlementStoreError::Proof(ProofChkErr::PriorRootMix));
                }
                let expected_prior_digest = prior_roots.journal_digest.unwrap_or_else(|| {
                    hjmt_checkpoint_digest(
                        prior.settlement_root(),
                        prior.backend_root(),
                        prior.version(),
                    )
                });
                if prior.journal_digest() != Some(expected_prior_digest) {
                    return Err(SettlementStoreError::Proof(
                        ProofChkErr::JournalCheckpointMix,
                    ));
                }
                if self.hjmt_last_version_for_path(&proof_blob.item().path())? != prior.version() {
                    return Err(SettlementStoreError::Proof(ProofChkErr::PriorRootMix));
                }
            }
        }

        Ok(())
    }

    pub fn apply_fee_ops(
        &mut self,
        ops: Vec<StoreOp>,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_apply_fee_ops(ops, envelope, actor)
    }

    pub fn apply_attested_fee_ops(
        &mut self,
        ops: Vec<StoreOp>,
        txs: Vec<CheckpointExecTx>,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.require_hjmt_mode()?;
        self.hjmt_apply_fee_attest_exec(ops, txs, envelope, actor)
    }

    pub fn fee_support_ctx(&self, ops: &[StoreOp]) -> Result<FeeSupportCtx, SettlementStoreError> {
        self.require_hjmt_mode()?;
        let root = self.settlement_root()?;
        fee_support_ctx(
            ops,
            None,
            root.into_bytes(),
            self.hjmt_roots.version.max(self.flat_version),
        )
    }

    pub fn fee_support_exec_ctx(
        &self,
        ops: &[StoreOp],
        txs: &[CheckpointExecTx],
    ) -> Result<FeeSupportCtx, SettlementStoreError> {
        self.require_hjmt_mode()?;
        let root = self.settlement_root()?;
        fee_support_ctx(
            ops,
            Some(txs),
            root.into_bytes(),
            self.hjmt_roots.version.max(self.flat_version),
        )
    }

    pub(crate) fn check_fee_support(
        &self,
        envelope: &FeeEnvelope,
        support: FeeSupportCtx,
        actor: FeeActorCtx,
    ) -> Result<FeeReplayRec, SettlementStoreError> {
        let replay_seen = self.fee_replays.contains_key(&envelope.replay_id());
        envelope.validate_support(support, actor, replay_seen)?;
        Ok(envelope.replay_rec(self.fee_replay_seq.saturating_add(1)))
    }

    pub fn fee_replay_rec(
        &self,
        replay_key: &FeeReplayKey,
    ) -> Result<Option<FeeReplayRec>, SettlementStoreError> {
        Ok(self.fee_replays.get(replay_key).copied())
    }

    pub(super) fn hjmt_claim_source_root(&self) -> Result<ClaimSourceRoot, SettlementStoreError> {
        Ok(ClaimSourceRoot::new_settlement(
            CLAIM_ROOT_VERSION,
            self.settlement_root()?,
        ))
    }

    pub(super) fn hjmt_claim_source_proof_settlement(
        &self,
        path: &SettlementPath,
    ) -> Result<ClaimSourceProof, SettlementStoreError> {
        let claim_root = self.hjmt_claim_source_root()?;
        let proof_blob = self.hjmt_settlement_proof_blob(path)?;
        Self::validate_proof_blob(&proof_blob)?;
        Self::claim_source_proof_from_blob(claim_root, proof_blob)
    }

    fn claim_source_proof_from_blob(
        claim_root: ClaimSourceRoot,
        proof_blob: ProofBlob,
    ) -> Result<ClaimSourceProof, SettlementStoreError> {
        let proof_blob = proof_blob.encode()?;
        // Storage remains the authoritative issuer of V1 inner proof semantics,
        // but this witness path is membership-only and does not prove Pedersen
        // conservation by itself.
        ClaimSourceProof::new(
            claim_root.root_version(),
            claim_root.into_bytes(),
            ClaimProofVer::V1,
            proof_blob,
        )
        .map_err(|err| SettlementStoreError::Backend(err.to_string()))
    }

    pub(crate) fn sorted_paths(&self) -> Vec<SettlementPath> {
        let mut paths = self.model.paths();
        paths.sort_unstable();
        paths
    }

    fn current_right_leaf(&self, path: &SettlementPath) -> Result<RightLeaf, SettlementStoreError> {
        let item = self
            .get_settlement_item(path)?
            .ok_or(SettlementStoreError::PathMiss)?;
        Ok(*item.right_leaf()?)
    }

    fn current_voucher_leaf(
        &self,
        path: &SettlementPath,
    ) -> Result<VoucherLeaf, SettlementStoreError> {
        let item = self
            .get_settlement_item(path)?
            .ok_or(SettlementStoreError::PathMiss)?;
        Ok(item.voucher_leaf()?.clone())
    }

    fn current_terminal_item(
        &self,
        path: &SettlementPath,
    ) -> Result<StoreItem, SettlementStoreError> {
        let item = self
            .get_settlement_item(path)?
            .ok_or(SettlementStoreError::PathMiss)?;
        let _ = item.terminal_leaf()?;
        Ok(item)
    }
}

fn validate_voucher_leaf_contract(leaf: &VoucherLeaf) -> Result<(), SettlementStoreError> {
    if leaf.terminal_id.is_zero() {
        return Err(SettlementStoreError::ObjectDelta(
            "voucher terminal id must not be zero".to_string(),
        ));
    }
    if leaf.validity.valid_until < leaf.validity.valid_from {
        return Err(SettlementStoreError::ObjectDelta(
            "voucher validity window is malformed".to_string(),
        ));
    }
    if leaf.remaining_value > leaf.face_value {
        return Err(SettlementStoreError::ObjectDelta(
            "voucher remaining value exceeds face value".to_string(),
        ));
    }
    if leaf.policy_id == [0u8; 32] || leaf.action_pool_id == [0u8; 32] {
        return Err(SettlementStoreError::ObjectDelta(
            "voucher policy hash and action pool id must be explicit".to_string(),
        ));
    }
    Ok(())
}

fn fee_support_ctx(
    ops: &[StoreOp],
    txs: Option<&[CheckpointExecTx]>,
    pre_state_root: [u8; 32],
    pre_state_version: u64,
) -> Result<FeeSupportCtx, SettlementStoreError> {
    let codec = BincodeCodec;
    let ops_bytes = codec.serialize(&ops.to_vec())?;
    let op_count = u64::try_from(ops.len()).unwrap_or(u64::MAX);
    let pre_state_version = pre_state_version.to_be_bytes();
    let (required_units, domain_id, transition_id) = if let Some(txs) = txs {
        let txs_bytes = codec.serialize(&txs.to_vec())?;
        let tx_count = u64::try_from(txs.len()).unwrap_or(u64::MAX);
        (
            op_count.saturating_add(tx_count),
            hash_zk::<StorFeeExecDom>(
                "domain",
                &[
                    ops_bytes.as_slice(),
                    txs_bytes.as_slice(),
                    &pre_state_root,
                    &pre_state_version,
                ],
            ),
            hash_zk::<StorFeeExecDom>(
                "transition",
                &[
                    ops_bytes.as_slice(),
                    txs_bytes.as_slice(),
                    &pre_state_root,
                    &pre_state_version,
                ],
            ),
        )
    } else {
        (
            op_count,
            hash_zk::<StorFeeOpsDom>(
                "domain",
                &[ops_bytes.as_slice(), &pre_state_root, &pre_state_version],
            ),
            hash_zk::<StorFeeOpsDom>(
                "transition",
                &[ops_bytes.as_slice(), &pre_state_root, &pre_state_version],
            ),
        )
    };

    Ok(FeeSupportCtx {
        required_units,
        domain_id,
        transition_id,
    })
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use z00z_core::assets::{AssetLeaf, AssetPackPlain};
    use z00z_crypto::ZkPackEncrypted;
    use z00z_utils::config::{ConfigSource, EnvConfig};

    use crate::settlement::store::{test_env_lock, TEST_HJMT_INJ_STAGE_ENV};
    use crate::settlement::SettlementStore;
    use crate::settlement::{
        proof::HjmtPriorProofEnvelope, DefinitionId, ProofChkErr, ProofItem, SerialId,
        SettlementPath, StoreItem, TerminalId, TerminalLeaf,
    };

    use crate::backend::types::SettlementStoreError;

    const BACKEND_ENV: &str = "Z00Z_SETTLEMENT_BACKEND_MODE";

    fn test_path(definition: u8, serial: u32, asset: u8) -> SettlementPath {
        SettlementPath::new(
            DefinitionId::new([definition; 32]),
            SerialId::new(serial),
            TerminalId::new([asset; 32]),
        )
    }

    fn test_item(path: SettlementPath, value: u64) -> StoreItem {
        let payload = AssetPackPlain {
            value,
            blinding: [3u8; 32],
            s_out: [4u8; 32],
        }
        .to_bytes();
        let leaf: TerminalLeaf = AssetLeaf {
            asset_id: path.terminal_id().into_bytes(),
            serial_id: path.serial_id.get(),
            r_pub: [1u8; 32],
            owner_tag: [2u8; 32],
            c_amount: [5u8; 32],
            enc_pack: ZkPackEncrypted {
                version: 1,
                ciphertext: payload,
                tag: [0u8; 16],
            },
            range_proof: vec![9u8; 4],
            tag16: 11,
        }
        .into();
        StoreItem::new(path, leaf).expect("test item")
    }

    #[test]
    fn proof_keeps_bind_err() {
        let mut store = SettlementStore::new();
        let path = test_path(7, 9, 11);
        store
            .put_settlement_item(test_item(path, 33))
            .expect("put item");

        let proof_blob = store.settlement_proof_blob(&path).expect("proof blob");
        let tampered = proof_blob.with_root_bind(1, [0x42; 32]);
        let err = SettlementStore::validate_proof_blob(&tampered).expect_err("tampered proof");

        match err {
            SettlementStoreError::Proof(ProofChkErr::RootBindMix) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn deletion_rejects_stale_prior_version() {
        let mut store = SettlementStore::new();
        let path = test_path(9, 1, 7);

        store
            .put_settlement_item(test_item(path, 900))
            .expect("put first item");
        let first_blob = store
            .settlement_proof_blob(&path)
            .expect("first proof blob");

        store
            .put_settlement_item(test_item(path, 901))
            .expect("rewrite item");
        store.del_settlement_item(&path).expect("delete item");

        let deletion_blob = store
            .settlement_proof_blob(&path)
            .expect("deletion proof blob");
        let stale_item = ProofItem::new_settlement(
            deletion_blob.item().settlement_root(),
            path,
            deletion_blob.item().def_leaf(),
            deletion_blob.item().ser_leaf(),
            first_blob.item().leaf().clone(),
        )
        .expect("stale item");
        let stale_prior = HjmtPriorProofEnvelope::new(
            1,
            first_blob.item().settlement_root(),
            first_blob.backend_root(),
            first_blob.hjmt_journal_digest().expect("journal digest"),
            first_blob.item().def_leaf(),
            first_blob.item().ser_leaf(),
            first_blob
                .hjmt_bucket_root_leaf()
                .expect("bucket root leaf"),
            first_blob.definition_proof().to_vec(),
            first_blob.serial_proof().to_vec(),
            first_blob
                .hjmt_bucket_proof()
                .expect("bucket proof")
                .to_vec(),
            first_blob.terminal_proof().to_vec(),
        );

        let err = store
            .validate_settlement_proof_blob(
                &deletion_blob
                    .rebind(stale_item)
                    .with_terminal_leaf_hash(first_blob.terminal_leaf_hash())
                    .with_hjmt_prior(stale_prior),
            )
            .expect_err("stale prior must reject");

        match err {
            SettlementStoreError::Proof(ProofChkErr::PriorRootMix) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn reload_del_rejects_stale_ver() {
        let guard = test_env_lock().lock().expect("env lock");
        let previous_mode = EnvConfig.get(BACKEND_ENV).expect("read backend env");
        let previous_inj_stage = EnvConfig
            .get(TEST_HJMT_INJ_STAGE_ENV)
            .expect("read inj-stage env");
        std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
        std::env::set_var(BACKEND_ENV, "hjmt");

        {
            let temp = tempdir().expect("tempdir");
            let path = test_path(10, 1, 8);

            let mut store = SettlementStore::load(temp.path()).expect("load store");
            store
                .put_settlement_item(test_item(path, 1_000))
                .expect("put first item");
            let first_blob = store
                .settlement_proof_blob(&path)
                .expect("first proof blob");

            store
                .put_settlement_item(test_item(path, 1_001))
                .expect("rewrite item");
            store.del_settlement_item(&path).expect("delete item");
            drop(store);

            let reloaded = SettlementStore::load(temp.path()).expect("reload store");
            let deletion_blob = reloaded
                .settlement_proof_blob(&path)
                .expect("deletion proof blob");
            let stale_item = ProofItem::new_settlement(
                deletion_blob.item().settlement_root(),
                path,
                deletion_blob.item().def_leaf(),
                deletion_blob.item().ser_leaf(),
                first_blob.item().leaf().clone(),
            )
            .expect("stale item");
            let stale_prior = HjmtPriorProofEnvelope::new(
                1,
                first_blob.item().settlement_root(),
                first_blob.backend_root(),
                first_blob.hjmt_journal_digest().expect("journal digest"),
                first_blob.item().def_leaf(),
                first_blob.item().ser_leaf(),
                first_blob
                    .hjmt_bucket_root_leaf()
                    .expect("bucket root leaf"),
                first_blob.definition_proof().to_vec(),
                first_blob.serial_proof().to_vec(),
                first_blob
                    .hjmt_bucket_proof()
                    .expect("bucket proof")
                    .to_vec(),
                first_blob.terminal_proof().to_vec(),
            );

            let err = reloaded
                .validate_settlement_proof_blob(
                    &deletion_blob
                        .rebind(stale_item)
                        .with_terminal_leaf_hash(first_blob.terminal_leaf_hash())
                        .with_hjmt_prior(stale_prior),
                )
                .expect_err("stale prior must reject after reload");

            match err {
                SettlementStoreError::Proof(ProofChkErr::PriorRootMix) => {}
                other => panic!("unexpected error: {other:?}"),
            }
        }

        if let Some(value) = previous_mode {
            std::env::set_var(BACKEND_ENV, value);
        } else {
            std::env::remove_var(BACKEND_ENV);
        }
        if let Some(value) = previous_inj_stage {
            std::env::set_var(TEST_HJMT_INJ_STAGE_ENV, value);
        } else {
            std::env::remove_var(TEST_HJMT_INJ_STAGE_ENV);
        }
        drop(guard);
    }
}
