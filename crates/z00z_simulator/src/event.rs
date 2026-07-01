//! Simulator RPC-style structured events.
//!
//! Each event is emitted as a JSON file under `{outputs_dir}/events/`.
//! Event files use the format `<event_name>.event.json`.

use std::path::Path;

use serde::Serialize;
use z00z_utils::io::{create_dir_all, save_json};

/// Per-actor claim summary included in [`ClaimGenesisEvent`].
#[derive(Debug, Serialize)]
pub struct ActorClaimSummary {
    /// Actor name.
    pub name: String,
    /// Total assets assigned to this actor.
    pub assets_count: usize,
    /// Sum of all asset amounts.
    pub total_amount: u64,
    /// Count of distinct asset definition IDs.
    pub unique_terminal_ids: usize,
}

/// Event emitted when Stage 3 genesis claim completes successfully.
///
/// Written to `{events_dir}/claim_genesis.event.json`.
#[derive(Debug, Serialize)]
pub struct ClaimGenesisEvent {
    /// Event discriminator.
    pub event: &'static str,
    /// Scenario identifier from config.
    pub scenario_id: u32,
    /// Stage number (3).
    pub stage: u32,
    /// Claim path mode.
    pub claim_mode: String,
    /// Event payload compatibility version.
    pub compatibility_version: u32,
    /// Distribution mode: `class_split` | `coin_sets` | `uniform_all`.
    pub mode: String,
    /// RNG kind: `"mock:<seed>"` or `"system"`.
    pub rng_kind: String,
    /// Total input assets loaded from `.bin` files.
    pub input_assets: usize,
    /// Total assets distributed across all actors.
    pub distributed: usize,
    /// Per-actor summaries.
    pub actor_claims: Vec<ActorClaimSummary>,
    /// Unix timestamp (seconds) when the event was emitted.
    pub timestamp_unix: u64,
}

impl ClaimGenesisEvent {
    /// Emit event JSON to `{events_dir}/claim_genesis.event.json`.
    pub fn emit(&self, events_dir: &Path) -> Result<(), String> {
        create_dir_all(events_dir).map_err(|e| e.to_string())?;
        let path = events_dir.join("claim_genesis.event.json");
        save_json(&path, self).map_err(|e| e.to_string())?;
        Ok(())
    }
}
