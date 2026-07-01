---
phase: 033-crypto-audit-scenario-2
plan: 12
subsystem: wallets
tags: [wallet, semantics, leaf_ad_id, request-routing, accepted-path]
requires:
  - phase: 033-11
    provides: claim-trust clarification surfaces that Plan 12 keeps narrow at wallet-local and route-binding seams
provides:
  - receiver-secret ownership wording stays wallet-local at public spend seams
  - leaf_ad_id wording stays limited to shipped wallet, scan, report, and spend-witness bridge paths
  - request-bound and card-bound receive wording stays explicitly distinct
affects: [phase-033-late-wallet-slices, spend-witness, checkpoint, route-semantics]
tech-stack:
  added: []
  patterns: [raw-source wording guards for semantic boundaries, isolated cargo target validation]
key-files:
  created: [.planning/phases/033-crypto-audit-scenario-2/033-12-SUMMARY.md]
  modified:
    - crates/z00z_wallets/src/core/tx/spend_verification.rs
    - crates/z00z_wallets/src/core/tx/witness_gate.rs
    - crates/z00z_wallets/src/services/wallet_service.rs
    - crates/z00z_wallets/src/services/wallet_service_actions_receive.rs
    - crates/z00z_wallets/tests/test_asset_ownership_security.rs
    - crates/z00z_wallets/tests/test_e2e_req_flow.rs
    - crates/z00z_wallets/tests/test_scenario1_semantics.rs
key-decisions:
  - "Kept receiver-secret ownership wording wallet-local instead of widening public spend verification into a public theorem."
  - "Kept leaf_ad_id accepted-path claims limited to shipped wallet, scan, report, and spend-witness bridge paths."
  - "Kept request-bound receive as the preferred privacy lane and rejected card-path equivalence wording."
patterns-established:
  - "Semantic boundary comments that are guarded by raw-source tests must preserve exact contiguous phrasing."
  - "When shared cargo target locking interferes with validation, use an isolated CARGO_TARGET_DIR instead of disturbing other work."
requirements-completed: [PH32-SEM, PH32-HONEST]
duration: continued-session
completed: 2026-04-07
---

# Phase 033: Plan 12 Summary

**Wallet-local ownership, accepted-path leaf_ad_id scope, and request-vs-card receive wording now stay narrow and test-guarded across the shipped wallet seams.**

## Performance

- **Duration:** continued session
- **Started:** continued from prior execution context
- **Completed:** 2026-04-07T20:48:17Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- Kept receiver-secret ownership semantics explicitly wallet-local at the public spend boundary.
- Kept leaf_ad_id wording scoped to shipped wallet, scan, report, and spend-witness bridge paths instead of repository-wide closure.
- Kept request-bound receive wording distinct from plain or card-bound compatibility lanes and added route-wording guards.
- Revalidated the affected wallet tests, bootstrap gate, and full release-style cargo test on an isolated target directory.
- Completed three narrow review passes against the exact Plan 12 diff using crypto, security, and doublecheck criteria; no substantive in-scope findings remained after the wording fixes below.

## Task Commits

Plan 12 landed as one tightly coupled semantic-boundary code commit because the affected wording guards span shared source and test seams:

1. **Task 34: Receiver-Held Secret As The Ownership Gate** - `9bda37fc` (fix, shared Plan 12 semantic-boundary commit)
2. **Task 35: Canonical Decrypt-Associated Asset Binding** - `9bda37fc` (fix, shared Plan 12 semantic-boundary commit)
3. **Task 36: Request-Bound Route Versus Card-Bound Route** - `9bda37fc` (fix, shared Plan 12 semantic-boundary commit)

**Plan metadata:** pending metadata commit after state updates

## Files Created/Modified

- `crates/z00z_wallets/src/core/tx/spend_verification.rs` - narrows public spend wording so wallet-local ownership and accepted-path leaf_ad_id claims stay separate.
- `crates/z00z_wallets/src/core/tx/witness_gate.rs` - keeps the decrypt/state split scoped to shipped bridge paths.
- `crates/z00z_wallets/src/services/wallet_service.rs` - states request-aware receive as the preferred privacy lane and card/plain paths as compatibility-only.
- `crates/z00z_wallets/src/services/wallet_service_actions_receive.rs` - keeps recv_one, recv_range, and receive_asset wording distinct across request-bound and card-bound routes.
- `crates/z00z_wallets/tests/test_asset_ownership_security.rs` - keeps ownership-language guards aligned to wallet-local semantics.
- `crates/z00z_wallets/tests/test_e2e_req_flow.rs` - adds route-wording guards for request-vs-card semantics.
- `crates/z00z_wallets/tests/test_scenario1_semantics.rs` - keeps Scenario 1 semantics wording aligned with wallet-local ownership and accepted-path leaf_ad_id scope.

## Decisions Made

- Continued to treat receiver-secret ownership as a wallet-local accepted-path invariant until any future public verifier proves the same theorem end to end.
- Treated leaf_ad_id as canonical only on the already shipped wallet, scan, report, and spend-witness bridge surfaces, not as repository-wide closure.
- Preserved request-bound receive preference without upgrading card-path compatibility into an equivalent privacy theorem.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed raw-source ownership-theorem guard regression**

- **Found during:** Task 34 (Receiver-Held Secret As The Ownership Gate)
- **Issue:** The spend verification wording was line-wrapped so the exact wallet-local ownership theorem phrase was no longer visible to the raw-source guard.
- **Fix:** Restored contiguous wording in `spend_verification.rs` and aligned the Scenario 1 semantics comment to the same wallet-local ownership phrasing.
- **Files modified:** `crates/z00z_wallets/src/core/tx/spend_verification.rs`, `crates/z00z_wallets/tests/test_scenario1_semantics.rs`
- **Verification:** Targeted wallet tests passed, bootstrap gate passed, full release-style cargo test passed.
- **Committed in:** `9bda37fc`

**2. [Rule 1 - Bug] Fixed request-aware receive wording guard regression**

- **Found during:** Task 36 (Request-Bound Route Versus Card-Bound Route)
- **Issue:** The exact `preferred request-aware receive lane` phrase was split across lines, so the route-distinction raw-source guard no longer matched.
- **Fix:** Restored contiguous wording in `wallet_service_actions_receive.rs` and reran the route-focused tests.
- **Files modified:** `crates/z00z_wallets/src/services/wallet_service_actions_receive.rs`, `crates/z00z_wallets/tests/test_e2e_req_flow.rs`
- **Verification:** Targeted wallet tests passed, bootstrap gate passed, full release-style cargo test passed.
- **Committed in:** `9bda37fc`

---

**Total deviations:** 2 auto-fixed (2 bug fixes)
**Impact on plan:** Both fixes were narrow, required for correctness of the existing semantic guards, and did not expand scope beyond Plan 12.

## Issues Encountered

- Shared cargo target locking made the first validation loop unreliable; validation was moved to `CARGO_TARGET_DIR=target/phase033-plan12` to isolate the run.
- The repository pre-commit hook required `cargo fmt --all` before accepting the Plan 12 commit.

## Threat Flags

None.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 12 is ready for the next late-phase wallet slices with the semantic boundaries now locked behind raw-source wording guards and passing validation.
- No Plan 12 blocker remains open.

## Self-Check

PASSED

- FOUND: `.planning/phases/033-crypto-audit-scenario-2/033-12-SUMMARY.md`
- FOUND: `9bda37fc`

---
*Phase: 033-crypto-audit-scenario-2*
*Completed: 2026-04-07*
