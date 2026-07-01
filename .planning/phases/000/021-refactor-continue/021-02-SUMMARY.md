---
phase: 021-refactor-continue
plan: 02
subsystem: simulator
tags: [scenario_1, claim_lane, canonical_stage_surface, release_verification, regression_tests]
requires:
  - phase: 021-refactor-continue
    provides: wave-01 canonical stage scaffold for Scenario 1 root files
provides:
  - claim prepare and claim publish ownership split across canonical stage_3.rs and stage_4.rs
  - release-verified claim regression coverage aligned to canonical stage and lane owner files
  - review-clean planning docs for the claim-lane wave closure
affects: [021-03, 021-04, 021-05, scenario_1, release-gates]
tech-stack:
  added: []
  patterns: [canonical root stage dispatch, lane-owner source-shape guards, release-first regression closure]
key-files:
  created:
    - .planning/phases/021-refactor-continue/021-02-SUMMARY.md
  modified:
    - crates/z00z_simulator/src/scenario_1/mod.rs
    - crates/z00z_simulator/src/scenario_1/runner.rs
    - crates/z00z_simulator/src/scenario_1/stage_3.rs
    - crates/z00z_simulator/src/scenario_1/stage_4.rs
    - crates/z00z_simulator/tests/test_claim_acceptance.rs
    - crates/z00z_storage/tests/snapshot/test_replay_bound.rs
    - crates/z00z_wallets/tests/test_s5_closure_gate.rs
    - .planning/phases/021-refactor-continue/021-01-PLAN.md
    - .planning/phases/021-refactor-continue/021-01-SUMMARY.md
    - .planning/phases/021-refactor-continue/021-02-PLAN.md
key-decisions:
  - Keep claim publish on canonical root stage_4.rs and keep stage_3.rs limited to claim prepare plus claim genesis.
  - Fix release-gate fallout in cross-crate source-shape tests instead of reintroducing compatibility shims in simulator production code.
  - Close the plan only after two consecutive clean review passes on the updated planning docs.
patterns-established:
  - Cross-crate source-shape guards must follow the real lane owner file under stage_*_utils when canonical root files become thin facades.
  - Planning docs for completed waves must be kept aligned with the current canonical Scenario 1 root contract before later-wave closure.
requirements-completed: [SCN1-06]
duration: multi-session
completed: 2026-03-27
---

# Phase 021 Plan 02: Claim Lane Canonical Cutover Summary

Claim publish now lives on canonical stage_4.rs, claim prepare stays on stage_3.rs, and release-gate regressions were closed without restoring compatibility shims.

## Performance

- **Duration:** multi-session
- **Completed:** 2026-03-27
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- Kept the claim lane split on the canonical root surface: stage 3 owns claim prepare and claim genesis, and stage 4 owns claim publish.
- Re-ran the claim-focused release chain and the full release suite, then fixed the remaining cross-crate source-shape fallout in storage and wallet tests.
- Completed the required review loop with two consecutive clean passes after aligning 021 planning docs to the canonical stage-numbered Scenario 1 contract.

## Task Commits

Each task was committed atomically in the available workspace state:

1. **Task 1: Move claim-publish behavior into its dedicated canonical stage and settle run_claim_genesis ownership** - `88b933a8` (feat, landed inside a broader versioned workspace commit that included the claim-lane cutover)
2. **Task 2: Lock claim-lane regressions to the new file split without changing artifacts** - `32180752` (test)

**Plan metadata:** pending in current execution slice

## Files Created/Modified

- `crates/z00z_simulator/src/scenario_1/stage_3.rs` - slim claim-prepare and claim-genesis owner after publish extraction.
- `crates/z00z_simulator/src/scenario_1/stage_4.rs` - canonical root claim-publish owner used by runner dispatch.
- `crates/z00z_simulator/src/scenario_1/mod.rs` - re-exports canonical claim entrypoints from the split stage files.
- `crates/z00z_simulator/src/scenario_1/runner.rs` - dispatches stage 4 directly to `stage_4::run_claim_publish`.
- `crates/z00z_simulator/tests/test_claim_acceptance.rs` - points tx-lane source-shape assertions at the real tx lane owner file.
- `crates/z00z_storage/tests/snapshot/test_replay_bound.rs` - points snapshot replay assertions at tx and bundle lane implementation owners.
- `crates/z00z_wallets/tests/test_s5_closure_gate.rs` - points wallet closure assertions at the tx lane implementation owner.
- `.planning/phases/021-refactor-continue/021-02-PLAN.md` - aligns success criteria and artifact references to canonical stage numbering.
- `.planning/phases/021-refactor-continue/021-01-PLAN.md` - aligns the earlier scaffold-wave narrative to the canonical root stage surface.
- `.planning/phases/021-refactor-continue/021-01-SUMMARY.md` - aligns the historical wave-01 summary to canonical stage numbering.

