//! Scenario 1 stage 13 facade: HJMT settlement examples.

use std::path::{Component, Path, PathBuf};

use crate::config::Stage13HjmtCfg;
use crate::{DesignStage, SimContext, StageResult};

pub(crate) mod hjmt_examples;
pub(crate) mod report;
pub(crate) mod scan;
pub mod shared_cases;
pub(crate) mod storage;
pub(crate) mod tamper;

#[cfg(test)]
mod test_mod;

pub fn run_hjmt_examples(ctx: &mut SimContext, stage: &DesignStage) -> StageResult {
    match run_stage13_hjmt(ctx, stage) {
        Ok(()) => StageResult::Ok,
        Err(err) => StageResult::Fail(format!(
            "stage {} ({}) failed: {}",
            stage.stage, stage.name, err
        )),
    }
}

pub(crate) fn run_stage13_hjmt(ctx: &mut SimContext, stage: &DesignStage) -> Result<(), String> {
    let cfg = ctx
        .config
        .stage13_hjmt_settlement_examples
        .clone()
        .ok_or_else(|| "stage13_hjmt_settlement_examples config missing".to_string())?;

    validate_contract_cfg_fields(&cfg)?;
    let paths = build_paths(&ctx.outputs_dir, &cfg)?;
    hjmt_examples::generate(ctx, stage, &cfg, &paths)
}

pub(crate) fn validate_contract_cfg_fields(cfg: &Stage13HjmtCfg) -> Result<(), String> {
    if !cfg.enabled {
        return Err("stage13_hjmt_settlement_examples must stay enabled".to_string());
    }
    if cfg.backend_modes.is_empty() {
        return Err("stage13 backend_modes must not be empty".to_string());
    }
    for mode in &cfg.backend_modes {
        match mode.as_str() {
            "generalized" | "adaptive" => {}
            other => {
                return Err(format!(
                    "stage13 backend_modes contains unsupported mode: {other}"
                ));
            }
        }
    }
    if !cfg.backend_modes.iter().any(|mode| mode == "generalized") {
        return Err("stage13 backend_modes must include generalized".to_string());
    }
    if !cfg.backend_modes.iter().any(|mode| mode == "adaptive") {
        return Err("stage13 backend_modes must include adaptive".to_string());
    }
    if cfg.expected_right_classes.is_empty() {
        return Err("stage13 expected_right_classes must not be empty".to_string());
    }
    for (field, value) in [
        ("rights_manifest_file", cfg.rights_manifest_file.as_str()),
        ("output_dir", cfg.output_dir.as_str()),
        ("examples_file", cfg.examples_file.as_str()),
        ("tamper_report_file", cfg.tamper_report_file.as_str()),
        (
            "proof_size_report_file",
            cfg.proof_size_report_file.as_str(),
        ),
        (
            "cache_scheduler_metrics_file",
            cfg.cache_scheduler_metrics_file.as_str(),
        ),
        ("replay_roots_file", cfg.replay_roots_file.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!("stage13 {field} must not be empty"));
        }
    }
    Ok(())
}

pub(crate) fn resolve_stage13_path(base: &Path, rel: &str, field: &str) -> Result<PathBuf, String> {
    let candidate = PathBuf::from(rel);
    if candidate.is_absolute() {
        return Err(format!(
            "stage13 {field} must stay inside the scenario outputs sandbox"
        ));
    }
    if candidate
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(format!("stage13 {field} must not use parent segments"));
    }
    Ok(base.join(candidate))
}

fn build_paths(
    outputs_dir: &Path,
    cfg: &Stage13HjmtCfg,
) -> Result<hjmt_examples::Stage13Paths, String> {
    let output_dir = resolve_stage13_path(outputs_dir, &cfg.output_dir, "output_dir")?;
    let examples_path = resolve_stage13_path(outputs_dir, &cfg.examples_file, "examples_file")?;
    let tamper_path =
        resolve_stage13_path(outputs_dir, &cfg.tamper_report_file, "tamper_report_file")?;
    let proof_size_path = resolve_stage13_path(
        outputs_dir,
        &cfg.proof_size_report_file,
        "proof_size_report_file",
    )?;
    let cache_metrics_path = resolve_stage13_path(
        outputs_dir,
        &cfg.cache_scheduler_metrics_file,
        "cache_scheduler_metrics_file",
    )?;
    let replay_roots_path =
        resolve_stage13_path(outputs_dir, &cfg.replay_roots_file, "replay_roots_file")?;
    let manifest_src = resolve_stage13_path(
        outputs_dir,
        &cfg.rights_manifest_file,
        "rights_manifest_file",
    )?;
    let manifest_dst = output_dir.join("genesis_settlement_manifest.json");

    Ok(hjmt_examples::Stage13Paths {
        output_dir: output_dir.clone(),
        store_dir: output_dir.join("store"),
        manifest_src,
        manifest_dst,
        examples_path,
        tamper_path,
        proof_size_path,
        cache_metrics_path,
        replay_roots_path,
        logger_path: resolve_stage13_path(outputs_dir, report::STAGE13_LOG_FILE, "logger_path")?,
    })
}
