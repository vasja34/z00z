//! Claim service contract for Scenario-1 migration.
//!
//! This module defines typed request/response payloads used by orchestration
//! layers to call claim-domain logic in `z00z_wallets::core`.

use serde::{Deserialize, Serialize};

/// Claim lifecycle checkpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimLifeStep {
    /// Stage was started and claim plan is not yet written.
    Started,
    /// Claim artifacts were produced.
    ArtifactsWritten,
    /// Wallets were updated with import decisions.
    WalletsUpdated,
    /// Genesis bins were consumed.
    BinsConsumed,
}

impl ClaimLifeStep {
    /// Returns true when transition is legal in canonical transition order.
    ///
    /// # Examples
    ///
    /// ```
    /// use z00z_wallets::claim::ClaimLifeStep;
    ///
    /// assert!(ClaimLifeStep::Started.can_move_to(ClaimLifeStep::ArtifactsWritten));
    /// assert!(!ClaimLifeStep::Started.can_move_to(ClaimLifeStep::WalletsUpdated));
    /// ```
    pub fn can_move_to(self, next: Self) -> bool {
        use ClaimLifeStep::{ArtifactsWritten, BinsConsumed, Started, WalletsUpdated};

        matches!(
            (self, next),
            (Started, Started)
                | (Started, ArtifactsWritten)
                | (ArtifactsWritten, ArtifactsWritten)
                | (ArtifactsWritten, WalletsUpdated)
                | (WalletsUpdated, WalletsUpdated)
                | (WalletsUpdated, BinsConsumed)
                | (BinsConsumed, BinsConsumed)
        )
    }
}

/// Import decision status without free-form strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimImportOutcome {
    /// Asset accepted and inserted.
    Accepted,
    /// Asset already exists and is treated as idempotent success.
    AlreadyExists,
    /// Asset rejected by validation/policy.
    Rejected,
}

/// One assignment produced by claim planning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimAssign {
    /// Target actor name.
    pub actor: String,
    /// Asset id in lower-hex format.
    pub asset_id_hex: String,
    /// Asset amount in plaintext model.
    pub amount: u64,
}

/// One typed import decision row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimDecision {
    /// Target actor name.
    pub actor: String,
    /// Asset id in lower-hex format.
    pub asset_id_hex: String,
    /// Typed import result.
    pub outcome: ClaimImportOutcome,
    /// Stable machine code, for example `IMPORT_ACCEPTED`.
    pub code: String,
}

/// Snapshot counters emitted by claim service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimCounters {
    /// Input asset row count.
    pub input_count: usize,
    /// Distributed asset row count.
    pub distributed_count: usize,
    /// Accepted decision count.
    pub accepted_count: usize,
    /// Already-exists decision count.
    pub exists_count: usize,
    /// Rejected decision count.
    pub rejected_count: usize,
}

/// Audit event row emitted by claim service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimAudit {
    /// Logical step id, for example `S3-3`.
    pub step: String,
    /// Event kind.
    pub event: String,
    /// Event status.
    pub status: String,
    /// Additional detail message.
    pub detail: String,
}

/// Claim service request payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimServiceRequest {
    /// Run id fingerprint.
    pub run_id: String,
    /// Distribution mode key.
    pub mode: String,
    /// RNG kind key.
    pub rng_kind: String,
    /// Whether bins must be consumed after successful claim.
    pub consume_bins: bool,
    /// Actor names ordered by claim planning policy.
    pub actors: Vec<String>,
    /// Asset ids planned for claim import.
    pub asset_ids: Vec<String>,
    /// Previous checkpoint if resume mode is active.
    pub prev_step: Option<ClaimLifeStep>,
}

/// Claim service response payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimServiceResponse {
    /// Planned assignment rows.
    pub assignments: Vec<ClaimAssign>,
    /// Typed import decisions.
    pub decisions: Vec<ClaimDecision>,
    /// Next canonical lifecycle step.
    pub next_state: ClaimLifeStep,
    /// Snapshot-ready counters.
    pub snapshot_counters: ClaimCounters,
    /// Audit events for orchestration logging.
    pub audit_events: Vec<ClaimAudit>,
}

