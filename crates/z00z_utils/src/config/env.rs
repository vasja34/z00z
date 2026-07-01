//! EnvConfig implementation

use crate::config::{ConfigError, ConfigSource};

/// Configuration source that reads from environment variables
///
/// # Examples
///
/// ```no_run
/// use z00z_utils::config::{ConfigSource, EnvConfig};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// let config = EnvConfig;
/// let port = config.get("PORT").ok().flatten();
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct EnvConfig;

impl ConfigSource for EnvConfig {
    type Error = ConfigError;

    fn get(&self, key: &str) -> Result<Option<String>, Self::Error> {
        match std::env::var(key) {
            Ok(value) => Ok(Some(value)),
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(std::env::VarError::NotUnicode(_)) => Err(ConfigError::Parse {
                key: key.to_string(),
                value: "<non-utf8-env>".to_string(),
                error: "environment variable is not valid UTF-8".to_string(),
            }),
        }
    }
}
