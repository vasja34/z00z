---
phase: 030
plan: 22
subsystem: z00z_crypto roots and facade normalization
summary: Reduce the remaining oversized z00z_crypto implementation roots below the continuation band while preserving lib.rs as the single shallow crypto facade.
tags:
  - phase-030
  - z00z-crypto
  - facade
  - seams
  - range-proofs
  - kdf
  - ecdh
requirements-completed:
  - PH30-SEAMS
  - PH30-PROTECTED
  - PH30-VERIFY
affects:
  - crates/z00z_crypto/src
  - crates/z00z_crypto/tests
  - crates/z00z_wallets/tests/test_kdf.rs
provides:
  - Thin stable crypto roots with extracted sidecar seams under the continuation band
  - A shallow `z00z_crypto::lib.rs` facade that re-exports moved wrapper API from `lib_api.rs`
  - Hidden backend contract ownership split into focused backend metadata and trait seams
key_files:
  created:
    - crates/z00z_crypto/src/kdf_tests.rs
    - crates/z00z_crypto/src/types_validation.rs
    - crates/z00z_crypto/src/types_tests.rs
    - crates/z00z_crypto/src/aead_test_only.rs
    - crates/z00z_crypto/src/aead_transport.rs
    - crates/z00z_crypto/src/aead_zkpack.rs
    - crates/z00z_crypto/src/aead_tests.rs
    - crates/z00z_crypto/src/hash_typed.rs
    - crates/z00z_crypto/src/hash_domains.rs
    - crates/z00z_crypto/src/hash_policy.rs
    - crates/z00z_crypto/src/hash_zk.rs
    - crates/z00z_crypto/src/hash_convenience.rs
    - crates/z00z_crypto/src/hash_tests.rs
    - crates/z00z_crypto/src/hash_simple_tests.rs
    - crates/z00z_crypto/src/hash_hmac_rfc4231_tests.rs
    - crates/z00z_crypto/src/ecdh_tests.rs
    - crates/z00z_crypto/src/ecdh_stealth_tests.rs
    - crates/z00z_crypto/src/lib_api.rs
    - crates/z00z_crypto/src/backend/backend_info.rs
    - crates/z00z_crypto/src/backend/backend_trait.rs
  modified:
    - crates/z00z_crypto/src/aead.rs
    - crates/z00z_crypto/src/kdf.rs
    - crates/z00z_crypto/src/hash.rs
    - crates/z00z_crypto/src/types.rs
    - crates/z00z_crypto/src/lib.rs
    - crates/z00z_crypto/src/backend.rs
    - crates/z00z_crypto/src/ecdh.rs
    - crates/z00z_crypto/src/ecdh_stealth.rs
decisions:
  - Keep `z00z_crypto::lib.rs` as the only shallow caller-visible crypto facade and move the long wrapper API into `lib_api.rs` behind root-level re-exports.
  - Keep backend ownership hidden by turning `backend.rs` into a thin facade over `backend_info` and `backend_trait` instead of widening backend surface area.
  - Preserve canonical ownership boundaries by keeping AAD, transcript, and domain or KDF semantics out of `backend.rs`, `ecdh.rs`, and `ecdh_stealth.rs`.
metrics:
  duration: current-session
  completed_at: 2026-04-03
  tasks_completed: 2/2
---

# Phase 030 Plan 22: Crypto Continuation Split Summary

Reduced the remaining oversized `z00z_crypto` implementation roots below the continuation band while preserving `lib.rs` as the single shallow crypto facade and keeping backend or ECDH ownership hidden behind stable re-exports.

## Outcomes

- Task 1 closed the remaining crypto facade residue:
  - `kdf.rs`, `types.rs`, `aead.rs`, and `hash.rs` now stay under the continuation band through extracted sibling seams for validation, transport, zkpack, typed hashing, domain policy, and tests.
  - `lib.rs` now ends at line `360` and keeps only the shallow caller-visible facade plus `pub use lib_api::*` re-exports for the moved wrapper API.
  - `lib_api.rs` owns the long public wrapper functions for commitments, range proofs, hash derivation, Schnorr signing, and batch verification without changing the root public story.
