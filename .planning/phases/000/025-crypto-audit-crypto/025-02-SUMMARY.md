---
phase: 025-crypto-audit-crypto
plan: 02
subsystem: crypto-hardening
tags: [fail_closed, zkpack, experimental_surface, wallet_aead, release-verification]
requires:
  - phase: 021-refactor-continue
    provides: release-style workspace gates and canonical wallet zkpack facade already exist on the active branch
  - phase: 025-01
    provides: claim_v2 and source-proof contracts are already frozen for later wallet migration
provides:
  - fail-closed scalar, hash, and hmac helper entry points on the stable crypto facade
  - explicit zkpack wire decode errors instead of ambiguous option-based parsing
  - a non-default experimental namespace for the custom crypto zkpack path
affects: [025-03, 025-04, 025-05, wallet_stealth, claim_flow]
tech-stack:
  added: []
  patterns: [fail-closed helpers, compatibility shims without silent fallback, experimental facade boundary]
key-files:
  created:
    - crates/z00z_crypto/tests/test_fail_closed.rs
    - .planning/phases/025-crypto-audit-crypto/025-02-SUMMARY.md
  modified:
    - crates/z00z_crypto/src/hash.rs
    - crates/z00z_crypto/src/kdf.rs
    - crates/z00z_crypto/src/types.rs
    - crates/z00z_crypto/src/lib.rs
    - crates/z00z_crypto/src/zkpack.rs
key-decisions:
  - Export stable fail-closed `try_*` helpers and keep legacy helper names only as fail-stop compatibility shims with no constant fallback.
  - Keep the wallet ChaCha20-Poly1305 facade as the only blessed production zkpack path while moving the custom crypto pack path under `experimental`.
  - Preserve `ZkPackEncrypted` as the shared wire type, but require typed wire errors for invalid version or length handling.
patterns-established:
  - Fail-open cryptographic compatibility surfaces can remain temporarily only if they abort loudly instead of fabricating values.
  - Production guidance for pack encryption is now anchored to the wallet facade rather than to the custom crypto helper module.
requirements-completed: [PH25-ZKPACK, PH25-FAILCLOSED]
duration: multi-session
completed: 2026-03-27
---

# Phase 025 Plan 02: Fail-Closed Helpers And ZkPack Boundary Summary

📌 The stable crypto facade now exposes explicit fail-closed helper entry points, and the repository now presents the wallet AEAD facade as the only blessed production `ZkPack` path.

## Performance

- 📌 Duration: multi-session
- 📌 Completed: 2026-03-27
- 📌 Tasks: 2
- 📌 Files modified: 6

## Accomplishments

- 📌 Added `try_hmac_sha256`, `try_hmac_sha256_raw`, `try_hash_to_scalar_domain`, and `Z00ZScalar::try_from_hash` so callers can receive typed cryptographic failures instead of silent fallback values.
- 📌 Removed the unbounded recovery loop from `Z00ZScalar::random` and replaced constant-output compatibility behavior with fail-stop wrappers.
- 📌 Reclassified the custom crypto pack API under `z00z_crypto::experimental::aead_zkpack`, leaving the wallet ChaCha20-Poly1305 facade as the only blessed production reference path.
- 📌 Reworked `ZkPackEncrypted` wire parsing and serialization to return `ZkPackWireError` instead of ambiguous `Option` results.
- 📌 Added focused regression coverage in `test_fail_closed.rs` and kept the existing wallet zkpack vectors as the proof that the production path remains intact.

## Task Commits

📌 No git commits were created in this execution slice.

📌 The workspace contains unrelated pre-existing diffs, so git fixation remains deferred until a clean staging boundary can be created through the repo-owned versioning workflow.

## Files Created/Modified

- 📌 `crates/z00z_crypto/src/hash.rs` adds fail-closed HMAC and hash-to-scalar helpers and removes silent fallback outputs.
- 📌 `crates/z00z_crypto/src/kdf.rs` adds a fail-closed scalar-derivation helper compatible with the existing KDF facade.
- 📌 `crates/z00z_crypto/src/types.rs` adds `try_from_hash` and removes the public random-scalar recovery loop.
- 📌 `crates/z00z_crypto/src/lib.rs` re-exports the new `try_*` helpers and moves the custom zkpack module under the `experimental` namespace.
- 📌 `crates/z00z_crypto/src/zkpack.rs` introduces `ZkPackWireError` and explicit wire-format failure reporting.
- 📌 `crates/z00z_crypto/tests/test_fail_closed.rs` freezes deterministic non-zero behavior for the new fail-closed helper surface.

## Decisions Made

- 📌 Kept compatibility helper names only where broad callsite churn would have been disproportionate for this wave, but removed all silent fallback semantics from those surfaces.
- 📌 Chose an `experimental` namespace instead of a final facade deletion in this wave so later wallet and claim migrations can land before the full legacy cutover in `025-05`.
- 📌 Preserved the wallet facade and existing wallet zkpack vectors as the canonical production contract instead of re-blessing the custom crypto pack helpers.

## Deviations from Plan

### Auto-fixed Issues

📌 **1. [Rule 3 - Blocking Issue] Replaced the first test version after it reached into a private scalar RNG helper**

- 📌 Found during: Task 1 test compile validation
- 📌 Issue: the first integration test draft attempted to call the private `Z00ZScalar::random_from_rng` helper, which is not accessible from crate integration tests.
- 📌 Fix: removed the private-helper assertion and replaced it with public `Z00ZScalar::try_from_hash` regression coverage.
- 📌 Files modified: `crates/z00z_crypto/tests/test_fail_closed.rs`
- 📌 Verification: `cargo test -p z00z_crypto --release --features test-fast test_fail_closed -- --nocapture`

---

📌 Total deviations: 1 auto-fixed issue

📌 Impact on plan: the fix kept the regression coverage on the stable public surface and avoided widening visibility for internal RNG helpers.

## Issues Encountered

- 📌 Codacy analysis passed on the edited Rust source files and reported only a pre-existing large-function warning in `crates/z00z_crypto/src/lib.rs`.
- 📌 Codacy analysis for `crates/z00z_crypto/tests/test_fail_closed.rs` failed because `.codacy/cli.sh install` did not complete, so the new test file could not be analyzed through that tool in this execution slice.
- 📌 A background terminal status briefly surfaced unrelated `z00z_core` diagnostics, but the completed verification run returned exit code `0` with a green tail across the crypto and zkpack suites.

## Verification Evidence

- 📌 `bash ./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- 📌 `cargo test -p z00z_crypto --release --features test-fast test_fail_closed -- --nocapture`
- 📌 `cargo test -p z00z_crypto --release --features test-fast`
- 📌 `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump test_zkpack -- --nocapture`
- 📌 `cargo test --release --features test-fast --features wallet_debug_dump`

## User Setup Required

📌 None.

## Next Phase Readiness

- 📌 Later stealth-binding work can now consume fail-closed crypto helpers instead of auditing compatibility fallbacks again.
- 📌 Wallet and simulator migration can now treat the wallet AEAD facade as the single production zkpack reference while the custom crypto surface remains visibly experimental.
- 📌 The final legacy-surface cleanup in `025-05` now has an explicit intermediate boundary instead of a mixed production-facing facade.

## Known Stubs

📌 None.

## Self-Check

📌 PASSED - the summary file exists, `test_fail_closed.rs` exists, the stable crypto facade now exports fail-closed helper names, the custom zkpack path is nested under `experimental`, and the recorded release-style verification commands completed successfully in this execution slice.

---

📌 Phase: 025-crypto-audit-crypto

📌 Completed: 2026-03-27
