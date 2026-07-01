---
phase: 018-a-b-c
plan: 03
subsystem: scenario_1
tags:
  - simulator
  - stage8
  - checkpoint
  - proof-path
  - acceptance
dependency_graph:
  requires:
    - .planning/phases/018-a-b-c/018-01-SUMMARY.md
    - .planning/phases/018-a-b-c/018-02-SUMMARY.md
    - crates/z00z_simulator/src/scenario_1/stage_8.rs
    - crates/z00z_storage/src/checkpoint/store.rs
  provides:
    - Finalized Stage 8 checkpoint summary surfaces
    - Explicit Stage 6 to Stage 8 proof-path coverage
    - One release-validated end-to-end acceptance lane through final publication
  affects:
    - crates/z00z_simulator/src/scenario_1/scenario_design.yaml
    - crates/z00z_simulator/src/scenario_1/stage_4.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/paths.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/reports.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/reports_diff.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/reports_rows.rs
    - crates/z00z_simulator/src/scenario_1/stage_7.rs
    - crates/z00z_simulator/src/scenario_1/stage_8.rs
    - crates/z00z_simulator/tests/test_scenario1_unified_gate.rs
    - crates/z00z_simulator/tests/test_stage6_checkpoint_final_gate.rs
    - crates/z00z_simulator/tests/test_stage7_jmt_wallet_scan.rs
    - crates/z00z_simulator/tests/test_stage8_proof_path.rs
tech_stack:
  added:
    - crates/z00z_simulator/tests/test_stage8_proof_path.rs
    - transactions/checkpoint/artifact
    - transactions/checkpoint/link
    - transactions/checkpoint/audit
  patterns:
    - Finalized and draft-only checkpoint lanes remain explicitly separate
    - Proof bytes stay bound from tx package to exec input to sealed artifact
    - Wallet evidence artifacts merge refreshed Charlie state into Stage 4 report surfaces
key_files:
  created:
    - .planning/phases/018-a-b-c/018-03-SUMMARY.md
    - crates/z00z_simulator/tests/test_stage8_proof_path.rs
  modified:
    - crates/z00z_simulator/src/scenario_1/scenario_design.yaml
    - crates/z00z_simulator/src/scenario_1/stage_4.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/paths.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/reports.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/reports_diff.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/reports_rows.rs
    - crates/z00z_simulator/src/scenario_1/stage_7.rs
    - crates/z00z_simulator/src/scenario_1/stage_8.rs
    - crates/z00z_simulator/tests/test_scenario1_unified_gate.rs
    - crates/z00z_simulator/tests/test_stage6_checkpoint_final_gate.rs
    - crates/z00z_simulator/tests/test_stage7_jmt_wallet_scan.rs
decisions:
  - Stage 8 finalized output now exposes machine-readable artifact_path, link_path, and audit_path fields alongside checkpoint_id_hex.
  - The explicit proof-path regression binds tx proof bytes across Stage 6 exec input, Stage 7 draft state, and Stage 8 sealed publication instead of relying on placeholder digest fields.
  - Charlie wallet refresh artifacts must merge into the standard Stage 4 report lane so finalized acceptance keeps Bob-visible and Charlie-visible evidence in one stable contract.
metrics:
  completed_date: 2026-03-25
  validation_profile: release
---

# Phase 018 Plan 03: Finalized Checkpoint Acceptance Summary

Finalized Stage 8 publication now exposes sealed artifact surfaces, explicit proof-path binding, and one release-validated acceptance lane from claim continuity through Charlie wallet refresh to checkpoint finalization.

## Outcome

- Extended the Stage 8 summary contract so finalized `OpaqueTest` execution records `checkpoint_id_hex` plus concrete `artifact_path`, `link_path`, and `audit_path` fields while draft-only lanes remain explicitly null.
- Added one proof-path regression that proves Stage 6 exec input, Stage 7 draft apply, and Stage 8 sealed artifact all carry the same proof-bearing transaction bytes.
- Strengthened the unified acceptance gate so it proves Charlie JMT scan evidence, wallet invariant success, ledger continuity, and finalized checkpoint publication together.
- Stabilized the wallet evidence lane by merging refreshed Charlie artifacts back into the standard Stage 4 report files instead of replacing them with Charlie-only snapshots.

## Work Completed

### Task 1: Expose finalized Stage 8 artifact, link, and audit surfaces in summary outputs

