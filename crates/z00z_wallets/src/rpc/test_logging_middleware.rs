use super::*;
use crate::rpc::logging::config::RpcLoggingTruncationConfig;
use z00z_utils::logger::VecLogger;
use z00z_utils::rng::SystemRngProvider;
use z00z_utils::time::SystemTimeProvider;

fn base_record() -> RpcLogRecord {
    RpcLogRecord {
        ts: "1970-01-01 00:00:00.001".to_string(),
        level: "info".to_string(),
        recorded_at: 1,
        event: "rpc.response",
        method: "m".to_string(),
        origin: "runtime".to_string(),
        request_id: "00".repeat(16),
        duration_ms: Some(1.0),
        risk: RiskLevel::Low,
        result: Some(AuditResult::Success),
        wallet_id: Some("w".to_string()),
        wallet_id_full: None,
        params_summary: Some(Value::String("P".repeat(500))),
        response_summary: Some(Value::String("R".repeat(500))),
        error_code: None,
        error_message: None,
    }
}

fn test_config(max_line_bytes: usize) -> RpcLoggingConfig {
    RpcLoggingConfig {
        enabled: true,
        level: "info".to_string(),
        output: super::super::config::RpcLoggingOutputConfig {
            path: "outputs/log/rpc_logger.log".to_string(),
            rotation: super::super::config::RpcLoggingRotationConfig {
                max_bytes: 1024,
                keep_files: 2,
            },
        },
        max_line_bytes,
        truncation: RpcLoggingTruncationConfig {
            non_secret_min_bytes: 24,
            head_chars: 6,
            tail_chars: 6,
        },
    }
}

