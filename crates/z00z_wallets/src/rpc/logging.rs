//! Wallet RPC logging middleware.
//!
//! This module implements a transport wrapper that emits privacy-preserving JSONL
//! records for each RPC call.

#[path = "logging_config.rs"]
mod config;
#[path = "logging_middleware.rs"]
mod middleware;
#[path = "logging_policy.rs"]
mod policy;
#[path = "logging_record.rs"]
mod record;
#[path = "logging_summary.rs"]
mod summarize;

#[cfg(test)]
pub use config::WalletConfigEnvLock;
pub use config::{
    RpcLogConfigError, RpcLoggingConfig, RpcLoggingOutputConfig, RpcLoggingRotationConfig,
    RpcLoggingTruncationConfig,
};
pub use middleware::LoggedRpcTransport;
pub use record::{to_audit_entry, RpcLogRecord};

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;
use z00z_networks_rpc::RpcTransport;
use z00z_utils::logger::{Logger, RotatingFileLogger, RotationPolicy};
use z00z_utils::rng::SecureRngProvider;
use z00z_utils::time::TimeProvider;

#[derive(Debug, Error)]
pub enum RpcLogInitError {
    #[error(transparent)]
    Config(#[from] RpcLogConfigError),

    #[error("failed to initialize RPC log sink at {path}: {source}")]
    Sink {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

fn normalize_configured_path(path: &str) -> PathBuf {
    let configured = PathBuf::from(path);
    if configured.is_absolute() {
        return configured;
    }

    let prefix = PathBuf::from("crates").join("z00z_wallets");
    if configured.starts_with(&prefix) {
        if let Ok(cwd) = std::env::current_dir() {
            if cwd.ends_with(&prefix) {
                if let Ok(stripped) = configured.strip_prefix(&prefix) {
                    return stripped.to_path_buf();
                }
            }
        }
    }

    configured
}

/// Guard that prevents accidental double-wrapping of a transport with the logging middleware.
///
/// This avoids emitting duplicate JSONL records when a composition root calls the wrapper helper
/// multiple times.
pub struct RpcLoggingInstaller {
    installed: AtomicBool,
}

impl RpcLoggingInstaller {
    pub fn new() -> Self {
        Self {
            installed: AtomicBool::new(false),
        }
    }

    pub fn maybe_wrap_transport<T, R>(
        &self,
        transport: T,
        config: RpcLoggingConfig,
        logger: Arc<dyn Logger>,
        time: Arc<dyn TimeProvider>,
        rng: R,
    ) -> Box<dyn RpcTransport>
    where
        T: RpcTransport + 'static,
        R: SecureRngProvider + 'static,
    {
        if !config.enabled {
            return Box::new(transport);
        }

        let already_installed = self.installed.swap(true, Ordering::SeqCst);
        if already_installed {
            Box::new(transport)
        } else {
            Box::new(LoggedRpcTransport::new(
                transport, config, logger, time, rng,
            ))
        }
    }
}

impl Default for RpcLoggingInstaller {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a rotating file logger sink from RPC logging config.
pub fn build_rpc_file_logger(
    config: &RpcLoggingConfig,
) -> Result<Arc<dyn Logger>, RpcLogInitError> {
    let rotation = RotationPolicy {
        max_bytes: config.output.rotation.max_bytes,
        keep_files: config.output.rotation.keep_files,
    };

    let effective_path = normalize_configured_path(&config.output.path);

    let logger = RotatingFileLogger::new(&effective_path, rotation).map_err(|source| {
        RpcLogInitError::Sink {
            path: effective_path.clone(),
            source,
        }
    })?;

    Ok(Arc::new(logger))
}

/// Wrap a transport with logging middleware when enabled.
pub fn maybe_wrap_transport<T, R>(
    transport: T,
    config: RpcLoggingConfig,
    logger: Arc<dyn Logger>,
    time: Arc<dyn TimeProvider>,
    rng: R,
) -> Box<dyn RpcTransport>
where
    T: RpcTransport + 'static,
    R: SecureRngProvider + 'static,
{
    if config.enabled {
        Box::new(LoggedRpcTransport::new(
            transport, config, logger, time, rng,
        ))
    } else {
        Box::new(transport)
    }
}
