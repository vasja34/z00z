//! RPC logging configuration loaded from the embedded wallet YAML config.

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use thiserror::Error;
use z00z_utils::config::{ConfigError, ConfigSource, EnvConfig, YamlConfig};
use z00z_utils::io;

use crate::config::{default_wallet_config_path, DEFAULT_WALLET_CONFIG_YAML};

#[derive(Debug, Error)]
pub enum RpcLogConfigError {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Io(#[from] z00z_utils::io::IoError),
}

#[derive(Debug, Clone)]
pub struct RpcLoggingRotationConfig {
    pub max_bytes: u64,
    pub keep_files: usize,
}

#[derive(Debug, Clone)]
pub struct RpcLoggingOutputConfig {
    pub path: String,
    pub rotation: RpcLoggingRotationConfig,
}

#[derive(Debug, Clone)]
pub struct RpcLoggingTruncationConfig {
    pub non_secret_min_bytes: usize,
    pub head_chars: usize,
    pub tail_chars: usize,
}

#[derive(Debug, Clone)]
pub struct RpcLoggingConfig {
    pub enabled: bool,
    pub level: String,
    pub output: RpcLoggingOutputConfig,
    pub max_line_bytes: usize,
    pub truncation: RpcLoggingTruncationConfig,
}

#[doc(hidden)]
pub struct WalletConfigEnvLock {
    _permit: tokio::sync::OwnedSemaphorePermit,
}

fn wallet_config_env_lock() -> Arc<tokio::sync::Semaphore> {
    static ENV_LOCK: OnceLock<Arc<tokio::sync::Semaphore>> = OnceLock::new();
    ENV_LOCK
        .get_or_init(|| Arc::new(tokio::sync::Semaphore::new(1)))
        .clone()
}

impl RpcLoggingConfig {
    pub fn from_embedded_wallet_yaml() -> Result<Self, RpcLogConfigError> {
        Self::from_wallet_yaml(DEFAULT_WALLET_CONFIG_YAML)
    }

    /// Load wallet YAML at runtime.
    ///
    /// Priority:
    /// 1) `Z00Z_WALLET_CONFIG_PATH`
    /// 2) `${CARGO_MANIFEST_DIR}/src/config/wallet_config.yaml`
    /// 3) Embedded default (compile-time)
    pub fn from_default_wallet_yaml() -> Result<Self, RpcLogConfigError> {
        let env = EnvConfig;
        if let Some(path) = env.get("Z00Z_WALLET_CONFIG_PATH")? {
            match io::read_to_string(PathBuf::from(path)) {
                Ok(contents) => return Self::from_wallet_yaml(&contents),
                Err(z00z_utils::io::IoError::Io(err))
                    if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => return Err(err.into()),
            }
        }

        let default_path = default_wallet_config_path();
        match io::read_to_string(&default_path) {
            Ok(contents) => return Self::from_wallet_yaml(&contents),
            Err(z00z_utils::io::IoError::Io(err)) if err.kind() == std::io::ErrorKind::NotFound => {
            }
            Err(err) => return Err(err.into()),
        }

        Self::from_embedded_wallet_yaml()
    }

    pub fn from_wallet_yaml_file(path: impl AsRef<Path>) -> Result<Self, RpcLogConfigError> {
        let contents = io::read_to_string(path)?;
        Self::from_wallet_yaml(&contents)
    }

