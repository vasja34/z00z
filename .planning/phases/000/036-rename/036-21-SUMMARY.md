---
phase: 036-rename
plan: 21
status: completed
updated: 2026-04-21
---

# 036-21 Summary

## Scope

This summary records the attribute-audit execution closeout for `036-21-PLAN.md`, the self-contained continuation rooted in `.planning/phases/036-rename/036-attrib-spec.md`.

## Outcome

`036-21` closes as a repository-backed reconciliation of every embedded attribute row in the `036-attrib-spec.md` matrix.

The plan removed every `REMOVE` row owned by the embedded tables, refactored every `REFACTOR` row so the attribute is no longer needed, preserved the two `NARROW` rows as intentionally reduced exceptions, and explicitly kept the one blank `KEEP AS-IS` row unchanged.

### Embedded Table Closeout

#### `#[allow(unused_imports)]`

- `crates/z00z_utils/examples/file_io.rs`: removed the attribute; the serde derives keep the import live.
- `crates/z00z_storage/src/assets/store_internal/tx_plan.rs`: removed the two parent-scope unused-import attributes and moved child-module dependencies onto direct `tx_plan_types` imports in `tx_plan_batch.rs`, `tx_plan_batches.rs`, and `tx_plan_engine.rs`.
- `crates/z00z_wallets/src/db/redb_wallet_store.rs` line-108 row: removed the targeted `validate_objects_on_open` import.
- `crates/z00z_wallets/src/db/redb_wallet_store_open.rs`: removed the targeted `validate_objects_on_open` re-export.
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`: `NARROW` closed truthfully, not by deletion. The broad grouped re-export was split so only externally needed but locally unused items remain under `#[allow(unused_imports)]`, while `build_exec_input` and `in_ref_from_input` moved to a plain re-export.
- `crates/z00z_wallets/src/core/tx/tx_verifier.rs`: `NARROW` closed truthfully. The re-export surface now contains only `REGULAR_TX_PACKAGE_TYPE`, `REGULAR_TX_TYPE`, and `TX_PACKAGE_KIND`.

#### `#[allow(deprecated)]`

- `crates/z00z_wallets/src/core/security/encryption.rs`: removed the targeted `#[allow(deprecated)]` on `Argon2Params::test_fast()` without widening the fast-path surface.

#### `#[allow(dead_code)]`

- `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs`: removed the `ZERO_ROOT` attribute and kept the constant live by routing `rebuild_def(...)` through it.
- `crates/z00z_storage/benches/common/fixture.rs`: removed the `dead_code` attribute while preserving `read_paths_case()` and `proof_paths_case()` because they have live bench callers.
- `crates/z00z_storage/benches/common/output.rs`: removed the `dead_code` attribute while preserving `write_note(...)` because it has a live bench caller.

#### `#[allow(unreachable_code)]`

- `crates/z00z_wallets/src/services/wallet_service_session_derivation.rs`: replaced the cfg-tail pattern with cfg-selected `persist_session_snapshot(...)` helpers.
- `crates/z00z_wallets/src/services/wallet_service_session_snapshot.rs`: replaced the cfg-tail pattern with cfg-selected `persist_session_snapshot(...)` helpers.
- `crates/z00z_wallets/src/services/wallet_service_store_persistence_pack.rs`: replaced the cfg-tail pattern with cfg-selected `save_wallet_wlt(...)` helpers.
- `crates/z00z_wallets/src/services/wallet_service_store_load_restore.rs`: replaced the cfg-tail pattern with cfg-selected `load_wallet_snapshot_bytes(...)` helpers.

All four wasm32 boundaries remain fail-closed and explicit.

#### `#[allow(clippy::too_many_arguments)]`

