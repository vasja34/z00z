use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::Deserialize;
use z00z_utils::io::{load_json, read_to_string};

#[derive(Debug, Deserialize)]
struct AuditLogRow {
    timestamp: u64,
    wallet_id: String,
    asset_id: String,
    action: String,
    reason_code: String,
}

use z00z_simulator::scenario_1::support::claim_shared_cases;

struct AuditRunArtifacts {
    out: PathBuf,
    expected: usize,
}

fn load_case(case_name: &str, resume_fault: Option<&str>) -> AuditRunArtifacts {
    let out = match (case_name, resume_fault) {
        ("claim_audit_log_base_v1", None) => claim_shared_cases::default_stage3_out(),
        ("claim_audit_log_reject_v1", Some("reject_first")) => {
            claim_shared_cases::reject_first_stage3_out()
        }
        _ => panic!("unexpected audit case: {case_name:?} / {resume_fault:?}"),
    };
    let snap: serde_json::Value =
        load_json(out.join("stage_3_snapshot.json")).expect("load stage_3_snapshot");
    let expected = snap
        .get("distributed_assets_count")
        .and_then(|v| v.as_u64())
        .expect("distributed_assets_count") as usize;

    AuditRunArtifacts { out, expected }
}

fn base_case() -> &'static AuditRunArtifacts {
    static BASE: OnceLock<AuditRunArtifacts> = OnceLock::new();
    BASE.get_or_init(|| load_case("claim_audit_log_base_v1", None))
}

fn reject_case() -> &'static AuditRunArtifacts {
    static REJECT: OnceLock<AuditRunArtifacts> = OnceLock::new();
    REJECT.get_or_init(|| load_case("claim_audit_log_reject_v1", Some("reject_first")))
}

fn read_audit_rows(out: &Path) -> Vec<AuditLogRow> {
    let path = out.join("claim").join("audit_log.json");
    load_json(&path).expect("read audit_log.json")
}

#[test]
fn test_audit_log_complete() {
    let case = base_case();

    let rows = read_audit_rows(&case.out);
    assert_eq!(
        rows.len(),
        case.expected,
        "one audit row per claim decision"
    );

    for row in rows {
        assert!(row.timestamp > 0);
        assert!(!row.wallet_id.is_empty());
        assert_eq!(row.asset_id.len(), 64);
        assert!(row.asset_id.chars().all(|ch| ch.is_ascii_hexdigit()));
        assert_eq!(row.action, "import_accepted");
        assert_eq!(row.reason_code, "IMPORT_ACCEPTED_NEW");
    }
}

#[test]
fn test_audit_log_no_secrets() {
    let case = base_case();

    let path = case.out.join("claim").join("audit_log.json");
    let text = read_to_string(&path).expect("read audit_log.json");
    let text = text.to_ascii_lowercase();

    for secret_pattern in [
        "private_key",
        "secret_key",
        "seed phrase",
        "mnemonic",
        "blinding",
        "\"secret\":",
        "secret_bytes",
    ] {
        assert!(
            !text.contains(secret_pattern),
            "audit log leaked secret pattern: {secret_pattern}"
        );
    }
}

#[test]
fn test_audit_order_by_wallet() {
    let case = base_case();

    let rows = read_audit_rows(&case.out);
    assert!(!rows.is_empty(), "audit log must not be empty");

    let mut seen = std::collections::BTreeSet::new();
    let mut prev_wallet = String::new();

    for row in rows {
        if row.wallet_id != prev_wallet {
            assert!(
                !seen.contains(&row.wallet_id),
                "wallet_id blocks must not interleave"
            );
            seen.insert(row.wallet_id.clone());
            prev_wallet = row.wallet_id;
        }
    }

    assert!(
        seen.len() <= 3,
        "expected at most three actor wallet blocks, got {}",
        seen.len()
    );
}

#[test]
fn test_audit_log_reject_traceable() {
    let case = reject_case();

    let rows = read_audit_rows(&case.out);
    let has_reject = rows
        .iter()
        .any(|row| row.action == "import_rejected" && row.reason_code.starts_with("IMPORT_"));
    assert!(has_reject, "reject row with reason code is required");
}
