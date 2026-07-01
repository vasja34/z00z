---
phase: 029-crypto-audit-wallets
plan: "02"
subsystem: wallets
tags: [wallets, view-key, rotation, regression, receiver-keys]
requires:
  - phase: 029-crypto-audit-wallets
    provides: 029-RECONCILIATION.md scope freeze and PH29 view-key target inventory
provides:
  - live-only wallet receiver-key retrieval contract at the service layer
  - explicit view-version retrieval surface for historical rotation state
  - regression anchors that block rotated helper re-entry into live scan and spend paths
affects: [029-03, 029-04, 029-05, 029-06]
tech-stack:
  added: []
  patterns: [live-default key retrieval, explicit historical view-version lookup, hot-path helper source guards]
key-files:
  created:
    - .planning/phases/029-crypto-audit-wallets/029-02-PLAN.md
    - crates/z00z_wallets/tests/test_view_key_contract.rs
  modified:
    - .planning/phases/029-crypto-audit-wallets/029-RECONCILIATION.md
    - crates/z00z_wallets/src/core/key/mod.rs
    - crates/z00z_wallets/src/core/key/stealth_keys.rs
    - crates/z00z_wallets/src/services/wallet_service.rs
    - crates/z00z_wallets/tests/test_e2e_send_scan.rs
    - crates/z00z_wallets/tests/test_rpc_key_derive_e2e.rs
key-decisions:
  - "Treat WalletService::receiver_keys(...) as the canonical live-only retrieval surface, even after persisted rotation metadata changes."
  - "Expose historical wallet rotation state only through explicit view-version lookup instead of default service retrieval."
  - "Keep rotated helper confinement enforceable through both source guards and negative spend-path regression coverage."
patterns-established:
  - "Live default, explicit history: default wallet flows derive live receiver keys and versioned recovery requires an explicitly named API."
  - "Contract anchors: send-scan, RPC derivation, and service tests each pin one aspect of the same live-versus-historical rule."
requirements-completed: [PH29-VIEWKEY]
duration: checkpointed
completed: 2026-03-30
---

# Phase 029 Plan 02: Live View-Key Contract Summary

Live-only wallet receiver-key retrieval with explicit view-version recovery and regression guards against rotated helper drift.

## Performance

- **Duration:** checkpointed
- **Started:** not recorded in resumed session
- **Completed:** 2026-03-30T11:21:52Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Renamed and confined the historical helper so the live protocol path remains `derive_view_secret_key(...)` while rotation stays explicit.
- Made `WalletService::receiver_keys(...)` live-only by default and added `receiver_keys_for_view_version(...)` for persisted historical recovery.
- Added lock-step regression coverage proving live send, scan, spend, and explicit rotation behavior stay aligned without silent helper drift.

## Task Commits

Each completed task now has a committed boundary:

1. **Task 1: Confine the live protocol path to `derive_view_secret_key(...)` and quarantine versioned derivation** - `902f7469` (fix)
2. **Task 2: Add lock-step regression coverage for sender, scanner, spend, and rotation flows** - `02f8a0d5` (test)

**Plan metadata:** recorded in the final docs commit for Plan 02 closure

## Files Created/Modified

- `.planning/phases/029-crypto-audit-wallets/029-02-PLAN.md` - plan artifact carrying the PH29 live-versus-historical contract and verification gates.
- `.planning/phases/029-crypto-audit-wallets/029-RECONCILIATION.md` - updated Gate 0 evidence to reflect the explicit rotated-helper naming and hot-path confinement.
- `crates/z00z_wallets/src/core/key/mod.rs` - narrowed the public key facade so rotated derivation is not re-exported through the broad default surface.
- `crates/z00z_wallets/src/core/key/stealth_keys.rs` - renamed the versioned helper to `derive_rotated_view_secret_key(...)` and kept rotation semantics explicit.
- `crates/z00z_wallets/src/services/wallet_service.rs` - moved default receiver-key retrieval to the live path and added explicit view-version lookup for historical state.
- `crates/z00z_wallets/tests/test_e2e_send_scan.rs` - kept the end-to-end send-scan anchor tied to the live default path.
- `crates/z00z_wallets/tests/test_rpc_key_derive_e2e.rs` - tightened the derivation anchor around the live-versus-historical contract.
- `crates/z00z_wallets/tests/test_view_key_contract.rs` - new regression suite covering live path, explicit rotation, negative spend substitution, and hot-path helper guards.

