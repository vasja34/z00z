<!-- markdownlint-disable MD001 MD022 MD032 MD033 MD047 -->
# Phase 062: Gaps-Closing-2 - Context

**Gathered:** 2026-06-24
**Status:** Execution complete; summaries now exist for `PLAN-062-G01`
through `PLAN-062-G27`, no active Phase 062 execution lane remains, and the
phase is in validation closeout on the current tree as tracked in
`.planning/STATE.md` and `.planning/ROADMAP.md`
**Source:** PRD Express Path (`.planning/phases/062-Gaps-Closing-2/062-TODO.md` plus the referenced phase corpus and live code anchors)

<domain>
## 🎯 Phase Boundary

Phase 062 turns `.planning/phases/062-Gaps-Closing-2/062-TODO.md` into a numbered execution packet that closes every local correctness gap it names. `062-TODO.md` is normative, not advisory. The phase keeps the existing phase folder, preserves `TASK-001` through `TASK-125` exactly, and creates grouped execution plans `PLAN-062-G01` through `PLAN-062-G27` without renumbering, merging away, or silently reinterpreting any task.

This phase is a full closure packet for storage root authority, wallet lifecycle and import taxonomy, privacy and field-native claim normalization, object and right flows, genesis manifest cleanup, local and distributed HJMT, thin signed-index mode, and the mandatory local full-system blockers `TASK-121` through `TASK-125`.

### What Phase 062 Delivers

1. A complete coverage-checked grouped plan set for all 125 canonical tasks.
2. One task-to-plan mapping where every `TASK-NNN` belongs to exactly one grouped `PLAN-062-GNN`.
3. A plan-level and task-level execution contract naming concrete code, docs, config, tests, simulator artifacts, and evidence gates.
4. A local-simulation closure boundary for distributed HJMT, thin-mode helper semantics, node RPC, broadcast, fee-rate sourcing, remote scan worker, and spend-policy enforcement.

### What Phase 062 Does NOT Deliver

- No new phase directory.
- No placeholder, scaffold-only, TODO-only, panic-only, string-only, or no-op closure paths.
- No compile-only proof for runtime behavior.
- No docs-only proof for code behavior.
- No live claim for external DA transport, remote process deployment, or remote network transport where only a local deterministic simulator exists.
- No edits under `crates/z00z_crypto/tari/**`.

</domain>

<decisions>
## ⚙️ Implementation Decisions

- **D-01:** `.planning/phases/062-Gaps-Closing-2/062-TODO.md` is the canonical Phase 062 authority.
- **D-02:** Coverage is phase-failing unless all 125 unique task ids and all 27 grouped plan ids are present and mapped exactly once.
- **D-03:** Every grouped plan carries both a plan-level matrix and a task-level matrix. Every included task row declares artifacts, tests, expected results, and one `implementation_depth` from `full`, `simulated-full`, or `live-claim-removed`.
- **D-04:** Only external transport, remote process boundaries, unavailable third-party networks, and wall-clock or fault scheduling may be simulated. Real cryptography, package verification, planner output, route tables, HJMT journals, storage commit or recovery, wallet history, fee policy, publication bindings, validator or watcher checks, and component-local state must stay real.
- **D-05:** `TASK-121` through `TASK-125` are outside the strict source corpus, but mandatory for local full-system closure. They cannot be deferred.
- **D-06:** Wallet receive authority remains the live `recv_range` lane plus atomic wallet persistence. Inbox helpers, remote scan workers, and local node simulations stay subordinate to that lane.
- **D-07:** Local HJMT proof, route, journal, checkpoint, and publication evidence must be closed before distributed claims expand.
- **D-08:** `config/hjmt_runtime` stays runtime or orchestration fixture ownership unless only backend schema/default fragments move.
- **D-09:** `GenesisConfig` stays the single bootstrap authority even if Phase 062 adds a production root manifest plus referenced subfiles.
- **D-10:** Every `<verify>` block starts with bootstrap fail-fast, runs slice-specific commands, runs `cargo test --release` when Rust/tests are affected, runs `/GSD-Review-Tasks-Execution` in YOLO mode at least three times until two consecutive runs are clean, and uses `/z00z-git-versioning` for commits.

### the agent's Discretion

