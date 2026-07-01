#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use z00z_utils::io::read_to_string;

const APP_RPC: &str = include_str!("../src/rpc/app_rpc.rs");
const APP_IMPL: &str = include_str!("../src/rpc/app_rpc_impl.rs");
const APP_WIRING: &str = include_str!("../src/rpc/app_dispatcher_wiring.rs");
const ROUTE_SRC: &str = include_str!("../src/rpc/wallet_dispatcher_routes.rs");
const RPC_MOD: &str = include_str!("../src/rpc/mod.rs");

fn workspace_root() -> PathBuf {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or(crate_dir)
}

fn read_json(path: &Path) -> Value {
    let text = read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|err| panic!("failed to decode {}: {err}", path.display()))
}

fn run_audit() -> Value {
    let temp = tempfile::tempdir().expect("audit tempdir");
    let csv = temp.path().join("audit_rpc_methods.csv");
    let md = temp.path().join("audit_rpc_methods.md");
    let json = temp.path().join("audit_rpc_methods.json");
    let root = workspace_root();

    let status = Command::new("python3")
        .current_dir(&root)
        .arg("crates/z00z_wallets/scripts/audit_rpc_method_wiring.py")
        .arg("--workspace")
        .arg(&root)
        .arg("--csv-out")
        .arg(&csv)
        .arg("--md-out")
        .arg(&md)
        .arg("--json-out")
        .arg(&json)
        .status()
        .expect("run audit_rpc_method_wiring.py");

    assert!(
        status.success(),
        "audit_rpc_method_wiring.py must succeed after split-route parsing"
    );

    read_json(&json)
}

fn row_by_rpc<'a>(rows: &'a [Value], rpc: &str) -> &'a Value {
    rows.iter()
        .find(|row| row.get("rpc").and_then(Value::as_str) == Some(rpc))
        .unwrap_or_else(|| panic!("missing audit row for {rpc}"))
}

fn warnings(report: &Value) -> Vec<String> {
    report["warnings"]
        .as_array()
        .expect("warnings array")
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn guard_kind(row: &Value) -> &str {
    row["guard_kind"].as_str().unwrap_or("")
}

fn route_uses_cap_guard(source: &str, rpc: &str) -> bool {
    let anchor = format!("dispatcher.register_typed(\n        \"{rpc}\"");
    let inline_anchor = format!("dispatcher.register_typed(\"{rpc}\"");
    let Some(start) = source.find(&anchor).or_else(|| source.find(&inline_anchor)) else {
        return false;
    };
    let tail = &source[start..];
    let end = tail
        .find("dispatcher.register_typed(")
        .filter(|idx| *idx > 0)
        .unwrap_or(tail.len());
    let body = &tail[..end];
    body.contains("typed_handler_cap(")
        && (body.contains("verify_touch_cap(session)")
            || body.contains("verify_no_touch_cap(session)")
            || body.contains("verify_rotate_cap(session)"))
}

#[test]
fn open_wallet_route_live() {
    assert!(APP_RPC.contains("#[method(name = \"app.wallet.open_wallet_source\")]"));
    assert!(APP_IMPL.contains("async fn open_wallet_source(&self, source: WalletSource)"));
    assert!(APP_WIRING.contains("\"app.wallet.open_wallet_source\""));
    assert!(APP_WIRING.contains("rpc.open_wallet_source(source).await"));
    assert!(RPC_MOD.contains("pub fn register_all_wallet_rpc_methods("));
    assert!(RPC_MOD.contains("pub fn register_all_app_rpc_methods("));
}

#[test]
fn audit_sees_split_routes() {
    let report = run_audit();
    let errors = report["errors"]
        .as_array()
        .expect("errors array")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    assert!(
        errors.is_empty(),
        "route audit must have zero dispatcher errors, got: {errors:#?}"
    );

    let rows = report["rows"].as_array().expect("rows array");
    let warnings = warnings(&report);
    for rpc in [
        "app.wallet.open_wallet_source",
        "wallet.asset.list_assets",
        "wallet.key.derive_receiver",
        "wallet.object.preview_package",
        "wallet.tx.send_transaction",
        "wallet.session.unlock_wallet",
    ] {
        let row = row_by_rpc(rows, rpc);
        assert_eq!(
            row["dispatcher_registered"].as_bool(),
            Some(true),
            "{rpc} must stay registered in the dispatcher audit"
        );
        let dispatcher_file = row["dispatcher_file"]
            .as_str()
            .unwrap_or_else(|| panic!("{rpc} missing dispatcher_file"));
        assert!(
            dispatcher_file.starts_with("crates/z00z_wallets/"),
            "{rpc} dispatcher path must stay canonical, got: {dispatcher_file}"
        );
        assert!(
            !dispatcher_file.starts_with("z00z_wallet/"),
            "{rpc} dispatcher path must not use the legacy alias, got: {dispatcher_file}"
        );
    }

    let expected_guards = [
        ("wallet.session.lock_wallet", "no_touch"),
        ("wallet.session.show_seed_phrase", "no_touch"),
        ("wallet.backup.create_backup", "touch"),
        ("wallet.backup.list_backups", "touch"),
        ("wallet.backup.configure_backup", "touch"),
        ("wallet.key.derive_receiver", "no_touch"),
        ("wallet.key.get_receiver_card", "touch"),
        ("wallet.key.create_payment_request", "touch"),
        ("wallet.key.validate_payment_request", "touch"),
        ("wallet.key.export_public_material", "touch"),
        ("wallet.key.rotate_master_key", "no_touch"),
        ("wallet.key.list_receivers", "touch"),
        ("wallet.key.label_receiver", "touch"),
    ];
    for (rpc, expected) in expected_guards {
        let row = row_by_rpc(rows, rpc);
        assert_eq!(
            guard_kind(row),
            expected,
            "{rpc} must keep {expected} guard"
        );
    }

    let asset_row = row_by_rpc(rows, "wallet.asset.list_assets");
    assert_eq!(
        asset_row["rpc_impl_file"].as_str(),
        Some("crates/z00z_wallets/src/rpc/asset_rpc_server.rs"),
        "asset audit rows must resolve the live server implementation file"
    );

    let object_row = row_by_rpc(rows, "wallet.object.preview_package");
    assert_eq!(
        object_row["rpc_impl_file"].as_str(),
        Some("crates/z00z_wallets/src/rpc/object_rpc_impl.rs"),
        "object audit rows must resolve the live direct-owner implementation file"
    );

    assert!(
        warnings
            .iter()
            .all(|warning| !warning.contains("wallet.object.")),
        "direct-owner object RPC rows must not be reported as stub/unwired: {warnings:#?}"
    );
}

#[test]
fn priv_route_requires_cap() {
    assert!(route_uses_cap_guard(
        ROUTE_SRC,
        "wallet.key.rotate_master_key"
    ));
    assert!(!route_uses_cap_guard(
        r#"
dispatcher.register_typed(
    "wallet.key.rotate_master_key",
    typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: KeyRotateParams| async move {
        rpc.rotate_master_key(p.session, p.password, p.confirmation).await
    }),
);
"#,
        "wallet.key.rotate_master_key",
    ));
}
