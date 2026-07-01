#![forbid(unsafe_code)]

use z00z_aggregators::BatchId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSignal {
    pub provider_name: String,
    pub batch_id: BatchId,
    pub stage: ProviderStage,
    pub outcome: ProviderOutcome,
    pub blob_ref: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderStage {
    Publish,
    Resolve,
    Observe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderOutcome {
    Pending,
    Success,
    RetryPending,
    Missing,
    Failed,
}