- Exact helper-module placement within the owning crate, provided authority boundaries stay intact.
- Whether a field-native or thin-mode sub-claim closes by full implementation or explicit live-claim removal when the task text itself allows it.
- Extra helper tests or docs that improve local proof without deferring local correctness.

</decisions>

<canonical_refs>
## 📚 Canonical References

### Planning Authority

- `.planning/phases/062-Gaps-Closing-2/062-TODO.md` — canonical task, grouping, gate, and evidence authority
- `.planning/phases/062-Gaps-Closing-2/GAPS.md` — tasks `001` through `075`, verification commands, closure matrix, doublecheck map, blocker and completion templates
- `.planning/ROADMAP.md` — registered Phase 062 scope
- `.planning/STATE.md` — active phase marker
- `.github/copilot-instructions.md` — repository execution rules

### Genesis And Asset/Core Ownership

- `.planning/phases/062-Gaps-Closing-2/asset-only.md`
- `crates/z00z_core/src/genesis/README.md`
- `crates/z00z_core/src/assets/object_family.rs`
- `crates/z00z_core/src/assets/asset_error.rs`

### Wallet Closure

- `crates/z00z_wallets/src/rpc/tx_types.rs`
- `crates/z00z_wallets/src/rpc/chain_types.rs`
- `crates/z00z_wallets/src/persistence/tx_storage.rs`
- `crates/z00z_wallets/src/services/wallet_actions_receive.rs`
- `crates/z00z_wallets/src/chain/chain_client_impl.rs`
- `crates/z00z_wallets/src/chain/broadcast_impl.rs`
- `crates/z00z_wallets/src/tx/fee_estimator.rs`
- `crates/z00z_wallets/src/chain/scan_engine_impl.rs`
- `crates/z00z_wallets/src/wallet/policy.rs`

### HJMT Local And Distributed Closure

- `.planning/phases/062-Gaps-Closing-2/HJMT-REPORT.md`
- `.planning/phases/062-Gaps-Closing-2/HJMT-Sharding-Storage-Techpaper.md`
- `.planning/phases/062-Gaps-Closing-2/HJMT-RAID -Sharding.md`
- `.planning/phases/062-Gaps-Closing-2/HJMT-структуры.md`
- `crates/z00z_runtime/aggregators/src/batch_planner.rs`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs`
- `config/hjmt_runtime/sim_5a7s`

### Thin Mode

- `.planning/phases/062-Gaps-Closing-2/Z00Z-Thin-Transaction-Mode.md`
- `crates/z00z_wallets/src/rpc/tx_types.rs`
- `crates/z00z_wallets/src/rpc/tx_rpc_impl.rs`
- `crates/z00z_storage/src/settlement/root_types.md`

</canonical_refs>

<normative_mirror>
## Normative Mirror

This section mirrors the non-inventory normative parts of `062-TODO.md` so the
Phase 062 review packet can be audited from `062-CONTEXT.md` without treating
context as a second authority. `062-TODO.md` remains canonical.

## Verdict

The table below is mirrored from `062-TODO.md` to preserve audit-fidelity for
plan review.

| Rule | Decision |
| --- | --- |
| Plan granularity | One GSD plan per coherent implementation group; every plan contains explicit task ids. |
| Required grouped plan ids | `PLAN-062-G01` through `PLAN-062-G27`. |
| Atomic fallback ids | `PLAN-062-T001` through `PLAN-062-T125`, only if a group MUST be split. |
| Task ids | `TASK-001` through `TASK-125`, with no alternate task namespaces. |
| Strict phase-corpus count | `TASK-001` through `TASK-120` = 120 MD-backed tasks, but yolo/full-system execution MUST also include `TASK-121` through `TASK-125`. |
| Full-system count | `TASK-001` through `TASK-125` = 120 strict corpus tasks plus 5 audit-derived live-system simulation blockers, required as 27 grouped GSD plans. |
| Grouped rows | The `Required GSD Plan Groups` table is the plan source of truth. Waves are scheduling hints only. |
| Requirement status | Every task row is normative. There are no nice-to-have recommendation rows. |
| Required implementation reading | Every task row has exact Markdown source links to read before code changes. |

## Normative Language

The keywords `MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, and `MAY` are
normative for Phase 062 plan generation and implementation.

