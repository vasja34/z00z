---
phase: 030-refactor-long-files
plan: 04
subsystem: simulator-utils
tags: [rust, simulator, io, seams, facade]
requires:
  - phase: 029-crypto-audit-wallets
    provides: release-style wallet and simulator verification anchors
provides:
  - stable simulator stage roots with extracted Stage 4 and Stage 6 helper seams
  - stable `z00z_utils::io` facade with split filesystem helper responsibilities
  - structural regression coverage aligned to the new seam owners
affects: [030-05, 030-10, z00z_simulator, z00z_utils, z00z_storage, z00z_wallets]
tech-stack:
  added: []
  patterns: [stage-root facade split, io facade-owner split, structural seam regression gating]
key-files:
  created:
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/input_selection_scan.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/output_construction.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/stage4_reporting.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_preparation_core.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/wallet_state_capture.rs
    - crates/z00z_simulator/src/scenario_1/stage_6_utils/bridge_output_router.rs
    - crates/z00z_simulator/src/scenario_1/stage_6_utils/demo_checkpoint_agg.rs
    - crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs
    - crates/z00z_simulator/src/scenario_1/stage_6_utils/fragment_construction.rs
    - crates/z00z_simulator/src/scenario_1/stage_6_utils/prep_snapshot_loader.rs
    - crates/z00z_simulator/src/scenario_1/stage_6_utils/stage6_logging.rs
    - crates/z00z_utils/src/io/atomic_write.rs
    - crates/z00z_utils/src/io/bincode_io.rs
    - crates/z00z_utils/src/io/file_read.rs
    - crates/z00z_utils/src/io/json_io.rs
    - crates/z00z_utils/src/io/yaml_io.rs
  modified:
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs
    - crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs
    - crates/z00z_simulator/tests/test_claim_acceptance.rs
    - crates/z00z_simulator/tests/test_stage4_split.rs
    - crates/z00z_storage/tests/snapshot/test_replay_bound.rs
    - crates/z00z_utils/src/io/fs.rs
    - crates/z00z_wallets/tests/test_s5_closure_gate.rs
key-decisions:
  - Keep `tx_lane_impl.rs`, `bundle_lane_impl.rs`, and `z00z_utils::io` as stable caller-visible facades while moving helper logic into sibling seams.
  - Fix stale structural tests to assert the new seam owners instead of reintroducing compatibility clutter into production code.
  - Treat release-style verification fallout as scope-local cleanup when the split leaves dead helpers, unused imports, or missing seam reexports behind.
patterns-established:
  - "Stage-root facade split: stage roots stay shallow while sibling seams own preparation, validation, routing, and logging helpers."
  - "Facade-owner split: fs.rs owns the I/O helper seams and reexports through the stable `z00z_utils::io` surface."
requirements-completed: [PH30-SEAMS, PH30-VERIFY]
duration: 2h 09m
completed: 2026-03-31
---

# Phase 030 Plan 04 Summary

📌 Stable simulator stage roots and the `z00z_utils::io` facade were preserved while helper-heavy logic moved into coherent seam modules, with structural regressions and release-style validation closing the split.

## Performance

- 📌 Duration: 2h 09m
- 📌 Started: 2026-03-31T11:00:00Z
- 📌 Completed: 2026-03-31T13:08:59Z
- 📌 Tasks: 2
- 📌 Files modified: 25

## Accomplishments

- 📌 Split Stage 4 and Stage 6 simulator helper lanes into semantically real preparation, validation, routing, snapshot, fragment, and reporting seams while keeping the outer stage roots stable.
- 📌 Split `crates/z00z_utils/src/io/fs.rs` into focused atomic-write, read-path, and codec-specific I/O seams without widening the public `z00z_utils::io` API.
- 📌 Updated structural regression tests to follow the new seam owners and completed targeted plus release-style validation on the post-split tree.

## Task Commits

📌 No git commit was created in this closeout because the repository contains unrelated dirty files and the current workflow requires explicit version-manager git operations only on request.

## Files Created/Modified

- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs` - Kept as the stable Stage 4 root while helper internals moved to sibling seams.
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` - Kept as the stable Stage 6 root and adjusted to reexport shared seam helpers needed by sibling modules.
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs` - Reexported extracted Stage 4 helpers needed across the split surface.
- `crates/z00z_utils/src/io/fs.rs` - Reduced to the stable owner facade over private I/O seams and added a structural split regression.
- `crates/z00z_simulator/tests/test_claim_acceptance.rs` - Moved source-shape assertions to `tx_validation_gates.rs` where the validation calls now live.
- `crates/z00z_storage/tests/snapshot/test_replay_bound.rs` - Aligned snapshot-loading assertions with `prep_snapshot_loader.rs`.
- `crates/z00z_wallets/tests/test_s5_closure_gate.rs` - Aligned Stage 5 closure assertions with `tx_validation_gates.rs`.
- `crates/z00z_simulator/tests/test_stage4_split.rs` - Preserved structural regression coverage for the simulator split.

## Decisions Made

- 📌 Preserve stable stage and I/O entrypoints, and move only helper-heavy internals into sibling seams.
- 📌 Prefer truthful test updates over compatibility shims when source-shape checks point at modules that no longer own the behavior.
- 📌 Keep helper ownership single-sourced so release gates cannot drift on duplicate validation helpers or local size constants.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Restored shared helper visibility across simulator seam boundaries**

- **Found during:** Task 1 validation
- **Issue:** Extracted helpers such as `out_hash_hex`, `decode_hex32`, `parse_list_asset_rows`, and `extract_next_cursor` were no longer reachable across the new seam boundaries.
- **Fix:** Reexported the shared helpers from the stable stage roots and helper facades so sibling seam modules could resolve them without widening the public stage surface.
- **Files modified:** `crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs`, `crates/z00z_simulator/src/scenario_1/stage_4_utils/input_selection_scan.rs`, `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`, `crates/z00z_simulator/src/scenario_1/stage_6_utils/demo_checkpoint_agg.rs`, `crates/z00z_simulator/src/scenario_1/stage_6_utils/fragment_construction.rs`
- **Verification:** Re-ran the Stage 4 split test, genesis integration test, release Scenario 1 binary, and the full simulator release suite.
- **Committed in:** not committed in this closeout

**2. [Rule 1 - Bug] Updated structural tests to the new seam owners**

- **Found during:** Wider release-style validation
- **Issue:** Several tests still asserted that helper implementations lived in the old monolithic root files even though the split had moved them into the new seam modules.
- **Fix:** Updated the simulator, storage, and wallet structural tests to point at `tx_validation_gates.rs` and `prep_snapshot_loader.rs` instead of forcing production compatibility clutter.
- **Files modified:** `crates/z00z_simulator/tests/test_claim_acceptance.rs`, `crates/z00z_storage/tests/snapshot/test_replay_bound.rs`, `crates/z00z_wallets/tests/test_s5_closure_gate.rs`
- **Verification:** Re-ran the full simulator release suite and the max-safe workspace verification gate.
- **Committed in:** not committed in this closeout

**3. [Rule 1 - Bug] Removed dead split fallout surfaced by `-D warnings`**

- **Found during:** `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run`
- **Issue:** The split left a duplicate `validate_bob_count` helper and an unused `distinct_serial_target` import in `tx_lane_impl.rs`, which failed the warning-as-error gate.
- **Fix:** Removed the dead duplicate helper and the unused import, then reformatted the workspace.
- **Files modified:** `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs`
- **Verification:** Re-ran the max-safe workspace verification gate to a clean summary.
- **Committed in:** not committed in this closeout

---

📌 Total deviations: 3 auto-fixed bugs
📌 Impact on plan: The fixes kept scope inside the seam split and validation closure work required by `PH30-VERIFY`.

## Issues Encountered

- 📌 Release-style validation exposed stale source-shape tests in downstream crates that were coupled to the old helper-file ownership.
- 📌 The repository was already dirty outside Phase 030, so closeout must stay documentation-only until an explicit version-manager git request is given.

## User Setup Required

📌 None - no external service configuration or secrets were required for this plan.

## Next Phase Readiness

- 📌 Later Phase 030 wallet and caller-normalization waves can build on these smaller simulator and I/O facades without reopening the Stage 4, Stage 6, or `z00z_utils::io` ownership split.
- 📌 Structural tests now encode the truthful seam owners, reducing pressure to add compatibility clutter in later refactor waves.

## Verification

- 📌 `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- 📌 `cargo test -p z00z_utils --release --all-targets -- --nocapture`
- 📌 `cargo test -p z00z_utils --release --features test-fast --all-targets -- --nocapture`
- 📌 `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_split -- --nocapture`
- 📌 `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_genesis_integration -- --nocapture`
- 📌 `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump`
- 📌 `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump -- --nocapture`
- 📌 `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run`

## Self-Check

📌 PASSED for summary creation, planning-state sync, targeted verification coverage, and the clean visible max-safe verification summary (`planned=312 skipped=21 failed=0`).

📌 Git closeout intentionally left undone because no explicit commit or push request was given and the repository contains unrelated dirty files that must stay outside any staged release flow.

---
*Phase: 030-refactor-long-files*
*Completed: 2026-03-31*
