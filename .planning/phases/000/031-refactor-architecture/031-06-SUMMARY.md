---
phase: 031-refactor-architecture
plan: "06"
subsystem: auth
tags: [wallets, rpc, identity, session, authorization, drift]
requires:
  - phase: 031-02
    provides: inventory-backed wallet caller map and seam inventory for wallet drift follow-ups
  - phase: 031-03
    provides: wallet core seam narrowing used by persisted identity enforcement
  - phase: 031-04
    provides: wallet RPC seam extraction used by transport-level auth hardening
  - phase: 031-05
    provides: wallet service split used by persisted identity and session-bound lock fixes
provides:
  - persisted wallet identity remains authoritative after discovery, open, and unlock flows
  - wallet-owned derive and address paths resolve chain identity from persisted wallet state instead of runtime env drift
  - lock_wallet transport calls require an authenticated session token and expose an exact unauthenticated denial anchor
affects: [wallets, rpc, session-management, address-derivation, follow-up facade cleanup]
tech-stack:
  added: []
  patterns: [persisted-wallet-identity-authority, session-bound-rpc-mutation]
key-files:
  created: []
  modified:
    - crates/z00z_wallets/src/services/wallet_service_store_create_unlock_open.rs
    - crates/z00z_wallets/src/services/wallet_service_store_support.rs
    - crates/z00z_wallets/src/services/wallet_service_session_derivation.rs
    - crates/z00z_wallets/src/services/wallet_service_session_derivation_recovery.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/wallet_impl.rs
    - crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring_register.rs
    - crates/z00z_wallets/tests/test_open_wallet_source_discovery.rs
    - crates/z00z_wallets/tests/test_rpc_wiring_spec_a.rs
    - crates/z00z_wallets/tests/test_wallet_service_errors.rs
key-decisions:
  - "Persisted (wallet_id, network, chain) became the only authoritative post-discovery identity; runtime wallet env may validate but cannot redefine a known wallet."
  - "Wallet derive and emitted address flows must resolve chain identity through persisted wallet ownership helpers instead of reading Z00Z_WALLET_* after wallet_id is known."
  - "wallet.session.lock_wallet is session-bound at the RPC boundary; list visibility is not sufficient authorization for mutation."
patterns-established:
  - "Persisted wallet identity authority: session and reopened-wallet flows must prefer active or persisted wallet identity before any runtime fallback."
  - "RPC mutation authorization: wallet state-changing methods must prove session authorization in the transport layer, not only in service internals."
requirements-completed: [PH31-WLT-ID]
duration: continued-session
completed: 2026-04-04
---

# Phase 031 Plan 06: Wallet Identity and Lock Authorization Summary

## Outcome

Persisted wallet identity now governs open, unlock, derive, and address flows, while `lock_wallet` rejects unauthenticated transport callers through an explicit session-bound RPC contract.

## Performance

- **Duration:** continued-session
- **Started:** 2026-04-04T15:49:40Z
- **Completed:** 2026-04-04T15:49:40Z
- **Tasks:** 2
- **Files modified:** 19

## Accomplishments

- Refactored discovery and open flows so persisted wallet identity, not runtime wallet env drift, governs follow-up unlock, derive, and address behavior once a wallet is known.
- Added exact drift regression anchors for both path and bytes lanes, including `open_wallet_bytes_rejects_identity_mismatch` and persisted-chain address emission coverage.
- Bound `wallet.session.lock_wallet` to authenticated session transport handling and proved denial via `lock_wallet_rejects_unauthenticated_transport_call`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Enforce persisted wallet identity as the only post-discovery source of truth** - pending
2. **Task 2: Bind `lock_wallet` to an explicit authorized caller contract** - pending

**Plan metadata:** pending

_Commit creation was deferred in this continuation because the repository worktree contains unrelated user changes and the canonical release-oriented version-manager `--stage-all` flow would capture unrelated diffs._

## Files Created/Modified

