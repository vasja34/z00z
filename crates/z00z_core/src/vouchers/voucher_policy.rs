use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{rights::RightRequirementV1, AssetError};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoucherPolicyV1 {
    pub receiver_must_accept: bool,
    pub allow_transfer: bool,
    pub allow_partial_redeem: bool,
    pub allow_refund: bool,
    pub preserve_beneficiary_on_transfer: bool,
    pub preserve_refund_authority_on_transfer: bool,
    #[serde(default)]
    pub required_rights: BTreeSet<RightRequirementV1>,
}

impl VoucherPolicyV1 {
    pub fn validate(&self) -> Result<(), AssetError> {
        if !self.receiver_must_accept {
            return Err(AssetError::InvalidAsset(
                "voucher policy must require explicit receiver acceptance".into(),
            ));
        }

        if self.allow_transfer
            && (!self.preserve_beneficiary_on_transfer
                || !self.preserve_refund_authority_on_transfer)
        {
            return Err(AssetError::InvalidAsset(
                "voucher transfer must preserve beneficiary and refund authority".into(),
            ));
        }

        for right in &self.required_rights {
            right.validate()?;
        }

        Ok(())
    }
}
