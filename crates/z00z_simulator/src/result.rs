//! Shared stage and scenario results.

/// Stage execution result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StageResult {
    /// Stage completed successfully.
    Ok,
    /// Stage completed with warning.
    Warn(String),
    /// Stage failed.
    Fail(String),
}

/// Stage output state tracked by scenario runner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageState {
    /// Stage id from design YAML.
    pub stage: u32,
    /// Stage name.
    pub name: String,
    /// Stage execution result.
    pub result: StageResult,
}

/// Full scenario result aggregated across stages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioResult {
    /// Scenario id from config YAML.
    pub scenario_id: u32,
    /// Per-stage outcomes in execution order.
    pub stages: Vec<StageState>,
    /// True when run stopped on critical failure.
    pub is_aborted: bool,
}

impl ScenarioResult {
    /// Creates empty scenario result.
    pub fn new(scenario_id: u32) -> Self {
        Self {
            scenario_id,
            stages: Vec::new(),
            is_aborted: false,
        }
    }

    /// Returns true when all stages ended with `Ok`.
    pub fn is_ok(&self) -> bool {
        self.stages
            .iter()
            .all(|item| matches!(item.result, StageResult::Ok))
            && !self.is_aborted
    }
}
