---
phase: 025-crypto-audit-crypto
plan: 01
subsystem: crypto-storage
tags: [claim_v2, claim_source_root, storage_proof, checkpoint_metadata, release-verification]
requires:
  - phase: 021-refactor-continue
    provides: release-style simulator and workspace gates already aligned to the current branch state
provides:
  - typed claim_v2 statement, signature, and source-proof contracts in z00z_crypto
  - storage-owned authoritative claim-source root and proof retrieval APIs in z00z_storage
  - checkpoint publication metadata carrying trusted claim-root state
affects: [025-04, 025-05, claim_flow, checkpoint_pipeline]
tech-stack:
  added: []
  patterns: [typed versioned claim contracts, storage-owned proof seam, release-first verification]
key-files:
  created:
    - crates/z00z_crypto/src/claim/v2.rs
    - crates/z00z_crypto/tests/test_claim_v2_contract.rs
    - crates/z00z_storage/tests/test_claim_source_proof.rs
    - .planning/phases/025-crypto-audit-crypto/025-01-SUMMARY.md
  modified:
    - crates/z00z_crypto/src/claim/mod.rs
    - crates/z00z_crypto/src/lib.rs
    - crates/z00z_storage/src/assets/types.rs
    - crates/z00z_storage/src/assets/store.rs
    - crates/z00z_storage/src/assets/mod.rs
    - crates/z00z_storage/src/checkpoint/artifact.rs
    - crates/z00z_storage/src/checkpoint/build.rs
    - crates/z00z_storage/src/checkpoint/ids.rs
    - crates/z00z_storage/tests/test_checkpoint_ids.rs
key-decisions:
  - Keep claim_v2 as a new explicit contract module instead of widening claim_v1 compatibility surfaces.
  - Keep authoritative claim-source roots and proof retrieval storage-owned, with crypto consuming typed contracts rather than inventing root semantics.
  - Publish trusted claim-root metadata through checkpoint draft, pub-in, and artifact surfaces as optional state carried from the storage root.
patterns-established:
  - Claim statement framing, root-version policy, and source-proof-version policy are frozen before wallet and simulator migration.
  - Storage proof ownership can be exposed without coupling z00z_storage directly to higher-level wallet migration logic.
requirements-completed: [PH25-CLAIM-V2, PH25-SOURCE-PROOF]
duration: multi-session
completed: 2026-03-27
---

# Phase 025 Plan 01: Claim V2 And Source-Proof Contracts Summary

📌 Claim_v2 now has a concrete typed contract surface in z00z_crypto, and z00z_storage now owns the authoritative claim-source root and proof seam needed for later wallet and simulator migration.

## Performance

- 📌 Duration: multi-session
- 📌 Completed: 2026-03-27
- 📌 Tasks: 2
- 📌 Files modified: 12

## Accomplishments

- 📌 Added `ClaimStmtV2`, `ClaimAuthoritySigV2`, `ClaimSourceProof`, version markers, and typed error handling in `z00z_crypto` so later claim migrations no longer need to rediscover canonical framing.
- 📌 Added storage-owned `ClaimSourceRoot`, `claim_source_root()`, and `claim_source_proof()` so the authoritative claim-source root is produced by storage instead of by wallet or simulator placeholder logic.
- 📌 Extended checkpoint draft, public-input, and artifact surfaces with optional claim-root metadata populated from the checkpoint post-state root.
- 📌 Added focused regression coverage for claim_v2 contract framing and storage claim-source root/proof roundtrips.
- 📌 Closed the one post-implementation regression by updating stale checkpoint artifact test fixtures to the new optional `claim_root` field shape.

## Task Commits

📌 No git commits were created in this execution slice.

📌 The implementation was completed and verified in the current workspace state without creating task-local commits.

## Files Created/Modified

- 📌 `crates/z00z_crypto/src/claim/v2.rs` defines the new claim_v2 vocabulary, canonical statement framing helpers, version tags, and signature verification path.
- 📌 `crates/z00z_crypto/src/claim/mod.rs` wires the new module into the claim facade.
- 📌 `crates/z00z_crypto/src/lib.rs` re-exports the new production-facing claim_v2 types.
- 📌 `crates/z00z_crypto/tests/test_claim_v2_contract.rs` freezes contract framing and version-rejection behavior.
- 📌 `crates/z00z_storage/src/assets/types.rs` introduces `ClaimSourceRoot` as the storage-owned authoritative root contract.
- 📌 `crates/z00z_storage/src/assets/store.rs` adds the authoritative root getter and typed source-proof retrieval seam.
- 📌 `crates/z00z_storage/src/assets/mod.rs` exposes the new storage-owned claim root type.
- 📌 `crates/z00z_storage/src/checkpoint/artifact.rs` carries optional trusted claim-root metadata through checkpoint publication surfaces.
- 📌 `crates/z00z_storage/src/checkpoint/build.rs` auto-populates claim-root metadata from the checkpoint `new_root` value.
- 📌 `crates/z00z_storage/tests/test_claim_source_proof.rs` verifies claim-source root bytes and proof retrieval behavior.
- 📌 `crates/z00z_storage/src/checkpoint/ids.rs` updates the internal artifact-shell test fixture to the current checkpoint wire shape.
- 📌 `crates/z00z_storage/tests/test_checkpoint_ids.rs` updates the integration artifact-shell fixture to the current checkpoint wire shape.