| Keyword | Meaning in Phase 062 |
| --- | --- |
| `MUST` | Required for plan generation, implementation, or phase closure. Failure to satisfy a `MUST` item blocks the plan/task. |
| `MUST NOT` | Forbidden. A generated plan/task that violates a `MUST NOT` item is invalid. |
| `SHOULD` | Expected default. A generated plan/task MAY deviate only with explicit evidence that the stronger requirement is inapplicable and with a replacement gate. |
| `SHOULD NOT` | Expected prohibition. A generated plan/task MAY deviate only with explicit evidence and a replacement gate. |
| `MAY` | Allowed only where a `MUST` gate already defines the required closure path. `MAY` never makes a task optional. |

## Source Corpus

| Source | Lines | Role |
| --- | ---: | --- |
| [`GAPS.md`](./GAPS.md) | 2549 | Phase 062 ordered backlog, DoD, validation commands, source closure matrix, doublecheck map |
| [`asset-only.md`](./asset-only.md) | 85 | asset/core ownership cleanup and production manifest intent |
| [`HJMT-REPORT.md`](./HJMT-REPORT.md) | 949 | implemented HJMT evidence and remaining distributed/runtime gaps |
| [`HJMT-RAID -Sharding.md`](./HJMT-RAID%20-Sharding.md) | 2416 | RAID-like sharding/failover design notes and implementation pressure |
| [`HJMT-Sharding-Storage-Techpaper.md`](./HJMT-Sharding-Storage-Techpaper.md) | 895 | HJMT sharding/storage architecture and rollout constraints |
| [`HJMT-структуры.md`](./HJMT-структуры.md) | 32 | HJMT structure creation ownership note |
| [`Z00Z-Thin-Transaction-Mode.md`](./Z00Z-Thin-Transaction-Mode.md) | 1006 | thin signed-index transaction mode design |

## Count Answer

For GSD plan generation from this audit:

| Mode | Required grouped plans | Atomic tasks | Meaning |
| --- | ---: | ---: | --- |
| Strict Phase 062 corpus | 22 | 120 | MD-backed minimum only; not sufficient for yolo/full-system closure. |
| Full-system yolo mode | 27 | 125 | Mandatory mode for this run: strict corpus plus 5 wallet/node full-system simulation blockers found in code and registered below. |

Generate **27 grouped plans** in yolo/full-system mode: `PLAN-062-G01`
through `PLAN-062-G27`. Do not stop at `TASK-120`; `TASK-121` through
`TASK-125` are mandatory local full-system simulation closures. The 125 task
rows remain the mandatory traceability inventory and can be split into atomic
fallback plans only when a group becomes too large or risky.

## Required GSD Plan Groups

To avoid a third grouped-plan authority while still reflecting the TODO
contract:

- `062-TODO.md` remains the canonical 27-row grouped-plan table.
- `062-COVERAGE.md` mirrors each grouped row with scope, dependencies, and
  acceptance.
- `062-01-PLAN.md` through `062-27-PLAN.md` materialize each
  grouped row as an executable plan.
- `062-TEST-SPEC.md` and `062-TESTS-TASKS.md` preserve the same `G01` through
  `G27` execution order for verification.

## Pre-Plan Blockers

| Blocker | Required action |
| --- | --- |
| Path drift from the Phase 061 wallet refactor | Apply the rewrite map before assigning files to a plan; old `adapters/rpc`, `db/redb_wallet_store`, and `services/wallet/actions` paths are stale. |
| Old backend env wording | Treat `Z00Z_SETTLEMENT_BACKEND_MODE`, not `Z00Z_STORAGE_BACKEND`, as the current storage backend env. |
| Old TODO pointer wording | Current file is `.planning/phases/062-Gaps-Closing-2/GAPS.md`; any `.planning/phases/TODO-gaps.md` reference is legacy unless intentionally retained as a historical pointer. |
| No postpone escape | Any blocker for local correctness, cryptography, consensus, conflict resolution, wallet/node flow, DA/publication semantics, scan, fee, or policy MUST be closed in Phase 062 by code and tests, or converted into a local simulator scenario using real project primitives. Only external deployment adapters remain non-live, and they MUST be excluded from live claims. |
| Current-vs-simulated-live ambiguity | Thin signed-index and distributed HJMT replication/quorum/catch-up/rollout/dispatch MUST be implemented even without a real network; implement the behavior in local deterministic simulators using real runtime/storage/crypto primitives, and leave only real transport/chain-network adapters outside live claims. |
| Full-system simulation mandate | Fake only external transport, remote node process boundaries, external DA transport, wall-clock/fault scheduling, and unavailable third-party networks. Use real cryptography, route tables, planner output, HJMT journal entries, storage commit/recovery paths, package verification, wallet history, fee policy, publication bindings, validator/watcher checks, and per-component state. |
| Local distributed simulation mandate | For distributed HJMT, fake only the transport and clock/fault scheduler. Use real route tables, planner output, HJMT journal entries, storage commit/recovery paths, publication bindings, validator/watcher checks, and per-aggregator state. Required scenarios include message delay/drop/reorder, partition/heal, divergent roots, stale lineage, standby unavailable/catch-up, route rollout ack, remote dispatch, and restart after partial journal stages. |
| HJMT runtime fixture ownership | Treat `config/hjmt_runtime` as a repo-level runtime home consumed by `z00z_rollup_node`/simulator tests, not as a storage-owned semantic layer. If the fixture is moved, move it under runtime/orchestration ownership and keep only storage backend schema/default fragments in storage. |
| Dirty/refactor worktree | Generate plans from current workspace paths, not historical paths. |

