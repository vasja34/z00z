---
phase: 025-crypto-audit-crypto
plan: 03
subsystem: stealth-binding
tags: [stealth_bind, tag16, leaf_ad, range_ctx_hash, wallet_migration, release-verification]
requires:
  - phase: 025-02
    provides: fail-closed crypto helpers and the narrowed production zkpack boundary already exist on the active branch
provides:
  - crypto-owned canonical `tag16` and `leaf_ad` helpers preserved with wallet-production outputs
  - a frozen `range_ctx_hash` contract for later claim_v2 and tx-digest migration
  - wallet scan and tx consumers migrated onto one shared crypto binding surface
affects: [025-04, 025-05, wallet_scan, wallet_tx, claim_flow]
tech-stack:
  added: []
  patterns: [crypto-owned binding contract, production-compatible domain preservation, frozen vector guards]
key-files:
  created:
    - crates/z00z_crypto/src/stealth_bind.rs
    - crates/z00z_crypto/tests/test_stealth_bind_vectors.rs
    - .planning/phases/025-crypto-audit-crypto/025-03-SUMMARY.md
  modified:
    - crates/z00z_crypto/src/domains.rs
    - crates/z00z_crypto/src/hash.rs
    - crates/z00z_crypto/src/lib.rs
    - crates/z00z_crypto/src/claim/v2.rs
    - crates/z00z_crypto/tests/test_hash_policy.rs
    - crates/z00z_crypto/tests/domain_separation_tests.rs
    - crates/z00z_wallets/src/core/stealth/tag.rs
    - crates/z00z_wallets/src/core/address/stealth_scan_support.rs
    - crates/z00z_wallets/src/core/tx/builder.rs
    - crates/z00z_wallets/src/core/tx/output_flow.rs
    - crates/z00z_core/src/assets/wire_pkg.rs
key-decisions:
  - Preserve the wallet-production `tag16` and `leaf_ad` domain strings exactly inside the new crypto-owned helper layer instead of redirecting callers to the older generic crypto KDF helper.
  - Introduce `range_ctx_hash` now as a frozen crypto binding seam, but record only DST and framing stability rather than claiming proven circuit equivalence.
  - Keep wallet-facing `tag.rs` as a compatibility shim while moving canonical ownership into `z00z_crypto` so live call sites can migrate incrementally.
patterns-established:
  - Wallet-visible stealth formulas can be centralized in crypto without changing live outputs if the production domain strings and framing are preserved exactly.
  - Binding helpers that later feed claim or tx digests must ship with mutation-sensitive vectors and explicit domain-registry coverage in the same wave.
requirements-completed: [PH25-STEALTH-BIND]
duration: multi-session
completed: 2026-03-27
---

# Phase 025 Plan 03: Crypto-Owned Stealth Binding Summary

📌 `tag16`, `leaf_ad`, and the new `range_ctx_hash` binding contract now live under `z00z_crypto`, while wallet scan and tx consumers share that one canonical implementation without changing the production outputs they already depended on.

## Performance

- 📌 Duration: multi-session
- 📌 Completed: 2026-03-27
- 📌 Tasks: 2
- 📌 Files modified: 12

## Accomplishments

- 📌 Added `crates/z00z_crypto/src/stealth_bind.rs` with canonical `compute_tag16`, `compute_leaf_ad`, `encode_leaf_preimage`, `range_ctx_hash`, and `LEAF_PREIMAGE_SIZE` exports.
- 📌 Added `StealthTag16ProdDomain`, `StealthLeafAdProdDomain`, and `RangeCtxDomain` so the new helper layer preserves the wallet-production DST values exactly and registers the new range-context binding domain explicitly.
- 📌 Re-exported the stealth-binding surface from `z00z_crypto::lib.rs` and migrated wallet scan and tx consumers to import the crypto-owned helpers directly.
- 📌 Reduced `crates/z00z_wallets/src/core/stealth/tag.rs` to a wallet-facing compatibility shim, keeping only the request-bound helper locally while removing wallet ownership of the canonical formula bodies.
- 📌 Added frozen vectors in `crates/z00z_crypto/tests/test_stealth_bind_vectors.rs` plus domain-registry and framing guards in the hash-policy and domain-separation test suites.
- 📌 Added `output_range_ctx_hash` as the stable downstream seam so the upcoming claim_v2 migration can use the crypto-owned binding helper without rediscovering where it lives.

## Task Commits

📌 No git commits were created in this execution slice.

📌 The workspace still contains unrelated pre-existing diffs, so git fixation remains deferred until the repo-owned versioning workflow can stage a clean boundary.

## Files Created/Modified

