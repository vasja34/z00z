<!-- markdownlint-disable MD001 MD022 MD032 MD033 MD041 MD047 -->
# Phase 064: Gaps-Closing-3 - Context

**Gathered:** 2026-06-30  
**Status:** Planned from the current-tree Phase 064 corpus on the existing
`.planning/phases/064-Gaps-Closing-3/` directory only  
**Source:** PRD Express Path (`.planning/phases/064-Gaps-Closing-3/064-TODO.md`, the referenced Markdown corpus,
and the live simulator, wallet, storage, runtime, rollup, and core anchors)

<domain>
## 🎯 Phase Boundary

Phase 064 converts the current `.planning/phases/064-Gaps-Closing-3/064-TODO.md` recommendation audit into an
executable local-only closure packet for simulator truth, packet artifact
homes, wallet asset mutation live paths, RPC truth, wallet-sensitive surfaces,
checkpoint and snapshot boundaries, local DA/runtime adversarial coverage, and
repository boundary guardrails.

`.planning/phases/064-Gaps-Closing-3/064-TODO.md` is normative, not advisory. The existing
`.planning/phases/064-Gaps-Closing-3/` folder is the only canonical Phase 064
root. Historical tracked side files removed from the current worktree are not
silently restored and do not become a second planning authority.

### Current-Tree Planning Rule

- `.planning/phases/064-Gaps-Closing-3/064-TODO.md` contains zero canonical `TASK-NNN` rows and zero explicit
  grouped-plan rows.
- The invoking prompt still speaks in the older Phase 062 `TASK-NNN` dialect.
- Phase 064 planning therefore follows the accepted current-tree Phase 063
  precedent: use the current recommendation inventory as canonical execution
  scope and introduce planning-local `REC-064-*` traceability ids only.
- `REC-064-*` ids are coverage handles for this packet. They do not pretend
  that missing `TASK-NNN` rows exist, and they must never become a second
  authority namespace.

### Phase 064 Delivers

1. `.planning/phases/064-Gaps-Closing-3/064-CONTEXT.md` with a recommendation-to-plan transfer table and a
   current-tree coverage answer.
2. Ordered executable plans `.planning/phases/064-Gaps-Closing-3/064-01-PLAN.md` through `.planning/phases/064-Gaps-Closing-3/064-05-PLAN.md`.
3. One canonical local gap-closing packet with no duplicate simulator,
   wallet, storage, runtime, or network authority layer.

### Phase 064 Does Not Deliver

- No invented `TASK-NNN` inventory.
- No restored historical side docs as live authority.
- No real network, real OnionNet, real remote-chain, or real DA live claim.
- No compile-only, docs-only, placeholder-only, or panic-only closure path.
- No edits under `crates/z00z_crypto/tari/**`.

</domain>

<decisions>
## ⚙️ Locked Decisions

- **D-01:** `.planning/phases/064-Gaps-Closing-3/064-TODO.md` remains the single canonical planning authority for
  Phase 064.
- **D-02:** The current-tree execution inventory is 28 recommendation rows
  grouped into 5 ordered numbered plans.
- **D-03:** `REC-064-*` labels are planning-local coverage ids only and must
  never be promoted into fake canonical task ids.
- **D-04:** `scenario_1` remains the canonical executable home for simulator
  closure; default publication must become truthful before downstream local
  claims expand.
- **D-05:** Wallet local mutation work stays on the `LocalNodeSim` /
  `ChainClientImpl` / `BroadcastImpl` / `TxStorage` path. Real remote
  transport remains adapter-only.
- **D-06:** `wallet.object.*` stays a live post-genesis typed-object path;
  planning must not waste execution waves re-proving its existence instead of
  closing the real gaps around it.
- **D-07:** Storage owns settlement, checkpoint, snapshot, and proof truth.
  Runtime owns planning, placement, publication binding, and local
  distributed simulation. Rollup consumes public theorem bundles only.
