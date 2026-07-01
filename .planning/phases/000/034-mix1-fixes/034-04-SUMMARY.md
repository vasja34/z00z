---
phase: 034-mix1-fixes
plan: 04
subsystem: checkpoint-backend-contract
tags: [checkpoint, storage, simulator, wallet, fail-closed, backend-owned]
requires:
  - phase: 034-03
    provides: narrowed spend/nullifier baseline and truthful sender-authority sequencing
provides:
  - Backend-owned checkpoint proof acceptance contract
  - Fail-closed checkpoint finalization with no compat-only authority path
  - Reload and Scenario 1 promotion bound to the same backend payload truth
affects: [PH34-CHECKPOINT-BACKEND]
tech-stack:
  added: []
  patterns: [backend-owned checkpoint proof contract, fail-closed compat rejection, storage-simulator acceptance alignment]
key-files:
  created:
    - /home/vadim/Projects/z00z/.planning/phases/034-mix1-fixes/034-04-SUMMARY.md
  modified:
    - /home/vadim/Projects/z00z/crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs
    - /home/vadim/Projects/z00z/crates/z00z_storage/src/checkpoint/artifact_final.rs
    - /home/vadim/Projects/z00z/crates/z00z_storage/src/checkpoint/artifact_stmt.rs
    - /home/vadim/Projects/z00z/crates/z00z_storage/src/assets/store_internal/redb_backend.rs
    - /home/vadim/Projects/z00z/crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_12.rs
    - /home/vadim/Projects/z00z/crates/z00z_storage/tests/test_checkpoint_finalization.rs
    - /home/vadim/Projects/z00z/crates/z00z_storage/tests/test_checkpoint_store_api.rs
    - /home/vadim/Projects/z00z/crates/z00z_storage/tests/test_redb_rehydrate.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_checkpoint_acceptance.rs
key-decisions:
  - "Make backend payload equality the live checkpoint authority surface instead of compat-only proof bytes or externally supplied verifier trust."
  - "Reject compat-only finalized checkpoint proofs with `CheckpointError::ArtifactCompatMix` instead of silently treating them as authoritative."
  - "Keep legacy decode support only as backward-format handling, not as the acceptance story for finalize, reload, or simulator promotion."
patterns-established:
  - "Finalize, persisted reload, and Scenario 1 promotion now all consume the same backend-bound checkpoint acceptance truth."
requirements-completed: [PH34-CHECKPOINT-BACKEND]
completed: 2026-04-10
reviewed: 2026-04-10T00:00:00Z
---

# Phase 034 Plan 04 Summary

## Outcome

Plan 04 is complete. Checkpoint proof acceptance is now defined by one
backend-owned contract, live finalization no longer treats compat-only proof
objects as authoritative, and storage reload plus Scenario 1 promotion both
consume the same backend payload truth.

## Accomplishments

- Bound live checkpoint acceptance to backend-owned payload semantics instead of
  compatibility-only proof bytes.
- Tightened checkpoint finalization so compat-only proof objects are rejected
  with `CheckpointError::ArtifactCompatMix` instead of silently falling back to
  a legacy authority path.
- Aligned finalized artifact validation and RedB reload validation around the
  same backend payload, exec identity, and state-root invariants.
- Kept legacy decode handling explicitly non-authoritative so backward-format
  parsing remains possible without becoming the closure story.
- Revalidated the simulator checkpoint acceptance seam against the backend-owned
  path instead of the old compat-wording path.

## Verification

- `cargo test -p z00z_storage --release test_checkpoint_finalization -- --nocapture`
  passed.
- `cargo test -p z00z_storage --release test_checkpoint_store_api -- --nocapture`
  passed.
- `cargo test -p z00z_storage --release test_redb_rehydrate -- --nocapture`
  passed.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump test_checkpoint_acceptance -- --nocapture`
  passed.
- Package-local simulator doctest validation was rechecked green during closeout.
- The broader workspace doc-wave suspicion was rechecked and no live blocker was
  confirmed during this closeout pass.

## Issues Encountered

- The core live gap was not in persisted reload validation; it was in
  finalization, which still accepted compat-only proof objects by falling back
  to a legacy authority path.
- Broader doctest status initially looked ambiguous enough to block a truthful
  closeout, so it had to be reclassified before the plan could be closed on
  evidence instead of suspicion.

## Next Phase Readiness

- `034-05` can now start from a backend-authoritative checkpoint contract rather
  than a mixed compat/backend story.
- Later validation waves can treat checkpoint finalize, reload, and simulator
  promotion as one aligned acceptance seam.

## Known Stubs

- This plan does not claim a new generic standalone checkpoint proof system.
  The closure is limited to the backend-owned acceptance contract already
  present in the live storage and simulator seams.

## Threat Flags

None for the Plan 04 checkpoint-backend scope after the compat-only
finalization fallback was removed.

## Self-Check

PASSED.

---
*Phase: 034-mix1-fixes*
*Completed: 2026-04-10*
