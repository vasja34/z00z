use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use z00z_utils::{
    config::{from_yaml_value, YamlValue},
    io::load_yaml_bounded,
};

use crate::{
    config_name::{validate_domain_name, validate_underscore_name},
    AssetError,
};

const MAX_CONFIG_FILE_SIZE: u64 = 1024 * 1024;

const FORBIDDEN_RIGHT_KEYS: &[&str] = &[
    "fee",
    "fee_domain",
    "payer",
    "payer_commitment",
    "sponsor",
    "sponsor_commitment",
    "budget",
    "budget_units",
    "relay",
    "processing_support",
    "processing_support_ref",
    "support",
    "support_ref",
    "reserve",
    "amount",
    "nominal",
    "backing",
    "value",
];

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum RightClassConfig {
    MachineCapability,
    DataAccess,
    ServiceEntitlement,
    ValidatorMandate,
    OneTimeUse,
}

impl RightClassConfig {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MachineCapability => "machine_capability",
            Self::DataAccess => "data_access",
            Self::ServiceEntitlement => "service_entitlement",
            Self::ValidatorMandate => "validator_mandate",
            Self::OneTimeUse => "one_time_use",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RightsConfigEntry {
    pub id: String,
    pub right_class: RightClassConfig,
    pub issuer_scope: String,
    pub provider_scope: String,
    pub holder_fixture: String,
    pub control_fixture: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beneficiary_fixture: Option<String>,
    pub count: u32,
    pub domain_name: String,
    pub valid_from: u64,
    pub valid_until: u64,
    #[serde(default)]
    pub challenge_from: u64,
    #[serde(default)]
    pub challenge_until: u64,
    pub revocation_policy_id: String,
    pub transition_policy_id: String,
    pub challenge_policy_id: String,
    pub disclosure_policy_id: String,
    pub retention_policy_id: String,
    pub payload_commitment_seed: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, String>>,
}

impl RightsConfigEntry {
    pub fn validate(&self) -> Result<(), AssetError> {
        for (field_name, value) in [
            ("id", self.id.as_str()),
            ("issuer_scope", self.issuer_scope.as_str()),
            ("provider_scope", self.provider_scope.as_str()),
            ("holder_fixture", self.holder_fixture.as_str()),
            ("control_fixture", self.control_fixture.as_str()),
            ("domain_name", self.domain_name.as_str()),
            ("revocation_policy_id", self.revocation_policy_id.as_str()),
            ("transition_policy_id", self.transition_policy_id.as_str()),
            ("challenge_policy_id", self.challenge_policy_id.as_str()),
            ("disclosure_policy_id", self.disclosure_policy_id.as_str()),
            ("retention_policy_id", self.retention_policy_id.as_str()),
            (
                "payload_commitment_seed",
                self.payload_commitment_seed.as_str(),
            ),
        ] {
            if value.trim().is_empty() {
                return Err(AssetError::InvalidAsset(
                    format!("rights.{field_name} must not be empty").into(),
                ));
            }
        }

        if let Some(beneficiary_fixture) = self.beneficiary_fixture.as_deref() {
            if beneficiary_fixture.trim().is_empty() {
                return Err(AssetError::InvalidAsset(
                    "rights.beneficiary_fixture must not be empty when present".into(),
                ));
            }
            validate_underscore_name("rights.beneficiary_fixture", beneficiary_fixture)?;
        }

        validate_underscore_name("rights.id", self.id.as_str())?;
        validate_underscore_name("rights.issuer_scope", self.issuer_scope.as_str())?;
        validate_underscore_name("rights.provider_scope", self.provider_scope.as_str())?;
        validate_underscore_name("rights.holder_fixture", self.holder_fixture.as_str())?;
        validate_underscore_name("rights.control_fixture", self.control_fixture.as_str())?;
        validate_domain_name("rights.domain_name", self.domain_name.as_str())?;
        validate_underscore_name(
            "rights.revocation_policy_id",
            self.revocation_policy_id.as_str(),
        )?;
        validate_underscore_name(
            "rights.transition_policy_id",
            self.transition_policy_id.as_str(),
        )?;
        validate_underscore_name(
            "rights.challenge_policy_id",
            self.challenge_policy_id.as_str(),
        )?;
        validate_underscore_name(
            "rights.disclosure_policy_id",
            self.disclosure_policy_id.as_str(),
        )?;
        validate_underscore_name(
            "rights.retention_policy_id",
            self.retention_policy_id.as_str(),
        )?;
        validate_underscore_name(
            "rights.payload_commitment_seed",
            self.payload_commitment_seed.as_str(),
        )?;

        let purpose = self
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("purpose"))
            .map(String::as_str)
            .ok_or_else(|| {
                AssetError::InvalidAsset("rights.metadata.purpose must not be empty".into())
            })?;
        if purpose.trim().is_empty() {
            return Err(AssetError::InvalidAsset(
                "rights.metadata.purpose must not be empty".into(),
            ));
        }

