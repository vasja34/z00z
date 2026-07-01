---
phase: 018-a-b-c
plan: 02
subsystem: scenario_1
tags:
  - simulator
  - stage7
  - jmt
  - wallet
  - checkpoint
dependency_graph:
  requires:
    - .planning/phases/018-a-b-c/018-01-SUMMARY.md
    - crates/z00z_simulator/src/scenario_1/stage_6.rs
    - crates/z00z_storage/src/assets/store.rs
    - crates/z00z_storage/src/assets/proof.rs
  provides:
    - Proof-validated committed-state JMT wallet scan
    - Charlie runtime refresh after canonical Stage 7 apply
    - Wallet evidence refresh and invariant gate bound to committed outputs
  affects:
    - crates/z00z_simulator/src/scenario_1/jmt_wallet_scan.rs
    - crates/z00z_simulator/src/scenario_1/stage_6.rs
    - crates/z00z_simulator/src/scenario_1/stage_7.rs
    - crates/z00z_simulator/tests/test_stage6_checkpoint.rs
    - crates/z00z_simulator/tests/test_stage6_checkpoint_storage_bridge.rs
    - crates/z00z_simulator/tests/test_stage7_jmt_wallet_scan.rs
    - crates/z00z_simulator/tests/test_scenario1_unified_gate.rs
tech_stack:
  added:
    - crates/z00z_simulator/src/scenario_1/jmt_wallet_scan.rs
    - transactions/charlie_jmt_scan.json
    - transactions/wallets_state_before.json
    - transactions/wallets_state_after.json
    - transactions/wallets_state_diff.json
  patterns:
    - Proof before ownership detection
    - Persisted bridge-output reuse across Stage 6 and Stage 7
    - Wallet delta equals detected committed amount invariant
key_files:
  created:
    - .planning/phases/018-a-b-c/018-02-SUMMARY.md
    - crates/z00z_simulator/src/scenario_1/jmt_wallet_scan.rs
    - crates/z00z_simulator/tests/test_stage7_jmt_wallet_scan.rs
  modified:
    - crates/z00z_simulator/src/scenario_1/stage_4.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/paths.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/reports.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/reports_capture.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/reports_diff.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/reports_rows.rs
    - crates/z00z_simulator/src/scenario_1/stage_6.rs
    - crates/z00z_simulator/src/scenario_1/stage_7.rs
    - crates/z00z_simulator/tests/test_stage6_checkpoint.rs
    - crates/z00z_simulator/tests/test_scenario1_unified_gate.rs
decisions:
  - Stage 7 reuses persisted Stage 6 bridge outputs instead of rebuilding recipient outputs from Stage 4 transaction rows.
  - Charlie runtime refresh runs only from proof-validated committed post-apply rows and writes refreshed standard wallet evidence artifacts.
  - Wallet lifecycle validation uses canonical receive statuses internally and normalizes exported artifacts back to pending and confirmed for report compatibility.
metrics:
  completed_date: 2026-03-25
  validation_profile: release
---

# Phase 018 Plan 02: Stage 7 JMT Wallet Scan Summary

Proof-validated committed-state JMT scan drives Charlie wallet refresh, wallet evidence export, and invariant-gated Stage 7 acceptance.

## Outcome

- Added one reusable committed-state JMT scan helper that proves inclusion before running wallet ownership detection.
- Refreshed Charlie runtime from canonical Stage 7 committed outputs instead of detached leaf data or reconstructed Stage 4 outputs.
- Wrote machine-readable and standard wallet evidence artifacts that make the `leaf scan` versus `JMT scan` distinction explicit.
- Restored Stage 6 checkpoint expectations so they match persisted bridge-output truth under the new Stage 6 to Stage 7 seam.

## Work Completed

### Task 1: Add committed-state proof-validated JMT scan helper

- Added `jmt_wallet_scan.rs` with candidate enumeration, proof validation, actor scan, and artifact reporting helpers.
- Locked the scan contract to `proof_blob + chk_blob` before stealth ownership detection.
- Added release regression coverage proving committed-proof success and detached leaf rejection.

