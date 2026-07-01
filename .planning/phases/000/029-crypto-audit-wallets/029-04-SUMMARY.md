---
phase: 029-crypto-audit-wallets
plan: "04"
subsystem: wallets
tags: [wallets, panic, seed-salt, simulator, errors]
requires:
  - phase: 029-crypto-audit-wallets
    provides: 029-RECONCILIATION.md scope freeze and 029-03 KDF governance baseline
provides:
  - runtime panic inventory for wallet service and related operator-facing seams
  - typed runtime error propagation instead of operator-reachable panic sites
  - random wallet-owned seed-salt contract for new snapshots and export or reveal flows
  - simulator Stage 2 compatibility with the persisted export seed-salt boundary
affects: [029-05, 029-06]
tech-stack:
  added: []
  patterns: [runtime panic classification, wallet-owned seed salt, persisted export salt reuse, bounded legacy rejection]
key-files:
  created:
    - .planning/phases/029-crypto-audit-wallets/029-04-SUMMARY.md
    - .planning/phases/029-crypto-audit-wallets/029-PANIC-INVENTORY.md
    - crates/z00z_wallets/tests/test_wallet_service_errors.rs
    - crates/z00z_wallets/tests/test_seed_salt_policy.rs
  modified:
    - crates/z00z_simulator/src/scenario_1/stage_2.rs
    - crates/z00z_simulator/src/scenario_1/stage_2_utils/artifacts.rs
    - crates/z00z_simulator/src/scenario_1/stage_2_utils/flow.rs
    - crates/z00z_simulator/src/scenario_1/stage_2_utils/mod.rs
    - crates/z00z_wallets/src/core/wallet/snapshot.rs
    - crates/z00z_wallets/src/services/wallet_service.rs
    - crates/z00z_wallets/tests/test_show_seed_phrase_plaintext.rs
key-decisions:
  - "Classify wallet panic sites explicitly and only rewrite operator-reachable runtime seams to typed WalletError propagation."
  - "Make seed-salt wallet-owned and persisted in the snapshot instead of deriving new-write salt from wallet_id."
  - "Require export and reveal verification paths, including simulator Stage 2, to reuse the persisted seed_salt from the encrypted export payload."
patterns-established:
  - "Typed runtime failure first: operator-facing wallet flows fail closed with WalletError instead of process termination."
  - "Persisted salt ownership: one wallet snapshot field governs reveal and export salt semantics across wallet and simulator code."
requirements-completed: [PH29-PANIC, PH29-SEEDSALT]
duration: checkpointed
completed: 2026-03-30
---

# Phase 029 Plan 04: Panic And Seed-Salt Closure Summary

Typed runtime wallet failures plus a wallet-owned random seed-salt contract for new writes, reveal flows, export flows, and simulator verification.

## Performance

- **Duration:** checkpointed
- **Started:** not recorded in resumed session
- **Completed:** 2026-03-30T16:45:23Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- 📌 Built `.planning/phases/029-crypto-audit-wallets/029-PANIC-INVENTORY.md` as the runtime-versus-test panic ledger for wallet service and adjacent findings.
- 📌 Replaced operator-reachable runtime failure paths with typed wallet errors instead of letting `expect()` or `unwrap()` terminate the process in live flows.
- 📌 Moved new wallet persistence to a snapshot-owned `seed_salt` field and rejected partial current-format snapshots that omit it.
- 📌 Kept seed reveal and export verification aligned to the persisted salt, then updated the simulator Stage 2 export verification path so local proof checks use the same contract.

## Task Commits

The resumed execution wave landed the plan-owned implementation in one version-managed commit because the validated code and regression files were already present in a single working-tree slice:

1. **Task 1: Classify and replace runtime `expect()` or `unwrap()` sites with typed wallet errors** - `a3a26e08` (feat, shared implementation commit)
2. **Task 2: Move new writes to random wallet-owned seed salt and keep deterministic salt as legacy-only fallback** - `a3a26e08` (feat, shared implementation commit)

**Plan metadata:** recorded in the final docs commit for Plan 04 closure

## Files Created Or Modified