    pub fn from_wallet_yaml(wallet_config_yaml: &str) -> Result<Self, RpcLogConfigError> {
        let default = Self::default_config();

        let config = YamlConfig::from_yaml_str(wallet_config_yaml)?;

        let enabled = config
            .get_typed::<bool>("wallet.logger.rpc.enabled")?
            .unwrap_or(default.enabled);

        let level = config
            .get("wallet.logger.rpc.level")?
            .unwrap_or(default.level);

        let output_path = config
            .get("wallet.logger.rpc.output.path")?
            .unwrap_or(default.output.path);

        let rotation_max_bytes = config
            .get_typed::<u64>("wallet.logger.rpc.output.rotation.max_bytes")?
            .unwrap_or(default.output.rotation.max_bytes);

        let rotation_keep_files = config
            .get_typed::<usize>("wallet.logger.rpc.output.rotation.keep_files")?
            .unwrap_or(default.output.rotation.keep_files);

        let max_line_bytes = config
            .get_typed::<usize>("wallet.logger.rpc.max_line_bytes")?
            .unwrap_or(default.max_line_bytes);

        let truncation_non_secret_min_bytes = config
            .get_typed::<usize>("wallet.logger.rpc.truncation.non_secret_min_bytes")?
            .unwrap_or(default.truncation.non_secret_min_bytes);

        let truncation_head_chars = config
            .get_typed::<usize>("wallet.logger.rpc.truncation.head_chars")?
            .unwrap_or(default.truncation.head_chars);

        let truncation_tail_chars = config
            .get_typed::<usize>("wallet.logger.rpc.truncation.tail_chars")?
            .unwrap_or(default.truncation.tail_chars);

        Ok(Self {
            enabled,
            level,
            output: RpcLoggingOutputConfig {
                path: output_path,
                rotation: RpcLoggingRotationConfig {
                    max_bytes: rotation_max_bytes,
                    keep_files: rotation_keep_files,
                },
            },
            max_line_bytes,
            truncation: RpcLoggingTruncationConfig {
                non_secret_min_bytes: truncation_non_secret_min_bytes,
                head_chars: truncation_head_chars,
                tail_chars: truncation_tail_chars,
            },
        })
    }

    fn default_config() -> Self {
        Self {
            enabled: false,
            level: "info".to_string(),
            output: RpcLoggingOutputConfig {
                path: "outputs/log/rpc_logger.log".to_string(),
                rotation: RpcLoggingRotationConfig {
                    max_bytes: 4 * 1024 * 1024,
                    keep_files: 3,
                },
            },
            max_line_bytes: 8192,
            truncation: RpcLoggingTruncationConfig {
                non_secret_min_bytes: 24,
                head_chars: 6,
                tail_chars: 6,
            },
        }
    }

