---
phase: 036-rename
plan: 23
subsystem: crypto
tags: [claim-contract, root-version, wallets, storage, simulator]
requires:
  - phase: 036-20
    provides: Partial shim-removal boundary that remains open and must not be over-claimed.
  - phase: 036-22
    provides: The latest self-contained Phase 036 continuation baseline before the claim rename wave.
provides:
  - Claim owner surface renamed to claim_contract with CLAIM_ROOT_VERSION as the only live root-version export.
  - Wallet, storage, and simulator claim consumers migrated to root_version and SourceRootVersion.
  - Truthful closeout evidence for the 21-row claim rename table and coverage-proof table.
affects: [036-rename, claim, wallets, storage, simulator]
tech-stack:
  added: []
  patterns: [raw-byte-claim-root-version, claim-contract-domain-label]
key-files:
  created: [crates/z00z_crypto/src/claim/claim_contract.rs, .planning/phases/036-rename/036-23-SUMMARY.md]
  modified: [crates/z00z_crypto/src/claim/mod.rs, crates/z00z_crypto/src/lib.rs, crates/z00z_crypto/tests/test_claim_contract.rs, crates/z00z_wallets/src/core/tx/claim_auth.rs, crates/z00z_wallets/src/core/tx/claim_tx_helpers.rs, crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs, crates/z00z_wallets/src/core/tx/claim_errors.rs, crates/z00z_wallets/src/core/tx/claim_tx.rs, crates/z00z_storage/src/assets/store_internal/store_query.rs, crates/z00z_storage/src/checkpoint/build.rs, crates/z00z_storage/src/assets/types_identity.rs, crates/z00z_simulator/src/claim_pkg_consumer.rs, crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs, crates/z00z_wallets/src/core/tx/test_claim_tx.rs, crates/z00z_storage/tests/test_claim_source_proof.rs]
key-decisions:
  - "Retired ClaimRootVer in favor of the raw-byte CLAIM_ROOT_VERSION constant while preserving CLAIM_TAG and claim proof framing."
  - "Scoped the compatibility-breaking claim_v2 -> claim_contract label migration to claim_stmt_hash and mirrored test expectations only."
  - "Fixed the omitted simulator claim_pkg_consumer downstream accessor as an in-scope compile blocker after the root_version rename."
patterns-established:
  - "Claim root-version ownership now lives on one exported u8 constant instead of a wrapper type."
  - "Claim source roots and proofs compare root_version as raw bytes while proof-version semantics stay unchanged."
requirements-completed: [None]
duration: not-recorded
completed: 2026-04-21
---

# Phase 036 Plan 23: Claim Contract Summary

Claim owner surface renamed to claim_contract with raw-byte root_version wiring across crypto, wallets, storage, and simulator claim consumers.

## Performance

- **Duration:** not recorded in this inline execution pass
- **Started:** not recorded
- **Completed:** 2026-04-21T14:51:29Z
- **Tasks:** 3
- **Files modified:** 18

## Accomplishments

- Replaced the crypto-owned `v2` claim module with `claim_contract.rs`, removed `ClaimRootVer`, exported `CLAIM_ROOT_VERSION`, and renamed the owned `root_ver` surface to `root_version`.
- Migrated the direct wallet, storage, and simulator claim consumers to `CLAIM_ROOT_VERSION`, `root_version`, and `SourceRootVersion` without changing `ClaimProofVer`, `CLAIM_TAG`, or proof-byte framing.
- Added a direct claim-statement hash regression test so the row-9 `claim_v2` -> `claim_contract` label migration is proved explicitly instead of only through sign-and-verify roundtrips.
- Closed the full 21-row claim rename matrix and the embedded coverage-proof table with green bootstrap and release-style validations plus truthful residue classification: zero live `ClaimRootVer`/`SourceRootVer`/`claim-v2`/`root_ver()` survivors and one intentional `claim_v2` legacy-label mirror in the crypto regression test.

## Task Commits

No task commits were created in this execution pass. The `036-23` changes remain in the working tree.

## Files Created/Modified

