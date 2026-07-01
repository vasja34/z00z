use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    config_name::{validate_domain_name, validate_underscore_name},
    AssetError, ObjectFamily,
};

use super::{RightActionV1, RightClassConfig};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightScopeV1 {
    ObjectFamily(ObjectFamily),
    SpecificObject(String),
    Policy(String),
    Domain(String),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RightRequirementV1 {
    pub right_policy: String,
    pub allowed_actions: BTreeSet<RightActionV1>,
    pub scope: RightScopeV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u32>,
    pub delegation_allowed: bool,
    pub attenuation_only: bool,
}

impl RightRequirementV1 {
    pub fn validate(&self) -> Result<(), AssetError> {
        if self.right_policy.trim().is_empty() {
            return Err(AssetError::InvalidAsset(
                "right requirement policy must not be empty".into(),
            ));
        }
        validate_underscore_name("right_requirement.right_policy", self.right_policy.as_str())?;

        if self.allowed_actions.is_empty() {
            return Err(AssetError::InvalidAsset(
                "right requirement must declare at least one action".into(),
            ));
        }

        if matches!(self.max_uses, Some(0)) {
            return Err(AssetError::InvalidAsset(
                "right requirement max_uses must be greater than zero".into(),
            ));
        }

        match &self.scope {
            RightScopeV1::SpecificObject(value) | RightScopeV1::Policy(value) => {
                if value.trim().is_empty() {
                    return Err(AssetError::InvalidAsset(
                        "right requirement scope must not be empty".into(),
                    ));
                }
                validate_underscore_name("right_requirement.scope", value)?;
            }
            RightScopeV1::Domain(value) => {
                if value.trim().is_empty() {
                    return Err(AssetError::InvalidAsset(
                        "right requirement scope must not be empty".into(),
                    ));
                }
                validate_domain_name("right_requirement.scope", value)?;
            }
            RightScopeV1::ObjectFamily(_) => {}
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RightPolicyV1 {
    pub right_class: RightClassConfig,
    pub allowed_actions: BTreeSet<RightActionV1>,
    pub delegation_allowed: bool,
    pub zero_value_only: bool,
}

impl RightPolicyV1 {
    pub fn validate(&self) -> Result<(), AssetError> {
        if self.allowed_actions.is_empty() {
            return Err(AssetError::InvalidAsset(
                "right policy must declare at least one action".into(),
            ));
        }

        if !self.zero_value_only {
            return Err(AssetError::InvalidAsset(
                "right policies must remain zero-value".into(),
            ));
        }

        Ok(())
    }
}