- `.planning/phases/029-crypto-audit-wallets/029-PANIC-INVENTORY.md` - explicit classification of runtime, fixture-only, and test-only panic sites with current-tree evidence.
- `crates/z00z_wallets/src/services/wallet_service.rs` - typed runtime error propagation, export salt decoding, and wallet-owned seed-salt usage in reveal or export paths.
- `crates/z00z_wallets/src/core/wallet/snapshot.rs` - snapshot version 4 with mandatory persisted `seed_salt` for current-format wallets.
- `crates/z00z_wallets/tests/test_wallet_service_errors.rs` - regression coverage for typed runtime failures in operator-reachable service seams.
- `crates/z00z_wallets/tests/test_seed_salt_policy.rs` - regression coverage for random persisted seed-salt behavior and rejection of incomplete current snapshots.
- `crates/z00z_wallets/tests/test_show_seed_phrase_plaintext.rs` - encrypted seed-reveal anchor updated to consume the persisted salt from the exported snapshot contract.
- `crates/z00z_simulator/src/scenario_1/stage_2.rs` - Stage 2 kept aligned with the wallet-owned export contract.
- `crates/z00z_simulator/src/scenario_1/stage_2_utils/artifacts.rs` - export payload decryption now accepts explicit persisted seed-salt input.
- `crates/z00z_simulator/src/scenario_1/stage_2_utils/flow.rs` - Stage 2 flow decodes `seed_salt` from the export payload before local verification.
- `crates/z00z_simulator/src/scenario_1/stage_2_utils/mod.rs` - shared stage exports kept aligned with the revised Stage 2 helper surface.

## Decisions Made

- 📌 Current-format wallet snapshots now require an embedded `seed_salt`; older snapshots are compatibility-only instead of being silently rewritten.
- 📌 The canonical source of reveal and export salt is the persisted wallet snapshot, not a reconstructed `wallet_id`-derived value.
- 📌 Panic findings remain phase-owned only when they are operator-reachable; fixture-only and test-only invariants stay documented rather than being misreported as runtime blockers.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated simulator Stage 2 verification to decode persisted export seed salt**

- **Found during:** Plan verification after the wallet-owned seed-salt rollout
- **Issue:** Simulator Stage 2 local verification still assumed the superseded deterministic seed-salt path and surfaced a generic cryptographic failure even after wallet-side tests passed.
- **Fix:** Added export seed-salt decoding in the wallet service and simulator helpers, then rewired the Stage 2 verification flow to use the persisted wallet-owned salt from the encrypted export payload.
- **Files modified:** `crates/z00z_wallets/src/services/wallet_service.rs`, `crates/z00z_simulator/src/scenario_1/stage_2.rs`, `crates/z00z_simulator/src/scenario_1/stage_2_utils/artifacts.rs`, `crates/z00z_simulator/src/scenario_1/stage_2_utils/flow.rs`, `crates/z00z_simulator/src/scenario_1/stage_2_utils/mod.rs`
- **Verification:** release-style wallet plan tests plus the simulator Stage 2 validation flow
- **Committed in:** `a3a26e08`

---

**Total deviations:** 1 auto-fixed blocking issue
**Impact on plan:** The fix stayed inside the seed-salt compatibility boundary and was required to make the new wallet-owned salt contract observable outside the wallet crate.

## Issues Encountered

- The working tree already contained the validated plan-owned implementation before formal execute-phase closure, so the plan landed as one shared implementation commit instead of two isolated task commits.
- Codacy analysis on the final verification-driven `wallet_impl.rs` adjustment reported only pre-existing complexity warnings outside the current fix.

## User Setup Required

None.

## Next Phase Readiness

- 📌 Phase 029 can proceed with wallet snapshots, reveal flows, export flows, and simulator verification all sharing one wallet-owned seed-salt contract.
- 📌 Runtime wallet errors now fail closed through typed error surfaces, which reduces ambiguity for later validator and metadata policy work.

## Self-Check

PASSED

- Verified artifact exists: `.planning/phases/029-crypto-audit-wallets/029-PANIC-INVENTORY.md`
- Verified artifact exists: `crates/z00z_wallets/tests/test_wallet_service_errors.rs`
- Verified artifact exists: `crates/z00z_wallets/tests/test_seed_salt_policy.rs`
- Verified summary exists: `.planning/phases/029-crypto-audit-wallets/029-04-SUMMARY.md`
- Verified task commit exists: `a3a26e08`

---
*Phase: 029-crypto-audit-wallets*
*Completed: 2026-03-30*
