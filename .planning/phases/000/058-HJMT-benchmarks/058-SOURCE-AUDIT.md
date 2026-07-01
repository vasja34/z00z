# Phase 058: Source Audit

**Date:** 2026-06-16
**Status:** execution-backed closeout audit after the final integrated-upgrade refresh

## 🎯 Purpose

This document proves that the full `058-TODO.md` authority is routed into the
Phase 058 execution packet without turning TODO contract names into false live
repository facts.

It is evidence for review and doublecheck, not a replacement for
`058-CONTEXT.md`, `058-EVIDENCE-LEDGER.md`, or the numbered plans.

## 🔑 Coverage Summary

| Source authority | Packet destination | Status |
| --- | --- | --- |
| `058-TODO.md` phase boundary and release-blocker language | `058-CONTEXT.md` domain + decisions | Covered |
| `058-TODO.md` workstreams | `058-01` through `058-07` | Covered |
| `058-TODO.md` claim-discipline, Appendix C, fixture-family, and unsupported rows | `058-EVIDENCE-LEDGER.md` | Covered |
| `058-TODO.md` gates `058-G1`..`058-G13` | `058-CONTEXT.md` gate map + numbered plans | Covered |
| `058-TODO.md` tests, benches, execution profiles, artifacts, fixtures, and exit criteria | `058-TEST-SPEC.md`, `058-TESTS-TASKS.md`, `058-05`..`058-07` | Covered |
| Upgrade, fixture, and design-doc corpus named by `058-TODO.md` | `058-CONTEXT.md` cross-read contract + plan coverage contracts | Covered |
| Live/proposed path honesty and no-duplicate-authority rule | `058-CONTEXT.md` decisions D-02 through D-17 | Covered |

## 🧭 Live Path Corrections Used By This Packet

