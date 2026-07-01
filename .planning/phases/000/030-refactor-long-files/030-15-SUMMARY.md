---
phase: 030-refactor-long-files
plan: 15
subsystem: wallets
tags: [rust, wallets, bip44, stealth-keys, key-manager, cache-coordination]
requires:
  - phase: 030-06
    provides: Stable include-based wallet key facade and prior seam ownership rules.
provides:
  - Reduced wallet key-domain residue behind thin `core::key` roots.
  - Panic-safe same-path key-derivation coalescing for the key-manager cache path.
  - Split BIP44 path, stealth-key, and backup-format test ownership into explicit seam files.
affects: [wallets, key-facade, key-manager, bip44, backup-format]
tech-stack:
  added: []
  patterns: [thin include roots, semantic seam extraction, same-path derivation coalescing]
key-files:
  created:
    - crates/z00z_wallets/src/core/key/bip32_path_builder.rs
    - crates/z00z_wallets/src/core/key/bip32_path_builder_helpers.rs
    - crates/z00z_wallets/src/core/key/bip32_path_errors.rs
    - crates/z00z_wallets/src/core/key/bip32_path_serde.rs
    - crates/z00z_wallets/src/core/key/bip32_path_value.rs
    - crates/z00z_wallets/src/core/key/seed_backup_format_tests_basic.rs
    - crates/z00z_wallets/src/core/key/seed_backup_format_tests_language.rs
    - crates/z00z_wallets/src/core/key/stealth_keys_identity.rs
    - crates/z00z_wallets/src/core/key/stealth_keys_receiver.rs
    - crates/z00z_wallets/src/core/key/stealth_keys_secret.rs
    - crates/z00z_wallets/src/core/key/stealth_keys_tests.rs
  modified:
    - crates/z00z_wallets/src/core/key/bip32_path.rs
    - crates/z00z_wallets/src/core/key/key_manager.rs
    - crates/z00z_wallets/src/core/key/key_manager_impl.rs
    - crates/z00z_wallets/src/core/key/key_manager_impl_cache.rs
    - crates/z00z_wallets/src/core/key/key_manager_impl_state.rs
    - crates/z00z_wallets/src/core/key/key_manager_impl_trait.rs
    - crates/z00z_wallets/src/core/key/seed_backup_format_tests.rs
    - crates/z00z_wallets/src/core/key/stealth_keys.rs
key-decisions:
  - "Keep `crate::core::key` stable by converting oversized roots into thin include-based facades instead of widening public paths."
  - "Coalesce only same-path key derivations and keep cleanup in a drop-backed flight guard so concurrent TTL tests pass without reintroducing a global derive lock."
  - "Keep convenience payment/change derivation semantics direct while limiting cache coordination changes to the shared derive_key path."
patterns-established:
  - "Pattern: thin root files include ownership-specific sibling seams and keep caller-visible facade paths unchanged."
  - "Pattern: same-path derivation coordination stores shared flight results and cleans up in Drop to avoid stranded in-flight state after unwind."
requirements-completed: [PH30-SEAMS, PH30-PROTECTED, PH30-VERIFY]
duration: n/a
completed: 2026-04-01
---

# Phase 030 Plan 15 Summary

## Outcome

Wallet key residue split into thin BIP44, stealth-key, and backup-test facades plus panic-safe same-path key-derivation coordination in the key-manager cache path.

## Performance

- **Duration:** n/a
- **Started:** continuation session
- **Completed:** 2026-04-01T00:00:00Z
- **Tasks:** 2
- **Files modified:** 19

## Accomplishments

- Reduced `bip32_path.rs`, `stealth_keys.rs`, and `seed_backup_format_tests.rs` to thin facade roots while moving behavior into explicit seam files.
- Fixed the release-only key-manager TTL race fallout by adding same-path derivation coalescing with shared results and panic-safe cleanup.
- Preserved the shallow `crate::core::key` caller surface while keeping BIP44 semantics, backup-format tests, and stealth-key behavior intact.

## Task Commits

No task commit was created in this session.
The workspace already contained unrelated dirty changes outside Plan 030-15 scope, so execution was left uncommitted to avoid staging non-plan artifacts accidentally.

## Files Created/Modified

