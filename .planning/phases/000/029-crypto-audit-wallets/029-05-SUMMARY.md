---
phase: 029-crypto-audit-wallets
plan: "05"
subsystem: wallets
tags: [wallets, key-manager, receiver-secret, file-key-store, zeroize]
requires:
  - phase: 029-crypto-audit-wallets
    provides: 029-04 seed-salt and runtime error baseline
provides:
  - loud key-manager gap-limit invariant handling
  - enforced receiver-secret usability validation at object boundaries
  - zeroizing SafePassword persistence boundary for FileKeyStore
affects: [029-06]
tech-stack:
  added: []
  patterns: [checked invariant failure, constructor-bound validation, zeroizing password ownership]
key-files:
  created:
    - .planning/phases/029-crypto-audit-wallets/029-05-SUMMARY.md
    - crates/z00z_wallets/tests/test_receiver_secret_validation.rs
    - crates/z00z_wallets/tests/test_file_key_store.rs
  modified:
    - crates/z00z_wallets/src/core/key/key_manager.rs
    - crates/z00z_wallets/src/core/key/stealth_keys.rs
    - crates/z00z_wallets/src/core/storage/file_key_store.rs
    - crates/z00z_wallets/tests/test_key_manager.rs
key-decisions:
  - "Replace silent saturating gap-limit math with explicit corruption errors when last_used_plus1 exceeds next_index."
  - "Require ReceiverSecret usability validation at from-bytes, load, and decrypt boundaries so unusable secrets cannot escape constructors."
  - "Use SafePassword as the persistence boundary for FileKeyStore instead of cloneable plaintext Vec<u8> password containers."
patterns-established:
  - "Loud invariants: impossible allocation state becomes a typed error instead of silent zero-gap behavior."
  - "Secret wrapper ownership: password material stays behind zeroizing wrappers at the persistence seam."
requirements-completed: [PH29-KEYMGR, PH29-SECRET]
duration: checkpointed
completed: 2026-03-30
---

# Phase 029 Plan 05: Key-Manager And Secret Boundary Summary

Loud key-allocation invariants, constructor-bound receiver-secret validation, and zeroizing password ownership for persisted receiver secrets.

## Performance

- **Duration:** checkpointed
- **Started:** not recorded in resumed session
- **Completed:** 2026-03-30T16:45:23Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- 📌 Replaced silent key-manager gap-limit masking with explicit typed failures when persisted counters represent impossible allocation state.
- 📌 Added `ReceiverSecret` usability validation so invalid receiver secrets cannot survive object creation, loading, or decryption boundaries.
- 📌 Swapped FileKeyStore password handling onto `SafePassword` so secret-bearing persistence no longer relies on cloneable plaintext password buffers.
- 📌 Added focused regression coverage around gap-limit failure semantics, invalid receiver-secret rejection, and zeroizing persistence boundaries.

## Task Commits

The resumed execution wave landed the plan-owned implementation in one version-managed commit because the validated code and regression files were already present in a single working-tree slice:

1. **Task 1: Replace silent key-manager invariant masking and enforce usable receiver-secret construction** - `a3a26e08` (feat, shared implementation commit)
2. **Task 2: Replace cloneable password bytes in `FileKeyStore` with zeroizing wrappers** - `a3a26e08` (feat, shared implementation commit)

**Plan metadata:** recorded in the final docs commit for Plan 05 closure

## Files Created Or Modified

- `crates/z00z_wallets/src/core/key/key_manager.rs` - explicit corruption handling for impossible BIP-44 allocation state and preserved cache-validation behavior.
- `crates/z00z_wallets/src/core/key/stealth_keys.rs` - `ReceiverSecret::validate_usable()` and constructor or load enforcement so unusable secrets are rejected before escape.
- `crates/z00z_wallets/src/core/storage/file_key_store.rs` - SafePassword-owned receiver-secret persistence boundary instead of cloneable plaintext password buffers.
- `crates/z00z_wallets/tests/test_key_manager.rs` - coverage for loud gap-limit invariants without regressing existing TTL expectations.
- `crates/z00z_wallets/tests/test_receiver_secret_validation.rs` - focused coverage for invalid receiver-secret construction, load, and decrypt rejection.
- `crates/z00z_wallets/tests/test_file_key_store.rs` - regression coverage for zeroizing password-wrapper persistence and receiver-secret roundtrip behavior.

## Decisions Made

- 📌 Impossible key-manager counter state is corruption, not a recoverable zero-gap condition.
- 📌 Receiver secrets must prove canonical live view-key derivability before they become reusable wallet objects.
- 📌 Secret-bearing persistence boundaries in the wallet crate use the same zeroizing password-wrapper policy as stronger wallet encryption paths.

## Deviations from Plan

None - the validated plan-owned implementation matched the required hardening scope.

## Issues Encountered

- The working tree already contained the validated plan-owned implementation before formal execute-phase closure, so the plan landed as one shared implementation commit instead of two isolated task commits.

## User Setup Required

None.

## Next Phase Readiness

- 📌 Phase 029 can proceed with explicit key-allocation corruption semantics and constructor-bound receiver-secret validation already in place.
- 📌 Password-bearing persistence seams now match the zeroizing ownership policy expected by the remaining validation and metadata wave.

## Self-Check

PASSED

- Verified artifact exists: `crates/z00z_wallets/tests/test_receiver_secret_validation.rs`
- Verified artifact exists: `crates/z00z_wallets/tests/test_file_key_store.rs`
- Verified summary exists: `.planning/phases/029-crypto-audit-wallets/029-05-SUMMARY.md`
- Verified task commit exists: `a3a26e08`

---
*Phase: 029-crypto-audit-wallets*
*Completed: 2026-03-30*