| Contract or concern in TODO | Verified live anchor or status | Planning rule |
| --- | --- | --- |
| Aggregator YAML surface | `config/hjmt_runtime/sim_5a7s/aggregators/agg-*/aggregator-config.yaml` | Reuse the checked runtime home; do not invent a second config root. |
| Planner YAML surface | `config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml` | Planner-mode and route-cadence proof must land here or a documented successor. |
| Storage YAML surface | `config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml` | Startup, journal, and backend evidence stay on the checked runtime home. |
| Runtime fixture manifest | `config/hjmt_runtime/sim_5a7s/manifest.json` | Final `SIM-5A7S` and `SIM-5A7S-PUB` closure must stay bound to this home. |
| Live simulator config | `crates/z00z_simulator/src/scenario_1/scenario_config.yaml` | Remains the executable runtime-config anchor. |
| Live simulator design sync | `crates/z00z_simulator/src/scenario_1/scenario_design.yaml` | Must change in the same slice as stage or artifact drift. |
| Live runtime observability | `crates/z00z_simulator/src/scenario_1/runtime_observability.rs` | Extend the checked packet in place rather than inventing a second trace exporter. |
| Inherited trace pack `cfg_flow.json` through `watch_flow.json` | declared in `scenario_config.yaml`, emitted by `runtime_observability.rs`, rechecked by `test_hjmt_runtime_config.rs`, `test_scenario_settlement.rs`, and `test_scenario1_stage_surface.rs` | Keep the inherited lineage packet authoritative and extend it honestly. |
| `proc_flow.json` lineage row inherited from earlier phases | declared in `scenario_config.yaml` and validated by `test_scenario1_stage_surface.rs` | Preserve it even though `058-TODO.md` does not repeat it in every list. |
| `run_meta.json`, `wallet_scan.json`, `sim_summary.md` | `verified live` on the `scenario_1` public release packet via `scenario_config.yaml`, `runtime_observability.rs`, `test_scenario_settlement.rs`, `test_scenario1_stage_surface.rs`, and Stage 13 shared-case guards | Keep these exact homes canonical and do not reintroduce alternate packet paths. |
| `hist_flow.json`, `occ_flow.json` | `verified live` on the same `scenario_1` public release packet via `scenario_config.yaml`, `runtime_observability.rs`, `test_scenario_settlement.rs`, `test_scenario1_stage_surface.rs`, and `test_hjmt_e2e.rs` | Keep the emitted exact homes canonical and attached to the imported-artifact replay lane. |
| `asset_flow.json`, `right_flow.json` | `pending exact-home` inventory rows on the same `scenario_1` public release packet | Keep them explicit on the live lane without claiming emitted files before those exact homes land. |
| Live storage bench homes | `crates/z00z_storage/benches/settlement_proofs.rs`, `settlement_hjmt.rs`, `settlement_shard.rs`, `settlement_nested.rs`, `adaptive_policy_bench.rs` | Extend the current owner homes only. |
| Bench evidence contract | `crates/z00z_storage/benches/settlement_benches.md` | Keep one checked evidence ledger for measured runs. |
| TODO archive path `crates/z00z_storage/outputs/assets/` | retired wording; the canonical live evidence home is `crates/z00z_storage/outputs/settlement/` | Phase 058 closes the ambiguity by restating the final home honestly and keeping one canonical archive path. |
| `test_hjmt_batch_proof.rs` | `verified live` at `crates/z00z_storage/tests/test_hjmt_batch_proof.rs` | Reuse directly. |
| `test_hjmt_batch_proof_negative.rs` | `verified live` at `crates/z00z_storage/tests/test_hjmt_batch_proof_negative.rs` | Reuse directly. |
| `test_hjmt_root_generation.rs` | `verified live` at `crates/z00z_storage/tests/test_hjmt_root_generation.rs` | Reuse directly. |
| `test_hjmt_shard_routing.rs` | `verified live` at `crates/z00z_runtime/aggregators/tests/test_hjmt_shard_routing.rs` | Reuse directly. |
| `test_hjmt_topology.rs` | `verified live` at `crates/z00z_rollup_node/tests/test_hjmt_topology.rs` | Reuse directly. |
| `test_hjmt_process.rs` | `verified live` at `crates/z00z_rollup_node/tests/test_hjmt_process.rs` | Reuse directly. |
| `test_hjmt_planner.rs` | `verified live` at `crates/z00z_runtime/aggregators/tests/test_hjmt_planner.rs` | Reuse directly. |
| `test_hjmt_preflight.rs` | `verified live` at `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs` | Reuse directly. |
| `test_hjmt_failover_same_lineage.rs` | `verified live` at `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs` | Reuse directly. |
| `test_hjmt_split_brain_fencing.rs` | `verified live` at `crates/z00z_runtime/aggregators/tests/test_hjmt_split_brain_fencing.rs` | Reuse directly. |
| `test_hjmt_join.rs` | `verified live` at `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs` | Reuse directly. |
| `test_hjmt_publish.rs` | `verified live` at `crates/z00z_runtime/aggregators/tests/test_hjmt_publish.rs` | Reuse directly. |
| `test_hjmt_migrate.rs` | `verified live` at `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs` | Reuse directly. |
| `test_hjmt_historical_proofs.rs` | `verified live` at `crates/z00z_storage/tests/test_hjmt_historical_proofs.rs` | Reuse directly. |
| `test_hjmt_transition_proofs.rs` | `verified live` at `crates/z00z_storage/tests/test_hjmt_transition_proofs.rs` | Exact transition-proof acceptance home now freezes split or merge and policy-drift closure on the storage seam. |
| `test_hjmt_privacy_regression.rs` | `verified live` at `crates/z00z_storage/tests/test_hjmt_privacy_regression.rs` | Exact privacy-regression acceptance home now proves occupancy binding and tamper rejection on the storage seam. |
| `test_hjmt_e2e.rs` | `verified live` at `crates/z00z_simulator/tests/test_hjmt_e2e.rs` | Exact end-to-end acceptance home now stays on the release-simulator seam instead of inventing a storage-owned E2E path. |
| `test_hjmt_batch_commit.rs` | `verified live` at `crates/z00z_storage/tests/test_hjmt_batch_commit.rs` | Exact batch-commit acceptance home now proves root and oracle convergence on the storage seam. |
| `test_hjmt_batch_recovery.rs` | `verified live` at `crates/z00z_storage/tests/test_hjmt_batch_recovery.rs` | Exact batch-recovery acceptance home now proves injected recovery continuity without a second recovery authority path. |
| `test_hjmt_storage_boundary.rs` | `verified live` at `crates/z00z_storage/tests/test_hjmt_storage_boundary.rs` | Exact storage-boundary acceptance home is now frozen on the storage-owned seam. |
| `test_hjmt_backend_conformance.rs` | `verified live` at `crates/z00z_storage/tests/test_hjmt_backend_conformance.rs` | Exact backend-conformance acceptance home now proves the RedB baseline and startup-contract gate. |
| `test_hjmt_import_export.rs` | `verified live` at `crates/z00z_storage/tests/test_hjmt_import_export.rs` | Exact import/export readiness home now proves route, publication, proof, and recovery artifact roundtrips plus tamper rejects. |
| Validator/watcher publication acceptance | `verified live` at `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs` and `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs` | These live files close TODO rows even though the exact filenames differ from paper language. |
| Continuation of the multi-aggregator simulation lane | `verified live` at `crates/z00z_simulator/tests/test_scenario_settlement.rs` and `test_scenario1_stage_surface.rs` | Use the checked release-simulator lane instead of inventing a new `test_hjmt_multi_aggregator_sim.rs`. |