/// Build deterministic dry-run plan from request payload.
///
/// This bootstrap planner is intentionally simple and stable:
/// - sorts actors and asset ids,
/// - assigns assets round-robin,
/// - emits typed `Accepted` decisions.
///
/// # Examples
///
/// ```
/// use z00z_wallets::claim::{ClaimLifeStep, service::{plan_dry_run, ClaimServiceRequest},
/// };
///
/// let req = ClaimServiceRequest {
///     run_id: "run-1".to_string(),
///     mode: "uniform_all".to_string(),
///     rng_kind: "mock:7".to_string(),
///     consume_bins: false,
///     actors: vec!["bob".to_string(), "alice".to_string()],
///     asset_ids: vec!["bb".to_string(), "aa".to_string()],
///     prev_step: Some(ClaimLifeStep::Started),
/// };
///
/// let plan = plan_dry_run(&req);
/// assert_eq!(plan.assignments.len(), 2);
/// assert_eq!(plan.snapshot_counters.accepted_count, 2);
/// assert_eq!(plan.next_state, ClaimLifeStep::ArtifactsWritten);
/// ```
pub fn plan_dry_run(req: &ClaimServiceRequest) -> ClaimServiceResponse {
    let mut actors = req.actors.clone();
    actors.sort();

    let mut asset_ids = req.asset_ids.clone();
    asset_ids.sort();

    let mut assigns = Vec::with_capacity(asset_ids.len());
    let mut decisions = Vec::with_capacity(asset_ids.len());

    for (idx, asset_id_hex) in asset_ids.iter().enumerate() {
        let actor = if actors.is_empty() {
            "unassigned".to_string()
        } else {
            actors[idx % actors.len()].clone()
        };

        assigns.push(ClaimAssign {
            actor: actor.clone(),
            asset_id_hex: asset_id_hex.clone(),
            amount: 0,
        });

        decisions.push(ClaimDecision {
            actor,
            asset_id_hex: asset_id_hex.clone(),
            outcome: ClaimImportOutcome::Accepted,
            code: "IMPORT_ACCEPTED".to_string(),
        });
    }

    let accepted_count = decisions.len();
    let counters = ClaimCounters {
        input_count: asset_ids.len(),
        distributed_count: assigns.len(),
        accepted_count,
        exists_count: 0,
        rejected_count: 0,
    };

    ClaimServiceResponse {
        assignments: assigns,
        decisions,
        next_state: ClaimLifeStep::ArtifactsWritten,
        snapshot_counters: counters,
        audit_events: vec![ClaimAudit {
            step: "S3-2".to_string(),
            event: "dry_run_plan".to_string(),
            status: "ok".to_string(),
            detail: "deterministic assignment/decision plan".to_string(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::{plan_dry_run, ClaimLifeStep, ClaimServiceRequest};

    #[test]
    fn test_claim_step_is_strict() {
        assert!(ClaimLifeStep::Started.can_move_to(ClaimLifeStep::ArtifactsWritten));
        assert!(ClaimLifeStep::ArtifactsWritten.can_move_to(ClaimLifeStep::WalletsUpdated));
        assert!(ClaimLifeStep::WalletsUpdated.can_move_to(ClaimLifeStep::BinsConsumed));
        assert!(!ClaimLifeStep::Started.can_move_to(ClaimLifeStep::WalletsUpdated));
        assert!(!ClaimLifeStep::ArtifactsWritten.can_move_to(ClaimLifeStep::BinsConsumed));
        assert!(!ClaimLifeStep::BinsConsumed.can_move_to(ClaimLifeStep::Started));
    }

    #[test]
    fn test_dry_run_is_deterministic() {
        let req = ClaimServiceRequest {
            run_id: "run-1".to_string(),
            mode: "uniform_all".to_string(),
            rng_kind: "mock:7".to_string(),
            consume_bins: true,
            actors: vec![
                "charlie".to_string(),
                "alice".to_string(),
                "bob".to_string(),
            ],
            asset_ids: vec!["cc".to_string(), "aa".to_string(), "bb".to_string()],
            prev_step: Some(ClaimLifeStep::Started),
        };

        let left = plan_dry_run(&req);
        let right = plan_dry_run(&req);
        assert_eq!(left, right);
        assert_eq!(left.snapshot_counters.accepted_count, 3);
    }
}
