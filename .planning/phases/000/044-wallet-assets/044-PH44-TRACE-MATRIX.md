---
phase: 044-wallet-assets
artifact: requirement-trace-matrix
status: evidence-backed
updated: 2026-05-12
owner: Z00Z Wallets
scope: PH44-SEND, PH44-ADMIT, PH44-RECONCILE, PH44-HISTORY, PH44-OFFLINE
---

# PH44 Requirement Trace Matrix

## ✅ Scope

This matrix maps phase requirements to executable test evidence and touched
runtime seams.

## ✅ Matrix

| Requirement | Runtime seam | Test evidence | Status |
| --- | --- | --- | --- |
| `PH44-HISTORY` | `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs` canonical `wallet_<stem>_tx_history.jsonl` append-only journal with `TxConfirmationEvidence` on confirmed rows | `tx_history_appends_admission_sequence`, `jsonl_replay_preserves_full_tx_package_bytes` in `crates/z00z_wallets/tests/test_tx_store_integration.rs`; evidence-backed receipt assertions in `test_tx_import_reconcile_portable` | evidence-backed |
| `PH44-SEND` | `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_send.rs` and `build_transaction_impl` canonical receiver-card path | `test_tx_send_idempotent_same`, `test_tx_send_limits_10`, `test_tx_build_raw_tx`, `test_tx_build_limits_20`, `test_tx_send_honors_asset_id` in `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs`; `test_wallet_tx_transaction_idempotent` in `crates/z00z_wallets/tests/test_rpc_wiring_spec_a.rs` | validated |
| `PH44-ADMIT` | `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs` through `WalletTxAdmitter`, `SimulatedWalletTxAdmitter`, and typed confirmation evidence store | `test_tx_broadcast_admits_without_confirming`, `test_tx_cancel_pending_tx`, `test_tx_broadcast_not_retry`, `test_tx_broadcast_retries_exhausted`; `tx_history_appends_admission_sequence` in `crates/z00z_wallets/tests/test_tx_store_integration.rs` | evidence-backed |
| `PH44-RECONCILE` | `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs` evidence-backed reconcile path plus receipt projection | `test_tx_reconcile_requires_confirmation_evidence`, `test_tx_reconcile_rejects_mismatched_evidence`, `test_tx_import_reconcile_portable`, `test_tx_history_includes_receipt`, `test_tx_get_includes_receipt`, `test_tx_get_filters_status`, `test_tx_list_reflects_cancel` | evidence-backed |
| `PH44-OFFLINE` | `PortableWalletTxPackage` export/import boundary around canonical `TxPackage` bytes | `test_tx_export_portable_json`, `test_tx_import_reconcile_portable`, package parity and tamper tests in `crates/z00z_wallets/tests/test_tx_parity.rs`, `crates/z00z_wallets/tests/test_tx_roundtrip.rs`, `crates/z00z_wallets/tests/test_tx_tamper.rs`, `crates/z00z_wallets/tests/test_tx_wrong_root.rs` | evidence-backed |

## ✅ Execution Evidence

- Command used:
  - `cargo test -p z00z_wallets --release --features test-fast test_tx_ -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast test_wallet_tx_transaction_idempotent -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast tx_ -- --nocapture`
- Additional focused verification on 2026-05-12:
  - `cargo check -p z00z_wallets --tests --features test-fast,wallet_debug_dump`
  - `cargo test -p z00z_wallets --features test-fast,wallet_debug_dump --lib adapters::rpc::methods::tx_impl::tests::test_tx_impl_body::test_tx_broadcast_admits_without_confirming -- --exact`
  - `cargo test -p z00z_wallets --features test-fast,wallet_debug_dump --lib adapters::rpc::methods::tx_impl::tests::test_tx_impl_body::test_tx_reconcile_requires_confirmation_evidence -- --exact`
  - `cargo test -p z00z_wallets --features test-fast,wallet_debug_dump --lib adapters::rpc::methods::tx_impl::tests::test_tx_impl_body::test_tx_reconcile_rejects_mismatched_evidence -- --exact`
  - `cargo test -p z00z_wallets --features test-fast,wallet_debug_dump --lib adapters::rpc::methods::tx_impl::tests::test_tx_impl_body::test_tx_import_reconcile_portable -- --exact`
  - `cargo test -p z00z_wallets --features test-fast,wallet_debug_dump --lib adapters::rpc::methods::tx_impl::tests::test_tx_impl_body::test_tx_history_includes_receipt -- --exact`
  - `cargo test -p z00z_wallets --features test-fast,wallet_debug_dump --lib adapters::rpc::methods::tx_rpc_storage::tests::tx_info_to_details_decodes_package_rows -- --exact`
  - `cargo test -p z00z_wallets --features test-fast,wallet_debug_dump --test test_tx_store_integration tx_history_appends_admission_sequence -- --exact`
- Result snapshot:
  - `test_wallet_tx_transaction_idempotent` completed successfully under the dispatcher integration test filter.
  - `tx_` completed successfully, including `tx_history_appends_admission_sequence` and `jsonl_replay_preserves_full_tx_package_bytes` in `test_tx_store_integration`.
  - The tx-focused unit set under `src/lib.rs` reported `164 passed; 0 failed` before integration binaries were filtered and executed.
  - All focused PH44 evidence, receipt, and append-only journal checks passed on the current dirty tree after the final `tx_rpc_storage.rs` edits.

## ✅ Notes

- Canonical compact receiver cards are now used in tx send/build fixture calls.
- Wallet spendable asset seeding is explicit in fixture setup where send/build
  success is expected.
- No alias/shim recipient path is introduced; invalid recipient aliases are
  rejected by contract (`SEND_RECIPIENT_INVALID`).
- Broadcast/admission tests use real `TxPackage` bytes instead of forced success
  marker strings.
- Offline import rejects metadata-hash tampering before reconcile confirms the
  canonical package.
- Reconcile now rejects missing or mismatched confirmation evidence and uses
  stored typed evidence for receipts instead of synthesizing receipt roots from
  tx hash alone.
