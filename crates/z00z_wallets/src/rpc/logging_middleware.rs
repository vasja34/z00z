//! Logged RPC transport wrapper.

use super::config::RpcLoggingConfig;
use super::policy::risk_for_method;
use super::record::RpcLogRecord;
use super::summarize::{
    extract_wallet_id, summarize_params, summarize_response, truncate_non_secret,
};
use async_trait::async_trait;
use z00z_networks_rpc::{RpcError, RpcTransport};
use z00z_utils::codec::{Codec, JsonCodec, Value};
use z00z_utils::config::{ConfigSource, EnvConfig};
use z00z_utils::logger::Logger;
use z00z_utils::rng::{RngCoreExt, SecureRngProvider};
use z00z_utils::time::{format_unix_timestamp_millis_utc, Instant, TimeProvider};

use crate::rpc::types::security::{AuditResult, RiskLevel};

/// Transport wrapper that emits best-effort JSONL logs.
pub struct LoggedRpcTransport<T, R>
where
    T: RpcTransport,
    R: SecureRngProvider,
{
    inner: T,
    config: RpcLoggingConfig,
    logger: std::sync::Arc<dyn Logger>,
    time: std::sync::Arc<dyn TimeProvider>,
    rng: std::sync::Mutex<R::Rng>,
    origin: String,
}

