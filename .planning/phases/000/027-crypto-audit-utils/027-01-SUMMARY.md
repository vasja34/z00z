---
phase: 027-crypto-audit-utils
plan: "01"
subsystem: utils
tags: [rust, utils, os-hardening, memlock, wallets]
requires: []
provides:
  - lifetime-bound borrowed memlock guard for safe local callers
  - owned locked secret wrapper for long-lived wallet session state
  - public integration coverage for borrowed and owned memlock holder shapes
affects: [027-02, 027-03, 027-04, 027-05, 027-06, z00z_utils, z00z_wallets]
tech-stack:
  added: []
  patterns: [lifetime-bound-guard, owned-locked-secret-wrapper, lock-before-fill-handoff]
key-files:
  created:
    - .planning/phases/027-crypto-audit-utils/027-01-SUMMARY.md
  modified:
    - crates/z00z_utils/src/os_hardening.rs
    - crates/z00z_utils/tests/test_os_hardening_integration.rs
    - crates/z00z_wallets/src/db/redb_wallet_store.rs
key-decisions:
  - "Keep the public borrowed `LockedBytes<'a>` contract for local safe callers and add a separate owned wrapper for long-lived wallet session state."
  - "Initialize owned locked secrets through a fill-based constructor that locks zeroed storage before writing secret bytes on successful OS lock paths."
  - "Convert wallet-open bootstrap secrets into owned locked storage, then explicitly drop the original mnemonic and hidden bootstrap locals immediately after handoff."
patterns-established:
  - "Borrow for local scope, own for session scope: use lifetime-bound guards only while the backing borrow is local, and switch to owned locked storage for long-lived holders."
  - "Lock before fill when possible: acquire best-effort page locks on zeroed storage before writing sensitive bytes into a long-lived buffer."
requirements-completed: [PH27-MEMLOCK]
duration: multi-session
completed: 2026-03-29
---

# Phase 027 Plan 01 Summary

📌 **Closed the Phase 027 memlock blocker with a lifetime-bound public guard in `z00z_utils`, an owned locked secret wrapper for long-lived wallet state, and executable validation across release, Miri, and downstream wallet feature builds.**

## Performance

- **Duration:** multi-session
- **Completed:** 2026-03-29T10:26:48+00:00
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- ✅ Replaced the address-only `LockedBytes` shape with a lifetime-bound `LockedBytes<'a>` guard backed by `NonNull<u8>` plus `PhantomData<&'a mut [u8]>`.
- ✅ Kept zeroize-before-unlock behavior and sanitized `Debug` output so the public memlock surface no longer exposes raw addresses.
- ✅ Added `OwnedLockedBytes<N>` for long-lived secret holders and wired the wallet `os_hardening` path to that owned wrapper instead of trying to store borrowed guards in `OpenedWlt`.
- ✅ Eliminated transient plaintext copies on the wallet open and verification paths by copying directly into locked storage, comparing by reference, and dropping bootstrap mnemonic and hidden locals immediately after handoff.
- ✅ Extended integration coverage so the public API proves both the borrowed guard contract and an owned session-like holder shape.

## Task Commits

📌 This execution closed the plan from validated working-tree state.

1. **Task 1: Replace the address-only guard with a lifetime-bound `LockedBytes<'a>` contract** - not separately committed in this execution
2. **Task 2: Add guard-shape regression coverage and Miri-oriented validation hooks** - not separately committed in this execution

**Plan metadata:** not committed in this execution; repo-owned git/versioning checkpoint remains deferred.

## Files Created/Modified

- `crates/z00z_utils/src/os_hardening.rs` - introduced shared low-level lock state, lifetime-bound borrowed guard, owned locked secret wrapper, lock-before-fill constructor, and redacted debug surfaces
- `crates/z00z_utils/tests/test_os_hardening_integration.rs` - added public holder-shape coverage for `OwnedLockedBytes` alongside the existing borrowed guard seam checks
- `crates/z00z_wallets/src/db/redb_wallet_store.rs` - moved `OpenedWlt` secret storage to owned locked buffers under `os_hardening`, removed borrowed lock fields, shortened bootstrap secret lifetimes, and removed extra stack copies from password verification

## Decisions Made

