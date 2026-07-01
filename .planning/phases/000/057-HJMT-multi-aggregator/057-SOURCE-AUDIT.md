# Phase 057: Source Audit

**Date:** 2026-06-13
**Status:** review-hardened planning, closeout, and renormalized continuation coverage map

## 🎯 Purpose

This document proves that the full `057-TODO.md` planning authority is routed
into the Phase 057 planning packet without creating a duplicate design layer.
It is evidence for review and doublecheck, not a replacement for
`057-CONTEXT.md` or the numbered plans.

## 🔑 Coverage Summary

| Source authority | Packet destination | Status |
| --- | --- | --- |
| `057-TODO.md` boundary language | `057-CONTEXT.md` domain + decisions | Covered |
| `057-TODO.md` workstreams | `057-01` through `057-06` | Covered |
| `057-TODO.md` gates `057-G1`..`057-G11` | `057-CONTEXT.md` gate map + numbered plans | Covered |
| `057-TODO.md` tests, benches, execution profiles, artifacts, fixtures, exit criteria | `057-06-PLAN.md` plus dependent slices and the test packet | Covered |
| Upgrade, fixture, and design-doc corpus named by `057-TODO.md` | `057-CONTEXT.md` cross-read contracts + plan coverage contracts | Covered |
| No-duplicate-layer and no-concept-drift rule | `057-CONTEXT.md` decisions D-02 through D-04 and D-12 through D-19 | Covered |
| Runtime on/off matrix join-or-leave blocker row inherited from Phase 056 | `057-03`, `057-06`, and closeout evidence | Covered |

## 🧭 Live Path Corrections Used By This Packet

| Contract or concern in TODO | Verified live anchor | Planning rule |
| --- | --- | --- |
| Runtime publication seam | `crates/z00z_runtime/aggregators/src/service.rs`, `types.rs`, `recovery.rs` | Extend the runtime-owned publication handoff in place; do not invent a second publication authority. |
| Route lineage and placement lineage | `crates/z00z_runtime/aggregators/src/batch_planner.rs`, `placement.rs`, `scheduler.rs`, `shard_exec.rs` | Publication consumes this lineage; it must not replace it. |
| Committed shard roots and proof truth | `crates/z00z_storage/src/settlement/hjmt_commit.rs`, `hjmt_store.rs`, `hjmt_proof.rs`, `proof_batch.rs`, `proof_batch_verify.rs` | Keep committed shard roots and proof truth storage-owned. |
| Publication handoff validation | `crates/z00z_rollup_node/src/config.rs` | Reuse the existing shard coverage, generation, route-digest, and ordering validation anchor. |
| Runtime status projection | `crates/z00z_rollup_node/src/runtime.rs`, `status.rs` | Extend current publication/status surfaces instead of adding a second reporting root. |
| Validator acceptance | `crates/z00z_runtime/validators/src/checkpoint.rs`, `engine.rs`, `verdict.rs` | Validators accept the canonical publication contract only. |
| Watcher evidence export | `crates/z00z_runtime/watchers/src/publication.rs`, `evidence_export.rs`, `engine.rs`, `status.rs` | Watchers export and alert on the same publication digest; no derived local truth. |
| Live simulator config | `crates/z00z_simulator/src/scenario_1/scenario_config.yaml` | Remains the executable scenario-config anchor. |
| Live simulator design sync | `crates/z00z_simulator/src/scenario_1/scenario_design.yaml` | Must change in the same slice as scenario-stage or publication-flow drift. |
| Live runtime observability | `crates/z00z_simulator/src/scenario_1/runtime_observability.rs` | Extend the existing Phase 056 trace packet with publication-layer evidence. |
| Current live storage bench homes | `crates/z00z_storage/benches/settlement_hjmt.rs`, `settlement_shard.rs`, `settlement_nested.rs`, `settlement_proofs.rs` | Extend existing bench homes or direct successors only; do not invent a second benchmark harness. |
| Contract names `aggregator-config.yaml`, `planner-config.yaml`, `storage-config.yaml` | checked-in runtime home under `config/hjmt_runtime/` | Use the live runtime home and keep exact publication additions path-honest. |
| Contract names `leaf_flow.json`, `proof_flow.json`, `pub_flow.json`, `val_flow.json`, `watch_flow.json` | `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`, `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`, `crates/z00z_simulator/tests/test_scenario_settlement.rs`, `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` | Keep the trace filenames and lineage semantics bound to the live simulator observability path and its acceptance suites. |
| Contract name `test_hjmt_multi_aggregator_sim.rs` | `crates/z00z_simulator/tests/test_scenario_settlement.rs` and `test_scenario1_stage_surface.rs` | Extend the current simulator acceptance lanes first instead of assuming a new test home already exists. |

