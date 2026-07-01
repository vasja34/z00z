# Phase 047 Wallet Addon Specification v2

**Status:** Post-redesign addon truth specification  
**Date:** 2026-05-20  
**Primary crates:** `z00z_wallets`, `z00z_simulator`  
**Relationship:** This document adapts the useful content from `047-wallet-addon-spec.md` to the landed Phase 047 codebase. It does not replace `047-wallet-redesign-spec.md`, which remains the canonical architecture and migration specification for the redesign itself.

## 🎯 Purpose

This file is the current-state addon specification for the Phase 047 wallet redesign after the main storage cutover landed. The old addon spec was written in future tense as an implementation packet for simulator, lifecycle, backup/restore, and wording follow-up work. That implementation work is now largely complete.

The role of `047-wallet-addon-spec2.md` is different:

- Preserve the valuable wallet-runtime and simulator truth from `047-wallet-addon-spec.md`.
- Reframe landed work as verified baseline facts instead of pending tasks.
- Keep the compatibility boundaries, non-goals, and wording guardrails that still matter after the redesign.
- Record only evidence-backed future follow-up items, not already-completed Phase 047 work.

## 📌 Document Position

Use the three Phase 047 documents this way:

1. `047-wallet-redesign-spec.md` is the canonical design and migration specification.
2. `047-wallet-addon-spec.md` is the historical implementation-oriented addon packet that drove the final simulator and wording waves.
3. `047-wallet-addon-spec2.md` is the post-cutover addon truth spec for the current repository state.

This means `spec2` must not reopen already-closed Phase 047 steps such as adding `OwnedAssetPayload`, wiring Stage 13, or moving the tx lifecycle to owned-asset authority. Those are already part of the verified baseline.

## ✅ Verified Baseline After Phase 047

### 🔑 Profile And Config Authority

Normal wallet create, save, reopen, and recovery flows now persist profile/config state through `WalletProfilePayload`, and live defaults come from YAML-backed wallet config instead of hardcoded literals.

Evidence anchors:

- `047-03-SUMMARY.md`
- `047-VALIDATION.md`
- `crates/z00z_wallets/src/wallet_config.yaml`
- `crates/z00z_wallets/src/db/redb_wallet_store/tables.rs`

Operational meaning:

- Ordinary non-asset saves do not treat Snapshot as the main wallet authority.
- Profile-only `.wlt` persistence is the normal path.
- Legacy snapshot handling remains compatibility-only.

### 🔑 Owned-Asset Authority

The live wallet-owned asset authority is now the encrypted `.wlt` object store via `OwnedAssetPayload` and the internal `WalletAssetStore` implementation.

Evidence anchors:

- `047-04-SUMMARY.md`
- `047-VALIDATION.md`
- `crates/z00z_wallets/src/db/redb_wallet_store/tables.rs`
- `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_reachability.rs`

Operational meaning:

- One owned asset maps to one encrypted `.wlt` object with one stable `object_id`.
- Duplicate insertions fail closed unless they are explicitly idempotent.
- The in-memory `wallet_claimed_assets` map is cache/fallback state, not durable truth.

### 🔑 Receive And Scan Persistence

The live receive/scan path now persists discovered owned assets and scan cursor progression on the same wallet-native authority plane.

Evidence anchors:

- `047-05-SUMMARY.md`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs`
- `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs`
- `crates/z00z_wallets/src/db/redb_wallet_store/mutations/upserts.rs`

Operational meaning:

- `recv_range(...)` remains the wallet-side ownership detector.
- Scan-hit asset persistence and `ScanStatePayload` progression are coupled through one `.wlt` write transaction.
- Replay of the same scan chunk is idempotent when the owned-asset wire payload matches.

### 🔑 Canonical Transaction Lifecycle

The canonical live spend lifecycle is `wallet.tx.*`, and the build, reservation, cancel, import, broadcast, and reconcile paths now operate through owned-asset authority rather than Snapshot-owned claim vectors.

Evidence anchors:

- `047-06-SUMMARY.md`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_helpers.rs`

Operational meaning:

