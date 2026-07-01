//! Shared scenario design types and loader.

use std::{collections::HashSet, path::Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use z00z_utils::codec::{Codec, CodecError, YamlCodec};
use z00z_utils::io;

/// Root design document parsed from `scenario_design.yaml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignDoc {
    /// Ordered stage list.
    pub stages: Vec<DesignStage>,
}

/// One scenario stage declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignStage {
    /// Numeric stage id.
    pub stage: u32,
    /// Stage name.
    pub name: String,
    /// Optional stage description.
    #[serde(default)]
    pub description: Option<String>,
    /// Optional Rust entry hint.
    #[serde(default)]
    pub rust_entry: Option<String>,
    /// Optional config source hint.
    #[serde(default)]
    pub config_source: Option<String>,
    /// Stage steps.
    #[serde(default)]
    pub steps: Vec<DesignStep>,
}

/// One design step inside a stage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignStep {
    /// Stable step id (for reports).
    pub id: String,
    /// Human action summary.
    pub action: String,
    /// Optional call hint.
    #[serde(default)]
    pub call: Option<String>,
    /// Optional note.
    #[serde(default)]
    pub note: Option<String>,
    /// Expected post conditions.
    #[serde(default)]
    pub post_conditions: Vec<String>,
}

/// Design loading and validation errors.
#[derive(Debug, Error)]
pub enum DesignErr {
    /// Failed to read design file.
    #[error("failed to read design yaml: {0}")]
    Io(#[from] z00z_utils::io::IoError),
    /// Failed to decode design file.
    #[error("failed to decode design yaml: {0}")]
    Decode(#[from] CodecError),
    /// Design consistency validation failed.
    #[error("invalid design: {0}")]
    Invalid(String),
}

impl DesignDoc {
    /// Loads design from YAML file and validates structure.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, DesignErr> {
        let bytes = io::read_file(path.as_ref())?;
        let codec = YamlCodec;
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let parsed = std::panic::catch_unwind(|| codec.deserialize::<Self>(&bytes));
        std::panic::set_hook(prev_hook);
        let doc = match parsed {
            Ok(Ok(doc)) => doc,
            _ => Self::parse_stage_outline(&bytes)?,
        };
        doc.validate()?;
        Ok(doc)
    }

    fn parse_stage_outline(bytes: &[u8]) -> Result<Self, DesignErr> {
        let text = std::str::from_utf8(bytes)
            .map_err(|e| DesignErr::Decode(CodecError::Yaml(e.to_string())))?;

        let mut stages = Vec::new();
        let mut current: Option<DesignStage> = None;

        for line in text.lines() {
            let trimmed = line.trim_start();
            let indent = line.len().saturating_sub(trimmed.len());

            if let Some(stage_id) = parse_outline_stage_id(trimmed, indent)? {
                if let Some(stage) = current.take() {
                    stages.push(stage);
                }

                current = Some(DesignStage {
                    stage: stage_id,
                    name: String::new(),
                    description: None,
                    rust_entry: None,
                    config_source: None,
                    steps: Vec::new(),
                });
                continue;
            }

            if let Some(stage) = current.as_mut() {
                update_outline_stage_name(stage, trimmed, indent);
            }
        }

        if let Some(stage) = current.take() {
            stages.push(stage);
        }

        if stages.is_empty() {
            return Err(DesignErr::Invalid(
                "stages must not be empty (outline parser)".to_string(),
            ));
        }

        Ok(Self { stages })
    }
    /// Validates stage/step identity and required fields.
    pub fn validate(&self) -> Result<(), DesignErr> {
        if self.stages.is_empty() {
            return Err(DesignErr::Invalid("stages must not be empty".to_string()));
        }

        let mut stage_ids = HashSet::new();
        for stage in &self.stages {
            validate_stage_header(stage, &mut stage_ids)?;

            let mut step_ids = HashSet::new();
            for step in &stage.steps {
                validate_step(stage.stage, step, &mut step_ids)?;
            }
        }

        Ok(())
    }
}

fn parse_outline_stage_id(trimmed: &str, indent: usize) -> Result<Option<u32>, DesignErr> {
    if indent > 2 || !trimmed.starts_with("- stage:") {
        return Ok(None);
    }

    let raw = trimmed.trim_start_matches("- stage:").trim();
    let stage_id = raw
        .parse::<u32>()
        .map_err(|_| DesignErr::Invalid(format!("invalid stage id in design outline: {}", raw)))?;
    Ok(Some(stage_id))
}

fn update_outline_stage_name(stage: &mut DesignStage, trimmed: &str, indent: usize) {
    if indent <= 4 && trimmed.starts_with("name:") {
        let raw = trimmed.trim_start_matches("name:").trim();
        stage.name = raw.trim_matches('"').trim_matches('\'').to_string();
    }
}

fn validate_stage_header(
    stage: &DesignStage,
    stage_ids: &mut HashSet<u32>,
) -> Result<(), DesignErr> {
    if !stage_ids.insert(stage.stage) {
        return Err(DesignErr::Invalid(format!(
            "duplicate stage id: {}",
            stage.stage
        )));
    }
    if stage.name.trim().is_empty() {
        return Err(DesignErr::Invalid(format!(
            "stage {} has empty name",
            stage.stage
        )));
    }
    if stage
        .description
        .as_deref()
        .is_none_or(|text| text.trim().is_empty())
    {
        return Err(DesignErr::Invalid(format!(
            "stage {} has empty description",
            stage.stage
        )));
    }
    if stage
        .rust_entry
        .as_deref()
        .is_none_or(|text| text.trim().is_empty())
    {
        return Err(DesignErr::Invalid(format!(
            "stage {} has empty rust_entry",
            stage.stage
        )));
    }
    if stage
        .config_source
        .as_deref()
        .is_none_or(|text| text.trim().is_empty())
    {
        return Err(DesignErr::Invalid(format!(
            "stage {} has empty config_source",
            stage.stage
        )));
    }
    if stage.steps.is_empty() {
        return Err(DesignErr::Invalid(format!(
            "stage {} has no steps",
            stage.stage
        )));
    }
    Ok(())
}

fn validate_step(
    stage_id: u32,
    step: &DesignStep,
    step_ids: &mut HashSet<String>,
) -> Result<(), DesignErr> {
    if step.id.trim().is_empty() {
        return Err(DesignErr::Invalid(format!(
            "stage {} has step with empty id",
            stage_id
        )));
    }
    if !step_ids.insert(step.id.clone()) {
        return Err(DesignErr::Invalid(format!(
            "stage {} has duplicate step id: {}",
            stage_id, step.id
        )));
    }
    if step.action.trim().is_empty() {
        return Err(DesignErr::Invalid(format!(
            "stage {} step {} has empty action",
            stage_id, step.id
        )));
    }
    Ok(())
}
