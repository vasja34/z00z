# 035-18 Summary

## Scope

This summary records the completion state for `035-18-PLAN.md`, covering task
`035-44 File Rename Wave B - Wallet DB And Egui Canonical Files`, task
`035-45 Signature Rename Wave A - Module, Path, And Include Mirrors`, and task
`035-46 Signature Rename Wave B - Types, Functions, And Methods`.

## Outcome

Plan 18 is fully closed.

Phase 035 now has the second curated rename slice closed on repository-backed
filename, mirror, and declaration evidence rather than on partial rename drift.
The approved wallet DB rename rows are live under the canonical
`wallet_io.rs`, `wallet_store.rs`, and `wallet_validate.rs` filenames, the
approved egui rename rows now resolve through their canonical tab filenames,
the required module or include mirrors point at those final filenames, and the
plan-owned declaration drift is closed with `parse_core_wallet_id` and
`with_output_dir_and_time` landing on the approved spellings while `WltStore`
remains preserved on the store seam.

Residual old-name hits remain outside the approved Plan 18 rename rows,
including internal helper or compatibility seams plus docs and examples
outside the Plan 18 review surface; they were explicitly treated as out of
scope because the live code, tests, and review loop stayed clean after the
final corrections on the curated rename slice.

## Repository Changes

- `.planning/phases/035-mix2-fixes/035-TODO.md` now records `035-44`,
  `035-45`, and `035-46` as closed checklist items while leaving the later
  cross-file sweep and acceptance rows for Plan 19.
- `.planning/phases/035-mix2-fixes/035-18-SUMMARY.md` now captures the
  closeout evidence and boundary for this second rename slice.
- `crates/z00z_wallets/src/db/wallet_io.rs`,
  `crates/z00z_wallets/src/db/wallet_store.rs`, and
  `crates/z00z_wallets/src/db/wallet_validate.rs` remain the canonical wallet
  DB filenames for the approved Wave B rows, and
  `crates/z00z_wallets/src/db/mod.rs` mirrors them correctly.
- `crates/z00z_wallets/src/egui_views/add_wallet_tab.rs`,
  `crates/z00z_wallets/src/egui_views/app_create_wallet_tab.rs`,
  `crates/z00z_wallets/src/egui_views/app_logout_tab.rs`,
  `crates/z00z_wallets/src/egui_views/app_logs_tab.rs`, and the approved
  `network_*_tab.rs` rows now hold the plan-owned egui filename renames.
- `crates/z00z_wallets/src/services/wallet_service_session_derivation.rs`,
  `crates/z00z_wallets/src/services/wallet_service_session_seed_derivation.rs`,
  `crates/z00z_wallets/src/services/wallet_service_session_guards.rs`,
  `crates/z00z_wallets/src/services/wallet_service_session_runtime.rs`, and
  `crates/z00z_wallets/src/services/wallet_service_session_snapshot.rs` now use
  the curated `parse_core_wallet_id` spelling.
- `crates/z00z_wallets/src/services/wallet_service_session_build.rs`,
  `crates/z00z_wallets/src/services/wallet_service_session_construction.rs`,
  `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_impl_suite.rs`,
  `crates/z00z_wallets/src/adapters/rpc/methods/wallet_impl_tests.rs`,
  `crates/z00z_wallets/tests/test_tx_store_integration.rs`, and
  `crates/z00z_wallets/tests/test_stub_behavior.rs` now use the curated
  `with_output_dir_and_time` constructor spelling.
- `crates/z00z_wallets/src/services/wallet_service_types.rs` preserves the
  approved `WltStore` store-trait import instead of drifting to the rejected
  `WalletStore` spelling.
- `.planning/ROADMAP.md` and `.planning/STATE.md` now advance continuity to
  `035-19-PLAN.md`.

## Validation

- Post-fix file diagnostics on the touched wallet service and test surfaces:
  clean.
- `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump`:
  passed.
- `cargo test --release --features test-fast --features wallet_debug_dump`:
  passed on a fresh tracked rerun, including the late simulator acceptance
  suites and the doc-test tail for `z00z_crypto`, `z00z_storage`,
  `z00z_utils`, and `z00z_wallets`.
- `.planning/phases/035-mix2-fixes/035-18-REVIEW.md` is clean and records zero
  remaining findings on the final Plan 18 review surface.

## Review Loop

The mandatory `GSD-Review-Tasks-Execution` loop exceeded the minimum
three-review requirement before closure was accepted.

- Review pass 3 blocked on two remaining curated declaration drifts:
  `core_wallet_id_rpc_wallet` still needed to become `parse_core_wallet_id`,
  and `with_output_dir_and_dependencies` still needed to become
  `with_output_dir_and_time`.
- Review pass 4 was the first clean pass after those final declaration and
  callsite corrections landed.
- Review pass 5 was the second consecutive clean pass on that corrected Plan 18
  surface, which satisfied the closeout rule.

Closure is accepted only on clean review passes 4 and 5 after the final
rename-drift corrections were applied.

## Current Boundary

This summary closes only the Phase 035 rename slice for `035-44`, `035-45`,
and `035-46`. It does not claim completion of the later cross-file reference
sweep, no-change guard verification, acceptance closure, or any broader
Phase 035 work reserved for `035-19-PLAN.md`.