- 📌 The public Phase 027 fix stays two-tiered: `LockedBytes<'a>` remains the safe local API, while `OwnedLockedBytes<N>` owns long-lived secret state that must outlive a local borrow.
- 📌 `OwnedLockedBytes::new_best_effort_with(...)` is the canonical handoff seam for existing secret wrappers because it avoids transient stack copies and can lock zeroed storage before fill.
- 📌 The wallet `os_hardening` path now treats bootstrap `Hidden` and mnemonic values as short-lived staging objects only; they are dropped immediately after copying into the owned locked session state.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] The lifetime-safe public guard broke the downstream wallet `os_hardening` feature build**

- **Found during:** final validation of Task 1
- **Issue:** `OpenedWlt` could no longer store borrowed `LockedBytes<'a>` guards without threading lifetimes through long-lived wallet session types.
- **Fix:** added `OwnedLockedBytes<N>` in `z00z_utils` and switched the wallet `os_hardening` holder shape to owned locked secret storage.
- **Files modified:** `crates/z00z_utils/src/os_hardening.rs`, `crates/z00z_wallets/src/db/redb_wallet_store.rs`
- **Verification:** `cargo check -p z00z_wallets --features os_hardening`
- **Committed in:** not committed in this execution

**2. [Rule 1 - Bug] The first owned-wrapper draft reintroduced transient plaintext copies during wallet-open handoff**

- **Found during:** final review loop
- **Issue:** by-value handoff and stack-local comparisons materialized unprotected copies of master key and BIP-39 seed bytes outside locked storage.
- **Fix:** added the fill-based owned constructor, switched wallet handoff to `copy_from_slice(...)` into locked storage, and changed password verification to constant-time compare on references.
- **Files modified:** `crates/z00z_utils/src/os_hardening.rs`, `crates/z00z_wallets/src/db/redb_wallet_store.rs`
- **Verification:** `cargo test -p z00z_utils --release --lib`; `cargo check -p z00z_wallets --features os_hardening`
- **Committed in:** not committed in this execution

**3. [Rule 2 - Missing Critical] The public integration seam did not yet prove an owned session-like holder shape**

- **Found during:** final review loop
- **Issue:** the public integration target only proved the borrowed `LockedBytes<'a>` contract, leaving the new owned holder shape protected only by compile success.
- **Fix:** added a public integration test that stores `OwnedLockedBytes<32>` inside a holder struct, verifies stable reveal semantics, and keeps debug output redacted.
- **Files modified:** `crates/z00z_utils/tests/test_os_hardening_integration.rs`
- **Verification:** `cargo test -p z00z_utils --release --test test_os_hardening_integration`; `cargo +nightly miri test -p z00z_utils --test test_os_hardening_integration`
- **Committed in:** not committed in this execution

---

**Total deviations:** 3 auto-fixed (2 bug, 1 missing critical)
**Impact on plan:** scope stayed inside `PH27-MEMLOCK`, but the closure proof had to include the real downstream wallet feature seam because the lifetime fix surfaced a genuine compatibility regression there.

## Issues Encountered

- ⚠️ The initial lifetime-safe guard design was correct for local callers but exposed a real architectural mismatch for long-lived wallet session storage.
- ⚠️ Final security review still surfaced broader pre-existing secret-lifetime opportunities elsewhere in the wallet decode or store flow; those were not introduced by this plan and remain outside this plan’s closure boundary.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- ✅ `PH27-MEMLOCK` is now closed with release, Miri, and downstream wallet feature evidence.
- ✅ Later Phase 027 waves can build on a stable `z00z_utils` secret-memory seam without carrying the lifetime blocker forward.
- ✅ The wallet `os_hardening` feature compiles again without unsafe code in `z00z_wallets`.

## Validation Evidence

- ✅ `cargo test -p z00z_utils --release --lib` -> `149 passed; 0 failed`
- ✅ `cargo test -p z00z_utils --release --test test_os_hardening_integration` -> `8 passed; 0 failed`
- ✅ `cargo +nightly miri test -p z00z_utils --test test_os_hardening_integration` -> `5 passed; 0 failed`
- ✅ `cargo check -p z00z_wallets --features os_hardening` -> build passed
- ✅ Final architecture review -> `CLEAN`
- ✅ Final general review -> `CLEAN`
- ✅ Final security review on the changed memlock seam found no new blocker inside the closed handoff path after the last fixes; broader wallet secret-lifetime follow-ups remain out of scope for this plan

## Self-Check: PASSED

- ✅ Summary artifact created at `.planning/phases/027-crypto-audit-utils/027-01-SUMMARY.md`
- ✅ Validation evidence recorded and matched the final tested working tree
- ✅ No commit hashes were claimed because git checkpointing was not performed in this execution

---

*Phase: 027-crypto-audit-utils*
*Completed: 2026-03-29*
