#![cfg(not(target_arch = "wasm32"))]
#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcError, RpcTransport};
use z00z_utils::time::{SystemTimeProvider, TimeProvider};

use z00z_wallets::rpc::{
    logging::{build_rpc_file_logger, LoggedRpcTransport, RpcLoggingConfig},
    methods::{
        AppRpcImpl, AssetRpcImpl, BackupRpcImpl, ChainRpcImpl, ChainScanRpcImpl, KeyRpcImpl,
        NetworkRpcImpl, StorageRpcImpl, TxRpcImpl, WalletRpcImpl,
    },
    register_all_wallet_rpc_methods,
};
use z00z_wallets::services::{AppService, WalletService};

static TEST_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn workspace_root() -> std::path::PathBuf {
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or(crate_dir)
}

fn read_audit_rpc_methods(csv_path: &std::path::Path) -> Vec<String> {
    let content = z00z_utils::io::read_to_string(csv_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", csv_path.display()));

    let mut methods = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        if idx == 0 {
            continue;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let rpc = line.split(',').next().unwrap_or("").trim();
        if rpc.is_empty() {
            continue;
        }
        methods.push(rpc.to_string());
    }

    methods
}

fn generate_audit_rpc_methods(csv_path: &std::path::Path) {
    let script = workspace_root().join("crates/z00z_wallets/scripts/audit_rpc_method_wiring.sh");
    let md_path = csv_path.with_extension("md");
    let json_path = csv_path.with_extension("json");
    let status = std::process::Command::new("bash")
        .current_dir(workspace_root())
        .arg(script)
        .arg("--csv-out")
        .arg(csv_path)
        .arg("--md-out")
        .arg(&md_path)
        .arg("--json-out")
        .arg(&json_path)
        .status()
        .expect("run audit rpc wiring script");

    assert!(
        csv_path.exists() && md_path.exists() && json_path.exists(),
        "audit rpc wiring script must generate audit artifacts (exit={:?})",
        status.code()
    );
}

fn load_audit_rpc_methods() -> Vec<String> {
    let csv_path =
        workspace_root().join("crates/z00z_wallets/outputs/audit_rpc/audit_rpc_methods.csv");
    if csv_path.exists() {
        return read_audit_rpc_methods(&csv_path);
    }

    let temp = tempfile::tempdir().expect("audit rpc tempdir");
    let temp_csv = temp.path().join("audit_rpc_methods.csv");
    generate_audit_rpc_methods(&temp_csv);
    read_audit_rpc_methods(&temp_csv)
}

fn parse_logged_json_line(line: &str, idx: usize) -> Value {
    let json = line
        .find('{')
        .and_then(|index| line.get(index..))
        .unwrap_or_else(|| panic!("tail line {idx} is missing JSON payload: {line}"));

    serde_json::from_str(json)
        .unwrap_or_else(|e| panic!("invalid json on tail line {idx}: {e}; line={line}"))
}

fn params_for_method(
    method: &str,
    wallet_name: &str,
    wallet_id: Option<&str>,
    password: &str,
    session: Option<&Value>,
) -> Value {
    match method {
        "app.wallet.create_wallet" => json!({
            "name": wallet_name,
            "password": password,
            "seed_phrase": null
        }),
        "app.wallet.list_wallets" => json!({}),
        // Session-gated wallet methods use the typed `wallet_id` field.
        "wallet.session.unlock_wallet" => {
            if let Some(id) = wallet_id {
                json!({"wallet_id": id, "password": password})
            } else {
                json!({"wallet_id": "missing", "password": password})
            }
        }
        "wallet.session.lock_wallet" => {
            if let Some(id) = wallet_id {
                json!({"wallet_id": id})
            } else {
                json!({"wallet_id": "missing"})
            }
        }
        "wallet.session.show_seed_phrase" => {
            if let Some(session) = session {
                json!({"session": session})
            } else {
                json!({})
            }
        }
        // Safe read-only-ish calls (best-effort)
        "wallet.asset.list_assets" => {
            if let Some(id) = wallet_id {
                json!({"wallet_id": id, "limit": 10, "cursor": null, "filter": null})
            } else {
                json!({"wallet_id": "missing", "limit": 10, "cursor": null, "filter": null})
            }
        }
        "wallet.key.rotate_master_key" => {
            if let Some(session) = session {
                json!({"session": session, "confirmation": "ROTATE"})
            } else {
                json!({})
            }
        }
        "wallet.key.list_receivers" => {
            if let Some(session) = session {
                json!({"session": session, "limit": 10, "cursor": null, "filter": null})
            } else {
                json!({})
            }
        }
        "wallet.key.validate_receiver_card" => json!({"card_compact": "stub-card"}),
        "wallet.key.label_receiver" => {
            if let Some(session) = session {
                json!({
                    "session": session,
                    "receiver_id": "00".repeat(32),
                    "label": "Receiver Label"
                })
            } else {
                json!({})
            }
        }
        "wallet.tx.list_pending_transactions" => {
            if let Some(session) = session {
                json!({"session": session, "pagination": {"limit": 10, "cursor": null}})
            } else {
                json!({})
            }
        }
        "wallet.tx.send_transaction" => {
            if let Some(session) = session {
                json!({
                    "session": session,
                    "asset_id": vec![0u8; 32],
                    "recipient": "invalid-recipient-log-fixture",
                    "amount": 1,
                    "memo": "SECRET_MEMO",
                    "idempotency_key": "idempotency-key-audit-csv",
                    "timestamp": 1700000000
                })
            } else {
                json!({})
            }
        }
        _ => json!({}),
    }
}

struct FileOnlyTransport<T>(T);

#[async_trait(?Send)]
impl<T: RpcTransport> RpcTransport for FileOnlyTransport<T> {
    async fn call(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        self.0.call(method, params).await
    }
}

#[tokio::test]
async fn test_replay_audit_methods_csv() {
    let _guard = TEST_LOCK.lock().await;

    let methods = load_audit_rpc_methods();
    assert!(
        !methods.is_empty(),
        "audit_rpc_methods.csv must contain methods"
    );

    // Use the default wallet config (crates/z00z_wallets/src/config/wallet_config.yaml).
    let config =
        RpcLoggingConfig::from_default_wallet_yaml().expect("RPC logging config must load");
    assert!(
        config.enabled,
        "wallet.logger.rpc.enabled must be true for this test"
    );

    let log_path = std::path::PathBuf::from(&config.output.path);

    // Read the current log file to ensure it exists and is readable.
    // The logger may rotate files; this test validates the presence of a unique record.
    let _before = z00z_utils::io::read_to_string(&log_path).unwrap_or_default();

    let wallets_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("outputs/wallets");
    let service = Arc::new(WalletService::with_output_dir(wallets_dir));

    let wallet_rpc = Arc::new(WalletRpcImpl::new(Arc::clone(&service)));
    let app_service = Arc::new(AppService::with_wallet_service(Arc::clone(&service)));
    let app_rpc = Arc::new(AppRpcImpl::new(Arc::clone(&app_service)));
    let asset_rpc = Arc::new(AssetRpcImpl::new());
    let tx_rpc = Arc::new(TxRpcImpl::new(Arc::clone(&service)));
    let backup_rpc = Arc::new(BackupRpcImpl::new(Arc::clone(&service)));
    let key_rpc = Arc::new(KeyRpcImpl::new(Arc::clone(&service)));
    let chain_rpc = Arc::new(ChainRpcImpl::new(Arc::clone(&app_service)));
    let network_rpc = Arc::new(NetworkRpcImpl::with_app_service(Arc::clone(&app_service)));
    let scan_rpc = Arc::new(ChainScanRpcImpl::new(Arc::clone(&app_service)));
    let storage_rpc = Arc::new(StorageRpcImpl::new(Arc::clone(&service)));

    let dispatcher = Arc::new(RpcDispatcher::new());
    register_all_wallet_rpc_methods(
        dispatcher.as_ref(),
        Arc::clone(&app_rpc),
        Arc::clone(&wallet_rpc),
        Arc::clone(&asset_rpc),
        Arc::clone(&tx_rpc),
        Arc::clone(&backup_rpc),
        Arc::clone(&key_rpc),
        Arc::clone(&chain_rpc),
        Arc::clone(&network_rpc),
        Arc::clone(&scan_rpc),
        Arc::clone(&storage_rpc),
    )
    .expect("wallet RPC registration should succeed");

    let base = FileOnlyTransport(LocalRpcTransport::new(dispatcher));

    let logger = build_rpc_file_logger(&config).expect("RPC log sink must initialize");
    let time: Arc<dyn z00z_utils::time::TimeProvider> = Arc::new(SystemTimeProvider);
    let rng = z00z_utils::rng::SystemRngProvider;

    let transport = LoggedRpcTransport::new(base, config, logger, time, rng);

    let password = "Aa1!bB2@cC3#dD4$eE5%";
    // Keep the name short so it is not truncated by the RPC logger.
    let wallet_name = format!(
        "auditcsv-{}",
        SystemTimeProvider.compat_unix_timestamp_millis()
    );

    // Ensure we have at least one wallet for methods that require it.
    let created = transport
        .call(
            "app.wallet.create_wallet",
            params_for_method(
                "app.wallet.create_wallet",
                &wallet_name,
                None,
                password,
                None,
            ),
        )
        .await
        .expect("create_wallet must succeed");

    let wallet_id_value = &created["wallet_id"];
    let wallet_id = wallet_id_value
        .as_str()
        .or_else(|| wallet_id_value.get(0).and_then(|v| v.as_str()))
        .or_else(|| wallet_id_value.get("0").and_then(|v| v.as_str()))
        .unwrap_or("stub-wallet-id")
        .to_string();

    // Pre-unlock so we can replay session-gated methods (e.g., show_seed_phrase).
    let session = transport
        .call(
            "wallet.session.unlock_wallet",
            params_for_method(
                "wallet.session.unlock_wallet",
                &wallet_name,
                Some(&wallet_id),
                password,
                None,
            ),
        )
        .await
        .expect("unlock_wallet must succeed");

    // Keep this test fast: we only need a representative sample to validate
    // "does not hang" + logging behavior.
    for method in methods.into_iter().take(20) {
        let params = params_for_method(
            &method,
            &wallet_name,
            Some(&wallet_id),
            password,
            Some(&session),
        );

        // Best-effort replay: we only require that it does not hang.
        let _ = timeout(Duration::from_millis(250), transport.call(&method, params)).await;
    }

    let after = z00z_utils::io::read_to_string(&log_path)
        .unwrap_or_else(|e| panic!("failed to read log file at {}: {e}", log_path.display()));

    // Validate a tail window of records rather than relying on fragile substring slicing.
    // The log file is append-only and may not end in a newline prior to this test.
    let tail: Vec<&str> = after.lines().rev().take(100).collect();
    assert!(
        tail.iter()
            .any(|line| line.contains("\"event\":\"rpc.request\"")),
        "expected at least one rpc.request record in log tail"
    );

    let mut saw_unique_create_wallet_request = false;

    for (idx, line) in tail.iter().enumerate() {
        let s = line.trim();
        if s.is_empty() {
            continue;
        }
        let v = parse_logged_json_line(s, idx);

        if v.get("event") == Some(&Value::String("rpc.request".to_string()))
            && v.get("method") == Some(&Value::String("app.wallet.create_wallet".to_string()))
            && v.get("params_summary")
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                == Some(wallet_name.as_str())
        {
            saw_unique_create_wallet_request = true;
        }

        assert!(v.get("ts").is_some(), "missing ts on tail line {idx}");
        assert!(v.get("level").is_some(), "missing level on tail line {idx}");
        assert!(v.get("event").is_some(), "missing event on tail line {idx}");
        assert!(
            v.get("method").is_some(),
            "missing method on tail line {idx}"
        );
        assert!(
            v.get("request_id").is_some(),
            "missing request_id on tail line {idx}"
        );
    }

    assert!(
        saw_unique_create_wallet_request,
        "expected a create_wallet rpc.request record with name={wallet_name} in log tail"
    );
}
