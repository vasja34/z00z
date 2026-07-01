---
phase: 020-refactor-scenario-1
plan: 01
subsystem: simulator
tags: [scenario_1, stage_split, claim_prepare, claim_publish, release-verification]
requires:
  - phase: 019
    provides: storage-owned claim publication and nullifier transition contracts
  - phase: 019.1-rename
    provides: renamed simulator paths and synchronized local documentation
provides:
  - explicit Scenario 1 stages 3 and 4 for claim_prepare and claim_publish
  - extracted Stage 3 helper seams for claim package build/write and audit logging
  - release-verified Wave 0 guard for the 12-stage Scenario 1 map
affects: [020-02, 020-03, 020-04, scenario_1]
tech-stack:
  added: []
  patterns: [stage facade over helper seams, release-only simulator verification]
key-files:
  created:
    - crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs
    - crates/z00z_simulator/src/scenario_1/stage_3_utils/audit.rs
    - .planning/phases/020-refactor-scenario-1/020-01-SUMMARY.md
  modified:
    - crates/z00z_simulator/src/scenario_1/runner.rs
    - crates/z00z_simulator/src/scenario_1/mod.rs
    - crates/z00z_simulator/src/scenario_1/stage_3.rs
    - crates/z00z_simulator/src/scenario_1/stage_3_utils/mod.rs
    - crates/z00z_simulator/src/scenario_1/scenario_config.yaml
    - crates/z00z_simulator/src/scenario_1/scenario_design.yaml
    - crates/z00z_simulator/tests/test_scenario1_stage_surface.rs
    - .planning/phases/020-refactor-scenario-1/020-VALIDATION.md
    - .planning/phases/020-refactor-scenario-1/020-TEST-SPEC.md
key-decisions:
  - Keep the Stage 3 public helper and snapshot API stable while moving claim package and audit logic behind stage_3_utils.
  - Honor the explicit user override to run simulator verification in --release mode only for this execution slice.
patterns-established:
  - Stage 3 remains a facade module while non-trivial claim package and audit behavior lives in dedicated helper files.
  - Phase validation artifacts must be updated in the same wave as source-shape guards and helper seam extraction.
requirements-completed: [SCN1-04]
duration: multi-session
completed: 2026-03-26
---

# Phase 020 Plan 01: Claim Lane Split Summary

**Scenario 1 claim lane split into `claim_prepare` and `claim_publish` with extracted Stage 3 claim-package and audit seams plus release-verified 12-stage guard coverage**

## Performance

- **Duration:** multi-session
- **Completed:** 2026-03-26
- **Tasks:** 2
- **Files modified:** 11+

## Accomplishments

- Expanded Scenario 1 to an explicit 12-stage map where stages 3 and 4 are `claim_prepare` and `claim_publish`.
- Extracted real Stage 3 helper seams into `stage_3_utils/claim_pkg.rs` and `stage_3_utils/audit.rs` while preserving the Stage 3 public helper surface used by tests.
- Verified the 020-01 claim-lane slice in release mode with the stage-surface guard and the full claim test family, then synchronized validation artifacts to match the codebase.

## Task Commits

No task commits were created in this execution slice.

Reason: the workspace already contained unrelated in-flight changes, and no explicit commit request was given during this run.

## Files Created/Modified

- `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs` - extracted claim package build and bundle write seam from `stage_3.rs`
- `crates/z00z_simulator/src/scenario_1/stage_3_utils/audit.rs` - extracted audit row and audit log write seam from `stage_3.rs`
- `crates/z00z_simulator/src/scenario_1/stage_3_utils/mod.rs` - re-exported new helper seams behind the Stage 3 facade
- `crates/z00z_simulator/src/scenario_1/stage_3.rs` - reduced direct responsibility to facade/orchestration plus stable public exports
- `crates/z00z_simulator/src/scenario_1/runner.rs` - binds stages 3 and 4 to explicit claim entrypoints
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml` - records the explicit 12-stage Scenario 1 map
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` - guards the 12-stage surface in release mode
- `.planning/phases/020-refactor-scenario-1/020-VALIDATION.md` - marks Wave 0 artifacts as present and 020-01 release checks as green
- `.planning/phases/020-refactor-scenario-1/020-TEST-SPEC.md` - records that claim helper seams now live behind `stage_3_utils`

## Decisions Made

- Kept `stage_3.rs` as the public compatibility facade so downstream tests continue using the same Stage 3 API while the internal seams move under `stage_3_utils`.
- Materialized the plan's proposed helper names `claim_pkg.rs` and `audit.rs` instead of documenting a deviation to a different helper layout.
- Used release-only verification for this plan slice because the user explicitly required all simulator launches to run in `--release` mode.

## Deviations from Plan

### Execution Deviations

### Release-only verification override

- **Issue:** `020-01-PLAN.md` lists both debug and release verification commands.
- **Adjustment:** Ran the release-profile commands only, all with `--features test-fast --features wallet_debug_dump`, per explicit user instruction.
- **Impact:** No coverage loss for the requested execution mode; `020-01` evidence is grounded in the stricter release path.

**Total deviations:** 1
**Impact on plan:** verification mode narrowed to the user-mandated release path; implementation scope unchanged.

## Issues Encountered

- The first helper extraction left visibility and helper-scope compile errors; these were resolved by promoting the intended re-exports through `stage_3_utils/mod.rs`, trimming obsolete imports in `stage_3.rs`, and restoring local conservation helpers needed by `verify_claim_conservation`.
- `020-VALIDATION.md` and `020-TEST-SPEC.md` still described Wave 0 as missing even though the guard files already existed; both documents were updated in the same wave.

## Verification Evidence

- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_acceptance -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_emit -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_tx_pipeline -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_snapshot -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_persist -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_integration -- --nocapture`

## User Setup Required

None - no external setup required for this plan slice.

## Next Phase Readiness

- The claim lane is split and release-verified, so phase 020 can move to the Stage 4 source-shape and continuity slice in `020-02-PLAN.md`.
- `STATE.md` is already marked as phase 020 in progress; remaining bookkeeping can advance after the next plan summary or an explicit commit sweep.

## Self-Check

PASSED - summary reflects files that exist in the workspace and verification commands that were executed in release mode during this run.

---
*Phase: 020-refactor-scenario-1*
*Completed: 2026-03-26*
