---
phase: 033-crypto-audit-scenario-2
plan: 6
subsystem: testing
tags: [checkpoint, redb, simulator, replay, rehydrate]
requires:
  - phase: 033-05
    provides: stage-surface truth guards and checkpoint continuity wording baselines
provides:
  - explicit compatibility-only draft/final checkpoint boundary wording
  - stronger persisted RedB binding documentation for checkpoint reload
  - narrower replay and stale-artifact wording aligned to current spent-row evidence
affects: [phase-033, checkpoint-storage, simulator-stage-surface]
tech-stack:
  added: []
  patterns: [source-shape regression guards, persisted-vs-raw boundary wording, replay-scope narrowing]
key-files:
  created: []
  modified:
    - crates/z00z_simulator/src/scenario_1/stage_12.rs
    - crates/z00z_simulator/src/scenario_1/storage_view.rs
    - crates/z00z_simulator/tests/test_stage6_checkpoint_final_gate.rs
    - crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs
    - crates/z00z_storage/src/assets/store_internal/store_rows.rs
    - crates/z00z_storage/src/error.rs
    - crates/z00z_storage/tests/test_redb_rehydrate.rs
key-decisions:
  - "Keep legacy-final artifacts explicitly compatibility-only instead of treating their existence as a weak-finality exception."
  - "Describe raw checkpoint identity as weaker than the persisted RedB rehydrate path rather than overclaiming proof-payload injectivity."
  - "Narrow spent-row replay wording until a dedicated rehydrate-then-replay theorem exists."
patterns-established:
  - "Checkpoint wording guards should assert semantic fragments, not brittle wrapped prose."
  - "Broad release-gate failures after wording changes must be traced back to source-shape guard suites before plan closeout."
requirements-completed: [PH32-CLAIM-TRUST, PH32-CHECKPOINT, PH32-HONEST]
duration: 70min
completed: 2026-04-07
---

# Phase 033 Plan 06: Checkpoint Truth Boundary Summary

Checkpoint draft/final compatibility boundaries, persisted RedB binding semantics, and replay-scope wording now match the actual simulator and storage evidence set.

## Performance

- **Duration:** 70 min
- **Started:** 2026-04-07T06:21:09Z
- **Completed:** 2026-04-07T07:31:23Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- Pinned Task 16 with explicit compatibility-only wording so draft and final checkpoint classes remain unambiguous across simulator finalize/load surfaces and storage error taxonomy.
- Pinned Task 17 by documenting the stronger persisted RedB binding over the weaker raw checkpoint artifact surface.
- Pinned Task 18 by narrowing replay and stale-artifact wording for spent-row behavior and extending rehydrate source-shape regression coverage.

## Task Commits

Each task was committed atomically:

1. **Task 16: Draft Versus Final Truth** - `38cea809` (fix)
2. **Task 17: Injective Persistence Contract** - `c14fd04b` (fix)
3. **Task 18: Replay And Stale-Artifact Resistance** - `eb32b316` (fix)

**Plan metadata:** pending

## Files Created/Modified

- `crates/z00z_simulator/src/scenario_1/stage_12.rs` - keeps stage 8 wording package-coupled and compatibility-only for legacy-final artifacts.
- `crates/z00z_simulator/src/scenario_1/storage_view.rs` - documents the compatibility-only final export lane and narrows rehydrate/replay claims.
- `crates/z00z_simulator/tests/test_stage6_checkpoint_final_gate.rs` - adds Task 16 source-shape guards for the draft/final boundary.
- `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs` - states the stronger persisted binding contract during RedB reload.
- `crates/z00z_storage/src/assets/store_internal/store_rows.rs` - narrows spent-row replay wording to the current proof-bearing bundle evidence.
- `crates/z00z_storage/src/error.rs` - keeps checkpoint compatibility mismatch classification explicit.
- `crates/z00z_storage/tests/test_redb_rehydrate.rs` - adds Task 17 and Task 18 source-shape regression checks over persisted binding and replay/stale wording.

## Decisions Made

- Kept the plan on the documentation-and-regression path for Task 18 instead of inventing a broader spent-row theorem that the codebase does not yet prove.
- Accepted a narrow Task 17 source commit because the shared rehydrate test hunk overlapped Task 18 coverage and was cleaner to land with the replay-scope commit.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Repaired a stage 12 wording regression exposed by the broad release gate**

- **Found during:** Post-task broad validation after Tasks 16-18 were green locally
- **Issue:** `test_checkpoint_continuity_wording_stays_package_coupled` failed because `stage_12.rs` no longer said `accepted package-coupled path`, which made stage 12 look closer to a standalone checkpoint authority surface.
- **Fix:** Updated the stage 8 comment to preserve the exact package-coupled continuity wording while keeping the new compatibility-only boundary language.
- **Files modified:** `crates/z00z_simulator/src/scenario_1/stage_12.rs`
- **Verification:** `cargo test --release -p z00z_simulator --test test_scenario1_stage_surface --features test-fast --features wallet_debug_dump` and the full workspace release gate passed.
- **Committed in:** `38cea809`

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** The auto-fix was required for correctness of the wording guard surface. No scope creep.

## Issues Encountered

- The full release gate first failed on a source-shape wording regression in `test_scenario1_stage_surface`; the failure was in-scope and fixed immediately.
- Pre-commit formatting checks required rustfmt-style wrapping for one `include_str!` constant in `test_redb_rehydrate.rs` before the task commits could land cleanly.
- Splitting the mixed Task 17 and Task 18 `test_redb_rehydrate.rs` hunk by index automation was unreliable, so the source-only Task 17 commit was kept narrow and the shared rehydrate test additions landed with Task 18.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 06 is ready for downstream checkpoint caution/high-severity work because the raw-vs-persisted and draft-vs-final seams are now explicit and regression-tested.
- No in-scope blockers remain from this plan.

## Self-Check: PASSED

- Verified summary and all referenced plan files exist in the workspace.
- Verified task commits `38cea809`, `c14fd04b`, and `eb32b316` exist in git history.

---
*Phase: 033-crypto-audit-scenario-2*
*Completed: 2026-04-07*