        if self.count == 0 {
            return Err(AssetError::InvalidAsset(
                "rights.count must be greater than zero".into(),
            ));
        }

        if self.valid_until < self.valid_from {
            return Err(AssetError::InvalidAsset(
                "rights validity window is malformed".into(),
            ));
        }

        if (self.challenge_from != 0 || self.challenge_until != 0)
            && self.challenge_until < self.challenge_from
        {
            return Err(AssetError::InvalidAsset(
                "rights challenge window is malformed".into(),
            ));
        }

        if let Some(metadata) = &self.metadata {
            for (key, value) in metadata {
                if key.trim().is_empty() {
                    return Err(AssetError::InvalidMetadata(
                        "rights.metadata keys must not be empty".into(),
                    ));
                }
                if value.trim().is_empty() {
                    return Err(AssetError::InvalidMetadata(
                        format!("rights.metadata.{key} must not be empty").into(),
                    ));
                }
            }
        }

        Ok(())
    }
}

fn reject_forbidden_right_keys(right_yaml: &YamlValue) -> Result<(), AssetError> {
    let mapping = right_yaml
        .as_mapping()
        .ok_or_else(|| AssetError::InvalidAsset("rights entries must be mappings".into()))?;

    for key in mapping.keys() {
        let key = key
            .as_str()
            .ok_or_else(|| AssetError::InvalidAsset("rights keys must be strings".into()))?;
        if FORBIDDEN_RIGHT_KEYS.contains(&key) {
            return Err(AssetError::InvalidAsset(
                format!(
                    "rights.{key} is forbidden; fee and processing-support semantics must stay outside RightLeaf"
                )
                .into(),
            ));
        }
    }

    Ok(())
}

pub(crate) fn parse_rights_from_yaml(
    yaml: &YamlValue,
) -> Result<Vec<RightsConfigEntry>, AssetError> {
    let rights = yaml
        .get("rights")
        .and_then(|value| value.as_sequence())
        .ok_or_else(|| AssetError::InvalidAsset("Missing 'rights' array in config".into()))?;

    if rights.is_empty() {
        return Err(AssetError::InvalidAsset(
            "'rights' array in config must not be empty".into(),
        ));
    }

    let mut entries = Vec::with_capacity(rights.len());
    let mut seen_ids = BTreeSet::new();

    for right_yaml in rights {
        reject_forbidden_right_keys(right_yaml)?;

        let right: RightsConfigEntry = from_yaml_value(right_yaml.clone())?;
        right.validate()?;

        if !seen_ids.insert(right.id.clone()) {
            return Err(AssetError::InvalidAsset(
                format!("duplicate rights.id entry: {}", right.id).into(),
            ));
        }

        entries.push(right);
    }

    Ok(entries)
}

/// Canonical rights-config loader.
///
/// The semantic owner of rights-config parsing and validation lives under
/// `crate::rights`, and the canonical live fixture lives at
/// `configs/devnet_rights_config.yaml`.
pub fn load_rights_from_yaml(path: &Path) -> Result<Vec<RightsConfigEntry>, AssetError> {
    let yaml: YamlValue = load_yaml_bounded(path, MAX_CONFIG_FILE_SIZE)?;
    parse_rights_from_yaml(&yaml)
}
