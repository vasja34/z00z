---
phase: 020-refactor-scenario-1
plan: 04
subsystem: simulator
tags: [scenario_1, yaml-sync, release-verification, stage-surface, legacy-test-alignment]
requires:
  - phase: 020-03
    provides: explicit transfer and bundle split through final stage 10
provides:
  - final synchronized 12-stage runner and YAML surface through stage 12
  - release-green Scenario 1 closure across targeted gates, full suite, and release binary
  - aligned legacy stage4 and apply-path tests that match the post-split stage boundaries
affects: [phase-020-closeout, scenario_1]
tech-stack:
  added: []
  patterns: [release-only simulator verification, legacy test contract normalization, explicit stage-map guardrail]
key-files:
  created:
    - .planning/phases/020-refactor-scenario-1/020-04-SUMMARY.md
  modified:
    - crates/z00z_simulator/src/scenario_1/runner.rs
    - crates/z00z_simulator/src/scenario_1/scenario_design.yaml
    - crates/z00z_simulator/tests/support/test_stage4_support.rs
    - crates/z00z_simulator/tests/test_s7_examples.rs
    - crates/z00z_simulator/tests/test_scenario1_stage_surface.rs
    - crates/z00z_simulator/tests/test_stage4_cfg_guards.rs
    - crates/z00z_simulator/tests/test_stage4_claim_gate.rs
    - crates/z00z_simulator/tests/test_stage4_gates.rs
    - crates/z00z_simulator/tests/test_stage4_output_crypto.rs
    - crates/z00z_simulator/tests/test_stage4_selection.rs
    - crates/z00z_simulator/tests/test_stage4_split.rs
    - .planning/phases/020-refactor-scenario-1/020-VALIDATION.md
    - .planning/phases/020-refactor-scenario-1/020-TEST-SPEC.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
key-decisions:
  - Keep final stage 11 and 12 `rust_entry` bindings explicit in both runner fallback design and `scenario_design.yaml`.
  - Normalize legacy stage4-oriented tests at the test-contract layer instead of widening runtime compatibility shims in production code.
  - Treat `stage_4_snapshot.json` as a valid upstream `claim_publish` artifact on later tx-plan failures.
patterns-established:
  - Scenario 1 closure now requires the targeted release gates, the full `z00z_simulator` release suite, and the release `scenario_1` binary.
  - Legacy stage-number assertions must follow the explicit 12-stage map instead of old coarse-stage boundaries.
requirements-completed: [SCN1-03, SCN1-04, SCN1-05]
duration: multi-session
completed: 2026-03-26
---

# Phase 020 Plan 04: Final Stage Surface And Release Closure Summary

📌 Phase 020 is now closed on one synchronized 12-stage Scenario 1 map, green release-style simulator gates, and updated legacy tests that match the post-split runtime contract.

## Performance

- **Duration:** multi-session
- **Completed:** 2026-03-26
- **Tasks:** 2
- **Files modified:** 15+

## Accomplishments

- 📌 Synchronized the final runner fallback surface and `scenario_design.yaml` so stages 11 and 12 explicitly bind `stage_7::run_apply(ctx, stage)` and `stage_8::run_finalize(ctx, stage)`.
- 📌 Strengthened `test_scenario1_stage_surface.rs` so it asserts the full 12-stage executed order and the final stage-entry mapping through stage 12.
- 📌 Updated `test_s7_examples.rs` to validate the post-split apply/finalize continuity at stages 9, 11, and 12, including `checkpoint_s7.json` reporting stage `11`.
- 📌 Normalized stale stage4-era tests so they now target the post-split tx-plan boundary at stage `5`, accept `stage_4_snapshot.json` as an upstream `claim_publish` artifact, and use fee-safe happy-path fixtures where the earlier low-input fixtures now fail legitimately.
- 📌 Revalidated the full Scenario 1 lane in release mode, including targeted gates, the full `z00z_simulator` suite, and the release `scenario_1` binary.

## Task Commits

📌 No task commits were created in this execution slice.

📌 Reason: no explicit commit request was given during this run, and the closure work was kept as local workspace changes only.

## Files Created/Modified

