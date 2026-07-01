use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{config_name::validate_underscore_name, AssetError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionTrustTierV1 {
    Deterministic,
    VerifierAttested,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionKindV1 {
    TimeWindow,
    AcceptanceRequired,
    ReplayNonce,
    MaximumUses,
    RightScope,
    DisclosureCommitment,
    ExternalAttestation,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConditionDescriptorV1 {
    pub label: String,
    pub kind: ConditionKindV1,
    pub trust_tier: ConditionTrustTierV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verifier: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}

impl ConditionDescriptorV1 {
    pub fn validate(&self) -> Result<(), AssetError> {
        if self.label.trim().is_empty() {
            return Err(AssetError::InvalidAsset(
                "condition descriptor label must not be empty".into(),
            ));
        }
        validate_underscore_name("condition.label", self.label.as_str())?;

        match self.trust_tier {
            ConditionTrustTierV1::Deterministic => {
                if self.verifier.is_some() {
                    return Err(AssetError::InvalidAsset(
                        "deterministic conditions must not declare a verifier".into(),
                    ));
                }
            }
            ConditionTrustTierV1::VerifierAttested => {
                let verifier = self.verifier.as_deref().ok_or_else(|| {
                    AssetError::InvalidAsset(
                        "verifier-attested conditions must declare a verifier".into(),
                    )
                })?;
                if verifier.trim().is_empty() {
                    return Err(AssetError::InvalidAsset(
                        "condition verifier must not be empty".into(),
                    ));
                }
                validate_underscore_name("condition.verifier", verifier)?;
            }
        }

        for (key, value) in &self.metadata {
            if key.trim().is_empty() || value.trim().is_empty() {
                return Err(AssetError::InvalidMetadata(
                    "condition metadata must not contain empty keys or values".into(),
                ));
            }
        }

        Ok(())
    }
}