- 📌 `crates/z00z_crypto/src/stealth_bind.rs` is the new canonical stealth-binding module.
- 📌 `crates/z00z_crypto/src/domains.rs` adds the preserved wallet stealth domains and the new `RangeCtxDomain` registry entry.
- 📌 `crates/z00z_crypto/src/hash.rs` updates the domain registry tables so `Z00Z/RANGECTX` is frozen in the public hash-policy surface.
- 📌 `crates/z00z_crypto/src/lib.rs` re-exports the stealth-binding helpers for live consumers.
- 📌 `crates/z00z_crypto/src/claim/v2.rs` documents the intended fill path for `range_ctx_hash` in the later claim migration wave.
- 📌 `crates/z00z_crypto/tests/test_stealth_bind_vectors.rs` freezes golden vectors for `tag16`, `leaf_ad`, preimage layout, and range-context mutation sensitivity.
- 📌 `crates/z00z_crypto/tests/test_hash_policy.rs` and `crates/z00z_crypto/tests/domain_separation_tests.rs` freeze the new domain-sensitive framing.
- 📌 `crates/z00z_wallets/src/core/stealth/tag.rs`, `crates/z00z_wallets/src/core/address/stealth_scan_support.rs`, `crates/z00z_wallets/src/core/tx/builder.rs`, and `crates/z00z_wallets/src/core/tx/output_flow.rs` now consume the crypto-owned helpers.
- 📌 `crates/z00z_core/src/assets/wire_pkg.rs` was updated during verification so the workspace release gate could consume the typed zkpack wire API introduced in the previous plan.

## Decisions Made

- 📌 Preserved the wallet-production domain strings exactly instead of redirecting to the existing generic `LeafAdDomain`, because changing the DST would have broken live wallet vectors and scan compatibility.
- 📌 Added `range_ctx_hash` as a new explicit binding seam now, but limited the claim in this wave to frozen framing and mutation coverage rather than asserting native-to-circuit equivalence without evidence.
- 📌 Kept the wallet compatibility facade in place so downstream call sites can migrate without reviving duplicated formulas.

## Deviations from Plan

### Auto-fixed Issues

📌 **1. [Rule 3 - Blocking Issue] Opened the leaf-preimage helper to integration tests after the first vector test hit a private `#[cfg(test)]` boundary**

- 📌 Found during: Task 1 test compile validation
- 📌 Issue: the first integration-test version could not import `encode_leaf_preimage` because the helper was still hidden behind a test-only gate in the library.
- 📌 Fix: removed the `#[cfg(test)]` gate, re-exported `encode_leaf_preimage` and `LEAF_PREIMAGE_SIZE`, and kept the helper as part of the explicit frozen binding contract.
- 📌 Files modified: `crates/z00z_crypto/src/stealth_bind.rs`, `crates/z00z_crypto/src/lib.rs`
- 📌 Verification: `cargo test -p z00z_crypto --release --features test-fast test_stealth_bind_vectors -- --nocapture`

📌 **2. [Rule 3 - Blocking Issue] Repaired stale `ZkPackEncrypted` call sites that still treated typed wire APIs as `Option` values**

- 📌 Found during: release-style workspace verification
- 📌 Issue: `crates/z00z_core/src/assets/wire_pkg.rs` still called `.ok_or_else(...)` on `ZkPackEncrypted::from_bytes(...)` and `to_bytes(...)` after the phase-025 fail-closed wire change had moved those APIs to `Result`.
- 📌 Fix: converted the parse and serialization paths to `map_err(...)`-based `Result` handling so the workspace release gate could compile through the updated wire contract.
- 📌 Files modified: `crates/z00z_core/src/assets/wire_pkg.rs`
- 📌 Verification: `cargo test --release --features test-fast --features wallet_debug_dump`

---

📌 Total deviations: 2 auto-fixed issues

📌 Impact on plan: both fixes were required to make the new crypto binding surface testable and to let the release-style workspace gate compile against the typed zkpack wire contract already established in `025-02`.

## Issues Encountered

- 📌 The existing generic crypto `derive_leaf_ad` path was not semantically compatible with the wallet-production stealth formulas, so the ownership move required a dedicated helper module with preserved wallet DST values rather than a trivial re-export.
- 📌 The first release-style verification pass exposed stale `ZkPackEncrypted` `Option` call sites outside the immediate stealth-binding files.
- 📌 The new helper layer initially left one unused import and one missing-doc warning in wallet files; both were cleaned up before closure.

## Verification Evidence

- 📌 `bash ./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- 📌 `cargo test -p z00z_crypto --release --features test-fast test_stealth_bind_vectors -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump test_kdf -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump test_stealth_scanner_prefilter -- --nocapture`
- 📌 `cargo test --release --features test-fast --features wallet_debug_dump`

## User Setup Required

📌 None.

## Next Phase Readiness

- 📌 Wallet and simulator claim migration can now rely on one stable `range_ctx_hash` import path.
- 📌 Live scan and tx code already consume the crypto-owned `tag16` and `leaf_ad` surface, so later claim or digest work no longer needs to chase duplicated formula ownership.
- 📌 Domain-registry and vector coverage now make silent DST or framing drift visible before `025-04` starts wiring claim_v2 live paths.

## Known Stubs

📌 None.

## Self-Check

📌 PASSED - the summary file exists, the crypto-owned stealth-binding module and vector test file exist, wallet consumers call the crypto surface, `PH25-STEALTH-BIND` is complete, and the recorded bootstrap plus release-style verification commands completed successfully in this execution slice.

---

📌 Phase: 025-crypto-audit-crypto

📌 Completed: 2026-03-27
