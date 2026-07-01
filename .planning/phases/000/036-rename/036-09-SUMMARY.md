# 036-09 Summary

## Scope

This summary records the completion state for `036-09-PLAN.md`, covering task
`036-06 Clean Local And Test-Only Residue After Production Naming Stabilizes`.

## Outcome

Plan 09 is closed for the active Phase 036 slice.

The Step 5 cleanup wave is now complete only on the raw rows classified as
`rename now` inside the canonical local and test residue inventory. Cleanup-safe
locals and fixture strings were simplified to match the production terminology
already stabilized by Plans 04 through 08, while explicit legacy and
version-scenario helpers classified as `keep` remained intact.

The temporary repair drift in
`crates/z00z_core/tests/assets/test_integration_assets_test12.rs` was resolved
before closure: the file was reconstructed back to its original test semantics,
then narrowed again to the authorized residue-only renames from the Step 5 raw
inventory.

## Repository Changes

- `crates/z00z_core/src/assets/registry_tests.rs`,
  `crates/z00z_core/tests/assets/test_integration_assets_test12.rs`,
  `crates/z00z_core/tests/genesis/test_crypto_security.rs`,
  `crates/z00z_crypto/src/commitments.rs`,
  `crates/z00z_crypto/tests/test_pedersen.rs`,
  `crates/z00z_wallets/src/core/address/z00z_address/tests.rs`,
  `crates/z00z_wallets/src/core/wallet/snapshot_tests.rs`, and
  `crates/z00z_wallets/tests/test_tx_poison.rs` now use the simplified local
  cleanup names authorized by the Step 5 raw rows without erasing explicit
  scenario coverage.
- `crates/z00z_wallets/src/db/redb_wallet_store_crypto_ops.rs` now uses the
  cleanup-safe `aad` local on the import-side decrypt path, matching the
  canonical Step 5 rename-now row.
- `crates/z00z_wallets/src/services/wallet_service_store_transfer_import.rs`
  now uses `decoded_container` on the import path and `encrypted_container` on
  the export path, matching the row-11 closure state for both local bindings in
  that file.
- `.planning/phases/036-rename/036-TODO-2.md` now truthfully marks the Step 5
  checklist complete and includes the full canonical Step 5 file surface,
  including `crates/z00z_wallets/src/db/redb_wallet_store_crypto_ops.rs` and
  `crates/z00z_wallets/src/services/wallet_service_store_transfer_import.rs`.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`: passed
- `cargo test --release --features test-fast --features wallet_debug_dump`:
  failed outside Plan 09 scope in read-only vendor
  `crates/z00z_crypto/tari/crypto/` doctests because multiple
  `tari_utilities` versions break `tari_crypto --doc`
- `cargo test -p z00z_wallets --release --features test-fast --test test_tx_poison`:
  passed
- `cargo test -p z00z_storage --release --features test-fast --test test_redb_rehydrate`:
  passed
- `cargo test -p z00z_crypto --release --features test-fast --test test_claim_v2_contract`:
  passed
- `cargo test -p z00z_core --release --features test-fast --test assets_tests test_version_monotonicity`:
  passed
- `cargo test -p z00z_core --release --features test-fast --test assets_tests test_reads_during_snapshot_updates`:
  passed
- `cargo test -p z00z_core --release --features test-fast --test assets_tests test_arc_validity_after_update`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --lib core::address::z00z_address::tests::version_constant_is_single_source -- --exact`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --lib core::backup::backup_importer_impl::tests::test_import_legacy_v1_backup_is_rejected -- --exact`:
  passed
- editor diagnostics on all Plan 09 modified files and the updated planning
  artifacts: clean

## Review Loop

The review loop closed truthfully in five passes:

1. review pass 1 found that a temporary repair in
   `test_integration_assets_test12.rs` had widened beyond the authorized Step 5
   cleanup boundary
2. review passes 2 and 3 narrowed the file back to residue-only scope and then
   found one remaining planning-authority drift because `036-TODO-2.md` was
   still missing two Step 5 files already present in the canonical plan and raw
   spec
3. the TODO file surface was synchronized, deterministic validation stayed
   green, and review pass 4 found no significant in-scope issues after that
   final fix cycle
4. review pass 5 also found no significant in-scope issues, making passes 4
   and 5 the required consecutive clean review runs after the last fix cycle

The exact runtime commands for `/crypto-architect`, `/security-audit`, and
`/doublecheck` were not directly available in this environment, so the review
evidence used the repo-local best-effort path: canonical spec rereads,
path-specific source inspection, planning-authority reconciliation, editor
diagnostics, targeted test reruns, and repeated review passes in YOLO mode.

## Current Boundary

This summary closes only Plan 09 of Phase 036. It does not claim execution of
the final closure-validation wave now queued under `036-10-PLAN.md`.
