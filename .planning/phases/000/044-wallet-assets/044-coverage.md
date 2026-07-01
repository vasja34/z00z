---
phase: 044-wallet-assets
artifact: coverage
status: evidence-synced
created: 2026-05-09
updated: 2026-05-11
owner: Z00Z Wallets and Storage
scope: phase closeout ledger for Phase 044 test coverage and regression guards
---

# Phase 044 Coverage Ledger

Status: all scenario homes are mapped, the closeout artifacts exist, and the
release validation is green.

Closeout evidence rerun on 2026-05-09:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release --features test-fast --features wallet_debug_dump`
- `rg -n "BuiltTxStub|pending = 0|inputs: vec!\\[\\]|outputs: vec!\\[\\]|wallet_tx_history_dir|collect_tx_history_records|format!\\(\\\"\\{tx_hash\\}\\.json\\\"\\)" crates/z00z_wallets/src crates/z00z_wallets/tests .planning/phases/044-wallet-assets`
- `git diff --check`

Execution slice 06 evidence rerun on 2026-05-10:

- `cargo test -p z00z_wallets --release --features test-fast test_tx_ -- --nocapture`
- `cargo test -p z00z_wallets --release --features test-fast --test test_tx_parity -- --nocapture`
- `cargo test -p z00z_wallets --release --features test-fast --test test_tx_roundtrip -- --nocapture`
- `cargo test -p z00z_wallets --release --features test-fast --test test_tx_tamper -- --nocapture`
- `cargo test -p z00z_wallets --release --features test-fast --test test_tx_wrong_root -- --nocapture`

Execution slice PH44-SEND/ADMIT/RECONCILE evidence rerun on 2026-05-11:

- `cargo check -p z00z_wallets --features test-fast`
- `cargo test -p z00z_wallets --release --features test-fast test_wallet_tx_transaction_idempotent -- --nocapture`
- `cargo test -p z00z_wallets --release --features test-fast tx_ -- --nocapture`
- `rg -n "BuiltTxStub|pending = 0|inputs: vec!\[\]|outputs: vec!\[\]|wallet_tx_history_dir|collect_tx_history_records|format!\(\"\{tx_hash\}\.json\"\)|Migrated|migration_source_hash|build_migrated|TxService|stub-recipient-receiver|__force_network_error_then_success__" crates/z00z_wallets/src crates/z00z_wallets/tests`

## Scenario Coverage

| Scenario ID | Primary homes | Evidence slot | Status | Notes |
| --- | --- | --- | --- | --- |
| `044-SC-01` | `044-coverage.md`, `044-CONTEXT.md`, `044-TEST-SPEC.md`, `044-TESTS-TASKS.md` | `044-01-SUMMARY.md` | landed | Coverage ledger and drift bars stay explicit. |
| `044-SC-02` | `crates/z00z_wallets/src/persistence/assets/test_asset_storage_impl_suite.rs`, `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs`, `crates/z00z_wallets/src/tx/selection/test_asset_selector_suite.rs` | `044-01-SUMMARY.md` | landed | Reservation is atomic and only `Available` inputs are selectable. |
| `044-SC-03` | `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs`, `crates/z00z_wallets/tests/test_rpc_wiring_spec_a.rs`, `crates/z00z_wallets/tests/test_stealth_output.rs`, `crates/z00z_wallets/src/tx/verify/test_tx_verifier_suite.rs` | `044-01-SUMMARY.md`, `044-PH44-TRACE-MATRIX.md` | landed | Sender build/send uses canonical receiver cards, explicit spendable inputs, and real tx package bytes. |
| `044-SC-04` | `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_body.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_pending_body.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_cursor_filters.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_receipt_sort.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_impl_body.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_impl_suite.rs` | `044-02-SUMMARY.md` | landed | Journal-backed history and pending views stay truthful. |
| `044-SC-05` | `crates/z00z_wallets/src/services/wallet/tests/test_wallet_paths_suite.rs`, `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/test_wallet_impl_suite.rs`, `crates/z00z_wallets/tests/test_wallet_persistence_backup_service.rs` | `044-02-SUMMARY.md` and `044-03-SUMMARY.md` | landed | Wallet-stem naming stays canonical. |
| `044-SC-06` | `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs`, `crates/z00z_wallets/tests/test_tx_store_integration.rs` | `044-02-SUMMARY.md`, `044-PH44-TRACE-MATRIX.md` | landed | JSONL storage folds rows deterministically and appends Created -> Submitted -> Admitted -> Confirmed for admitted txs. |
| `044-SC-07` | `crates/z00z_wallets/src/adapters/rpc/methods/test_backup_impl_suite.rs`, `crates/z00z_wallets/src/backup/export/test_backup_exporter_suite.rs`, `crates/z00z_wallets/src/backup/import/test_backup_importer_suite.rs`, `crates/z00z_wallets/src/backup/crypto/test_wallet_backup_suite.rs`, `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs`, `crates/z00z_wallets/tests/test_tx_parity.rs`, `crates/z00z_wallets/tests/test_tx_roundtrip.rs`, `crates/z00z_wallets/tests/test_tx_tamper.rs`, `crates/z00z_wallets/tests/test_tx_wrong_root.rs` | `044-03-SUMMARY.md` | landed | Backup, restore, and portable submission preserve canonical bytes. |
| `044-SC-08` | `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs`, `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs`, `crates/z00z_wallets/tests/test_tx_store_integration.rs` | `044-03-SUMMARY.md` | landed | Live storage uses canonical JSONL only; no legacy tx-history fallback remains in live Rust source. |
| `044-SC-09` | `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_send.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs`, `crates/z00z_wallets/tests/test_tx_parity.rs`, `crates/z00z_wallets/tests/test_tx_roundtrip.rs`, `crates/z00z_wallets/tests/test_tx_tamper.rs`, `crates/z00z_wallets/tests/test_tx_wrong_root.rs` | `044-03-SUMMARY.md`, `044-PH44-TRACE-MATRIX.md` | landed | Portable export/import wraps canonical `TxPackage` bytes, rejects metadata tampering, and stays role-neutral by tx hash. |
| `044-SC-10` | `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_broadcast_body.rs`, `crates/z00z_wallets/src/tx/state/test_state_update_suite.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs`, `crates/z00z_wallets/tests/test_tx_store_integration.rs` | `044-04-SUMMARY.md`, `044-PH44-TRACE-MATRIX.md` | landed | Admission and confirmation require explicit trait-backed evidence and no forced-success marker remains in live Rust source. |
| `044-SC-11` | `crates/z00z_wallets/src/tx/state/test_state_update_suite.rs`, `crates/z00z_wallets/tests/test_tx_store_integration.rs`, `crates/z00z_wallets/tests/test_tx_balance.rs` | `044-04-SUMMARY.md` | landed | Reconciliation remains storage-authoritative. |
| `044-SC-12` | `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs`, `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs`, `crates/z00z_wallets/tests/test_direct_tx_receive.rs`, `crates/z00z_wallets/tests/test_e2e_req_flow.rs`, `crates/z00z_wallets/tests/test_e2e_send_scan.rs`, `crates/z00z_wallets/tests/test_e2e_runtime_parity.rs`, `crates/z00z_wallets/tests/test_claim_snapshot_core.rs`, `crates/z00z_wallets/tests/test_tx_balance.rs` | `044-04-SUMMARY.md` | landed | Report-only receive stays non-persistent. |
| `044-SC-13` | `crates/z00z_wallets/tests/test_tx_balance.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_catalog.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_history.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs` | `044-05-SUMMARY.md` | landed | Balance derives from lifecycle truth. |
| `044-SC-14` | `crates/z00z_wallets/tests/test_spec_terms_guard.rs`, `crates/z00z_wallets/tests/test_tx_drift.rs`, `crates/z00z_wallets/tests/test_wallet_service_errors.rs`, `crates/z00z_wallets/tests/test_tx_store_integration.rs` | `044-05-SUMMARY.md` | landed | Forbidden shapes are rejected. |

## Canonical Identifier Set

- `PH44-LEDGER`, `PH44-SEND`, `PH44-OFFLINE`, `PH44-ADMIT`, `PH44-RECONCILE`
- `PH44-RECEIVE`, `PH44-BALANCE`, `PH44-HISTORY`, `PH44-CANCEL`, `PH44-DRIFT`
- `P-044-001` through `P-044-012`
- `EV-044-001` through `EV-044-026`
- `D-044-001` through `D-044-013`
- `AC-044-001` through `AC-044-024`
- `T-044-001` through `T-044-024`
- `PT-044-001` through `PT-044-016`

## Existing Test Impact Matrix

| Impact group | Representative homes | Evidence slot | Status | Notes |
| --- | --- | --- | --- | --- |
| Wallet service, backup, and restore | `test_wallet_service_suite.rs`, `test_wallet_persistence_backup_service.rs`, `test_backup_impl_suite.rs`, `test_backup_exporter_suite.rs`, `test_backup_importer_suite.rs` | `044-02-SUMMARY.md`, `044-03-SUMMARY.md` | updated | History, backup, and restore paths now use the canonical live JSONL authority. |
| Journal, history, and tx-store | `test_tx_history_body.rs`, `test_tx_pending_body.rs`, `test_tx_history_cursor_filters.rs`, `test_tx_history_receipt_sort.rs`, `test_tx_store_integration.rs` | `044-02-SUMMARY.md` | updated | Details, pending, and history stay journal-backed. |
| Portable package and wrong-root / tamper | `test_tx_parity.rs`, `test_tx_roundtrip.rs`, `test_tx_tamper.rs`, `test_tx_wrong_root.rs` | `044-03-SUMMARY.md` | updated | Canonical bytes stay role-neutral and fail closed on drift. |
| Admission, reconciliation, and receive finalization | `test_tx_broadcast_body.rs`, `test_state_update_suite.rs`, `asset_impl_tests.rs`, `test_direct_tx_receive.rs`, `test_e2e_req_flow.rs`, `test_e2e_send_scan.rs`, `test_e2e_runtime_parity.rs` | `044-04-SUMMARY.md` | updated | Broadcast cannot fake success and report-only receive remains non-persistent. |
| Balance, drift, and terminology guards | `test_tx_balance.rs`, `test_spec_terms_guard.rs`, `test_tx_drift.rs`, `test_wallet_service_errors.rs`, `test_tx_store_integration.rs` | `044-05-SUMMARY.md` | updated | `pending = 0` drift and stub-success shapes are blocked. |
| Coverage and summary artifacts | `044-coverage.md`, `044-01-SUMMARY.md` through `044-05-SUMMARY.md`, `044-SUMMARY.md` | phase closeout | landed | The audit trail is now self-contained inside the phase directory. |

## Closeout Notes

- The base plan did not require new runtime test files.
- The existing anchors in the test spec are sufficient for Phase 044.
- Any future gap should be recorded here instead of creating a parallel layer.