## Requirement Gate Contract

| Gate | Required input | Required output | Pass condition |
| --- | --- | --- | --- |
| Input gate | `Source refs to read first`, task dependencies, group dependencies, and relevant current-code anchors. | A plan-local `inputs` section with exact files/anchors and code surfaces to inspect before editing. | A task MUST NOT start without source refs, dependency order, and current-code surface. |
| Output gate | Task `Scope` plus group `Scope`. | A plan-local `outputs` section listing concrete code/docs/config/test artifacts to create or modify. | Each row MUST produce an implementation artifact, local simulator artifact, test artifact, or explicit live-claim removal. |
| Acceptance gate | Task `Acceptance / tests`, group `Acceptance / tests`, and `TASK-072` commands. | A plan-local `acceptance` section with commands, negative cases, and evidence rows. | Acceptance MUST be executable or evidence-backed; prose-only completion is invalid. |
| Simulation gate | Any missing network/node/DA/process behavior. | A local deterministic simulator scenario using real project primitives and fake transport/clock only. | External deployment gaps MUST NOT block local correctness coverage. |
| Artifact gate | Group scope, task scope, and current-code anchors. | `plan_artifacts` and `task_artifacts` matrices listing exact files/APIs/configs/docs/simulator outputs to create or modify. | Every plan and every included task MUST name concrete artifacts before implementation. |
| Test gate | Acceptance rows, negative cases, and simulator requirements. | `plan_tests` and `task_tests` matrices listing exact commands, test modules, scenarios, and expected failures/successes. | Every plan and every included task MUST name tests that prove artifact behavior. |
| Result gate | Completion evidence, simulator outputs, logs, digests, and command results. | `plan_results` and `task_results` matrices with pass/fail evidence, output artifacts, and proof that behavior is not a placeholder. | Every plan and every included task MUST provide result evidence before closure. |
| No scaffold gate | Any new artifact created by the plan. | Evidence that public APIs, simulators, tests, and docs exercise real project primitives and do not stop at stubs. | Placeholder, scaffold-only, TODO-only, panic-only, string-only, or no-op implementations MUST fail the task. |
| Evidence gate | `TASK-075` blocker/completion format and final closure tasks. | Completion evidence linking source refs, changed files, tests, simulator outputs, and unresolved adapter-only exclusions. | A task MUST NOT close without traceable evidence or a phase-failing blocker. |
| Nice-to-have rejection gate | Any wording from source docs that sounds advisory. | Reclassified requirement, live-claim removal, or phase-failing blocker. | Generated plans MUST NOT label a task as optional, advisory, or best-effort. |

## Artifact/Test/Result Proof Contract