### Task 2: Refresh Charlie runtime and wallet evidence from committed state

- Persisted Stage 6 `bridge_outputs` and reused those exact outputs in Stage 7 apply instead of rebuilding them from Stage 4 rows.
- Added Charlie-only runtime refresh, imported matched committed outputs through wallet RPC, and captured before/after/diff report artifacts.
- Added one invariant gate requiring Charlie wallet delta to equal the total detected committed JMT amount.
- Updated Stage 6 checkpoint regression tests so `created_delta` now proves bridge-output semantics rather than stale Stage 4 fee/change assumptions.

## Task Commits

1. **Task 1: Add committed-state proof-validated JMT scan helper** - `f7f31722` (feat)
2. **Task 2: Refresh Charlie runtime and wallet evidence from committed state** - `f5a3392a` (feat)

## Validation

### Release Tests

- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage6_checkpoint -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage6_checkpoint_storage_bridge -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage7_jmt_wallet_scan -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_scenario1_unified_gate -- --nocapture`

### Result

- All listed release validations passed.
- `test_stage6_checkpoint` finished green after aligning stale checkpoint expectations with persisted bridge outputs.
- `test_stage7_jmt_wallet_scan` finished green with the Charlie update path and committed-proof checks.
- `test_scenario1_unified_gate` finished green with the new JMT scan and wallet diff artifacts required by the acceptance lane.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Stage 7 rebuilt outputs with mismatched asset identity**

- **Found during:** Task 2 Stage 7 release verification
- **Issue:** Proof validation succeeded, but Charlie ownership scan found no committed matches because Stage 7 rebuilt outputs differently from Stage 6 bridge rows.
- **Fix:** Persisted exact Stage 6 `bridge_outputs` and reused them for Stage 7 checkpoint and runtime refresh work.
- **Files modified:** `crates/z00z_simulator/src/scenario_1/stage_6.rs`, `crates/z00z_simulator/src/scenario_1/stage_7.rs`
- **Verification:** `test_stage7_jmt_wallet_scan` and `test_stage6_checkpoint_storage_bridge` passed in release.
- **Committed in:** `f5a3392a`

**2. [Rule 2 - Missing Critical] Wallet evidence refresh needed a balance invariant gate**

- **Found during:** Task 2 wallet evidence integration
- **Issue:** Refreshed artifacts alone did not prove that imported committed outputs matched the wallet-visible amount delta.
- **Fix:** Added before/after total comparison and required delta equality with the committed JMT detected amount.
- **Files modified:** `crates/z00z_simulator/src/scenario_1/stage_7.rs`, `crates/z00z_simulator/src/scenario_1/stage_4_utils/reports_diff.rs`
- **Verification:** `test_stage7_jmt_wallet_scan` and `test_scenario1_unified_gate` passed in release.
- **Committed in:** `f5a3392a`

**3. [Rule 1 - Bug] Stage 6 checkpoint tests still encoded old created-delta semantics**

- **Found during:** Task 2 release verification
- **Issue:** Checkpoint regression tests still expected original Stage 4 fee and change outputs inside `created_delta`, which was no longer true after bridge-output persistence became canonical.
- **Fix:** Reworked the tests to read `checkpoint_bridge_s6.json`, compare `created_delta` with persisted `bridge_outputs`, and assert Stage 4 fee/change outputs are excluded.
- **Files modified:** `crates/z00z_simulator/tests/test_stage6_checkpoint.rs`
- **Verification:** `test_stage6_checkpoint` passed in release.
- **Committed in:** `f5a3392a`

## Auth Gates

None.

## Known Stubs

None.

## Deferred Issues

- Codacy complexity warnings remain on large pre-existing stage files. They were not introduced by this plan and are better handled by the planned Scenario 1 refactor phase.

## Self-Check: PASSED

- Confirmed this summary file exists at `.planning/phases/018-a-b-c/018-02-SUMMARY.md`.
- Confirmed both task commits exist: `f7f31722` and `f5a3392a`.
- Confirmed the release regression set listed above completed green after the final checkpoint-test alignment.
