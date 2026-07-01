---
phase: 033-crypto-audit-scenario-2
plan: 5
subsystem: testing
tags: [scenario-1, wording-guards, checkpoint, spend-boundary, rehydrate, simulator]
requires:
  - phase: 033-04
    provides: truthful spend-boundary, theft-window, and package-coupled checkpoint wording used as the immediate baseline for Tasks 13-15
provides:
  - explicit nullifier-gap-only PH32-SPEND status freeze
  - synchronized delivered or partial or not-proved whole-chain bucket wording
  - accepted-path-only placeholder checkpoint-closure wording guard
affects: [033-06, 033-08, 033-09, PH32-SPEND, PH32-CHECKPOINT, PH32-HONEST]
tech-stack:
  added: []
  patterns: [source-shape wording guards, accepted-path-only checkpoint closure language, atomic version-manager task commits]
key-files:
  created: [.planning/phases/033-crypto-audit-scenario-2/033-05-SUMMARY.md]
  modified:
    - .planning/REQUIREMENTS.md
    - .planning/phases/033-crypto-audit-scenario-2/033-CONTEXT.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - crates/z00z_wallets/src/core/tx/spend_verification.rs
    - crates/z00z_wallets/src/core/tx/spend_rules.rs
    - crates/z00z_wallets/tests/test_s5_closure_gate.rs
    - crates/z00z_simulator/src/scenario_1/scenario_design.yaml
    - crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs
    - crates/z00z_simulator/src/scenario_1/stage_12.rs
    - crates/z00z_simulator/tests/test_scenario1_stage_surface.rs
    - crates/z00z_storage/tests/test_redb_rehydrate.rs
    - versions.yaml
key-decisions:
  - "Keep nullifier semantics as the exact still-open PH32-SPEND element instead of widening missingness to the already-live proof/auth boundary."
  - "Represent the whole-chain answer as delivered, partial, and not-proved buckets across active planning and design artifacts."
  - "Describe placeholder checkpoint closure only on the accepted current-stack path, never as recursive-proof authority or broader PH32-SPEND completion."
patterns-established:
  - "Narrative-bucket pattern: whole-chain security claims must be synchronized across context, roadmap, state, and design surfaces before later reclassification work."
  - "Accepted-path closure pattern: checkpoint placeholder-lane fixes may prove current-stack fail-closed behavior without upgrading the result into standalone authority."
requirements-completed: [PH32-SPEND, PH32-CHECKPOINT, PH32-HONEST]
duration: 34 min
completed: 2026-04-07
---

# Phase 033: Plan 05 Summary

## Outcome

Plan 05 now freezes the single open spend-side nullifier gap, the layered whole-chain bucket model, and the accepted-path-only checkpoint placeholder closure as explicit regression-tested repository truth.

## Performance

- **Duration:** 34 min
- **Started:** 2026-04-07T05:45:42Z
- **Completed:** 2026-04-07T06:19:21Z
- **Tasks:** 3
- **Files modified:** 13

## Accomplishments

- Task 13 reclassified `PH32-SPEND` honestly so nullifier semantics remain the exact still-open public-contract element while the current proof/auth boundary stays acknowledged as already live.
- Task 14 synchronized the delivered or partial or not-proved whole-chain answer across the active context, roadmap, state, design surface, and closure-gate regression coverage.
- Task 15 locked checkpoint placeholder-proof and spent-state closure to the accepted current-stack path only and denied any upgrade into recursive-proof authority or broader `PH32-SPEND` completion.

## Task Commits

Each task was committed atomically:

1. **Task 13: The Requirement That Remains Open** - `759802fc` (feat)
2. **Task 14: Full-Chain Crypto Closure Versus Partial Security** - `a2c7248b` (feat)
3. **Task 15: Placeholder Success Paths Truly Closed** - `30f3b5e2` (feat)

**Plan metadata:** pending

## Files Created/Modified

