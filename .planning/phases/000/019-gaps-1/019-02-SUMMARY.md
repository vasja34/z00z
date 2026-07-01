---
phase: 019-gaps-1
plan: 02
subsystem: wallet
tags: [receive-taxonomy, stealth-scanner, rpc, simulator, parity]
requires:
  - phase: 019-01
    provides: storage-owned claim nullifier transition used as the prior gap-closure baseline
provides:
  - report-first runtime receive contract across wallet and RPC entrypoints
  - explicit ReceiveReport-based parity coverage for public runtime receive flows
  - taxonomy-safe tx verification path that classifies before reconstructing owned outputs
affects: [019-03, wallet-backups, runtime-receive, rpc-taxonomy]
tech-stack:
  added: []
  patterns: [report-first receive classification, explicit invalid-input taxonomy, compatibility-only scan_leaf reconstruction]
key-files:
  created: []
  modified:
    - crates/z00z_wallets/src/core/address/stealth_scanner.rs
    - crates/z00z_wallets/src/services/wallet_service.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/asset_impl.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/tx_impl.rs
    - crates/z00z_wallets/tests/test_e2e_runtime_parity.rs
    - crates/z00z_wallets/examples/wallet_reload.rs
key-decisions:
  - "Treat ReceiveReport as the authoritative public runtime receive contract and keep ScanResult only for compatibility-only owned-output reconstruction."
  - "Update direct RPC and parity/example callers in the same pass so report-first semantics do not drift behind public entrypoints."
patterns-established:
  - "Runtime receive callers classify first with scan_report/recv_one and only reconstruct owned outputs after a detected report."
  - "Examples and parity tests track ReceiveReport semantics directly instead of assuming ScanResult on public boundaries."
requirements-completed: [PH19-SCAN]
duration: unknown
completed: 2026-03-25
---

# Phase 019 Plan 02: Receive Taxonomy Summary

## Outcome

Report-first runtime receive classification across wallet and RPC entrypoints with explicit parity coverage and compatibility-only owned-output reconstruction.

## Performance

- **Duration:** unknown
- **Started:** 2026-03-25T16:11:40Z
- **Completed:** 2026-03-25T17:06:00Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Switched the public runtime receive boundary to a report-first `ReceiveReport` contract in the scanner, wallet service, and RPC-facing call paths.
- Hardened direct tx verification and asset receive paths so they classify receive status before using compatibility-only `scan_leaf()` reconstruction.
- Updated parity and example coverage so the public runtime path is validated against report-first semantics instead of raw `ScanResult` expectations.

## Task Commits

Each task was committed atomically:

1. **Task 1: Harden authoritative runtime receive report semantics** - `1f4b312f` (feat)
2. **Task 2: Migrate direct adapters and high-value receive callers to taxonomy-safe entry points** - `1f4b312f` (feat)

**Plan metadata:** pending state reconciliation

_Note: The task work landed as one focused receive-taxonomy commit because the report-first contract and caller migration were interleaved in the same validated change set._

## Files Created/Modified

- `crates/z00z_wallets/src/core/address/stealth_scanner.rs` - Adds the runtime report-first scanner surface used as the authoritative classification entrypoint.
- `crates/z00z_wallets/src/services/wallet_service.rs` - Aligns `recv_one` with `ReceiveReport` semantics at the public wallet-service boundary.
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl.rs` - Migrates asset receive RPC flows and tests to report-first taxonomy handling.
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl.rs` - Makes tx verification classify outputs first and reconstruct owned payloads only after detection.
- `crates/z00z_wallets/tests/test_e2e_runtime_parity.rs` - Verifies canonical/runtime parity using `ReceiveReport` on the public runtime path.
- `crates/z00z_wallets/examples/wallet_reload.rs` - Updates the runtime reload example to the report-first receive contract.

## Decisions Made

- Use `ReceiveReport` as the only authoritative public receive taxonomy surface for runtime callers.
- Keep `ScanResult` as a secondary compatibility helper where owned-output materialization is still needed after classification.
- Carry the taxonomy migration through parity/example surfaces in the same plan so public callers do not diverge.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated wallet reload example for report-first API drift**

- **Found during:** Task 2 (direct caller migration)
- **Issue:** `examples/wallet_reload.rs` still expected `recv_one()` to return `ScanResult`, which broke targeted wallet test execution after the public API migrated to `ReceiveReport`.
- **Fix:** Reworked the example to validate `ReceiveReport` detection semantics and print summary data from the asset itself.
- **Files modified:** `crates/z00z_wallets/examples/wallet_reload.rs`
- **Verification:** `cargo test -p z00z_wallets asset_receive_api_sync --features test-fast -- --nocapture`
- **Committed in:** `1f4b312f`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The fix stayed inside the report-first receive migration boundary and was required to keep wallet test targets compiling.

## Issues Encountered

- `.planning/STATE.md` still points to phase 019 plan 1, so plan-level state advancement and metadata commit were not applied here to avoid overwriting unresolved 019-01 workflow bookkeeping.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The report-first receive migration is validated on targeted RPC, tx verification, parity, and broader `z00z_wallets` test coverage.
- Phase bookkeeping still needs reconciliation because 019-01 does not yet have its summary/state closure while 019-02 code is now complete.

## Self-Check: PASSED

- Found summary target: `.planning/phases/019-gaps-1/019-02-SUMMARY.md`
- Found task commit: `1f4b312f`

---
_Phase: 019-gaps-1_
_Completed: 2026-03-25_
