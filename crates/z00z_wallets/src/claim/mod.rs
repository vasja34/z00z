//! Claim receipt data model and signing helpers.

/// Claim receipt types and crypto helpers.
pub mod claim_receipt;
/// Claim conservation invariants.
pub mod conservation;
/// Claim distribution policy engine.
pub mod distribution;
/// Claim import status mapping model.
pub mod import_model;
/// Claim nullifier derivation and state key types.
pub mod nullifier;
/// Claim nullifier replay-protection store.
pub mod nullifier_store;
/// Claim registry and replay-protection gate.
pub mod registry;
/// Claim service request/response contract and lifecycle enums.
pub mod service;
/// Claim resume state machine contract.
pub mod state_machine;

pub use claim_receipt::{
    claim_scope_hash, sign_claim_receipt, verify_claim_receipt, ClaimReceipt, CLAIM_CTX,
};
pub use conservation::{verify_claim_conservation, ConservationError};
pub use distribution::{assign_class_split, assign_coin_sets, assign_uniform_all, count_assigned};
pub use import_model::{map_import_err, map_import_ok, map_replay_code, ImportEval};
pub use nullifier::{derive_nullifier, NullifierEntry, NullifierKey, NullifierStatus};
pub use nullifier_store::{
    bind_paths, claim_match, create_nullifier_lease, global_nullifier_store, InMemNullStore,
    NullAuditRow, NullFinalizeErr, NullReserveErr, NullifierClaim, NullifierConflict,
    NullifierLease, NullifierStateStore,
};
pub use nullifier_store::{clear_bind, clear_rows, get_entry, read_audit, read_entry};
pub use registry::{
    global_claim_registry, mark_final, ClaimConflict, ClaimFinalizeErr, ClaimReservation,
    ClaimReserveErr, ClaimRow, GlobalClaimRegistry, InMemClaimRegistry,
};
pub use service::{
    ClaimAssign, ClaimAudit, ClaimCounters, ClaimDecision, ClaimImportOutcome, ClaimLifeStep,
    ClaimServiceRequest, ClaimServiceResponse,
};
pub use state_machine::{
    add_row, has_row, read_state, rehydrate_rows, verify_resume_wire, write_state, ClaimStateFile,
    ClaimStateRow,
};