- `crates/z00z_crypto/src/claim/claim_contract.rs` - new owner module with `CLAIM_ROOT_VERSION`, `root_version` fields, and the `claim_contract` domain label.
- `crates/z00z_crypto/src/claim/v2.rs` - deleted owner module replaced by `claim_contract.rs`.
- `crates/z00z_crypto/src/claim/mod.rs` and `crates/z00z_crypto/src/lib.rs` - public claim facade now re-exports `CLAIM_ROOT_VERSION` instead of `ClaimRootVer`.
- `crates/z00z_wallets/src/core/tx/claim_tx_helpers.rs`, `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs`, `crates/z00z_wallets/src/core/tx/claim_errors.rs`, and `crates/z00z_wallets/src/core/tx/claim_tx.rs` - wallet claim verifier and helper path migrated to `claim contract`, `CLAIM_ROOT_VERSION`, and `SourceRootVersion`.
- `crates/z00z_storage/src/assets/store_internal/store_query.rs`, `crates/z00z_storage/src/assets/types_identity.rs`, and `crates/z00z_storage/src/checkpoint/build.rs` - storage-owned claim roots now carry raw-byte `root_version` and the exported root-version constant.
- `crates/z00z_simulator/src/claim_pkg_consumer.rs` and `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs` - simulator authoritative proof checks and tests now use `root_version` and raw-byte mismatch fixtures.
- `crates/z00z_crypto/tests/test_claim_contract.rs`, `crates/z00z_wallets/src/core/tx/test_claim_tx.rs`, and `crates/z00z_storage/tests/test_claim_source_proof.rs` - targeted tests updated to the new claim contract wording and raw root-version contract.

## Decisions Made

- Preserved the on-wire claim statement layout by keeping `CLAIM_TAG`, `ClaimProofVer`, and field ordering unchanged while replacing the root-version wrapper with a validated raw byte.
- Treated the `claim_v2` -> `claim_contract` domain-label migration as the only compatibility-breaking literal change in this wave.
- Kept `036-20-SUMMARY.md` as the authoritative partial shim-removal boundary and did not use the clean claim-lane scan to over-claim broader Phase 036 closure.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Restored explicit root-version validation after removing the wrapper type**

- **Found during:** Task 1 (`036-23.A Rename the crypto-owned claim module, retire ClaimRootVer, and move the owner surface onto CLAIM_ROOT_VERSION`)
- **Issue:** Converting the claim root version surface from `ClaimRootVer` to raw `u8` removed the old wrapper-enforced non-zero invariant for public `ClaimStmt` construction and bincode-decoded `ClaimSourceProof` payloads.
- **Fix:** Added explicit shape checks for `root_version` and `proof_ver` in `ClaimStmt::chk_shape()` and `ClaimSourceProof::chk_shape()`, and enforced them from `new()`, `to_bytes()`, and `from_bytes()`.
- **Files modified:** `crates/z00z_crypto/src/claim/claim_contract.rs`
- **Verification:** `cargo test -p z00z_crypto --release --features test-fast --test test_claim_contract -- --nocapture`; `cargo test --release --features test-fast --features wallet_debug_dump`
- **Committed in:** not committed in this pass

**2. [Rule 3 - Blocking] Migrated the omitted simulator authoritative-bundle consumer**

- **Found during:** Task 2 (`036-23.B Migrate wallet, storage, and simulator claim consumers to root_version and CLAIM_ROOT_VERSION without widening proof semantics`)
- **Issue:** `crates/z00z_simulator/src/claim_pkg_consumer.rs` still called `root_ver()` on proof and storage root objects, so the row-owned API rename would have left the simulator claim package consumer uncompilable.
- **Fix:** Updated the simulator consumer to compare `root_version()` values, keeping the authoritative-bundle contract aligned with the renamed storage and crypto surfaces.
- **Files modified:** `crates/z00z_simulator/src/claim_pkg_consumer.rs`
- **Verification:** `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_pkg_crypto_support -- --nocapture`; `cargo test --release --features test-fast --features wallet_debug_dump`
- **Committed in:** not committed in this pass

**3. [Rule 4 - Validation Drift] Replaced the nonexistent wallet test target, added direct row-9 hash evidence, and made the old-name scans identifier-exact**

- **Found during:** review of Tasks 1 through 3 against `036-23-PLAN.md`
- **Issue:** The summary claimed a successful `cargo test -p z00z_wallets --release --features test-fast --test test_claim_tx -- --nocapture` run even though `test_claim_tx` is not an integration-test target, the crypto test surface did not directly prove that `claim_stmt_hash()` now binds the `claim_contract` label instead of the legacy `claim_v2` label, and the documented old-name scans still used substring matches that falsely flag `BadRootVersion` and `SourceRootVersion` as if the old identifiers survived.
- **Fix:** Replaced the wallet-targeted validation command with the valid library filter `cargo test -p z00z_wallets --release --features test-fast core::tx::claim_tx::claim_tx_tests -- --nocapture`, updated the plan verify blocks to use identifier-exact old-name residue scans, added `test_claim_hash_uses_claim_contract_label` to the crypto contract test file, and rewrote the residue closeout to classify the one intentional `claim_v2` test-only mirror separately from zero live code-path residues.
- **Files modified:** `.planning/phases/036-rename/036-23-PLAN.md`, `.planning/phases/036-rename/036-23-SUMMARY.md`, `crates/z00z_crypto/tests/test_claim_contract.rs`
- **Verification:** `cargo test -p z00z_crypto --release --features test-fast --test test_claim_contract -- --nocapture`; `cargo test -p z00z_wallets --release --features test-fast core::tx::claim_tx::claim_tx_tests -- --nocapture`; exact fixed-string residue scans across the 036-23 claim surface; `cargo test --release --features test-fast --features wallet_debug_dump`
- **Committed in:** not committed in this pass

