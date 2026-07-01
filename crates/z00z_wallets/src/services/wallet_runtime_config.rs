use std::ffi::OsString;
use std::path::PathBuf;
use std::str::FromStr;
use z00z_core::ChainType;
use z00z_utils::codec::{Codec, YamlCodec};
use z00z_utils::config::{ConfigSource, EnvConfig, YamlValue};
use z00z_utils::io;

use crate::config::{default_wallet_config_path, DEFAULT_WALLET_CONFIG_YAML};
use crate::db::WalletIdentity;
use crate::receiver::{DEFAULT_CACHE_SIZE, MAX_CACHE_SIZE};
use crate::rpc::types::wallet::PersistWalletSettings;
use crate::security::password::RuntimePasswordPolicy;
use crate::wallet::{AutoLockPolicy, LockTrigger};
use crate::WalletError;
use crate::WalletResult;

fn env_value_os(key: &str) -> Option<OsString> {
    EnvConfig.get(key).ok().flatten().map(OsString::from)
}

fn env_value_string(key: &str) -> Option<String> {
    EnvConfig.get(key).ok().flatten()
}

fn wallet_yaml_override_best_effort() -> Option<String> {
    if let Some(path) =
        env_value_string("Z00Z_WALLET_CONFIG_PATH").filter(|value| !value.trim().is_empty())
    {
        if let Ok(contents) = io::read_to_string(PathBuf::from(path)) {
            return Some(contents);
        }
    }

    let default_path = default_wallet_config_path();
    io::read_to_string(&default_path).ok()
}

fn wallet_config_override_yaml_checked() -> WalletResult<Option<String>> {
    if let Some(path) =
        env_value_string("Z00Z_WALLET_CONFIG_PATH").filter(|value| !value.trim().is_empty())
    {
        return io::read_to_string(PathBuf::from(path))
            .map(Some)
            .map_err(|e| {
                WalletError::InvalidConfig(format!("failed to read Z00Z_WALLET_CONFIG_PATH: {e}"))
            });
    }

    let default_path = default_wallet_config_path();
    if io::path_exists(&default_path).map_err(|e| {
        WalletError::InvalidConfig(format!("failed to probe wallet_config.yaml: {e}"))
    })? {
        return io::read_to_string(&default_path).map(Some).map_err(|e| {
            WalletError::InvalidConfig(format!("failed to read wallet_config.yaml: {e}"))
        });
    }

    Ok(None)
}

fn merge_wallet_yaml_value(base: &mut YamlValue, overlay: YamlValue) {
    match (base, overlay) {
        (YamlValue::Mapping(base_map), YamlValue::Mapping(overlay_map)) => {
            for (key, overlay_value) in overlay_map {
                if let Some(base_value) = base_map.get_mut(&key) {
                    merge_wallet_yaml_value(base_value, overlay_value);
                } else {
                    base_map.insert(key, overlay_value);
                }
            }
        }
        (base_value, overlay_value) => {
            *base_value = overlay_value;
        }
    }
}

fn merge_wallet_config_yaml(base_yaml: &str, override_yaml: &str) -> WalletResult<String> {
    let mut merged: YamlValue = YamlCodec.deserialize(base_yaml.as_bytes()).map_err(|e| {
        WalletError::InvalidConfig(format!("invalid embedded wallet_config.yaml: {e}"))
    })?;
    let override_value: YamlValue = YamlCodec
        .deserialize(override_yaml.as_bytes())
        .map_err(|e| WalletError::InvalidConfig(format!("invalid wallet_config.yaml: {e}")))?;
    merge_wallet_yaml_value(&mut merged, override_value);
    String::from_utf8(YamlCodec.serialize(&merged).map_err(|e| {
        WalletError::InvalidConfig(format!("failed to serialize wallet_config.yaml: {e}"))
    })?)
    .map_err(|e| WalletError::InvalidConfig(format!("failed to serialize wallet_config.yaml: {e}")))
}

