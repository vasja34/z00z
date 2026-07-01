# Phase 062 TODO

Date: 2026-06-23

Scope: every Markdown source in `.planning/phases/062-Gaps-Closing-2` was checked as a planning source against the current workspace code. This audit is a GSD input artifact. Graph data was not used as implementation evidence.

## Verdict

The prior audit format was not strict enough for GSD plan generation because it mixed task-id namespaces (`TASK`, `META`, `ASSET`, `HJMT`, `THIN`, `SYS`) and did not put concrete source links on every task row.

This revision makes the inventory plan-ready:

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

The keywords `MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, and `MAY` are normative for Phase 062 plan generation and implementation.

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

Generate **27 grouped plans** in yolo/full-system mode: `PLAN-062-G01` through `PLAN-062-G27`. Do not stop at `TASK-120`; `TASK-121` through `TASK-125` are mandatory local full-system simulation closures. The 125 task rows remain the mandatory traceability inventory and can be split into atomic fallback plans only when a group becomes too large or risky.

## Required GSD Plan Groups

These are the plan files to generate first. Each plan MUST copy the listed task ids and their exact source refs from the canonical inventory.

| Plan id | Tasks | Scope | Depends on | Acceptance / tests |
| --- | --- | --- | --- | --- |
| PLAN-062-G01 | `TASK-071`-`TASK-075` | Plan-generation contract, DoD, verification commands, source closure map, blocker/completion templates. | None | Generated plans preserve task ids, source refs, dependencies, acceptance, and evidence format. |
| PLAN-062-G02 | `TASK-001`-`TASK-004` | Storage root authority, backend env normalization, and root-model closeout. | PLAN-062-G01 | Root/env drift is patched; guardrail tests and docs prove one semantic settlement-root authority. |
| PLAN-062-G03 | `TASK-005`-`TASK-009` | Claim-root, checkpoint, route-publication, local-publication, restart/tamper, and simulator checkpoint evidence. | PLAN-062-G02 | Checkpoint/root/publication tests and evidence rows prove route binding, tamper rejection, and reload consistency. |
| PLAN-062-G04 | `TASK-010`-`TASK-013`, `TASK-103` | HJMT benchmark/proof-size evidence and measurement sidecar/guardrail closure. | PLAN-062-G03 | Durable-root-published metrics are separated; sidecar or equivalent measurement guardrail is implemented and tested when claims require it. |
| PLAN-062-G05 | `TASK-014`-`TASK-018`, `TASK-022`, `TASK-038` | Wallet lifecycle DTOs, tx-history storage/projection, response threading, and restart-safe history convergence. | PLAN-062-G01 | Lifecycle DTO/storage/RPC tests pass; row hash/fold/current view/tombstone/status behavior is proven. |
| PLAN-062-G06 | `TASK-019`-`TASK-021`, `TASK-025`, `TASK-029`, `TASK-035`, `TASK-037` | Typed wallet tx error taxonomy across unsupported versions, package verify/import, parse failures, and docs. | PLAN-062-G05 | Public typed errors/error codes replace string-only failure surfaces; import/verify no-mutation tests pass. |
| PLAN-062-G07 | `TASK-024`, `TASK-026`-`TASK-028`, `TASK-036` | Receive scan outcome DTO/status, worker validation, and atomic cursor persistence. | PLAN-062-G05, PLAN-062-G06 | `RuntimeReceiveScanOutcome`/equivalent, worker no-mutation, and failpoint restart atomicity are tested. |
| PLAN-062-G08 | `TASK-030`, `TASK-031` | Request-bound inbox helper and inbox no-mutation/order behavior. | PLAN-062-G07 | Inbox consumes request validation, is implemented as non-authoritative/off-consensus, and cannot replace receive scan authority. |
| PLAN-062-G09 | `TASK-032`-`TASK-034`, `TASK-039` | Simulator wallet lifecycle/import/scan/history evidence and docs. | PLAN-062-G05 through PLAN-062-G08 | Simulator evidence joins scan/import/history/publication digests without secret leakage. |
| PLAN-062-G10 | `TASK-040`, `TASK-041`, `TASK-046` | Field-native/Poseidon2 package closure, negative package cases, and live-claim cleanup. | PLAN-062-G01 | Field-native parity is implemented if claimed live; otherwise live claims are removed and canonical real-crypto package tests pass. |
| PLAN-062-G11 | `TASK-042`-`TASK-045`, `TASK-047`, `TASK-048` | Privacy, reveal, selective disclosure, logging, backup/export, and transport/package hygiene. | PLAN-062-G06, PLAN-062-G10 | Reveal matrix and sensitive-log/backup/report grep have no unowned leaks; privacy docs are scoped. |
| PLAN-062-G12 | `TASK-023`, `TASK-049`-`TASK-051`, `TASK-057`, `TASK-060` | Cash/object separation, object inventory, fee envelope, voucher lifecycle, and object wallet docs. | PLAN-062-G01, PLAN-062-G05 | Object RPC/simulator tests cover unknown policy, missing right, voucher-as-cash, right-as-value, and fee boundaries. |
| PLAN-062-G13 | `TASK-052`-`TASK-056`, `TASK-058`, `TASK-059`, `TASK-061`, `TASK-062` | Business rights, local adapter, agentic rights, machine capability, and related closeout docs. | PLAN-062-G12 | Local-only scenarios prove adapter bounds, agentic rights, machine capability reuse/replay/wrong-action failures. |
| PLAN-062-G14 | `TASK-076`-`TASK-078`, `TASK-082`-`TASK-084` | Asset/core genesis manifest split, referenced YAML loader, validation, schema/goldens, and `actions_config.yaml` decision. | PLAN-062-G01 | Manifest and refs feed `GenesisConfig`; duplicate/path/schema tests pass; action catalog decision is explicit. |
| PLAN-062-G15 | `TASK-079`-`TASK-081` | Asset/shared object vocabulary, generic error ownership, and misplaced owner test relocation. | PLAN-062-G01 | `ObjectFamily`/`ObjectRoleV1`, generic errors, and owner tests move or re-export cleanly without drift. |
| PLAN-062-G16 | `TASK-085`-`TASK-089`, `TASK-102`, `TASK-104`, `TASK-120` | Local HJMT proof/root/route/history, validator/watcher rollup, storage/wallet boundary, HJMT structure ownership, and `config/hjmt_runtime` layout ownership. | PLAN-062-G02 through PLAN-062-G04 | Local HJMT evidence is closed; wallet sees public proofs/API only; storage-owned scope creation and runtime fixture ownership are documented. |
| PLAN-062-G17 | `TASK-090`-`TASK-092`, `TASK-099`-`TASK-101`, `TASK-105` | Local distributed HJMT simulation for journal replication, quorum, standby catch-up, consensus/membership, split-brain negatives, and adapter-only exclusion register. | PLAN-062-G16 | A deterministic local-network simulator drives multiple aggregator runtimes over real planner/storage/journal/proof primitives; only real transport/chain-network adapters are excluded from live claims. |
| PLAN-062-G18 | `TASK-093`-`TASK-098` | Local distributed HJMT route rollout, scheduler, remote dispatch, cross-shard policy, storage locks, and observability simulation. | PLAN-062-G16, PLAN-062-G17 | Route activation, dispatch, process acks, storage locks, and drift observability are tested in the local simulator; static/local limitations cannot replace simulator coverage. |
| PLAN-062-G19 | `TASK-106`-`TASK-108`, `TASK-111`, `TASK-119` | Thin signed-index DTO, snapshot/authentication model, helper APIs, and signature/root-name drift correction. | PLAN-062-G05, PLAN-062-G16 | Thin index surfaces are implemented over real signing/digest/root primitives and preserve canonical package/root semantics. |
| PLAN-062-G20 | `TASK-109`, `TASK-110`, `TASK-112`, `TASK-113` | Thin cache/refresh/fallback, helper expansion, builder sharing, and no thin-specific runtime semantics. | PLAN-062-G19 | Thin expands before runtime admission; no `ThinWorkItem`, thin verdict, or second theorem exists. |
| PLAN-062-G21 | `TASK-114`-`TASK-118` | Thin restart/cache/default-thick, equivalence, negative index, fallback, and privacy tests. | PLAN-062-G19, PLAN-062-G20 | Thick fallback is always safe; stale/wrong/equivocated/withheld/expired cases fail closed. |
| PLAN-062-G22 | `TASK-063`-`TASK-070` | Final closure register, stale-term guardrails, task evidence, focused/broad validation, drift grep, and phase status. | PLAN-062-G02 through PLAN-062-G21 | All task evidence and blockers are closed or phase-failing; phase status changes only after validation and doublecheck. |
| PLAN-062-G23 | `TASK-121` | Wallet `ChainClient` node RPC behavior through local node simulation and adapter seams. | PLAN-062-G05, PLAN-062-G06 | Tip/block/header/submit/status/network-info paths work against local simulated node state, with integration-gated real-node adapter tests when available. |
| PLAN-062-G24 | `TASK-122` | Broadcast submission, retry, confirmation polling, and tx-store lifecycle integration. | PLAN-062-G23 | Broadcast retry/polling persists lifecycle updates and handles timeout, reject, duplicate, reorg/replacement, and confirmation paths. |
| PLAN-062-G25 | `TASK-123` | Simulated-live network fee-rate source. | PLAN-062-G23 | Fee estimator uses local simulated source with cache/fallback/stale/zero/spike tests and adapter seam for real network source. |
| PLAN-062-G26 | `TASK-124` | Local remote scan worker simulation and trust-boundary implementation. | PLAN-062-G07 | Remote worker is implemented against local chain/node simulation with trust-boundary, no-mutation, restart, and malicious/stale worker tests. |
| PLAN-062-G27 | `TASK-125` | Wallet daily-spend and confirmation policy enforcement. | PLAN-062-G12, PLAN-062-G23 | Policy tests enforce daily limits, confirmation requirements, restart persistence, multi-send aggregation, and rejection surfaces. |

## Pre-Plan Blockers

These blockers MUST be resolved before generating grouped GSD plans:

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

Every task row is a requirement. Generated plans MUST reject rows that cannot be mapped through all gates below. Generated plans MUST NOT downgrade a row into a recommendation.

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

Each generated grouped plan MUST include both a plan-level matrix and a task-level matrix. The task-level matrix MUST have one row for every included `TASK-NNN`.

| Matrix | Required columns | Closure rule |
| --- | --- | --- |
| `plan_artifacts` | `artifact_id`, `path_or_api`, `kind`, `owner_crate`, `created_or_modified`, `source_task_ids` | The plan MUST list every code/docs/config/test/simulator artifact needed for the group. |
| `plan_tests` | `test_id`, `command`, `test_module_or_scenario`, `positive_or_negative`, `proves` | The plan MUST list executable tests or simulator scenarios proving the group behavior. |
| `plan_results` | `result_id`, `command_or_artifact`, `expected_result`, `evidence_path`, `closure_status` | The plan MUST list the evidence that demonstrates correctness and non-placeholder implementation. |
| `task_artifacts` | `task_id`, `artifact_id`, `path_or_api`, `kind`, `implementation_depth` | Every task MUST list its own concrete artifacts; `implementation_depth` MUST be `full`, `simulated-full`, or `live-claim-removed`. |
| `task_tests` | `task_id`, `test_id`, `command_or_scenario`, `positive_or_negative`, `failure_modes` | Every task MUST list tests for success and required negative/failure cases. |
| `task_results` | `task_id`, `result_id`, `proof_artifact`, `pass_condition`, `anti_placeholder_evidence` | Every task MUST list result proof that its implementation is exercised through real project primitives. |

Generated plans SHOULD include concise rationale for why each artifact/test/result maps to the task source refs. Generated plans MUST NOT close a task with empty matrices, TODO comments, uncalled helper code, compile-only proof, docs-only proof for runtime behavior, or tests that only assert a scaffold exists.

## Current Wallet Path Rewrite Map

Historical intermediate aliases from the Phase 061 refactor were collapsed on
2026-06-25. This table is now the single canonical rewrite map for Phase 062
execution.

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

Waves are scheduling groups only. They are not replacement plan ids.

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

## Canonical Task Inventory

Each row below is one atomic task. Grouped plans MUST copy every included task row into the generated plan body. `Source refs to read first` is mandatory before implementation.

| Task id | Atomic fallback id | Source refs to read first | Scope | Depends on | Acceptance / tests |
| --- | --- | --- | --- | --- | --- |
| TASK-001 | PLAN-062-T001 | [body](./GAPS.md#L136), [reading map](./GAPS.md#L2449) | Settlement root authority and storage migration boundary. | None | Docs/code prove one semantic settlement-root authority; stale root names are removed or registered. |
| TASK-002 | PLAN-062-T002 | [body](./GAPS.md#L166), [reading map](./GAPS.md#L2450) | Backend env normalization to current HJMT mode. | TASK-001 | Guardrail tests accept only unset or `Z00Z_SETTLEMENT_BACKEND_MODE=hjmt`; old env wording is patched. |
| TASK-003 | PLAN-062-T003 | [body](./GAPS.md#L194), [reading map](./GAPS.md#L2451) | Backend root MUST NOT become public semantic root. | TASK-002 | Backend equivalence stays internal; tests/docs reject second public-root authority. |
| TASK-004 | PLAN-062-T004 | [body](./GAPS.md#L222), [reading map](./GAPS.md#L2452) | Root-model closeout evidence. | TASK-003 | TODO evidence cites current root/proof files and no legacy fallback remains. |
| TASK-005 | PLAN-062-T005 | [body](./GAPS.md#L251), [reading map](./GAPS.md#L2453) | Claim-root to checkpoint authority closure. | TASK-004 | Checkpoint claim-root and proof binding are documented and negatively tested. |
| TASK-006 | PLAN-062-T006 | [body](./GAPS.md#L284), [reading map](./GAPS.md#L2454) | Checkpoint tamper/reload evidence. | TASK-005 | Reload/tamper tests reuse the same checkpoint verifier path. |
| TASK-007 | PLAN-062-T007 | [body](./GAPS.md#L321), [reading map](./GAPS.md#L2455) | Local publication boundary and simulator evidence. | TASK-006 | Local publication/DA simulation proves checkpoint/root binding; external DA transport is excluded from live claims until an adapter exists. |
| TASK-008 | PLAN-062-T008 | [body](./GAPS.md#L351), [reading map](./GAPS.md#L2456) | Publication restart/tamper harness. | TASK-007 | Scenario evidence proves publication/checkpoint/root binding across restart/tamper. |
| TASK-009 | PLAN-062-T009 | [body](./GAPS.md#L378), [reading map](./GAPS.md#L2457) | Simulator checkpoint evidence pack. | TASK-008 | Stage artifact shape, tamper rejection, and restart consistency are covered. |
| TASK-010 | PLAN-062-T010 | [body](./GAPS.md#L411), [reading map](./GAPS.md#L2458) | Benchmark/proof-size evidence guardrail. | TASK-009 | Performance reports remain evidence metadata and do not change storage authority. |
| TASK-011 | PLAN-062-T011 | [body](./GAPS.md#L438), [reading map](./GAPS.md#L2459) | Performance-report overclaim guardrail. | TASK-010 | Guardrails reject proof-size/TPS overclaims not backed by local artifact verification. |
| TASK-012 | PLAN-062-T012 | [body](./GAPS.md#L463), [reading map](./GAPS.md#L2460) | Measurement sidecar/guardrail decision. | TASK-011 | If benchmark/proof-size claims need a sidecar, implement it; otherwise tests prove existing measured artifacts are sufficient and no unmeasured claim remains. |
| TASK-013 | PLAN-062-T013 | [body](./GAPS.md#L492), [reading map](./GAPS.md#L2461) | Sidecar or equivalent measurement guardrail implementation. | TASK-012 | Sidecar/guardrail compares active storage/proof behavior without becoming authority; no skip-only closure is allowed for live claims. |
| TASK-014 | PLAN-062-T014 | [body](./GAPS.md#L530), [reading map](./GAPS.md#L2462) | Wallet receive/import/history authority. | TASK-013 or TASK-012 skip | Receive, package import, asset persistence, and tx history stay one authority lane. |
| TASK-015 | PLAN-062-T015 | [body](./GAPS.md#L582), [reading map](./GAPS.md#L2463) | Public tx lifecycle DTOs and row taxonomy. | TASK-014 | `RuntimeTxLifecycle` and terminal lifecycle states are added or exact equivalents proven. |
| TASK-016 | PLAN-062-T016 | [body](./GAPS.md#L611), [reading map](./GAPS.md#L2464) | Durable tx-history listing/details APIs. | TASK-015 | History listing/details project durable JSONL state, not an in-memory pending list. |
| TASK-017 | PLAN-062-T017 | [body](./GAPS.md#L647), [reading map](./GAPS.md#L2465) | Tx-history replay/fold extension. | TASK-016 | Row taxonomy extension preserves hash chain and fold semantics. |
| TASK-018 | PLAN-062-T018 | [body](./GAPS.md#L685), [reading map](./GAPS.md#L2466) | RPC lifecycle fields across wallet responses. | TASK-017 | Send/details/reconcile/import/verify responses expose lifecycle projection. |
| TASK-019 | PLAN-062-T019 | [body](./GAPS.md#L721), [reading map](./GAPS.md#L2467) | Unsupported receive-version taxonomy. | TASK-018 | Decode/report layers expose stable typed reject classes. |
| TASK-020 | PLAN-062-T020 | [body](./GAPS.md#L755), [reading map](./GAPS.md#L2468) | Offline package verify report hardening. | TASK-019 | Verify/report remains pre-broadcast and returns deterministic reject classes. |
| TASK-021 | PLAN-062-T021 | [body](./GAPS.md#L787), [reading map](./GAPS.md#L2469) | Package import hardening and rollback. | TASK-020 | Import path is idempotent/conflict-aware and rollback-safe. |
| TASK-022 | PLAN-062-T022 | [body](./GAPS.md#L832), [reading map](./GAPS.md#L2470) | Admission/confirmation lifecycle visibility. | TASK-021 | Admission/confirmation rows become user-visible without claiming consensus finality. |
| TASK-023 | PLAN-062-T023 | [body](./GAPS.md#L868), [reading map](./GAPS.md#L2471) | Cancel/export/history mutation tests. | TASK-022 | Row order, rollback, tombstone, folded view, and cash/object separation are tested. |
| TASK-024 | PLAN-062-T024 | [body](./GAPS.md#L896), [reading map](./GAPS.md#L2472) | Worker evidence no-mutation boundary. | TASK-023 | Invalid worker evidence cannot mutate assets or scan cursor. |
| TASK-025 | PLAN-062-T025 | [body](./GAPS.md#L931), [reading map](./GAPS.md#L2473) | RPC typed unsupported-version mapping. | TASK-024 | Stable wallet/RPC error codes cover unsupported package/import/receive versions. |
| TASK-026 | PLAN-062-T026 | [body](./GAPS.md#L967), [reading map](./GAPS.md#L2474) | Public receive scan outcome DTO. | TASK-025 | `RuntimeReceiveScanOutcome` and `last_receive_outcome` or exact equivalents exist. |
| TASK-027 | PLAN-062-T027 | [body](./GAPS.md#L1002), [reading map](./GAPS.md#L2475) | Scan orchestration authority and cursor persistence. | TASK-026 | Scan status binds to wallet scan authority and documented limits. |
| TASK-028 | PLAN-062-T028 | [body](./GAPS.md#L1041), [reading map](./GAPS.md#L2476) | Worker-assisted scan re-entry. | TASK-027 | Worker-assisted evidence re-enters authoritative scan path after strict validation. |
| TASK-029 | PLAN-062-T029 | [body](./GAPS.md#L1072), [reading map](./GAPS.md#L2477) | Portable parse and verify report error mapping. | TASK-028 | Parse/verify failures expose deterministic typed reject classes. |
| TASK-030 | PLAN-062-T030 | [body](./GAPS.md#L1103), [reading map](./GAPS.md#L2478) | Request-bound inbox helper implementation/decision. | TASK-029 | Inbox helper consumes payment-request validation and remains non-authoritative/off-consensus. |
| TASK-031 | PLAN-062-T031 | [body](./GAPS.md#L1138), [reading map](./GAPS.md#L2479) | Inbox ordering and no-mutation tests. | TASK-030 | Inbox ordering cannot replace scan authority or persistence. |
| TASK-032 | PLAN-062-T032 | [body](./GAPS.md#L1170), [reading map](./GAPS.md#L2480) | Simulator receive/import/history evidence. | TASK-031 | Simulator proves wallet authority after lower-level wallet tasks are stable. |
| TASK-033 | PLAN-062-T033 | [body](./GAPS.md#L1213), [reading map](./GAPS.md#L2481) | Simulator artifact inventory and redaction. | TASK-032 | Simulator evidence includes wallet scan/history flows without secret leakage. |
| TASK-034 | PLAN-062-T034 | [body](./GAPS.md#L1241), [reading map](./GAPS.md#L2482) | Receive/import/history documentation closeout. | TASK-033 | Docs point to code-owned authority paths: receive lane, tx storage, package import. |
| TASK-035 | PLAN-062-T035 | [body](./GAPS.md#L1271), [reading map](./GAPS.md#L2483) | Unsupported receive-version doc/test closeout. | TASK-034 | Receive-pack decode and RPC/error mapping tests are cited. |
| TASK-036 | PLAN-062-T036 | [body](./GAPS.md#L1301), [reading map](./GAPS.md#L2484) | Scan status tests and docs. | TASK-035 | Tests prove status projection plus persistence cursor behavior. |
| TASK-037 | PLAN-062-T037 | [body](./GAPS.md#L1330), [reading map](./GAPS.md#L2485) | Offline package hardening tests/docs. | TASK-036 | Verify/report/import separation and portable metadata mismatch tests pass. |
| TASK-038 | PLAN-062-T038 | [body](./GAPS.md#L1358), [reading map](./GAPS.md#L2486) | Tx-history convergence tests/docs. | TASK-037 | JSONL hash/fold, current view, tombstone, and status-update tests pass. |
| TASK-039 | PLAN-062-T039 | [body](./GAPS.md#L1388), [reading map](./GAPS.md#L2487) | Simulator wallet evidence closeout. | TASK-038 | Simulator joins scan/import/history/publication digests without secret leakage. |
| TASK-040 | PLAN-062-T040 | [body](./GAPS.md#L1412), [reading map](./GAPS.md#L2488) | Field-native pack migration decision. | TASK-039 | Current fixed wire behavior is frozen before any field-native migration claim. |
| TASK-041 | PLAN-062-T041 | [body](./GAPS.md#L1444), [reading map](./GAPS.md#L2489) | Field-pack negative cases. | TASK-040 | Tamper/version/replay/error-code package tests pass or missing cases are added. |
| TASK-042 | PLAN-062-T042 | [body](./GAPS.md#L1480), [reading map](./GAPS.md#L2490) | Privacy/stealth primitive closure. | TASK-041 | Request validation, receive taxonomy, and wallet-local secret boundaries hold. |
| TASK-043 | PLAN-062-T043 | [body](./GAPS.md#L1514), [reading map](./GAPS.md#L2491) | Selective disclosure and redaction. | TASK-042 | Logging and backup metadata policies back every public reveal claim. |
| TASK-044 | PLAN-062-T044 | [body](./GAPS.md#L1541), [reading map](./GAPS.md#L2492) | Package hygiene and portable metadata. | TASK-043 | Package parse/import/export/log surfaces are redaction-safe. |
| TASK-045 | PLAN-062-T045 | [body](./GAPS.md#L1583), [reading map](./GAPS.md#L2493) | Backup/export hygiene. | TASK-044 | Encrypted export versioning, AAD, manifest, and metadata redaction align. |
| TASK-046 | PLAN-062-T046 | [body](./GAPS.md#L1616), [reading map](./GAPS.md#L2494) | Field-native/Poseidon2 implementation or live-claim removal closeout. | TASK-045 | Future proof-system names cannot be read as live pack truth; either implement the field-native path with tests or remove the live claim while proving canonical real-crypto package behavior. |
| TASK-047 | PLAN-062-T047 | [body](./GAPS.md#L1642), [reading map](./GAPS.md#L2495) | Privacy closeout docs. | TASK-046 | Docs state wallet/package privacy limits and no transport anonymity claim. |
| TASK-048 | PLAN-062-T048 | [body](./GAPS.md#L1671), [reading map](./GAPS.md#L2496) | Transport/package privacy verification. | TASK-047 | Logs, backups, reports, and simulator artifacts do not leak secret material. |
| TASK-049 | PLAN-062-T049 | [body](./GAPS.md#L1695), [reading map](./GAPS.md#L2497) | Multi-asset/object-family bounded closure. | TASK-048 | Existing object-family anchors are verified and missing negative cases are added. |
| TASK-050 | PLAN-062-T050 | [body](./GAPS.md#L1733), [reading map](./GAPS.md#L2498) | Fee envelope and rights wallet extensions. | TASK-049 | `wallet.object.*` owns typed inventory; `wallet.asset.*` remains cash-only. |
| TASK-051 | PLAN-062-T051 | [body](./GAPS.md#L1766), [reading map](./GAPS.md#L2499) | Voucher lifecycle scenario. | TASK-050 | Voucher issue/redeem/reject path uses existing voucher/action/verifier surfaces. |
| TASK-052 | PLAN-062-T052 | [body](./GAPS.md#L1799), [reading map](./GAPS.md#L2500) | Payroll/B2B/useful-work local scenarios. | TASK-051 | Scenarios stay local object/right/voucher evidence, not oracle/live-service truth. |
| TASK-053 | PLAN-062-T053 | [body](./GAPS.md#L1832), [reading map](./GAPS.md#L2501) | Local adapter model without live chains. | TASK-052 | Mock DA/runtime boundaries are used; no live chain/DA scheduling is introduced. |
| TASK-054 | PLAN-062-T054 | [body](./GAPS.md#L1866), [reading map](./GAPS.md#L2502) | Adapter metadata and forged-input negatives. | TASK-053 | Forged adapter metadata and replay/custody overclaims are rejected. |
| TASK-055 | PLAN-062-T055 | [body](./GAPS.md#L1898), [reading map](./GAPS.md#L2503) | Agentic rights local simulation. | TASK-054 | Agentic rights bind to live right fixtures and local policy verdicts. |
| TASK-056 | PLAN-062-T056 | [body](./GAPS.md#L1938), [reading map](./GAPS.md#L2504) | Machine capability local simulation. | TASK-055 | `RightClass::MachineCapability` one-time/reuse/wrong-action behavior is tested. |
| TASK-057 | PLAN-062-T057 | [body](./GAPS.md#L1979), [reading map](./GAPS.md#L2505) | Multi-asset bounded doc closeout. | TASK-056 | Known object families, unknown-policy quarantine, and no voucher/right-as-cash leak are evidenced. |
| TASK-058 | PLAN-062-T058 | [body](./GAPS.md#L2004), [reading map](./GAPS.md#L2506) | Local adapter doc closeout. | TASK-057 | Docs prove local simulation status over real policy/right/voucher primitives and no live chain/DA/testnet claim. |
| TASK-059 | PLAN-062-T059 | [body](./GAPS.md#L2029), [reading map](./GAPS.md#L2507) | Voucher/payroll/B2B evidence closeout. | TASK-058 | Object-flow artifacts and negative cases are cited. |
| TASK-060 | PLAN-062-T060 | [body](./GAPS.md#L2054), [reading map](./GAPS.md#L2508) | Fee/right wallet docs closeout. | TASK-059 | `wallet.object.*` and `wallet.asset.*` boundaries are documented with tests. |
| TASK-061 | PLAN-062-T061 | [body](./GAPS.md#L2083), [reading map](./GAPS.md#L2509) | Agentic rights docs closeout. | TASK-060 | Agent budget/service/data-access profiles bind to live right fixtures. |
| TASK-062 | PLAN-062-T062 | [body](./GAPS.md#L2108), [reading map](./GAPS.md#L2510) | Machine capability docs closeout. | TASK-061 | Bounded right usage, replay/missing-right failures, and no full-wallet grant are documented. |
| TASK-063 | PLAN-062-T063 | [body](./GAPS.md#L2132), [reading map](./GAPS.md#L2511) | Closure register creation. | TASK-062 | Live, compatibility, simulation-only, adapter-only, and removed-claim terms are classified; no local correctness blocker remains unresolved. |
| TASK-064 | PLAN-062-T064 | [body](./GAPS.md#L2169), [reading map](./GAPS.md#L2512) | Residual privacy/metadata hardening. | TASK-063 | Secret reveal, backup/export metadata, and public wallet-id caveats are audited. |
| TASK-065 | PLAN-062-T065 | [body](./GAPS.md#L2201), [reading map](./GAPS.md#L2513) | Documentation normalization. | TASK-064 | Future proof-system and field-native pack claims are removed or scoped. |
| TASK-066 | PLAN-062-T066 | [body](./GAPS.md#L2229), [reading map](./GAPS.md#L2514) | Cross-crate closure tests and pointer normalization. | TASK-065 | Rule-owner crate tests precede simulator integration; stale TODO pointers are normalized. |
| TASK-067 | PLAN-062-T067 | [body](./GAPS.md#L2255), [reading map](./GAPS.md#L2515) | Focused final validation. | TASK-066 | Focused validation suites from the source task pass; any blocker is converted into a Phase 062 implementation/simulation task or fails the phase gate. |
| TASK-068 | PLAN-062-T068 | [body](./GAPS.md#L2313), [reading map](./GAPS.md#L2516) | Broad repo validation. | TASK-067 | Normal repo validation passes or exact unrelated failures are documented. |
| TASK-069 | PLAN-062-T069 | [body](./GAPS.md#L2342), [reading map](./GAPS.md#L2517) | Final drift grep. | TASK-068 | TODO/source/root/wallet/package/simulator drift is reconciled. |
| TASK-070 | PLAN-062-T070 | [body](./GAPS.md#L2375), [reading map](./GAPS.md#L2518) | Final completion/fail-gate status. | TASK-069 | Phase status changes only after all evidence fields are accurate and no local implementation blocker remains unresolved. |
| TASK-071 | PLAN-062-T071 | [execution contract](./GAPS.md#L15-L28), [DoD](./GAPS.md#L82-L90) | Execute the planning contract, not only code tasks. | None | Generated plans preserve numeric order, evidence requirements, and DoD. |
| TASK-072 | PLAN-062-T072 | [final verification commands](./GAPS.md#L92-L132) | Preserve focused verification commands as plan acceptance. | TASK-071 | Each generated plan records relevant focused commands; blockers MUST be closed in Phase 062 or become explicit local simulator tests. |
| TASK-073 | PLAN-062-T073 | [source closure matrix](./GAPS.md#L2407-L2435) | Preserve source-section closure mapping. | TASK-001 through TASK-070 | Closure matrix rows are updated only when every linked task is complete. |
| TASK-074 | PLAN-062-T074 | [doublecheck reading map](./GAPS.md#L2437-L2518) | Preserve per-task doublecheck sources. | TASK-001 through TASK-070 | Every plan includes body refs plus reading-map refs before implementation. |
| TASK-075 | PLAN-062-T075 | [blocker template](./GAPS.md#L2520-L2534), [completion template](./GAPS.md#L2535-L2549) | Preserve blocker/completion evidence format. | TASK-071 through TASK-074 | All task plans use exact evidence fields and classify blockers as closed-by-code, closed-by-local-simulation, external-adapter-only, or phase-failing. |
| TASK-076 | PLAN-062-T076 | [asset ownership finding](./asset-only.md#L25-L36), [YAML authority](./asset-only.md#L50-L69) | Keep `GenesisConfig` as canonical authority while adding production root manifest intent. | TASK-071 | Manifest work cannot create a second bootstrap authority. |
| TASK-077 | PLAN-062-T077 | [root manifest requirement](./asset-only.md#L56-L68), [order](./asset-only.md#L78-L84) | Split production YAML into root manifest plus referenced subfiles. | TASK-076 | Loader assembles manifest and refs into the same `GenesisConfig` shape. |
| TASK-078 | PLAN-062-T078 | [complexity notes](./asset-only.md#L71-L76), [order](./asset-only.md#L78-L84) | Add deterministic path resolution, duplicate rejection, and merged validation. | TASK-077 | Schema/golden tests reject duplicate keys and invalid referenced files. |
| TASK-079 | PLAN-062-T079 | [shared vocabulary evidence](./asset-only.md#L31-L33), [move list](./asset-only.md#L43-L48) | Move or re-export shared object vocabulary out of `assets`. | TASK-071 | `ObjectFamily` and `ObjectRoleV1` ownership is corrected without breaking imports. |
| TASK-080 | PLAN-062-T080 | [shared error evidence](./asset-only.md#L34-L36), [move list](./asset-only.md#L43-L48) | Re-home generic asset/object/genesis error variants. | TASK-079 | `AssetError` no longer misrepresents generic object/genesis validation ownership. |
| TASK-081 | PLAN-062-T081 | [mixed tests evidence](./asset-only.md#L35-L36), [move list](./asset-only.md#L43-L48) | Move misplaced owner tests out of asset-owned include paths. | TASK-079, TASK-080 | Rights/voucher/policy/action tests live under their owner module test surfaces. |
| TASK-082 | PLAN-062-T082 | [manifest complexity](./asset-only.md#L71-L76), [order](./asset-only.md#L78-L84) | Add schema and golden vectors for root manifest plus referenced files. | TASK-076 through TASK-078 | Golden tests prove deterministic merge and validation behavior. |
| TASK-083 | PLAN-062-T083 | [compatibility fixture evidence](./asset-only.md#L48-L54), [production manifest requirement](./asset-only.md#L64-L69) | Update docs to distinguish compatibility `assets_config.yaml` from production manifest layout. | TASK-076, TASK-082 | Docs do not describe compatibility config as production authority. |
| TASK-084 | PLAN-062-T084 | [actions note](./asset-only.md#L3-L7), [actions decision](./asset-only.md#L52-L54), [production manifest requirement](./asset-only.md#L64-L69) | Decide `actions_config.yaml` explicitly. | TASK-083 | `actions_config.yaml` is intentionally absent or backed by an approved schema/tests. |
| TASK-085 | PLAN-062-T085 | [implemented summary](./HJMT-REPORT.md#L48-L71), [proof gap](./HJMT-Sharding-Storage-Techpaper.md#L288-L299) | Close local HJMT proof/root/route model evidence. | TASK-071 | Local proof/root/route tests are cited; missing proof-layer work is separated. |
| TASK-086 | PLAN-062-T086 | [route table](./HJMT-REPORT.md#L211-L316), [routing gap](./HJMT-Sharding-Storage-Techpaper.md#L319-L330) | Verify route snapshot digest/golden vectors and tamper vectors. | TASK-085 | Route-table bytes/digests are deterministic and tamper tests fail closed. |
| TASK-087 | PLAN-062-T087 | [batch planning](./HJMT-REPORT.md#L271-L316), [routing gap](./HJMT-Sharding-Storage-Techpaper.md#L319-L330) | Keep single-shard batch guardrails and cross-shard rejection. | TASK-086 | Cross-shard batches reject unless a separate cross-shard protocol is implemented. |
| TASK-088 | PLAN-062-T088 | [transition/history gap](./HJMT-Sharding-Storage-Techpaper.md#L367-L377), [adaptive schedule](./HJMT-RAID%20-Sharding.md#L892-L910) | Close historical/adaptive transition proof safety. | TASK-085 | Historical proofs remain valid under historical route/policy/epoch/root generation. |
| TASK-089 | PLAN-062-T089 | [root publication](./HJMT-REPORT.md#L505-L583), [validator gate](./HJMT-Sharding-Storage-Techpaper.md#L352-L365) | Close checkpoint/root/history proof evidence. | TASK-085, TASK-086 | Publication route binding, prior-root continuity, and shard-leaf checks are evidenced. |
| TASK-090 | PLAN-062-T090 | [distributed omissions](./HJMT-REPORT.md#L65-L71), [sync limits](./HJMT-REPORT.md#L585-L605), [journal strategy](./HJMT-Sharding-Storage-Techpaper.md#L610-L623) | Implement local-simulator distributed journal replication across multiple aggregator states. | TASK-085 through TASK-089 | Replication uses real HJMT journal entries and storage recovery semantics; delay/drop/reorder/partition/replay cases are tested without claiming real network transport. |
| TASK-091 | PLAN-062-T091 | [root conflict limit](./HJMT-REPORT.md#L715-L715), [weak points](./HJMT-REPORT.md#L825-L827), [production target](./HJMT-Sharding-Storage-Techpaper.md#L591-L602) | Implement local-simulator quorum/root consensus for same-shard competing roots. | TASK-090 | Divergent same-shard roots resolve deterministically or freeze/fail closed with quorum evidence and dispute telemetry in local simulation. |
| TASK-092 | PLAN-062-T092 | [standby omissions](./HJMT-REPORT.md#L65-L68), [sync limits](./HJMT-REPORT.md#L599-L605), [failover gap](./HJMT-Sharding-Storage-Techpaper.md#L379-L388) | Implement local-simulator standby catch-up/state-transfer proof. | TASK-090 | Standby readiness is bound to latest journal lineage/version/root proof; stale, missing, and partially replayed standby states fail closed. |
| TASK-093 | PLAN-062-T093 | [route rollout weak point](./HJMT-REPORT.md#L825-L829), [rollout strategy](./HJMT-Sharding-Storage-Techpaper.md#L687-L718) | Implement local-simulator atomic live route-table rollout. | TASK-086, TASK-090 | Route activation requires checkpoint/process acknowledgement evidence; mixed-generation rollout, stale digest, and late-joiner cases fail closed. |
| TASK-094 | PLAN-062-T094 | [scheduler limitation](./HJMT-REPORT.md#L308-L316), [weak points](./HJMT-REPORT.md#L825-L834), [durability gap](./HJMT-Sharding-Storage-Techpaper.md#L333-L350) | Implement local-simulator scheduler waves over shard-owned aggregator workers. | TASK-087 | Scheduler waves drive multiple shard owners through planner/dispatch/journal publication; performance evidence is separated from durable-root-published TPS unless measured at root publication. |
| TASK-095 | PLAN-062-T095 | [remote dispatch absence](./HJMT-REPORT.md#L333-L335), [planner gaps](./HJMT-REPORT.md#L800-L819), [planner ownership](./HJMT-Sharding-Storage-Techpaper.md#L220-L270) | Implement local remote-dispatch simulation between independent aggregator workers/process models. | TASK-093, TASK-094 | Planner decisions are delivered to the owning aggregator through a local transport boundary; wrong owner, unavailable owner, duplicate delivery, reorder, and restart cases are tested. |
| TASK-096 | PLAN-062-T096 | [cross-shard weak point](./HJMT-REPORT.md#L830-L830), [routing gap](./HJMT-Sharding-Storage-Techpaper.md#L319-L330) | Enforce cross-shard rejection in local distributed simulation unless a real cross-shard transaction protocol is implemented. | TASK-087, TASK-095 | Cross-shard attempts through planner and remote dispatch fail closed; no implicit distributed transaction fallback exists. |
| TASK-097 | PLAN-062-T097 | [storage lock weak point](./HJMT-REPORT.md#L831-L831), [storage seams](./HJMT-Sharding-Storage-Techpaper.md#L414-L488) | Add explicit process/storage lock enforcement for local simulator and shared-root hazard cases. | TASK-090 | Concurrent writers, stale owners, duplicate processes, and shared-root misconfiguration are rejected by code/tests, not config comments only. |
| TASK-098 | PLAN-062-T098 | [weak points](./HJMT-REPORT.md#L825-L834), [observability gap](./HJMT-Sharding-Storage-Techpaper.md#L394-L404) | Add simulator and production observability for shard stalls, freeze, disputes, drift, and failover state. | TASK-090 through TASK-097 | Observability records every simulator fault and decision path but does not become proof/consensus truth. |
| TASK-099 | PLAN-062-T099 | [tech options](./HJMT-Sharding-Storage-Techpaper.md#L566-L624), [OpenRaft notes](./HJMT-RAID%20-Sharding.md#L1352-L1372) | Implement a local consensus-adapter seam behind journal replication with deterministic quorum/term/log semantics. | TASK-090, TASK-091 | Local consensus behavior is exercised behind journal seams with real root/journal inputs; external OpenRaft/network binding is adapter-only and cannot replace simulated consensus tests. |
| TASK-100 | PLAN-062-T100 | [membership warning](./HJMT-Sharding-Storage-Techpaper.md#L626-L633), [do not build](./HJMT-RAID%20-Sharding.md#L1512-L1523) | Add local-simulator membership/change-management tests for distributed HJMT. | TASK-099 | Join, leave, decommission, rejoin, stale member, and generation-bound membership changes are tested through the local substrate. |
| TASK-101 | PLAN-062-T101 | [split-brain behavior](./HJMT-REPORT.md#L653-L715), [simulation evidence](./HJMT-Sharding-Storage-Techpaper.md#L644-L685) | Add standby unavailable/catch-up/failover negative tests. | TASK-092, TASK-093 | Wrong-lineage, stale, standby-down, and unavailable paths fail closed. |
| TASK-102 | PLAN-062-T102 | [route checker](./HJMT-REPORT.md#L557-L583), [verification run](./HJMT-REPORT.md#L877-L929) | Collect validator/watcher route-binding rollup evidence. | TASK-089 | Validator/watcher/shared checker evidence is cited and missing negatives are added. |
| TASK-103 | PLAN-062-T103 | [bottleneck](./HJMT-REPORT.md#L29-L30), [weak point](./HJMT-REPORT.md#L834-L834), [durability gap](./HJMT-Sharding-Storage-Techpaper.md#L333-L350) | Add benchmark acceptance thresholds or mark benchmark lanes informational. | TASK-010, TASK-011 | Durable-root-published TPS, journal sync, latency, blocked time, RSS, and CPU are separated. |
| TASK-104 | PLAN-062-T104 | [wallet boundary](./HJMT-RAID%20-Sharding.md#L1699-L1740), [storage boundary](./HJMT-Sharding-Storage-Techpaper.md#L414-L488) | Close wallet-storage and aggregator-RedB boundary docs. | TASK-085 | Wallet sees public proofs/API only; aggregator does not know raw RedB internals. |
| TASK-105 | PLAN-062-T105 | [bottom line](./HJMT-REPORT.md#L931-L949), [rollout strategy](./HJMT-Sharding-Storage-Techpaper.md#L687-L730) | Create adapter-only register after local distributed simulation is implemented. | TASK-090 through TASK-104 | Register is limited to external transport/chain-network deployment adapters; local replication, quorum, catch-up, rollout, dispatch, consensus, and failure semantics are implemented/tested in Phase 062. |
| TASK-106 | PLAN-062-T106 | [implementation surfaces](./Z00Z-Thin-Transaction-Mode.md#L812-L820), [signature alignment](./Z00Z-Thin-Transaction-Mode.md#L937-L959), [guardrails](./Z00Z-Thin-Transaction-Mode.md#L990-L1002) | Add public thin transaction DTO/wrapper. | TASK-014 through TASK-021 | Thin wrapper preserves canonical package semantics and is backed by real digest/signature/root checks. |
| TASK-107 | PLAN-062-T107 | [signed index model](./Z00Z-Thin-Transaction-Mode.md#L312-L368), [glossary](./Z00Z-Thin-Transaction-Mode.md#L890-L905) | Implement signed index entry/snapshot model. | TASK-106 | Snapshot entry/context/authentication model is explicit, non-authoritative, signed, digest-pinned, and checkpoint-bound. |
| TASK-108 | PLAN-062-T108 | [authentication/refresh](./Z00Z-Thin-Transaction-Mode.md#L346-L390), [failure modes](./Z00Z-Thin-Transaction-Mode.md#L674-L714) | Implement signature, checkpoint context, expiry, and equivocation checks. | TASK-107 | Stale/wrong/equivocated/withheld snapshots fail closed. |
| TASK-109 | PLAN-062-T109 | [publication refresh](./Z00Z-Thin-Transaction-Mode.md#L375-L396), [fallback rules](./Z00Z-Thin-Transaction-Mode.md#L623-L642) | Implement wallet cache pin/refresh/fallback behavior. | TASK-107, TASK-108 | Cache uncertainty defaults to thick mode. |
| TASK-110 | PLAN-062-T110 | [expansion duties](./Z00Z-Thin-Transaction-Mode.md#L491-L510), [runtime vocabulary](./Z00Z-Thin-Transaction-Mode.md#L532-L564) | Implement helper expansion before normalized runtime admission. | TASK-106 through TASK-109 | Thin expansion collapses into existing runtime vocabulary before publication. |
| TASK-111 | PLAN-062-T111 | [signed-index APIs](./Z00Z-Thin-Transaction-Mode.md#L825-L832), [absorbed inputs](./Z00Z-Thin-Transaction-Mode.md#L912-L930) | Implement helper index storage and APIs. | TASK-107, TASK-108 | Snapshot fetch, digest pinning, refresh, and typed stale/missing/conflict errors exist. |
| TASK-112 | PLAN-062-T112 | [semantic equivalence](./Z00Z-Thin-Transaction-Mode.md#L515-L532), [test classes](./Z00Z-Thin-Transaction-Mode.md#L847-L859) | Share thick/thin builder semantics. | TASK-106, TASK-110 | Thin and thick modes construct the same transaction meaning. |
| TASK-113 | PLAN-062-T113 | [runtime vocabulary](./Z00Z-Thin-Transaction-Mode.md#L532-L564), [concept guardrails](./Z00Z-Thin-Transaction-Mode.md#L990-L1002) | Guard against thin mode changing settlement/publication semantics. | TASK-106 through TASK-112 | No downstream `ThinWorkItem`, thin verdict, or thin settlement theorem is introduced. |
| TASK-114 | PLAN-062-T114 | [fallback rules](./Z00Z-Thin-Transaction-Mode.md#L623-L642), [cache management](./Z00Z-Thin-Transaction-Mode.md#L840-L841), [tests](./Z00Z-Thin-Transaction-Mode.md#L847-L859) | Add restart/cache/default-thick tests. | TASK-109 | Restart/restore/cache corruption defaults to thick until authenticated snapshot is pinned. |
| TASK-115 | PLAN-062-T115 | [verification requirement](./Z00Z-Thin-Transaction-Mode.md#L515-L532), [test classes](./Z00Z-Thin-Transaction-Mode.md#L847-L849) | Add thin/thick semantic equivalence tests. | TASK-112 | Both modes reach the same checkpoint-facing result. |
| TASK-116 | PLAN-062-T116 | [failure modes](./Z00Z-Thin-Transaction-Mode.md#L674-L714), [test classes](./Z00Z-Thin-Transaction-Mode.md#L847-L854) | Add stale, wrong, equivocated, withheld, and expired index negative tests. | TASK-108 | All wrong-index classes fail closed with typed errors. |
| TASK-117 | PLAN-062-T117 | [thick fallback](./Z00Z-Thin-Transaction-Mode.md#L599-L642), [fallback tests](./Z00Z-Thin-Transaction-Mode.md#L854-L859) | Add fallback/recovery tests from thin failure to thick mode. | TASK-109, TASK-116 | Wallet can switch helper or resubmit thick without changing meaning. |
| TASK-118 | PLAN-062-T118 | [privacy risks](./Z00Z-Thin-Transaction-Mode.md#L717-L740), [test classes](./Z00Z-Thin-Transaction-Mode.md#L847-L859) | Add privacy/logging/metadata tests for thin index surfaces. | TASK-111 | Helper metadata leakage is bounded and documented. |
| TASK-119 | PLAN-062-T119 | [Appendix C](./Z00Z-Thin-Transaction-Mode.md#L937-L982), [guardrails](./Z00Z-Thin-Transaction-Mode.md#L990-L1002) | Resolve Appendix C root-name/signature drift against current code before implementation. | TASK-106 | Thin docs use current live root/package names and do not create a second authority. |
| TASK-120 | PLAN-062-T120 | [HJMT structure note](./HJMT-структуры.md#L1-L32), [runtime/storage ownership map](./HJMT-Sharding-Storage-Techpaper.md#L84-L92), [do not move runtime planning into storage](./HJMT-Sharding-Storage-Techpaper.md#L135-L153), [planner ownership](./HJMT-Sharding-Storage-Techpaper.md#L220-L237), [config-level delivery limit](./HJMT-REPORT.md#L819-L819) | Close HJMT structure and runtime-fixture ownership: storage-created scopes versus aggregator orchestration, and `config/hjmt_runtime` as runtime home rather than storage-owned semantic truth. | TASK-085, TASK-104 | Storage-owned scope creation is documented; `config/hjmt_runtime` is either retained as repo-level runtime fixture or moved under runtime/orchestration ownership, while only storage backend schema/default fragments are eligible to move into storage; any aggregator orchestration feature is separately scoped. |
| TASK-121 | PLAN-062-T121 | [local full-system simulation register](#local-full-system-simulation-closure-register) | Implement wallet `ChainClient` node RPC behavior against local node simulation and adapter seams. | TASK-014 through TASK-021 | Tip/block/header/submit/status/network-info paths work against local simulated node state plus integration-gated real-node adapter tests when available. |
| TASK-122 | PLAN-062-T122 | [local full-system simulation register](#local-full-system-simulation-closure-register) | Implement broadcast submission, retry, confirmation polling, and tx-store integration. | TASK-121 | Broadcast retry/polling persists lifecycle updates and handles timeout, reject, duplicate, reorg/replacement, and confirmation paths. |
| TASK-123 | PLAN-062-T123 | [local full-system simulation register](#local-full-system-simulation-closure-register) | Wire fee-rate source into fee estimation through local node/network simulation. | TASK-121 | Fee estimator uses simulated live source with cache/fallback/stale/zero/spike tests; external network source remains adapter-only. |
| TASK-124 | PLAN-062-T124 | [local full-system simulation register](#local-full-system-simulation-closure-register) | Implement remote scan worker against local chain/node simulation. | TASK-026 through TASK-031, TASK-121 | Remote worker has trust-boundary/no-mutation/restart/stale/malicious-worker tests and cannot replace authoritative local receive verification. |
| TASK-125 | PLAN-062-T125 | [local full-system simulation register](#local-full-system-simulation-closure-register) | Enforce wallet policy daily-spend and confirmation guarantees for simulated/live spending flows. | TASK-049 through TASK-062, TASK-121 | Policy tests enforce daily limits, confirmation requirements, restart persistence, multi-send aggregation, and rejection surfaces. |

## Local Full-System Simulation Closure Register

These five rows are outside the strict Phase 062 source corpus but mandatory for yolo/full-system closure. They MUST be implemented through local node/network simulation with real wallet/storage/crypto/policy primitives; only unavailable external network adapters remain non-live.

| Task id | Code evidence | Required closure |
| --- | --- | --- |
| TASK-121 | [`chain_client_impl.rs`](../../../crates/z00z_wallets/src/chain/chain_client_impl.rs#L19-L99) now routes tip/block/header/submit/status/network-info calls through `LocalNodeSim` or an explicit remote adapter seam. | Preserve local node RPC simulation as the live wallet path and keep only real remote transport adapter-only. |
| TASK-122 | [`broadcast_impl.rs`](../../../crates/z00z_wallets/src/chain/broadcast_impl.rs#L15-L27), [`broadcast_impl.rs`](../../../crates/z00z_wallets/src/chain/broadcast_impl.rs#L106-L160) now own durable submit/retry/confirm persistence on the wallet chain/tx-store seam. | Preserve retry, timeout, reject, duplicate, reorg/replacement, and confirmation coverage on the same durable lifecycle path. |
| TASK-123 | [`fee_estimator.rs`](../../../crates/z00z_wallets/src/tx/fee_estimator.rs#L139-L158), [`local_node_sim.rs`](../../../crates/z00z_wallets/src/chain/local_node_sim.rs#L28-L37) now provide simulated-live fee-rate sourcing with cache/fallback seams and an adapter-only remote source boundary. | Preserve cache/fallback/stale/zero/spike fee-source behavior on the live estimator path without introducing a second authority plane. |
| TASK-124 | [`scan_engine_impl.rs`](../../../crates/z00z_wallets/src/chain/scan_engine_impl.rs#L1-L31), [`wallet_actions_receive.rs`](../../../crates/z00z_wallets/src/services/wallet_actions_receive.rs#L443-L492) now keep the remote worker subordinate to authoritative wallet-local receive verification. | Preserve restart, stale, malicious, and no-mutation worker behavior through the local node simulation and the authoritative receive lane. |
| TASK-125 | [`policy.rs`](../../../crates/z00z_wallets/src/wallet/policy.rs#L85-L129), [`tx_storage_impl.rs`](../../../crates/z00z_wallets/src/persistence/tx_storage_impl.rs#L275-L343), [`tx_rpc_support.rs`](../../../crates/z00z_wallets/src/rpc/tx_rpc_support.rs#L55-L105) now derive spend context from canonical tx history and enforce daily-limit/confirmation gates on the live RPC path. | Preserve definition-id keyed spend aggregation, restart persistence, and typed daily-limit/confirmation failures for simulated/live spending flows. |

## Current Code Evidence Anchors

These anchors explain why several tasks are classified as missing or partial.

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

## GSD Plan Generation Contract

Every grouped generated plan MUST copy its row from `Required GSD Plan Groups`, copy the included task rows from `Canonical Task Inventory`, and pass the `Requirement Gate Contract` plus `Artifact/Test/Result Proof Contract`.

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

Before moving this audit into PLANS generation:

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
