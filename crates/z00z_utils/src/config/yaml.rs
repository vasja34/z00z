//! YamlConfig implementation

use crate::{
    config::{ConfigError, ConfigSource},
    io::read_file_bounded,
};
use std::path::Path;

/// Maximum supported YAML config size for `YamlConfig::from_file`.
pub const YAML_CONFIG_MAX_BYTES: u64 = 256 * 1024;

/// Configuration source that reads from YAML files
///
/// # Examples
///
/// ```no_run
/// use z00z_utils::config::{ConfigSource, YamlConfig};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// let config = YamlConfig::from_file("config.yaml")?;
/// let port = config.get("server.port").ok().flatten();
/// # Ok(())
/// # }
/// ```
pub struct YamlConfig {
    data: serde_yaml::Value,
}

impl YamlConfig {
    /// Load configuration from a YAML file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let bytes = read_file_bounded(path.as_ref(), YAML_CONFIG_MAX_BYTES)?;
        let content = String::from_utf8(bytes)
            .map_err(|err| ConfigError::Yaml(format!("invalid UTF-8 YAML: {err}")))?;
        let data = serde_yaml::from_str(&content).map_err(|e| ConfigError::Yaml(e.to_string()))?;
        Ok(Self { data })
    }

    /// Load configuration from an in-memory YAML document.
    pub fn from_yaml_str(yaml: &str) -> Result<Self, ConfigError> {
        let data = serde_yaml::from_str(yaml).map_err(|e| ConfigError::Yaml(e.to_string()))?;
        Ok(Self { data })
    }
}

impl std::str::FromStr for YamlConfig {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        YamlConfig::from_yaml_str(s)
    }
}

impl ConfigSource for YamlConfig {
    type Error = ConfigError;

    fn get(&self, key: &str) -> Result<Option<String>, Self::Error> {
        // Navigate YAML tree using dot-notation: "server.port"
        let keys: Vec<&str> = key.split('.').collect();
        let mut current = &self.data;

        for k in keys {
            current = match current.get(k) {
                Some(v) => v,
                None => return Ok(None),
            };
        }

        // Convert YAML value to string
        match current {
            serde_yaml::Value::String(s) => Ok(Some(s.clone())),
            serde_yaml::Value::Number(n) => Ok(Some(n.to_string())),
            serde_yaml::Value::Bool(b) => Ok(Some(b.to_string())),
            serde_yaml::Value::Null => Err(ConfigError::Parse {
                key: key.to_string(),
                value: "<null-yaml>".to_string(),
                error: "YAML leaf is null; expected a scalar value".to_string(),
            }),
            serde_yaml::Value::Sequence(_)
            | serde_yaml::Value::Mapping(_)
            | serde_yaml::Value::Tagged(_) => Err(ConfigError::Parse {
                key: key.to_string(),
                value: "<non-scalar-yaml>".to_string(),
                error: "YAML leaf is not a scalar value".to_string(),
            }),
        }
    }
}
