#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use async_trait::async_trait;
use z00z_utils::codec::{json, Codec, Value};
use z00z_utils::io::read_to_string;

use z00z_networks_rpc::{RpcError, RpcTransport};
use z00z_utils::rng::SystemRngProvider;
use z00z_utils::time::SystemTimeProvider;

use chrono::{TimeZone as _, Utc};

use z00z_wallets::rpc::logging::{build_rpc_file_logger, LoggedRpcTransport, RpcLoggingConfig};

fn effective_log_path(cwd: &std::path::Path, configured_path: &str) -> std::path::PathBuf {
    let configured = std::path::PathBuf::from(configured_path);
    if configured.is_absolute() {
        return configured;
    }

    let prefix = std::path::PathBuf::from("crates").join("z00z_wallets");
    if configured.starts_with(&prefix) && cwd.ends_with(&prefix) {
        if let Ok(stripped) = configured.strip_prefix(&prefix) {
            return cwd.join(stripped);
        }
    }

    cwd.join(configured)
}

fn should_dump_rpc_logs() -> bool {
    std::env::var("Z00Z_TEST_RPC_LOGS").is_ok()
}

fn parse_logged_json_line(line: &str) -> Value {
    let json = line
        .find('{')
        .and_then(|index| line.get(index..))
        .unwrap_or_else(|| panic!("expected JSON payload in log line, got: {line}"));

    z00z_utils::codec::JsonCodec
        .deserialize(json.as_bytes())
        .unwrap_or_else(|e| panic!("invalid json: {e}; line={line}"))
}

fn test_assert_ts_valid_recent(ts: &str) {
    let naive = chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S%.3f")
        .unwrap_or_else(|e| panic!("ts must match YYYY-MM-DD HH:MM:SS.mmm (UTC): ts={ts} err={e}"));

    let logged = Utc.from_utc_datetime(&naive);
    let now = Utc::now();
    let drift_ms = (now - logged).num_milliseconds().abs();

    assert!(
        drift_ms <= 5 * 60 * 1000,
        "ts too far from current time: ts={ts} now={} drift_ms={drift_ms}",
        now.format("%Y-%m-%d %H:%M:%S%.3f")
    );
}

struct OkTransport;

#[async_trait(?Send)]
impl RpcTransport for OkTransport {
    async fn call(&self, _method: &str, _params: Value) -> Result<Value, RpcError> {
        Ok(json!({"ok": true}))
    }
}

#[tokio::test]
async fn test_rpc_logging_embedded_wallet() {
    let config =
        RpcLoggingConfig::from_default_wallet_yaml().expect("RPC logging config must load");
    assert!(
        config.enabled,
        "wallet.logger.rpc.enabled must be true for this test"
    );

    let cwd = std::env::current_dir().expect("failed to read current_dir");
    let log_path = effective_log_path(&cwd, &config.output.path);
    println!("Configured RPC log path: {}", log_path.display());

    let logger = build_rpc_file_logger(&config).expect("RPC log sink must initialize");
    let time: Arc<dyn z00z_utils::time::TimeProvider> = Arc::new(SystemTimeProvider);
    let rng = SystemRngProvider;

    let transport = LoggedRpcTransport::new(OkTransport, config, logger, time, rng);

    let before = read_to_string(&log_path).unwrap_or_default();

    let _ = transport
        .call(
            "app.wallet.create_wallet",
            json!({
                "name": "My Wallet",
                "password": "super_secret",
                "seed_phrase": "also_secret"
            }),
        )
        .await
        .expect("rpc call failed");

    assert!(
        log_path.exists(),
        "log file was not created at {}",
        log_path.display()
    );

    let contents = read_to_string(&log_path)
        .unwrap_or_else(|e| panic!("failed to read log file at {}: {e}", log_path.display()));

    let appended = contents.get(before.len()..).unwrap_or("");
    if should_dump_rpc_logs() {
        println!(
            "\n--- BEGIN appended configured-path RPC logs {} ---",
            log_path.display()
        );
        println!("{appended}");
        println!("--- END appended configured-path RPC logs ---\n");
    }
    for line in appended.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let v = parse_logged_json_line(line);
        let ts = v
            .get("ts")
            .and_then(|s| s.as_str())
            .expect("ts must be present and string");
        test_assert_ts_valid_recent(ts);
    }

    assert!(
        contents.contains("app.wallet.create_wallet"),
        "log file does not contain expected method"
    );
    assert!(
        contents.contains("<redacted>"),
        "log file does not redact secrets"
    );
}