## ✅ TODO Section Transfer Matrix

| TODO section | Reflected in context | Reflected in plans |
| --- | --- | --- |
| `Mission` and `Phase Boundary` | Yes | `058-01`..`058-07` |
| `Source Map` | Yes | all numbered plans |
| `Embedded audit contract` | Yes | `058-01`..`058-07` |
| `Mandatory implementation gates` | Yes | gate-owned plans |
| `Workstream 1` | Yes | `058-01` |
| `Workstream 2` | Yes | `058-02` |
| `Workstream 3` | Yes | `058-05` |
| `Workstream 4` | Yes | `058-03` |
| `Workstream 5` | Yes | `058-06`, `058-07` |
| `Required tests and benchmarks` | Yes | `058-TEST-SPEC.md`, `058-TESTS-TASKS.md`, `058-05`..`058-07` |
| `Required execution profiles` | Yes | `058-02`, `058-04`, `058-05` |
| `Required scenario coverage` | Yes | `058-02`, `058-03`, `058-04`, `058-06`, `058-07` |
| `Required artifacts` | Yes | `058-01`, `058-02`, `058-04`, `058-05`, `058-06`, `058-07` |
| `Fixture closure` | Yes | `058-07` |
| `Exit criteria` | Yes | `058-07` |

## 🔒 Gate And Fixture Routing

| Gate or fixture class | Primary plan owner | Why |
| --- | --- | --- |
| `058-G1` | `058-01` | Claim discipline must freeze before any final evidence packet is honest. |
| `058-G2` and `058-G3` | `058-02` | Release-mode observability and doc sync are one simulator closure packet. |
| `058-G4`, `058-G9`, and `058-G10` | `058-03` | Config realism, import/export, restart, and startup reject logic are one operational-readiness slice. |
| `058-G5` and `058-G6` | `058-04` | Final runtime and publication packets must close together on the same release lane. |
| `058-G7`, `058-G8`, and `058-G12` | `058-05` | Heavy-profile discipline, benchmark matrix, and honest compression verdict are one measurement packet. |
| `058-G11` | `058-06` | Dynamic scope, wallet proof-before-ownership, and historical playback are one user-visible readiness packet. |
| `058-G13` and all fixture families | `058-07` | Final fixture closure and verdict classification aggregate every earlier slice. |

## 🧪 Doublecheck Claims To Re-Verify After Execution

1. No plan or summary claims a TODO-only artifact as already-live before the
   exact file exists.
2. No current authority file treats `outputs/assets/` as a checked evidence
   home; the closeout packet retires that wording and keeps
   `outputs/settlement/` canonical.
3. Every final score or readiness claim points back to one measured report and
   one verdict row in the evidence ledger.
4. Every fixture family and `12.1` evidence-gap class maps to one exact
   regeneration command and one evidence pointer.
5. The packet preserves the mandatory bootstrap-first validation order, the
   broad `cargo test --release` gate, and the `/z00z-git-versioning` commit
   rule.

## 🔍 Workspace-First Doublecheck Result

Workspace-first verification confirms that Phase 058 should execute as a
seven-slice packet over existing runtime, publication, simulator, bench, and
fixture seams rather than as a new architectural phase. The repository already
contains exact live anchors for route tables, shard leaves, checkpoint
publication, failover manifests, publication acceptance, the canonical `5x7`
runtime home, the release-simulator trace packet, and the current bench
harness. `058-EVIDENCE-LEDGER.md` now freezes the honest row-by-row status of
those anchors, including the current `outputs/settlement` bench home, the
partial `SRL` and `CPP` fixture families, the still-proposed
`batch_commit`/`batch_recovery` acceptance homes, and the exact Appendix C
artifacts that remain
checked, successor-live, or open.