fn yaml_value_at_path<'a>(root: &'a YamlValue, key: &str) -> Option<&'a YamlValue> {
    let mut current = root;
    for part in key.split('.') {
        let map = match current {
            YamlValue::Mapping(map) => map,
            _ => return None,
        };
        current = map.get(YamlValue::String(part.to_string()))?;
    }
    Some(current)
}

fn optional_yaml_typed<T>(root: &YamlValue, key: &str) -> WalletResult<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    let Some(value) = yaml_value_at_path(root, key) else {
        return Ok(None);
    };
    let typed = z00z_utils::config::from_yaml_value::<T>(value.clone())
        .map_err(|e| WalletError::InvalidConfig(format!("invalid {key}: {e}")))?;
    Ok(Some(typed))
}

/// Resolve the wallet persistence output directory.
///
/// Priority:
/// 1) `Z00Z_WALLET_OUTPUT_DIR` environment variable
/// 2) `src/config/wallet_config.yaml` default config (embedded)
/// 3) Default: `outputs/wallets` (relative to current working directory)
pub(crate) fn resolve_wallet_output_dir() -> PathBuf {
    let yaml = load_wallet_config_yaml();
    resolve_output_dir_sources(&env_value_os, &yaml)
}

#[cfg(test)]
pub(crate) fn resolve_wallet_identity() -> WalletIdentity {
    let yaml = load_wallet_config_yaml();
    resolve_wallet_identity_sources(&env_value_os, &yaml)
}

pub(crate) fn resolve_wallet_identity_checked() -> WalletResult<WalletIdentity> {
    let config = load_wallet_config_checked()?;
    let configured_chain = config
        .get("wallet.chain.type")
        .map_err(|e| WalletError::InvalidConfig(format!("invalid wallet.chain.type: {e}")))?
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            config
                .get("wallet.chain.id")
                .ok()
                .flatten()
                .filter(|value| !value.trim().is_empty())
        })
        .ok_or_else(|| {
            WalletError::InvalidConfig(
                "missing required wallet config key: wallet.chain.type".to_string(),
            )
        })?;

    if let Some(value) = env_value_os("Z00Z_WALLET_NETWORK") {
        let network = value.to_string_lossy().trim().to_string();
        if network.is_empty() {
            return Err(WalletError::InvalidConfig(
                "Z00Z_WALLET_NETWORK must not be empty".to_string(),
            ));
        }

        let chain = env_value_os("Z00Z_WALLET_CHAIN")
            .map(|value| value.to_string_lossy().trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| configured_chain.clone());
        return Ok(WalletIdentity { network, chain });
    }

    let network = required_string(&config, "wallet.network.type")?;

    Ok(WalletIdentity {
        network,
        chain: configured_chain,
    })
}

pub(crate) fn resolve_wallet_chain_type_checked() -> Result<ChainType, WalletError> {
    let identity = resolve_wallet_identity_checked()?;
    wallet_identity_chain_type(&identity)
}

pub(crate) fn wallet_identity_chain_type(
    identity: &WalletIdentity,
) -> Result<ChainType, WalletError> {
    ChainType::from_str(identity.chain.as_str()).map_err(|err| {
        WalletError::InvalidConfig(format!("invalid wallet chain '{}': {err}", identity.chain))
    })
}

pub(crate) fn wallet_identity_chain_matches(
    expected: &WalletIdentity,
    actual: &WalletIdentity,
) -> WalletResult<()> {
    if expected.network != actual.network {
        return Err(WalletError::WalletNetworkMismatch {
            expected: expected.network.clone(),
            actual: actual.network.clone(),
        });
    }

    if expected.chain != actual.chain {
        return Err(WalletError::WalletChainMismatch {
            expected: expected.chain.clone(),
            actual: actual.chain.clone(),
        });
    }

    Ok(())
}

