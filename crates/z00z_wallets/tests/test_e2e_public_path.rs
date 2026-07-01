//! E2E Phase 3 section 7 public-path policy.
//! Policed scope:
//! - `crates/z00z_wallets/tests/test_e2e*.rs`
//! - `crates/z00z_wallets/tests/test_s6_recv_examples.rs`
//! - `crates/z00z_simulator/examples/scenario_1/simulator_interop*.{rs,inc}`
//! - `specs/013-stealth-address/E2E-stealth-EXAMPLES.md`
//!
//! Allowed public stealth surfaces:
//! - `z00z_wallets`
//! - `z00z_wallets::stealth::ecdh`
//! - `z00z_wallets::stealth::kdf`
//! - `z00z_wallets::receiver`
//!
//! Forbidden public-story paths:
//! - `z00z_wallets::stealth::*`
//! - `z00z_wallets::{ecdh,kdf,scan}::*`
//! - `z00z_wallets::{ecdh,kdf,scan}::*`
//! - internal `receiver::asset_scan::wallet_asset_scanner::*`

use std::{fs, path::PathBuf};

const BAD: &[&str] = &[
    "z00z_wallets::stealth::",
    "z00z_wallets::ecdh::",
    "z00z_wallets::kdf::",
    "z00z_wallets::scan::",
    "z00z_wallets::ecdh::",
    "z00z_wallets::kdf::",
    "z00z_wallets::scan::",
    "stealth::output_validator",
    "receiver::asset_scan::wallet_asset_scanner::",
    "wallet_asset_scanner::",
    "z00z_wallets::tx::",
    "use z00z_wallets::tx::{",
];

const OK: &[&str] = &[
    "z00z_wallets::{",
    "z00z_wallets::receiver::",
    "z00z_wallets::stealth::ecdh",
    "z00z_wallets::stealth::kdf",
    "z00z_wallets::stealth::zkpack",
];

fn root_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn scope_list() -> Vec<PathBuf> {
    let root = root_dir();
    let test_dir = root.join("tests");
    let mut out = Vec::new();

    for entry in fs::read_dir(&test_dir).expect("read tests") {
        let path = entry.expect("entry").path();
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };

        if name.starts_with("test_e2e")
            && name.ends_with(".rs")
            && name != "test_e2e_public_path.rs"
        {
            out.push(path);
        }
    }

    out.sort();
    out.push(test_dir.join("test_s6_recv_examples.rs"));
    let sim_interop_dir = root.join("../z00z_simulator/examples/scenario_1");
    let mut sim_interop_files = fs::read_dir(&sim_interop_dir)
        .expect("read simulator interop example dir")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let name = path.file_name()?.to_str()?;
            let ext = path.extension().and_then(|ext| ext.to_str());
            ((name.starts_with("simulator_interop") && ext == Some("rs"))
                || name == "simulator_interop_support.inc")
                .then_some(path)
        })
        .collect::<Vec<_>>();
    sim_interop_files.sort();
    out.extend(sim_interop_files);
    let stealth_spec = root.join("../../specs/013-stealth-address/E2E-stealth-EXAMPLES.md");
    if stealth_spec.exists() {
        out.push(stealth_spec);
    } else {
        eprintln!(
            "skip missing optional doc scope: {}",
            stealth_spec.display()
        );
    }
    out
}

fn relative_path_string(path: &std::path::Path) -> String {
    let root = root_dir();
    path.strip_prefix(root.join("../.."))
        .or_else(|_| path.strip_prefix(&root))
        .map(|rel| rel.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}

fn scan_text(path: &std::path::Path, text: &str) -> String {
    if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
        return text.to_string();
    }

    let mut out = String::new();
    let mut in_code = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

fn has_ok(path: &std::path::Path, scan: &str, text: &str) -> bool {
    if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
        return true;
    }

    OK.iter().any(|ok| scan.contains(ok) || text.contains(ok))
}

#[test]
fn test_e2e_public_path() {
    let mut bad_hits = Vec::new();
    let mut miss_ok = Vec::new();

    for path in scope_list() {
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let rel = relative_path_string(&path);
        let scan = scan_text(&path, &text);

        for bad in BAD {
            if scan.contains(bad) {
                bad_hits.push(format!("{rel} -> {bad}"));
            }
        }

        if !has_ok(&path, &scan, &text) {
            miss_ok.push(rel);
        }
    }

    assert!(
        bad_hits.is_empty(),
        "forbidden stealth path(s):\n{}",
        bad_hits.join("\n")
    );
    assert!(
        miss_ok.is_empty(),
        "missing public stealth path(s):\n{}",
        miss_ok.join("\n")
    );
}
