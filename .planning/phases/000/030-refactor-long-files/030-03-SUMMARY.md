---
phase: 030-refactor-long-files
plan: 03
subsystem: crypto
tags: [rust, crypto, facade, seams, tari]
requires:
  - phase: 029-crypto-audit-wallets
    provides: wallet-facing crypto constraints and release-style verification anchors
provides:
  - stable crypto facade with internal seam modules for hash, kdf, aead, support types, and Tari backend helpers
  - public-surface regression coverage for protected seam ownership
  - phase-local verification evidence for post-review crypto facade state
affects: [030-04, 030-05, 030-06, z00z_wallets, docs/code-review]
tech-stack:
  added: []
  patterns: [facade-owner seam extraction, private backend helper modules, public-surface regression gating]
key-files:
  created:
    - crates/z00z_crypto/src/blake2_hash.rs
    - crates/z00z_crypto/src/sha256_hash.rs
    - crates/z00z_crypto/src/hmac_sha256.rs
    - crates/z00z_crypto/src/secret_bytes.rs
    - crates/z00z_crypto/src/argon2_params.rs
    - crates/z00z_crypto/src/argon2_kdf.rs
    - crates/z00z_crypto/src/hkdf_kdf.rs
    - crates/z00z_crypto/src/aead_error.rs
    - crates/z00z_crypto/src/aead_primitives.rs
    - crates/z00z_crypto/src/aead_envelope.rs
    - crates/z00z_crypto/src/aead_aad.rs
    - crates/z00z_crypto/src/protocol_constants.rs
    - crates/z00z_crypto/src/crypto_constants.rs
    - crates/z00z_crypto/src/scalar_type.rs
    - crates/z00z_crypto/src/backend_init.rs
    - crates/z00z_crypto/src/backend_commitment.rs
    - crates/z00z_crypto/src/backend_range_proofs.rs
    - crates/z00z_crypto/src/backend_batch.rs
    - crates/z00z_crypto/src/backend_tari_tests.rs
    - docs/code-review/2026-03-31-phase-030-plan-03-task-2-crypto-facade-review.md
    - docs/code-review/2026-03-31-phase-030-plan-03-task-2-crypto-facade-review-pass-2.md
  modified:
    - crates/z00z_crypto/src/hash.rs
    - crates/z00z_crypto/src/kdf.rs
    - crates/z00z_crypto/src/aead.rs
    - crates/z00z_crypto/src/types.rs
    - crates/z00z_crypto/src/backend_tari.rs
    - crates/z00z_crypto/tests/test_public_surface.rs
    - reports/full_verify-report-long-running-tests.txt
key-decisions:
  - Keep `z00z_crypto::lib.rs` as the only top-level public crypto facade while moving internal logic into sibling seam files.
  - Keep `domains.rs` and `kdf_domains.rs` as canonical owners for domain-separation and KDF-info constants rather than creating parallel namespaces.
  - Move Tari backend tests into `backend_tari_tests.rs` so the production owner file shrinks without weakening coverage.
patterns-established:
  - "Facade-owner split: caller-visible modules stay shallow and re-export cohesive sibling seams."
  - "Protected crypto seams stay private unless a later normalization wave explicitly widens the surface."
requirements-completed: [PH30-PROTECTED, PH30-VERIFY]
duration: 2h 24m
completed: 2026-03-31
---

# Phase 030 Plan 03 Summary

📌 Stable `z00z_crypto` facade preserved while hash, KDF, AEAD, support-type, and Tari backend internals moved into cohesive private seam modules with public-surface regressions guarding ownership.

## Performance

- 📌 Duration: 2h 24m
- 📌 Started: 2026-03-31T08:34:52Z
- 📌 Completed: 2026-03-31T10:58:26Z
- 📌 Tasks: 2
- 📌 Files modified: 25

## Accomplishments

- 📌 Split `hash.rs`, `kdf.rs`, and `aead.rs` into semantic seam files while keeping the approved facade namespaces and wallet-facing behavior stable.
- 📌 Split `types.rs` and `backend_tari.rs` into support-type and backend-helper modules without exposing new public seams.
- 📌 Added public-surface regression coverage and completed the post-review verification stack on the final workspace state.

