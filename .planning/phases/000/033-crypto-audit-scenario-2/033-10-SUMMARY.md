---
phase: 033-crypto-audit-scenario-2
plan: 10
subsystem: testing
tags: [claim, simulator, wallets, storage, requirements]
requires:
  - phase: 033-09
    provides: governance truth, honest reclassification gate, active-artifact authority
provides:
  - explicit full-tuple claim signature mutation coverage
  - plausible package drift rejection coverage for recipient-binding and source-asset-path variants
  - narrowed helper-owned authority wording across storage, simulator, and requirements surfaces
affects: [PH32-CLAIM-BIND, PH32-CLAIM-TRUST, scenario-1-claim-consumer]
tech-stack:
  added: []
  patterns: [direct mutation matrices for signed tuples, plausible-package drift regression tests, helper-owned authority wording]
key-files:
  created: []
  modified:
    - crates/z00z_crypto/tests/test_claim_v2_contract.rs
    - crates/z00z_wallets/src/core/tx/test_claim_tx.rs
    - crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs
    - crates/z00z_simulator/src/claim_pkg_consumer.rs
    - crates/z00z_storage/src/assets/store_internal/store_query.rs
    - .planning/REQUIREMENTS.md
key-decisions:
  - "Treat Task 28 as a coverage-closure task because the full tuple was already bound in code; add explicit mutation evidence instead of widening production logic."
  - "Use plausible recipient-binding rebinding and real source-asset-id drift at the simulator consumer seam to close Task 29 without inventing malformed-input cases."
  - "Narrow all remaining helper-vs-authority wording to the helper-owned canonical contract instead of claiming persisted storage-backed continuity."
patterns-established:
  - "Claim tuple assurance: prove signed-field coverage with direct mutation tests at both contract and live verifier seams."
  - "Claim-trust wording: describe helper-derived continuity explicitly and keep persisted storage-backed continuity as a separate, still-stronger boundary."
requirements-completed: [PH32-CLAIM-BIND, PH32-CLAIM-TRUST]
duration: 35min
completed: 2026-04-07
---

# Phase 033 Plan 10: Claim Tuple Drift Coverage Summary

Full claim-tuple signature drift coverage, plausible package-shape rejection tests, and helper-owned authority wording aligned to live Scenario 1 claim seams.

## Performance

- **Duration:** 35 min
- **Started:** 2026-04-07T12:29:26Z
- **Completed:** 2026-04-07T13:03:59Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments

- Added an explicit full-tuple mutation matrix for `ClaimStmtV2` so the authority signature is now directly regression-tested across every signed field and output-leaf shape change.
- Added plausible package-shape drift tests at the live simulator consumer seam for recipient-binding rebinding and source-asset-path drift.
- Removed lingering “storage authoritative” overclaim language from the helper seam and tied `PH32-CLAIM-TRUST` explicitly to the helper-owned canonical contract rather than persisted continuity.

## Task Commits

Each task was committed atomically:

1. **Task 28: Full Authenticated Claim Tuple** - `6584ed08` (feat)
2. **Task 29: Tuple Drift Under Plausible Package Shape** - `cdc8677f` (feat)
3. **Task 30: Self-Consistency Versus Authority** - `e61da19f` (feat)

**Plan metadata:** summary, state, and roadmap now agree that Plan 10 is complete and Plan 11 is next.

## Files Created/Modified

- `crates/z00z_crypto/tests/test_claim_v2_contract.rs` - expanded the claim-v2 signature test into a direct mutation matrix for the full authenticated tuple.
- `crates/z00z_wallets/src/core/tx/test_claim_tx.rs` - added a live verifier test showing rebinding `chain_id`, nullifier, and scope still breaks the stale authority tuple.
- `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs` - added recipient-binding and real source-asset-path drift tests plus narrowed helper-seam wording expectations.
- `crates/z00z_simulator/src/claim_pkg_consumer.rs` - renamed helper seam mismatch messages away from storage-authoritative wording.
- `crates/z00z_storage/src/assets/store_internal/store_query.rs` - documented that the helper contract must not be described as persisted storage authority.
- `.planning/REQUIREMENTS.md` - tied `PH32-CLAIM-TRUST` explicitly to `AssetStore::claim_source_contract_for_item(...)` and the helper-owned canonical boundary.

## Decisions Made

- Task 28 stayed test-only because the implementation already signed the full tuple; the missing artifact was explicit field-by-field evidence.
- Task 29 used real package components from alternate generated claim packages so the drift cases stayed structurally plausible instead of degrading into malformed-input tests.
- Task 30 closed by narrowing language and linkage rather than widening implementation claims beyond the helper-owned boundary the code actually proves.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Cleared stale verification processes that held the build lock**

- **Found during:** Task 28 and Task 30 verification
- **Issue:** Earlier bootstrap and release-test processes continued holding or waiting on the Cargo target lock, causing later verify commands to stall at startup.
- **Fix:** Identified and terminated stale bootstrap, release-test, and version-manager wrapper processes after the relevant local commits had already landed.
- **Files modified:** None
- **Verification:** Subsequent targeted cargo tests and workspace-wide release validation completed with exit code 0.
- **Committed in:** task-local verification flow; no source changes required

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** No scope creep. The deviation only removed stale verification processes so planned tests could run to completion.

## Issues Encountered

- Task 28 was a TDD coverage-closure case rather than a behavior bug: the new direct mutation tests went green immediately because the tuple binding was already correct in code.
- The repository-owned version-manager flow repeatedly kept background push and verification wrappers alive after the local commit had landed, so later task verification had to clear those stale processes to avoid target-lock stalls.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 10 now leaves the claim tuple, plausible package drift, and helper-owned authority seam explicit and regression-tested.
- Phase 033 is ready to advance to `033-11-PLAN.md`, which continues the narrower `PH32-CLAIM-TRUST` seam and remaining claim reject-path work.

## Self-Check: PASSED

- Verified `033-10-SUMMARY.md` exists in `.planning/phases/033-crypto-audit-scenario-2/`.
- Verified task commits `6584ed08`, `cdc8677f`, and `e61da19f` exist in local git history.
- Verified `.planning/STATE.md` and `.planning/ROADMAP.md` both point to Plan 11 as the next action after Plan 10 completion.

---
*Phase: 033-crypto-audit-scenario-2*
*Completed: 2026-04-07*
