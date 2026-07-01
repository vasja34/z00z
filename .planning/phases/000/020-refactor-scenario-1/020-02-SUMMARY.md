---
phase: 020-refactor-scenario-1
plan: 02
subsystem: simulator
tags: [scenario_1, tx_plan, tx_prepare, stage_4, release-verification, continuity]
requires:
  - phase: 020-01
    provides: explicit claim_prepare and claim_publish stage surface
  - phase: 019.1-rename
    provides: simulator rename cleanup and summary-backed phase bookkeeping
provides:
  - explicit Scenario 1 stages 5 and 6 for tx_plan and tx_prepare
  - release-verified Stage 4 continuity across path remap, tamper, and pipeline guards
  - summary-backed closure for the tx-lane split without inventing non-existent helper files
affects: [020-03, 020-04, scenario_1]
tech-stack:
  added: []
  patterns: [stage facade over existing helper tree, release-only simulator verification]
key-files:
  created:
    - .planning/phases/020-refactor-scenario-1/020-02-SUMMARY.md
  modified:
    - crates/z00z_simulator/src/scenario_1/stage_4.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs
    - crates/z00z_simulator/src/scenario_1/scenario_design.yaml
    - crates/z00z_simulator/src/scenario_1/scenario_config.yaml
    - crates/z00z_simulator/tests/test_stage4_source_shape.rs
    - crates/z00z_simulator/tests/test_stage4_cfg_paths.rs
    - crates/z00z_simulator/tests/test_stage4_chain_path.rs
    - crates/z00z_simulator/tests/test_stage4_tamper.rs
    - crates/z00z_simulator/tests/test_pipeline_genesis_tx.rs
    - .planning/phases/020-refactor-scenario-1/020-VALIDATION.md
    - .planning/phases/020-refactor-scenario-1/020-TEST-SPEC.md
key-decisions:
  - Keep the tx-lane entrypoints on the preferred `stage_4::run_tx_plan` and `stage_4::run_tx_prepare` surface.
  - Treat the helper filenames proposed in `020-02-PLAN.md` as aspirational only and keep the implementation aligned to the existing `stage_4_utils` tree.
  - Honor the explicit user override to validate the simulator in release mode only.
patterns-established:
  - Stage 4 remains the public tx-lane facade while helper ownership stays under the existing `stage_4_utils` lineage.
  - Plan closure for phase 020 must be summary-backed and tied to release evidence, not inferred from source shape alone.
requirements-completed: [SCN1-04]
duration: multi-session
completed: 2026-03-26
---

# Phase 020 Plan 02: Tx Lane Split Summary

**Scenario 1 tx lane is now explicit as `tx_plan` and `tx_prepare`, with release-verified continuity across path remap, tamper guards, and claim-to-tx pipeline evidence**

## Performance

- **Duration:** multi-session
- **Completed:** 2026-03-26
- **Tasks:** 2
- **Files modified:** 10+

## Accomplishments

- Confirmed the explicit 12-stage Scenario 1 map remains stable with stages 5 and 6 bound to `tx_plan` and `tx_prepare`.
- Closed the tx-lane split on the real code structure: `stage_4.rs` stays the public facade and delegates into the existing `stage_4_utils` tree instead of forcing the proposed helper filenames from the plan.
- Revalidated the tx-lane continuity surface in release mode across source shape, tamper handling, path remap behavior, chain-root continuity, and the claim-to-tx pipeline bridge.

## Task Commits

No task commits were created in this execution slice.

Reason: the workspace already contained unrelated in-flight changes, and no explicit commit request was given during this run.

## Files Created/Modified

- `crates/z00z_simulator/src/scenario_1/stage_4.rs` - exposes the explicit `run_tx_plan` and `run_tx_prepare` entrypoints while keeping the Stage 4 public surface stable.
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs` - keeps the split logic behind the existing helper tree.
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml` - binds stages 5 and 6 to `tx_plan` and `tx_prepare`.
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml` - stays aligned with the split tx-lane config sources.
- `crates/z00z_simulator/tests/test_stage4_source_shape.rs` - guards the split tx-lane stage surface.
- `crates/z00z_simulator/tests/test_stage4_cfg_paths.rs` - guards Stage 4 path remap stability.
- `crates/z00z_simulator/tests/test_stage4_chain_path.rs` - guards claim-root and prep-root continuity across the split lane.
- `crates/z00z_simulator/tests/test_stage4_tamper.rs` - guards Stage 4 tamper paths under the split lane.
- `crates/z00z_simulator/tests/test_pipeline_genesis_tx.rs` - guards cross-stage continuity from claim publication into tx preparation.
- `.planning/phases/020-refactor-scenario-1/020-VALIDATION.md` - records green release evidence for the tx-lane split.
- `.planning/phases/020-refactor-scenario-1/020-TEST-SPEC.md` - records the current release gate set for the split Stage 4 lane.

## Decisions Made

- Kept the tx-lane split on the preferred public entrypoints `stage_4::run_tx_plan` and `stage_4::run_tx_prepare`.
- Closed the plan against the real helper layout under `stage_4_utils/` instead of fabricating `contracts.rs`, `selection.rs`, `outputs.rs`, `prep.rs`, `verify.rs`, or `tamper.rs` just to mirror the proposed file list.
- Used release-only verification for this plan slice because the user explicitly required all simulator validation to run in `--release` mode.

## Deviations from Plan

### Execution Deviations

### Proposed helper filenames stayed proposed

- **Issue:** `020-02-PLAN.md` lists several proposed Stage 4 helper files that do not exist in the current tree.
- **Adjustment:** Closed the plan on the actual `stage_4.rs` plus `stage_4_utils/` layout and recorded that deviation in the summary and validation artifacts.
- **Impact:** No behavior loss; the split is real and release-verified, but the implementation remains aligned to the existing module tree.

### Release-only verification override

- **Issue:** `020-02-PLAN.md` includes both non-release and release verification commands.
- **Adjustment:** Ran the release-profile command set only, all with `--features test-fast --features wallet_debug_dump`, per explicit user instruction.
- **Impact:** Plan evidence is grounded in the stricter execution mode already required for this workspace.

**Total deviations:** 2
**Impact on plan:** helper layout stayed within the real `stage_4_utils` tree and verification mode was narrowed to the user-mandated release path.

## Issues Encountered

- The plan’s helper-file inventory had drifted from the real Stage 4 module tree; the closure pass treated those paths as proposed only and summarized the actual layout instead.
- Release runs emitted one non-blocking warning for `Stage4ResolvedPaths::logger_path` being unused in `stage_4.rs`; it did not affect any `020-02` gate.

## Verification Evidence

- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_source_shape -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_tamper -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_cfg_paths -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_chain_path -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_pipeline_genesis_tx -- --nocapture`

## User Setup Required

None - no external setup required for this plan slice.

## Next Phase Readiness

- The tx lane is split and release-verified, so phase 020 can move to `020-03-PLAN.md` for the transfer and bundle split.
- The next bookkeeping step is to keep phase state pointed at plan 03 and carry the same release-only discipline into the transfer or bundle continuity gates.

## Self-Check

PASSED - summary reflects files that exist in the workspace and verification commands that were executed in release mode during this run.

---
*Phase: 020-refactor-scenario-1*
*Completed: 2026-03-26*
