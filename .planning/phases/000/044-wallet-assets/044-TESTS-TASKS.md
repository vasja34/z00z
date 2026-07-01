---
phase: 044-wallet-assets
artifact: tests-tasks
status: evidence-synced
source: 044-TEST-SPEC.md
updated: 2026-05-09
owner: Z00Z Wallets and Storage
scope: implementation order for Phase 044 test coverage
---

# Phase 044 Tests Tasks

## Purpose

This document turns `044-TEST-SPEC.md` into one concrete implementation order
for test work. The ordered waves are now backed by `044-coverage.md`,
`044-01-SUMMARY.md` through `044-05-SUMMARY.md`, and the phase-wide
`044-SUMMARY.md`.

It stays subordinate to `044-TODO.md` and the numbered `044-*-PLAN.md` files.
The order below is about how to land test coverage without duplicating authority
layers or inventing a parallel test surface.

## Scope Inputs

- `044-TEST-SPEC.md`
- the `Existing Test Impact Matrix Appendix` in `044-TEST-SPEC.md`
- `044-CONTEXT.md`
- `044-TODO.md`
- `044-01-PLAN.md` through `044-05-PLAN.md`
- existing test anchors listed in the spec

## Execution Strategy

- Freeze the coverage ledger and drift bars first so implementation does not
  invent a new authority layer.
- Land the sender/reservation path before any history, backup, or reconciliation
  assertions, because later waves depend on truthful tx state.
- Keep journal/history and JSONL path-contract tests ahead of backup and
  migration, because backup and import must consume the canonical history shape.
- Keep admission, reconciliation, and receiver-finalization tests ahead of the
  final balance wave, because `available` and `pending` depend on their results.
- Finish with balance views, source-shape guards, and coverage synchronization.

## Task Waves

### Wave T0: Harness And Coverage Lock-In

- files to inspect:
  - `044-CONTEXT.md`
  - `044-TODO.md`
  - `044-01-PLAN.md` through `044-05-PLAN.md`
  - `044-coverage.md` when it exists
- deliverables:
  - scenario-to-anchor map for `044-SC-01` through `044-SC-14`
  - explicit reuse map for existing test anchors
  - explicit no-new-duplicate-layer decision
  - source-shape guard list for `BuiltTxStub`, `pending = 0`, empty details, fake
    success, report-only drift, and per-tx JSON live-store drift
- success conditions:
  - every scenario in the spec has one primary home
  - no scenario requires a second tx schema, assembler, verifier, or receiver
    path
  - `044-coverage.md` is ready to receive final evidence
- command gate:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `rg -n "EV-044-|D-044-|PH44-|AC-044-|T-044-|PT-044-" .planning/phases/044-wallet-assets/044-coverage.md`
  - `rg -n "BuiltTxStub|pending = 0|inputs: vec!\\[\\]|outputs: vec!\\[\\]|wallet_tx_history_dir|collect_tx_history_records|format!\\(\\\"\\{tx_hash\\}\\.json\\\"\\)" crates/z00z_wallets/src crates/z00z_wallets/tests .planning/phases/044-wallet-assets`

### Wave T1: Asset Ledger And Sender Build/Send

- files to extend:
  - `crates/z00z_wallets/src/persistence/assets/test_asset_storage_impl_suite.rs`
  - `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs`
  - `crates/z00z_wallets/src/tx/selection/test_asset_selector_suite.rs`
  - `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs`
  - `crates/z00z_wallets/tests/test_stealth_output.rs`
  - `crates/z00z_wallets/src/tx/verify/test_tx_verifier_suite.rs`
- deliverables:
  - reservation and release are atomic
  - concurrent reservation on the same input fails closed
  - sender build uses real selected inputs, real outputs, and canonical package bytes
  - build failure releases reservations and never leaves `BuiltTxStub` behavior behind
- success conditions:
  - only one build can reserve a given input
  - valid builds expose real tx bytes only after reservation
  - invalid builds release reservations and surface a typed failure
- command gate:
  - `cargo test -p z00z_wallets --features test-fast --test test_asset_storage_impl_suite -- --nocapture`
  - `cargo test -p z00z_wallets --features test-fast --test test_tx_send_body -- --nocapture`
  - `cargo test -p z00z_wallets --features test-fast --test test_stealth_output -- --nocapture`

### Wave T2: Journal, Path Contract, And JSONL Storage

- files to extend:
  - `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_body.rs`
  - `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_pending_body.rs`
  - `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_cursor_filters.rs`
  - `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_receipt_sort.rs`
  - `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_impl_body.rs`
  - `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_impl_suite.rs`
  - `crates/z00z_wallets/src/services/wallet/tests/test_wallet_paths_suite.rs`
  - `crates/z00z_wallets/src/adapters/rpc/methods/test_wallet_impl_suite.rs`
  - `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs`
  - `crates/z00z_wallets/tests/test_tx_store_integration.rs`
- deliverables:
  - tx details, pending lists, and history are journal-backed
  - `wallet_<stem>.wlt` and `wallet_<stem>_tx_history.jsonl` are the only canonical sibling names
  - `TxStorageImpl` reads and rewrites the canonical JSONL file deterministically and preserves exact package bytes
- success conditions:
  - no fabricated empty details or fake zero-pending rows appear
  - live writes target the JSONL sidecar only
  - malformed hashes, corrupt tails, and partial rows fail closed
