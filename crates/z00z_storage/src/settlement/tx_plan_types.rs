use jmt::{RootHash, Version};
use z00z_core::vouchers::VoucherLifecycleV1;

use crate::backend::{
    memory::MemTreeInner,
    roots::{HjmtRoots, TreeRoots},
};
use crate::settlement::{
    ClaimNullRec, ClaimNullifier, DefinitionId, FeeEnvelope, FeeReplayKey, FeeReplayRec,
    RightAction, RightActionCtx, SettlementLeaf, SettlementPath, SettlementStateRoot, StoreItem,
    TerminalId, VoucherLeaf,
};
use std::collections::{BTreeMap, HashMap};

use super::{model::SettlementModel, store::SettlementStore, SettlementStoreError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommittedObjectKindV1 {
    Asset,
    Voucher,
    Right,
}

impl CommittedObjectKindV1 {
    #[must_use]
    pub const fn from_leaf(leaf: &SettlementLeaf) -> Self {
        match leaf {
            SettlementLeaf::Terminal(_) => Self::Asset,
            SettlementLeaf::Voucher(_) => Self::Voucher,
            SettlementLeaf::Right(_) => Self::Right,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoucherActionCtx {
    pub now: u64,
    pub expected_holder: Option<[u8; 32]>,
    pub expected_beneficiary: Option<[u8; 32]>,
    pub expected_refund_target: Option<[u8; 32]>,
    pub acceptance_confirmed: bool,
    pub policy_allows_reject: bool,
    pub policy_allows_refund: bool,
    pub policy_allows_transfer: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoucherAction {
    Issue,
    Accept,
    Reject,
    Transfer,
    RedeemFull,
    RedeemPartial,
    Refund,
    Expire,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettlementActionV1 {
    /// Compatibility lane for raw store operations. Not an object-family
    /// lifecycle selector.
    CompatibilityStoreOps,
    /// Terminal asset settlement lane. This covers value-conserving asset
    /// deltas only; voucher issue and right create use their own typed
    /// variants, and this is not a generic public asset-mint selector.
    AssetMutation,
    /// Voucher lifecycle lane. Voucher actions operate on value-bearing
    /// `VoucherLeaf` objects with explicit policy and backing checks.
    Voucher(VoucherAction),
    /// Right lifecycle lane. Rights remain zero-value authority objects.
    Right(RightAction),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettlementObjectDeltaV1 {
    pub path: SettlementPath,
    pub object_kind: CommittedObjectKindV1,
    pub prior_leaf: Option<SettlementLeaf>,
    pub next_leaf: Option<SettlementLeaf>,
    pub declared_value_units: Option<u64>,
}

impl SettlementObjectDeltaV1 {
    pub fn deleted(
        path: SettlementPath,
        prior_leaf: SettlementLeaf,
        declared_value_units: Option<u64>,
    ) -> Self {
        Self {
            path,
            object_kind: CommittedObjectKindV1::from_leaf(&prior_leaf),
            prior_leaf: Some(prior_leaf),
            next_leaf: None,
            declared_value_units,
        }
    }

    pub fn created(
        path: SettlementPath,
        next_leaf: SettlementLeaf,
        declared_value_units: Option<u64>,
    ) -> Self {
        Self {
            path,
            object_kind: CommittedObjectKindV1::from_leaf(&next_leaf),
            prior_leaf: None,
            next_leaf: Some(next_leaf),
            declared_value_units,
        }
    }

    pub fn updated(
        path: SettlementPath,
        prior_leaf: SettlementLeaf,
        next_leaf: SettlementLeaf,
        declared_value_units: Option<u64>,
    ) -> Self {
        Self {
            path,
            object_kind: CommittedObjectKindV1::from_leaf(&prior_leaf),
            prior_leaf: Some(prior_leaf),
            next_leaf: Some(next_leaf),
            declared_value_units,
        }
    }

    pub fn bind_item(item: &StoreItem, declared_value_units: Option<u64>) -> Self {
        Self::created(item.path(), item.leaf().clone(), declared_value_units)
    }

    pub(crate) fn check_contract(
        &self,
        expect_prior: bool,
        expect_next: bool,
    ) -> Result<(), SettlementStoreError> {
        match (self.prior_leaf.as_ref(), self.next_leaf.as_ref()) {
            (Some(prior), Some(next)) => {
                if !expect_prior || !expect_next {
                    return Err(SettlementStoreError::ObjectDelta(
                        "typed object delta shape drifted from its section".to_string(),
                    ));
                }
                let prior_kind = CommittedObjectKindV1::from_leaf(prior);
                let next_kind = CommittedObjectKindV1::from_leaf(next);
                if prior_kind != self.object_kind || next_kind != self.object_kind {
                    return Err(SettlementStoreError::ObjectDelta(
                        "typed object delta family drifted across prior and next leaves"
                            .to_string(),
                    ));
                }
                prior.check_path(self.path).map_err(|err| {
                    SettlementStoreError::ObjectDelta(format!(
                        "typed object delta prior path binding failed: {err}"
                    ))
                })?;
                next.check_path(self.path).map_err(|err| {
                    SettlementStoreError::ObjectDelta(format!(
                        "typed object delta next path binding failed: {err}"
                    ))
                })?;
            }
            (Some(prior), None) => {
                if !expect_prior || expect_next {
                    return Err(SettlementStoreError::ObjectDelta(
                        "typed object delete delta shape drifted from its section".to_string(),
                    ));
                }
                let prior_kind = CommittedObjectKindV1::from_leaf(prior);
                if prior_kind != self.object_kind {
                    return Err(SettlementStoreError::ObjectDelta(
                        "typed object delete delta family drifted from prior leaf".to_string(),
                    ));
                }
                prior.check_path(self.path).map_err(|err| {
                    SettlementStoreError::ObjectDelta(format!(
                        "typed object delta prior path binding failed: {err}"
                    ))
                })?;
            }
            (None, Some(next)) => {
                if expect_prior || !expect_next {
                    return Err(SettlementStoreError::ObjectDelta(
                        "typed object create delta shape drifted from its section".to_string(),
                    ));
                }
                let next_kind = CommittedObjectKindV1::from_leaf(next);
                if next_kind != self.object_kind {
                    return Err(SettlementStoreError::ObjectDelta(
                        "typed object create delta family drifted from next leaf".to_string(),
                    ));
                }
                next.check_path(self.path).map_err(|err| {
                    SettlementStoreError::ObjectDelta(format!(
                        "typed object delta next path binding failed: {err}"
                    ))
                })?;
            }
            (None, None) => {
                return Err(SettlementStoreError::ObjectDelta(
                    "typed object delta must bind a prior leaf, a next leaf, or both".to_string(),
                ));
            }
        }

        if self.object_kind == CommittedObjectKindV1::Right
            && self.declared_value_units.unwrap_or(0) != 0
        {
            return Err(SettlementStoreError::ObjectDelta(
                "right deltas must not carry declared value units".to_string(),
            ));
        }

        Ok(())
    }

    fn prior_units(&self) -> Result<Option<u64>, SettlementStoreError> {
        self.value_units(self.prior_leaf.as_ref())
    }

    fn next_units(&self) -> Result<Option<u64>, SettlementStoreError> {
        self.value_units(self.next_leaf.as_ref())
    }

    fn value_units(
        &self,
        leaf: Option<&SettlementLeaf>,
    ) -> Result<Option<u64>, SettlementStoreError> {
        let Some(leaf) = leaf else {
            return Ok(None);
        };

        match leaf {
            SettlementLeaf::Terminal(_) => Ok(self.declared_value_units),
            SettlementLeaf::Voucher(voucher) => {
                if let Some(units) = self.declared_value_units {
                    if units != voucher_live_units(voucher) {
                        return Err(SettlementStoreError::ObjectDelta(
                            "voucher delta declared units drift from voucher remaining value"
                                .to_string(),
                        ));
                    }
                }
                Ok(Some(voucher_live_units(voucher)))
            }
            SettlementLeaf::Right(_) => Ok(Some(0)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectDeltaSetV1 {
    pub version: u8,
    pub selected_action: SettlementActionV1,
    pub policy_descriptor_hash: [u8; 32],
    pub voucher_action_ctx: Option<VoucherActionCtx>,
    pub deleted_objects: Vec<SettlementObjectDeltaV1>,
    pub created_objects: Vec<SettlementObjectDeltaV1>,
    pub updated_objects: Vec<SettlementObjectDeltaV1>,
    pub fee_envelope: Option<FeeEnvelope>,
    pub prior_root: SettlementStateRoot,
    pub expected_new_root: SettlementStateRoot,
}

impl ObjectDeltaSetV1 {
    #[must_use]
    pub fn new(
        selected_action: SettlementActionV1,
        policy_descriptor_hash: [u8; 32],
        voucher_action_ctx: Option<VoucherActionCtx>,
        deleted_objects: Vec<SettlementObjectDeltaV1>,
        created_objects: Vec<SettlementObjectDeltaV1>,
        updated_objects: Vec<SettlementObjectDeltaV1>,
        fee_envelope: Option<FeeEnvelope>,
        prior_root: SettlementStateRoot,
        expected_new_root: SettlementStateRoot,
    ) -> Self {
        Self {
            version: 1,
            selected_action,
            policy_descriptor_hash,
            voucher_action_ctx,
            deleted_objects,
            created_objects,
            updated_objects,
            fee_envelope,
            prior_root,
            expected_new_root,
        }
    }

    pub(crate) fn check_contract(&self) -> Result<(), SettlementStoreError> {
        if self.version != 1 {
            return Err(SettlementStoreError::ObjectDelta(
                "typed object delta version is unsupported".to_string(),
            ));
        }
        if let Some(fee_envelope) = self.fee_envelope {
            fee_envelope
                .check()
                .map_err(|err| SettlementStoreError::ObjectDelta(err.to_string()))?;
        }

        let mut seen = std::collections::BTreeSet::new();
        for delta in &self.deleted_objects {
            delta.check_contract(true, false)?;
            if !seen.insert(delta.path) {
                return Err(SettlementStoreError::ObjectDelta(
                    "typed object delta contains duplicate settlement paths".to_string(),
                ));
            }
        }
        for delta in &self.created_objects {
            delta.check_contract(false, true)?;
            if !seen.insert(delta.path) {
                return Err(SettlementStoreError::ObjectDelta(
                    "typed object delta contains duplicate settlement paths".to_string(),
                ));
            }
        }
        for delta in &self.updated_objects {
            delta.check_contract(true, true)?;
            if !seen.insert(delta.path) {
                return Err(SettlementStoreError::ObjectDelta(
                    "typed object delta contains duplicate settlement paths".to_string(),
                ));
            }
        }

        self.check_role_boundaries()?;
        self.check_policy_binding()?;
        self.check_right_transfer_semantics()?;
        self.check_conservation()?;
        self.check_voucher_semantics()?;
        Ok(())
    }

    pub fn validate_contract(&self) -> Result<(), SettlementStoreError> {
        self.check_contract()
    }

    fn check_role_boundaries(&self) -> Result<(), SettlementStoreError> {
        if let Some(fee_envelope) = self.fee_envelope {
            if let Some(support_ref) = fee_envelope.support_ref {
                for voucher in self
                    .created_objects
                    .iter()
                    .chain(self.updated_objects.iter())
                    .filter_map(|delta| delta.next_leaf.as_ref())
                    .filter_map(|leaf| match leaf {
                        SettlementLeaf::Voucher(voucher) => Some(voucher),
                        SettlementLeaf::Terminal(_) | SettlementLeaf::Right(_) => None,
                    })
                {
                    if matches!(voucher.backing, super::VoucherBackingRef::ReserveCommitment(backing) if backing == support_ref)
                        || matches!(voucher.backing, super::VoucherBackingRef::GenesisReserve(backing) if backing == support_ref)
                    {
                        return Err(SettlementStoreError::ObjectDelta(
                            "fee support must not become voucher backing".to_string(),
                        ));
                    }
                }
            }
        }

        for right in self
            .deleted_objects
            .iter()
            .chain(self.created_objects.iter())
            .chain(self.updated_objects.iter())
            .filter(|delta| delta.object_kind == CommittedObjectKindV1::Right)
        {
            if right.declared_value_units.unwrap_or(0) != 0 {
                return Err(SettlementStoreError::ObjectDelta(
                    "right deltas must remain zero-value".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn check_policy_binding(&self) -> Result<(), SettlementStoreError> {
        match self.selected_action {
            SettlementActionV1::CompatibilityStoreOps | SettlementActionV1::AssetMutation => Ok(()),
            SettlementActionV1::Voucher(_) => {
                let mut seen_any = false;
                for voucher in self.voucher_prior_next_leaves() {
                    seen_any = true;
                    if voucher.policy_id != self.policy_descriptor_hash {
                        return Err(SettlementStoreError::ObjectDelta(
                            "voucher delta policy hash does not match committed voucher policy"
                                .to_string(),
                        ));
                    }
                }
                if !seen_any {
                    return Err(SettlementStoreError::ObjectDelta(
                        "voucher action requires voucher delta members".to_string(),
                    ));
                }
                Ok(())
            }
            SettlementActionV1::Right(action) => {
                let mut seen_any = false;
                for leaf in self
                    .deleted_objects
                    .iter()
                    .chain(self.created_objects.iter())
                    .chain(self.updated_objects.iter())
                    .flat_map(|delta| [delta.prior_leaf.as_ref(), delta.next_leaf.as_ref()])
                    .flatten()
                {
                    let SettlementLeaf::Right(right) = leaf else {
                        continue;
                    };
                    seen_any = true;
                    let expect = match action {
                        RightAction::Create
                        | RightAction::Transfer
                        | RightAction::Consume
                        | RightAction::Challenge => right.transition_policy_id,
                        RightAction::Revoke => right.revocation_policy_id,
                        RightAction::Expire => self.policy_descriptor_hash,
                    };
                    if expect != self.policy_descriptor_hash {
                        return Err(SettlementStoreError::ObjectDelta(
                            "right delta policy hash does not match selected right action"
                                .to_string(),
                        ));
                    }
                }
                if !seen_any {
                    return Err(SettlementStoreError::ObjectDelta(
                        "right action requires right delta members".to_string(),
                    ));
                }
                Ok(())
            }
        }
    }

    fn check_conservation(&self) -> Result<(), SettlementStoreError> {
        if matches!(
            self.selected_action,
            SettlementActionV1::CompatibilityStoreOps
        ) {
            return Ok(());
        }

        let input_units = self.input_units()?;
        let output_units = self.output_units()?;
        if input_units != output_units {
            return Err(SettlementStoreError::ObjectDelta(format!(
                "typed object delta conservation mismatch: inputs {input_units} != outputs {output_units}"
            )));
        }
        Ok(())
    }

    fn check_right_transfer_semantics(&self) -> Result<(), SettlementStoreError> {
        if self.selected_action != SettlementActionV1::Right(RightAction::Transfer) {
            return Ok(());
        }
        if !self.deleted_objects.is_empty() || !self.created_objects.is_empty() {
            return Err(SettlementStoreError::ObjectDelta(
                "right transfer must update one live right in place".to_string(),
            ));
        }

        let mut seen_any = false;
        for delta in &self.updated_objects {
            let Some(SettlementLeaf::Right(prior)) = delta.prior_leaf.as_ref() else {
                continue;
            };
            let Some(SettlementLeaf::Right(next)) = delta.next_leaf.as_ref() else {
                continue;
            };
            seen_any = true;
            prior
                .check_path(delta.path)
                .map_err(|err| SettlementStoreError::ObjectDelta(err.to_string()))?;
            next.check_path(delta.path)
                .map_err(|err| SettlementStoreError::ObjectDelta(err.to_string()))?;
            next.validate_action(
                RightAction::Transfer,
                RightActionCtx::default(),
                Some(prior),
            )
            .map_err(|err| SettlementStoreError::ObjectDelta(err.to_string()))?;
        }

        if !seen_any {
            return Err(SettlementStoreError::ObjectDelta(
                "right transfer must update exactly one right".to_string(),
            ));
        }

        Ok(())
    }

    fn check_voucher_semantics(&self) -> Result<(), SettlementStoreError> {
        let SettlementActionV1::Voucher(action) = self.selected_action else {
            return Ok(());
        };
        let ctx = self.voucher_action_ctx.ok_or_else(|| {
            SettlementStoreError::ObjectDelta(
                "voucher action requires explicit voucher action context".to_string(),
            )
        })?;
        match action {
            VoucherAction::Issue => self.check_issue(ctx),
            VoucherAction::Accept => self.check_accept(ctx),
            VoucherAction::Reject => self.check_reject(ctx),
            VoucherAction::Transfer => self.check_transfer(ctx),
            VoucherAction::RedeemFull => self.check_redeem_full(ctx),
            VoucherAction::RedeemPartial => self.check_redeem_partial(ctx),
            VoucherAction::Refund => self.check_refund(ctx),
            VoucherAction::Expire => self.check_expire(ctx),
        }
    }

    fn check_issue(&self, _ctx: VoucherActionCtx) -> Result<(), SettlementStoreError> {
        let created = self.created_vouchers()?;
        if created.len() != 1 || !self.updated_objects.is_empty() {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher issue must create exactly one voucher and no residual updates".to_string(),
            ));
        }
        let voucher = created[0];
        if voucher.face_value == 0 || voucher.remaining_value == 0 {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher issue must create a positive-value voucher".to_string(),
            ));
        }
        if voucher.remaining_value > voucher.face_value {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher issue remaining value exceeds face value".to_string(),
            ));
        }
        match voucher.backing {
            super::VoucherBackingRef::ConsumedAsset {
                definition_id,
                serial_id,
            } => {
                let deleted_assets = self.deleted_assets()?;
                if deleted_assets.len() != 1 {
                    return Err(SettlementStoreError::ObjectDelta(
                        "voucher issue from consumed asset must delete exactly one asset"
                            .to_string(),
                    ));
                }
                let deleted_path = deleted_assets[0].path;
                if deleted_path.definition_id.into_bytes() != definition_id
                    || deleted_path.serial_id.get() != serial_id
                {
                    return Err(SettlementStoreError::ObjectDelta(
                        "voucher issue backing does not match deleted source asset".to_string(),
                    ));
                }
            }
            super::VoucherBackingRef::ReserveCommitment(backing)
            | super::VoucherBackingRef::GenesisReserve(backing) => {
                if backing == [0u8; 32] {
                    return Err(SettlementStoreError::ObjectDelta(
                        "voucher issue reserve backing must not be zero".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn input_units(&self) -> Result<u64, SettlementStoreError> {
        Ok(sum_delta_units(&self.deleted_objects, true)?
            .saturating_add(sum_delta_units(&self.updated_objects, true)?))
    }

    fn output_units(&self) -> Result<u64, SettlementStoreError> {
        if !matches!(
            self.selected_action,
            SettlementActionV1::Voucher(VoucherAction::Issue)
        ) {
            return Ok(sum_delta_units(&self.created_objects, false)?
                .saturating_add(sum_delta_units(&self.updated_objects, false)?));
        }

        let mut total = sum_delta_units(&self.updated_objects, false)?;
        for delta in &self.created_objects {
            let units = match delta
                .next_leaf
                .as_ref()
                .expect("checked create delta has next leaf")
            {
                SettlementLeaf::Voucher(voucher)
                    if matches!(
                        voucher.backing,
                        super::VoucherBackingRef::ReserveCommitment(_)
                            | super::VoucherBackingRef::GenesisReserve(_)
                    ) =>
                {
                    0
                }
                SettlementLeaf::Voucher(_) => delta.next_units()?.unwrap_or(0),
                SettlementLeaf::Terminal(_) | SettlementLeaf::Right(_) => {
                    delta.next_units()?.unwrap_or(0)
                }
            };
            total = total.saturating_add(units);
        }
        Ok(total)
    }

    fn check_accept(&self, ctx: VoucherActionCtx) -> Result<(), SettlementStoreError> {
        if !self.deleted_objects.is_empty() || !self.created_objects.is_empty() {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher accept must update one live voucher in place".to_string(),
            ));
        }
        let updated = self.updated_vouchers()?;
        if updated.len() != 1 {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher accept must update exactly one voucher".to_string(),
            ));
        }
        let (prior, next) = updated[0];
        if prior.lifecycle != VoucherLifecycleV1::PendingAcceptance
            || next.lifecycle != VoucherLifecycleV1::Active
        {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher accept requires pending -> active lifecycle transition".to_string(),
            ));
        }
        if prior.receiver_must_accept && !ctx.acceptance_confirmed {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher accept must fail closed without explicit holder acceptance".to_string(),
            ));
        }
        self.check_holder_beneficiary_bind(prior, ctx)
    }

    fn check_reject(&self, ctx: VoucherActionCtx) -> Result<(), SettlementStoreError> {
        let deleted = self.deleted_vouchers()?;
        let created_assets = self.created_assets()?;
        if deleted.len() != 1 || created_assets.len() != 1 || !self.updated_objects.is_empty() {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher reject must delete one voucher and create one refund asset".to_string(),
            ));
        }
        let voucher = deleted[0];
        if !voucher.allow_reject || !ctx.policy_allows_reject {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher reject is not allowed by policy".to_string(),
            ));
        }
        self.check_holder_beneficiary_bind(voucher, ctx)?;
        self.require_declared_asset_units(created_assets[0], voucher.remaining_value)?;
        self.check_refund_output_bind(voucher, created_assets[0])?;
        Ok(())
    }

    fn check_transfer(&self, ctx: VoucherActionCtx) -> Result<(), SettlementStoreError> {
        if !self.deleted_objects.is_empty() || !self.created_objects.is_empty() {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher transfer must update one live voucher in place".to_string(),
            ));
        }
        let updated = self.updated_vouchers()?;
        if updated.len() != 1 {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher transfer must update exactly one voucher".to_string(),
            ));
        }
        let (prior, next) = updated[0];
        if !ctx.policy_allows_transfer {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher transfer is not allowed by policy".to_string(),
            ));
        }
        if prior.beneficiary_commitment != next.beneficiary_commitment
            || prior.refund_target_commitment != next.refund_target_commitment
            || prior.backing != next.backing
            || prior.face_value != next.face_value
            || prior.remaining_value != next.remaining_value
        {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher transfer must not drift beneficiary, refund target, backing, or value"
                    .to_string(),
            ));
        }
        self.check_holder_beneficiary_bind(prior, ctx)
    }

    fn check_redeem_full(&self, ctx: VoucherActionCtx) -> Result<(), SettlementStoreError> {
        let deleted = self.deleted_vouchers()?;
        let created_assets = self.created_assets()?;
        if deleted.len() != 1 || created_assets.len() != 1 || !self.updated_objects.is_empty() {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher full redeem must delete one voucher and create one asset".to_string(),
            ));
        }
        let voucher = deleted[0];
        self.ensure_live_redeemable(voucher, ctx)?;
        self.require_declared_asset_units(created_assets[0], voucher.remaining_value)?;
        Ok(())
    }

    fn check_redeem_partial(&self, ctx: VoucherActionCtx) -> Result<(), SettlementStoreError> {
        let created_assets = self.created_assets()?;
        let updated = self.updated_vouchers()?;
        if self
            .deleted_objects
            .iter()
            .any(|delta| delta.object_kind == CommittedObjectKindV1::Voucher)
            || created_assets.len() != 1
            || updated.len() != 1
        {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher partial redeem must create one asset and update one voucher".to_string(),
            ));
        }
        let (prior, next) = updated[0];
        self.ensure_live_redeemable(prior, ctx)?;
        if next.lifecycle != VoucherLifecycleV1::PartiallyRedeemed {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher partial redeem must move voucher into partially_redeemed".to_string(),
            ));
        }
        if next.remaining_value == 0 || next.remaining_value >= prior.remaining_value {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher partial redeem residual value is malformed".to_string(),
            ));
        }
        if prior.face_value != next.face_value
            || prior.backing != next.backing
            || prior.beneficiary_commitment != next.beneficiary_commitment
            || prior.refund_target_commitment != next.refund_target_commitment
        {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher partial redeem must preserve backing, face value, beneficiary, and refund target"
                    .to_string(),
            ));
        }
        let redeemed_units = prior.remaining_value.saturating_sub(next.remaining_value);
        self.require_declared_asset_units(created_assets[0], redeemed_units)?;
        Ok(())
    }

    fn check_refund(&self, ctx: VoucherActionCtx) -> Result<(), SettlementStoreError> {
        let deleted = self.deleted_vouchers()?;
        let created_assets = self.created_assets()?;
        if deleted.len() != 1 || created_assets.len() != 1 || !self.updated_objects.is_empty() {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher refund must delete one voucher and create one asset".to_string(),
            ));
        }
        let voucher = deleted[0];
        if !ctx.policy_allows_refund {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher refund is not allowed by policy".to_string(),
            ));
        }
        if ctx.now <= voucher.validity.valid_until
            && voucher.lifecycle != VoucherLifecycleV1::Expired
            && voucher.lifecycle != VoucherLifecycleV1::Rejected
        {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher refund requires expiry or an explicit rejected/expired lifecycle"
                    .to_string(),
            ));
        }
        self.check_holder_beneficiary_bind(voucher, ctx)?;
        self.require_declared_asset_units(created_assets[0], voucher.remaining_value)?;
        self.check_refund_output_bind(voucher, created_assets[0])?;
        Ok(())
    }

    fn check_expire(&self, ctx: VoucherActionCtx) -> Result<(), SettlementStoreError> {
        if !self.deleted_objects.is_empty() || !self.created_objects.is_empty() {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher expire must update one live voucher in place".to_string(),
            ));
        }
        let updated = self.updated_vouchers()?;
        if updated.len() != 1 {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher expire must update exactly one voucher".to_string(),
            ));
        }
        let (prior, next) = updated[0];
        if ctx.now <= prior.validity.valid_until {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher expire requires the validity window to be over".to_string(),
            ));
        }
        if next.lifecycle != VoucherLifecycleV1::Expired {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher expire must set lifecycle to expired".to_string(),
            ));
        }
        if prior.remaining_value != next.remaining_value || prior.backing != next.backing {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher expire must preserve remaining value and backing".to_string(),
            ));
        }
        Ok(())
    }

    fn deleted_assets(&self) -> Result<Vec<&SettlementObjectDeltaV1>, SettlementStoreError> {
        self.collect_deltas(&self.deleted_objects, CommittedObjectKindV1::Asset)
    }

    fn created_assets(&self) -> Result<Vec<&SettlementObjectDeltaV1>, SettlementStoreError> {
        self.collect_deltas(&self.created_objects, CommittedObjectKindV1::Asset)
    }

    fn deleted_vouchers(&self) -> Result<Vec<&VoucherLeaf>, SettlementStoreError> {
        let deltas = self.collect_deltas(&self.deleted_objects, CommittedObjectKindV1::Voucher)?;
        deltas
            .into_iter()
            .map(|delta| {
                let SettlementLeaf::Voucher(voucher) = delta
                    .prior_leaf
                    .as_ref()
                    .expect("checked delete delta has prior leaf")
                else {
                    unreachable!("checked delete delta has voucher prior leaf");
                };
                Ok(voucher)
            })
            .collect()
    }

    fn created_vouchers(&self) -> Result<Vec<&VoucherLeaf>, SettlementStoreError> {
        let deltas = self.collect_deltas(&self.created_objects, CommittedObjectKindV1::Voucher)?;
        deltas
            .into_iter()
            .map(|delta| {
                let SettlementLeaf::Voucher(voucher) = delta
                    .next_leaf
                    .as_ref()
                    .expect("checked create delta has next leaf")
                else {
                    unreachable!("checked create delta has voucher next leaf");
                };
                Ok(voucher)
            })
            .collect()
    }

    fn updated_vouchers(&self) -> Result<Vec<(&VoucherLeaf, &VoucherLeaf)>, SettlementStoreError> {
        let deltas = self.collect_deltas(&self.updated_objects, CommittedObjectKindV1::Voucher)?;
        deltas
            .into_iter()
            .map(|delta| {
                let SettlementLeaf::Voucher(prior) = delta
                    .prior_leaf
                    .as_ref()
                    .expect("checked update delta has prior leaf")
                else {
                    unreachable!("checked update delta has voucher prior leaf");
                };
                let SettlementLeaf::Voucher(next) = delta
                    .next_leaf
                    .as_ref()
                    .expect("checked update delta has next leaf")
                else {
                    unreachable!("checked update delta has voucher next leaf");
                };
                Ok((prior, next))
            })
            .collect()
    }

    fn voucher_prior_next_leaves(&self) -> impl Iterator<Item = &VoucherLeaf> {
        self.deleted_objects
            .iter()
            .chain(self.created_objects.iter())
            .chain(self.updated_objects.iter())
            .flat_map(|delta| [delta.prior_leaf.as_ref(), delta.next_leaf.as_ref()])
            .flatten()
            .filter_map(|leaf| match leaf {
                SettlementLeaf::Voucher(voucher) => Some(voucher),
                SettlementLeaf::Terminal(_) | SettlementLeaf::Right(_) => None,
            })
    }

    fn collect_deltas<'a>(
        &'a self,
        deltas: &'a [SettlementObjectDeltaV1],
        kind: CommittedObjectKindV1,
    ) -> Result<Vec<&'a SettlementObjectDeltaV1>, SettlementStoreError> {
        Ok(deltas
            .iter()
            .filter(|delta| delta.object_kind == kind)
            .collect())
    }

    fn check_holder_beneficiary_bind(
        &self,
        voucher: &VoucherLeaf,
        ctx: VoucherActionCtx,
    ) -> Result<(), SettlementStoreError> {
        if let Some(holder) = ctx.expected_holder {
            if holder != voucher.holder_commitment {
                return Err(SettlementStoreError::ObjectDelta(
                    "voucher holder commitment binding mismatch".to_string(),
                ));
            }
        }
        if let Some(beneficiary) = ctx.expected_beneficiary {
            if beneficiary != voucher.beneficiary_commitment {
                return Err(SettlementStoreError::ObjectDelta(
                    "voucher beneficiary commitment binding mismatch".to_string(),
                ));
            }
        }
        if let Some(refund_target) = ctx.expected_refund_target {
            if refund_target != voucher.refund_target_commitment {
                return Err(SettlementStoreError::ObjectDelta(
                    "voucher refund target commitment binding mismatch".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn ensure_live_redeemable(
        &self,
        voucher: &VoucherLeaf,
        ctx: VoucherActionCtx,
    ) -> Result<(), SettlementStoreError> {
        self.check_holder_beneficiary_bind(voucher, ctx)?;
        if ctx.now > voucher.validity.valid_until {
            return Err(SettlementStoreError::ObjectDelta(
                "expired voucher use must fail closed".to_string(),
            ));
        }
        if voucher.lifecycle == VoucherLifecycleV1::PendingAcceptance
            && voucher.receiver_must_accept
            && !ctx.acceptance_confirmed
        {
            return Err(SettlementStoreError::ObjectDelta(
                "forced acceptance must fail closed before voucher redeem".to_string(),
            ));
        }
        if matches!(
            voucher.lifecycle,
            VoucherLifecycleV1::Redeemed
                | VoucherLifecycleV1::Refunded
                | VoucherLifecycleV1::Rejected
        ) {
            return Err(SettlementStoreError::ObjectDelta(
                "double redeem or closed voucher use must fail closed".to_string(),
            ));
        }
        Ok(())
    }

    fn check_refund_output_bind(
        &self,
        voucher: &VoucherLeaf,
        refund_asset: &SettlementObjectDeltaV1,
    ) -> Result<(), SettlementStoreError> {
        if refund_asset.path.terminal_id.into_bytes() != voucher.refund_target_commitment {
            return Err(SettlementStoreError::ObjectDelta(
                "voucher refund output target does not match declared refund target commitment"
                    .to_string(),
            ));
        }

        match voucher.backing {
            super::VoucherBackingRef::ConsumedAsset {
                definition_id,
                serial_id,
            } => {
                if refund_asset.path.definition_id.into_bytes() != definition_id
                    || refund_asset.path.serial_id.get() != serial_id
                {
                    return Err(SettlementStoreError::ObjectDelta(
                        "voucher refund output source context does not match declared consumed-asset backing"
                            .to_string(),
                    ));
                }
            }
            super::VoucherBackingRef::ReserveCommitment(backing)
            | super::VoucherBackingRef::GenesisReserve(backing) => {
                if refund_asset.path.definition_id.into_bytes() != backing {
                    return Err(SettlementStoreError::ObjectDelta(
                        "voucher refund output source context does not match declared reserve backing"
                            .to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    fn require_declared_asset_units(
        &self,
        delta: &SettlementObjectDeltaV1,
        expect_units: u64,
    ) -> Result<(), SettlementStoreError> {
        let units = delta.declared_value_units.ok_or_else(|| {
            SettlementStoreError::ObjectDelta(
                "typed asset delta is missing declared value units".to_string(),
            )
        })?;
        if units != expect_units {
            return Err(SettlementStoreError::ObjectDelta(
                "typed asset delta declared value does not match voucher accounting".to_string(),
            ));
        }
        Ok(())
    }
}

fn voucher_live_units(voucher: &VoucherLeaf) -> u64 {
    match voucher.lifecycle {
        VoucherLifecycleV1::Redeemed
        | VoucherLifecycleV1::Rejected
        | VoucherLifecycleV1::Refunded => 0,
        VoucherLifecycleV1::PendingAcceptance
        | VoucherLifecycleV1::Active
        | VoucherLifecycleV1::PartiallyRedeemed
        | VoucherLifecycleV1::Expired => voucher.remaining_value,
    }
}

fn sum_delta_units(
    deltas: &[SettlementObjectDeltaV1],
    prior: bool,
) -> Result<u64, SettlementStoreError> {
    let mut total = 0u64;
    for delta in deltas {
        let units = if prior {
            delta.prior_units()?
        } else {
            delta.next_units()?
        };
        total = total.saturating_add(units.unwrap_or(0));
    }
    Ok(total)
}

#[derive(Clone)]
pub(super) struct StoreSnap {
    pub(super) flat_inner: MemTreeInner,
    pub(super) flat_version: Version,
    pub(super) flat_root: Option<RootHash>,
    pub(super) model: SettlementModel,
    pub(super) tree_roots: TreeRoots,
    pub(super) path_by_terminal_id: HashMap<TerminalId, SettlementPath>,
    pub(super) nullifier: BTreeMap<ClaimNullifier, ClaimNullRec>,
    pub(super) claim_null_seq: u64,
    pub(super) fee_replays: BTreeMap<FeeReplayKey, FeeReplayRec>,
    pub(super) fee_replay_seq: u64,
    pub(super) settlement_root_by_ver: HashMap<Version, SettlementStateRoot>,
    pub(super) model_by_ver: HashMap<Version, SettlementModel>,
    pub(super) hjmt_roots_by_ver: HashMap<Version, HjmtRoots>,
    pub(super) last_object_delta: Option<ObjectDeltaSetV1>,
    pub(super) object_deltas_by_ver: HashMap<Version, ObjectDeltaSetV1>,
}

#[derive(Default)]
pub(super) struct SeenOps {
    path_set: std::collections::BTreeSet<SettlementPath>,
    terminal_id_map: HashMap<TerminalId, SettlementPath>,
}

impl SeenOps {
    pub(super) fn touch(&mut self, path: SettlementPath) -> Result<(), SettlementStoreError> {
        if !self.path_set.insert(path) {
            return Err(SettlementStoreError::OpPathDup);
        }

        if let Some(prev) = self.terminal_id_map.insert(path.terminal_id(), path) {
            if prev != path {
                return Err(SettlementStoreError::PathTerminalMix);
            }
        }

        Ok(())
    }

    pub(super) fn into_paths(self) -> Vec<SettlementPath> {
        self.path_set.into_iter().collect()
    }
}

pub(super) struct NextState {
    pub(super) def_ids: Vec<DefinitionId>,
    pub(super) model: SettlementModel,
    pub(super) terminal_path_ops: HashMap<TerminalId, Option<SettlementPath>>,
}

impl NextState {
    pub(super) fn new(store: &SettlementStore, def_ids: &[DefinitionId], path_cap: usize) -> Self {
        Self {
            def_ids: def_ids.to_vec(),
            model: store.model.scope_clone(def_ids),
            terminal_path_ops: HashMap::with_capacity(path_cap),
        }
    }

    pub(super) fn path_opt(
        &self,
        store: &SettlementStore,
        terminal_id: &TerminalId,
    ) -> Option<SettlementPath> {
        match self.terminal_path_ops.get(terminal_id) {
            Some(path) => *path,
            None => store.path_by_terminal_id.get(terminal_id).copied(),
        }
    }

    pub(super) fn merge_into(self, store: &mut SettlementStore) {
        store.model.merge_scope(self.model, &self.def_ids);

        for (terminal_id, path) in self.terminal_path_ops {
            match path {
                Some(path) => {
                    store.path_by_terminal_id.insert(terminal_id, path);
                }
                None => {
                    store.path_by_terminal_id.remove(&terminal_id);
                }
            }
        }
    }
}
