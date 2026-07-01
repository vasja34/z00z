use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use serde::Deserialize;
use z00z_simulator::{config::ScenarioCfg, scenario_1::runner, StageResult};
use z00z_utils::{
    codec::{Codec, YamlCodec},
    io::{create_dir_all, load_json, write_file},
};

use z00z_simulator::scenario_1::support::fixture_cache;

#[derive(Debug, Deserialize)]
struct AuditLogRow {
    asset_id: String,
    action: String,
    reason_code: String,
}

fn mk_cfg_in(base: &Path, resume_fault: Option<&str>) -> (PathBuf, PathBuf, PathBuf) {
    create_dir_all(base).expect("create test base");

    let out = base.join("outputs/scenario_1");
    create_dir_all(&out).expect("create output dir");

    let mut cfg = ScenarioCfg::from_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_config.yaml"),
    )
    .expect("load scenario config");
    cfg.stage1_genesis
        .get_or_insert_with(Default::default)
        .genesis_config = z00z_core::config_paths::devnet_genesis_path()
        .to_string_lossy()
        .to_string();
    cfg.outputs.dir = out.to_string_lossy().to_string();

    if let Some(stage3) = cfg.stage3_claim.as_mut() {
        stage3.consume_bins = Some(false);
        stage3.resume_fault = resume_fault.map(ToString::to_string);
    }

    let cfg_path = base.join("scenario_config.yaml");
    let cfg_bytes = YamlCodec.serialize(&cfg).expect("serialize cfg");
    write_file(&cfg_path, &cfg_bytes).expect("write cfg");

    let design_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_design.yaml");
    (cfg_path, design_path, out)
}

fn replay_out() -> &'static PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| {
        let root =
            fixture_cache::ensure_shared_case("wallet_claim_replay_replay_first_v1", |base| {
                let (cfg_path, design_path, out) = mk_cfg_in(base, Some("replay_first"));
                let res = runner::run_with_paths(&cfg_path, &design_path).expect("scenario run");

                let st3 = res
                    .stages
                    .iter()
                    .find(|s| s.stage == 3)
                    .expect("stage 3 exists");
                assert!(matches!(st3.result, StageResult::Ok));
                assert!(
                    out.join("claim").join("audit_log.json").exists(),
                    "shared replay fixture must contain audit log"
                );
            });
        root.join("outputs/scenario_1")
    })
}

fn read_audit_rows(out: &Path) -> Vec<AuditLogRow> {
    let path = out.join("claim").join("audit_log.json");
    load_json(&path).expect("read audit_log.json")
}

#[test]
fn test_double_claim_audit_log() {
    let rows = read_audit_rows(replay_out());
    let mut by_asset: HashMap<&str, Vec<&AuditLogRow>> = HashMap::new();
    for row in &rows {
        by_asset.entry(row.asset_id.as_str()).or_default().push(row);
    }

    let mut has_pair = false;
    for rows in by_asset.values() {
        let has_new = rows
            .iter()
            .any(|row| row.action == "import_accepted" && row.reason_code == "IMPORT_ACCEPTED_NEW");
        let has_replay = rows.iter().any(|row| {
            row.action == "import_accepted" && row.reason_code == "IMPORT_ALREADY_EXISTS"
        });
        if has_new && has_replay {
            has_pair = true;
            break;
        }
    }

    assert!(
        has_pair,
        "audit log must contain both first claim and replay outcome for same asset"
    );
}
