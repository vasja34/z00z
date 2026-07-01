---
phase: 044-wallet-assets
plan: 05
artifact: summary
status: complete
created: 2026-05-09
updated: 2026-05-09
owner: Z00Z Wallets and Storage
scope: wave 05 closeout for balance views, regression matrix, and source-shape guards
---

# Phase 044 Wave 05 Summary

Wave 05 closed public balance truth, the regression matrix, and the
source-shape guards that keep the phase from drifting back to stub or fake
paths.

Representative homes:

- `crates/z00z_wallets/tests/test_tx_balance.rs`
- `crates/z00z_wallets/tests/test_spec_terms_guard.rs`
- `crates/z00z_wallets/tests/test_tx_drift.rs`
- `crates/z00z_wallets/tests/test_wallet_service_errors.rs`
- `crates/z00z_wallets/tests/test_tx_store_integration.rs`

Validation evidence:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release --features test-fast --features wallet_debug_dump`
- `rg -n "BuiltTxStub|pending = 0|inputs: vec!\\[\\]|outputs: vec!\\[\\]|wallet_tx_history_dir|collect_tx_history_records|format!\\(\\\"\\{tx_hash\\}\\.json\\\"\\)" crates/z00z_wallets/src crates/z00z_wallets/tests .planning/phases/044-wallet-assets`

Outcome:

- `available` and `pending` now derive from lifecycle truth.
- The AC/T/PT regression matrix is represented in coverage.
- Stub-success, empty-detail, and per-tx JSON live-store regressions stay
  blocked.