| Matrix | Required columns | Closure rule |
| --- | --- | --- |
| `plan_artifacts` | `artifact_id`, `path_or_api`, `kind`, `owner_crate`, `created_or_modified`, `source_task_ids` | The plan MUST list every code/docs/config/test/simulator artifact needed for the group. |
| `plan_tests` | `test_id`, `command`, `test_module_or_scenario`, `positive_or_negative`, `proves` | The plan MUST list executable tests or simulator scenarios proving the group behavior. |
| `plan_results` | `result_id`, `command_or_artifact`, `expected_result`, `evidence_path`, `closure_status` | The plan MUST list the evidence that demonstrates correctness and non-placeholder implementation. |
| `task_artifacts` | `task_id`, `artifact_id`, `path_or_api`, `kind`, `implementation_depth` | Every task MUST list its own concrete artifacts; `implementation_depth` MUST be `full`, `simulated-full`, or `live-claim-removed`. |
| `task_tests` | `task_id`, `test_id`, `command_or_scenario`, `positive_or_negative`, `failure_modes` | Every task MUST list tests for success and required negative/failure cases. |
| `task_results` | `task_id`, `result_id`, `proof_artifact`, `pass_condition`, `anti_placeholder_evidence` | Every task MUST list result proof that its implementation is exercised through real project primitives. |

## Current Wallet Path Rewrite Map

### TODO Literal Mirror

Historical intermediate aliases from the Phase 061 refactor were collapsed on
2026-06-25. This table is now the single canonical rewrite map mirrored from
`062-TODO.md`.

| Old path pattern in source docs | Current path |
| --- | --- |
| `crates/z00z_wallets/src/adapters/rpc/types/tx.rs` | `crates/z00z_wallets/src/rpc/tx_types.rs` |
| `crates/z00z_wallets/src/adapters/rpc/types/chain.rs` | `crates/z00z_wallets/src/rpc/chain_types.rs` |
| `crates/z00z_wallets/src/adapters/rpc/error_mapping.rs` | `crates/z00z_wallets/src/rpc/error_mapping.rs` |
| `crates/z00z_wallets/src/adapters/rpc/methods/chain_impl.rs` | `crates/z00z_wallets/src/rpc/chain_rpc_impl.rs` |
| `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs` | `crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs` |
| `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_helpers.rs` | `crates/z00z_wallets/src/rpc/tx_rpc_server_helpers.rs` |
| `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs` | `crates/z00z_wallets/src/rpc/tx_rpc_server_lifecycle.rs` |
| `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_send.rs` | `crates/z00z_wallets/src/rpc/tx_rpc_server_send.rs` |
| `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_storage.rs` | `crates/z00z_wallets/src/persistence/tx_storage.rs` plus `crates/z00z_wallets/src/rpc/tx_rpc_server_history.rs` |
| `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_impl.rs` | `crates/z00z_wallets/src/rpc/tx_rpc_impl.rs`, `crates/z00z_wallets/src/rpc/tx_rpc_support.rs`, and `crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs` |
| `crates/z00z_wallets/src/adapters/rpc/methods/object.rs` | `crates/z00z_wallets/src/rpc/object_rpc.rs` |
| `crates/z00z_wallets/src/adapters/rpc/methods/object_impl.rs` | `crates/z00z_wallets/src/rpc/object_rpc_impl.rs` |
| `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs` | `crates/z00z_wallets/src/redb_store/owned_assets.rs` |
| `crates/z00z_wallets/src/db/redb_wallet_store/owned_objects.rs` | `crates/z00z_wallets/src/redb_store/owned_objects.rs` |
| `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs` | `crates/z00z_wallets/src/services/wallet_actions_receive.rs` |
| `crates/z00z_wallets/src/receiver/request/mod.rs` | `crates/z00z_wallets/src/receiver/request.rs` plus `crates/z00z_wallets/src/receiver/payment_request_*` helpers |

## Plan Waves

| Wave | Task range | Scope |
| --- | --- | --- |
| W0 | `TASK-071`-`TASK-075` | Plan-generation contract, DoD, validation commands, closure map, and blocker templates. |
| W1 | `TASK-001`-`TASK-013`, `TASK-085`-`TASK-089`, `TASK-103`, `TASK-104`, `TASK-120` | Storage root, checkpoint, local HJMT proof/root/journal/benchmark/boundary closeout. |
| W2 | `TASK-014`-`TASK-039` | Wallet lifecycle, tx history, import/verify, receive scan, request inbox, simulator evidence. |
| W3 | `TASK-040`-`TASK-048` | Field-native package decision, privacy, reveal, and logging hygiene. |
| W4 | `TASK-049`-`TASK-062` | Object, voucher, right, fee, adapter, agentic, and machine-capability closure. |
| W5 | `TASK-076`-`TASK-084` | Asset/core ownership and genesis manifest cleanup. |
| W6 | `TASK-090`-`TASK-102`, `TASK-105` | Local distributed HJMT simulation for replication/quorum/catch-up/route rollout/scheduler/remote-dispatch/observability, with only real-network adapters excluded from live claims. |
| W7 | `TASK-106`-`TASK-119` | Thin signed-index implementation over real signing/digest/root/package primitives. |
| W8 | `TASK-063`-`TASK-070` | Final closure register, stale-term guardrails, evidence, verification, and phase status. |
| W9 | `TASK-121`-`TASK-125` | Mandatory local full-system wallet/node simulation blockers: node RPC, broadcast, fee source, remote scan worker, and spend/confirmation policy. |

