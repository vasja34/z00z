---
phase: 044-wallet-assets
slice: 07
type: execution
status: implemented-verified
created: 2026-05-12
updated: 2026-05-12
owner: Z00Z Wallets
scope: PH44-SEND, PH44-ADMIT, PH44-RECONCILE, PH44-HISTORY, PH44-OFFLINE
---

# 044-07 PH44 Send Admit Reconcile Slice

## 🎯 Objective

Close the PH44 send/admit/reconcile gap by making confirmation evidence a typed
wallet-journal artifact and by requiring reconciliation to consume that evidence
before mutating tx history from pending/admitted to confirmed.

## ✅ Runtime Changes

- Added `TxConfirmationEvidence` to the canonical tx persistence layer.
- Stored typed checkpoint evidence on confirmed `TxRecord` rows instead of
  synthesizing receipt data from tx hash alone.
- Changed broadcast/admission flow to expose typed simulated confirmation
  evidence while leaving tx journal rows pending/admitted.
- Changed `reconcile_transaction_impl(...)` to require stored typed evidence,
  validate tx id, tx hash, chain id, roots, spent input ids, created output ids,
  and verified status, then append a single confirmed row.
- Kept repeated reconcile idempotent by returning the stored evidence-backed
  confirmation without appending duplicate confirmed rows.

## ✅ Test Evidence Added

- `test_tx_broadcast_admits_without_confirming`
- `test_tx_reconcile_requires_confirmation_evidence`
- `test_tx_reconcile_rejects_mismatched_evidence`
- Extended `test_tx_import_reconcile_portable` to assert evidence-backed
  receipts and idempotent repeated reconciliation.

## ✅ Requirement Trace

| Requirement | Evidence |
| --- | --- |
| `PH44-SEND` | Send still executes canonical build plus broadcast/admission before returning pending tx ids. |
| `PH44-ADMIT` | Broadcast stores typed simulated confirmation evidence but leaves journal rows pending/admitted. |
| `PH44-RECONCILE` | Reconcile rejects missing or mismatched evidence and confirms only after effect validation. |
| `PH44-HISTORY` | Confirmed rows persist `TxConfirmationEvidence`; details/history receipts use stored evidence. |
| `PH44-OFFLINE` | Portable import/export test now proves evidence-backed reconcile and idempotence. |

## ✅ Verification Status

- Compile gate passed:
  `cargo check -p z00z_wallets --tests --features test-fast,wallet_debug_dump`
- Focused PH44 lifecycle tests passed:
  - `cargo test -p z00z_wallets --features test-fast,wallet_debug_dump --lib adapters::rpc::methods::tx_impl::tests::test_tx_impl_body::test_tx_broadcast_admits_without_confirming -- --exact`
  - `cargo test -p z00z_wallets --features test-fast,wallet_debug_dump --lib adapters::rpc::methods::tx_impl::tests::test_tx_impl_body::test_tx_reconcile_requires_confirmation_evidence -- --exact`
  - `cargo test -p z00z_wallets --features test-fast,wallet_debug_dump --lib adapters::rpc::methods::tx_impl::tests::test_tx_impl_body::test_tx_reconcile_rejects_mismatched_evidence -- --exact`
  - `cargo test -p z00z_wallets --features test-fast,wallet_debug_dump --lib adapters::rpc::methods::tx_impl::tests::test_tx_impl_body::test_tx_import_reconcile_portable -- --exact`
- Receipt and journal projection checks passed:
  - `cargo test -p z00z_wallets --features test-fast,wallet_debug_dump --lib adapters::rpc::methods::tx_impl::tests::test_tx_impl_body::test_tx_history_includes_receipt -- --exact`
  - `cargo test -p z00z_wallets --features test-fast,wallet_debug_dump --lib adapters::rpc::methods::tx_rpc_storage::tests::tx_info_to_details_decodes_package_rows -- --exact`
  - `cargo test -p z00z_wallets --features test-fast,wallet_debug_dump --test test_tx_store_integration tx_history_appends_admission_sequence -- --exact`
- This slice is verification-complete for PH44 send/admit/reconcile/history or
  offline seams; broader wallet-crate or full-workspace verification remains a
  separate closeout step.