impl<T, R> LoggedRpcTransport<T, R>
where
    T: RpcTransport,
    R: SecureRngProvider,
{
    pub fn new(
        inner: T,
        config: RpcLoggingConfig,
        logger: std::sync::Arc<dyn Logger>,
        time: std::sync::Arc<dyn TimeProvider>,
        rng: R,
    ) -> Self {
        let origin = EnvConfig
            .get("Z00Z_RPC_LOG_ORIGIN")
            .ok()
            .flatten()
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty())
            .and_then(|value| {
                if value == "test" {
                    Some("test".to_string())
                } else if value == "runtime" {
                    Some("runtime".to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "runtime".to_string());

        Self {
            inner,
            config,
            logger,
            time,
            rng: std::sync::Mutex::new(rng.rng()),
            origin,
        }
    }

    fn emit(&self, level: &str, msg: &str) {
        match level {
            "error" => self.logger.error(msg),
            "warn" => self.logger.warn(msg),
            "debug" => self.logger.debug(msg),
            "trace" => self.logger.trace(msg),
            _ => self.logger.info(msg),
        }
    }

    fn request_id(&self) -> String {
        let mut rng = match self.rng.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let mut bytes = [0u8; 16];
        rng.fill_bytes_ext(&mut bytes);
        hex::encode(bytes)
    }

    fn bounded_error_template(&self, err: &RpcError) -> (&'static str, &'static str, AuditResult) {
        match err {
            RpcError::AuthFailed => ("auth_failed", "Authentication failed", AuditResult::Denied),
            RpcError::SessionExpired => ("session_expired", "Session expired", AuditResult::Denied),
            RpcError::SessionInvalid => ("session_invalid", "Session invalid", AuditResult::Denied),
            RpcError::RateLimited(_) => (
                "rate_limited",
                "Rate limit exceeded",
                AuditResult::RateLimited,
            ),
            RpcError::WalletLocked => ("wallet_locked", "Permission denied", AuditResult::Denied),
            RpcError::InvalidParams(message) => {
                if message.contains("Invalid confirmation") {
                    (
                        "invalid_confirmation",
                        "Confirmation denied",
                        AuditResult::Denied,
                    )
                } else {
                    ("invalid_params", "Validation failed", AuditResult::Failure)
                }
            }
            RpcError::MethodNotFound(_) => {
                ("method_not_found", "Permission denied", AuditResult::Denied)
            }
            RpcError::TransportError(_)
            | RpcError::RequestFailed(_)
            | RpcError::InvalidResponse(_) => ("transport", "Internal error", AuditResult::Failure),
            RpcError::Internal(_) | RpcError::WalletNotFound(_) => {
                ("internal", "Internal error", AuditResult::Failure)
            }
        }
    }

    fn format_ts(&self, ts_ms: u64) -> String {
        format_unix_timestamp_millis_utc(ts_ms)
    }

    fn serialize_with_cap(&self, mut record: RpcLogRecord) -> Option<String> {
        let max = self.config.max_line_bytes;

        let mut json = String::from_utf8(JsonCodec.serialize(&record).ok()?).ok()?;
        if json.len() <= max {
            return Some(json);
        }

        record.response_summary = None;
        json = String::from_utf8(JsonCodec.serialize(&record).ok()?).ok()?;
        if json.len() <= max {
            return Some(json);
        }

        record.params_summary = None;
        json = String::from_utf8(JsonCodec.serialize(&record).ok()?).ok()?;
        if json.len() <= max {
            return Some(json);
        }

        record.error_message = Some("Internal error");
        record.error_code = Some("internal");
        let json = String::from_utf8(JsonCodec.serialize(&record).ok()?).ok()?;
        if json.len() <= max {
            Some(json)
        } else {
            None
        }
    }
}

#[async_trait(?Send)]
impl<T, R> RpcTransport for LoggedRpcTransport<T, R>
where
    T: RpcTransport,
    R: SecureRngProvider,
{
    async fn call(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        if !self.config.enabled {
            return self.inner.call(method, params).await;
        }

        let start = Instant::now();
        let request_id_full = self.request_id();
        let request_id = truncate_non_secret(&request_id_full, &self.config.truncation);
        let level = self.config.level.to_lowercase();
        let risk: RiskLevel = risk_for_method(method);
        let wallet_id_full = extract_wallet_id(&params);
        let wallet_id = wallet_id_full
            .as_deref()
            .map(|id| truncate_non_secret(id, &self.config.truncation));
        let params_summary = summarize_params(method, &params, &self.config.truncation);

        let recorded_at = self.time.compat_unix_timestamp_millis();
        let ts = self.format_ts(recorded_at);

        let request_record = RpcLogRecord {
            ts,
            level: level.clone(),
            recorded_at,
            event: "rpc.request",
            method: method.to_string(),
            origin: self.origin.clone(),
            request_id: request_id.clone(),
            duration_ms: None,
            risk,
            result: None,
            wallet_id: wallet_id.clone(),
            wallet_id_full: wallet_id_full.clone(),
            params_summary,
            response_summary: None,
            error_code: None,
            error_message: None,
        };

        if let Some(line) = self.serialize_with_cap(request_record) {
            self.emit(&self.config.level, &line);
        }

        let result = self.inner.call(method, params).await;
        let duration_us = start.elapsed().as_micros();
        let duration_ms = {
            let ms = (duration_us as f64) / 1000.0;
            (ms * 1000.0).round() / 1000.0
        };

        match &result {
            Ok(value) => {
                let response_summary = summarize_response(method, value, &self.config.truncation);

                let recorded_at = self.time.compat_unix_timestamp_millis();
                let ts = self.format_ts(recorded_at);
                let record = RpcLogRecord {
                    ts,
                    level: level.clone(),
                    recorded_at,
                    event: "rpc.response",
                    method: method.to_string(),
                    origin: self.origin.clone(),
                    request_id,
                    duration_ms: Some(duration_ms),
                    risk,
                    result: Some(AuditResult::Success),
                    wallet_id,
                    wallet_id_full,
                    params_summary: None,
                    response_summary,
                    error_code: None,
                    error_message: None,
                };

                if let Some(line) = self.serialize_with_cap(record) {
                    self.emit(&self.config.level, &line);
                }
            }
            Err(err) => {
                let (code, msg, result_class) = self.bounded_error_template(err);

                let recorded_at = self.time.compat_unix_timestamp_millis();
                let ts = self.format_ts(recorded_at);
                let record = RpcLogRecord {
                    ts,
                    level,
                    recorded_at,
                    event: "rpc.error",
                    method: method.to_string(),
                    origin: self.origin.clone(),
                    request_id,
                    duration_ms: Some(duration_ms),
                    risk,
                    result: Some(result_class),
                    wallet_id,
                    wallet_id_full,
                    params_summary: None,
                    response_summary: None,
                    error_code: Some(code),
                    error_message: Some(msg),
                };

                if let Some(line) = self.serialize_with_cap(record) {
                    self.emit(&self.config.level, &line);
                }
            }
        }

        result
    }
}

#[cfg(test)]
#[path = "test_logging_middleware.rs"]
mod tests;