## Current Code Evidence Anchors

### TODO Literal Mirror

| Area | Current evidence |
| --- | --- |
| Wallet lifecycle/error DTOs | `crates/z00z_wallets/src/rpc/tx_types.rs` has `TxStatus` and string `errors`, but no public `RuntimeTxLifecycle`, `RuntimeTxErrorCode`, or `error_codes` model. |
| Receive scan outcome | `crates/z00z_wallets/src/rpc/chain_types.rs` has `RuntimeScanStatus`, but no `RuntimeReceiveScanOutcome` or `last_receive_outcome`. |
| Tx history storage | `crates/z00z_wallets/src/persistence/tx_storage.rs` and `tx_storage_impl.rs` have durable history rows, but no conflict/already-spent lifecycle projection API. |
| Current receive lane | `crates/z00z_wallets/src/services/wallet_actions_receive.rs` has authoritative receive and worker evidence validation that new scan-outcome work MUST extend. |
| Object/voucher/right anchors | `crates/z00z_storage/src/settlement/record.rs`, `tx_plan_types.rs`, `crates/z00z_wallets/src/rpc/test_asset_impl.rs`, and simulator object-flow tests already provide partial coverage. |
| Local HJMT anchors | `crates/z00z_runtime/aggregators/src/batch_planner.rs`, `crates/z00z_storage/src/settlement/proof_batch.rs`, `proof_batch_verify.rs`, `hjmt_journal.rs`, and `hjmt_commit.rs` provide local proof/root/journal evidence. Distributed gaps MUST be closed by a local network/process simulator that reuses these primitives instead of inventing doc-only behavior. |
| HJMT runtime config home | `config/hjmt_runtime/sim_5a7s` contains planner, storage, route-table, and aggregator-process configs; `crates/z00z_rollup_node/src/config.rs` loads it as one `NodeConfig::from_hjmt_home`, while simulator tests point to it as `hjmt_runtime.config_root`. This is runtime/orchestration fixture ownership; storage owns durable/proof semantics, not runtime placement. |
| Asset/core ownership drift | `crates/z00z_core/src/assets/object_family.rs` and `assets/asset_error.rs` still own shared object/error concepts. |
| Thin signed-index absence | Current code has thin storage/witness wording, but no implemented signed-index transport lane matching the thin-mode paper; Phase 062 MUST implement it over real signing/digest/root primitives. |

The mirrored TODO anchors above already use current workspace homes; no extra
execution remap remains for these evidence rows.

## GSD Plan Generation Contract

| Required field | Source |
| --- | --- |
| `plan_id` | `Plan id` column from `Required GSD Plan Groups`. |
| `task_ids` | `Tasks` column from `Required GSD Plan Groups`. |
| `task_rows` | Matching rows from `Canonical Task Inventory`, including atomic fallback ids. |
| `source_refs` | Every included task's `Source refs to read first`; do not generate a plan without these links. |
| `inputs` | Source refs, task dependencies, group dependencies, and current-code anchors that MUST be read before implementation. |
| `outputs` | Group `Scope` plus every included task `Scope`, converted into concrete files/APIs/configs/tests/simulator artifacts/live-claim removals. |
| `dependencies` | Group `Depends on` plus relevant task-level dependencies. |
| `acceptance_tests` | Group `Acceptance / tests`, every included task's acceptance, plus any commands from `TASK-072`. |
| `simulation_gate` | Required local simulator scenarios for any network/node/DA/process behavior not backed by real deployment adapters. |
| `negative_tests` | Failure/tamper/stale/replay/restart/conflict cases implied by acceptance and source refs. |
| `plan_artifacts` | Plan-level artifact matrix from `Artifact/Test/Result Proof Contract`. |
| `plan_tests` | Plan-level test/scenario matrix from `Artifact/Test/Result Proof Contract`. |
| `plan_results` | Plan-level result/evidence matrix from `Artifact/Test/Result Proof Contract`. |
| `task_artifacts` | One artifact row per included task id from `Artifact/Test/Result Proof Contract`. |
| `task_tests` | One test/scenario row per included task id from `Artifact/Test/Result Proof Contract`. |
| `task_results` | One result/evidence row per included task id from `Artifact/Test/Result Proof Contract`. |
| `anti_placeholder_gate` | Evidence that artifacts are exercised through real project primitives and are not placeholder/scaffold-only. |
| `current_code_refs` | `Current Code Evidence Anchors` and the task source reading map when applicable. |
| `blockers` | `Pre-Plan Blockers` and per-task blocker template from `TASK-075`. |
| `evidence_gate` | Completion evidence proving each task passed input/output/acceptance/simulation gates, or a phase-failing blocker. |
| `not_recommendation_gate` | Explicit statement that all included rows are mandatory requirements, not nice-to-have recommendations. |

