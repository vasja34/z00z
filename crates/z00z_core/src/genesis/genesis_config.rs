//! Genesis Configuration Module
//!
//! Defines configuration structures for genesis generation from YAML files.
//! Uses z00z_utils for consistent codec and I/O operations.
//! Validation logic moved to validator.rs module.

use crate::assets::{
    policy_flags::{BURNABLE, FUNGIBLE, GAS, MINTABLE},
    AssetClass, AssetError, ObjectFamily,
};
use crate::genesis::manifest_ref_loader;
use crate::genesis::validator::{validate_config_schema, GenesisError};
use crate::policies::PolicyConfigEntryV1;
use crate::rights::{parse_rights_from_yaml, RightsConfigEntry};
use crate::vouchers::VoucherBootstrapEntryV1;
use std::collections::BTreeMap;
use std::path::Path;

use z00z_utils::config::{from_yaml_value, YamlValue};
use z00z_utils::io::load_yaml_bounded;

pub(crate) const MAX_CONFIG_FILE_SIZE: u64 = 1024 * 1024;

/// Top-level genesis configuration loaded from YAML
///
/// Contains all parameters needed for genesis generation:
/// - Chain identification and cryptographic parameters
/// - Typed bootstrap entries for assets, rights, policies, and vouchers
/// - Wallet and policy profiles used by live bootstrap docs and fixtures
/// - Output file paths
///
/// # Example YAML Structure
///
/// ```yaml
/// chain:
///   id: 1
///   type: mainnet
///   name: "z00z-mainnet-1"
///   magic_bytes: [0x5A, 0x30, 0x30, 0x5A]
///   domains:
///     genesis_seed: [...]
/// assets: [...]
/// rights: [...]
/// policies: [...]
/// vouchers: [...]
/// outputs:
///   assets_export_path: "outputs/genesis/"
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenesisConfig {
    pub chain: ChainConfig,
    pub assets: Vec<AssetConfigEntry>,
    pub rights: Vec<RightsConfigEntry>,
    #[serde(default)]
    pub policies: Vec<PolicyConfigEntryV1>,
    #[serde(default)]
    pub vouchers: Vec<VoucherBootstrapEntryV1>,
    #[serde(default)]
    pub wallet_profiles: Vec<WalletProfileConfig>,
    #[serde(default)]
    pub policy_profiles: Vec<PolicyProfileConfig>,
    pub outputs: OutputsConfig,
    pub performance: PerformanceConfig,
}

/// Chain identification and cryptographic parameters
///
/// # Fields
///
/// - `id`: Chain identifier (1=mainnet, 2=testnet, etc.)
/// - `chain_type`: Chain type string parsed through `ChainType::from_str`
/// - `name`: Chain name / identifier (e.g. "z00z-mainnet-1")
/// - `magic_bytes`: 4-byte network magic for message framing
/// - `domains`: Domain-specific configuration including genesis seed
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChainConfig {
    pub id: u32,
    #[serde(rename = "type")]
    pub chain_type: String,
    pub name: String,
    pub magic_bytes: [u8; 4],
    pub domains: DomainsConfig,
}

/// Domain-specific configuration containing genesis seed
///
/// Currently holds the genesis seed. Additional live domain parameters must
/// extend this typed contract rather than introduce side YAML owner surfaces.
///
/// # Security
///
/// The `genesis_seed` MUST pass the explicit weak-seed policy and protected-network
/// checks enforced by `validator::validate_genesis_seed`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DomainsConfig {
    pub genesis_seed: [u8; 32],
}

/// Single asset definition from YAML configuration
///
/// Defines an asset class to be generated during genesis.
/// Parsed using same logic as `AssetDefinitionRegistry` for synchronization.
///
/// # Fields
///
/// - `id`: Unique asset identifier (e.g., "Z00Z", "ETH")
/// - `class`: Asset class (Coin, Token, Nft, Void)
/// - `name`: Human-readable name
/// - `symbol`: Trading symbol (defaults to `id` if not specified)
/// - `domain_name`: Owning domain identifier for the generated asset definition
/// - `policy`: Generation parameters (serials, nominal value, etc.)
/// - `metadata`: Optional key-value metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssetConfigEntry {
    pub id: String,
    pub class: AssetClass,
    pub name: String,
    pub symbol: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub domain_name: String,
    pub policy: PolicyConfig,
    pub metadata: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ProfileAnchor {
    One(String),
    Many(Vec<String>),
}