- `wallet.tx.build_transaction` selects only spendable owned assets and excludes reserved inputs.
- Cancel releases only the reservation associated with the cancelled `tx_id`.
- Import and reconcile update owned-asset state without rewriting a full Snapshot vector.
- Evidence mismatch or finalize failure leaves wallet asset state fail-closed.

### 🔑 Backup, Restore, And Export Boundary

Backup/export is now a manifest-backed explicit wallet-state pack built from profile-first `.wlt` state, owned assets, scan state, and the separate tx-history plane.

Evidence anchors:

- `047-07-SUMMARY.md`
- `047-VALIDATION.md`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup.rs`
- `crates/z00z_wallets/src/wallet/snapshot/snapshot_types.rs`

Operational meaning:

- `WalletPlusHistory` restores a staged `.wlt` plus JSONL tx-history sidecar.
- New exports do not use Snapshot as the main live state carrier.
- Snapshot survives only as a one-shot legacy import/restore bridge when no explicit profile pack exists.

### 🔑 Stage 13 Simulator Proof

Scenario 1 Stage 13 is no longer a planned addon. It is a landed lifecycle proof over the live wallet paths.

Evidence anchors:

- `047-08-SUMMARY.md`
- `crates/z00z_simulator/src/scenario_1/stage_13.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/flow.rs`
- `crates/z00z_simulator/src/scenario_1/runner_contract_table.in`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`

Operational meaning:

- Stage 13 exercises the real `wallet.tx.*` lifecycle.
- The stage proves backup/restore, tamper rejection, reopen continuity, and honest wording for the current owned-asset authority model.
- The stage does not treat Snapshot as the target live authority.

### 🔑 Truth And Live-Path Enforcement

Phase 047 now has explicit truth tests that reject stale wording and placeholder framing on the wallet/RPC/simulator surface.

Evidence anchors:

- `crates/z00z_wallets/tests/test_phase047_truth.rs`
- `crates/z00z_wallets/tests/test_live_path_enforcement.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- `047-VALIDATION.md`

Operational meaning:

- Old `stub`, `placeholder`, and `Phase 1` narratives are no longer acceptable on the landed Phase 047 surfaces.
- `wallet.asset.import_asset` is enforced as a live verification path.
- Stage 13 wording is pinned to the post-redesign storage truth.

## 🔄 What From The Old Addon Spec Still Matters

The original `047-wallet-addon-spec.md` still contains useful material, but not every section should survive unchanged.

| Old section class | v2 treatment | Reason |
| --- | --- | --- |
| `Verified Current State` | Keep, but rewrite as current baseline facts | Most of the important boundaries are still valid, but they are now landed facts instead of setup for future work. |
| `Phase Decisions` | Keep as current invariants and guardrails | The decisions around authority boundaries, Stage 13 honesty, backup scope, and compatibility surfaces remain useful. |
| `Required Implementation Order` and `Implementation Details` | Rewrite as historical provenance, not pending plan | The numbered 047-01..08 plan/summaries already closed this work. |
| `Required Tests` | Keep as verification anchors | The value is now in proving the runtime truth through current tests and summaries. |
| `Acceptance Criteria` | Keep, but rewrite into present-tense runtime contract | The criteria are valuable only if they reflect current behavior. |
| `Doublecheck Register` | Keep and refresh | This is still the best compact inventory of the live seams that matter. |
| `Non-Goals` | Keep and refresh | The non-goals still protect the design from drift. |
| `Definition of Done` | Replace with closeout position | Phase 047 is already complete on its own planning packet. |

## ⚠️ Current Truth Constraints And Guardrails

### 🔑 Snapshot Is Compatibility-Only

`WalletPersistenceState.claimed_assets` still exists in the codebase, but it must be treated as a compatibility bridge instead of the live wallet-owned asset authority.

Evidence anchors:

- `crates/z00z_wallets/src/wallet/snapshot/snapshot_types.rs`
- `047-07-SUMMARY.md`
- `047-08-SUMMARY.md`

Required guardrail:

- Do not describe Snapshot as the target or default live claimed-asset plane.

### 🔑 Tx History Remains A Separate JSONL Plane

Phase 047 did not move canonical transaction history into `.wlt`. The live history plane remains `wallet_<stem>_tx_history.jsonl`.

Evidence anchors:

- `047-wallet-redesign-spec.md`
- `047-07-SUMMARY.md`
- `047-08-SUMMARY.md`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs`