## Verification Checklist

| Check | Command / evidence |
| --- | --- |
| Continuous task count | `rg -o "TASK-[0-9]{3}" 062-TODO.md` MUST include `TASK-001` through `TASK-125`. |
| Grouped plan count | `rg -o "PLAN-062-G[0-9]{2}" 062-TODO.md` MUST include `PLAN-062-G01` through `PLAN-062-G27`. |
| Atomic fallback id count | `rg -o "PLAN-062-T[0-9]{3}" 062-TODO.md` MUST include `PLAN-062-T001` through `PLAN-062-T125`. |
| No alternate task namespaces | `rg "ASSET-[0-9]{3}|HJMT-[0-9]{3}|THIN-[0-9]{3}|SYS-[0-9]{3}|META-[0-9]{3}|\\| Unit \\|" 062-TODO.md` MUST return no rows. |
| Source-link compatibility | Every `Canonical Task Inventory` row has at least one Markdown link in `Source refs to read first`. |
| Source-corpus cross-link coverage | Canonical task source refs cover all seven source MD files in `062-Gaps-Closing-2`: `GAPS.md`, `asset-only.md`, `HJMT-REPORT.md`, `HJMT-RAID -Sharding.md`, `HJMT-Sharding-Storage-Techpaper.md`, `HJMT-структуры.md`, and `Z00Z-Thin-Transaction-Mode.md`. |
| All task rows linkable | Every table row beginning with `TASK-` has at least one Markdown link, including local full-system simulation register rows. |
| Requirement gates present | `Requirement Gate Contract` includes input, output, acceptance, simulation, evidence, and nice-to-have rejection gates. |
| Generated-plan gate fields | `GSD Plan Generation Contract` includes `inputs`, `outputs`, `acceptance_tests`, `simulation_gate`, `negative_tests`, `evidence_gate`, and `not_recommendation_gate`. |
| Artifact/test/result fields | `GSD Plan Generation Contract` includes `plan_artifacts`, `plan_tests`, `plan_results`, `task_artifacts`, `task_tests`, `task_results`, and `anti_placeholder_gate`. |
| No scaffold closure | `Requirement Gate Contract` includes `No scaffold gate`; closure evidence MUST prove behavior through real project primitives, not placeholders/scaffolds. |
| No recommendation-only rows | `rg "nice-to-have|recommendation rows|optional|best-effort" 062-TODO.md` is allowed to match only gate language that rejects those categories, never a task closure path. |
| HJMT runtime fixture ownership | `TASK-120` and the pre-plan blocker MUST keep `config/hjmt_runtime` as runtime/orchestration fixture ownership unless this phase deliberately moves only storage backend fragments into storage. |
| Full-system closure boundary | `TASK-001` through `TASK-120` are source-corpus backed; `TASK-121` through `TASK-125` are mandatory local full-system simulation blockers. |

## Canonical Task Inventory

To avoid creating a third 125-row task table while still reflecting the TODO
inventory:

- `062-TODO.md` remains the canonical task-inventory table.
- `062-COVERAGE.md` mirrors every `TASK-NNN -> PLAN-062-GNN` assignment for
  review.
- Each grouped `PLAN-062-GNN-PLAN.md` file preserves its owned rows in
  `copied_task_rows` and `Coverage Appendix`.
