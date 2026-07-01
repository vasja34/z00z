---
phase: 056-HJMT-storage-aggregator
artifact: tests-tasks
status: execution-backed
source:
  - 056-TEST-SPEC.md
  - 056-TODO.md
  - 056-CONTEXT.md
  - 056-01-PLAN.md
  - 056-02-PLAN.md
  - 056-03-PLAN.md
  - 056-04-PLAN.md
  - 056-05-PLAN.md
  - 056-06-PLAN.md
  - 056-07-PLAN.md
updated: 2026-06-12
---

# Phase 056 Test Tasks

**Phase:** `056-HJMT-storage-aggregator`
**Status:** execution-backed planning artifact
**Companion spec:** `056-TEST-SPEC.md`

## 🎯 Goal

This file captured the implementation order for the full Phase 056 test
packet. Phase 056 is now implemented, so this document remains the historical
owner-home map, review baseline, and regression checklist for follow-up work
on the same HJMT runtime surface.

The packet was executed without guessing:

- which live file home owns each scenario;
- which fixtures and traces are required;
- which assertions prove correctness;
- which failure rows must reject;
- which execution profiles and bench lanes must exist;
- which anti-drift checks must block duplicate authority layers.

## ⚙️ Historical Execution Order

1. Land topology and process tests.
2. Land route-table and planner-equivalence tests.
3. Land semantic handoff and scope-birth tests.
4. Land failover, restart, and persistence tests.
5. Land startup preflight and config-drift tests.
6. Extend simulator evidence tests.
7. Extend bench and guardrail closeout tests.

## 📋 Task Ledger

Live completion evidence for these rows now lives in
`056-01-SUMMARY.md` through `056-07-SUMMARY.md`, while this ledger preserves
the executed work order and acceptance targets.

| Task ID | Scenario IDs | File Homes | Action | Done When |
| --- | --- | --- | --- | --- |
| `056-TT-01` | `056-SC-01` | `crates/z00z_rollup_node/tests/test_hjmt_topology.rs` | Create the canonical `SIM-5A7S` topology test and one additional positive non-`5x7` topology test. Assert five aggregators, seven shards, one dual-primary owner, standby coverage, route coverage, and topology-generic acceptance. | `SIM-5A7S` and a second positive topology pass; zero or inconsistent topologies reject. |
| `056-TT-02` | `056-SC-02` | `crates/z00z_rollup_node/tests/test_hjmt_process.rs`, `crates/z00z_rollup_node/tests/test_hjmt_node_lifecycle.rs` | Create process-isolation and lifecycle tests. Assert separate PID, port, data dir, journal path, log path, startup command, kill, and restart behavior. | Canonical runtime path is proven process-based and independently controllable. |
| `056-TT-03` | `056-SC-03` | `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_route_table_v1/`, `crates/z00z_runtime/aggregators/tests/test_hjmt_shard_routing.rs` | Create route golden vectors `SRT-G-001`..`SRT-G-004`, tamper vectors `SRT-T-001`..`SRT-T-008`, and route codec tests. | Route bytes are canonical and digest-bound; tamper rows reject fail-closed. |
| `056-TT-04` | `056-SC-04` | `crates/z00z_runtime/aggregators/tests/test_hjmt_planner.rs` | Create planner-mode equivalence coverage for broad, hot-shard, hot-serial, delete-heavy, search-heavy, proof-heavy, mixed-scope, and cross-shard rows. | Central and per-aggregator planners agree on accepted digest and reject matrix. |
| `056-TT-05` | `056-SC-05` | `crates/z00z_storage/tests/test_hjmt_scope_birth.rs`, `crates/z00z_storage/tests/test_live_guardrails.rs` | Create semantic-handoff and scope-birth coverage. Assert first-seen `definition_id`, first-seen `serial_id`, first terminal/right creation, mixed scope batches, duplicate terminal reject, path/leaf mismatch reject, and `scope_flow.json` schema. | Runtime remains semantic-only and first-scope behavior is restart-safe. |
| `056-TT-06` | `056-SC-06`, `056-SC-07` | `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_split_brain_fencing.rs`, `crates/z00z_storage/src/settlement/test_live_recovery.rs`, `crates/z00z_runtime/aggregators/tests/fixtures/failover_v1/` | Create positive failover vector `FOV-001`, negative vectors `FOV-T-001`, `FOV-T-002`, and route-migration/crash rows. Extend storage recovery checks for first-scope replay, lineage, generation, and import/export roundtrip. | Same-lineage takeover is the only accepted path; every illegal row rejects with explicit class. |
| `056-TT-07` | `056-SC-08` | `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs` | Create startup reject-matrix tests for missing or inconsistent `aggregator-config.yaml`, `planner-config.yaml`, and `storage-config.yaml`, plus bad route digest, invalid placement, wrong lineage, unsupported backend generation, bad proof bytes, bad handoff metadata, and bad hash-domain tags. When supplied, include `scenario_config.yaml` as digest evidence only; structural scenario-config rejects remain owned by simulator coverage. | Startup fails closed before live work on every invalid node-owned row and emits config digests for all runtime-visible config surfaces passed into preflight. |
| `056-TT-08` | `056-SC-09` | `crates/z00z_simulator/tests/test_scenario_settlement.rs`, `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` | Extend simulator tests to assert Phase 056 trace pack, config-digest linkage, route-digest linkage, lineage linkage, design/runtime sync, and missing-trace rejects for `SIM-SMALL`, `SIM-MEDIUM`, and `SIM-CACHE-EDGE`. | Simulator proves the real runtime plane and stale or detached traces fail. |
| `056-TT-09` | `056-SC-10` | `crates/z00z_storage/tests/test_bench_lanes.rs`, `crates/z00z_storage/benches/settlement_shard.rs`, `crates/z00z_storage/benches/settlement_hjmt.rs` | Extend bench-lane and bench-home guards. Add shard-parallel commit, shard-scaling, and cache-edge logical lanes on existing bench homes only. | Bench evidence exists without a second bench harness. |
| `056-TT-10` | `056-SC-11` | `crates/z00z_runtime/aggregators/tests/test_live_guardrails.rs`, `crates/z00z_storage/tests/test_live_guardrails.rs`, `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` | Add anti-drift source-shape checks for no second planner authority, no second semantic storage authority, no shared WAL truth, no in-process canonical mesh, and no second simulator evidence lane. | Any duplicate-authority pattern causes guardrail failure. |