## ✅ TODO Section Transfer Matrix

| TODO section | Reflected in context | Reflected in plans |
| --- | --- | --- |
| `Mission` | Yes | `057-01`..`057-06` |
| `This phase owns` | Yes | `057-01`..`057-06` |
| `This phase does not own` | Yes | `057-01`, `057-03`, `057-05`, `057-06` |
| `Phase handoff` | Yes | `057-03`, `057-04`, `057-05`, `057-06` |
| `Global upgrade rules active in every HJMT phase` | Yes | all numbered plans |
| `Inputs inherited from Phase 056 and consumed here` | Yes | `057-01`..`057-06` |
| `Primary upgrade sections owned by Phase 057` | Yes | `057-01`..`057-06` |
| `Required cross-read sections` | Yes | all numbered plans |
| `Fixture ownership for this phase` | Yes | `057-01`, `057-02`, `057-04`, `057-06` |
| `Embedded audit contract` | Yes | `057-01`, `057-02`, `057-03`, `057-04`, `057-05`, `057-06` |
| `Canonical SIM-5A7S-PUB publication profile` | Yes | `057-03`, `057-04`, `057-05`, `057-06` |
| `Release blockers owned or co-owned by Phase 057` | Yes | `057-01`..`057-06` |
| `Phase-owned acceptance subset` | Yes | `057-01`..`057-06` |
| `Mandatory implementation gates` | Yes | gate-owned plans |
| `Workstream 1` | Yes | `057-01` |
| `Workstream 2` | Yes | `057-02` |
| `Workstream 3` | Yes | `057-04` |
| `Workstream 4` | Yes | `057-05` |
| `Workstream 5` | Yes | `057-03`, `057-05` |
| `Required test coverage` | Yes | `057-06` |
| `Required benchmark slices` | Yes | `057-06` |
| `Required execution profiles` | Yes | `057-03`, `057-05`, `057-06` |
| `Required scenario coverage` | Yes | `057-01`..`057-06` |
| `Required artifacts` | Yes | `057-03`, `057-04`, `057-05`, `057-06` |
| `Fixture ownership` | Yes | `057-01`, `057-02`, `057-04`, `057-06` |
| `Exit criteria` | Yes | `057-06` |

## 🔒 Gate And Fixture Routing

| Gate or fixture class | Primary plan owner | Why |
| --- | --- | --- |
| `057-G1`, `057-G2`, and `057-G3` | `057-01` | Root-generation and both canonical publication objects must freeze together before later slices are honest. |
| `057-G4` | `057-02` | Layered public proof composition and compatibility belong to the proof-focused slice. |
| `057-G5` | `057-03` | The real Phase 056 `5x7` runtime lineage becomes the `SIM-5A7S-PUB` public checkpoint lane here. |
| `057-G6`, `057-G7`, and `057-G8` | `057-04` | Join, route-generation transfer, carry-forward, and crash containment are one transition packet. |
| `057-G9`, `057-G10`, and `057-G11` | `057-05` | Downstream contract sameness, scope continuity, and lineage-trace honesty close together. |
| `SRL-G-*` and `SRL-T-*` | `057-01` | Leaf bytes and tamper rejects are foundational fixture work. |
| `CPP-G-*` and `CPP-T-*` | `057-01` | Checkpoint bytes, ordering, monotonicity, and prior-root linkage are frozen with the publication object contract. |
| `FOV-G-002`..`FOV-G-004` | `057-04` and `057-06` | Carry-forward, partial failure, and route-migration crash rows land in the transition slice and close in the matrix slice. |
| Upgrade `12.1` fixture classes | `057-01`..`057-06`, closed by `057-06` | The gap set spans root generations, publication evidence, join, transfer, and continuity closeout. |

## 🧪 Doublecheck Coverage Claims To Re-Verify After Plan Creation

1. Every major `057-TODO.md` section is mapped to `057-CONTEXT.md` and at
   least one numbered plan.
2. No numbered plan claims a live repo path for `leaf_flow.json`,
   `proof_flow.json`, `pub_flow.json`, `val_flow.json`, or `watch_flow.json`
   before execution verifies the exact homes.
3. No numbered plan duplicates runtime route truth, storage proof truth, or
   watcher or validator publication truth.
4. Gates `057-G1` through `057-G11` each have one primary owner plan and one
   explicit closeout path.
5. The packet preserves the user's mandatory verify order and the required
   `/z00z-git-versioning` commit rule.

## 📋 Literal Bullet Ledger

This ledger exists because section-level coverage is not enough for a strict
publication-phase review. Each bullet class below must stay explicit in both
context and the numbered packet.