- `crates/z00z_core/src/assets/definition.rs::new()`: explicit `KEEP AS-IS`. This row was blank in the source table and remains unchanged in this plan.
- `crates/z00z_core/src/genesis/genesis_output_support.rs::write_genesis_report(...)`: refactored to consume `GenesisReportArgs`.
- `crates/z00z_simulator/src/scenario_1/stage_3_finalize.rs::finish_claim_after_event(...)`: refactored to consume `ClaimFinalizeArgs`.
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs::prepare_tx_package_artifacts(...)`: refactored to consume `TxPrepArgs`.
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs::persist_and_confirm_runtime(...)`: refactored to consume `TxRuntimeArgs`.
- `crates/z00z_wallets/src/adapters/rpc/methods/tx.rs`: `NARROW` closed truthfully. The module-wide and trait-wide escape hatches were removed; the exact item-level allowance on the RPC method surface remains.
- `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring.rs::register_all_wallet_rpc_methods(...)`: refactored behind `RpcModules`.
- `crates/z00z_wallets/src/core/wallet/snapshot_impl.rs::new(...)`: refactored to consume `WalletSnapshotArgs`.
- `crates/z00z_wallets/src/core/wallet/snapshot_impl.rs::new_with_checksum(...)`: refactored to consume `WalletSnapshotArgs`.

#### `clippy::items_after_test_module`

- `crates/z00z_wallets/src/core/wallet/chain_id.rs`: removed the targeted attribute; the module now satisfies the ordering rule without the escape hatch.

## Residual Attribute Truth

The plan-owned residue scan over the 036-21 file set reports only three remaining `allow(...)` markers:

- `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`: expected `NARROW` residual for externally needed re-exports.
- `crates/z00z_wallets/src/adapters/rpc/methods/tx.rs`: expected item-level `NARROW` residual for the RPC signature that still legitimately exceeds the Clippy threshold.
- `crates/z00z_wallets/src/db/redb_wallet_store.rs` line 93: pre-existing unrelated `#[allow(unused_imports)]` row outside the embedded `036-21` matrix; this plan did not claim ownership of that row.

`036-21` therefore closes the embedded matrix truthfully without claiming a zero-allow state outside its own source table.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`: passed after the Task B/C refactor pass.
- `cargo test -p z00z_wallets --release --features test-fast --test test_open_wallet_source_discovery`: passed.
- `cargo test -p z00z_wallets --release --features test-fast --test test_wallet_export_pack_boundary`: passed.
- `cargo test -p z00z_wallets --release --features test-fast --test test_rpc_dispatcher_roundtrip`: passed.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump`: passed on the authoritative rerun that was allowed to reach process completion after the earlier terminated session.
- `rg -n '#\[allow\((unused_imports|deprecated|dead_code|unreachable_code)\)\]|#\[allow\(clippy::items_after_test_module\)\]|#\[allow\(clippy::too_many_arguments\)\]|#!\[allow\(clippy::too_many_arguments\)\]' ...plan-owned file set...`: reports only the two intentional `NARROW` residuals plus one unrelated pre-existing row outside the matrix.

The simulator release command emitted unrelated pre-existing `unused import: Z00ZCommitment` warnings in simulator tests; no failures were observed and the changed stage 3 or stage 4 surfaces stayed green across unit, integration, and end-to-end lanes.

## Review Loop

The environment still does not expose `/.github/prompts/gsd-review-tasks-execution.prompt.md` as a direct runnable CLI entrypoint, so the review evidence remains the repository-backed substitute.

- Review pass 1: exact-context reread of the Task A/B/C edits plus file-level diagnostics found one real issue during execution (`load_wallet_snapshot_bytes` type and equality drift), which was fixed before validation.
- Review pass 2: post-fix diagnostics, targeted release tests, and the plan-owned residue scan found no significant issues inside the embedded `036-21` table scope.
- Review pass 3: the mandatory bootstrap gate, targeted wallet tests, and the preferred full `z00z_simulator` release command remained green, so the second consecutive no-significant-issue review pass is satisfied for the `036-21` scope.

## Canonical Artifact Sync

- `.planning/phases/036-rename/036-21-PLAN.md`
- `.planning/phases/036-rename/036-21-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`

## Current Boundary

`036-21` is summary-backed complete as the self-contained attribute-remediation continuation.

Phase 036 still remains open, but not because of the attribute matrix. The open boundary remains the separate partial shim-removal truth recorded in `036-20-SUMMARY.md`. This summary does not overwrite or supersede that partial closeout.
