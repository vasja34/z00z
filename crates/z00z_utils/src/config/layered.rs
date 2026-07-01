//! LayeredConfig implementation

use crate::config::{ConfigError, ConfigSource, EnvConfig, YamlConfig};
use std::io::ErrorKind;
use std::path::Path;

/// Layered configuration with priority: environment lookup for the same key string > YAML > none
///
/// This config source checks multiple sources in priority order:
/// 1. Environment lookup using the same key string (highest priority)
/// 2. YAML file (medium priority)
/// 3. None (if key not found)
///
/// # Examples
///
/// ```no_run
/// use z00z_utils::config::{ConfigSource, LayeredConfig};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// let config = LayeredConfig::with_yaml("/etc/z00z/config.yaml")?;
/// let port = config.get_typed::<u16>("server.port").ok().flatten().unwrap_or(8080);
/// // The environment is checked first with the same key string.
/// # Ok(())
/// # }
/// ```
pub struct LayeredConfig {
    env: EnvConfig,
    yaml: Option<YamlConfig>,
}

impl LayeredConfig {
    /// Create a fail-closed LayeredConfig from the current working directory's `config.yaml`.
    ///
    /// This is a convenience constructor for controlled CLI-style environments.
    /// Services and libraries should prefer [`LayeredConfig::with_yaml`] with an
    /// explicit trusted path.
    pub fn new() -> Result<Self, ConfigError> {
        Self::with_yaml("config.yaml")
    }

    /// Create a fail-closed LayeredConfig with an explicit YAML path.
    pub fn with_yaml(yaml_path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        Ok(Self {
            env: EnvConfig,
            yaml: Some(YamlConfig::from_file(yaml_path)?),
        })
    }

    /// Create a LayeredConfig that treats only missing YAML as an allowed downgrade.
    pub fn with_optional_yaml(yaml_path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        match YamlConfig::from_file(yaml_path.as_ref()) {
            Ok(yaml) => Ok(Self {
                env: EnvConfig,
                yaml: Some(yaml),
            }),
            Err(ConfigError::Io(err)) if err.kind() == ErrorKind::NotFound => Ok(Self::env_only()),
            Err(err) => Err(err),
        }
    }

    /// Create a LayeredConfig with only ENV variables (no YAML).
    pub fn env_only() -> Self {
        Self {
            env: EnvConfig,
            yaml: None,
        }
    }
}

impl ConfigSource for LayeredConfig {
    type Error = ConfigError;

    fn get(&self, key: &str) -> Result<Option<String>, Self::Error> {
        // Priority: ENV > YAML > None
        if let Some(value) = self.env.get(key)? {
            return Ok(Some(value));
        }

        if let Some(ref yaml) = self.yaml {
            if let Some(value) = yaml.get(key)? {
                return Ok(Some(value));
            }
        }

        Ok(None)
    }
}
