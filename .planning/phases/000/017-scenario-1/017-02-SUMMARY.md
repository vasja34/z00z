---
phase: 017-scenario-1
plan: 02
subsystem: testing
tags: [scenario-1, wallet, tx, witness, validation]
requires:
  - phase: 016-jmt-search-and-redb
    provides: storage root and witness semantics for canonical path-bound resolution
provides:
  - wallet boundary validation for wrong-root, wrong-path, and tampered witness bytes
  - path-bound resolved input behavior for regular transfer inputs
  - canonical wallet-facing checkpoint summary validation
affects: [017-scenario-1, wallet-boundary, tx-verification]
tech-stack:
  added: []
  patterns: [typed witness boundary, canonical AssetPath resolution, tamper rejection]
key-files:
  created: []
  modified:
    - crates/z00z_wallets/src/core/tx/state_update.rs
    - crates/z00z_wallets/tests/test_tx_wrong_root.rs
    - crates/z00z_wallets/tests/test_tx_tamper.rs
    - crates/z00z_wallets/tests/test_tx_roundtrip.rs
key-decisions:
  - "Require canonical storage witness bytes at the first wallet/storage boundary"
  - "Keep resolved inputs bound to full AssetPath semantics instead of compact refs alone"
patterns-established:
  - "Wallet-side validation must reject wrong roots before any state application"
  - "Roundtrip coverage must prove canonical path-bound resolution"
requirements-completed: [SCN1-02]
duration: phase 017 session
completed: 2026-03-24
---

# Phase 017: Scenario 1 Summary

The wallet/storage boundary now rejects wrong roots and tampered witness bytes while preserving full path-bound resolution semantics.

## Performance

- **Duration:** phase 017 session
- **Started:** 2026-03-24T11:20:49Z
- **Completed:** 2026-03-24T11:20:49Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- The first wallet/storage boundary rejects wrong-root, wrong-path, and tampered witness bytes before state application.
- Resolved inputs remain tied to canonical AssetPath semantics instead of compact refs alone.
- Roundtrip coverage proves the wallet-facing checkpoint summary path uses canonical storage witness bytes.

## Task Commits

1. **Task 1: Wrong-root and tamper rejection** - pending
2. **Task 2: Path-bound roundtrip coverage** - pending

**Plan metadata:** pending

## Files Created/Modified

- crates/z00z_wallets/src/core/tx/state_update.rs - typed wallet/storage witness boundary
- crates/z00z_wallets/tests/test_tx_wrong_root.rs - wrong-root regression coverage
- crates/z00z_wallets/tests/test_tx_tamper.rs - tamper regression coverage
- crates/z00z_wallets/tests/test_tx_roundtrip.rs - canonical roundtrip coverage

## Decisions Made

- Canonical witness validation belongs at the wallet boundary before application.
- The regular-transfer contract keeps path semantics explicit instead of collapsing to compact refs.

## Deviations from Plan

None - plan executed as specified.

## Issues Encountered

Some harnesses are compile-only in debug; release verification was used to confirm the runtime behavior.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

The wallet boundary is ready for the storage-backed checkpoint bridge and the later apply/finalization stages.

---
*Phase: 017-scenario-1*
*Completed: 2026-03-24*
