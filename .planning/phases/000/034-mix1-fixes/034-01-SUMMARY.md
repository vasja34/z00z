---
phase: 034-mix1-fixes
plan: 01
subsystem: claim-continuity
tags: [claim-source, storage-membership, simulator, wallet, fail-closed]
requires:
  - phase: 034-01
    provides: persisted claim-source contract seam in storage
  - phase: 034-02
    provides: migrated producer and verifier callers on one storage-backed authority path
provides:
  - Persisted-membership-backed claim_source_contract_for_item authority seam
  - Migrated claim package producer and consumer using bundle-backed canonical membership
  - Wallet claim verifier bound to carried claim_source_proof root
  - Empty-bundle fail-closed rejection across patch, write, and load paths
affects: [034-03, PH34-CLAIM-CONTINUITY, scenario-1-claim-flow]
tech-stack:
  added: []
  patterns: [storage-backed authority seam, bundle-backed claim membership, fail-closed package canonicalization]
key-files:
  created:
    - /home/vadim/Projects/z00z/.planning/phases/034-mix1-fixes/034-01-SUMMARY.md
  modified:
    - /home/vadim/Projects/z00z/crates/z00z_storage/src/assets/store_internal/store_query.rs
    - /home/vadim/Projects/z00z/crates/z00z_storage/tests/test_claim_source_proof.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/test_claim_tx.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/src/claim_pkg_consumer.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_3_runtime.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_claim_pkg_runtime.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_stage3_nullifier_store.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_stage4_claim_gate.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/tests/support/test_s5_sender_examples_support.rs
key-decisions:
  - "Make persisted store membership the only authoritative claim-source seam instead of re-deriving a helper-owned one-item tree."
  - "Use one bundle-backed canonical membership pass for producer patching and consumer verification so simulator and wallet paths consume the same proof truth."
  - "Reject empty claim bundles at canonicalization, write, and load boundaries rather than relying on downstream drift checks."
patterns-established:
  - "Live claim-source roots now come from persisted membership state or the carried claim_source_proof derived from that state, never from local synthetic reconstruction."
requirements-completed: [PH34-CLAIM-CONTINUITY]
completed: 2026-04-09
revision: 8a595297
reviewed: 2026-04-09T18:44:21Z
---

# Phase 034 Plan 01 Summary

## Outcome

Plan 01 is complete. The helper-owned claim-source authority path is retired from live storage, wallet, and simulator seams, and claim continuity now closes on one storage-backed membership contract.

## Accomplishments

- Replaced `AssetStore::claim_source_contract_for_item(...)` with a persisted-membership-backed seam that rejects missing membership and path drift fail closed.
- Migrated live claim producer and verifier callers so bundle patching, persisted claim packages, simulator verification, and wallet verification all consume the same canonical proof truth.
- Removed the remaining helper-owned single-package authority fallback by forcing `build_claim_package(...)` through `patch_claim_bundle_membership(...)` before serialization.
- Added storage regressions for positive persisted-membership roundtrip, missing-membership rejection, stale-item drift rejection, and synthetic one-item non-authority.
- Added Stage 4 regressions that freeze empty-bundle rejection at the patch, write, and load boundaries.

## Verification

- `cargo test -p z00z_storage --release --test test_claim_source_proof -- --nocapture` passed with 6 tests green.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_pkg_crypto_support -- --nocapture` passed with 12 tests green.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_pkg_runtime -- --nocapture` passed with 14 tests green.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_claim_gate -- --nocapture` passed with 7 tests green.
- The scoped Phase 034 review loop converged with two consecutive `No findings` passes in `/home/vadim/Projects/z00z/.planning/phases/034-mix1-fixes/034-REVIEW.md`.

## Files Created Or Modified

- `/home/vadim/Projects/z00z/crates/z00z_storage/src/assets/store_internal/store_query.rs` now derives claim-source proof material from persisted membership already present in the store.
- `/home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` now binds the claim statement source root to the carried `claim_source_proof` instead of local synthetic reconstruction.
- `/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs` now canonicalizes packages through bundle-backed membership patching and rejects empty bundles fail closed.
- `/home/vadim/Projects/z00z/crates/z00z_simulator/src/claim_pkg_consumer.rs` now verifies each carried package against the same bundle-backed canonical membership contract and rejects empty bundles on load.
- `/home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_stage4_claim_gate.rs` freezes the negative-path regressions for bad packages, missing bundles, and empty bundles.

## Issues Encountered

- An initial targeted validation attempt used filters that returned mostly `filtered out` results; that evidence was rejected and replaced with exact integration-binary runs.
- The new empty-bundle regressions in `test_stage4_claim_gate.rs` were first inserted as inner items and therefore did not execute; the misplaced brace was fixed before the final validation run.
- A final scoped review found that `patch_claim_bundle_membership(...)` still accepted empty input; the helper now rejects empty bundles directly, closing the remaining fail-open seam.

## Next Phase Readiness

- `034-03` can start from a clean claim-continuity baseline: persisted claim membership is authoritative, helper fallback authority is removed from live seams, and the phase review loop for Plan 01 is clean.
- Phase documentation reclassification remains blocked until the spend-nullifier and checkpoint-backend chains complete under later plans.

## Known Stubs

None for the Plan 01 claim-continuity scope.

## Threat Flags

None. The active claim seam is now fail closed on missing membership, path drift, source-root drift, proof drift, malformed bundles, and empty bundles.

## Self-Check

PASSED.

---
*Phase: 034-mix1-fixes*
*Completed: 2026-04-09*
