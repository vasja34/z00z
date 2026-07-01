use std::path::PathBuf;

pub(crate) const DEFAULT_WALLET_CONFIG_YAML: &str = include_str!("wallet_config.yaml");

pub(crate) fn default_wallet_config_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("config")
        .join("wallet_config.yaml")
}
