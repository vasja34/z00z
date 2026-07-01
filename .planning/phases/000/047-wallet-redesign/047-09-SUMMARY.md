---
phase: 047-wallet-redesign
plan: 9
status: completed
updated: 2026-05-21
---

# ✅ 047-09 Summary

## ✅ Outcome

Phase 047 plan 09 now lands one canonical normal-path wallet story:

- path-based `open_wallet_source` validates sibling JSONL history before commit,
  imports it when present, reconciles it for an already managed wallet, and
  rolls back a newly copied `.wlt` if sidecar publish fails;
- `WalletSource::Bytes` stays wallet-bytes-only;
- payload export/import stays wallet-state-only and does not carry tx-history
  sidecar bytes;
- backup create/restore now uses one canonical wallet-plus-history contract,
  including wrong-stem and oversize rejection on both export and restore;
- live tx-history writes enforce the same JSONL size ceiling as import/export;
- failed backup attempts no longer consume the hourly create-backup window, and
  a second concurrent backup on the same wallet is rate-limited while the first
  remains in progress.

## 🔧 Landed Changes

- Reworked `open_wallet_source` in
  `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_create_unlock_open.rs`
  so path imports validate canonical JSONL sidecars before `.wlt` commit, use
  the canonical writer for sidecar publish, and roll back new `.wlt` files on
  post-copy sidecar failure.
- Split wallet-state payload transfer from full backup history replay by using
  wallet-state-only export packs for `export_wallet_payload` and
  `import_wallet_payload`, while backup create uses the explicit history-carrying
  pack path.
- Tightened backup validation in
  `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup.rs`
  so import/export reject wrong-stem and oversized JSONL sidecars, and backup
  rate limiting is success-based instead of failure-burning.
- Added one shared JSONL size ceiling source of truth in
  `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs` and reused it for
  live writes and backup/export validation.
- Updated `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md` to describe only the
  canonical wallet-state payload route and the explicit backup-history route.

## 🧪 Validation

Executed and passed:

- `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --lib open_source_`
- `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --lib restore_`
- `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --lib backup_`
- `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --lib persist_rejects_oversized_rows`
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`

Focused regressions added and executed for:

- path import of valid sidecar;
- path import reconciliation for an existing managed wallet;
- malformed and oversized sidecar rejection on path import;
- rollback of a new `.wlt` when sidecar publish fails;
- payload import/export staying wallet-state-only;
- default restore replaying history;
- wrong-stem rejection on restore and backup export;
- oversized live persistence rejection;
- oversized backup export rejection;
- retry after failed backup without consuming the rate-limit window;
- second concurrent backup being rate-limited while the first is in progress.

Review loop status:

- review pass 9: no significant issues;
- review pass 10: no significant issues.

## ⚠️ Remaining Workspace Gate

The required full release command was rerun:

- `cargo test --release --features test-fast --features wallet_debug_dump`

The rerun confirmed the same workspace-wide blocker outside this slice:

- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs::test_boundary_wording_stays_narrow`

This failure should be treated as external to 047-09 rather than a regression
from the wallet changes.
