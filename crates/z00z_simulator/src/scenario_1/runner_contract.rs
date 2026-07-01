use std::path::Path;

use crate::{DesignDoc, DesignErr, DesignStage};

use super::runner::Scenario1Err;

struct CanonicalStepSpec {
    id: &'static str,
    action: &'static str,
    post_conditions: &'static [&'static str],
}

struct CanonicalStageSpec {
    stage: u32,
    name: &'static str,
    description: &'static str,
    rust_entry: &'static str,
    config_source: &'static str,
    steps: &'static [CanonicalStepSpec],
}

const CANONICAL_STAGE_SPECS: [CanonicalStageSpec; 13] = include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/scenario_1/runner_contract_table.json"
));

pub(super) fn load_design(path: &Path) -> Result<DesignDoc, Scenario1Err> {
    let design = DesignDoc::from_file(path).map_err(Scenario1Err::Design)?;
    validate_canonical_design(&design)?;
    Ok(design)
}

fn validate_canonical_design(design: &DesignDoc) -> Result<(), Scenario1Err> {
    if design.stages.len() != CANONICAL_STAGE_SPECS.len() {
        return Err(design_invalid(format!(
            "scenario_1 design must contain {} stages, got {}",
            CANONICAL_STAGE_SPECS.len(),
            design.stages.len()
        )));
    }

    for (stage, spec) in design.stages.iter().zip(CANONICAL_STAGE_SPECS.iter()) {
        validate_canonical_stage(stage, spec)?;
    }

    Ok(())
}

fn validate_canonical_stage(
    stage: &DesignStage,
    spec: &CanonicalStageSpec,
) -> Result<(), Scenario1Err> {
    if stage.stage != spec.stage {
        return Err(design_invalid(format!(
            "scenario_1 stage order mismatch: expected stage {} got {}",
            spec.stage, stage.stage
        )));
    }
    if stage.name != spec.name {
        return Err(design_invalid(format!(
            "scenario_1 stage {} name mismatch: expected {} got {}",
            spec.stage, spec.name, stage.name
        )));
    }
    if stage.description.as_deref() != Some(spec.description) {
        return Err(design_invalid(format!(
            "scenario_1 stage {} description mismatch",
            spec.stage
        )));
    }
    if stage.rust_entry.as_deref() != Some(spec.rust_entry) {
        return Err(design_invalid(format!(
            "scenario_1 stage {} rust_entry mismatch",
            spec.stage
        )));
    }
    if stage.config_source.as_deref() != Some(spec.config_source) {
        return Err(design_invalid(format!(
            "scenario_1 stage {} config_source mismatch",
            spec.stage
        )));
    }

    if stage.steps.len() != spec.steps.len() {
        return Err(design_invalid(format!(
            "scenario_1 stage {} step count mismatch",
            spec.stage
        )));
    }

    for (step, spec_step) in stage.steps.iter().zip(spec.steps.iter()) {
        validate_canonical_step(spec.stage, step, spec_step)?;
    }

    Ok(())
}

fn validate_canonical_step(
    stage_id: u32,
    step: &crate::DesignStep,
    spec: &CanonicalStepSpec,
) -> Result<(), Scenario1Err> {
    if step.id != spec.id {
        return Err(design_invalid(format!(
            "scenario_1 stage {} step id mismatch: expected {} got {}",
            stage_id, spec.id, step.id
        )));
    }
    if step.action != spec.action {
        return Err(design_invalid(format!(
            "scenario_1 stage {} step {} action mismatch",
            stage_id, spec.id
        )));
    }

    let actual_post: Vec<&str> = step
        .post_conditions
        .iter()
        .map(|text| text.as_str())
        .collect();
    if actual_post != spec.post_conditions {
        return Err(design_invalid(format!(
            "scenario_1 stage {} step {} post_conditions mismatch",
            stage_id, spec.id
        )));
    }

    Ok(())
}

fn design_invalid(msg: String) -> Scenario1Err {
    Scenario1Err::Design(DesignErr::Invalid(msg))
}
