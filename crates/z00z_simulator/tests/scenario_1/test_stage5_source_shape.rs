use std::path::Path;

use z00z_simulator::{DesignDoc, DesignStage};
use z00z_utils::io::read_to_string;

use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::stage_runner_support;

use stage_runner_support::make_cfg;

fn step_ids(stage: &DesignStage) -> Vec<&str> {
    stage.steps.iter().map(|step| step.id.as_str()).collect()
}

fn assert_log_stages(path: &Path, expected: &[u32]) {
    let rows = read_to_string(path)
        .unwrap_or_else(|err| panic!("read log {} failed: {err}", path.display()));
    for stage_id in expected {
        assert!(
            rows.contains(&format!("\"stage\":{stage_id}")),
            "log {} missing stage {stage_id}",
            path.display()
        );
    }
}

fn assert_surface_contract(design: &DesignDoc) {
    let names: Vec<(u32, String)> = design
        .stages
        .iter()
        .map(|stage| (stage.stage, stage.name.clone()))
        .collect();

    assert_eq!(names[6], (7, "transfer_receive".to_string()));
    assert_eq!(names[7], (8, "transfer_claim".to_string()));
    assert_eq!(names[8], (9, "bundle_build".to_string()));
    assert_eq!(names[9], (10, "bundle_publish".to_string()));
    assert_eq!(
        design.stages[6].rust_entry.as_deref(),
        Some("stage_7::run_transfer_receive(ctx, stage)")
    );
    assert_eq!(
        design.stages[7].rust_entry.as_deref(),
        Some("stage_8::run_transfer_claim(ctx, stage)")
    );
    assert_eq!(
        design.stages[8].rust_entry.as_deref(),
        Some("stage_9::run_bundle_build(ctx, stage)")
    );
    assert_eq!(
        design.stages[9].rust_entry.as_deref(),
        Some("stage_10::run_bundle_publish(ctx, stage)")
    );
    assert_eq!(
        design.stages[10].rust_entry.as_deref(),
        Some("stage_11::run_apply(ctx, stage)")
    );
    assert_eq!(
        design.stages[11].rust_entry.as_deref(),
        Some("stage_12::run_finalize(ctx, stage)")
    );
}

fn assert_split_steps(design: &DesignDoc) {
    assert_eq!(
        step_ids(&design.stages[6]),
        vec!["S5-1", "S5-2", "S5-3", "S5-4", "S5-5", "S5-6"]
    );
    assert_eq!(
        step_ids(&design.stages[7]),
        vec!["S5-1", "S5-2", "S5-3", "S5-4", "S5-6", "S5-7", "S5-8"]
    );
    assert_eq!(
        step_ids(&design.stages[8]),
        vec!["S6-1", "S6-2", "S6-3", "S6-4", "S6-5", "S6-6", "S6-7", "S6-8"]
    );
    assert_eq!(
        step_ids(&design.stages[9]),
        vec!["S6-1", "S6-2", "S6-3", "S6-4", "S6-5", "S6-6", "S6-7", "S6-8"]
    );
    assert_eq!(
        step_ids(&design.stages[10]),
        vec!["S7-1", "S7-2", "S7-3", "S7-4"]
    );
    assert_eq!(
        step_ids(&design.stages[11]),
        vec!["S8-1", "S8-2", "S8-3", "S8-4"]
    );
}

#[test]
fn test_stage5_surface_transfer_split() {
    let (_cfg_path, design_path, _out_dir) = make_cfg(|_| {});

    let design = DesignDoc::from_file(&design_path).expect("load design");
    assert_surface_contract(&design);
    assert_split_steps(&design);

    let root = fixture_cache::ensure_shared_case("stage5_source_shape_default_shared_v1", |base| {
        let (cfg_path, design_path, out_dir) = stage_runner_support::make_cfg_in(base, |_| {});
        let _ctx = stage_runner_support::run_stage_setup(
            &cfg_path,
            &design_path,
            &[1_u32, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        );
        assert!(out_dir.exists(), "shared stage5 fixture output dir missing");
    });
    let out_dir = root.join("outputs/scenario_1");
    assert!(
        out_dir.exists(),
        "shared stage5 fixture output dir missing after cache restore"
    );
}

#[test]
fn test_stage5_coverage_owned_logs() {
    let root = fixture_cache::ensure_shared_case("stage5_source_shape_logs_shared_v1", |base| {
        let (cfg_path, design_path, out_dir) = stage_runner_support::make_cfg_in(base, |cfg| {
            let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
            stage4.paths.logs_dir = "tx_logs".to_string();
            stage4.paths.logger_file = "tx_logs/tx_logger.json".to_string();

            let stage5 = cfg.stage5_transfer.as_mut().expect("stage5 cfg");
            stage5.paths.logs_dir = "transfer_logs".to_string();
            stage5.paths.logger_file = "transfer_logger.json".to_string();

            let stage6 = cfg.stage6_bundle.as_mut().expect("stage6 cfg");
            stage6.paths.logs_dir = "bundle_logs".to_string();
            stage6.paths.logger_file = "bundle_logger.json".to_string();

            let stage7 = cfg.stage7_apply.as_mut().expect("stage7 cfg");
            stage7.paths.logs_dir = "apply_logs".to_string();
            stage7.paths.logger_file = "apply_logger.json".to_string();

            let stage8 = cfg.stage8_finalize.as_mut().expect("stage8 cfg");
            stage8.paths.logs_dir = "finalize_logs".to_string();
            stage8.paths.logger_file = "finalize_logger.json".to_string();
        });
        let _ctx = stage_runner_support::run_stage_setup(
            &cfg_path,
            &design_path,
            &[1_u32, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        );
        assert!(
            out_dir
                .join("transfer_logs")
                .join("transfer_logger.json")
                .exists(),
            "shared stage5 transfer log missing"
        );
    });
    let out_dir = root.join("outputs/scenario_1");

    assert_log_stages(&out_dir.join("tx_logs").join("tx_logger.json"), &[5, 6]);
    assert_log_stages(
        &out_dir.join("transfer_logs").join("transfer_logger.json"),
        &[7, 8],
    );
    assert_log_stages(
        &out_dir.join("bundle_logs").join("bundle_logger.json"),
        &[9, 10],
    );
    assert_log_stages(&out_dir.join("apply_logs").join("apply_logger.json"), &[11]);
    assert_log_stages(
        &out_dir.join("finalize_logs").join("finalize_logger.json"),
        &[12],
    );
}
