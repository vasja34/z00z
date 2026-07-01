use z00z_utils::io::create_dir_all;

use crate::{DesignStage, SimContext, StageResult};

pub fn run_tx_plan(ctx: &mut SimContext, stage: &DesignStage) -> StageResult {
    match run_plan(ctx, stage) {
        Ok(()) => StageResult::Ok,
        Err(err) => StageResult::Fail(format!(
            "stage {} ({}) failed: {}",
            stage.stage, stage.name, err
        )),
    }
}

fn run_plan(ctx: &mut SimContext, stage: &DesignStage) -> Result<(), String> {
    let cfg = crate::scenario_1::stage_6::tx_validation_gates::stage4_cfg(ctx)?;
    crate::scenario_1::stage_6::tx_validation_gates::validate_stage4_cfg(ctx, cfg)?;

    let paths = crate::scenario_1::stage_6::paths::resolve_stage4_paths(ctx, cfg);
    create_dir_all(&paths.logs_dir).map_err(|e| e.to_string())?;

    let mut lines = Vec::new();
    crate::scenario_1::stage_6::reporting::push_log(
        &mut lines,
        stage.stage,
        "P5-1",
        "validate_config",
        "ok",
        "stage4_tx_prepare enabled with logged_local transport",
    )?;
    crate::scenario_1::stage_6::reporting::push_log(
        &mut lines,
        stage.stage,
        "P5-2",
        "validate_plan_refs",
        "ok",
        &crate::scenario_1::stage_6::tx_validation_gates::stage4_flags_summary(cfg),
    )?;
    crate::scenario_1::stage_6::reporting::push_log(
        &mut lines,
        stage.stage,
        "P5-3",
        "defer_mutation",
        "ok",
        "stage 6 owns tx package and tx-history writes",
    )?;
    crate::scenario_1::stage_6::reporting::flush_logs(&paths.logger_file, &lines)?;
    Ok(())
}
