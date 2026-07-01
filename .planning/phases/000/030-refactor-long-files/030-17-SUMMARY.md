---
phase: 030
plan: 17
subsystem: z00z_wallets wallet-backup
summary: Reduce wallet-core and backup residue below the continuation band while preserving shallow wallet and backup facades.
tags:
  - phase-030
  - wallet-backup
  - wallet-core
  - refactor
  - restore
requirements:
  - PH30-SEAMS
  - PH30-FACADE
  - PH30-VERIFY
affects:
  - crates/z00z_wallets/src/core/wallet
  - crates/z00z_wallets/src/core/backup
  - crates/z00z_wallets/tests/test_backup_restore_identity.rs
provides:
  - Smaller wallet-core and backup roots below the >400 continuation band
  - Explicit wallet and backup seam files behind unchanged shallow facades
  - Importer-side backup payload validation parity with exporter verification
key_files:
  created:
    - crates/z00z_wallets/src/core/backup/backup_exporter_tests.rs
    - crates/z00z_wallets/src/core/backup/backup_exporter_verify.rs
    - crates/z00z_wallets/src/core/backup/backup_importer_tests.rs
    - crates/z00z_wallets/src/core/backup/backup_wire.rs
    - crates/z00z_wallets/src/core/backup/wallet_backup_container.rs
    - crates/z00z_wallets/src/core/backup/wallet_backup_kdf.rs
    - crates/z00z_wallets/src/core/backup/wallet_backup_tests.rs
    - crates/z00z_wallets/src/core/wallet/errors_impls.rs
    - crates/z00z_wallets/src/core/wallet/errors_tests.rs
    - crates/z00z_wallets/src/core/wallet/errors_types.rs
    - crates/z00z_wallets/src/core/wallet/snapshot_impl.rs
    - crates/z00z_wallets/src/core/wallet/snapshot_tests.rs
    - crates/z00z_wallets/src/core/wallet/snapshot_types.rs
    - crates/z00z_wallets/src/core/wallet/stub_responses_asset.rs
    - crates/z00z_wallets/src/core/wallet/stub_responses_backup.rs
    - crates/z00z_wallets/src/core/wallet/stub_responses_tx.rs
    - crates/z00z_wallets/src/core/wallet/stub_responses_wallet.rs
    - crates/z00z_wallets/src/core/wallet/wallet_entity_asset_api.rs
    - crates/z00z_wallets/src/core/wallet/wallet_entity_backup_api.rs
    - crates/z00z_wallets/src/core/wallet/wallet_entity_constructor.rs
    - crates/z00z_wallets/src/core/wallet/wallet_entity_core.rs
    - crates/z00z_wallets/src/core/wallet/wallet_entity_key_api.rs
    - crates/z00z_wallets/src/core/wallet/wallet_entity_tx_api.rs
    - crates/z00z_wallets/src/core/wallet/wallet_entity_wallet_api.rs
  modified:
    - crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs
    - crates/z00z_wallets/src/core/backup/backup_importer_impl.rs
    - crates/z00z_wallets/src/core/backup/mod.rs
    - crates/z00z_wallets/src/core/backup/wallet_backup.rs
    - crates/z00z_wallets/src/core/wallet/errors.rs
    - crates/z00z_wallets/src/core/wallet/snapshot.rs
    - crates/z00z_wallets/src/core/wallet/stub_responses.rs
    - crates/z00z_wallets/src/core/wallet/wallet_entity.rs
    - reports/full_verify-report-long-running-tests.txt
decisions:
  - Keep wallet and backup caller surfaces stable by turning oversized roots into thin include-based facades with sibling seam ownership.
  - Centralize backup wire constants and payload structs in one internal module so importer and exporter verification cannot drift structurally.
  - Treat importer payload decode as a semantic validation boundary and make verify_password return false for corrupted or invalid backups instead of reporting misleading success.
metrics:
  duration: current-session
  completed_at: 2026-04-01
  tasks_completed: 2/2
---

# Phase 030 Plan 17: Wallet Core And Backup Residue Split Summary

Reduced the remaining oversized wallet-core and backup roots below the continuation band while preserving the shallow wallet and backup facades and hardening importer-side restore validation to match exporter expectations.

## Outcomes

- Wallet-core residue was already structurally split when the plan resumed, and the root files stayed thin while the summary-backed seam set remained in place:
  - `wallet_entity.rs`: 7
  - `snapshot.rs`: 20
  - `stub_responses.rs`: 34
  - `errors.rs`: 15
- Backup Task 2 finished with all remaining roots below the `>400` continuation band:
  - `backup_exporter_impl.rs`: 308
  - `backup_importer_impl.rs`: 371
  - `wallet_backup.rs`: 197
- `backup_wire.rs` now owns the shared backup format constants and serde wire types previously duplicated across importer and exporter flows.
- `backup_exporter_verify.rs` and the extracted backup test modules made exporter/importer and wallet-backup logic independently reviewable without changing the shallow `core::backup` entry surface.
- Importer decode now validates migrated snapshots and checksums through `validate_export_pack`, closing the semantic gap where corrupted or semantically invalid export packs could previously pass the early password check.
- `verify_password` now returns `false` for corrupted, tampered, or semantically invalid backups rather than surfacing misleading success or importer-only errors during precheck usage.

## Verification

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_wallets --release --test test_app_service_create_wallet -- --nocapture`
- `cargo test -p z00z_wallets --release --test test_open_wallet_source_discovery -- --nocapture`
- `cargo test -p z00z_wallets --release --test test_backup_restore_identity -- --nocapture`
- `cargo test -p z00z_wallets --release --lib backup_importer_impl`
- `cargo test --release --features test-fast --features wallet_debug_dump`
- `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run`
- Codacy analysis returned no issues on every edited wallet and backup file.
- Three YOLO review passes completed, and the final two consecutive review passes reported no significant issues.
- Final max-safe verification passed with `planned=313 skipped=21 failed=0` and refreshed `reports/full_verify-report-long-running-tests.txt`.

## Deviations from Plan

### Auto-fixed Issues

1. `[Rule 3 - Blocking issue]` Repaired malformed split fallout in `backup_exporter_impl.rs`, `backup_importer_impl.rs`, and `wallet_backup.rs` after the initial extraction patch damaged file tails and module structure.
2. `[Rule 3 - Blocking issue]` Updated extracted backup fixtures and tamper tests to mutate parsed containers and current snapshot constructors so verification exercises integrity logic instead of JSON parse failure.
3. `[Rule 3 - Blocking issue]` Fixed `cargo fmt --check` drift and a clippy `len_zero` warning exposed by the release-style and max-safe gates.
4. `[Rule 1 - Bug]` Added importer-side migrated snapshot and checksum validation parity after review found that decoded export packs were not being semantically validated as early as exporter verification.
5. `[Rule 1 - Bug]` Corrected `verify_password` semantics so corrupted or invalid backups return `false` instead of reporting success after decrypt-decompress only.

## Known Stubs

None.

## Deferred Issues

- Review surfaced a likely pre-existing architectural divergence between the binary `WalletBackupCrypto` backup container surface and the JSON importer flow. That contract mismatch was not introduced by this structural split and remains out of scope for Plan 030-17.

## Self-Check: PASSED

- Summary file created at `.planning/phases/030-refactor-long-files/030-17-SUMMARY.md`
- Wallet-core and backup residual roots verified below the continuation band for this plan
- Targeted wallet backup and restore regressions passed after the final importer validation fixes
- `full_verify.sh --max-safe-run` passed with `planned=313 skipped=21 failed=0`