    /// Global lock for tests that mutate `Z00Z_WALLET_CONFIG_PATH`.
    ///
    /// This env var is process-global; tests must serialize access.
    #[doc(hidden)]
    pub fn __lock_wallet_config_env() -> WalletConfigEnvLock {
        use std::time::Duration;

        let env_lock = wallet_config_env_lock();

        loop {
            if let Ok(permit) = env_lock.clone().try_acquire_owned() {
                return WalletConfigEnvLock { _permit: permit };
            }

            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// Async-aware global lock for tests that mutate `Z00Z_WALLET_CONFIG_PATH`.
    ///
    /// Async tests keep the same process-global semaphore so sync and async
    /// config-path tests cannot race each other.
    #[doc(hidden)]
    pub async fn __lock_wallet_config_env_async() -> WalletConfigEnvLock {
        let env_lock = wallet_config_env_lock();

        let permit = env_lock
            .acquire_owned()
            .await
            .expect("wallet config env lock must stay open");
        WalletConfigEnvLock { _permit: permit }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_yaml_is_error() {
        let err = RpcLoggingConfig::from_wallet_yaml("wallet: [")
            .expect_err("invalid wallet YAML must fail closed");
        assert!(matches!(
            err,
            RpcLogConfigError::Config(ConfigError::Yaml(_))
        ));
    }

    #[test]
    fn test_env_var_embedded_config() {
        let _guard = RpcLoggingConfig::__lock_wallet_config_env();

        let prev = std::env::var_os("Z00Z_WALLET_CONFIG_PATH");
        let dir = tempfile::tempdir().expect("tempdir must create");
        let cfg_path = dir.path().join("wallet_config.yaml");

        let yaml = r#"
wallet:
    logger:
        rpc:
            enabled: true
            level: "trace"
            max_line_bytes: 1234
            output:
                path: "outputs/log/test_rpc_logger.log"
                rotation:
                    max_bytes: 2048
                    keep_files: 1
"#;

        io::write_file(&cfg_path, yaml.as_bytes()).expect("write config must succeed");
        std::env::set_var("Z00Z_WALLET_CONFIG_PATH", &cfg_path);

        let cfg =
            RpcLoggingConfig::from_default_wallet_yaml().expect("valid override config must load");
        assert!(cfg.enabled);
        assert_eq!(cfg.level, "trace");
        assert_eq!(cfg.max_line_bytes, 1234);
        assert_eq!(cfg.output.rotation.max_bytes, 2048);
        assert_eq!(cfg.output.rotation.keep_files, 1);
        assert_eq!(cfg.output.path, "outputs/log/test_rpc_logger.log");

        match prev {
            Some(v) => std::env::set_var("Z00Z_WALLET_CONFIG_PATH", v),
            None => std::env::remove_var("Z00Z_WALLET_CONFIG_PATH"),
        }
    }

    #[test]
    fn test_missing_env_file_falls() {
        let _guard = RpcLoggingConfig::__lock_wallet_config_env();

        let prev = std::env::var_os("Z00Z_WALLET_CONFIG_PATH");
        let dir = tempfile::tempdir().expect("tempdir must create");
        let missing_path = dir.path().join("does_not_exist.yaml");

        let expected = RpcLoggingConfig::from_wallet_yaml(DEFAULT_WALLET_CONFIG_YAML)
            .expect("embedded wallet YAML must stay valid");

        std::env::set_var("Z00Z_WALLET_CONFIG_PATH", &missing_path);
        let actual = RpcLoggingConfig::from_default_wallet_yaml()
            .expect("missing override file must fall back to embedded config");

        assert_eq!(actual.enabled, expected.enabled);
        assert_eq!(actual.level, expected.level);
        assert_eq!(actual.output.path, expected.output.path);
        assert_eq!(
            actual.output.rotation.max_bytes,
            expected.output.rotation.max_bytes
        );
        assert_eq!(
            actual.output.rotation.keep_files,
            expected.output.rotation.keep_files
        );
        assert_eq!(actual.max_line_bytes, expected.max_line_bytes);
        assert_eq!(
            actual.truncation.non_secret_min_bytes,
            expected.truncation.non_secret_min_bytes
        );
        assert_eq!(actual.truncation.head_chars, expected.truncation.head_chars);
        assert_eq!(actual.truncation.tail_chars, expected.truncation.tail_chars);

        match prev {
            Some(v) => std::env::set_var("Z00Z_WALLET_CONFIG_PATH", v),
            None => std::env::remove_var("Z00Z_WALLET_CONFIG_PATH"),
        }
    }

    #[test]
    fn test_reads_rpc_logger_config() {
        let cfg = RpcLoggingConfig::from_wallet_yaml(
            r#"
wallet:
  logger:
    rpc:
      enabled: true
      level: "debug"
      output:
        path: "outputs/log/rpc_logger.log"
        rotation:
          max_bytes: 1024
          keep_files: 2
      max_line_bytes: 2048
      truncation:
        non_secret_min_bytes: 10
        head_chars: 3
        tail_chars: 4
"#,
        )
        .expect("valid RPC logger config must parse");

        assert!(cfg.enabled);
        assert_eq!(cfg.level, "debug");
        assert_eq!(cfg.output.rotation.max_bytes, 1024);
        assert_eq!(cfg.output.rotation.keep_files, 2);
        assert_eq!(cfg.max_line_bytes, 2048);
        assert_eq!(cfg.truncation.non_secret_min_bytes, 10);
        assert_eq!(cfg.truncation.head_chars, 3);
        assert_eq!(cfg.truncation.tail_chars, 4);
    }

    #[test]
    fn test_explicit_read_error_unsilenced() {
        let dir = tempfile::tempdir().expect("tempdir must create");
        let missing_path = dir.path().join("does_not_exist.yaml");

        let err = RpcLoggingConfig::from_wallet_yaml_file(&missing_path)
            .expect_err("explicit wallet YAML path must not fall back silently");

        assert!(matches!(
            err,
            RpcLogConfigError::Io(z00z_utils::io::IoError::Io(ref io_err))
                if io_err.kind() == std::io::ErrorKind::NotFound
        ));
    }
}
