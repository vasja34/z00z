---
phase: 018-a-b-c
plan: 01
subsystem: scenario_1
tags:
  - simulator
  - stage4
  - continuity
  - storage
dependency_graph:
  requires:
    - crates/z00z_simulator/src/scenario_1/stage_4.rs
    - crates/z00z_simulator/src/scenario_1/storage_view.rs
    - crates/z00z_storage/src/assets/store.rs
    - crates/z00z_storage/src/snapshot/store.rs
  provides:
    - Full claim-backed Stage 4 continuity
    - Canonical ledger continuity sidecar
    - Release-validated continuity regressions
  affects:
    - crates/z00z_simulator/src/scenario_1/stage_3.rs
    - crates/z00z_simulator/src/scenario_1/stage_3_utils/wallet_flow.rs
    - crates/z00z_simulator/src/scenario_1/stage_4.rs
    - crates/z00z_simulator/src/scenario_1/storage_view.rs
    - crates/z00z_simulator/tests/test_stage4_chain_path.rs
    - crates/z00z_simulator/tests/test_stage4_root_support.rs
    - crates/z00z_simulator/tests/test_stage6_checkpoint.rs
tech_stack:
  added:
    - outputs/storage/ledger_path.json
  patterns:
    - Full-store canonical snapshot
    - Fail-closed continuity validation
    - Storage-owned snapshot reference handoff
key_files:
  created:
    - .planning/phases/018-a-b-c/018-01-SUMMARY.md
  modified:
    - crates/z00z_simulator/src/scenario_1/stage_3.rs
    - crates/z00z_simulator/src/scenario_1/stage_3_utils/wallet_flow.rs
    - crates/z00z_simulator/src/scenario_1/stage_4.rs
    - crates/z00z_simulator/src/scenario_1/storage_view.rs
    - crates/z00z_simulator/tests/test_stage4_chain_path.rs
    - crates/z00z_simulator/tests/test_stage4_root_support.rs
    - crates/z00z_simulator/tests/test_stage4_tamper.rs
    - crates/z00z_simulator/tests/test_stage6_checkpoint.rs
decisions:
  - Stage 4 continuity must derive from the full claim-backed live store instead of a selected-input subset root.
  - Canonical continuity export stays separate from observational summaries and is emitted as outputs/storage/ledger_path.json.
  - Witness tamper must fire at SpendWitness verification time, not during canonical snapshot construction.
metrics:
  completed_date: 2026-03-24
  validation_profile: release
---

# Phase 018 Plan 01: Stage 4 Continuity Summary

Full claim-backed Stage 4 continuity with one canonical ledger-path sidecar and release-green simulator validation.

## Outcome

- Rebased Stage 4 continuity on the full claim-backed store instead of the selected execution subset.
- Added `outputs/storage/ledger_path.json` as the machine-readable continuity sidecar tying `claim_post`, `pre_tx`, `post_tx`, and final checkpoint state together.
- Preserved execution-input selection for transaction building while making snapshot continuity canonical and global.
- Restored fail-closed behavior for continuity drift and witness tampering under the new full-store model.

## Work Completed

### Task 1: Rebase Stage 4 continuity on the full claim-backed store

- Aligned Stage 3 wallet import with claim-backed assets and removed double-wrapping during import.
- Rebuilt the Stage 4 live claim store from verified claim packages and checked it against persisted `claim_post` continuity.
- Changed `build_canon_snapshot` usage so canonical snapshot materialization comes from the full claim-backed store.
- Kept selected rows as execution inputs only and validated them against the canonical full-store snapshot.
- Tightened `sync_prep_path` so canonical path or leaf drift fails closed instead of being silently normalized.

### Task 2: Emit one canonical ledger-path artifact for claim to post-apply continuity

- Added `outputs/storage/ledger_path.json` persistence through the storage export surface.
- Updated continuity export hooks for `claim_post`, `pre_tx`, `post_tx`, and final checkpoint publication.
- Extended regression coverage so ledger-path fields are asserted directly by Stage 4 chain-path tests.

## Validation

### Release Tests

- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage4_root_support -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage4_chain_path -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage4_tamper -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage6_checkpoint stage4_prep_order_kept -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator`

### Result

- All listed release validations passed.
- Final crate-wide result: `z00z_simulator` release suite completed green with no remaining failures.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Witness tamper failed too early as canonical leaf drift**

- **Found during:** Full `z00z_simulator` release verification
- **Issue:** The test witness tamper mutated selected inputs before canonical snapshot construction, causing `canonical leaf mismatch` instead of the intended `SpendWitness gate` failure class.
- **Fix:** Delayed `apply_wit_tamper` until immediately before `verify_spend_witness_gate`.
- **Files modified:** `crates/z00z_simulator/src/scenario_1/stage_4.rs`

**2. [Rule 1 - Bug] Stage 4 unit test still encoded subset-snapshot assumptions**

- **Found during:** Full `z00z_simulator` release verification
- **Issue:** `canonical_snapshot_keeps_prep_order` assumed canonical snapshot order matched execution-input order.
- **Fix:** Updated the test to assert that `prep.rows` preserves selection order while canonical snapshot entries follow store order.
- **Files modified:** `crates/z00z_simulator/src/scenario_1/stage_4.rs`

**3. [Rule 1 - Bug] Stage 6 checkpoint regression still assumed subset snapshot order**

- **Found during:** Full `z00z_simulator` release verification
- **Issue:** `stage4_prep_order_kept` compared full canonical snapshot contents directly against `tx.inputs` order.
- **Fix:** Updated the test to assert that every execution input is present in the canonical snapshot and that the snapshot may legitimately be larger than the execution subset.
- **Files modified:** `crates/z00z_simulator/tests/test_stage6_checkpoint.rs`

## Auth Gates

None.

## Known Stubs

None.

## Deferred Issues

- Broad Stage 8 proof-binding concerns reported in autonomous review remain outside the scope of Plan 018-01 and were not changed here.

## Self-Check: PASSED

- Confirmed this summary file exists at `.planning/phases/018-a-b-c/018-01-SUMMARY.md`.
- Confirmed `z00z_simulator` release validation completed green after the final Stage 4 and Stage 6 fixes.