- `.planning/phases/033-crypto-audit-scenario-2/033-05-SUMMARY.md` - Plan 05 closeout summary and evidence record.
- `.planning/REQUIREMENTS.md` - Keeps `PH32-SPEND` explicitly open on nullifier semantics only.
- `.planning/phases/033-crypto-audit-scenario-2/033-CONTEXT.md` - Adds the canonical layered whole-chain bucket wording.
- `.planning/ROADMAP.md` - Mirrors the layered whole-chain bucket wording at the active phase status surface.
- `.planning/STATE.md` - Records the layered whole-chain bucket wording in active phase decisions.
- `crates/z00z_wallets/src/core/tx/spend_verification.rs` - Narrows the public spend contract wording to the exact nullifier-gap seam.
- `crates/z00z_wallets/src/core/tx/spend_rules.rs` - Documents the delivered spend-rule boundary and adds the Task 13 wording guard.
- `crates/z00z_wallets/tests/test_s5_closure_gate.rs` - Adds the Task 14 whole-chain bucket regression guard.
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml` - Carries the layered whole-chain bucket note as descriptive surface comments.
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs` - States accepted-path-only placeholder closure and denies recursive-proof or broader spend closure.
- `crates/z00z_simulator/src/scenario_1/stage_12.rs` - States finalization remains accepted-path-only and not standalone checkpoint authority.
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` - Aligns the spend-boundary wording guard with the narrowed Task 13 semantics.
- `crates/z00z_storage/tests/test_redb_rehydrate.rs` - Adds the Task 15 accepted-path placeholder-closure regression guard and ties it to persisted proof-surface failures.
- `versions.yaml` - Repository-owned version bumps for Task 13 through Task 15 commits.

## Decisions Made

- Used wording and source-shape guards again because Plan 05 freezes truth surfaces and accepted-path scope rather than landing new proof machinery.
- Preserved `PH32-SPEND` as open instead of force-closing it through documentation drift, because the current regular spend statement still lacks nullifier semantics.
- Scoped checkpoint placeholder closure to accepted current-stack paths only, using existing rehydrate and checkpoint-acceptance failures as the proof surface rather than expanding the claim into recursive-proof or backend-authority language.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated simulator wording guard to the narrowed Task 13 spend-boundary semantics**

- **Found during:** Task 14 (Full-Chain Crypto Closure Versus Partial Security)
- **Issue:** `test_public_spend_boundary_wording_stays_narrow` still expected the pre-Task-13 literal sentence and failed once the spend verifier started describing the nullifier gap as the exact still-open element with an already-live proof/auth boundary.
- **Fix:** Rewrote the simulator wording guard to check the new narrowed semantics as stable fragments instead of the superseded literal phrase.
- **Files modified:** `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- **Verification:** `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture`
- **Committed in:** `a2c7248b`

**2. [Rule 3 - Blocking] Applied the rustfmt-required wrap in the Task 15 regression guard before commit**

- **Found during:** Task 15 (Placeholder Success Paths Truly Closed)
- **Issue:** The first version-manager run stopped at `cargo fmt --all --check` because `test_redb_rehydrate.rs` needed one boolean assertion wrapped to match repository formatting.
- **Fix:** Applied the rustfmt-required one-line wrapping and re-ran the targeted Task 15 guard before retrying the repository-owned commit flow.
- **Files modified:** `crates/z00z_storage/tests/test_redb_rehydrate.rs`
- **Verification:** `cargo test -p z00z_storage --release --features test-fast --test test_redb_rehydrate test_task15_accepted_path_placeholder_closure_stays_narrow -- --nocapture`
- **Committed in:** `30f3b5e2`

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** Both fixes were narrow correctness or commitability fixes inside the planned surfaces. No scope creep and no widened security claims.

## Issues Encountered

- The first Task 14 RED attempt accidentally targeted the wrong Cargo surface and filtered out the new integration test; rerunning with `--test test_s5_closure_gate` produced the expected failing state.
- Both Task 14 and the simulator fallout check needed fragment-based assertions instead of full literal wrapped lines, otherwise line wrapping would have produced false negatives unrelated to semantics.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 06 can now build on a stable truthful baseline: `PH32-SPEND` is still explicitly open only on nullifier semantics, the whole-chain answer is bucketed, and checkpoint placeholder closure is limited to accepted-path current-stack evidence.
- Later reclassification plans must land real new semantics before upgrading any of these boundaries.
- No blocker remains inside Plan 05 itself.

## Self-Check

PASSED

- FOUND: `.planning/phases/033-crypto-audit-scenario-2/033-05-SUMMARY.md`
- FOUND: `759802fc`
- FOUND: `a2c7248b`
- FOUND: `30f3b5e2`

---
*Phase: 033-crypto-audit-scenario-2*
*Completed: 2026-04-07*