- `062-TEST-SPEC.md` and `062-TESTS-TASKS.md` preserve the same grouped-plan
  ownership and execution order for verification.
- Any mismatch across these mirrors is phase-failing drift.

## Local Full-System Simulation Closure Register

These five rows are outside the strict Phase 062 source corpus but mandatory for
yolo/full-system closure. They MUST be implemented through local node/network
simulation with real wallet/storage/crypto/policy primitives; only unavailable
external network adapters remain non-live.

| Task id | Code evidence | Required closure |
| --- | --- | --- |
| TASK-121 | [`chain_client_impl.rs`](../../../crates/z00z_wallets/src/chain/chain_client_impl.rs#L19-L99) now routes tip/block/header/submit/status/network-info calls through `LocalNodeSim` or an explicit remote adapter seam. | Preserve local node RPC simulation as the live wallet path and keep only real remote transport adapter-only. |
| TASK-122 | [`broadcast_impl.rs`](../../../crates/z00z_wallets/src/chain/broadcast_impl.rs#L15-L27), [`broadcast_impl.rs`](../../../crates/z00z_wallets/src/chain/broadcast_impl.rs#L106-L160) now own durable submit/retry/confirm persistence on the wallet chain/tx-store seam. | Preserve retry, timeout, reject, duplicate, reorg/replacement, and confirmation coverage on the same durable lifecycle path. |
| TASK-123 | [`fee_estimator.rs`](../../../crates/z00z_wallets/src/tx/fee_estimator.rs#L139-L158), [`local_node_sim.rs`](../../../crates/z00z_wallets/src/chain/local_node_sim.rs#L28-L37) now provide simulated-live fee-rate sourcing with cache/fallback seams and an adapter-only remote source boundary. | Preserve cache/fallback/stale/zero/spike fee-source behavior on the live estimator path without introducing a second authority plane. |
| TASK-124 | [`scan_engine_impl.rs`](../../../crates/z00z_wallets/src/chain/scan_engine_impl.rs#L1-L31), [`wallet_actions_receive.rs`](../../../crates/z00z_wallets/src/services/wallet_actions_receive.rs#L443-L492) now keep the remote worker subordinate to authoritative wallet-local receive verification. | Preserve restart, stale, malicious, and no-mutation worker behavior through the local node simulation and the authoritative receive lane. |
| TASK-125 | [`policy.rs`](../../../crates/z00z_wallets/src/wallet/policy.rs#L85-L129), [`tx_storage_impl.rs`](../../../crates/z00z_wallets/src/persistence/tx_storage_impl.rs#L275-L343), [`tx_rpc_support.rs`](../../../crates/z00z_wallets/src/rpc/tx_rpc_support.rs#L55-L105) now derive spend context from canonical tx history and enforce daily-limit/confirmation gates on the live RPC path. | Preserve definition-id keyed spend aggregation, restart persistence, and typed daily-limit/confirmation failures for simulated/live spending flows. |

</normative_mirror>

<specifics>
## 🔎 Specific Ideas

- Coverage audit target: 125 unique task ids, 27 grouped plan ids, and exact one-to-one task-to-plan mapping.
- Phase waves stay aligned to the TODO packet: planning contract first, then storage/wallet authority, then privacy/object lanes, then genesis/HJMT, then thin mode, then final closure plus local full-system blockers.
- Each grouped plan includes copied task rows, plan and task matrices, current code refs, plan artifacts, task artifacts, plan tests, task tests, plan results, task results, evidence gate, anti-placeholder gate, and a coverage appendix.
- Only external or adapter seams may remain non-live after a local simulator is in place.

</specifics>

<deferred>
## 🚫 Deferred Ideas

None for local correctness. The only permitted non-live residuals after Phase 062 execution are external transport or deployment adapters that sit beyond the local deterministic simulator boundary and are explicitly registered as adapter-only exclusions.

</deferred>

<scope_fence>
## 🧱 Scope Fence

- Do not create a future-only work item for any local correctness hole that can be closed by current code, tests, or a local deterministic simulator.
- Do not let distributed HJMT claims survive as prose-only architecture notes.
- Do not let thin mode survive as naming or DTO drift without a real signed-index lane.
- Do not allow phase status to reach complete while any grouped plan lacks evidence, acceptance, or blocker classification.

</scope_fence>

---

*Phase: 062-gaps-closing-2*
*Context gathered: 2026-06-24 via PRD Express Path*
