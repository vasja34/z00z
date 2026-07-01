---
phase: 020-refactor-scenario-1
plan: 03
subsystem: simulator
tags: [scenario_1, transfer_receive, transfer_claim, bundle_build, bundle_publish, release-verification, continuity]
requires:
  - phase: 020-02
    provides: explicit tx_plan and tx_prepare stage surface
provides:
  - explicit Stage 5 split between receive and claim with a stable handoff artifact
  - explicit Stage 6 split between build and publish while keeping the Stage 6 bridge as the only downstream handoff
  - release-verified continuity across Stage 5 through Stage 12 Scenario 1 gates
affects: [020-04, scenario_1]
tech-stack:
  added: []
  patterns: [thin stage facade over utility module, staged handoff artifact, release-only simulator verification]
key-files:
  created:
    - .planning/phases/020-refactor-scenario-1/020-03-SUMMARY.md
    - crates/z00z_simulator/src/scenario_1/stage_5_utils/mod.rs
    - crates/z00z_simulator/src/scenario_1/stage_6_utils/mod.rs
  modified:
    - crates/z00z_simulator/src/scenario_1/mod.rs
    - crates/z00z_simulator/src/scenario_1/stage_5.rs
    - crates/z00z_simulator/src/scenario_1/stage_6.rs
    - crates/z00z_simulator/tests/test_stage5_receive_bridge.rs
    - .planning/phases/020-refactor-scenario-1/020-VALIDATION.md
    - .planning/phases/020-refactor-scenario-1/020-TEST-SPEC.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
key-decisions:
  - Keep the public split entrypoints on `stage_5::run_transfer_receive`, `stage_5::run_transfer_claim`, `stage_6::run_bundle_build`, and `stage_6::run_bundle_publish`.
  - Preserve the Stage 5 artifact contract at `stage = 7` even when `transfer_claim` runs as final stage 8.
  - Keep Stage 6 reuse explicit through `checkpoint_bridge_s6.json` and avoid inventing broader shared path or logging modules without proven duplication pressure.
patterns-established:
  - Stage-local utility modules can own split execution flow without widening the public scenario surface.
  - Release-only validation remains the closure gate for Scenario 1 refactor slices in this workspace.
requirements-completed: [SCN1-03, SCN1-05]
duration: multi-session
completed: 2026-03-26
---

# Phase 020 Plan 03: Transfer And Bundle Split Summary

📌 Scenario 1 now exposes explicit `transfer_receive`, `transfer_claim`, `bundle_build`, and `bundle_publish` execution slices, with downstream continuity preserved in release mode.

## Performance

- **Duration:** multi-session
- **Completed:** 2026-03-26
- **Tasks:** 2
- **Files modified:** 8+

## Accomplishments

- 📌 Added `stage_5_utils/mod.rs` and moved the Stage 5 orchestration split behind a receive handoff so stage 7 owns report-only receive work and stage 8 owns explicit claim plus final Stage 5 artifacts.
- 📌 Added `stage_6_utils/mod.rs` and moved the Stage 6 orchestration split behind explicit build and publish flows so stage 9 owns fragment or bridge or exec-input creation and stage 10 owns report publication.
- 📌 Preserved the current Stage 6 downstream contract by keeping `checkpoint_bridge_s6.json` as the only handoff consumed by later stages.
- 📌 Kept the Stage 5 snapshot and tx artifact contract stable at `stage = 7`, which matches the existing bridge tests and artifact readers.
- 📌 Revalidated the split in release mode across structural, transfer, bundle, apply, finalize, and unified Scenario 1 gates.

## Task Commits

📌 No task commits were created in this execution slice.

📌 Reason: the workspace already contains unrelated in-flight changes, and no explicit commit request was given during this run.

## Files Created/Modified

- `crates/z00z_simulator/src/scenario_1/mod.rs` - registers the new Stage 5 and Stage 6 utility modules.
- `crates/z00z_simulator/src/scenario_1/stage_5.rs` - keeps the public Stage 5 surface thin and delegates full or receive or claim flows into the utility module.
- `crates/z00z_simulator/src/scenario_1/stage_5_utils/mod.rs` - owns receive or claim orchestration plus the Stage 5 receive handoff artifact.
- `crates/z00z_simulator/src/scenario_1/stage_6.rs` - keeps the public Stage 6 surface thin and delegates full or build or publish flows into the utility module.
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/mod.rs` - owns Stage 6 build or publish orchestration while preserving the bridge contract.
- `crates/z00z_simulator/tests/test_stage5_receive_bridge.rs` - updates the bad-index guard to assert failure on final stage 7 `transfer_receive`.
- `.planning/phases/020-refactor-scenario-1/020-VALIDATION.md` - records green release evidence for plan 03.
- `.planning/phases/020-refactor-scenario-1/020-TEST-SPEC.md` - records the actual split implementation shape and release closure set.
- `.planning/ROADMAP.md` - marks plan 03 as complete and points Phase 020 at plan 04.
- `.planning/STATE.md` - advances the active plan pointer to `020-04`.

## Decisions Made

- 📌 Kept the split on the preferred Stage 5 and Stage 6 public entrypoints instead of inventing a second public facade.
- 📌 Used a small Stage 5 handoff artifact to keep `transfer_receive` and `transfer_claim` separate while preserving the existing Stage 5 output contract.
- 📌 Limited shared cleanup to stage-local utility modules because the broader `scenario_paths.rs` and `scenario_logging.rs` proposal was not justified by the real duplication left after the split.
- 📌 Kept release-only verification because the user explicitly requires simulator validation in `--release` mode.

## Deviations from Plan

### Proposed helper fan-out stayed compact

- **Issue:** `020-03-PLAN.md` proposed a wider helper tree under `stage_5_utils/` and `stage_6_utils/` than the current code actually needed.
- **Adjustment:** Closed the split through `stage_5_utils/mod.rs` and `stage_6_utils/mod.rs` without fabricating `context.rs`, `paths.rs`, `receive.rs`, `rpc_flow.rs`, `claim_flow.rs`, `artifacts.rs`, `logging.rs`, `fragments.rs`, or `report.rs`.
- **Impact:** The split is real and release-verified, but the helper layout stays aligned to the actual implementation pressure instead of the aspirational file list.

### Broader shared scenario helpers were not materialized

- **Issue:** The plan listed `scenario_paths.rs` and `scenario_logging.rs` as possible cleanup targets.
- **Adjustment:** Did not create those files because the remaining duplication after the Stage 5/6 split did not justify broad shared modules.
- **Impact:** Public behavior and continuity stay stable, and the refactor avoids speculative helper surfaces.

**Total deviations:** 2

## Issues Encountered

- 📌 The Stage 5 bad-index test still expected failure on the pre-split stage id `5`; it was updated to assert failure on final stage `7` `transfer_receive`.
- 📌 Release runs still emit the existing non-blocking `dead_code` warning for `Stage4ResolvedPaths::logger_path` in `stage_4.rs`; it is unrelated to this plan slice.

## Verification Evidence

- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage5_source_shape`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage5_receive_bridge`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_storage_bridge`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_final_gate`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage7_jmt_wallet_scan`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage8_proof_path`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_unified_gate`

## User Setup Required

📌 None.

## Next Phase Readiness

- 📌 Phase 020 can advance to `020-04-PLAN.md` for final YAML or gate closure.
- 📌 The next slice should decide whether any additional release guard beyond the current split suite is needed before closing the whole phase.

## Self-Check

📌 PASSED - summary references files that exist in the workspace and commands that were executed during this run.

---
*Phase: 020-refactor-scenario-1*
*Completed: 2026-03-26*
