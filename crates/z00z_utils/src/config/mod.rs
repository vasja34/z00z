//! Configuration abstraction
//!
//! This module provides a configuration source trait to enable dependency injection
//! of config implementations. It supports loading configuration from environment
//! variables, YAML files, and layered priority where the environment is checked
//! first with the same key string before YAML values are consulted.
//!
//! # Examples
//!
//! ```no_run
//! use z00z_utils::config::{ConfigSource, LayeredConfig};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//!
//! let config = LayeredConfig::with_yaml("/etc/z00z/config.yaml")?;
//! let port = config.get_typed::<u16>("server.port").ok().flatten().unwrap_or(8080);
//! let data_dir = config.get("server.data_dir").ok().flatten().unwrap_or("./data".to_string());
//! // The environment is checked first with the same key string.
//! # Ok(())
//! # }
//! ```

mod env;
mod layered;
mod traits;
mod yaml;

pub use env::EnvConfig;
pub use layered::LayeredConfig;
pub use traits::{ConfigError, ConfigSource};
pub use yaml::{YamlConfig, YAML_CONFIG_MAX_BYTES};

// Re-export serde_yaml::Value as YamlValue for consistency
// This provides a single source of truth for YAML value types across the codebase
pub use serde_yaml::Value as YamlValue;

/// Convert a YamlValue to a typed value
///
/// # Examples
///
/// ```
/// use z00z_utils::config::{YamlValue, from_yaml_value};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Config {
///     port: u16,
///     host: String,
/// }
///
/// let yaml = YamlValue::Null; // Example value
/// // let config: Config = from_yaml_value(yaml)?;
/// ```
pub fn from_yaml_value<T: serde::de::DeserializeOwned>(value: YamlValue) -> Result<T, ConfigError> {
    serde_yaml::from_value(value).map_err(|e| ConfigError::Yaml(e.to_string()))
}

#[cfg(test)]
mod test_config;
