//! Registry Catalog Parsing Helpers
//!
//! Internal helpers for translating registry YAML into wire-format asset
//! definitions. Public callers should enter through
//! `AssetDefinitionRegistry::load_catalog_from_yaml()`; this module stays owner-local
//! to `assets`.
//!
//! ## Design Principles
//!
//! - All orchestration stays in the registry owner path
//! - Parsing stays fail-closed and bounded
//! - Registry YAML is secondary data, not bootstrap authority
//!
//! ## Functions
//!
//! - [`parse_policy_flags`] - extract policy bits from YAML
//! - [`parse_asset_policy`] - extract `decimals`, `serials`, and `nominal`
//! - [`parse_asset_domain_name`] - extract top-level `domain_name`
//! - Registry catalog fields stay live as `decimals`, `serials`, `nominal`, and `domain_name`
//! - [`load_catalog_from_yaml`] - parse YAML into wire definitions
//! - [`compute_asset_id_from_catalog`] - deterministic ID generation
//!
//! ## Public Entry Example
//!
//! ```ignore
//! use std::path::Path;
//! use std::sync::Arc;
//! use z00z_core::assets::registry::AssetDefinitionRegistry;
//! use z00z_utils::prelude::{NoopLogger, NoopMetrics, SystemTimeProvider};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let registry = AssetDefinitionRegistry::load_catalog_from_yaml(
//!     Path::new("configs/devnet_assets_config.yaml"),
//!     Arc::new(NoopLogger),
//!     Arc::new(NoopMetrics),
//!     Arc::new(SystemTimeProvider),
//! )?;
//! assert!(registry.len()? > 0);
//! # Ok(())
//! # }
//! ```
//!
//! ## Catalog Format
//!
//! Expected YAML structure:
//!
//! ```yaml
//! assets:
//!   - name: "Z00Z Coin"
//!     symbol: "Z00Z"
//!     class: "coin"
//!     domain_name: "z00z.core.assets.native_coin.devnet.v1"
//!     policy:
//!       decimals: 8
//!       serials: 21000000
//!       nominal: 100000000
//!       flags:
//!         gas: true
//!         fungible: true
//!         mintable: false
//!         burnable: true
//! ```

use super::assets::{AssetClass, AssetError};
use super::definition::AssetDefinition;
use super::policy_flags::{BURNABLE, FUNGIBLE, GAS, MINTABLE};
use super::wire::DefinitionWire;
use crate::config_name::validate_domain_name;
use std::collections::BTreeMap;
use std::path::Path;
use z00z_utils::config::YamlValue;

const MAX_REGISTRY_CATALOG_FILE_SIZE: u64 = 1024 * 1024;

/// Parse policy flags from YAML configuration
///
/// Extracts and validates policy flags for a given asset class.
///
/// # Arguments
///
/// * `policy` - YAML value containing policy configuration
/// * `class` - Asset class for validation
///
/// # Returns
///
/// Policy flags as u8 bitmask
pub(crate) fn parse_policy_flags(policy: &YamlValue, class: AssetClass) -> Result<u8, AssetError> {
    let flags_section =
        match policy.get("flags") {
            Some(value) => Some(value.as_mapping().ok_or_else(|| {
                AssetError::InvalidAsset("policy.flags must be a mapping".into())
            })?),
            None => None,
        };

    if let Some(flags) = flags_section {
        for key in flags.keys() {
            let Some(key) = key.as_str() else {
                return Err(AssetError::InvalidAsset(
                    "policy.flags keys must be strings".into(),
                ));
            };
            if !matches!(key, "gas" | "fungible" | "mintable" | "burnable") {
                return Err(AssetError::InvalidAsset(
                    format!("unknown policy.flags key: {key}").into(),
                ));
            }
        }
    }

    let read_flag = |nested_key: &str, aliases: &[&str]| -> Result<Option<bool>, AssetError> {
        if let Some(flags) = flags_section {
            if let Some(value) = flags.get(YamlValue::String(nested_key.to_string())) {
                return value.as_bool().map(Some).ok_or_else(|| {
                    AssetError::InvalidAsset(
                        format!("policy.flags.{nested_key} must be a boolean").into(),
                    )
                });
            }
        }

        for alias in aliases {
            if let Some(value) = policy.get(*alias) {
                return value.as_bool().map(Some).ok_or_else(|| {
                    AssetError::InvalidAsset(format!("policy.{alias} must be a boolean").into())
                });
            }
        }

        Ok(None)
    };

    // Parse individual flags with backwards compatibility
    let gas = read_flag("gas", &["gas", "is_gas"])?.unwrap_or(false);
    let fungible = read_flag("fungible", &["fungible"])?
        .unwrap_or(matches!(class, AssetClass::Coin | AssetClass::Token));
    let mintable = read_flag("mintable", &["mintable", "is_mintable"])?.unwrap_or(false);
    let burnable = read_flag("burnable", &["burnable", "allow_burn"])?.unwrap_or(false);

    // Build flags bitfield
    // Bit 0: gas
    // Bit 1: fungible
    // Bit 2: mintable
    // Bit 4: burnable
    let mut flags = 0u8;
    if gas {
        flags |= GAS;
    }
    if fungible {
        flags |= FUNGIBLE;
    }
    if mintable {
        flags |= MINTABLE;
    }
    if burnable {
        flags |= BURNABLE;
    }

    Ok(flags)
}

pub(crate) fn has_policy_flag_overrides(policy: &YamlValue) -> Result<bool, AssetError> {
    if let Some(value) = policy.get("flags") {
        let mapping = value
            .as_mapping()
            .ok_or_else(|| AssetError::InvalidAsset("policy.flags must be a mapping".into()))?;
        for key in ["gas", "fungible", "mintable", "burnable"] {
            if mapping.contains_key(YamlValue::String(key.to_string())) {
                return Ok(true);
            }
        }
    }

    for key in [
        "gas",
        "is_gas",
        "fungible",
        "mintable",
        "is_mintable",
        "burnable",
        "allow_burn",
    ] {
        if policy.get(key).is_some() {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Parse asset policy configuration
///
/// Extracts decimals, serials, and nominal from YAML.
///
/// # Arguments
///
/// * `policy` - YAML value containing policy configuration
///
/// # Returns
///
/// Tuple of (decimals, serials, nominal)
pub fn parse_asset_policy(policy: &YamlValue) -> Result<(u8, u32, u64), AssetError> {
    let decimals = policy
        .get("decimals")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| AssetError::InvalidAsset("Missing 'decimals' in policy".into()))?
        .try_into()
        .map_err(|_| AssetError::InvalidAsset("'decimals' exceeds u8 range".into()))?;

    let serials = policy
        .get("serials")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| AssetError::InvalidAsset("Missing 'serials' in policy".into()))?
        .try_into()
        .map_err(|_| AssetError::InvalidAsset("'serials' exceeds u32 range".into()))?;

    let nominal = policy
        .get("nominal")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| AssetError::InvalidAsset("Missing 'nominal' in policy".into()))?;

    Ok((decimals, serials, nominal))
}

pub fn parse_asset_domain_name(asset_yaml: &YamlValue) -> Result<String, AssetError> {
    let domain_name = asset_yaml
        .get("domain_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AssetError::InvalidAsset("Missing 'domain_name' in asset".into()))?
        .to_string();
    validate_domain_name("asset.domain_name", domain_name.as_str())?;

    Ok(domain_name)
}

#[path = "registry_catalog_load.rs"]
mod registry_catalog_load;

pub(crate) use registry_catalog_load::load_catalog_from_yaml;