- Added finalized-only `artifact_path`, `link_path`, and `audit_path` fields to `Stage8Summary` and kept them absent for `draft_only` runs.
- Updated the Scenario 1 design YAML post-condition notes so the Stage 8 contract explicitly distinguishes draft-only from finalized publication.
- Extended the Stage 6 final gate tests so both finalized and draft-only lanes assert the new summary fields correctly.

### Task 2: Add explicit proof-path and end-to-end acceptance coverage

- Added `test_stage8_proof_path.rs` to bind tx proof bytes across exec input, draft checkpoint state, sealed artifact, link, and audit.
- Expanded the unified acceptance gate to assert finalized Stage 8 summary fields, Charlie scan artifact content, wallet diff evidence, and link/audit continuity.
- Kept the acceptance lane green by merging Charlie wallet state and diff artifacts into the standard Stage 4 report surfaces instead of overwriting prior report files.

## Task Commits

1. **Task 1: Expose finalized Stage 8 artifact, link, and audit surfaces in summary outputs** - `96957d5f` (feat)
2. **Task 2: Add explicit proof-path and end-to-end acceptance coverage** - `598baf09` (feat)

## Validation

### Release Tests

- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage6_checkpoint_final_gate -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage8_proof_path -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_scenario1_unified_gate -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --tests -- --nocapture`

### Result

- All listed release validations passed.
- `test_stage6_checkpoint_final_gate` proves finalized lanes publish sealed checkpoint surfaces while draft-only lanes keep those fields absent.
- `test_stage8_proof_path` proves proof bytes remain bound from the tx package into exec input and sealed publication.
- `test_scenario1_unified_gate` proves the full claim-to-finalize acceptance lane, including Charlie wallet evidence and finalized checkpoint continuity.
- The full `z00z_simulator` release test surface passed after the final acceptance-lane stabilization.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Finalized acceptance lane needed merged wallet evidence instead of Charlie-only file replacement**

- **Found during:** Task 2 release acceptance verification
- **Issue:** Stage 7 refresh work replaced Stage 4 report artifacts with Charlie-only snapshots, which broke the unified acceptance surface and hid Bob-side evidence expected by existing gates.
- **Fix:** Added wallet-state and diff merge helpers, reused Stage 4 resolved paths directly, and wrote refreshed Charlie evidence back into the shared report lane.
- **Files modified:** `crates/z00z_simulator/src/scenario_1/stage_4.rs`, `crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs`, `crates/z00z_simulator/src/scenario_1/stage_4_utils/paths.rs`, `crates/z00z_simulator/src/scenario_1/stage_4_utils/reports.rs`, `crates/z00z_simulator/src/scenario_1/stage_4_utils/reports_diff.rs`, `crates/z00z_simulator/src/scenario_1/stage_4_utils/reports_rows.rs`, `crates/z00z_simulator/src/scenario_1/stage_7.rs`
- **Verification:** `test_stage8_proof_path`, `test_scenario1_unified_gate`, and the full release simulator suite passed.
- **Committed in:** `598baf09`

**2. [Rule 2 - Missing Critical] Finalized acceptance needed one explicit proof-path regression**

- **Found during:** Task 2 completion review
- **Issue:** Existing final-gate tests proved publication outputs existed, but they did not explicitly bind the proof-bearing tx bytes across exec input, draft state, and sealed artifact publication.
- **Fix:** Added `test_stage8_proof_path.rs` and extended the unified gate to assert the sealed link and audit continuity surfaces.
- **Files modified:** `crates/z00z_simulator/tests/test_stage8_proof_path.rs`, `crates/z00z_simulator/tests/test_scenario1_unified_gate.rs`
- **Verification:** `test_stage8_proof_path` and `test_scenario1_unified_gate` passed in release.
- **Committed in:** `598baf09`

## Auth Gates

None.

## Known Stubs

None.

## Deferred Issues

- The `/GSD-Review-Tasks-Execution` slash workflow referenced by the plan was not invokable from the terminal-only execution environment. Release tests, static diagnostics, and phase summaries were used as the executable verification substitute.

## Self-Check: PASSED

- Confirmed this summary file exists at `.planning/phases/018-a-b-c/018-03-SUMMARY.md`.
- Confirmed both task commits exist: `96957d5f` and `598baf09`.
- Confirmed the release validation set listed above completed green after the final acceptance-lane stabilization.