- Task 2 closed the remaining backend and ECDH support residue:
  - `backend.rs` now ends at line `46` and delegates hidden ownership to `backend/backend_info.rs` and `backend/backend_trait.rs`.
  - `ecdh.rs` and `ecdh_stealth.rs` keep the same shallow support roots while their heavy tests live in extracted sidecar files.
- Canonical ownership stayed intact:
  - `z00z_crypto::lib.rs` remains the single shallow public crypto entry surface.
  - `backend.rs`, `ecdh.rs`, and `ecdh_stealth.rs` do not own AAD, transcript, or Merlin framing helpers.
  - Domain, KDF, AEAD, and hash ownership stay explicit in their canonical modules instead of fragmenting into new competing public roots.

## Verification

- File-level diagnostics were clean for:
  - `crates/z00z_crypto/src/lib.rs`
  - `crates/z00z_crypto/src/lib_api.rs`
  - `crates/z00z_crypto/src/backend.rs`
  - `crates/z00z_crypto/src/backend/backend_info.rs`
  - `crates/z00z_crypto/src/backend/backend_trait.rs`
- Confirmed root line-counts after the split:
  - `crates/z00z_crypto/src/aead.rs`: `214`
  - `crates/z00z_crypto/src/kdf.rs`: `289`
  - `crates/z00z_crypto/src/hash.rs`: `264`
  - `crates/z00z_crypto/src/types.rs`: `326`
  - `crates/z00z_crypto/src/lib.rs`: `360`
  - `crates/z00z_crypto/src/backend.rs`: `46`
  - `crates/z00z_crypto/src/ecdh.rs`: `293`
  - `crates/z00z_crypto/src/ecdh_stealth.rs`: `186`
- Executed verification commands:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo test -q -p z00z_crypto --release --test test_public_surface -- --nocapture`
  - `cargo test -q -p z00z_crypto --release --test test_hash_policy -- --nocapture`
  - `cargo test -q -p z00z_crypto --release --test test_domain_separation -- --nocapture`
  - `cargo test -q -p z00z_wallets --release --test test_kdf -- --nocapture`
  - `cargo test -q -p z00z_crypto --release --doc -- --nocapture`
  - `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run`
- Ownership guard passed:
  - no `build_aad|AAD_|transcript|Merlin` matches remained in `crates/z00z_crypto/src/backend.rs`, `crates/z00z_crypto/src/ecdh.rs`, or `crates/z00z_crypto/src/ecdh_stealth.rs`

## Deviations from Plan

### Auto-fixed Issues

1. `[Rule 3 - Blocking issue]` The live Phase 030 execution path stayed stuck in inline reconnaissance after `init execute-phase 030` reported `agents_installed: false`, so the continuation was completed manually inline instead of waiting for non-existent spawned executors.
2. `[Rule 1 - Bug]` Wiring `types.rs` to `types_validation.rs` initially failed because `ByteArray` was not imported in the extracted validation seam. The trait import was restored and diagnostics went green.
3. `[Rule 1 - Bug]` The extracted `aead_tests.rs` initially missed `CryptoError` in scope for match arms. The import was restored and the test seam compiled cleanly.
4. `[Rule 1 - Bug]` The first `hash.rs` extraction left duplicate inline and sidecar module definitions in place. The old inline modules were removed so the root became a truthful thin facade.
5. `[Rule 1 - Bug]` The initial `lib_api.rs` extraction missed the `CryptoBackend` trait import, and the new thin `backend.rs` still carried stale root imports. Both scope issues were corrected before final verification.

## Residual Risk

- The structural and verification gates for `030-22` are green, but repo-owned git closeout for this plan still needs to be done through the requested `z00z-git-versioning` workflow if the user wants this pass committed immediately.

## Self-Check: PASSED

- Summary file created at `.planning/phases/030-refactor-long-files/030-22-SUMMARY.md`
- All targeted root files named in `030-22-PLAN.md` are below the `>400` continuation band
- Targeted crypto verification and the max-safe gate completed successfully with exit code `0`
- `lib.rs` remains the single shallow public crypto facade while backend ownership stays hidden