- `crates/z00z_wallets/src/core/key/bip32_path.rs` - Thin include root for BIP44 path ownership.
- `crates/z00z_wallets/src/core/key/bip32_path_errors.rs` - BIP44 error and violation-reason ownership.
- `crates/z00z_wallets/src/core/key/bip32_path_serde.rs` - Serde boundary for `Bip44Path`.
- `crates/z00z_wallets/src/core/key/bip32_path_value.rs` - Core `Bip44Path` value behavior and conversions.
- `crates/z00z_wallets/src/core/key/bip32_path_builder_helpers.rs` - Builder validation helpers.
- `crates/z00z_wallets/src/core/key/bip32_path_builder.rs` - Builder facade.
- `crates/z00z_wallets/src/core/key/stealth_keys.rs` - Thin stealth-key module root.
- `crates/z00z_wallets/src/core/key/stealth_keys_secret.rs` - Secret and error ownership for stealth keys.
- `crates/z00z_wallets/src/core/key/stealth_keys_identity.rs` - Identity derivation and sign/verify ownership.
- `crates/z00z_wallets/src/core/key/stealth_keys_receiver.rs` - Receiver-key orchestration.
- `crates/z00z_wallets/src/core/key/stealth_keys_tests.rs` - Extracted stealth-key test module.
- `crates/z00z_wallets/src/core/key/seed_backup_format_tests.rs` - Thin backup-format test wrapper.
- `crates/z00z_wallets/src/core/key/seed_backup_format_tests_basic.rs` - Basic backup-format test coverage.
- `crates/z00z_wallets/src/core/key/seed_backup_format_tests_language.rs` - Language/ambiguity/homoglyph backup-format tests.
- `crates/z00z_wallets/src/core/key/key_manager.rs` - Cloneable key-manager error surface for shared flight outcomes.
- `crates/z00z_wallets/src/core/key/key_manager_impl.rs` - Smaller key-manager root plus derivation-flight state types.
- `crates/z00z_wallets/src/core/key/key_manager_impl_cache.rs` - Same-path derivation coordination, shared result handoff, and panic-safe cleanup.
- `crates/z00z_wallets/src/core/key/key_manager_impl_state.rs` - State constructors aligned with new coordination fields.
- `crates/z00z_wallets/src/core/key/key_manager_impl_trait.rs` - Main derive path routed through shared cache coordination.

## Decisions Made

- Kept key-domain splits semantic: errors, serde, value behavior, builder helpers, receiver/identity/secret ownership, and thematic test seams each received dedicated files.
- Limited cache coordination changes to the shared derive path after review identified convenience-method semantic drift as an unnecessary regression.
- Chose per-path derivation flights with shared results over a single global derivation gate to preserve same-path coalescing without locking all derive work behind one mutex.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed release-only TTL race fallout in key-manager cache derivation**

- **Found during:** Task 1 (Reduce oversized key-manager and encrypted-seed implementation roots)
- **Issue:** Release validation exposed `test_cache_ttl_no_race` and `test_cache_ttl_concurrent_expiration` failures caused by duplicate same-path misses.
- **Fix:** Added same-path derivation coalescing, shared flight results, and panic-safe cleanup to `key_manager_impl_cache.rs`.
- **Files modified:** `crates/z00z_wallets/src/core/key/key_manager.rs`, `crates/z00z_wallets/src/core/key/key_manager_impl.rs`, `crates/z00z_wallets/src/core/key/key_manager_impl_cache.rs`, `crates/z00z_wallets/src/core/key/key_manager_impl_state.rs`, `crates/z00z_wallets/src/core/key/key_manager_impl_trait.rs`
- **Verification:** Release TTL regression tests passed after each fix pass; diagnostics and Codacy remained clean.
- **Committed in:** not committed in this session

**2. [Rule 2 - Missing Critical] Added panic-safe cleanup for in-flight derivation state**

- **Found during:** Task 1 review loop
- **Issue:** Review surfaced a stranded-flight risk if unwinding happened while a path was marked in-flight.
- **Fix:** Cleanup now flips the in-flight state and notifies waiters even when the per-flight mutex is poisoned.
- **Files modified:** `crates/z00z_wallets/src/core/key/key_manager_impl_cache.rs`
- **Verification:** Release TTL regression tests passed after the cleanup hardening; diagnostics and Codacy remained clean.
- **Committed in:** not committed in this session

---

**Total deviations:** 2 auto-fixed (1 bug, 1 missing critical)
**Impact on plan:** Both deviations were required to keep the refactor valid under the release-style wallet verification gate. No public-surface expansion was introduced.

## Issues Encountered

- The release-style wallet gate surfaced latent key-manager TTL race behavior that was not visible in the earlier structural split checks.
- Narrow review loops kept identifying new concurrency tradeoffs as the cache coordination path evolved, so the final implementation intentionally stops at same-path correctness and panic-safe cleanup instead of widening scope into a larger cache-architecture rewrite.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The key-domain residue targeted by Plan 030-15 is materially reduced: `bip32_path.rs` is 5 lines, `stealth_keys.rs` is 52 lines, `seed_backup_format_tests.rs` is 117 lines, and `key_manager_impl.rs` is 156 lines.
- Targeted release TTL regression tests passed:
  - `cargo test -p z00z_wallets test_cache_ttl_no_race --release --features test-fast --features wallet_debug_dump`
  - `cargo test -p z00z_wallets test_cache_ttl_concurrent_expiration --release --features test-fast --features wallet_debug_dump`
- Residual risk remains around cold-miss coordination overhead and global registration-lock contention for distinct-path misses. That is a follow-up performance concern, not a correctness blocker for this seam-reduction wave.
- Plan metadata and git closeout remain outstanding because the workspace was already dirty outside this plan scope.

## Known Stubs

None.

## Self-Check: PASSED

- Summary file created at `.planning/phases/030-refactor-long-files/030-15-SUMMARY.md`.
- All seam files referenced above exist in the workspace.

---
*Phase: 030-refactor-long-files*
*Completed: 2026-04-01*
