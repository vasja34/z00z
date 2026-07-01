use std::{path::PathBuf, sync::OnceLock};

use serde_json::Value;
use z00z_simulator::scenario_1::stage_3::Stage3Snapshot;
use z00z_utils::io::load_json;

use z00z_simulator::scenario_1::support::claim_shared_cases;

static SNAPSHOT_OUT: OnceLock<PathBuf> = OnceLock::new();

fn snapshot_out() -> &'static PathBuf {
    SNAPSHOT_OUT.get_or_init(|| claim_shared_cases::stage6_out("claim_snapshot"))
}

#[test]
fn test_snapshot_reconciliation_run() {
    let out = snapshot_out();

    let snapshot_path = out.join("stage_3_snapshot.json");
    let snapshot: Stage3Snapshot = load_json(&snapshot_path).expect("load stage3 snapshot");
    assert_eq!(snapshot.stage, 3);

    let publish_audit: Value = load_json(out.join("claim_publish").join("audit_log.json"))
        .expect("load claim publish audit");
    assert_eq!(publish_audit.get("stage").and_then(|v| v.as_u64()), Some(4));
    assert_eq!(
        publish_audit
            .get("source_snapshot_file")
            .and_then(|v| v.as_str()),
        Some("stage_3_snapshot.json")
    );
    assert_eq!(
        publish_audit.get("status").and_then(|v| v.as_str()),
        Some("ok")
    );

    let publish_snapshot: Value =
        load_json(out.join("stage_4_snapshot.json")).expect("load stage4 snapshot");
    assert_eq!(
        publish_snapshot.get("stage").and_then(|v| v.as_u64()),
        Some(6)
    );
    assert_eq!(
        publish_snapshot.get("status").and_then(|v| v.as_str()),
        Some("ok")
    );
}