- command gate:
  - `cargo test -p z00z_wallets --features test-fast --test test_tx_history_body -- --nocapture`
  - `cargo test -p z00z_wallets --features test-fast --test test_tx_store_integration -- --nocapture`

### Wave T3: Backup, Migration, And Portable Submission

- files to extend:
  - `crates/z00z_wallets/src/adapters/rpc/methods/test_backup_impl_suite.rs`
  - `crates/z00z_wallets/src/backup/export/test_backup_exporter_suite.rs`
  - `crates/z00z_wallets/src/backup/import/test_backup_importer_suite.rs`
  - `crates/z00z_wallets/src/backup/crypto/test_wallet_backup_suite.rs`
  - `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs`
  - `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs`
  - `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_send.rs`
  - `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`
  - `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs`
  - `crates/z00z_wallets/tests/test_tx_parity.rs`
  - `crates/z00z_wallets/tests/test_tx_roundtrip.rs`
  - `crates/z00z_wallets/tests/test_tx_tamper.rs`
  - `crates/z00z_wallets/tests/test_tx_wrong_root.rs`
- deliverables:
  - backup preserves exact live JSONL bytes plus manifest
  - restore writes JSONL back without inventing rows
  - legacy per-tx JSON is regression-only and never regains live-store authority
  - portable package export/import stays role-neutral by tx hash
- success conditions:
  - tampered archive bytes or manifest fail closed
  - legacy dirs are never revived as live authority
  - sender and receiver submit the same canonical bytes through the same path
- command gate:
  - `cargo test -p z00z_wallets --features test-fast --test test_backup_importer_suite -- --nocapture`
  - `cargo test -p z00z_wallets --features test-fast --test test_tx_parity -- --nocapture`

### Wave T4: Admission, Reconciliation, And Receiver Finalization

- files to extend:
  - `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_broadcast_body.rs`
  - `crates/z00z_wallets/src/tx/state/test_state_update_suite.rs`
  - `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs`
  - `crates/z00z_wallets/tests/test_direct_tx_receive.rs`
  - `crates/z00z_wallets/tests/test_e2e_req_flow.rs`
  - `crates/z00z_wallets/tests/test_e2e_send_scan.rs`
  - `crates/z00z_wallets/tests/test_e2e_runtime_parity.rs`
  - `crates/z00z_wallets/tests/test_claim_snapshot_core.rs`
- deliverables:
  - RPC broadcast cannot fake success without admission evidence
  - checkpoint evidence drives reconciliation
  - report-only receive stays non-persistent
  - `recv_route(..., ReceiveNext::PersistClaim)` remains the only final receive path
- success conditions:
  - missing evidence leaves rows pending or quarantined
  - matching evidence finalizes exactly once
  - report-only branches do not mutate claims or balances
- command gate:
  - `cargo test -p z00z_wallets --features test-fast --test test_tx_broadcast_body -- --nocapture`
  - `cargo test -p z00z_wallets --features test-fast --test test_state_update_suite -- --nocapture`
  - `cargo test -p z00z_wallets --features test-fast --test asset_impl_tests -- --nocapture`

### Wave T5: Balance, Regression Matrix, And Source-Shape Guards

- files to extend:
  - `crates/z00z_wallets/tests/test_tx_balance.rs`
  - `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs`
  - `crates/z00z_wallets/tests/test_spec_terms_guard.rs`
  - `crates/z00z_wallets/tests/test_tx_drift.rs`
  - `crates/z00z_wallets/tests/test_wallet_service_errors.rs`
  - `crates/z00z_wallets/tests/test_tx_store_integration.rs`
  - `crates/z00z_wallets/tests/test_wallet_persistence_backup_service.rs`
  - `crates/z00z_wallets/tests/test_wallet_service_suite.rs`
- deliverables:
  - `available` and `pending` are derived from lifecycle truth
  - the existing-test impact matrix is fully represented or explicitly marked as no-change evidence
  - forbidden source shapes are blocked by tests and `rg` guards
  - `044-coverage.md` and `044-SUMMARY.md` receive final evidence
- success conditions:
  - `pending = 0` never hides unresolved pending rows
  - every AC/T/PT row has a test home and evidence slot
  - `BuiltTxStub`, fake success, empty tx details, and per-tx JSON live-store drift do not return
- command gate:
  - `cargo test -p z00z_wallets --features test-fast --test test_tx_balance -- --nocapture`
  - `cargo test --release --features test-fast --features wallet_debug_dump`
  - `rg -n "BuiltTxStub|pending = 0|inputs: vec!\\[\\]|outputs: vec!\\[\\]|wallet_tx_history_dir|collect_tx_history_records|format!\\(\\\"\\{tx_hash\\}\\.json\\\"\\)" crates/z00z_wallets/src crates/z00z_wallets/tests .planning/phases/044-wallet-assets`

## Review Checklist Per Wave

- [ ] Does the wave extend an existing anchor before proposing a new file?
- [ ] Does the scenario prove a real Phase 044 gap instead of re-proving a
      baseline behavior?
- [ ] Does the assertion map to the live canonical seam named in
      `044-TEST-SPEC.md`?
- [ ] Does the test stay truthful about optional, conditional, or legacy-only
      surfaces?
- [ ] Does the scenario preserve one canonical JSONL history lane, one backup
      authority, one admission path, and one final receive route?

## Done Condition

- the wave order matches the phase plan order
- each scenario in `044-TEST-SPEC.md` has a concrete home
- no test file invents a second authority layer or parallel storage path
- closeout artifacts can be synchronized without guessing what the tests prove
