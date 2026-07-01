---
phase: 025-crypto-audit-crypto
plan: 05
subsystem: facade-cleanup
tags: [claim_gate, public_surface, experimental_zkpack, legacy_claim_v1, release-verification]
requires:
  - phase: 025-02
    provides: initial experimental boundary for custom zkpack helpers
  - phase: 025-04
    provides: live wallet and simulator claim flows already on claim_v2 and authoritative source proofs
provides:
  - default z00z_crypto facade centered on claim_v2 exports only
  - explicit feature gates for legacy claim_v1 and custom crypto zkpack helper paths
  - public-surface regression coverage for facade hygiene and README wiring
affects: [phase-025-closeout, public-api, docs, compatibility-boundaries]
tech-stack:
  added: []
  patterns: [feature-gated compatibility surface, source-of-truth README wiring, bootstrap-first public-surface regression]
key-files:
  created:
    - crates/z00z_crypto/tests/test_public_surface.rs
    - .planning/phases/025-crypto-audit-crypto/025-05-SUMMARY.md
  modified:
    - crates/z00z_crypto/Cargo.toml
    - crates/z00z_crypto/src/lib.rs
    - crates/z00z_crypto/src/claim/mod.rs
    - crates/z00z_crypto/src/aead.rs
    - crates/z00z_crypto/src/README.md
key-decisions:
  - Keep claim_v2 as the only default claim surface and move claim_v1 behind one explicit `legacy-claim-v1` compatibility feature.
  - Keep `ZkPackEncrypted` as the stable wire contract on the default facade, but move custom crypto zkpack seal or open helpers behind `experimental-zkpack` instead of leaving a public bypass in `aead.rs`.
  - Point Cargo package metadata and rustdoc at the same `src/README.md` so the public-surface contract is documented in one source of truth.
requirements-completed: [PH25-CLAIM-GATE]
duration: multi-session
completed: 2026-03-28
---

# Phase 025 Plan 05: Default Facade Cleanup Summary

📌 The default `z00z_crypto` facade now advertises production `claim_v2` contracts and the stable `ZkPackEncrypted` wire type, while legacy claim_v1 helpers and custom crypto zkpack seal or open helpers are accessible only through explicit feature gates.

## Performance

- 📌 Duration: multi-session
- 📌 Completed: 2026-03-28
- 📌 Tasks: 2
- 📌 Files modified: 6

## Accomplishments

- 📌 Added `legacy-claim-v1` and `experimental-zkpack` feature gates in `crates/z00z_crypto/Cargo.toml` and split the root claim exports so the default facade keeps only `claim_v2` types.
- 📌 Gated both the public `claim` compatibility re-exports and the legacy `claim_v1` module declarations, so placeholder claim helpers no longer compile into the default production build.
- 📌 Removed the lingering custom-zkpack bypass by gating `aead::zkpack` and its `open_zkpack`, `seal_zkpack`, and `Pack` re-exports behind `experimental-zkpack`.
- 📌 Switched rustdoc and package metadata to the same `src/README.md` contract, which now documents `claim_v2`, `legacy-claim-v1`, `experimental-zkpack`, and the wallet `ChaCha20-Poly1305` facade as the blessed production seal or open path.
- 📌 Added `test_public_surface.rs` as a fast regression tripwire that checks feature-gate wiring, README alignment, and the absence of the old `aead_zkpack` default alias.

## Task Commits

📌 No git commits were created in this execution slice.

📌 The workspace remains broadly dirty with unrelated changes, so any later git fixation should use the repo-owned versioning workflow only after a deliberate staging boundary is chosen.

## Files Created/Modified

- 📌 `crates/z00z_crypto/Cargo.toml` now defines the explicit compatibility and experimental gates and points package metadata at `src/README.md`.
- 📌 `crates/z00z_crypto/src/lib.rs` now re-exports only production `claim_v2` types by default and keeps legacy claim helpers behind `legacy-claim-v1`.
- 📌 `crates/z00z_crypto/src/claim/mod.rs` gates legacy claim_v1 module declarations and public re-exports behind the same compatibility feature.
- 📌 `crates/z00z_crypto/src/aead.rs` gates the custom crypto zkpack helper surface so the default `aead` module no longer provides an unrestricted experimental bypass.
- 📌 `crates/z00z_crypto/src/README.md` now states the default production surface explicitly and distinguishes the stable wire contract from the feature-gated experimental helpers.
- 📌 `crates/z00z_crypto/tests/test_public_surface.rs` guards the feature-gate and README wiring expected by this phase.

## Decisions Made