## Decisions Made

- `WalletService::receiver_keys(...)` now means the live receiver-key path, regardless of persisted rotation metadata.
- Historical rotation remains supported, but only through the explicitly named `receiver_keys_for_view_version(...)` surface.
- Review findings about lower-level rotated scanner behavior were classified as residual explicit-path risk, not a blocker for the live default contract, because that path still requires an intentional rotation step before use.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed version-driven default receiver-key retrieval after review surfaced remaining drift**

- **Found during:** Task 1 review passes
- **Issue:** `WalletService::receiver_keys(...)` still replayed persisted `view_key_version`, so default service callers could silently drift onto rotated keys after `rotate_recv_view()`.
- **Fix:** Switched `receiver_keys(...)` to the live path, introduced `receiver_keys_for_view_version(...)` for explicit historical retrieval, and updated service-level rotation tests to prove the default path stays live.
- **Files modified:** `crates/z00z_wallets/src/services/wallet_service.rs`
- **Verification:** `cargo test -p z00z_wallets --release recv_ver_ -- --nocapture`, `cargo test -p z00z_wallets --release --test test_view_key_contract -- --nocapture`, `cargo test -p z00z_wallets --release --test test_e2e_send_scan -- --nocapture`, `cargo test -p z00z_wallets --release --test test_rpc_key_derive_e2e -- --nocapture --test-threads=1`, bootstrap suite, full release-style workspace tests, and a final architecture review pass.
- **Committed in:** `902f7469`

**2. [Rule 3 - Blocking] Rewired plan-owned anchors so verification exercised real live-path coverage**

- **Found during:** Task 2 verification and review passes
- **Issue:** The new contract needed direct anchor assertions and a new focused suite to stop stale helper semantics from surviving in pre-existing tests.
- **Fix:** Added `test_view_key_contract.rs`, strengthened the send-scan and RPC derivation anchors, and kept the hot-path source guard against `derive_rotated_view_secret_key(...)` in live scan and spend files.
- **Files modified:** `crates/z00z_wallets/tests/test_e2e_send_scan.rs`, `crates/z00z_wallets/tests/test_rpc_key_derive_e2e.rs`, `crates/z00z_wallets/tests/test_view_key_contract.rs`
- **Verification:** `cargo test -p z00z_wallets --release --test test_view_key_contract -- --nocapture`, `cargo test -p z00z_wallets --release --test test_e2e_send_scan -- --nocapture`, `cargo test -p z00z_wallets --release --test test_rpc_key_derive_e2e -- --nocapture --test-threads=1`, bootstrap suite, and full release-style workspace tests.
- **Committed in:** `02f8a0d5`

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** Both fixes stayed inside the PH29 view-key contract and were necessary to make the default wallet path singular instead of merely test-green.

## Issues Encountered

- Repeated review passes showed that hot receive paths had been fixed earlier, but the broader service retrieval surface still defaulted to persisted rotated state. This required one more root-cause fix instead of accepting test-green output as sufficient.
- Codacy analysis on the edited service file reported only pre-existing complexity metrics outside the current diff; no actionable issues were introduced by the plan-owned changes.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 029 can proceed to the next remediation wave with a singular live wallet receiver-key contract at the service layer.
- Existing anchors now protect against rotated helper re-entry into live scan and spend hot paths.
- A lower-level residual risk remains in explicitly rotated `ReceiverKeys` values carrying legacy scan state through scanner helpers after manual rotation; this does not affect the default wallet path but is worth keeping in mind for later core-facade audits.

## Self-Check

PASSED

- Verified artifact exists: `.planning/phases/029-crypto-audit-wallets/029-02-PLAN.md`
- Verified artifact exists: `.planning/phases/029-crypto-audit-wallets/029-RECONCILIATION.md`
- Verified artifact exists: `crates/z00z_wallets/tests/test_view_key_contract.rs`
- Verified summary exists: `.planning/phases/029-crypto-audit-wallets/029-02-SUMMARY.md`
- Verified task commit exists: `902f7469`
- Verified task commit exists: `02f8a0d5`

---
*Phase: 029-crypto-audit-wallets*
*Completed: 2026-03-30*