- **D-08:** Core and genesis cleanup remains subordinate to simulator, wallet,
  and runtime truth-restoration, not the other way around.
- **D-09:** OnionNet, real remote chain, real DA transport, slashing, and
  fraud-engine claims remain explicitly deferred until local closure is
  complete and a real transport implementation exists.
- **D-10:** Every plan `<verify>` block starts with
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`, then
  runs slice-specific commands, runs `cargo test --release` when Rust or
  tests are affected, runs `/GSD-Review-Tasks-Execution` at least three times
  until two consecutive clean runs, and uses `/z00z-git-versioning` if a
  commit is needed.
- **D-11:** No plan or implementation step may duplicate existing codebase
  logic, invent a parallel abstraction, or create a second semantic truth lane
  when a live owner crate, module, or surface already exists.
- **D-12:** Graphify is Phase 064 orientation-only. It may help with codebase
  structure discovery, but it must never be cited as factual authority for
  coverage, invariants, source truth, or acceptance claims.
- **D-13:** The TODO closeout-order directive remains explicit as numbered
  groups `1-5`, `6-13`, and `14-18`. Any pull-forward across those groups is
  allowed only when the item is an owner-boundary guard inseparable from the
  same canonical surface, and that exception must be recorded in the ordered
  closeout contract.

</decisions>

<threat_model>
## 🛡️ Threat Model And Trust Boundaries

- **Assets:** final checkpoint artifacts, emitted packet inventories, wallet
  session-gated sensitive actions, typed-object admissions, settlement roots
  and proofs, publication bindings, route tables, and repository boundary CI
  guardrails.
- **Adversaries:** placeholder drift, duplicate-owner refactors, stale
  documentation claims, route-registration gaps, raw-save bypasses, detached
  theorem inputs, secret leakage into default packets, and accidental
  live-network wording.
- **Trust boundaries:**
  - `crates/z00z_simulator/src/scenario_1/**` is the canonical local simulator
    evidence surface; packet prose, ad hoc scripts, or filtered harnesses are
    not second authorities.
  - `crates/z00z_wallets/src/chain/**` and
    `crates/z00z_wallets/src/rpc/**` own wallet-local mutation and RPC truth;
    remote-chain adapters stay non-live seams.
  - `crates/z00z_storage/src/**` owns checkpoint, snapshot, settlement, and
    proof truth; `crates/z00z_runtime/aggregators/**` owns planning, recovery,
    route tables, and publication binding; `crates/z00z_rollup_node/src/**`
    consumes public theorem bundles only.
  - `crates/z00z_crypto/**` is the only workspace crypto facade; vendor
    `crates/z00z_crypto/tari/**` remains read-only.
- **Failure model:**
  - compile-only, docs-only, placeholder-only, and grep-only proof is invalid;
  - any second simulator, wallet mutation, checkpoint, publication-binding, or
    network-truth lane is phase-failing;
  - fake live-network, fake DA, or fake OnionNet claims are phase-failing
    until a real owner exists and local proof is extended truthfully.

</threat_model>

<canonical_refs>
## 📚 Canonical References

### Planning Authority

- `.planning/phases/064-Gaps-Closing-3/064-TODO.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/GSD-Workflow.md`
- `.github/copilot-instructions.md`
- `.github/requirements/Z00Z_DESIGN_FOUNDATION.md`

### Source Markdown Corpus Named By `.planning/phases/064-Gaps-Closing-3/064-TODO.md`

- `crates/z00z_simulator/README.md`
- `crates/z00z_networks/onionnet/README.md`
- `crates/z00z_runtime/aggregators/README.md`
- `crates/z00z_storage/src/settlement/README.md`
- `crates/z00z_utils/README.md`
- `crates/z00z_crypto/README.md`
- `crates/z00z_extensions/README.md`
- `docs/Z00Z-Main-Whitepaper.md`
- `docs/tech-papers/Z00Z-Roadmap-Blueprint.md`
- `wiki/03-core-protocol/genesis-caveats.md`
- `wiki/04-wallet-and-rpc/receiver-request-flow.md`
- `wiki/04-wallet-and-rpc/wallet-object-packages.md`
- `wiki/04-wallet-and-rpc/wallet-object-quarantine.md`
- `wiki/04-wallet-and-rpc/wallet-stub-surface.md`
- `wiki/05-storage-runtime/prep-snapshot-replay.md`
- `wiki/06-simulator-and-quality/scenario-pipeline.md`
- `wiki/06-simulator-and-quality/scenario1-object-artifacts.md`

### Live Code Anchors

- `crates/z00z_simulator/src/config.rs`
- `crates/z00z_simulator/src/config/config_accessors.rs`
- `crates/z00z_simulator/src/scenario_1/stage_9/bundle_lane_impl.rs`
- `crates/z00z_simulator/src/scenario_1/stage_12/mod.rs`
- `crates/z00z_simulator/src/scenario_1/stage_12/finalize_flow.rs`
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
- `crates/z00z_simulator/src/scenario_1/support/fixture_cache.rs`
- `crates/z00z_simulator/src/scenario_1/support/stage_runner_support.rs`
- `crates/z00z_wallets/src/chain/local_node_sim.rs`
- `crates/z00z_wallets/src/chain/chain_client_impl.rs`
- `crates/z00z_wallets/src/chain/broadcast_impl.rs`
- `crates/z00z_wallets/src/rpc/asset_rpc_server_ops.rs`
- `crates/z00z_wallets/src/rpc/asset_rpc_server_catalog.rs`
- `crates/z00z_wallets/src/rpc/object_rpc_impl.rs`
- `crates/z00z_wallets/src/rpc/app_rpc_impl.rs`
- `crates/z00z_wallets/src/rpc/app_dispatcher_wiring.rs`
- `crates/z00z_wallets/src/rpc/wallet_dispatcher_routes.rs`
- `crates/z00z_wallets/scripts/audit_rpc_method_wiring.py`
- `crates/z00z_wallets/src/services/wallet_actions_backup.rs`
- `crates/z00z_wallets/src/services/wallet_session_manager.rs`
- `crates/z00z_wallets/src/services/wallet_session_runtime_limits.rs`
- `crates/z00z_wallets/src/services/wallet_store_restore.rs`
- `crates/z00z_wallets/src/services/wallet_actions_receive.rs`
- `crates/z00z_wallets/src/receiver/request.rs`
- `crates/z00z_wallets/src/stealth/output.rs`
- `crates/z00z_wallets/src/redb_store/owned_objects.rs`
- `crates/z00z_wallets/src/redb_store/object_queries.rs`
- `crates/z00z_storage/src/checkpoint/store.rs`
- `crates/z00z_storage/src/snapshot/store.rs`
- `crates/z00z_storage/src/settlement/object_package_contract.rs`
- `crates/z00z_rollup_node/src/lib.rs`
- `crates/z00z_rollup_node/src/da.rs`
- `crates/z00z_runtime/aggregators/src/batch_planner.rs`
- `crates/z00z_runtime/aggregators/src/recovery.rs`
- `crates/z00z_runtime/aggregators/src/types.rs`
- `crates/z00z_core/src/genesis/genesis_run.rs`
- `crates/z00z_core/src/vouchers/voucher_bootstrap.rs`
- `crates/z00z_crypto/src/lib.rs`

</canonical_refs>

<normative_mirror>
## 🧭 Inventory And Coverage Answer

| Inventory item | Count | Decision |
| --- | ---: | --- |
| Canonical recommendation rows mirrored into this packet | 28 | Phase 064 execution inventory |
| Canonical `TASK-NNN` rows in `.planning/phases/064-Gaps-Closing-3/064-TODO.md` | 0 | Must not be invented |
| Ordered numbered plans in this packet | 5 | `064-01` through `064-05` |
| Named Markdown sources read from `.planning/phases/064-Gaps-Closing-3/064-TODO.md` | 17 | Mandatory planning input |

## 🔐 Strict TODO Row-Class Lock

The packet locks the full TODO action and directive surface, not only the 28
recommendation rows. These preserved classes sum to 57 TODO rows or
directives, including repeated priority restatements that must remain explicit
rather than being silently collapsed away.

| TODO class | Count | Packet preservation |
| --- | ---: | --- |
| Main recommendation | 1 | `D-04`, `W1`, and `.planning/phases/064-Gaps-Closing-3/064-01-PLAN.md` |
| Iteration 1 surface-fact rows | 4 | `D-04`, `D-05`, `D-06`, `D-09`, canonical refs, and first-wave owner boundaries |
| Iteration 2 finding rows | 3 | `REC-064-P0-01`, `REC-064-P0-03`, and `W1` simulator truth restoration |
| Iteration 3 finding rows | 4 | `REC-064-P1-01`, `REC-064-P1-02`, `REC-064-P1-03`, and `.planning/phases/064-Gaps-Closing-3/064-02-PLAN.md` |
| Iteration 4 pattern rows | 4 | `REC-064-P0-02`, `REC-064-P0-03`, `REC-064-P2-02`, and hermeticity guardrails across `W1` and `W3` |
| First priority table rows | 6 | `REC-064-P0-01`, `REC-064-P0-02`, `REC-064-P1-01`, `REC-064-P1-03`, `REC-064-P0-03`, `REC-064-P2-04` |
| Corrected priority table rows | 10 | `REC-064-P0-02`, `REC-064-P0-01`, `REC-064-P0-03`, `REC-064-P1-01`, `REC-064-P1-02`, `REC-064-P1-03`, `REC-064-P2-01`, `REC-064-P2-02`, `REC-064-P2-03`, `REC-064-P2-04` |
| Ordered closeout sequence bullets | 5 | explicit top-level TODO step contract below plus numbered-plan dependencies |
| Numbered additional rows `1-5` | 5 | `REC-064-P0-04` through `REC-064-P0-08` |
| Numbered additional rows `6-13` | 8 | `REC-064-P1-04` through `REC-064-P1-11` |
| Numbered additional rows `14-18` | 5 | `REC-064-P2-05` through `REC-064-P2-09`, with the simulator-facade guard recorded in the ordered closeout contract |
| Closeout-order directive | 1 | `D-13` plus the ordered closeout contract below |
| Local-evidence discipline note | 1 | `D-12` and workspace-first review evidence only |

## 📍 Top-Level TODO Step Contract

| Top-level TODO step | Locked execution owner | Exact preservation contract |
| --- | --- | --- |
| `Top-level TODO step 1: simulator` | `.planning/phases/064-Gaps-Closing-3/064-01-PLAN.md` | Simulator final truth, hermetic canonical stages, exact-home packet artifacts, default packet secrecy, and simulator-facade discipline must close first on the canonical `scenario_1` owner surface. |
| `Top-level TODO step 2: wallet` | `.planning/phases/064-Gaps-Closing-3/064-02-PLAN.md` | Wallet-local mutation truth must land on `LocalNodeSim`, `ChainClientImpl`, `BroadcastImpl`, and durable tx storage instead of stub or fake lanes. |
| `Top-level TODO step 3: rpc` | `.planning/phases/064-Gaps-Closing-3/064-02-PLAN.md` | RPC truth remains attached to the same second-wave wallet owner set and must prove include-based registration coverage plus `app.wallet.open_wallet_source` wiring. |
| `Top-level TODO step 4: wallet services` | `.planning/phases/064-Gaps-Closing-3/064-03-PLAN.md` | Only live-backed wallet-service seams may be promoted; placeholder-only service names stay explicitly non-live unless the same owner slice closes them truthfully. |
| `Top-level TODO step 5: runtime/rollup` | `.planning/phases/064-Gaps-Closing-3/064-04-PLAN.md` | Storage, theorem, recovery, local DA, and publication-binding proof surfaces must stay local, deterministic, fail-closed, and singular. |

## 🔢 Ordered Closeout Contract

| Global TODO group | Locked coverage ids | Ordered execution contract |
| --- | --- | --- |
| `Numbered closeout group 1-5` | `REC-064-P0-04`, `REC-064-P0-05`, `REC-064-P0-06`, `REC-064-P0-07`, `REC-064-P0-08` | Must be closed before numbered group `6-13` can be claimed complete. Item `5` stays attached to `W1` because default release-packet secrecy is inseparable from canonical simulator truth. |
| `Numbered closeout group 6-13` | `REC-064-P1-04` through `REC-064-P1-11` | Lives across `W2` and `W3` after simulator, wallet-mutation, and RPC truth are restored. No item may bypass storage/runtime owners. |
| `Numbered closeout group 14-18` | `REC-064-P2-05`, `REC-064-P2-06`, `REC-064-P2-07`, `REC-064-P2-08`, `REC-064-P2-09` | Boundary-CI guardrails close last, except item `16` (`REC-064-P2-07`) which stays on `W1` because simulator facade discipline is an owner-boundary guard on the same first-wave surface. |

## ✅ Recommendation Transfer Table

| Coverage id | `.planning/phases/064-Gaps-Closing-3/064-TODO.md` source slice | Locked execution contract | Primary refs | Target plan |
| --- | --- | --- | --- | --- |
| `REC-064-P0-01` | Priority row `CRITICAL` | Promote default `scenario_1` publication from `draft_only` to a truthful local-final path. | `crates/z00z_simulator/README.md`; `crates/z00z_simulator/src/config.rs`; `crates/z00z_simulator/src/scenario_1/stage_12/mod.rs`; `crates/z00z_simulator/src/scenario_1/stage_12/finalize_flow.rs` | `.planning/phases/064-Gaps-Closing-3/064-01-PLAN.md` |
| `REC-064-P0-02` | Priority row `P0` | Remove `step_stub` fallback closure from canonical stages 9-12. | `crates/z00z_simulator/src/scenario_1/stage_9/bundle_lane_impl.rs`; `crates/z00z_simulator/src/scenario_1/stage_12/mod.rs`; `crates/z00z_simulator/src/scenario_1/support/fixture_cache.rs`; `crates/z00z_simulator/src/scenario_1/support/stage_runner_support.rs` | `.planning/phases/064-Gaps-Closing-3/064-01-PLAN.md` |
| `REC-064-P0-03` | Priority row `P0` | Promote `asset_flow.json`, `voucher_flow.json`, and `right_flow.json` from `pending_exact_home` to emitted packet artifacts. | `crates/z00z_simulator/README.md`; `wiki/06-simulator-and-quality/scenario1-object-artifacts.md`; `crates/z00z_simulator/src/scenario_1/runtime_observability.rs` | `.planning/phases/064-Gaps-Closing-3/064-01-PLAN.md` |
| `REC-064-P0-08` | Numbered `P0` item 5 | Reject plaintext secret leakage from the default release packet. | `wiki/06-simulator-and-quality/scenario-pipeline.md`; `crates/z00z_simulator/tests/scenario_1/test_stage2_secret_artifacts.rs` | `.planning/phases/064-Gaps-Closing-3/064-01-PLAN.md` |
| `REC-064-P2-07` | Numbered `P2` item 2 | Require simulator harness imports to stay on owner facades rather than deep internals. | `wiki/06-simulator-and-quality/scenario-pipeline.md`; `crates/z00z_simulator/src/lib.rs` | `.planning/phases/064-Gaps-Closing-3/064-01-PLAN.md` |
| `REC-064-P1-01` | Priority row `P1` | Replace `stub_default()` and `stub_tx_*` asset mutations with the live wallet-local chain/broadcast/store path. | `crates/z00z_wallets/src/chain/local_node_sim.rs`; `crates/z00z_wallets/src/chain/broadcast_impl.rs`; `crates/z00z_wallets/src/rpc/asset_rpc_server_ops.rs`; `crates/z00z_wallets/src/rpc/asset_rpc_server_catalog.rs` | `.planning/phases/064-Gaps-Closing-3/064-02-PLAN.md` |
| `REC-064-P1-02` | Priority row `P1` | Preserve `wallet.object.*` as the live post-genesis typed-object path and add guardrails against stale “stub” claims. | `wiki/04-wallet-and-rpc/wallet-object-packages.md`; `crates/z00z_wallets/src/rpc/wallet_dispatcher_routes.rs`; `crates/z00z_wallets/src/rpc/object_rpc_impl.rs` | `.planning/phases/064-Gaps-Closing-3/064-02-PLAN.md` |
| `REC-064-P1-03` | Priority row `P1` | Repair the RPC audit tool so it sees include-based route registration and close the `app.wallet.open_wallet_source` registration gap. | `crates/z00z_wallets/scripts/audit_rpc_method_wiring.py`; `crates/z00z_wallets/src/rpc/app_rpc_impl.rs`; `crates/z00z_wallets/src/rpc/app_dispatcher_wiring.rs` | `.planning/phases/064-Gaps-Closing-3/064-02-PLAN.md` |
| `REC-064-P2-01` | Priority row `P2` | Collapse placeholder service naming to honest live owners or explicit non-live seams. | `wiki/04-wallet-and-rpc/wallet-stub-surface.md`; `crates/z00z_wallets/src/services/` | `.planning/phases/064-Gaps-Closing-3/064-03-PLAN.md` |
| `REC-064-P0-04` | Numbered `P0` item 1 | Add wallet restore fault-injection coverage around staged `.wlt` and history commits. | `crates/z00z_wallets/src/services/wallet_actions_backup.rs` | `.planning/phases/064-Gaps-Closing-3/064-03-PLAN.md` |
| `REC-064-P0-05` | Numbered `P0` item 2 | Prove every sensitive RPC path goes through session verification gates. | `crates/z00z_wallets/src/services/wallet_session_manager.rs`; `crates/z00z_wallets/src/services/wallet_session_runtime_limits.rs` | `.planning/phases/064-Gaps-Closing-3/064-03-PLAN.md` |
| `REC-064-P0-06` | Numbered `P0` item 3 | Ban raw stealth-output builders from production app/RPC flows except explicit test/scanner allowlists. | `wiki/04-wallet-and-rpc/receiver-request-flow.md`; `crates/z00z_wallets/src/receiver/request.rs`; `crates/z00z_wallets/src/stealth/output.rs` | `.planning/phases/064-Gaps-Closing-3/064-03-PLAN.md` |
| `REC-064-P0-07` | Numbered `P0` item 4 | Add compile/doc/test guards so browser surfaces never advertise native wallet guarantees. | `wiki/04-wallet-and-rpc/receiver-request-flow.md`; `crates/z00z_wallets/src/services/wallet_store_restore.rs`; `crates/z00z_wallets/src/services/wallet_session_runtime_limits.rs` | `.planning/phases/064-Gaps-Closing-3/064-03-PLAN.md` |
| `REC-064-P1-04` | Numbered `P1` item 6 | Preserve quarantined rights/vouchers across restore/export/import and add explicit promotion/no-promotion contracts. | `wiki/04-wallet-and-rpc/wallet-object-quarantine.md`; `crates/z00z_wallets/src/redb_store/owned_objects.rs`; `crates/z00z_wallets/src/redb_store/object_queries.rs` | `.planning/phases/064-Gaps-Closing-3/064-03-PLAN.md` |
| `REC-064-P1-05` | Numbered `P1` item 1 | Ensure every `ObjectRejectCode` has stable RPC mapping, validator class, and coverage proof. | `crates/z00z_storage/src/settlement/object_package_contract.rs`; `crates/z00z_wallets/src/rpc/object_rpc_impl.rs` | `.planning/phases/064-Gaps-Closing-3/064-03-PLAN.md` |
| `REC-064-P2-02` | Priority row `P2` | Expand local DA and runtime negative coverage without claiming a real DA network. | `crates/z00z_runtime/aggregators/README.md`; `crates/z00z_runtime/aggregators/src/batch_planner.rs`; `crates/z00z_rollup_node/src/da.rs` | `.planning/phases/064-Gaps-Closing-3/064-04-PLAN.md` |
| `REC-064-P1-06` | Numbered `P1` item 2 | Keep `save_artifact()` off the canonical seal path except explicit allowlists. | `crates/z00z_storage/src/checkpoint/store.rs`; `crates/z00z_simulator/src/scenario_1/stage_4/storage_view.rs` | `.planning/phases/064-Gaps-Closing-3/064-04-PLAN.md` |
| `REC-064-P1-07` | Numbered `P1` item 3 | Cover every `PrepSnapshot` adversarial lane with deterministic negative tests. | `wiki/05-storage-runtime/prep-snapshot-replay.md`; `crates/z00z_storage/src/snapshot/store.rs` | `.planning/phases/064-Gaps-Closing-3/064-04-PLAN.md` |
| `REC-064-P1-08` | Numbered `P1` item 4 | Prevent downstream conflation between `backend_root`, `SettlementStateRoot`, flat terminal ids, and raw proof types. | `crates/z00z_storage/src/settlement/README.md`; `crates/z00z_storage/src/settlement/proof.rs` | `.planning/phases/064-Gaps-Closing-3/064-04-PLAN.md` |
| `REC-064-P1-09` | Numbered `P1` item 5 | Add theorem-boundary negative tests for detached statements, wrong proof payloads, wrong ids, and wrong link roots. | `crates/z00z_rollup_node/src/lib.rs` | `.planning/phases/064-Gaps-Closing-3/064-04-PLAN.md` |
| `REC-064-P1-10` | Numbered `P1` item 6 | Close every explicit recovery failover rejection branch with tests. | `crates/z00z_runtime/aggregators/src/recovery.rs` | `.planning/phases/064-Gaps-Closing-3/064-04-PLAN.md` |
| `REC-064-P1-11` | Numbered `P1` item 7 | Extend `PublicationBinding` anti-fork guardrails across simulator, rollup, and doc examples. | `crates/z00z_runtime/aggregators/README.md`; `crates/z00z_runtime/aggregators/src/types.rs` | `.planning/phases/064-Gaps-Closing-3/064-04-PLAN.md` |
| `REC-064-P2-03` | Priority row `P2` | Keep core/genesis cleanup truthful and sequenced after simulator/wallet truth restoration. | `wiki/03-core-protocol/genesis-caveats.md`; `crates/z00z_core/src/genesis/genesis_run.rs`; `crates/z00z_core/src/vouchers/voucher_bootstrap.rs` | `.planning/phases/064-Gaps-Closing-3/064-05-PLAN.md` |
| `REC-064-P2-04` | `DEFER` row | Keep OnionNet, real remote chain, real DA, slashing, and fraud-engine claims honestly deferred with guardrails against accidental live wording. | `crates/z00z_networks/onionnet/README.md`; `docs/Z00Z-Main-Whitepaper.md`; `docs/tech-papers/Z00Z-Roadmap-Blueprint.md` | `.planning/phases/064-Gaps-Closing-3/064-05-PLAN.md` |
| `REC-064-P2-05` | Numbered `P2` item 14 | Add boundary CI so business crates cannot quietly widen direct `std::fs`, `serde_*`, clock, or RNG usage outside allowed owners. | `crates/z00z_utils/README.md` | `.planning/phases/064-Gaps-Closing-3/064-05-PLAN.md` |
| `REC-064-P2-06` | Numbered `P2` item 1 | Enforce `z00z_crypto` as the only workspace crypto facade. | `crates/z00z_crypto/README.md`; `crates/z00z_crypto/src/lib.rs` | `.planning/phases/064-Gaps-Closing-3/064-05-PLAN.md` |
| `REC-064-P2-08` | Numbered `P2` item 3 | Add a guard that keeps `z00z_extensions` from becoming a semantic dumping ground. | `crates/z00z_extensions/README.md` | `.planning/phases/064-Gaps-Closing-3/064-05-PLAN.md` |
| `REC-064-P2-09` | Numbered `P2` item 4 | Replace internal GitHub links in wiki pages with local-path references and lock the rule in an offline-safe check. | `wiki/06-simulator-and-quality/scenario-pipeline.md` | `.planning/phases/064-Gaps-Closing-3/064-05-PLAN.md` |

## 🔎 Meta Guidance Transfer

| `.planning/phases/064-Gaps-Closing-3/064-TODO.md` section | Planning transfer |
| --- | --- |
| Five-iteration simulator/wallet/RPC diagnosis | Preserved by the explicit top-level TODO step contract above and the dependency chain `.planning/phases/064-Gaps-Closing-3/064-01-PLAN.md` -> `.planning/phases/064-Gaps-Closing-3/064-02-PLAN.md` -> `.planning/phases/064-Gaps-Closing-3/064-03-PLAN.md` -> `.planning/phases/064-Gaps-Closing-3/064-04-PLAN.md` -> `.planning/phases/064-Gaps-Closing-3/064-05-PLAN.md`. |
| “Do not go to network/onion/remote-chain now” | Preserved as `REC-064-P2-04` and repeated in `.planning/phases/064-Gaps-Closing-3/064-05-PLAN.md` guardrails. |
| “Only new points below; do not repeat simulator/wallet-asset/RPC/genesis/local-DA themes” | Preserved by keeping the 18 numbered sub-items separate from the top-level priority map and mapping both sets explicitly. |
| Local-only closure requirement | Preserved in every plan `simulation_gate`, `anti_placeholder_gate`, and `<verify>` contract. |

## 🪜 Execution Waves

| Wave | Plans | Scope |
| --- | --- | --- |
| `W1` | `.planning/phases/064-Gaps-Closing-3/064-01-PLAN.md` | Simulator final-truth, packet integrity, hermetic stage coverage, emitted artifact homes |
| `W2` | `.planning/phases/064-Gaps-Closing-3/064-02-PLAN.md`, `.planning/phases/064-Gaps-Closing-3/064-03-PLAN.md` | Wallet mutation truth, RPC truth, wallet-sensitive surfaces, and typed-object durability |
| `W3` | `.planning/phases/064-Gaps-Closing-3/064-04-PLAN.md` | Storage checkpoint/snapshot boundaries and runtime/rollup adversarial coverage |
| `W4` | `.planning/phases/064-Gaps-Closing-3/064-05-PLAN.md` | Core/defer guardrails and repository-wide boundary hygiene |

## 🚫 No-Drift Guardrails

- No invented `TASK-NNN` rows.
- No second simulator truth lane outside `scenario_1`.
- No second packet-secret policy outside the canonical default release packet path.
- No second wallet mutation authority outside the wallet-local chain/broadcast/store seam.
- No second publication binding or route-table acceptance authority outside runtime-owned types.
- No live wording for OnionNet, real remote chain, or real DA until code and local proof say so.
- No GitHub-hosted internal source refs in active wiki pages when the same repo path can be cited locally.
- No graphify-derived structural hint may be cited as factual source truth, coverage proof, or acceptance evidence.

</normative_mirror>