- 📌 Kept `ZkPackEncrypted` on the stable facade because the audit target was the production sealing boundary, not the canonical encrypted wire container.
- 📌 Gated legacy modules with `any(test, doctest, feature = ...)` so internal tests and doctests still compile while the default production build stays clean.
- 📌 Treated `src/README.md` as the single public-surface source of truth and aligned Cargo metadata to it instead of maintaining parallel README contracts.

## Deviations from Plan

### Auto-fixed Issues

📌 **1. [Rule 3 - Blocking Issue] Fixed internal legacy claim imports after the public re-export gate landed**

- 📌 Found during: Task 1 verification
- 📌 Issue: `claim/prover.rs` and `claim/verifier.rs` still imported legacy claim types through the public `claim` facade, so once the re-exports were gated the internal modules no longer compiled.
- 📌 Fix: rewired those imports to the local `proof` module and then gated the legacy module declarations themselves behind the compatibility feature.
- 📌 Files modified: `crates/z00z_crypto/src/claim/prover.rs`, `crates/z00z_crypto/src/claim/verifier.rs`, `crates/z00z_crypto/src/claim/mod.rs`
- 📌 Verification: `cargo test -p z00z_crypto --release --features test-fast`

📌 **2. [Rule 1 - Verification Trap] Fixed the new targeted public-surface test so the plan's cargo filter actually executed it**

- 📌 Found during: Task 1 targeted verification
- 📌 Issue: the first version of `test_public_surface.rs` used test names that did not match the plan's `cargo test ... test_public_surface` filter, so the command built the binary but ran zero targeted tests.
- 📌 Fix: renamed both test functions to carry the `test_public_surface` prefix.
- 📌 Files modified: `crates/z00z_crypto/tests/test_public_surface.rs`
- 📌 Verification: `cargo test -p z00z_crypto --release --features test-fast test_public_surface -- --nocapture`

📌 **3. [Rule 1 - API Leak] Closed the remaining custom-zkpack bypass found by review passes**

- 📌 Found during: repeated review passes
- 📌 Issue: even after the root facade cleanup, the default public `aead` module still exposed `zkpack`, `open_zkpack`, `seal_zkpack`, and `Pack`, which left `experimental-zkpack` as an alias instead of a real boundary.
- 📌 Fix: gated the `aead::zkpack` module and its public re-exports behind `experimental-zkpack`, and aligned Cargo README metadata with the updated `src/README.md` contract.
- 📌 Files modified: `crates/z00z_crypto/src/aead.rs`, `crates/z00z_crypto/Cargo.toml`, `crates/z00z_crypto/src/README.md`, `crates/z00z_crypto/tests/test_public_surface.rs`
- 📌 Verification: `cargo test -p z00z_crypto --release --features test-fast test_public_surface -- --nocapture` plus the full release stack below

---

📌 Total deviations: 3 auto-fixed issues

📌 Impact on plan: all three fixes were necessary to make the explicit feature gates real instead of nominal and to ensure the plan's targeted regression command exercised a meaningful guard.

## Issues Encountered

- 📌 The initial cleanup removed legacy root exports cleanly, but review passes correctly found that the old custom-zkpack surface was still reachable through `z00z_crypto::aead`.
- 📌 The targeted public-surface test started life as a source-text guard only; it remains a fast tripwire for this phase, not a full compile-time negative surface harness.
- 📌 One final architecture review still noted the broader longstanding root facade of concrete Tari backend re-exports in `lib.rs`; that is outside the specific `claim_v1` and custom-zkpack scope closed by Phase 025.

## Verification Evidence

- 📌 `bash ./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- 📌 `cargo test -p z00z_crypto --release --features test-fast test_public_surface -- --nocapture`
- 📌 `cargo test -p z00z_crypto --release --features test-fast`
- 📌 `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump test_claim_tx -- --nocapture`
- 📌 `cargo test --release --features test-fast --features wallet_debug_dump`
- 📌 Repeated review passes on the touched facade files; two consecutive passes returned `no significant issues` before closure.

## User Setup Required

📌 None.

## Next Phase Readiness

- 📌 Phase 025 is closed: live claim paths already use authoritative claim_v2 semantics, fail-open helpers are gone from the stable surface, stealth bindings are crypto-owned, and the default facade no longer advertises placeholder claim_v1 or custom zkpack helper APIs.
- 📌 The next follow-up, if desired, is outside this phase: reduce the remaining broad Tari concrete re-export surface in `z00z_crypto::lib.rs` or prepare a clean repo-owned versioning snapshot.

## Known Stubs

📌 None.

## Self-Check

📌 PASSED - the summary file exists, `PH25-CLAIM-GATE` is now complete, `test_public_surface` executes two passing targeted tests, the final bootstrap-first release stack completed with exit code `0`, and the roadmap plus state now close Phase 025 end-to-end.

---

📌 Phase: 025-crypto-audit-crypto

📌 Completed: 2026-03-28