- `crates/z00z_wallets/src/services/wallet_service_store_create_unlock_open.rs` - discovery and open flows now cache and validate authoritative persisted wallet identity.
- `crates/z00z_wallets/src/services/wallet_service_store_support.rs` - persisted identity helpers now prefer active-session or persisted wallet state before runtime fallback and expose persisted chain resolution.
- `crates/z00z_wallets/src/services/wallet_service_session_derivation.rs` - derive and address emission paths now resolve chain type from persisted wallet identity.
- `crates/z00z_wallets/src/services/wallet_service_session_derivation_recovery.rs` - recovery and rebuild paths now reuse persisted chain identity.
- `crates/z00z_wallets/src/adapters/rpc/methods/wallet_impl.rs` - lock_wallet now verifies an authenticated session token before mutating lock state.
- `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring_register.rs` - RPC wiring remains aligned with explicit session-bound wallet mutation handling.
- `crates/z00z_wallets/tests/test_open_wallet_source_discovery.rs` - exact path and bytes drift regression anchors cover authoritative identity reuse and mismatch rejection.
- `crates/z00z_wallets/tests/test_wallet_service_errors.rs` - persisted-chain regression coverage proves list-address flows use wallet-owned chain identity after open_wallet_source.
- `crates/z00z_wallets/tests/test_rpc_wiring_spec_a.rs` - transport-level lock authorization denial and authenticated success anchors cover the new auth posture.

## Decisions Made

- Persisted identity is authoritative after discovery or reopen because runtime env drift must never redefine wallet compatibility data for a known wallet.
- Active unlocked session state is the preferred identity source when a wallet file is currently locked, preventing WalletInUse regressions during post-open service calls.
- Transport mutation auth is explicit: enumeration remains available where intended, but lock mutation requires session authorization.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Avoided locked-file rediscovery when persisted identity was already available in memory**

- **Found during:** Task 1 (persisted identity enforcement)
- **Issue:** Follow-up list-address and derive flows could rediscover a locked `.wlt` file after `open_wallet_source`, returning `WalletInUse` instead of using the already-known wallet identity.
- **Fix:** Persisted identity helpers now prefer active-session cached identity before filesystem rediscovery, then fall back explicitly.
- **Files modified:** `crates/z00z_wallets/src/services/wallet_service_store_support.rs`, `crates/z00z_wallets/src/services/wallet_service_store_create_unlock_open.rs`
- **Verification:** `list_addresses_use_persisted_chain_after_open_wallet_source` and the `open_wallet_source` regression anchors pass.
- **Committed in:** pending

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The auto-fix was required to make the persisted-identity contract operational after live discovery and did not widen scope beyond the wallet identity seam.

## Issues Encountered

- The requested broad release-style wallet test sweep still fails outside this plan in `crates/z00z_wallets/tests/test_tx_assetpack.rs` because it imports `z00z_core::leaf::PackErr`, a stale path removed by earlier core refactors. This remains out of scope for `031-06` and is tracked in `deferred-items.md`.
- The review prompt `/.github/prompts/gsd-review-tasks-execution.prompt.md` was available as guidance text only in this environment, so final review evidence relied on exact plan-scoped regression tests plus auth/session surface inspection instead of prompt-runner automation.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Wallet identity drift is fail-closed for the covered path and bytes open lanes, and wallet mutation auth has an exact RPC denial proof.
- Later wallet facade cleanup can build on persisted identity helpers without preserving runtime-env ownership of chain semantics.
- A separate follow-up remains needed for the unrelated `test_tx_assetpack.rs` stale core import before broad wallet release gates are fully green.

## Self-Check

PASSED

- FOUND: `.planning/phases/031-refactor-architecture/031-06-SUMMARY.md`
- FOUND: `.planning/phases/031-refactor-architecture/deferred-items.md`
- Commit hashes remain intentionally pending because this continuation did not create a repo-level commit while unrelated user changes are present in the worktree.

---
_Phase: 031-refactor-architecture_
_Completed: 2026-04-04_
