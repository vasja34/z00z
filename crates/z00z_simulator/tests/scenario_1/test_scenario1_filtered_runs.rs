use std::{path::PathBuf, sync::OnceLock};

use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::read_file,
};

use z00z_simulator::{
    scenario_1::support::{fixture_cache, scenario_support, stage_runner_support},
    scenario_1::{stage_10, stage_11, stage_12, stage_7, stage_8, stage_9},
    StageResult,
};

const STAGE9_SRC: &str = include_str!("../../src/scenario_1/stage_9/mod.rs");
const STAGE10_SRC: &str = include_str!("../../src/scenario_1/stage_10/mod.rs");
const STAGE11_SRC: &str = include_str!("../../src/scenario_1/stage_11/mod.rs");
const STAGE12_SRC: &str = include_str!("../../src/scenario_1/stage_12/mod.rs");
const PIPELINE_DOC: &str =
    include_str!("../../../../wiki/06-simulator-and-quality/scenario-pipeline.md");

static FILTERED_STAGE_OUT: OnceLock<PathBuf> = OnceLock::new();

fn filtered_stage_out() -> &'static PathBuf {
    FILTERED_STAGE_OUT.get_or_init(|| {
        let root =
            fixture_cache::ensure_shared_case_precise("scenario1_filtered_stage_lane_v1", |base| {
                let (cfg_path, design_path, out) = scenario_support::make_cfg_in(base, |_| {});
                let mut session = stage_runner_support::run_stage4_session(&cfg_path, &design_path);
                for stage_id in [7_u32, 8, 9, 10, 11, 12] {
                    let stage = stage_runner_support::stage_by_id(&design_path, stage_id);
                    let result = match stage_id {
                        7 => stage_7::run_transfer_receive(&mut session, &stage),
                        8 => stage_8::run_transfer_claim(&mut session, &stage),
                        9 => stage_9::run_bundle_build(&mut session, &stage),
                        10 => stage_10::run_bundle_publish(&mut session, &stage),
                        11 => stage_11::run_apply(&mut session, &stage),
                        12 => stage_12::run_finalize(&mut session, &stage),
                        _ => unreachable!(),
                    };
                    assert!(
                        matches!(result, StageResult::Ok),
                        "stage {stage_id} must succeed: {result:?}"
                    );
                }

                let s8 = load_json(&out.join("transactions/checkpoint_s8.json"));
                assert_eq!(s8["status"].as_str(), Some("ok"));
                assert_eq!(s8["checkpoint_id_hex"].as_str().map(str::len), Some(64));
                assert_eq!(
                    s8["artifact_path"].as_str(),
                    Some("transactions/checkpoint/artifact")
                );
            });
        root.join("outputs/scenario_1")
    })
}

fn load_json(path: &std::path::Path) -> serde_json::Value {
    JsonCodec
        .deserialize(&read_file(path).expect("read json"))
        .expect("decode json")
}

#[test]
fn test_filtered_stage_lane_has_no_fallback_events() {
    let out = filtered_stage_out();
    let logger_path = out.join("logs/logger.json");
    let body = std::fs::read_to_string(&logger_path).expect("read stage logger");

    assert!(
        !body.contains("covered by stage fallback"),
        "canonical stages 9-12 must not report fallback coverage"
    );
    assert!(
        !body.contains("\"event\":\"step_stub\""),
        "canonical stages 9-12 must not emit synthetic step coverage rows"
    );
}

#[test]
fn test_filtered_stage_lane_keeps_default_finalization() {
    let out = filtered_stage_out();
    let s8 = load_json(&out.join("transactions/checkpoint_s8.json"));

    assert_eq!(s8["status"].as_str(), Some("ok"));
    assert_eq!(s8["checkpoint_id_hex"].as_str().map(str::len), Some(64));
    assert_eq!(
        s8["audit_path"].as_str(),
        Some("transactions/checkpoint/audit")
    );
}

#[test]
fn test_filtered_stage_lane_uses_stable_owner_facades() {
    for source in [STAGE9_SRC, STAGE10_SRC, STAGE11_SRC, STAGE12_SRC] {
        assert!(
            !source.contains("z00z_wallets::services::"),
            "scenario harness must not deep-import wallet services"
        );
        assert!(
            !source.contains("z00z_wallets::redb_store::"),
            "scenario harness must not deep-import wallet storage internals"
        );
        assert!(
            !source.contains("z00z_storage::backend::"),
            "scenario harness must stay on stable storage facades"
        );
    }

    assert!(
        PIPELINE_DOC.contains("Use stable facades"),
        "phase authority docs must keep the stable-facade rule explicit"
    );
}
