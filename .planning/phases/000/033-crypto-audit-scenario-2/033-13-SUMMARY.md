---
phase: 033-crypto-audit-scenario-2
plan: 13
subsystem: wallets
tags: [wallet, semantics, spend-boundary, checkpoint, accepted-path]
requires:
  - phase: 033-12
    provides: wallet-local ownership and route-boundary wording that Plan 13 keeps narrow through spend and checkpoint acceptance seams
provides:
  - post-scan exclusivity stays wallet-local and receiver-secret plus s_out gated
  - delivered persisted public spend contract stays narrower than full PH32-SPEND closure
  - structurally plausible stage4 artifacts stay weaker than semantic acceptance before state mutation
affects: [phase-033-late-wallet-slices, spend-verification, spend-rules, checkpoint-acceptance]
tech-stack:
  added: []
  patterns: [tdd wording guards, semantic fragment assertions, isolated cargo target validation]
key-files:
  created:
    - .planning/phases/033-crypto-audit-scenario-2/033-13-SUMMARY.md
  modified:
    - .planning/REQUIREMENTS.md
    - crates/z00z_wallets/src/services/wallet_service_actions_receive.rs
    - crates/z00z_wallets/src/core/tx/spend_verification.rs
    - crates/z00z_wallets/src/core/tx/spend_rules.rs
    - crates/z00z_simulator/src/scenario_1/stage_4.rs
    - crates/z00z_wallets/tests/test_scenario1_semantics.rs
    - crates/z00z_simulator/tests/test_checkpoint_acceptance.rs
    - crates/z00z_wallets/tests/test_e2e_req_flow.rs
    - crates/z00z_wallets/tests/test_asset_ownership_security.rs
    - crates/z00z_simulator/tests/test_scenario1_stage_surface.rs
    - crates/z00z_wallets/tests/test_s5_closure_gate.rs
key-decisions:
  - "Kept post-scan exclusivity wallet-local and receiver-secret plus s_out gated instead of widening it into a public theorem."
  - "Described the live spend boundary as the delivered persisted public spend contract and kept nullifier and authoritative-membership closure outside that boundary."
  - "Made semantic acceptance explicitly stronger than stage4 structural plausibility before any checkpoint or state mutation."
patterns-established:
  - "When semantic boundary wording is source-guarded, tests should assert stable fragments instead of exact wrapped prose."
  - "If a plan points at a stale test surface, redirect the task to the live canonical acceptance surface and record the deviation explicitly."
requirements-completed: [PH32-SPEND, PH32-HONEST]
duration: continued-session
completed: 2026-04-08
---

# Phase 033: Plan 13 Summary

**Wallet-local exclusivity, the exact delivered public spend boundary, and semantic-acceptance-above-structure now stay narrow and directly test-guarded across the live wallet and simulator seams.**

## Performance

- **Duration:** continued session
- **Started:** continued from prior execution context
- **Completed:** 2026-04-08T00:00:00Z
- **Tasks:** 3
- **Files modified:** 11

## Accomplishments

- Kept post-scan exclusivity explicitly wallet-local and tied to receiver-secret plus `s_out` accepted-path behavior.
- Scoped the live spend boundary to the delivered persisted public spend contract instead of implying full PH32-SPEND closure.
- Made stage4 structural artifacts explicitly non-authoritative until later semantic acceptance succeeds.
- Added direct negative acceptance coverage on the live checkpoint acceptance surface after the plan's listed wallet test surface proved stale.
- Repaired all in-scope wording-guard fallout exposed by the release-style verification loop.
- Revalidated the touched surfaces with targeted reruns and a fresh bootstrap pass on an isolated cargo target directory.

## Task Commits

1. **Task 37: Exclusivity After Scan**
   - `392d222d` `test(033-13): add failing post-scan exclusivity guard`
   - `dae2abd0` `feat(033-13): keep post-scan exclusivity wallet-local`
2. **Task 38: What The Spend Boundary Actually Proves**
   - `652b8ad7` `test(033-13): add failing public spend boundary guard`
   - `eb542a5d` `feat(033-13): scope the delivered public spend boundary`
3. **Task 39: Structural Plausibility Versus Semantic Acceptance**
   - `7d6da8cd` `test(033-13): add failing semantic acceptance guard`
   - `1caa7923` `feat(033-13): keep semantic acceptance above structural artifacts`

## Fallout-Fix Commits

- `2d261263` `fix(033-13): stabilize request-flow wording guards`
- `aa48c1c2` `fix(033-13): stabilize ownership boundary guards`
- `65382e03` `fix(033-13): stabilize simulator public spend boundary guard`
- `a98a61cd` `fix(033-13): stabilize nullifier gap wording guard`
- `6b9b00f8` `fix(033-13): stabilize S5 closure spend wording guard`

**Plan metadata:** pending metadata commit after state updates

## Files Created/Modified