#[test]
fn test_request_id_lowercase_hex() {
    struct StubTransport;

    #[async_trait(?Send)]
    impl RpcTransport for StubTransport {
        async fn call(&self, _method: &str, _params: Value) -> Result<Value, RpcError> {
            Ok(Value::Null)
        }
    }

    let logger: std::sync::Arc<dyn Logger> = std::sync::Arc::new(VecLogger::new());
    let time = std::sync::Arc::new(SystemTimeProvider);
    let rng = SystemRngProvider;

    let transport = LoggedRpcTransport::new(StubTransport, test_config(8192), logger, time, rng);
    let id = transport.request_id();
    assert_eq!(id.len(), 32);
    assert!(id.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')));
}

#[test]
fn test_serialize_cap_optional_fields() {
    let logger: std::sync::Arc<dyn Logger> =
        std::sync::Arc::new(z00z_utils::logger::VecLogger::new());
    let time = std::sync::Arc::new(SystemTimeProvider);
    let rng = SystemRngProvider;

    struct StubTransport;
    #[async_trait(?Send)]
    impl RpcTransport for StubTransport {
        async fn call(&self, _method: &str, _params: Value) -> Result<Value, RpcError> {
            Ok(Value::Null)
        }
    }

    let transport = LoggedRpcTransport::new(StubTransport, test_config(256), logger, time, rng);

    let huge = "A".repeat(10_000);
    let record = RpcLogRecord {
        ts: "1970-01-01 00:00:00.001".to_string(),
        level: "info".to_string(),
        recorded_at: 1,
        event: "rpc.request",
        method: "m".to_string(),
        origin: "runtime".to_string(),
        request_id: "00".repeat(16),
        duration_ms: None,
        risk: RiskLevel::Low,
        result: None,
        wallet_id: Some("w".to_string()),
        wallet_id_full: None,
        params_summary: Some(Value::String(huge)),
        response_summary: Some(Value::String("B".repeat(10_000))),
        error_code: None,
        error_message: None,
    };

    let line = transport
        .serialize_with_cap(record)
        .expect("must serialize");
    assert!(line.len() <= 256);
}

#[test]
fn test_serialize_cap_response_summary() {
    let logger: std::sync::Arc<dyn Logger> = std::sync::Arc::new(VecLogger::new());
    let time = std::sync::Arc::new(SystemTimeProvider);
    let rng = SystemRngProvider;

    struct StubTransport;
    #[async_trait(?Send)]
    impl RpcTransport for StubTransport {
        async fn call(&self, _method: &str, _params: Value) -> Result<Value, RpcError> {
            Ok(Value::Null)
        }
    }

    let full = base_record();
    let full_len = JsonCodec.serialize(&full).expect("must serialize").len();
    let mut no_response = full.clone();
    no_response.response_summary = None;
    let no_response_len = JsonCodec
        .serialize(&no_response)
        .expect("must serialize")
        .len();

    assert!(full_len > no_response_len);

    let max = no_response_len;
    assert!(full_len > max);

    let transport = LoggedRpcTransport::new(StubTransport, test_config(max), logger, time, rng);

    let line = transport.serialize_with_cap(full).expect("must serialize");
    let json: Value = JsonCodec
        .deserialize(line.as_bytes())
        .expect("must be json");

    assert!(json.get("response_summary").is_none());
    assert!(json.get("params_summary").is_some());
}

#[test]
fn test_serialize_cap_params_summary() {
    let logger: std::sync::Arc<dyn Logger> = std::sync::Arc::new(VecLogger::new());
    let time = std::sync::Arc::new(SystemTimeProvider);
    let rng = SystemRngProvider;

    struct StubTransport;
    #[async_trait(?Send)]
    impl RpcTransport for StubTransport {
        async fn call(&self, _method: &str, _params: Value) -> Result<Value, RpcError> {
            Ok(Value::Null)
        }
    }

    let full = base_record();
    let full_len = JsonCodec.serialize(&full).expect("must serialize").len();
    let mut no_response = full.clone();
    no_response.response_summary = None;
    let no_response_len = JsonCodec
        .serialize(&no_response)
        .expect("must serialize")
        .len();
    let mut no_params = no_response.clone();
    no_params.params_summary = None;
    let no_params_len = JsonCodec
        .serialize(&no_params)
        .expect("must serialize")
        .len();

    assert!(full_len > no_response_len);
    assert!(no_response_len > no_params_len);

    let max = no_params_len;
    assert!(full_len > max);
    assert!(no_response_len > max);

    let transport = LoggedRpcTransport::new(StubTransport, test_config(max), logger, time, rng);

    let line = transport.serialize_with_cap(full).expect("must serialize");
    let json: Value = JsonCodec
        .deserialize(line.as_bytes())
        .expect("must be json");

    assert!(json.get("response_summary").is_none());
    assert!(json.get("params_summary").is_none());
}

#[test]
fn test_serialize_cap_bounded_internal() {
    let logger: std::sync::Arc<dyn Logger> = std::sync::Arc::new(VecLogger::new());
    let time = std::sync::Arc::new(SystemTimeProvider);
    let rng = SystemRngProvider;

    const LONG_ERROR_CODE: &str = concat!(
        "some_very_long_error_code_",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "ccccccccccccccccccccccccccccccccccccccccccccccccbb"
    );
    const LONG_ERROR_MESSAGE: &str = concat!(
        "Some very long error message ",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "ccccccccccccccccccccccccccccccccccccccccccccccccbb"
    );

    struct StubTransport;
    #[async_trait(?Send)]
    impl RpcTransport for StubTransport {
        async fn call(&self, _method: &str, _params: Value) -> Result<Value, RpcError> {
            Ok(Value::Null)
        }
    }

    let mut full = base_record();
    full.error_code = Some(LONG_ERROR_CODE);
    full.error_message = Some(LONG_ERROR_MESSAGE);
    full.params_summary = Some(Value::String("P".repeat(10_000)));
    full.response_summary = Some(Value::String("R".repeat(10_000)));

    let full_len = JsonCodec.serialize(&full).expect("must serialize").len();
    let mut no_response = full.clone();
    no_response.response_summary = None;
    let no_response_len = JsonCodec
        .serialize(&no_response)
        .expect("must serialize")
        .len();
    let mut no_params = no_response.clone();
    no_params.params_summary = None;
    let no_params_len = JsonCodec
        .serialize(&no_params)
        .expect("must serialize")
        .len();

    let mut bounded = no_params.clone();
    bounded.error_code = Some("internal");
    bounded.error_message = Some("Internal error");
    let bounded_len = JsonCodec.serialize(&bounded).expect("must serialize").len();

    assert!(full_len > no_response_len);
    assert!(no_response_len > no_params_len);
    assert!(no_params_len > bounded_len);

    let max = bounded_len;
    assert!(full_len > max);
    assert!(no_response_len > max);
    assert!(no_params_len > max);

    let transport = LoggedRpcTransport::new(StubTransport, test_config(max), logger, time, rng);

    let line = transport.serialize_with_cap(full).expect("must serialize");
    let json: Value = JsonCodec
        .deserialize(line.as_bytes())
        .expect("must be json");

    assert_eq!(
        json.get("error_code").and_then(|v| v.as_str()),
        Some("internal")
    );
    assert_eq!(
        json.get("error_message").and_then(|v| v.as_str()),
        Some("Internal error")
    );
}

#[test]
fn test_serialize_cap_line_when() {
    let logger: std::sync::Arc<dyn Logger> = std::sync::Arc::new(VecLogger::new());
    let time = std::sync::Arc::new(SystemTimeProvider);
    let rng = SystemRngProvider;

    struct StubTransport;
    #[async_trait(?Send)]
    impl RpcTransport for StubTransport {
        async fn call(&self, _method: &str, _params: Value) -> Result<Value, RpcError> {
            Ok(Value::Null)
        }
    }

    let transport = LoggedRpcTransport::new(StubTransport, test_config(40), logger, time, rng);
    let record = base_record();
    assert!(transport.serialize_with_cap(record).is_none());
}