## Decisions Made

- 📌 Chose a new `claim/v2.rs` contract layer instead of extending placeholder `claim_v1` types, keeping staged migration explicit.
- 📌 Reused existing Tari Schnorr helpers and existing storage proof primitives instead of adding a new signature or proof dependency.
- 📌 Kept storage-owned proof semantics crate-local by exposing typed root/proof APIs and optional checkpoint metadata rather than introducing a second proof stack.

## Deviations from Plan

### Auto-fixed Issues

📌 **1. [Rule 3 - Blocking Issue] Fixed claim_v2 signer and verifier wrapper-type mismatches during implementation**

- 📌 Found during: Task 1 compile validation
- 📌 Issue: the first `claim_v2` implementation passed raw Tari secret/public key types into helpers that require `Z00ZScalar` and `Z00ZRistrettoPoint` wrappers.
- 📌 Fix: converted the signing secret with `Z00ZScalar::from_ristretto_secret_key`, derived the auth public key with `Z00ZRistrettoPoint::from_secret_key`, and verified against the wrapper-owned public key.
- 📌 Files modified: `crates/z00z_crypto/src/claim/v2.rs`
- 📌 Verification: `cargo test -p z00z_crypto --release --features test-fast`

📌 **2. [Rule 1 - Regression Fallout] Repaired stale checkpoint artifact-shell fixtures after adding optional claim-root metadata**

- 📌 Found during: Task 2 storage verification
- 📌 Issue: two checkpoint ID tests still serialized the pre-claim-root artifact shape, so bincode deserialization failed early with `UnexpectedEnd` instead of reaching the intended version-rejection checks.
- 📌 Fix: extended both internal and integration `ArtWire` fixtures with `claim_root: Option<ClaimSourceRoot>` and set them to `None` to match the new optional artifact field.
- 📌 Files modified: `crates/z00z_storage/src/checkpoint/ids.rs`, `crates/z00z_storage/tests/test_checkpoint_ids.rs`
- 📌 Verification: `cargo test -p z00z_storage --release --features test-fast`

---

📌 Total deviations: 2 auto-fixed issues

📌 Impact on plan: both fixes were required to make the new contract layer compile cleanly and to keep the checkpoint artifact test harness aligned with the new optional metadata surface.

## Issues Encountered

- 📌 The new optional `claim_root` field changed the checkpoint artifact shell shape, which invalidated two manually maintained test fixtures.
- 📌 Codacy analysis on the edited Rust files found no new actionable issues; only pre-existing file-size and complexity warnings remained on unrelated large files.
- 📌 Full release-style workspace verification completed successfully in this execution slice, so no out-of-scope workspace blocker was needed to close this plan.

## Verification Evidence

- 📌 `bash ./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- 📌 `cargo test -p z00z_crypto --release --features test-fast`
- 📌 `cargo test -p z00z_storage --release --features test-fast checkpoint::ids::tests::test_unsupported_version_rejects_art_id -- --exact`
- 📌 `cargo test -p z00z_storage --release --features test-fast`
- 📌 `cargo test --release --features test-fast --features wallet_debug_dump`

## User Setup Required

📌 None.

## Next Phase Readiness

- 📌 Wallet and simulator claim flows now have one concrete claim_v2 vocabulary to migrate onto in later phase-025 plans.
- 📌 Storage now exposes the authoritative root/proof seam needed to remove `ZERO_ROOT` and other placeholder claim-source assumptions from live consumers.
- 📌 Checkpoint publication already carries optional trusted claim-root metadata, so later consumers can ingest that trusted state without inventing a second publication contract.

## Known Stubs

📌 None.

## Self-Check

📌 PASSED - the summary file exists, the new claim_v2 and storage proof tests are present, and the recorded bootstrap, targeted crate, and full release-style workspace gates completed successfully in this execution slice.

---

📌 Phase: 025-crypto-audit-crypto

📌 Completed: 2026-03-27