## Decisions Made

- Kept the canonical stage-numbered root contract as the single source of truth for Scenario 1 instead of preserving suffix-named intermediary stage files.
- Treated failing storage and wallet source-shape tests as current-plan fallout and repaired those tests at the assertion layer.
- Accepted the pre-existing broader task-1 commit history and used a clean task-2 regression-test commit plus metadata closure rather than rewriting history.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed stale cross-crate source-shape guards after the canonical stage split**

- **Found during:** Task 2 release verification reruns
- **Issue:** `z00z_storage` and `z00z_wallets` tests still asserted ownership in old root files even though the real tx and bundle logic had moved into lane implementation files.
- **Fix:** Repointed the storage snapshot replay guard and wallet closure gate to `stage_4_utils/tx_lane_impl.rs` and `stage_6_utils/bundle_lane_impl.rs`, and aligned the simulator acceptance test wording to the same ownership model.
- **Files modified:** `crates/z00z_simulator/tests/test_claim_acceptance.rs`, `crates/z00z_storage/tests/snapshot/test_replay_bound.rs`, `crates/z00z_wallets/tests/test_s5_closure_gate.rs`
- **Verification:** `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_acceptance -- --nocapture`; `cargo test -p z00z_storage --release --features test-fast --test test_snapshot_suite test_stage_io_use_store -- --nocapture`; `cargo test -p z00z_wallets --release --features test-fast --test test_s5_closure_gate test_s5_track_map -- --nocapture`
- **Committed in:** `32180752`

**2. [Rule 1 - Spec Drift] Cleared review-pass drift in plan and summary docs after the canonical renumbering**

- **Found during:** Mandatory `/GSD-Review-Tasks-Execution` closure passes
- **Issue:** `021-02-PLAN.md` still had one stale `stage_4_claim_publish` success-criteria reference, and wave-01 docs still described suffix-named root files.
- **Fix:** Updated the active plan and the wave-01 plan/summary so they consistently describe the canonical stage-numbered Scenario 1 root surface.
- **Files modified:** `.planning/phases/021-refactor-continue/021-02-PLAN.md`, `.planning/phases/021-refactor-continue/021-01-PLAN.md`, `.planning/phases/021-refactor-continue/021-01-SUMMARY.md`
- **Verification:** Codacy CLI reported no supported analyzers for Markdown files; workspace diagnostics were clean; two consecutive review passes reported no blocking inconsistencies
- **Committed in:** pending in current execution slice

---

**Total deviations:** 2 auto-fixed (1 regression fallout, 1 planning drift)
**Impact on plan:** Both deviations were required to close release verification and review closure without widening simulator production scope.

## Issues Encountered

- The first post-cutover full release rerun exposed stale file-ownership assertions in `z00z_storage/tests/snapshot/test_replay_bound.rs`.
- The second full release rerun exposed a related stale ownership assertion in `z00z_wallets/tests/test_s5_closure_gate.rs`.
- Codacy analysis could not run on the edited Markdown planning files because no configured analyzer supports those file types in this workspace.

## Verification Evidence

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_acceptance -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_emit -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_integration -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_persist -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_snapshot -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_tx_pipeline -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump`
- `cargo test -p z00z_storage --release --features test-fast --test test_snapshot_suite test_stage_io_use_store -- --nocapture`
- `cargo test -p z00z_wallets --release --features test-fast --test test_s5_closure_gate test_s5_track_map -- --nocapture`
- `/GSD-Review-Tasks-Execution` review passes: one finding pass followed by two consecutive clean passes

## User Setup Required

None - no external setup required for this plan slice.

## Next Phase Readiness

- The claim lane is closed on the canonical root contract and no longer needs compatibility forwarding from `stage_3.rs`.
- Later waves can continue the same root-plus-utils pattern for tx, transfer, and bundle lanes without reintroducing suffix-named root files.
- `scenario_design.yaml` remains the remaining descriptive-contract gap reserved for later phase-021 closure work.

## Known Stubs

None.

## Self-Check

PASSED - summary file exists and recorded task commits `88b933a8` and `32180752` are present in git history.

---
*Phase: 021-refactor-continue*
*Completed: 2026-03-27*