- `.planning/REQUIREMENTS.md` - narrowed `PH32-SPEND` wording to the delivered persisted public spend contract.
- `crates/z00z_wallets/src/services/wallet_service_actions_receive.rs` - keeps post-scan exclusivity wallet-local and request-lane semantics distinct from compatibility lanes.
- `crates/z00z_wallets/src/core/tx/spend_verification.rs` - freezes the public spend boundary and states that semantic acceptance is stronger than structural plausibility.
- `crates/z00z_wallets/src/core/tx/spend_rules.rs` - keeps structural spend rules scoped to the delivered persisted contract and stabilizes its internal wording guard.
- `crates/z00z_simulator/src/scenario_1/stage_4.rs` - marks stage4 publish artifacts as structurally useful but non-authoritative before later semantic gates.
- `crates/z00z_wallets/tests/test_scenario1_semantics.rs` - adds direct source guards for wallet-local exclusivity and the narrowed spend boundary.
- `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs` - adds direct negative acceptance coverage proving stage4 artifacts remain weaker than semantic acceptance.
- `crates/z00z_wallets/tests/test_e2e_req_flow.rs` - stabilizes request-lane wording guards after the source wording tightened.
- `crates/z00z_wallets/tests/test_asset_ownership_security.rs` - stabilizes ownership-language guards around wallet-local exclusivity.
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` - stabilizes the simulator public-spend boundary guard against newline-sensitive wording drift.
- `crates/z00z_wallets/tests/test_s5_closure_gate.rs` - stabilizes the S5 closure guard around the still-open nullifier caveat.

## Decisions Made

- Kept receiver-secret plus `s_out` exclusivity as wallet-local accepted-path behavior instead of promoting it into a public theorem.
- Treated the delivered persisted public spend contract as the exact live boundary and kept nullifier replay semantics plus authoritative input-membership continuity explicitly outside it.
- Bound stage4 structural persistence to fail-closed later semantic gates instead of implying checkpoint or accepted-tx authority from early files.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking issue] Redirected Task 39 to the live checkpoint acceptance surface**

- **Found during:** Task 39
- **Issue:** The plan referenced `crates/z00z_wallets/tests/test_state_checkpoint.rs`, but that file no longer exists in the repository.
- **Fix:** Landed the semantic-acceptance negative coverage on the live canonical surface `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs` instead.
- **Files modified:** `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs`
- **Verification:** Targeted checkpoint acceptance rerun passed and later bootstrap validation passed.
- **Committed in:** `7d6da8cd`, `1caa7923`

**2. [Rule 1 - Bug] Fixed request-flow wording guard fallout**

- **Found during:** post-task release-style verification
- **Issue:** Tightened source wording broke brittle request/card route assertions.
- **Fix:** Replaced stale exact-literal checks with stable semantic fragment assertions.
- **Files modified:** `crates/z00z_wallets/tests/test_e2e_req_flow.rs`
- **Verification:** Targeted rerun passed.
- **Committed in:** `2d261263`

**3. [Rule 1 - Bug] Fixed wallet-local ownership wording guard fallout**

- **Found during:** post-task release-style verification
- **Issue:** Tightened wallet-local exclusivity wording broke ownership-boundary guards.
- **Fix:** Reframed the tests around stable wallet-local semantic fragments.
- **Files modified:** `crates/z00z_wallets/tests/test_asset_ownership_security.rs`
- **Verification:** Targeted rerun passed.
- **Committed in:** `aa48c1c2`

**4. [Rule 1 - Bug] Fixed simulator and spend-rule wording fallout on the narrowed boundary**

- **Found during:** post-task release-style verification
- **Issue:** Simulator stage-surface and internal spend-rule tests still asserted stale exact public-boundary phrasing.
- **Fix:** Converted those guards to stable fragment checks aligned with the new delivered-boundary language.
- **Files modified:** `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`, `crates/z00z_wallets/src/core/tx/spend_rules.rs`
- **Verification:** Targeted reruns passed.
- **Committed in:** `65382e03`, `a98a61cd`

**5. [Rule 1 - Bug] Fixed S5 closure wording fallout around the nullifier caveat**

- **Found during:** final release-style verification pass
- **Issue:** The S5 closure gate still required an older exact literal for the nullifier caveat.
- **Fix:** Updated the test to check stable fragments covering the delivered boundary and still-open nullifier semantics.
- **Files modified:** `crates/z00z_wallets/tests/test_s5_closure_gate.rs`
- **Verification:** Targeted rerun passed and a fresh bootstrap pass completed successfully.
- **Committed in:** `6b9b00f8`

---

**Total deviations:** 5 auto-fixed (4 bug fixes, 1 blocking stale-path redirect)
**Impact on plan:** All deviations were narrow, in-scope, and required to preserve correctness of the delivered semantic-boundary claims.

## Issues Encountered

- The plan's listed Task 39 test file had drifted out of the live tree and had to be redirected to the current canonical checkpoint acceptance surface.
- A broad workspace release-style rerun continued past the Plan 13 fallout into a historically documented unrelated vendor/doctest failure class under `crates/z00z_crypto/tari/crypto/`.
- Prompt-based `/GSD-Review-Tasks-Execution` automation is defined in repository prompts, but this executor session did not expose a separate prompt-runner tool for invoking it as an isolated automation step.

## Threat Flags

None.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 13 code and in-scope semantic-boundary validation are ready for the next nullifier and checkpoint-backend slices.
- The remaining full-workspace release-style failure class is recorded separately as unrelated vendor/doctest fallout and does not come from the Plan 13 touched surfaces.

## Self-Check

PASSED

- FOUND: `.planning/phases/033-crypto-audit-scenario-2/033-13-SUMMARY.md`
- FOUND: `1caa7923`
- FOUND: `6b9b00f8`

---
*Phase: 033-crypto-audit-scenario-2*
*Completed: 2026-04-08*