## Task Commits

📌 No git commit was created in this closeout because the current repository workflow only permits version-manager git operations on explicit request, and the worktree also contains unrelated dirty files outside Phase 030.

## Files Created/Modified

- `crates/z00z_crypto/src/hash.rs` - Reduced to the stable hash facade with sibling seam re-exports.
- `crates/z00z_crypto/src/kdf.rs` - Reduced to the stable KDF facade with sibling seam re-exports.
- `crates/z00z_crypto/src/aead.rs` - Reduced to the stable AEAD facade with sibling seam re-exports and truthful rustdoc anchor.
- `crates/z00z_crypto/src/types.rs` - Kept as the sole owner facade for extracted support constants and scalar wrapper types.
- `crates/z00z_crypto/src/backend_tari.rs` - Kept as the private backend owner delegating to helper seams.
- `crates/z00z_crypto/tests/test_public_surface.rs` - Added regression assertions for Task 1 and Task 2 seam ownership.
- `docs/code-review/2026-03-31-phase-030-plan-03-task-2-crypto-facade-review.md` - Review artifact for the Task 2 facade split.
- `docs/code-review/2026-03-31-phase-030-plan-03-task-2-crypto-facade-review-pass-2.md` - Clean follow-up review artifact confirming the protected facade shape.

## Decisions Made

- 📌 Keep the stable crypto facade shallow and move internal behavior into sibling seams instead of creating new public namespaces.
- 📌 Keep support constants and scalar wrappers behind `types.rs` so callers still consume one owner module.
- 📌 Treat the review-found local proof-size ceiling in `backend_range_proofs.rs` as a real drift bug and bind it back to the canonical `MAX_PROOF_SIZE_V1` owner constant.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Rebound single-proof verification to the canonical proof-size owner**

- **Found during:** Task 2 review loop
- **Issue:** `backend_range_proofs.rs` still carried a local proof-size ceiling instead of the extracted owner constant, which risked future drift.
- **Fix:** Imported and used `MAX_PROOF_SIZE_V1` from the canonical support-constant surface.
- **Files modified:** `crates/z00z_crypto/src/backend_range_proofs.rs`
- **Verification:** Re-ran targeted crypto and wallet anchors plus the release-style max-safe gate on the post-review workspace state.
- **Committed in:** not committed in this closeout

---

📌 Total deviations: 1 auto-fixed bug
📌 Impact on plan: The fix preserved the intended owner contract and did not widen scope beyond the Task 2 facade boundary.

## Issues Encountered

- 📌 The repository was already dirty outside Phase 030, so closeout must use staged-only version-manager flow instead of `--stage-all`.
- 📌 The captured verification log preserved the targeted anchor results clearly while the max-safe gate output remained noisy; the shell returned exit code 0 and refreshed `reports/full_verify-report-long-running-tests.txt` at `2026-03-31T13:55:55+03:00`.

## User Setup Required

📌 None - no external service configuration required.

## Next Phase Readiness

- 📌 Later Phase 030 normalization waves can clean consumer paths without revisiting crypto ownership boundaries introduced here.
- 📌 Wallet-facing KDF and public-facade anchors remain intact for downstream refactor waves.

## Verification

- 📌 `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- 📌 `cargo test -p z00z_crypto --release --test test_hash_policy -- --nocapture`
- 📌 `cargo test -p z00z_crypto --release --test test_public_surface -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --test test_kdf -- --nocapture`
- 📌 `cargo test -p z00z_crypto --release --features test-fast -- --nocapture`
- 📌 `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run`
- 📌 Review loop completed with one fix-bearing pass followed by two consecutive clean passes.

## Self-Check

📌 PASSED for summary creation, planning-state sync, key artifact existence, targeted post-review anchors, and the post-review `--max-safe-run` shell exit code.

📌 Git closeout intentionally left undone because no explicit commit or push request was given and the repository contains unrelated dirty files that must stay out of any staged release flow.

---
*Phase: 030-refactor-long-files*
*Completed: 2026-03-31*