---

**Total deviations:** 3 auto-fixed (1 missing critical, 1 blocking, 1 validation drift)
**Impact on plan:** The added fixes were required to preserve the pre-existing non-zero invariant, keep the renamed claim consumer graph compiling, and make the closeout evidence reproducible. No wider scope was introduced.

## Issues Encountered

- The row-2 file rename target, `crates/z00z_crypto/tests/test_claim_contract.rs`, was already present before `036-23` execution, so the work normalized its contents and expectations instead of moving a live `test_claim_v2_contract.rs` file.
- Naive substring scans for `BadRootVer` and `SourceRootVer` also match the intended new names `BadRootVersion` and `SourceRootVersion`. Closeout classification therefore now uses identifier-exact old-name residue scans plus the dedicated `root_ver()` regex when judging the live code path.
- The direct row-9 regression proof intentionally keeps one literal `claim_v2` label in `crates/z00z_crypto/tests/test_claim_contract.rs` so the test can prove the new `claim_contract` hash differs from the legacy label. That hit is intentional coverage-proof residue, not a live owner or consumer survivor.
- The broad release-style gate is not currently green in this workspace: `cargo test --release --features test-fast --features wallet_debug_dump` now fails outside the 036-23 claim surface on `z00z_crypto/tests/test_h2scalar.rs::test_h2scalar_golden_vectors`. The targeted 036-23 claim validations remain green, but this summary cannot claim a clean all-green workspace pass.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_crypto --release --features test-fast --test test_claim_contract -- --nocapture`
- `cargo test -p z00z_storage --release --features test-fast --test test_claim_source_proof -- --nocapture`
- `cargo test -p z00z_wallets --release --features test-fast core::tx::claim_tx::claim_tx_tests -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_pkg_crypto_support -- --nocapture`
- Identifier-exact residue classification across `crates/**/*.rs` excluding `crates/z00z_crypto/tari/**`: `ClaimRootVer`, `SourceRootVer`, `claim-v2`, and `root_ver()` all returned zero hits; `claim_v2` returned exactly one intentional legacy-label mirror in `crates/z00z_crypto/tests/test_claim_contract.rs`.
- `cargo test --release --features test-fast --features wallet_debug_dump` -> fails outside the 036-23 claim surface on `z00z_crypto/tests/test_h2scalar.rs::test_h2scalar_golden_vectors`

Targeted 036-23 claim validations completed successfully in this execution pass. The broad workspace gate remains red on the unrelated `test_h2scalar_golden_vectors` failure above.

## Coverage-Proof Closeout

- Rows 1-9 and 18 are closed on repository-backed code changes in the new `claim_contract` owner module, its public exports, and the crypto integration test.
- Rows 10-17 and 19-21 are closed on repository-backed wallet, storage, and simulator consumer/test updates, plus the one required simulator consumer blocker fix in `crates/z00z_simulator/src/claim_pkg_consumer.rs`.
- Identifier-exact old-name residue classification found no live `ClaimRootVer`, `SourceRootVer`, `claim-v2`, or `root_ver()` hits in `crates/**/*.rs` outside `crates/z00z_crypto/tari/**`, and it found exactly one intentional `claim_v2` legacy-label mirror in `crates/z00z_crypto/tests/test_claim_contract.rs` that exists only to prove the row-9 hash-domain migration.
- The release-style broad workspace gate still has one unrelated failing test outside the claim-contract lane, so this closeout proves the full 21-row claim matrix and its targeted validations, but it does not prove a globally green workspace.
- Historical mentions under `.planning/**` and `logs/**` remain expected archive residue and are not part of this rename wave.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `036-23` is summary-backed complete and its plan matrix is fully closed.
- `036-20-SUMMARY.md` remains the authoritative partial shim-removal boundary, so Phase 036 still stays open even though the claim-contract continuation is complete.

---
*Phase: 036-rename*
*Completed: 2026-04-21*
