# 036-01 Summary

## Scope

This summary records the completion state for `036-01-PLAN.md`, covering task
`036-01 Freeze Keep-Only And Future-Reserved Rows`, task
`036-02 Legacy Wallet-KDF Retirement`, and task
`036-03 Legacy Backup-KDF Retirement`.

## Outcome

Plan 01 is fully closed.

Phase 036 now has a truthful Step 0 hold baseline, the wallet KDF open path is
current-only and fail-closed, and the backup KDF path no longer accepts or
constructs the legacy compatibility contract.

## Repository Changes

- `036-TODO-1.md` now records the verified completion of the `036-01` hold-only
  checklist and keeps `036-02` and `036-03` marked complete on the same
  repository-backed evidence.
- `crates/z00z_wallets/src/db/redb_wallet_crypto.rs` now exposes only the
  current wallet KDF version markers and current unsuffixed wallet-key
  derivation surface.
- `crates/z00z_wallets/src/core/key/key_manager_redb.rs`,
  `crates/z00z_wallets/src/core/key/key_manager_redb_wallet.rs`, and
  `crates/z00z_wallets/src/db/redb_wallet_store_open_session.rs` now reject
  legacy persisted wallet KDF metadata instead of migrating or defaulting it.
- `crates/z00z_wallets/src/db/redb_wallet_store_migrations.rs`,
  `crates/z00z_wallets/src/db/redb_wallet_store_crypto_ops.rs`,
  `crates/z00z_wallets/src/db/redb_wallet_crypto_aad.rs`, and
  `crates/z00z_wallets/src/core/hashing.rs` no longer carry the legacy wallet
  migration or fallback helpers removed by `036-02`.
- `crates/z00z_wallets/src/core/backup/wallet_backup_kdf.rs`,
  `crates/z00z_wallets/src/core/backup/wallet_backup.rs`,
  `crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs`, and
  `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs` now keep one
  canonical backup KDF constructor and reject the legacy backup KDF contract.
- Regression coverage in wallet and backup tests now proves fail-closed
  behavior for missing or unsupported wallet metadata versions and for legacy
  backup KDF payloads.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`: passed
- `cargo test -p z00z_wallets --test test_redb_wlt_open --release --features test-fast`: passed
- `cargo test -p z00z_wallets --test test_wallet_kdf_migration --release --features test-fast`: passed
- `cargo test -p z00z_wallets --release --features test-fast`: passed for the Phase 036 wallet slice; one unrelated Phase 035 planning guard remained outside scope during earlier reruns
- `cargo test --release --features test-fast --features wallet_debug_dump`: blocked outside Plan 01 by pre-existing simulator planning-guard drift in `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`

## Review Loop

Repeated review passes found and resolved these Plan 01 issues:

1. dead legacy HKDF helper symbols still present after the wallet KDF delete wave
2. missing regression coverage for absent or unsupported `wallet.aad_secret_version`
   and `wallet.hkdf_info_version` metadata
3. stale legacy wallet-KDF test expectations that still assumed compatibility
   migration or `InvalidConfig` instead of the current fail-closed contract
4. missing proof that fresh wallets actually write the required metadata marker
   keys before the negative tests delete or overwrite them

After those fixes, the review loop reached clean closure for the Plan 01 code
slice with no remaining material findings in `036-01` through `036-03`.

## Current Boundary

This summary records only Plan 01 closure. It does not claim any work from
`036-02-PLAN.md`, `036-03-PLAN.md`, or the later backup-payload and hold-only
lanes.
