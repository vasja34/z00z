//! JSONL record model for RPC middleware logs.

use crate::rpc::types::common::PersistWalletId;
use crate::rpc::types::security::{AuditResult, PersistAuditLogEntry, RiskLevel};
use serde::Serialize;
use z00z_utils::codec::Value;

#[derive(Debug, Clone, Serialize)]
pub struct RpcLogRecord {
    pub ts: String,
    pub level: String,

    #[serde(skip_serializing)]
    pub recorded_at: u64,
    pub event: &'static str,
    pub method: String,

    pub origin: String,
    pub request_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<f64>,

    pub risk: RiskLevel,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<AuditResult>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_id: Option<String>,

    #[serde(skip_serializing)]
    pub wallet_id_full: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub params_summary: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_summary: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<&'static str>,
}

/// Convert an RPC log record to the existing security audit log entry.
///
/// This is a pure mapping function intended to guarantee stable field mapping.
pub fn to_audit_entry(record: &RpcLogRecord) -> PersistAuditLogEntry {
    let wallet_id = record
        .wallet_id_full
        .as_deref()
        .or(record.wallet_id.as_deref())
        .map(|id| PersistWalletId(id.to_string()));

    PersistAuditLogEntry {
        timestamp: record.recorded_at,
        wallet_id,
        method: record.method.clone(),
        client_ip: None,
        user_agent: None,
        result: record.result.unwrap_or(AuditResult::Failure),
        risk_level: record.risk,
        context: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z00z_utils::codec::{Codec, JsonCodec};

    #[test]
    fn test_json_serialization_skipped_fields() {
        let record = RpcLogRecord {
            ts: "1970-01-01 00:00:00.123".to_string(),
            level: "info".to_string(),
            recorded_at: 123,
            event: "rpc.response",
            method: "wallet.asset.list_assets".to_string(),
            origin: "runtime".to_string(),
            request_id: "00".repeat(16),
            duration_ms: Some(0.001),
            risk: RiskLevel::Low,
            result: Some(AuditResult::Success),
            wallet_id: Some("stub-wallet-id".to_string()),
            wallet_id_full: Some("stub-wallet-id-full".to_string()),
            params_summary: None,
            response_summary: None,
            error_code: None,
            error_message: None,
        };

        let value = JsonCodec
            .serialize(&record)
            .and_then(|bytes| JsonCodec.deserialize::<Value>(&bytes))
            .expect("record must serialize");
        assert!(
            value.get("recorded_at").is_none(),
            "recorded_at must be skipped"
        );
        assert!(
            value.get("wallet_id_full").is_none(),
            "wallet_id_full must be skipped"
        );
        assert_eq!(
            value.get("origin").and_then(|v| v.as_str()),
            Some("runtime"),
            "origin must be serialized"
        );
    }

    #[test]
    fn test_mapping_preserves_minimal_fields() {
        let record = RpcLogRecord {
            ts: "1970-01-01 00:00:00.123".to_string(),
            level: "info".to_string(),
            recorded_at: 123,
            event: "rpc.response",
            method: "wallet.asset.list_assets".to_string(),
            origin: "runtime".to_string(),
            request_id: "00".repeat(16),
            duration_ms: Some(0.001),
            risk: RiskLevel::Low,
            result: Some(AuditResult::Success),
            wallet_id: Some("stub-wallet-id".to_string()),
            wallet_id_full: None,
            params_summary: None,
            response_summary: None,
            error_code: None,
            error_message: None,
        };

        let entry = to_audit_entry(&record);
        assert_eq!(entry.timestamp, 123);
        assert_eq!(entry.method, "wallet.asset.list_assets");
        assert_eq!(
            entry.wallet_id.map(|w| w.0),
            Some("stub-wallet-id".to_string())
        );
        assert_eq!(entry.risk_level, RiskLevel::Low);
        assert_eq!(entry.result, AuditResult::Success);
    }
}
