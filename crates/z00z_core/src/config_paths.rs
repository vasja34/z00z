//! Canonical configuration paths for the live `z00z_core` YAML tree.
//!
//! Phase 063 keeps one live root under `configs/`. Callers should use
//! these helpers instead of hardcoding relative config paths, legacy
//! `src/genesis` / `src/assets` layouts, or example-local YAML.

use std::path::PathBuf;

pub const CORE_CONFIG_DIR: &str = "configs";

pub const DEVNET_GENESIS_CONFIG: &str = "devnet_genesis_config.yaml";
pub const DEVNET_ASSETS_CONFIG: &str = "devnet_assets_config.yaml";
pub const DEVNET_RIGHTS_CONFIG: &str = "devnet_rights_config.yaml";
pub const DEVNET_POLICIES_CONFIG: &str = "devnet_policies_config.yaml";
pub const DEVNET_VOUCHERS_CONFIG: &str = "devnet_vouchers_config.yaml";
pub const GENESIS_CONFIG_SCHEMA: &str = "schema_genesis_config.yaml";
pub const ASSETS_CONFIG_SCHEMA: &str = "schema_assets_config.yaml";
pub const RIGHTS_CONFIG_SCHEMA: &str = "schema_rights_config.yaml";
pub const POLICIES_CONFIG_SCHEMA: &str = "schema_policies_config.yaml";
pub const VOUCHERS_CONFIG_SCHEMA: &str = "schema_vouchers_config.yaml";

pub const DEVNET_GENESIS_CONFIG_REL: &str = "configs/devnet_genesis_config.yaml";
pub const DEVNET_ASSETS_CONFIG_REL: &str = "configs/devnet_assets_config.yaml";
pub const DEVNET_RIGHTS_CONFIG_REL: &str = "configs/devnet_rights_config.yaml";
pub const DEVNET_POLICIES_CONFIG_REL: &str = "configs/devnet_policies_config.yaml";
pub const DEVNET_VOUCHERS_CONFIG_REL: &str = "configs/devnet_vouchers_config.yaml";
pub const GENESIS_CONFIG_SCHEMA_REL: &str = "configs/schema_genesis_config.yaml";
pub const ASSETS_CONFIG_SCHEMA_REL: &str = "configs/schema_assets_config.yaml";
pub const RIGHTS_CONFIG_SCHEMA_REL: &str = "configs/schema_rights_config.yaml";
pub const POLICIES_CONFIG_SCHEMA_REL: &str = "configs/schema_policies_config.yaml";
pub const VOUCHERS_CONFIG_SCHEMA_REL: &str = "configs/schema_vouchers_config.yaml";

#[must_use]
pub fn core_config_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(CORE_CONFIG_DIR)
}

#[must_use]
pub fn core_config_path(file_name: &str) -> PathBuf {
    core_config_dir().join(file_name)
}

#[must_use]
pub fn devnet_genesis_path() -> PathBuf {
    core_config_path(DEVNET_GENESIS_CONFIG)
}

#[must_use]
pub fn devnet_assets_path() -> PathBuf {
    core_config_path(DEVNET_ASSETS_CONFIG)
}

#[must_use]
pub fn devnet_rights_path() -> PathBuf {
    core_config_path(DEVNET_RIGHTS_CONFIG)
}

#[must_use]
pub fn devnet_policies_path() -> PathBuf {
    core_config_path(DEVNET_POLICIES_CONFIG)
}

#[must_use]
pub fn devnet_vouchers_path() -> PathBuf {
    core_config_path(DEVNET_VOUCHERS_CONFIG)
}

pub fn genesis_schema_path() -> PathBuf {
    core_config_path(GENESIS_CONFIG_SCHEMA)
}

#[must_use]
pub fn assets_schema_path() -> PathBuf {
    core_config_path(ASSETS_CONFIG_SCHEMA)
}

#[must_use]
pub fn rights_schema_path() -> PathBuf {
    core_config_path(RIGHTS_CONFIG_SCHEMA)
}

#[must_use]
pub fn policies_schema_path() -> PathBuf {
    core_config_path(POLICIES_CONFIG_SCHEMA)
}

#[must_use]
pub fn vouchers_schema_path() -> PathBuf {
    core_config_path(VOUCHERS_CONFIG_SCHEMA)
}