Required guardrail:

- Do not claim a single-plane `.wlt`-only persistence model for tx history.

### 🔑 Compatibility `wallet.asset.*` Mutations Stay Non-Canonical

`wallet.asset.send_asset`, `split_asset`, `merge_assets`, `stake_assets`, `swap_assets`, and `unstake_assets` remain compatibility or UX surfaces rather than confirmed-ledger authority.

Evidence anchors:

- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_ops.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_rpc_stakes.rs`
- `crates/z00z_wallets/tests/test_phase047_truth.rs`

Required guardrails:

- Do not describe those methods as canonical spend authority.
- Keep compatibility stake IDs explicit as `compat_stake_*` echo surfaces, not a shadow write plane.

### 🔑 Receive Ownership Detection Stays Wallet-Side

Remote chunks, proofs, or future aggregator inputs remain evidence sources only. Ownership detection stays in the wallet lane around `recv_range(...)` and `StealthOutputScanner`.

Evidence anchors:

- `047-05-SUMMARY.md`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs`
- `crates/z00z_wallets/src/receiver/scan/stealth_scanner.rs`
- `crates/z00z_wallets/src/chain/scan_engine_impl.rs`

Required guardrails:

- Do not promote `ScanEngineImpl` to a live ownership authority.
- If a future worker split appears, keep the authority wallet-owned.

### 🔑 Master-Key Rotation Boundary Is Narrower Than Durable Seed Rotation

`wallet.key.rotate_master_key` remains valuable to document, but only as an authenticated, rate-limited, audited in-memory rederive boundary.

Evidence anchors:

- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_rpc.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_rotation.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/support.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/tests.rs`

Required guardrail:

- Do not claim persisted seed rotation or durable master-key rewrite unless a separate future phase lands it.

## 📋 Residual Work After The Landed Phase 047

No uncovered implementation gap was found inside the audited Phase 047 redesign surface. `047-VALIDATION.md` records full routed coverage for the landed phase.

The remaining items are future follow-up opportunities, not current Phase 047 defects:

1. Retire Snapshot compatibility paths only when legacy import/export and restore no longer need them.
2. Move canonical tx history into `.wlt` only through a new explicit storage phase; do not smuggle that claim into Phase 047.
3. Rework compatibility `wallet.asset.*` mutation RPCs onto canonical `wallet.tx.*` semantics if product requirements ever demand confirmed-ledger behavior there.
4. Land durable persisted seed/master-key rotation only as a separate security and migration slice.
5. If remote scan sourcing or worker split is expanded later, preserve wallet-side ownership detection and keep remote/JMT surfaces evidence-only.

## 🧪 Current Verification Anchors

The highest-signal proof surfaces for this addon scope are:

1. `047-03-SUMMARY.md` through `047-08-SUMMARY.md`
2. `047-SPEC-COVERAGE.md`
3. `047-VALIDATION.md`
4. `crates/z00z_wallets/tests/test_phase047_truth.rs`
5. `crates/z00z_wallets/tests/test_live_path_enforcement.rs`
6. `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`

Recommended focused commands:

```bash
cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_phase047_truth -- --nocapture
cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_live_path_enforcement -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture
```

Recommended simulator proof run:

```bash
cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump
```

Recommended final document/truth check:

```bash
rg -n "Snapshot authority|claimed_assets persistence remains" .planning/phases/047-wallet-redesign crates/z00z_wallets crates/z00z_simulator
```

## ✅ Runtime Contract (EARS)

- WHEN a wallet scan detects an owned output and persists it through the live receive lane, THE SYSTEM SHALL write the owned asset and scan cursor through the wallet-native `.wlt` authority plane.
- WHERE wallet-owned asset persistence is described, THE SYSTEM SHALL treat encrypted `.wlt` `OwnedAssetPayload` objects as canonical live authority and SHALL treat Snapshot `claimed_assets` as compatibility-only.
- WHEN `wallet.tx.build_transaction` selects inputs, THE SYSTEM SHALL select only spendable owned assets and SHALL exclude inputs reserved by pending transactions.
- WHEN `wallet.tx.cancel_transaction` succeeds, THE SYSTEM SHALL release only the reservation associated with the cancelled transaction.
- WHEN `wallet.tx.import_transaction` or `wallet.tx.reconcile_transaction` succeeds, THE SYSTEM SHALL update owned-asset state through the live authority plane without rewriting a full Snapshot vector.
- IF confirmation evidence, imported package state, or finalize-time asset persistence is inconsistent, THEN THE SYSTEM SHALL fail closed and SHALL leave wallet asset state coherent.
- WHERE tx history is described, THE SYSTEM SHALL treat `wallet_<stem>_tx_history.jsonl` as the canonical live history plane unless a later phase explicitly changes that boundary.
- WHERE send-like `wallet.asset.*` methods are described, THE SYSTEM SHALL label them as compatibility or UX surfaces and SHALL NOT describe them as canonical confirmed-ledger authority.
- WHEN Stage 13 is described or tested, THE SYSTEM SHALL describe the real `wallet.tx.*` lifecycle over owned assets plus canonical JSONL history and SHALL NOT revert to Snapshot-as-authority wording.
- WHEN `wallet.key.rotate_master_key` is described, THE SYSTEM SHALL describe the current in-memory rederive and security boundary only and SHALL NOT claim persisted seed rotation.

## 🔍 Doublecheck Register

| Area | Live anchor | Why it still matters |
| --- | --- | --- |
| Owned-asset authority | `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs` | Prevents drift back toward Snapshot-owned state. |
| Profile-first wallet save/open | `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_create_unlock_open.rs` | Keeps normal `.wlt` behavior honest after the redesign. |
| Receive/scan truth | `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs` | Protects wallet-side ownership detection and scan-state coupling. |
| Tx lifecycle | `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs` | Keeps build/reserve/cancel behavior on the owned-asset plane. |
| Tx finalize / reconcile | `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs` | Protects fail-closed import/reconcile semantics. |
| Backup/restore staging | `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup.rs` | Preserves explicit `.wlt` plus JSONL restore boundary. |
| Snapshot compatibility | `crates/z00z_wallets/src/wallet/snapshot/snapshot_types.rs` | Keeps the compatibility bridge from being misdescribed as live authority. |
| Stage 13 lifecycle proof | `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/flow.rs` | Keeps simulator truth aligned to the landed runtime. |
| Stage 13 truth guard | `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` | Blocks wording drift on the simulator surface. |
| Live-path enforcement | `crates/z00z_wallets/tests/test_live_path_enforcement.rs` | Blocks regression back to placeholder import behavior. |
| Wording honesty | `crates/z00z_wallets/tests/test_phase047_truth.rs` | Blocks stale `stub`/`Phase 1`/placeholder narratives on Phase 047 surfaces. |
| Compatibility stake IDs | `crates/z00z_wallets/src/adapters/rpc/methods/asset_rpc_stakes.rs` | Prevents compatibility staking from becoming a shadow mutable authority plane. |

## 🚫 Non-Goals

- Do not treat this document as a replacement for `047-wallet-redesign-spec.md`.
- Do not reopen already-landed Phase 047 implementation steps as if they were pending.
- Do not describe Snapshot as the target live asset authority.
- Do not describe tx history as already migrated into `.wlt`.
- Do not describe compatibility `wallet.asset.*` mutation surfaces as canonical confirmed-ledger authority.
- Do not describe `ScanEngineImpl` or any remote/JMT-side service as the live ownership detector for this wallet path.
- Do not describe `wallet.key.rotate_master_key` as durable seed rotation.

## ✅ Closeout Position

The old addon spec remains useful as historical provenance, but its future-tense implementation blocks are no longer the right shape for the current repository. The landed Phase 047 state is now:

- profile-first `.wlt` persistence,
- object-backed owned-asset authority,
- wallet-side receive/scan persistence,
- owned-asset-backed `wallet.tx.*` lifecycle,
- explicit `.wlt` plus JSONL backup/restore,
- and Stage 13 proof plus truth tests that keep those claims honest.

Read `047-wallet-addon-spec2.md` as the current addon truth and guardrail document for that landed state.

---
