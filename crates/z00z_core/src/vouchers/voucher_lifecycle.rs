use serde::{Deserialize, Serialize};

use crate::{config_name::validate_underscore_name, AssetError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoucherLifecycleV1 {
    PendingAcceptance,
    Active,
    PartiallyRedeemed,
    Redeemed,
    Rejected,
    Refunded,
    Expired,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoucherValidityWindowV1 {
    pub valid_from: u64,
    pub valid_until: u64,
}

impl VoucherValidityWindowV1 {
    pub fn validate(&self) -> Result<(), AssetError> {
        if self.valid_until < self.valid_from {
            return Err(AssetError::InvalidAsset(
                "voucher validity window is malformed".into(),
            ));
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoucherAcceptanceTermsV1 {
    pub receiver_must_accept: bool,
    pub allow_reject: bool,
    pub refund_target_fixture: String,
}

impl VoucherAcceptanceTermsV1 {
    pub fn validate(&self) -> Result<(), AssetError> {
        if !self.receiver_must_accept {
            return Err(AssetError::InvalidAsset(
                "voucher acceptance must stay explicit for the receiver".into(),
            ));
        }

        if self.refund_target_fixture.trim().is_empty() {
            return Err(AssetError::InvalidAsset(
                "voucher refund target must not be empty".into(),
            ));
        }
        validate_underscore_name(
            "voucher.refund_target_fixture",
            self.refund_target_fixture.as_str(),
        )?;

        Ok(())
    }
}