pub(crate) fn resolve_receiver_cache_size() -> Result<usize, WalletError> {
    let yaml = load_wallet_config_yaml_checked()?;
    resolve_receiver_cache_size_sources(&env_value_os, &yaml)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReceiverDeriveRateLimit {
    pub(crate) rate_per_sec: u32,
    pub(crate) burst: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WalletSettingsDefaults {
    pub(crate) auto_lock_timeout_secs: u64,
    pub(crate) default_fee: String,
    pub(crate) currency_display: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WalletBackupDefaults {
    pub(crate) auto_backup_enabled: bool,
    pub(crate) backup_interval_hours: u32,
    pub(crate) base_directory: PathBuf,
    pub(crate) encrypt_backups: bool,
    pub(crate) create_rate_limit_window_ms: u64,
}

pub(crate) fn resolve_receiver_derive_rate_limit(
) -> Result<Option<ReceiverDeriveRateLimit>, WalletError> {
    let yaml = load_wallet_config_yaml_checked()?;
    resolve_receiver_rate_limit(&env_value_os, &yaml)
}

pub(crate) fn resolve_wallet_settings_defaults() -> WalletResult<WalletSettingsDefaults> {
    let config = load_wallet_config_checked()?;
    let auto_lock_timeout_secs =
        required_typed::<u64>(&config, "wallet.settings.auto_lock_timeout_secs")?;
    if auto_lock_timeout_secs == 0 {
        return Err(WalletError::InvalidConfig(
            "wallet.settings.auto_lock_timeout_secs must be > 0".to_string(),
        ));
    }

    let default_fee = required_string(&config, "wallet.settings.default_fee")?;
    let currency_display = required_string(&config, "wallet.settings.currency_display")?;

    Ok(WalletSettingsDefaults {
        auto_lock_timeout_secs,
        default_fee,
        currency_display,
    })
}

pub(crate) fn resolve_wallet_settings_with_timestamps(
    now_ms: u64,
) -> WalletResult<PersistWalletSettings> {
    let defaults = resolve_wallet_settings_defaults()?;
    Ok(PersistWalletSettings {
        auto_lock_timeout: defaults.auto_lock_timeout_secs,
        default_fee: defaults.default_fee,
        currency_display: defaults.currency_display,
        policy_rules: None,
        created_at: now_ms,
        updated_at: now_ms,
    })
}

pub(crate) fn resolve_auto_lock_policy() -> WalletResult<AutoLockPolicy> {
    let config = load_wallet_config_checked()?;
    let value_root = load_wallet_config_value_checked()?;
    let settings_timeout =
        required_typed::<u64>(&config, "wallet.settings.auto_lock_timeout_secs")?;
    let policy_timeout = required_typed::<u64>(&config, "wallet.auto_lock.timeout_secs")?;
    if settings_timeout == 0 || policy_timeout == 0 {
        return Err(WalletError::InvalidConfig(
            "wallet auto-lock timeout must be > 0".to_string(),
        ));
    }
    if settings_timeout != policy_timeout {
        return Err(WalletError::InvalidConfig(
            "wallet.settings.auto_lock_timeout_secs must match wallet.auto_lock.timeout_secs"
                .to_string(),
        ));
    }

    let trigger_names =
        optional_yaml_typed::<Vec<String>>(&value_root, "wallet.auto_lock.triggers")?
            .unwrap_or_else(|| {
                vec![
                    "system_suspend".to_string(),
                    "screen_lock".to_string(),
                    "manual".to_string(),
                ]
            });

    let mut triggers = Vec::with_capacity(trigger_names.len());
    for name in trigger_names {
        let trigger = match name.trim() {
            "system_suspend" => LockTrigger::SystemSuspend,
            "screen_lock" => LockTrigger::ScreenLock,
            "app_backgrounded" => LockTrigger::AppBackgrounded,
            "manual" => LockTrigger::Manual,
            other => {
                return Err(WalletError::InvalidConfig(format!(
                    "invalid wallet.auto_lock.triggers entry: {other}"
                )));
            }
        };
        triggers.push(trigger);
    }

    Ok(AutoLockPolicy::new(
        std::time::Duration::from_secs(policy_timeout),
        triggers,
    ))
}

pub(crate) fn resolve_wallet_backup_defaults() -> WalletResult<WalletBackupDefaults> {
    let config = load_wallet_config_checked()?;
    let auto_backup_enabled = required_typed::<bool>(&config, "wallet.backup.auto_backup_enabled")?;
    let backup_interval_hours =
        required_typed::<u32>(&config, "wallet.backup.backup_interval_hours")?;
    if backup_interval_hours == 0 {
        return Err(WalletError::InvalidConfig(
            "wallet.backup.backup_interval_hours must be > 0".to_string(),
        ));
    }
    let base_directory = PathBuf::from(required_string(&config, "wallet.backup.location")?);
    let encrypt_backups = required_typed::<bool>(&config, "wallet.backup.encrypt_backups")?;
    let create_rate_limit_window_ms =
        required_typed::<u64>(&config, "wallet.backup.create_rate_limit_window_ms")?;
    if create_rate_limit_window_ms == 0 {
        return Err(WalletError::InvalidConfig(
            "wallet.backup.create_rate_limit_window_ms must be > 0".to_string(),
        ));
    }

    Ok(WalletBackupDefaults {
        auto_backup_enabled,
        backup_interval_hours,
        base_directory,
        encrypt_backups,
        create_rate_limit_window_ms,
    })
}

pub(crate) fn resolve_wallet_recovery_gap_limit() -> WalletResult<u32> {
    let config = load_wallet_config_checked()?;
    let gap_limit = required_typed::<u32>(&config, "wallet.recovery.gap_limit")?;
    if gap_limit == 0 {
        return Err(WalletError::InvalidConfig(
            "wallet.recovery.gap_limit must be > 0".to_string(),
        ));
    }
    Ok(gap_limit)
}

pub(crate) fn resolve_wallet_password_policy() -> WalletResult<RuntimePasswordPolicy> {
    let config = load_wallet_config_checked()?;
    let min_length =
        required_typed::<usize>(&config, "wallet.security.password_policy.min_length")?;
    let recommended_length = required_typed::<usize>(
        &config,
        "wallet.security.password_policy.recommended_length",
    )?;
    let max_length =
        required_typed::<usize>(&config, "wallet.security.password_policy.max_length")?;

    if min_length == 0 {
        return Err(WalletError::InvalidConfig(
            "wallet.security.password_policy.min_length must be > 0".to_string(),
        ));
    }
    if recommended_length < min_length {
        return Err(WalletError::InvalidConfig(
            "wallet.security.password_policy.recommended_length must be >= min_length".to_string(),
        ));
    }
    if max_length < recommended_length {
        return Err(WalletError::InvalidConfig(
            "wallet.security.password_policy.max_length must be >= recommended_length".to_string(),
        ));
    }

    Ok(RuntimePasswordPolicy {
        version: 1,
        min_length,
        recommended_length,
        max_length,
    })
}

fn load_wallet_config_yaml() -> String {
    if let Some(override_yaml) = wallet_yaml_override_best_effort() {
        if let Ok(merged_yaml) =
            merge_wallet_config_yaml(DEFAULT_WALLET_CONFIG_YAML, &override_yaml)
        {
            return merged_yaml;
        }
        return override_yaml;
    }

    DEFAULT_WALLET_CONFIG_YAML.to_string()
}

fn load_wallet_config_yaml_checked() -> WalletResult<String> {
    if let Some(override_yaml) = wallet_config_override_yaml_checked()? {
        return merge_wallet_config_yaml(DEFAULT_WALLET_CONFIG_YAML, &override_yaml);
    }

    Ok(DEFAULT_WALLET_CONFIG_YAML.to_string())
}

fn load_wallet_config_checked() -> WalletResult<z00z_utils::config::YamlConfig> {
    let yaml = load_wallet_config_yaml_checked()?;
    z00z_utils::config::YamlConfig::from_yaml_str(&yaml)
        .map_err(|e| WalletError::InvalidConfig(format!("invalid wallet_config.yaml: {e}")))
}

fn load_wallet_config_value_checked() -> WalletResult<YamlValue> {
    let yaml = load_wallet_config_yaml_checked()?;
    YamlCodec
        .deserialize(yaml.as_bytes())
        .map_err(|e| WalletError::InvalidConfig(format!("invalid wallet_config.yaml: {e}")))
}

fn required_string(config: &z00z_utils::config::YamlConfig, key: &str) -> WalletResult<String> {
    let value = config
        .get(key)
        .map_err(|e| WalletError::InvalidConfig(format!("invalid {key}: {e}")))?;
    let value = value.ok_or_else(|| {
        WalletError::InvalidConfig(format!("missing required wallet config key: {key}"))
    })?;
    if value.trim().is_empty() {
        return Err(WalletError::InvalidConfig(format!(
            "wallet config key must not be empty: {key}"
        )));
    }
    Ok(value)
}

fn required_typed<T>(config: &z00z_utils::config::YamlConfig, key: &str) -> WalletResult<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    config
        .get_typed::<T>(key)
        .map_err(|e| WalletError::InvalidConfig(format!("invalid {key}: {e}")))?
        .ok_or_else(|| {
            WalletError::InvalidConfig(format!("missing required wallet config key: {key}"))
        })
}

pub(crate) fn resolve_output_dir_sources(
    env: &dyn Fn(&str) -> Option<OsString>,
    wallet_config_yaml: &str,
) -> PathBuf {
    if let Some(value) = env("Z00Z_WALLET_OUTPUT_DIR") {
        let path = PathBuf::from(value);
        if !path.as_os_str().is_empty() {
            return path;
        }
    }

    if let Ok(config) = z00z_utils::config::YamlConfig::from_yaml_str(wallet_config_yaml) {
        if let Ok(Some(value)) = config.get("wallet.paths.output_dir") {
            if !value.trim().is_empty() {
                return PathBuf::from(value);
            }
        }
    }

    PathBuf::from("outputs").join("wallets")
}

#[cfg(test)]
pub(crate) fn resolve_wallet_identity_sources(
    env: &dyn Fn(&str) -> Option<OsString>,
    wallet_config_yaml: &str,
) -> WalletIdentity {
    if let Some(value) = env("Z00Z_WALLET_NETWORK") {
        let network = value.to_string_lossy().trim().to_string();
        if !network.is_empty() {
            let chain = env("Z00Z_WALLET_CHAIN")
                .map(|v| v.to_string_lossy().trim().to_string())
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| "devnet".to_string());

            return WalletIdentity { network, chain };
        }
    }

    if let Ok(config) = z00z_utils::config::YamlConfig::from_yaml_str(wallet_config_yaml) {
        let network = config
            .get("wallet.network.type")
            .ok()
            .flatten()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "p2p".to_string());

        let chain = config
            .get("wallet.chain.type")
            .ok()
            .flatten()
            .filter(|v| !v.trim().is_empty())
            .or_else(|| {
                config
                    .get("wallet.chain.id")
                    .ok()
                    .flatten()
                    .filter(|v| !v.trim().is_empty())
            })
            .unwrap_or_else(|| "devnet".to_string());

        return WalletIdentity { network, chain };
    }

    WalletIdentity {
        network: "p2p".to_string(),
        chain: "devnet".to_string(),
    }
}

