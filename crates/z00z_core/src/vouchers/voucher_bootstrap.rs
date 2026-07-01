use serde::{Deserialize, Serialize};

use crate::{
    actions::ActionPoolId, config_name::validate_underscore_name, policies::PolicyId, AssetError,
};

use super::{
    VoucherAcceptanceTermsV1, VoucherBackingReferenceV1, VoucherConfigEntry, VoucherLifecycleV1,
    VoucherValidityWindowV1,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Bootstrap-only voucher manifest entry.
///
/// `VoucherBootstrapEntryV1` is not the runtime voucher object. Genesis
/// materializes it into `VoucherConfigEntry`, and live settlement/wallet lanes
/// persist vouchers as `VoucherLeaf`.
pub struct VoucherBootstrapEntryV1 {
    pub id: String,
    pub domain_name: String,
    pub issuer_fixture: String,
    pub holder_fixture: String,
    pub beneficiary_fixture: String,
    pub backing: VoucherBackingReferenceV1,
    pub face_value: u64,
    pub remaining_value: u64,
    pub policy_label: String,
    pub lifecycle: VoucherLifecycleV1,
    pub validity: VoucherValidityWindowV1,
    pub acceptance: VoucherAcceptanceTermsV1,
    pub replay_nonce: [u8; 32],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disclosure_commitment: Option<[u8; 32]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_commitment: Option<[u8; 32]>,
}

impl VoucherBootstrapEntryV1 {
    pub fn validate(&self) -> Result<(), AssetError> {
        for (field, value) in [
            ("id", self.id.as_str()),
            ("domain_name", self.domain_name.as_str()),
            ("issuer_fixture", self.issuer_fixture.as_str()),
            ("holder_fixture", self.holder_fixture.as_str()),
            ("beneficiary_fixture", self.beneficiary_fixture.as_str()),
            ("policy_label", self.policy_label.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(AssetError::InvalidAsset(
                    format!("voucher.{field} must not be empty").into(),
                ));
            }
        }
        validate_underscore_name("voucher.policy_label", self.policy_label.as_str())?;

        let provisional = VoucherConfigEntry {
            id: self.id.clone(),
            domain_name: self.domain_name.clone(),
            issuer_fixture: self.issuer_fixture.clone(),
            holder_fixture: self.holder_fixture.clone(),
            beneficiary_fixture: self.beneficiary_fixture.clone(),
            backing: self.backing.clone(),
            face_value: self.face_value,
            remaining_value: self.remaining_value,
            policy_id: PolicyId::new([0x11; 32]),
            action_pool_id: ActionPoolId::new([0x22; 32]),
            lifecycle: self.lifecycle,
            validity: self.validity.clone(),
            acceptance: self.acceptance.clone(),
            replay_nonce: self.replay_nonce,
            disclosure_commitment: self.disclosure_commitment,
            audit_commitment: self.audit_commitment,
        };
        provisional.validate()?;
        Ok(())
    }

    pub fn materialize(
        &self,
        policy_id: PolicyId,
        action_pool_id: ActionPoolId,
    ) -> Result<VoucherConfigEntry, AssetError> {
        let config = VoucherConfigEntry {
            id: self.id.clone(),
            domain_name: self.domain_name.clone(),
            issuer_fixture: self.issuer_fixture.clone(),
            holder_fixture: self.holder_fixture.clone(),
            beneficiary_fixture: self.beneficiary_fixture.clone(),
            backing: self.backing.clone(),
            face_value: self.face_value,
            remaining_value: self.remaining_value,
            policy_id,
            action_pool_id,
            lifecycle: self.lifecycle,
            validity: self.validity.clone(),
            acceptance: self.acceptance.clone(),
            replay_nonce: self.replay_nonce,
            disclosure_commitment: self.disclosure_commitment,
            audit_commitment: self.audit_commitment,
        };
        config.validate()?;
        Ok(config)
    }
}
