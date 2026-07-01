---
phase: 033-crypto-audit-scenario-2
plan: 17
subsystem: validator-publish-spend-caution-wave
tags: [validator-trust, publish-trustlessness, full-zk-spend, caution-wave, simulator, wallets]
requires:
  - phase: 033-16
    provides: documentation allowlist governance plus the prior narrow caution baseline for stealth, `s_out`, and publish-proof wording
provides:
  - active Phase 033 context now freezes the safe final reading for incomplete validator trust, JMT publish trustlessness, and full-ZK spend claims
  - checkpoint validator wording now states that final cryptographic closure is the missing piece rather than implying an absence of live validator-facing checks
  - spend-verifier wording now keeps the live public spend contract real while explicitly below a finished full-ZK spend theorem
  - stage 11 and stage 12 now keep publish integrity package-coupled and explicitly below fully trustless status
affects: [phase-033-caution-wave, validator-trust-wording, publish-trustlessness-wording, public-spend-boundary-language]
tech-stack:
  added: []
  patterns: [narrow wording guards, caution-answer freezing, honest theorem-boundary language]
key-files:
  created:
    - .planning/phases/033-crypto-audit-scenario-2/033-17-SUMMARY.md
  modified:
    - .planning/phases/033-crypto-audit-scenario-2/033-CONTEXT.md
    - .planning/STATE.md
    - .planning/ROADMAP.md
    - crates/z00z_wallets/src/core/tx/state_checkpoint.rs
    - crates/z00z_wallets/src/core/tx/spend_verification.rs
    - crates/z00z_wallets/src/core/tx/spend_rules.rs
    - crates/z00z_simulator/src/scenario_1/stage_11.rs
    - crates/z00z_simulator/src/scenario_1/stage_12.rs
    - crates/z00z_simulator/tests/test_scenario1_stage_surface.rs
key-decisions:
  - "Task 51 was frozen as a live-validator-but-not-final-closure answer, and the missing piece is described as final cryptographic closure rather than absence of validator-facing checks."
  - "Task 52 stays below full trustlessness by tying publish language to the package-coupled stage-11 and stage-12 integrity path only."
  - "Task 53 keeps the persisted public spend contract real while explicitly narrower than a finished full-ZK spend theorem."
patterns-established:
  - "When `/GSD-Review-Tasks-Execution` cannot be run directly, replace it with explicit diagnostics, semantic phrase scan, and focused reread passes; require two consecutive clean semantic passes before closeout."
requirements-completed: [PH32-SPEND, PH32-CHECKPOINT, PH32-HONEST]
duration: continued-session
completed: 2026-04-08
---

# Phase 033: Plan 17 Summary

**Phase 033 now freezes the validator-trust, publish-trustlessness, and full-ZK-spend caution answers as narrow repository-backed statements instead of allowing those live seams to drift into final-closure language.**

## Performance

- **Duration:** continued session
- **Started:** continued from prior execution context
- **Completed:** 2026-04-08T00:00:00Z
- **Tasks:** 3
- **Files modified:** 9

## Accomplishments

- Added a `Remaining Caution Answers` section to the active Phase 033 context for Tasks 51, 52, and 53.
- Pinned checkpoint validator wording to the exact safe final reading: final cryptographic closure remains missing, but live validator-facing verification is real.
- Pinned publish wording in stage 11 and stage 12 below full trustlessness while preserving package-coupled integrity language.
- Pinned spend wording so the live public spend contract stays real but remains narrower than a finished full-ZK spend theorem.
- Added two new surface guards to `test_scenario1_stage_surface.rs` to lock the Task 51-53 caution phrases into the repository surface.

## Task Commits

- Plan 17 implementation and closeout were prepared as one scoped change set because the three caution answers share one guard-test surface and one repo-owned commit flow.
- Final commit hash is recorded in the phase execution report and git history rather than duplicated here pre-commit.

## Files Created/Modified

- `.planning/phases/033-crypto-audit-scenario-2/033-CONTEXT.md` - adds the explicit Task 51-53 safe final readings.
- `crates/z00z_wallets/src/core/tx/state_checkpoint.rs` - narrows validator wording to missing final cryptographic closure rather than absent validator-facing checks.
- `crates/z00z_wallets/src/core/tx/spend_verification.rs` - states that the live contract remains narrower than a finished full-ZK spend theorem.
- `crates/z00z_wallets/src/core/tx/spend_rules.rs` - mirrors the live-but-narrower public spend contract wording.
- `crates/z00z_simulator/src/scenario_1/stage_11.rs` - states publish is not yet strong enough to be called fully trustless.
- `crates/z00z_simulator/src/scenario_1/stage_12.rs` - keeps finalize-path publish language below full trustlessness.
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` - adds exact wording guards for Tasks 51-53.
- `.planning/STATE.md` - advances Phase 033 state after Plan 17 closeout.
- `.planning/ROADMAP.md` - records Plan 17 as summary-backed and advances the phase execution counters.

## Decisions Made

- Treated Task 51 as a validator-boundary clarification, not as evidence that final validator trustlessness exists.
- Kept Task 52 anchored to stage-11/stage-12 package-coupled integrity rather than a stronger publish-proof theorem.
- Kept Task 53 explicitly tied to the current persisted public spend boundary and the open nullifier-semantics gap.

## Deviations from Plan

- The environment did not expose a direct runner for `/GSD-Review-Tasks-Execution`, so the required review loop was completed manually with diagnostics, semantic phrase scan, and focused reread passes.
- The plan is closed with one scoped change set because Tasks 51-53 share one guard-test file and the repository-owned commit workflow would otherwise require multiple version bumps on a dirty worktree.

## Issues Encountered

- The broader `cargo test --release --features test-fast --features wallet_debug_dump` rerun hit a pre-existing `z00z_crypto/tari` doctest blocker caused by multiple `tari_utilities` versions in the vendored dependency graph. This sits in the read-only vendor boundary and is outside Plan 17 scope.
- Diagnostics still report pre-existing file-size and complexity warnings in `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` and `crates/z00z_simulator/src/scenario_1/stage_12.rs`; these were not introduced by Plan 17.

## Deferred Issues

- The read-only Tari vendor doctest and dependency-graph failure remains a broader verification blocker for workspace-wide release runs.
- Existing unrelated worktree changes outside the Plan 17 files were intentionally left untouched.

## User Setup Required

None.

## Next Phase Readiness

- Plan 17 now freezes the next caution-answer cluster needed before the concrete fix-set plans.
- Phase 033 can continue into Plan 18 using this summary as the baseline for genesis membership, checkpoint placeholder boundary, and receiver identity-binding wording.
- The bootstrap gate is green and the exact Plan 17 wording guards are green.

## Threat Flags

None.

## Known Stubs

None.

## Self-Check

PASSED

- FOUND: `.planning/phases/033-crypto-audit-scenario-2/033-17-SUMMARY.md`

---
*Phase: 033-crypto-audit-scenario-2*
*Completed: 2026-04-08*