pub(crate) fn resolve_receiver_cache_size_sources(
    env: &dyn Fn(&str) -> Option<OsString>,
    wallet_config_yaml: &str,
) -> Result<usize, WalletError> {
    if let Some(value) = env("Z00Z_WALLET_RECEIVER_CACHE_SIZE") {
        let raw = value.to_string_lossy();
        let parsed = raw.trim().parse::<usize>().map_err(|e| {
            WalletError::InvalidParams(format!(
                "Invalid Z00Z_WALLET_RECEIVER_CACHE_SIZE: expected usize: {e}"
            ))
        })?;
        return validate_receiver_cache_size(parsed);
    }

    let config = match z00z_utils::config::YamlConfig::from_yaml_str(wallet_config_yaml) {
        Ok(config) => config,
        Err(_) => return Ok(DEFAULT_CACHE_SIZE),
    };

    let size = config
        .get_typed::<usize>("wallet.receiver.cache_size")
        .ok()
        .flatten();

    match size {
        Some(value) => validate_receiver_cache_size(value),
        None => Ok(DEFAULT_CACHE_SIZE),
    }
}

pub(crate) fn resolve_receiver_rate_limit(
    env: &dyn Fn(&str) -> Option<OsString>,
    wallet_config_yaml: &str,
) -> Result<Option<ReceiverDeriveRateLimit>, WalletError> {
    let rate_env = env("Z00Z_WALLET_RECEIVER_DERIVE_RATE_PER_SEC")
        .map(|v| v.to_string_lossy().trim().to_string())
        .filter(|v| !v.is_empty());
    let burst_env = env("Z00Z_WALLET_RECEIVER_DERIVE_BURST")
        .map(|v| v.to_string_lossy().trim().to_string())
        .filter(|v| !v.is_empty());

    if rate_env.is_some() || burst_env.is_some() {
        let rate_raw = rate_env.ok_or_else(|| {
            WalletError::InvalidParams(
                "Invalid receiver derivation rate limit: missing Z00Z_WALLET_RECEIVER_DERIVE_RATE_PER_SEC"
                    .to_string(),
            )
        })?;
        let burst_raw = burst_env.ok_or_else(|| {
            WalletError::InvalidParams(
                "Invalid receiver derivation rate limit: missing Z00Z_WALLET_RECEIVER_DERIVE_BURST"
                    .to_string(),
            )
        })?;

        let rate_per_sec = rate_raw.parse::<u32>().map_err(|e| {
            WalletError::InvalidParams(format!(
                "Invalid Z00Z_WALLET_RECEIVER_DERIVE_RATE_PER_SEC: expected u32: {e}"
            ))
        })?;
        let burst = burst_raw.parse::<u32>().map_err(|e| {
            WalletError::InvalidParams(format!(
                "Invalid Z00Z_WALLET_RECEIVER_DERIVE_BURST: expected u32: {e}"
            ))
        })?;

        return validate_receiver_derive_rate_limit(rate_per_sec, burst).map(Some);
    }

    let config = match z00z_utils::config::YamlConfig::from_yaml_str(wallet_config_yaml) {
        Ok(config) => config,
        Err(_) => return Ok(None),
    };

    let enabled = config
        .get_typed::<bool>("wallet.receiver.rate_limit.enabled")
        .ok()
        .flatten();
    if enabled == Some(false) {
        return Ok(None);
    }

    let rate = config
        .get_typed::<u32>("wallet.receiver.rate_limit.rate_per_sec")
        .ok()
        .flatten();
    let burst = config
        .get_typed::<u32>("wallet.receiver.rate_limit.burst")
        .ok()
        .flatten();

    match (rate, burst) {
        (None, None) => Ok(None),
        (Some(rate_per_sec), Some(burst)) => {
            validate_receiver_derive_rate_limit(rate_per_sec, burst).map(Some)
        }
        _ => Err(WalletError::InvalidParams(
            "Invalid wallet.receiver.rate_limit: expected both rate_per_sec and burst".to_string(),
        )),
    }
}

fn validate_receiver_derive_rate_limit(
    rate_per_sec: u32,
    burst: u32,
) -> Result<ReceiverDeriveRateLimit, WalletError> {
    if rate_per_sec == 0 {
        return Err(WalletError::InvalidParams(
            "Invalid receiver derivation rate limit: rate_per_sec must be > 0".to_string(),
        ));
    }

    if burst == 0 {
        return Err(WalletError::InvalidParams(
            "Invalid receiver derivation rate limit: burst must be > 0".to_string(),
        ));
    }

    Ok(ReceiverDeriveRateLimit {
        rate_per_sec,
        burst,
    })
}

fn validate_receiver_cache_size(size: usize) -> Result<usize, WalletError> {
    if size == 0 {
        return Err(WalletError::InvalidParams(
            "Invalid wallet.receiver.cache_size: must be > 0".to_string(),
        ));
    }

    if size > MAX_CACHE_SIZE {
        return Err(WalletError::InvalidParams(format!(
            "Invalid wallet.receiver.cache_size: must be <= {MAX_CACHE_SIZE}"
        )));
    }

    Ok(size)
}

#[cfg(test)]
include!("test_wallet_runtime_config_suite.rs");