| TODO bullet class | Context anchor | Plan anchor |
| --- | --- | --- |
| `ShardRootLeafV1`, `CheckpointPublicationV1`, root-generation transitions, carry-forward, join-as-standby, join-as-owner, transfer continuity, validator/watcher acceptance | `057-CONTEXT.md` phase boundary + decisions | `057-01`..`057-06` |
| `SIM-5A7S-PUB` seven-leaf ordered publication and topology-generic rule | `057-CONTEXT.md` decisions D-07 and D-16 | `057-03`, `057-06` |
| Separate join-as-standby and join-as-owner evidence packets | `057-CONTEXT.md` decision D-08 | `057-04`, `057-06` |
| Route-generation-bound transfer and activation checkpoint | `057-CONTEXT.md` decision D-09 | `057-04`, `057-06` |
| Byte-identical carry-forward and no silent reroute | `057-CONTEXT.md` decisions D-10 and D-17 | `057-04`, `057-06` |
| Dynamic-scope publication continuity without extra registry | `057-CONTEXT.md` decision D-11 | `057-05`, `057-06` |
| Trace joins from `leaf_flow.json`..`watch_flow.json` back to Phase 056 `cfg_flow.json`, `tx_flow.json`, `route_flow.json`, `plan_flow.json`, `journal_flow.json`, `scope_flow.json`, `proc_flow.json`, and `recovery_flow.json` lineage | `057-CONTEXT.md` decisions D-12 and D-13 | `057-03`, `057-05`, `057-06` |
| YAML-defined old/new topology, route-generation boundaries, join mode, transfer target, owner/standby roles, and publication activation point | `057-CONTEXT.md` literal bullet preservation map | `057-03`, `057-05`, `057-06` |
| Config digests, process ids, journal paths, owner/standby assignments, process exit/restart verdicts, and inherited runtime on/off matrix rows | `057-CONTEXT.md` literal bullet preservation map | `057-03`, `057-04`, `057-05`, `057-06` |
| TODO contract names stay anchored to verified live homes where execution landed them, unresolved names stay proposed, and the four current storage bench homes remain the live benchmark anchors | `057-CONTEXT.md` decisions D-14 and D-19 | `057-03`, `057-06`, `057-07` |
| Required tests `test_hjmt_root_generation.rs`, `test_hjmt_historical_proofs.rs`, `test_hjmt_join.rs`, `test_hjmt_publish.rs`, `test_hjmt_migrate.rs`, simulator continuation | `057-CONTEXT.md` literal bullet preservation map | `057-01`..`057-06`, closed by `057-06` |
| Execution profiles `SIM-SMALL`, `SIM-MEDIUM`, `SIM-CACHE-EDGE`, reserved-only `SIM-BATCH-1000` | `057-CONTEXT.md` literal bullet preservation map | `057-03`, `057-05`, `057-06` |
| Publication artifacts, config and process evidence, and publication trace artifacts | `057-CONTEXT.md` literal bullet preservation map | `057-03`, `057-04`, `057-05`, `057-06` |
| Fixture ids `SRL-*`, `CPP-*`, `FOV-G-002`..`FOV-G-004`, and Upgrade `12.1` owned gaps | `057-CONTEXT.md` literal bullet preservation map | `057-01`, `057-04`, `057-06` |
| Exit-criteria literals for executable contracts, canonical seven-leaf publication, distinct join states, committed route generations, exact carry-forward, scenario/design sync, and no detached publication artifacts | `057-CONTEXT.md` literal bullet preservation map | `057-01`, `057-03`, `057-04`, `057-05`, `057-06` |

## 🔍 Second Doublecheck Result

Second workspace-first doublecheck against `057-TODO.md` confirms the
canonical Phase 057 execution packet is the six-plan core set `057-01-PLAN.md`
through `057-06-PLAN.md` plus the explicit `057-07-PLAN.md` continuation that
renormalizes the old superseded draft into a live closeout-guardrail slice.
`057-TEST-SPEC.md` and `057-TESTS-TASKS.md` stay as cross-cutting planning
artifacts, `057-06-SUMMARY.md` closes the original evidence matrix, and
`057-07-SUMMARY.md` closes the post-closeout authority-drift and ledger-honesty
continuation without creating a parallel execution packet. The corrected
packet keeps the predecessor Phase 056 anchors on their real workspace paths,
keeps the design-doc cross-reads active across every plan, keeps the co-owned
runtime on/off matrix row explicit, keeps YAML topology-field ownership
explicit, keeps the full Phase 056 lineage packet explicit from
`cfg_flow.json` through `recovery_flow.json`, keeps the exit-criteria literals
explicit in the numbered packet instead of implication-only coverage, and
keeps Phase 058 readiness or release judgment outside the Phase 057 closeout
slice.
