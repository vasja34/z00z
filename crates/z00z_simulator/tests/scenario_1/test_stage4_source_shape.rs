use z00z_simulator::{DesignDoc, DesignStage};

use z00z_simulator::scenario_1::stage_6::shared_cases;
use z00z_simulator::scenario_1::support::scenario_support;

use scenario_support::make_cfg;

const STAGE5_MOD_RS: &str = include_str!("../../src/scenario_1/stage_5/mod.rs");
const STAGE5_TX_PLAN_RS: &str = include_str!("../../src/scenario_1/stage_5/tx_plan.rs");
const STAGE6_RS: &str = include_str!("../../src/scenario_1/stage_6/mod.rs");
const STAGE6_TX_LANE_RS: &str = include_str!("../../src/scenario_1/stage_6/tx_lane_impl.rs");
const STAGE9_RS: &str = include_str!("../../src/scenario_1/stage_9/mod.rs");
const STAGE10_RS: &str = include_str!("../../src/scenario_1/stage_10/mod.rs");
const STAGE11_RS: &str = include_str!("../../src/scenario_1/stage_11/mod.rs");
const STAGE12_RS: &str = include_str!("../../src/scenario_1/stage_12/mod.rs");
const RUNNER_RS: &str = include_str!("../../src/scenario_1/runner.rs");

fn assert_surface_routes() {
    let retired_stage4_lane = ["tx", "lane", "impl"].join("_");
    let retired_stage5_proxy = "super::stage_4::run_tx_plan";
    let retired_stage6_proxy = "super::stage_4::run_tx_prepare";

    assert!(STAGE5_MOD_RS.contains("mod tx_plan;"));
    assert!(STAGE5_MOD_RS.contains("pub use tx_plan::run_tx_plan;"));
    assert!(STAGE5_TX_PLAN_RS.contains("\"P5-1\""));
    assert!(STAGE5_TX_PLAN_RS.contains("\"P5-2\""));
    assert!(STAGE5_TX_PLAN_RS.contains("\"P5-3\""));
    assert!(STAGE5_TX_PLAN_RS.contains("stage_6::paths::resolve_stage4_paths"));
    assert!(!STAGE5_TX_PLAN_RS.contains(retired_stage4_lane.as_str()));
    assert!(!STAGE5_TX_PLAN_RS.contains(retired_stage5_proxy));

    assert!(STAGE6_RS.contains("mod tx_lane_impl;"));
    assert!(STAGE6_RS.contains("pub use tx_lane_impl::run_tx_prepare;"));
    assert!(STAGE6_RS.contains("pub(crate) mod paths;"));
    assert!(STAGE6_TX_LANE_RS.contains("pub fn run_tx_prepare"));
    assert!(!STAGE6_RS.contains(retired_stage6_proxy));

    assert!(STAGE9_RS.contains("super::stage_6::find_actor"));
    assert!(STAGE10_RS.contains("pub fn run_bundle_publish"));
    assert!(STAGE10_RS.contains("fn push_log("));
    assert!(STAGE10_RS.contains("fn flush_logs("));
    assert!(!STAGE10_RS.contains("stage_9::logging"));
    assert!(STAGE11_RS.contains("pub fn run_apply"));
    assert!(RUNNER_RS.contains("stage_6::run_tx_prepare"));
    assert!(STAGE12_RS.contains("pub fn run_finalize"));
    assert!(STAGE12_RS.contains("fn push_log("));
    assert!(STAGE12_RS.contains("fn flush_logs("));
    assert!(!STAGE12_RS.contains("stage_9::logging"));
}

fn step_ids(stage: &DesignStage) -> Vec<&str> {
    stage.steps.iter().map(|step| step.id.as_str()).collect()
}

#[test]
fn test_stage4_surface_tx_split() {
    let (_cfg_path, design_path, _out_dir) = make_cfg(|_| {});

    let design = DesignDoc::from_file(&design_path).expect("load design");
    let names: Vec<(u32, String)> = design
        .stages
        .iter()
        .map(|stage| (stage.stage, stage.name.clone()))
        .collect();

    assert_eq!(names[4], (5, "tx_plan".to_string()));
    assert_eq!(names[5], (6, "tx_prepare".to_string()));
    assert_eq!(
        design.stages[4].rust_entry.as_deref(),
        Some("stage_5::run_tx_plan(ctx, stage)")
    );
    assert_eq!(
        design.stages[5].rust_entry.as_deref(),
        Some("stage_6::run_tx_prepare(ctx, stage)")
    );
    assert_eq!(
        step_ids(&design.stages[3]),
        vec!["P4-1", "P4-2", "P4-3", "P4-4"]
    );
    assert_eq!(step_ids(&design.stages[4]), vec!["P5-1", "P5-2", "P5-3"]);
    assert_eq!(
        step_ids(&design.stages[5]),
        vec![
            "S4-1", "S4-2", "S4-12", "S4-3", "S4-4", "S4-5", "S4-9", "S4-10", "S4-6", "S4-7",
            "S4-8", "S4-C1", "S4-11", "S4-13"
        ]
    );

    let out = shared_cases::default_stage6_out();
    assert!(
        out.join("transactions/tx_alice_to_bob_pkg.json").exists(),
        "shared stage4 fixture must contain tx package"
    );

    assert_surface_routes();
}