## 🧪 Required Fixture Tasks

| Fixture Task | Owner Home | Minimum Content |
| --- | --- | --- |
| Route golden vectors | `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_route_table_v1/` | Canonical bytes, digest, generation, placement summary, regeneration command, and expected verdict for `SRT-G-001`..`SRT-G-004`. |
| Route tamper vectors | same route fixture home | One exact mutation per fixture and explicit reject stage for `SRT-T-001`..`SRT-T-008`. |
| Failover vectors | `crates/z00z_runtime/aggregators/tests/fixtures/failover_v1/` | `FOV-001`, `FOV-T-001`, `FOV-T-002`, route-migration fixture, failover fixture, process map, lineage state, expected verdict, and evidence path. |
| Trace manifests | phase-owned runtime evidence outputs | Config digests, route digest, process ids, data dirs, journal paths, trace paths, and scenario profile id. |

## 🔍 Mandatory Review Loop

Every execution closeout for this packet must run
`/.github/prompts/gsd-review-tasks-execution.prompt.md`
(` /GSD-Review-Tasks-Execution `) in YOLO mode at least three times.

Do not stop the review loop until at least two consecutive runs report no
significant issues, and fix every material warning or finding before claiming
completion.

## ✅ Definition Of Done

Phase 056 test implementation is not done until all of the following are true:

1. Every gate `056-G1` through `056-G10` has at least one primary passing test
   home and one explicit evidence artifact.
2. `SIM-SMALL`, `SIM-MEDIUM`, and `SIM-CACHE-EDGE` are all proven by runtime
   or simulator coverage, and `SIM-BATCH-1000` remains reserved-only.
3. `aggregator-config.yaml`, `planner-config.yaml`, `storage-config.yaml`, and
   `scenario_config.yaml` are all loaded from disk and proven behavior-changing
   through explicit test assertions and config digests.
4. `SRT-G-*`, `SRT-T-*`, `FOV-*`, `Route migration fixture`, and
   `Failover fixture` all exist with regeneration instructions and exact
   expected verdicts.
5. `cfg_flow.json`, `tx_flow.json`, `route_flow.json`, `plan_flow.json`,
   `journal_flow.json`, `scope_flow.json`, `proc_flow.json`, and
   `recovery_flow.json` are all linked to one config-digest set, one route
   digest, one lineage view, and one process-topology view.
6. Guardrail tests fail if a second planner, second semantic storage lane,
   second simulator evidence lane, shared WAL truth layer, or in-process
   canonical mesh appears.
7. Bench-lane tests prove the shard lanes live in the existing storage bench
   homes only.