impl ProfileAnchor {
    fn validate(&self, field_name: &str) -> Result<(), AssetError> {
        match self {
            Self::One(value) => validate_non_empty(field_name, value),
            Self::Many(values) => {
                if values.is_empty() {
                    return Err(AssetError::InvalidAsset(
                        format!("{field_name} must not be empty").into(),
                    ));
                }

                for value in values {
                    validate_non_empty(field_name, value)?;
                }

                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileWindowConfig {
    pub valid_from: u64,
    pub valid_until: u64,
}

impl ProfileWindowConfig {
    fn validate(&self, field_name: &str) -> Result<(), AssetError> {
        if self.valid_until < self.valid_from {
            return Err(AssetError::InvalidAsset(
                format!("{field_name} is malformed").into(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WalletProfileConfig {
    pub id: String,
    pub object_family: ObjectFamily,
    pub live_anchor: ProfileAnchor,
    pub transitions: Vec<String>,
    pub fail_closed: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_anchor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backing_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transferability: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redeem_target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiry_rule: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disclosure_policy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention_policy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beneficiary_scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_trail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub challenge_window: Option<String>,
    #[serde(default)]
    pub action_whitelist: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locked_asset_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locked_amount: Option<String>,
    #[serde(default)]
    pub binding_surfaces: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_binding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lock_window: Option<ProfileWindowConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoke_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ordinary_spend: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accept_policy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partial_redeem: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub residual_policy: Option<String>,
}

impl WalletProfileConfig {
    pub fn validate(&self) -> Result<(), AssetError> {
        validate_non_empty("wallet_profiles.id", &self.id)?;
        self.live_anchor.validate("wallet_profiles.live_anchor")?;
        validate_string_list("wallet_profiles.transitions", &self.transitions)?;
        validate_string_list("wallet_profiles.fail_closed", &self.fail_closed)?;
        validate_opt_string("wallet_profiles.product_anchor", &self.product_anchor)?;
        validate_opt_string("wallet_profiles.backing_kind", &self.backing_kind)?;
        validate_opt_string("wallet_profiles.transferability", &self.transferability)?;
        validate_opt_string("wallet_profiles.redeem_target", &self.redeem_target)?;
        validate_opt_string("wallet_profiles.expiry_rule", &self.expiry_rule)?;
        validate_opt_string("wallet_profiles.disclosure_policy", &self.disclosure_policy)?;
        validate_opt_string("wallet_profiles.retention_policy", &self.retention_policy)?;
        validate_opt_string("wallet_profiles.provider_scope", &self.provider_scope)?;
        validate_opt_string("wallet_profiles.beneficiary_scope", &self.beneficiary_scope)?;
        validate_opt_string("wallet_profiles.audit_trail", &self.audit_trail)?;
        validate_opt_string("wallet_profiles.challenge_window", &self.challenge_window)?;
        validate_string_list("wallet_profiles.action_whitelist", &self.action_whitelist)?;
        validate_opt_string("wallet_profiles.quota", &self.quota)?;
        validate_opt_string("wallet_profiles.service_scope", &self.service_scope)?;
        validate_opt_string("wallet_profiles.replay_mode", &self.replay_mode)?;
        validate_opt_string("wallet_profiles.locked_asset_id", &self.locked_asset_id)?;
        validate_opt_string("wallet_profiles.locked_amount", &self.locked_amount)?;
        validate_string_list("wallet_profiles.binding_surfaces", &self.binding_surfaces)?;
        validate_opt_string("wallet_profiles.payload_binding", &self.payload_binding)?;
        if let Some(lock_window) = &self.lock_window {
            lock_window.validate("wallet_profiles.lock_window")?;
        }
        validate_opt_string("wallet_profiles.revoke_mode", &self.revoke_mode)?;
        validate_opt_string("wallet_profiles.ordinary_spend", &self.ordinary_spend)?;
        validate_opt_string("wallet_profiles.accept_policy", &self.accept_policy)?;
        validate_opt_string("wallet_profiles.partial_redeem", &self.partial_redeem)?;
        validate_opt_string("wallet_profiles.residual_policy", &self.residual_policy)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyProfileConfig {
    pub id: String,
    #[serde(default)]
    pub enforced_actions: Vec<String>,
    #[serde(default)]
    pub selected_fields: Vec<String>,
    #[serde(default)]
    pub require_purpose: bool,
    #[serde(default)]
    pub require_expiry: bool,
    #[serde(default)]
    pub bind_policy_id: bool,
    #[serde(default)]
    pub bind_checkpoint_anchor: bool,
    #[serde(default)]
    pub bind_retained_document_hash: bool,
    #[serde(default)]
    pub disclosure_receipt_required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retained_object_type: Option<String>,
    #[serde(default)]
    pub required_bindings: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention_years: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disclosure_receipt_type: Option<String>,
    #[serde(default)]
    pub applies_to_profiles: Vec<String>,
}

impl PolicyProfileConfig {
    pub fn validate(&self) -> Result<(), AssetError> {
        validate_non_empty("policy_profiles.id", &self.id)?;
        validate_string_list("policy_profiles.enforced_actions", &self.enforced_actions)?;
        validate_string_list("policy_profiles.selected_fields", &self.selected_fields)?;
        validate_opt_string("policy_profiles.retention_profile", &self.retention_profile)?;
        validate_opt_string(
            "policy_profiles.retained_object_type",
            &self.retained_object_type,
        )?;
        validate_string_list("policy_profiles.required_bindings", &self.required_bindings)?;
        if self.retention_years == Some(0) {
            return Err(AssetError::InvalidAsset(
                "policy_profiles.retention_years must be greater than zero".into(),
            ));
        }
        validate_opt_string(
            "policy_profiles.disclosure_receipt_type",
            &self.disclosure_receipt_type,
        )?;
        validate_string_list(
            "policy_profiles.applies_to_profiles",
            &self.applies_to_profiles,
        )?;
        Ok(())
    }
}

fn validate_non_empty(field_name: &str, value: &str) -> Result<(), AssetError> {
    if value.trim().is_empty() {
        return Err(AssetError::InvalidAsset(
            format!("{field_name} must not be empty").into(),
        ));
    }

    Ok(())
}

fn validate_opt_string(field_name: &str, value: &Option<String>) -> Result<(), AssetError> {
    if let Some(value) = value {
        validate_non_empty(field_name, value)?;
    }

    Ok(())
}

fn validate_string_list(field_name: &str, values: &[String]) -> Result<(), AssetError> {
    for value in values {
        validate_non_empty(field_name, value)?;
    }

    Ok(())
}

/// Asset policy parameters for genesis generation
///
/// # Fields
///
/// - `decimals`: Number of decimal places (e.g., 8 for Bitcoin-like)
/// - `serials`: Number of asset instances to generate
/// - `nominal`: Nominal value per asset unit (in smallest denomination)
/// # Example
///
/// For 1000 coins of 1.0 Z00Z with 8 decimals:
/// ```yaml
/// domain_name: "z00z.core.assets.coin.devnet.v1"
/// policy:
///   decimals: 8
///   serials: 1000
///   nominal: 100000000  # 1.0 * 10^8
/// ```
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PolicyConfig {
    pub decimals: u8,
    pub serials: u32,
    pub nominal: u64,
    #[serde(default)]
    pub policy_flags: u8,
    #[serde(default)]
    pub has_policy_flags: bool,
    #[serde(default)]
    pub allow_burn: bool,
    #[serde(default)]
    pub is_gas: bool,
    #[serde(default)]
    pub is_mintable: bool,
}

impl PolicyConfig {
    pub fn asset_flags(&self, class: AssetClass) -> u8 {
        if self.has_policy_flags {
            return self.policy_flags;
        }

        let mut flags = 0u8;
        if self.is_gas {
            flags |= GAS;
        }
        if matches!(class, AssetClass::Coin | AssetClass::Token) {
            flags |= FUNGIBLE;
        }
        if self.is_mintable {
            flags |= MINTABLE;
        }
        if self.allow_burn {
            flags |= BURNABLE;
        }
        flags
    }
}

/// Output file paths for generated genesis data
///
/// # Fields
///
/// - `assets_export_path`: Base directory for timestamped typed artifact exports
/// - `snapshot_export_path`: Directory for snapshot ZIP archives
/// - `logging_path`: Directory for log files and reports
///
/// Generated files:
/// - `{assets_export_path}/genesis_{network}_{timestamp}/genesis_{SYMBOL}.json` - Human-readable JSON
/// - `{assets_export_path}/genesis_{network}_{timestamp}/genesis_{SYMBOL}.bin` - Compact bincode format
/// - `{assets_export_path}/genesis_{network}_{timestamp}/genesis_rights.json` - Typed right artifacts
/// - `{assets_export_path}/genesis_{network}_{timestamp}/genesis_policies.json` - Typed policy artifacts
/// - `{assets_export_path}/genesis_{network}_{timestamp}/genesis_vouchers.json` - Typed voucher artifacts
/// - `{assets_export_path}/genesis_{network}_{timestamp}/genesis_settlement_manifest.json` - Cross-family summary manifest
/// - `{logging_path}/genesis_generation_{timestamp}.log` - Generation log
/// - `{assets_export_path}/genesis_{network}_{timestamp}/genesis_report_{timestamp}.txt` - Generation report
/// - `{snapshot_export_path}/genesis_snapshot_{network}_{timestamp}.zip` - Snapshot archive
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OutputsConfig {
    pub assets_export_path: String,
    pub snapshot_export_path: String,
    #[serde(default = "default_logging_path")]
    pub logging_path: String,
}

fn default_logging_path() -> String {
    "crates/z00z_core/outputs/log/".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PerformanceConfig {
    pub num_threads: ThreadCountConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ThreadCountConfig {
    Named(ThreadCountMode),
    Fixed(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreadCountMode {
    Auto,
}

impl PerformanceConfig {
    #[must_use]
    pub fn resolved_num_threads(&self) -> usize {
        self.num_threads.resolved_threads()
    }
}

impl ThreadCountConfig {
    #[must_use]
    pub fn resolved_threads(&self) -> usize {
        match self {
            Self::Named(ThreadCountMode::Auto) => std::thread::available_parallelism()
                .map(std::num::NonZeroUsize::get)
                .unwrap_or(1),
            Self::Fixed(threads) => (*threads as usize).max(1),
        }
    }

    #[must_use]
    pub fn configured_label(&self) -> String {
        match self {
            Self::Named(ThreadCountMode::Auto) => "auto".to_string(),
            Self::Fixed(threads) => threads.to_string(),
        }
    }
}

/// Load genesis config from YAML file
///
/// Uses SAME parsing logic as AssetDefinitionRegistry for "assets:" section.
/// This ensures synchronous parsing between genesis and assets modules.
///
/// Supports both self-contained manifests and canonical root manifests with
/// `manifest_refs` subfiles that rehydrate into the same `GenesisConfig`.
pub fn load_genesis_config(path: &str) -> Result<GenesisConfig, GenesisError> {
    if let Some(config) = manifest_ref_loader::load_ref_config(Path::new(path))? {
        return Ok(config);
    }

    let yaml: YamlValue = load_yaml_bounded(path, MAX_CONFIG_FILE_SIZE)
        .map_err(|e| GenesisError::ConfigLoadFailed(e.to_string()))?;
    parse_genesis_from_yaml(&yaml)
}

/// Parse genesis config from YAML value
///
/// CRITICAL: This function uses EXACT SAME parsing logic as AssetDefinitionRegistry
/// for the "assets:" section to ensure synchronization.
fn parse_genesis_from_yaml(yaml: &YamlValue) -> Result<GenesisConfig, GenesisError> {
    validate_canonical_root_keys(yaml)?;
    let assets = parse_assets_from_yaml(yaml)?;
    let (chain, outputs, performance) = parse_core_sections(yaml)?;
    let rights =
        parse_rights_from_yaml(yaml).map_err(|e| GenesisError::ConfigParseFailed(e.to_string()))?;
    let policies: Vec<PolicyConfigEntryV1> = yaml
        .get("policies")
        .map(|value| {
            from_yaml_value(value.clone())
                .map_err(|e| GenesisError::ConfigParseFailed(e.to_string()))
        })
        .transpose()?
        .unwrap_or_default();
    let vouchers: Vec<VoucherBootstrapEntryV1> = yaml
        .get("vouchers")
        .map(|value| {
            from_yaml_value(value.clone())
                .map_err(|e| GenesisError::ConfigParseFailed(e.to_string()))
        })
        .transpose()?
        .unwrap_or_default();
    let wallet_profiles: Vec<WalletProfileConfig> = yaml
        .get("wallet_profiles")
        .map(|value| {
            from_yaml_value(value.clone())
                .map_err(|e| GenesisError::ConfigParseFailed(e.to_string()))
        })
        .transpose()?
        .unwrap_or_default();
    let policy_profiles: Vec<PolicyProfileConfig> = yaml
        .get("policy_profiles")
        .map(|value| {
            from_yaml_value(value.clone())
                .map_err(|e| GenesisError::ConfigParseFailed(e.to_string()))
        })
        .transpose()?
        .unwrap_or_default();

    build_config(
        chain,
        assets,
        rights,
        policies,
        vouchers,
        wallet_profiles,
        policy_profiles,
        outputs,
        performance,
    )
}

fn validate_canonical_root_keys(yaml: &YamlValue) -> Result<(), GenesisError> {
    let Some(mapping) = yaml.as_mapping() else {
        return Err(GenesisError::ConfigParseFailed(
            "genesis config must be a top-level mapping".to_string(),
        ));
    };

    for (deprecated, canonical) in [("network", "chain"), ("refs", "manifest_refs")] {
        let deprecated_key = YamlValue::String(deprecated.to_string());
        if mapping.contains_key(&deprecated_key) {
            return Err(GenesisError::ConfigParseFailed(format!(
                "genesis config uses deprecated top-level key {deprecated}; use {canonical}"
            )));
        }
    }

    Ok(())
}

pub(crate) fn parse_assets_from_yaml(
    yaml: &YamlValue,
) -> Result<Vec<AssetConfigEntry>, GenesisError> {
    use crate::AssetError;

    // Parse assets section with the same field semantics as the secondary
    // registry-catalog loader, without treating it as a bootstrap authority.
    let assets_array = yaml
        .get("assets")
        .and_then(|v| v.as_sequence())
        .ok_or_else(|| {
            GenesisError::ConfigParseFailed("Missing 'assets' array in config".into())
        })?;

    let mut assets = Vec::new();

    for asset_yaml in assets_array {
        // Extract fields exactly as in the secondary registry-catalog lane.
        let id_str = asset_yaml
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| GenesisError::ConfigParseFailed("Missing 'id' field".into()))?;

        let symbol = asset_yaml
            .get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or(id_str)
            .to_string();

        let name = asset_yaml
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| GenesisError::ConfigParseFailed("Missing 'name' field".into()))?
            .to_string();
        let description = asset_yaml
            .get("description")
            .map(|value| {
                value.as_str().map(str::to_string).ok_or_else(|| {
                    GenesisError::ConfigParseFailed("description must be a string".into())
                })
            })
            .transpose()?;

        let class_str = asset_yaml
            .get("class")
            .and_then(|v| v.as_str())
            .ok_or_else(|| GenesisError::ConfigParseFailed("Missing 'class' field".into()))?;

        let class = match class_str.to_lowercase().as_str() {
            "coin" => AssetClass::Coin,
            "token" => AssetClass::Token,
            "nft" => AssetClass::Nft,
            "void" => AssetClass::Void,
            _ => {
                return Err(GenesisError::ConfigParseFailed(format!(
                    "Unknown asset class: {}",
                    class_str
                )))
            }
        };

        // Extract policy using assets::parse_asset_policy helper
        let policy = asset_yaml
            .get("policy")
            .ok_or_else(|| GenesisError::ConfigParseFailed("Missing 'policy' field".into()))?;

        let (decimals, serials, nominal) = crate::assets::parse_asset_policy(policy)
            .map_err(|e: AssetError| GenesisError::ConfigParseFailed(e.to_string()))?;
        let domain_name = crate::assets::parse_asset_domain_name(asset_yaml)
            .map_err(|e: AssetError| GenesisError::ConfigParseFailed(e.to_string()))?;
        let policy_flags = crate::assets::parse_policy_flags(policy, class)
            .map_err(|e: AssetError| GenesisError::ConfigParseFailed(e.to_string()))?;
        let has_policy_flags = crate::assets::has_policy_flag_overrides(policy)
            .map_err(|e: AssetError| GenesisError::ConfigParseFailed(e.to_string()))?;

        // Parse metadata (optional) and fail closed on malformed identity-bearing fields.
        let metadata = if let Some(value) = asset_yaml.get("metadata") {
            let mapping = value.as_mapping().ok_or_else(|| {
                GenesisError::ConfigParseFailed("metadata must be a mapping".into())
            })?;
            let mut parsed = BTreeMap::new();
            for (key, value) in mapping {
                let key = key.as_str().ok_or_else(|| {
                    GenesisError::ConfigParseFailed("metadata keys must be strings".into())
                })?;
                let value = value.as_str().ok_or_else(|| {
                    GenesisError::ConfigParseFailed("metadata values must be strings".into())
                })?;
                parsed.insert(key.to_string(), value.to_string());
            }
            Some(parsed)
        } else {
            None
        };

        // Create AssetConfigEntry with parsed data
        let asset_config = AssetConfigEntry {
            id: id_str.to_string(),
            class,
            name,
            symbol,
            description,
            domain_name,
            policy: PolicyConfig {
                decimals,
                serials,
                nominal,
                policy_flags,
                has_policy_flags,
                allow_burn: policy_flags & BURNABLE != 0,
                is_gas: policy_flags & GAS != 0,
                is_mintable: policy_flags & MINTABLE != 0,
            },
            metadata,
        };

        assets.push(asset_config);
    }

    Ok(assets)
}

pub(crate) fn parse_core_sections(
    yaml: &YamlValue,
) -> Result<(ChainConfig, OutputsConfig, PerformanceConfig), GenesisError> {
    let chain_yaml = yaml
        .get("chain")
        .ok_or_else(|| GenesisError::ConfigParseFailed("Missing 'chain' section".into()))?;

    let chain: ChainConfig = from_yaml_value(chain_yaml.clone())
        .map_err(|e| GenesisError::ConfigParseFailed(e.to_string()))?;

    let outputs: OutputsConfig = from_yaml_value(
        yaml.get("outputs")
            .ok_or_else(|| GenesisError::ConfigParseFailed("Missing 'outputs' section".into()))?
            .clone(),
    )
    .map_err(|e| GenesisError::ConfigParseFailed(e.to_string()))?;

    let performance: PerformanceConfig = from_yaml_value(
        yaml.get("performance")
            .ok_or_else(|| GenesisError::ConfigParseFailed("Missing 'performance' section".into()))?
            .clone(),
    )
    .map_err(|e| GenesisError::ConfigParseFailed(e.to_string()))?;

    Ok((chain, outputs, performance))
}

pub(crate) fn build_config(
    chain: ChainConfig,
    assets: Vec<AssetConfigEntry>,
    rights: Vec<RightsConfigEntry>,
    policies: Vec<PolicyConfigEntryV1>,
    vouchers: Vec<VoucherBootstrapEntryV1>,
    wallet_profiles: Vec<WalletProfileConfig>,
    policy_profiles: Vec<PolicyProfileConfig>,
    outputs: OutputsConfig,
    performance: PerformanceConfig,
) -> Result<GenesisConfig, GenesisError> {
    let config = GenesisConfig {
        chain,
        assets,
        rights,
        policies,
        vouchers,
        wallet_profiles,
        policy_profiles,
        outputs,
        performance,
    };

    validate_config_schema(&config)?;

    Ok(config)
}