- `crates/z00z_simulator/src/scenario_1/runner.rs` - aligns fallback design entrypoints with the final stage-signature shape.
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml` - locks final stage 11 and 12 `rust_entry` bindings to the current public entrypoints.
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` - enforces the full 12-stage executed surface.
- `crates/z00z_simulator/tests/test_s7_examples.rs` - aligns the apply-path example test with final stage numbering.
- `crates/z00z_simulator/tests/support/test_stage4_support.rs` - provides legacy stage4 test lookup compatibility for the post-split tx-plan stage.
- `crates/z00z_simulator/tests/test_stage4_cfg_guards.rs` - stops treating `stage_4_snapshot.json` as a post-tx artifact that must disappear on tx-plan failure.
- `crates/z00z_simulator/tests/test_stage4_claim_gate.rs` - keeps claim-gate negative-path assertions aligned with the upstream snapshot contract.
- `crates/z00z_simulator/tests/test_stage4_gates.rs` - uses a fee-safe happy-path fixture and logger stage `5` assertions.
- `crates/z00z_simulator/tests/test_stage4_output_crypto.rs` - uses a fee-safe fee-sink happy path and preserves negative-path post-state assertions without invalid snapshot assumptions.
- `crates/z00z_simulator/tests/test_stage4_selection.rs` - removes the stale snapshot-absence assertion on tx-plan failure.
- `crates/z00z_simulator/tests/test_stage4_split.rs` - expects negative-path rejection at stage `5` instead of stage `6`.
- `.planning/phases/020-refactor-scenario-1/020-VALIDATION.md` - records green `020-04` validation evidence.
- `.planning/phases/020-refactor-scenario-1/020-TEST-SPEC.md` - records the final closure set and post-split test-contract notes.
- `.planning/ROADMAP.md` - marks phase 020 complete.
- `.planning/STATE.md` - marks phase 020 and the current milestone progress complete.

## Decisions Made

- 📌 Kept the runtime surface explicit and fixed the remaining drift in the YAML and fallback design instead of adding another compatibility layer around stage lookup.
- 📌 Corrected stale stage4-era tests at the fixture and expectation layer because the production runtime was already behaving correctly.
- 📌 Treated the Stage 4 snapshot as an upstream artifact owned by `claim_publish`, which remains valid even when the later tx-plan gate rejects the run.
- 📌 Preserved the user-required release-only validation policy for final closeout.

## Deviations from Plan

### Legacy stage4 test normalization expanded beyond the initial file list

- **Issue:** The full release suite exposed several stale stage4-era tests that still assumed old tx-plan numbering, invalid low-input happy-path fixtures, or disappearance of `stage_4_snapshot.json` on later failures.
- **Adjustment:** Updated the affected tests and the shared stage4 test helper to match the final 12-stage runtime contract and the current upstream artifact boundaries.
- **Impact:** The production code remained minimal while the release suite now reflects the actual post-split behavior.

### Release closure uncovered one stale apply example after the targeted gate set was already green

- **Issue:** `test_s7_examples.rs` still asserted `checkpoint_s7.json.stage == 7` even though the explicit final map moved storage apply to stage `11`.
- **Adjustment:** Updated the test to assert the final stage ids and the storage-bridge continuity that the refactor actually guarantees.
- **Impact:** The release full-suite gate now agrees with the already-green release binary and targeted continuity tests.

**Total deviations:** 2

## Issues Encountered

- 📌 The only runtime output that remained during verification was the pre-existing non-blocking `dead_code` warning for `Stage4ResolvedPaths::logger_path` in `stage_4.rs`.
- 📌 The final closeout effort was dominated by stale tests, not production regressions.

## Verification Evidence

- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_cfg_paths -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_chain_path -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_acceptance -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage5_receive_bridge -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_storage_bridge -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_final_gate -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage7_jmt_wallet_scan -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage8_proof_path -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_pipeline_genesis_tx -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_unified_gate -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_s7_examples -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_card_gate -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_cfg_guards -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_gates -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_output_crypto -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_split -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump`
- `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump`

## User Setup Required

📌 None.

## Next Phase Readiness

- 📌 Phase 020 is complete.
- 📌 The next step is to define a new phase only if the remaining simulator or wallet backlog is intentionally promoted into active work.

## Self-Check

📌 PASSED - summary references files that exist in the workspace and verification commands that were executed during this closure slice.

---
*Phase: 020-refactor-scenario-1*
*Completed: 2026-03-26*
