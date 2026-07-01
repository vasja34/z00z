use super::*;
use z00z_utils::codec::{Codec, JsonCodec};

#[test]
fn test_persist_audit_entry_serialization() {
    let entry = PersistAuditLogEntry {
        timestamp: 1_700_000_000_000,
        wallet_id: Some(PersistWalletId("wallet-123".to_string())),
        method: "wallet.send".to_string(),
        client_ip: Some("192.168.1.1".to_string()),
        user_agent: Some("Mozilla/5.0".to_string()),
        result: AuditResult::Success,
        risk_level: RiskLevel::High,
        context: Some("amount: 1000".to_string()),
    };

    let codec = JsonCodec;
    let bytes = codec.serialize(&entry).unwrap();
    let deserialized: PersistAuditLogEntry = codec.deserialize(&bytes).unwrap();

    assert_eq!(deserialized.timestamp, 1_700_000_000_000);
    assert_eq!(deserialized.method, "wallet.send");
    assert_eq!(deserialized.result, AuditResult::Success);
    assert_eq!(deserialized.risk_level, RiskLevel::High);
}

#[test]
fn test_audit_result_enum() {
    let results = [
        AuditResult::Success,
        AuditResult::Failure,
        AuditResult::RateLimited,
        AuditResult::Denied,
    ];

    for result in results {
        let entry = PersistAuditLogEntry {
            timestamp: 0,
            wallet_id: None,
            method: "test".to_string(),
            client_ip: None,
            user_agent: None,
            result,
            risk_level: RiskLevel::Low,
            context: None,
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&entry).unwrap();
        let deserialized: PersistAuditLogEntry = codec.deserialize(&bytes).unwrap();

        assert_eq!(deserialized.result, result);
    }
}

#[test]
fn test_risk_level_ordering() {
    assert!(RiskLevel::Critical < RiskLevel::High);
    assert!(RiskLevel::High < RiskLevel::Medium);
    assert!(RiskLevel::Medium < RiskLevel::Low);
}

#[test]
fn test_risk_level_serialization() {
    let levels = [
        RiskLevel::Critical,
        RiskLevel::High,
        RiskLevel::Medium,
        RiskLevel::Low,
    ];

    for level in levels {
        let entry = PersistAuditLogEntry {
            timestamp: 0,
            wallet_id: None,
            method: "test".to_string(),
            client_ip: None,
            user_agent: None,
            result: AuditResult::Success,
            risk_level: level,
            context: None,
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&entry).unwrap();
        let deserialized: PersistAuditLogEntry = codec.deserialize(&bytes).unwrap();

        assert_eq!(deserialized.risk_level, level);
    }
}

#[test]
fn test_runtime_rate_limit_tier() {
    assert_eq!(RuntimeRateLimitTier::HIGH_RISK.requests_per_minute, 3);
    assert_eq!(RuntimeRateLimitTier::WRITE_OPS.requests_per_minute, 10);
    assert_eq!(RuntimeRateLimitTier::READ_OPS.requests_per_minute, 100);
    assert_eq!(RuntimeRateLimitTier::BULK_OPS.requests_per_minute, 20);
}

#[test]
fn test_audit_log_none_fields() {
    let entry = PersistAuditLogEntry {
        timestamp: 0,
        wallet_id: None,
        method: "test".to_string(),
        client_ip: None,
        user_agent: None,
        result: AuditResult::Success,
        risk_level: RiskLevel::Low,
        context: None,
    };

    let codec = JsonCodec;
    let bytes = codec.serialize(&entry).unwrap();
    let deserialized: PersistAuditLogEntry = codec.deserialize(&bytes).unwrap();

    assert!(deserialized.wallet_id.is_none());
    assert!(deserialized.client_ip.is_none());
    assert!(deserialized.user_agent.is_none());
    assert!(deserialized.context.is_none());
}

#[test]
fn test_audit_log_empty_context() {
    let entry = PersistAuditLogEntry {
        timestamp: 0,
        wallet_id: None,
        method: "test".to_string(),
        client_ip: None,
        user_agent: None,
        result: AuditResult::Success,
        risk_level: RiskLevel::Low,
        context: Some(String::new()),
    };

    let codec = JsonCodec;
    let bytes = codec.serialize(&entry).unwrap();
    let deserialized: PersistAuditLogEntry = codec.deserialize(&bytes).unwrap();

    assert!(deserialized.context.is_some());
    assert!(deserialized.context.unwrap().is_empty());
}
