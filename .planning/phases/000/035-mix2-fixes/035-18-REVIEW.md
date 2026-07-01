---
phase: 035-mix2-fixes
reviewed: 2026-04-13T23:59:00Z
depth: standard
files_reviewed: 25
files_reviewed_list:
  - crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_body.rs
  - crates/z00z_wallets/src/adapters/rpc/methods/test_tx_broadcast_body.rs
  - crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_cursor_filters.rs
  - crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_receipt_sort.rs
  - crates/z00z_wallets/src/adapters/rpc/methods/test_tx_impl_body.rs
  - crates/z00z_wallets/src/adapters/rpc/methods/test_tx_impl_suite.rs
  - crates/z00z_wallets/src/adapters/rpc/methods/test_tx_pending_body.rs
  - crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs
  - crates/z00z_wallets/src/core/key/bip32_manager.rs
  - crates/z00z_wallets/src/core/key/test_bip32_manager.inc.rs
  - crates/z00z_wallets/src/core/key/test_bip32_manager_entropy.inc.rs
  - crates/z00z_wallets/src/db/mod.rs
  - crates/z00z_wallets/src/db/wallet_io.rs
  - crates/z00z_wallets/src/db/wallet_store.rs
  - crates/z00z_wallets/src/db/wallet_validate.rs
  - crates/z00z_wallets/src/egui_views/tab_registry.rs
  - crates/z00z_wallets/src/services/session_service.rs
  - crates/z00z_wallets/src/services/wallet_service_session_build.rs
  - crates/z00z_wallets/src/services/wallet_service_session_construction.rs
  - crates/z00z_wallets/src/services/wallet_service_session_derivation.rs
  - crates/z00z_wallets/src/services/wallet_service_session_seed_derivation.rs
  - crates/z00z_wallets/src/services/wallet_service_session_guards.rs
  - crates/z00z_wallets/src/services/wallet_service_session_runtime.rs
  - crates/z00z_wallets/src/services/wallet_service_session_snapshot.rs
  - crates/z00z_wallets/src/services/wallet_service_store_create_unlock.rs
  - crates/z00z_wallets/src/services/wallet_service_store_create_unlock_open.rs
  - crates/z00z_wallets/src/services/wallet_service_store_load_restore.rs
  - crates/z00z_wallets/src/services/wallet_service_store_persistence_pack.rs
  - crates/z00z_wallets/src/services/wallet_service_store_support.rs
  - crates/z00z_wallets/tests/test_redb_wlt_open.rs
  - crates/z00z_wallets/tests/test_wlt_validator.rs
findings:
  critical: 0
  warning: 0
  info: 0
  total: 0
status: clean
---

# Phase 035 Plan 18 Code Review

**Reviewed:** 2026-04-13T23:59:00Z  
**Depth:** standard  
**Files Reviewed:** 25  
**Status:** clean

## Summary

Focused review covered the Phase 035 Plan 18 rename surface in `crates/z00z_wallets` for tasks `035-44`, `035-45`, and `035-46` against the authority in `035-18-PLAN.md`, `035-TODO.md`, and `035-a6-renames.md`.

The scoped file/path/include mirror wave is clean: the `wallet_io.rs` / `wallet_store.rs` / `wallet_validate.rs` filenames are live, `db/mod.rs` mirrors them correctly, the `test_bip32_manager*.inc.rs` include chain resolves, and the `test_tx_*` include graph no longer references the old `tx_impl_tests_body*.rs` names.

The previously reported declaration drifts are now fixed: `parse_core_wallet_id` is live in both split implementations and callsites, and `with_output_dir_and_time` replaced the old constructor name across the touched wallet-service surfaces.

Residual hits for old `Wlt*` / `open_wlt` spellings remain outside the approved Plan 18 rename rows, including internal helper or compatibility seams plus docs and example files under `crates/z00z_wallets/examples/` and `README.md`; per pass scope they are out of review scope unless they break build or tests, and nothing in the scoped renamed surfaces indicates that.

No significant findings remain in the scoped modified code.

---

_Reviewed: 2026-04-13T23:59:00Z_  
_Reviewer: the agent (gsd-code-reviewer)_  
_Depth: standard_
