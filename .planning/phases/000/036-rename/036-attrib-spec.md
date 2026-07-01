# Attribute Audit Spec

Date: 2026-04-21
Scope: Rust source files under `crates/**` excluding `crates/z00z_crypto/tari/**`
Phase context: `.planning/phases/036-rename`

<!-- markdownlint-disable MD060 -->

## Audit Rules

- Tables are split by attribute family, one table per requested attribute.
- Rows are exhaustive within the stated scope.
- The `Final recommendation` column is intentionally left blank when the final recommendation is `KEEP AS-IS`.
- Only non-default recommendations are written explicitly in the table, per request.
- The `Comments` column records 036-21 execution verification status; `✅` means the row was reconciled and verified against workspace artifacts for this wave.
- Recommendations are based on per-case pros/cons review of code intent, test/build gating, API contract value, and likely refactor cost.

## 📌 Table Legend

| Value in `Final recommendation` | Meaning |
| --- | --- |
| `REMOVE` | Safe to remove the attribute now. |
| `REPLACE` | Replace with narrower code/test/config change. |
| `REFACTOR` | Keep behavior, but restructure code so the attribute can be removed. |
| `NARROW` | Keep the exception, but reduce its scope. |
| *(blank)* | `KEEP AS-IS` |

## 📌 `#[allow(unused_imports)]`

Default evaluation rule: blank `Final recommendation` means `KEEP AS-IS`. Explicit recommendations below are only the cases where the current code should change.

| File | Line | Scoped item | Final recommendation | Comments |
| --- | --- | --- | --- | --- |
| `crates/z00z_utils/examples/file_io.rs` | 11 | `use serde::{Deserialize, Serialize};` | `REMOVE` | `✅` |
| `crates/z00z_storage/src/assets/store_internal/tx_plan.rs` | 1 | `use std::{ collections::{BTreeMap, BTreeSet, HashMap}, env };` | `REMOVE` | `✅` |
| `crates/z00z_storage/src/assets/store_internal/tx_plan.rs` | 21 | `use self::tx_plan_types::{ NextState, PlanMode, PlanScope, SeenOps, SerialItem, ShardItem, ShardKey, ShardPlan, StoreSnap, Touch };` | `REMOVE` | `✅` |
| `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` | 59 | `pub(crate) use self::exec_input_builder::{ build_cp_proof, build_exec_input, checkpoint_from_draft, draft_link, exec_out_from_wire, in_ref_from_input };` | `NARROW` | `✅` |
| `crates/z00z_wallets/src/db/redb_wallet_store.rs` | 108 | `use self::open::validate_objects_on_open;` | `REMOVE` | `✅` |
| `crates/z00z_wallets/src/db/redb_wallet_store_open.rs` | 11 | `pub(crate) use self::open_session::validate_objects_on_open;` | `REMOVE` | `✅` |
| `crates/z00z_wallets/src/core/tx/tx_verifier.rs` | 6 | `pub(crate) use super::tx_wire_types::{ default_regular_package_type, default_regular_tx_type, REGULAR_TX_PACKAGE_TYPE, REGULAR_TX_TYPE, TX_PACKAGE_KIND };` | `NARROW` | `✅` |

## 📌 `#[allow(deprecated)]`

| File | Line | Scoped item | Final recommendation | Comments |
| --- | --- | --- | --- | --- |
| `crates/z00z_wallets/src/core/security/encryption.rs` | 175 | `let params = Argon2Params::test_fast();` | `REMOVE` | `✅` |

## 📌 `#[allow(dead_code)]`

| File | Line | Scoped item | Final recommendation | Comments |
| --- | --- | --- | --- | --- |
| `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs` | 23 | `pub(crate) const ZERO_ROOT: [u8; 32] = [0u8; 32];` | `REMOVE` | `✅` |
| `crates/z00z_storage/benches/common/fixture.rs` | 228 | `impl BenchFx { pub fn read_paths_case(...); pub fn proof_paths_case(...); }` | `REMOVE` | `✅` |
| `crates/z00z_storage/benches/common/output.rs` | 41 | `pub(crate) fn write_note(name: &str, body: &str)` | `REMOVE` | `✅` |

## 📌 `#[allow(unreachable_code)]`

These four cases are the same structural pattern: a wasm32 `return Err(...)` branch followed by a trailing `Ok(())`. The behavior is correct, but the attribute can be eliminated by restructuring the function body into cfg-separated helpers or a cfg-selected expression.

| File | Line | Scoped item | Final recommendation | Comments |
| --- | --- | --- | --- | --- |
| `crates/z00z_wallets/src/services/wallet_service_session_derivation.rs` | 357 | trailing `Ok(())` after wasm32 guard | `REFACTOR` | `✅` |
| `crates/z00z_wallets/src/services/wallet_service_session_snapshot.rs` | 112 | trailing `Ok(())` after wasm32 guard | `REFACTOR` | `✅` |
| `crates/z00z_wallets/src/services/wallet_service_store_persistence_pack.rs` | 190 | trailing `Ok(())` after wasm32 guard | `REFACTOR` | `✅` |
| `crates/z00z_wallets/src/services/wallet_service_store_load_restore.rs` | 63 | trailing `Ok(())` after wasm32 guard | `REFACTOR` | `✅` |

## 📌 `#[allow(clippy::too_many_arguments)]`

Default evaluation rule: blank `Final recommendation` means `KEEP AS-IS`.

| File | Line | Scoped item | Final recommendation | Comments |
| --- | --- | --- | --- | --- |
| `crates/z00z_core/src/assets/definition.rs` | 238 | `pub fn new(` |  | `✅` |
| `crates/z00z_core/src/genesis/genesis_output_support.rs` | 158 | `pub(crate) fn write_genesis_report(` | `REFACTOR` | `✅` |
| `crates/z00z_simulator/src/scenario_1/stage_3_finalize.rs` | 3 | `pub(super) fn finish_claim_after_event(` | `REFACTOR` | `✅` |
| `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs` | 37 | `pub(super) fn prepare_tx_package_artifacts(` | `REFACTOR` | `✅` |
| `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs` | 306 | `pub(super) async fn persist_and_confirm_runtime(` | `REFACTOR` | `✅` |
| `crates/z00z_wallets/src/adapters/rpc/methods/tx.rs` | 5 | `#![allow(clippy::too_many_arguments)]` | `NARROW` | `✅` |
| `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring.rs` | 323 | `pub fn register_all_wallet_rpc_methods(` | `REFACTOR` | `✅` |
| `crates/z00z_wallets/src/core/wallet/snapshot_impl.rs` | 40 | `pub fn new(` | `REFACTOR` | `✅` |
| `crates/z00z_wallets/src/core/wallet/snapshot_impl.rs` | 156 | `pub fn new_with_checksum(` | `REFACTOR` | `✅` |

## 📌 Other `#[allow(clippy::...)]`

Default evaluation rule: blank `Final recommendation` means `KEEP AS-IS`.

| File | Line | Lint | Scoped item | Final recommendation | Comments |
| --- | --- | --- | --- | --- | --- |
| `crates/z00z_wallets/src/core/wallet/chain_id.rs` | 75 | `clippy::items_after_test_module` | `mod chain_id_tests {` | `REMOVE` | `✅` |

<!-- markdownlint-enable MD060 -->
