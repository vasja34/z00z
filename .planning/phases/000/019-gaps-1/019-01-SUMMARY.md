---
phase: 019-gaps-1
plan: 01
subsystem: storage
tags: [nullifier, simulator, storage, claim-pipeline, replay-protection]
requires:
  - phase: 018-A-B-C
    provides: storage-backed Scenario 1 apply and checkpoint path used by claim publication
provides:
  - storage-owned canonical nullifier transition for claim publication
  - fail-closed reserve, rollback, reload, and finalize semantics on the canonical publish path
  - simulator regression coverage for replay rejection and spent-state durability
affects: [019-02, 019-03, scenario-1, claim-publish]
tech-stack:
  added: []
  patterns: [storage-owned replay protection, atomic asset-plus-nullifier transition, explicit reservation lifecycle]
key-files:
  created: []
  modified:
    - crates/z00z_storage/src/assets/store.rs
    - crates/z00z_simulator/src/claim_pkg_consumer.rs
    - crates/z00z_simulator/src/claim_pkg_store.rs
    - crates/z00z_wallets/src/core/claim/nullifier_store.rs
    - crates/z00z_simulator/tests/test_stage3_nullifier_store.rs
    - crates/z00z_simulator/tests/test_claim_tx_pipeline.rs
key-decisions:
  - "Keep claim replay protection storage-owned by binding asset publication and nullifier advancement to one canonical transition."
  - "Treat direct publish_claims_store coverage as part of the canonical contract rather than simulator-only plumbing."
patterns-established:
  - "Claim publication advances asset state and replay-protection state together or fails closed."
  - "Reservation, rollback, reload, and finalize stay explicit while finalization depends on the storage-owned transition."
requirements-completed: [PH19-NULL]
duration: unknown
completed: 2026-03-25
---

# Phase 019 Plan 01: Nullifier Transition Summary

Storage-owned canonical claim replay protection with explicit reservation lifecycle and fail-closed publish semantics.

## Performance

- **Duration:** unknown
- **Started:** 2026-03-25T16:11:40Z
- **Completed:** 2026-03-25T17:35:55Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Bound claim replay protection to the storage-owned publish transition so asset state and nullifier state advance together.
- Revalidated reserve, rollback, reload, and finalize behavior on the canonical simulator path rather than on wallet-local state.
- Confirmed debug and release regression coverage for replay rejection, reservation preconditions, and spent-state persistence.

## Task Commits

Current branch history does not expose an isolated 019-01 task commit on top of `origin/z00z-simul`; this closure pass validated the existing tree state and completed the missing phase artifacts.

1. **Task 1: Define the canonical asset-plus-nullifier storage transition** - validated current branch state (no isolated local task commit)
2. **Task 2: Bind claim publish reserve, rollback, and finalize to the storage-owned path** - validated current branch state (no isolated local task commit)

**Plan metadata:** pending docs commit for summary and state reconciliation

## Files Created/Modified

- `crates/z00z_storage/src/assets/store.rs` - Hosts the canonical asset-plus-nullifier mutation seam.
- `crates/z00z_simulator/src/claim_pkg_consumer.rs` - Binds simulator claim publication to the storage-owned transition and fail-closed lifecycle.
- `crates/z00z_simulator/src/claim_pkg_store.rs` - Preserves explicit reservation, reload, and finalize orchestration.
- `crates/z00z_wallets/src/core/claim/nullifier_store.rs` - Keeps nullifier identity and lease semantics aligned with the canonical store contract.
- `crates/z00z_simulator/tests/test_stage3_nullifier_store.rs` - Covers reserve, rollback, reload, and finalize behavior on the canonical path.
- `crates/z00z_simulator/tests/test_claim_tx_pipeline.rs` - Covers replay rejection, publish preconditions, and spent-state persistence.

## Decisions Made

- Reuse the existing storage-owned commit seam instead of inventing a second nullifier persistence engine.
- Keep `NullifierClaim` as identity data and `NullifierLease` as the transaction-scoped reservation handle while making finalization depend on the canonical storage transition.

## Deviations from Plan

None - the current branch state already satisfied the plan contract, and this pass closed the missing artifacts after fresh validation.

## Issues Encountered

- `.planning` state lagged behind the validated code state and did not yet contain `019-01-SUMMARY.md`; the gap was resolved by rerunning targeted debug and release simulator gates and documenting the current tree state.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Storage-owned replay protection is validated in debug and release profiles and can serve as the baseline for later wallet-facing gap closures.
- Phase 019 can be closed once backup-convergence artifacts and state reconciliation are committed alongside this summary.

## Self-Check: PASSED

- Found summary target: `.planning/phases/019-gaps-1/019-01-SUMMARY.md`
- Found debug validation: `test_stage3_nullifier_store`, `test_claim_tx_pipeline`
- Found release validation: `test_stage3_nullifier_store`, `test_claim_tx_pipeline`

---
_Phase: 019-gaps-1_
_Completed: 2026-03-25_
