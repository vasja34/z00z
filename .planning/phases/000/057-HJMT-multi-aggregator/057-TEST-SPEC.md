---
phase: 057-HJMT-multi-aggregator
artifact: test-spec
status: execution-backed
source:
  - 057-TODO.md
  - 057-CONTEXT.md
  - 057-SOURCE-AUDIT.md
  - 057-01-PLAN.md
  - 057-02-PLAN.md
  - 057-03-PLAN.md
  - 057-04-PLAN.md
  - 057-05-PLAN.md
  - 057-06-PLAN.md
  - 057-07-PLAN.md
  - 057-07-SUMMARY.md
  - docs/tech-papers/Z00Z-HJMT-Upgrade.md
  - docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md
  - docs/tech-papers/Z00Z-HJMT-Design.md
updated: 2026-06-14
owner: Z00Z Runtime + Storage + Validators + Watchers + Simulator
scope: unit, integration, and Rust end-to-end coverage for the HJMT publication phase
---

<!-- markdownlint-disable MD060 -->

# Phase 057 Test Specification: HJMT Multi Aggregator

**Phase:** `057-HJMT-multi-aggregator`
**Status:** execution-backed
**Authority:** `057-TODO.md`, `057-CONTEXT.md`, `057-SOURCE-AUDIT.md`, the
numbered plans, the HJMT upgrade paper, the fixture checklist, and the live or
explicitly-proposed test homes listed below

## Purpose

This document defines the phase-local unit, integration, fixture, benchmark,
and Rust end-to-end test contract for Phase 057.

It is directly usable by another engineer or agent without guessing scenario
boundaries, state transitions, publication paths, proof paths, invariants,
success criteria, rejection rows, evidence artifacts, or test-file anchors.

For Phase 057, end-to-end means realistic Rust coverage across
`z00z_runtime/aggregators`, `z00z_storage`, `z00z_rollup_node`,
`z00z_runtime/validators`, `z00z_runtime/watchers`, and `z00z_simulator`. It
does not mean browser automation.

This packet is execution-backed. The tables below are the normative coverage
map and the current live execution ledger for review. Verified live owner homes
are marked explicitly; only unresolved contract names remain labeled
`proposed`.

## Workflow Status

- Mode: `execution-backed`.
- Source artifacts used:
  - `.planning/phases/057-HJMT-multi-aggregator/057-TODO.md`
  - `.planning/phases/057-HJMT-multi-aggregator/057-CONTEXT.md`
  - `.planning/phases/057-HJMT-multi-aggregator/057-SOURCE-AUDIT.md`
  - `.planning/phases/057-HJMT-multi-aggregator/057-01-PLAN.md`
  - `.planning/phases/057-HJMT-multi-aggregator/057-02-PLAN.md`
  - `.planning/phases/057-HJMT-multi-aggregator/057-03-PLAN.md`
  - `.planning/phases/057-HJMT-multi-aggregator/057-04-PLAN.md`
  - `.planning/phases/057-HJMT-multi-aggregator/057-05-PLAN.md`
  - `.planning/phases/057-HJMT-multi-aggregator/057-06-PLAN.md`
  - `.planning/phases/057-HJMT-multi-aggregator/057-07-PLAN.md`
  - `.planning/phases/057-HJMT-multi-aggregator/057-07-SUMMARY.md`
  - `docs/tech-papers/Z00Z-HJMT-Upgrade.md`
  - `docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md`
  - `docs/tech-papers/Z00Z-HJMT-Design.md`
- Testing posture:
  - Extend existing live seams first.
  - Add new test files only where they clarify publication authority
    boundaries or keep fixture families readable.
  - Keep route lineage and publication-request authority in
    `z00z_runtime/aggregators`.
  - Keep committed shard roots and proof truth in `z00z_storage`.
  - Keep checkpoint handoff validation in `z00z_rollup_node`.
  - Keep validator and watcher acceptance on their current crate seams.
  - Keep scenario, traces, and artifact verification in `z00z_simulator`.
  - Do not create a second planner, proof, publication, validator, watcher, or
    simulator authority layer in tests.

## Mandatory Source Cross-Read

Before implementing, reviewing, or summarizing any test from this packet,
read:

1. `057-CONTEXT.md` sections `Implementation Decisions`,
   `Cross-Crate Ownership Map`, `TODO Coverage Contract`, and
   `Literal bullet preservation map`.
2. `057-TODO.md` sections:
   `Canonical SIM-5A7S-PUB publication profile`,
   `Mandatory implementation gates`,
   `Workstream 1` through `Workstream 5`,
   `Required tests and benchmark slices`,
   `Required execution profiles`,
   `Required scenario coverage`,
   `Required artifacts`,
   `Fixture ownership`,
   and `Exit criteria`.
3. The exact `coverage_contract` section of every `057-0X-PLAN.md` that a test
   file closes.

## Non-Negotiable Test Rules

- Tests must call production route, publication, proof, validator, watcher,
  config, runner, and verification APIs. They must not prove the phase through
  mocks that bypass live owner seams.
- `SIM-5A7S-PUB` is an acceptance fixture only. Tests must also prove that
  publication topology remains generic for the inherited `SIM-5A7S` runtime
  packet and for any positive topology transition loaded from YAML when
  invariants hold, including any lawful `aggregator_count > 0`,
  `shard_count > 0`, and `old_aggregator_count -> new_aggregator_count`
  combination allowed by the TODO contract.
- `ShardRootLeafV1` and `CheckpointPublicationV1` must stay byte-authoritative
  and fail-closed under tampering.
- `policy_set_digest` semantics and the `policy_digest` naming rule from
  Upgrade `6.1.2` and `6.1.3` must stay explicit in leaf and publication
  assertions.
- Worked examples `6.8.1`, `6.8.2`, `6.8.4`, and `6.8.5` must be replayed as
  executable tests under the inherited 5x7 packet or an additional positive
  YAML topology where the TODO contract says topology-generic behavior is
  required.
- Join-as-standby and join-as-owner must remain distinct test states with
  distinct pre-activation and activation assertions.
- Carry-forward tests must prove byte-identical carried-forward leaves.
- Validators and watchers must accept the same digest and same verdict mapping.
- `leaf_flow.json`, `proof_flow.json`, `pub_flow.json`, `val_flow.json`, and
  `watch_flow.json` are evidence, not truth sources. Tests must link them back
  to Phase 056 `cfg_flow.json`, `tx_flow.json`, `route_flow.json`,
  `plan_flow.json`, `journal_flow.json`, `scope_flow.json`, `proc_flow.json`, and
  `recovery_flow.json`.
- Tests must keep config digests, process ids, journal paths, owner/standby
  assignments, process exit/restart verdicts, and inherited runtime on/off
  matrix join-or-leave rows in the same evidence packet as publication
  assertions.
- Transfer coverage must prove transfer to a remaining aggregator and transfer
  to a new aggregator as separate lawful route-generation-bound exemplars.
- The runtime YAML contract must stay explicit in tests:
  `aggregator-config.yaml`, `planner-config.yaml`, `storage-config.yaml`, and
  the live `scenario_config.yaml` surface must all be loaded from disk and
  must materially change publication behavior when their values change.
- Any YAML, fixture, or trace home not yet present in the repository must stay
  labeled as `proposed` in planning artifacts until execution lands it.

## Mandatory Verification Order

Every Rust or test-affecting change that touches this packet must verify in
this order:

```bash
./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_root_generation -- --nocapture
cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_publish -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_historical_proofs -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_live_proof_families -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof -- --nocapture
cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_stage8_proof_path -- --nocapture
cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_preflight -- --nocapture
cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_hjmt_runtime_config -- --nocapture
cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_join -- --nocapture
cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_migrate -- --nocapture
cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_failover_same_lineage -- --nocapture
cargo test -p z00z_validators --release
cargo test -p z00z_watchers --release
cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario_settlement -- --nocapture
cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario1_stage_surface -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_bench_lanes -- --nocapture
cargo bench -p z00z_storage --bench settlement_shard --no-run
cargo bench -p z00z_storage --bench settlement_hjmt --no-run
cargo test --release
```

If bootstrap fails, stop, fix the regression, and rerun bootstrap before any
broader validation.

Every execution slice must also run
`/.github/prompts/gsd-review-tasks-execution.prompt.md`
(` /GSD-Review-Tasks-Execution `) in YOLO mode at least three times and must
continue until at least two consecutive runs report no significant issues.

If a slice needs a versioning or release-flow commit, route it through
`/z00z-git-versioning`.

## Classification

### TDD And Integration Targets

| Seam | Class | Why It Matters |
| --- | --- | --- |
| `crates/z00z_storage/src/settlement/root_types.md`, `store.rs`, `hjmt_store.rs`, `hjmt_commit.rs` | unit / integration | Own the root-generation boundary, shard-leaf contract inputs, and committed shard-root lineage. |
| `crates/z00z_runtime/aggregators/src/service.rs`, `types.rs`, `recovery.rs` | integration | Own publication request assembly, checkpoint recording, and continuity state. |
| `crates/z00z_rollup_node/src/config.rs`, `runtime.rs`, `status.rs`, `da.rs` | integration | Own checkpoint handoff validation, composition-root wiring, and publication status projection. |
| `crates/z00z_storage/src/settlement/hjmt_proof.rs`, `proof.rs`, `proof_batch.rs`, `proof_batch_verify.rs` | integration | Own layered proof truth and historical compatibility inputs. |
| `crates/z00z_runtime/validators/src/checkpoint.rs`, `engine.rs`, `verdict.rs` | integration | Must accept only canonical publication truth. |
| `crates/z00z_runtime/watchers/src/publication.rs`, `evidence_export.rs`, `engine.rs`, `status.rs` | integration | Must export and alert on the same digest and verdict mapping that validators accept. |
| `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`, `runner.rs`, `runner_verify.rs` | integration / E2E | Must prove real publication traces, scenario sync, and lineage continuity on the checked-in runtime packet. |

### End-To-End Targets

| Home | Class | Why It Matters |
| --- | --- | --- |
| `crates/z00z_storage/tests/test_hjmt_root_generation.rs` | E2E / contract | Proves exact `ShardRootLeafV1` bytes, generation tags, and leaf tamper rejects. |
| `crates/z00z_runtime/aggregators/tests/test_hjmt_publish.rs` | E2E / checkpoint publication | Proves canonical ordered publication, prior-root continuity, route binding, and checkpoint tamper rejects. |
| `crates/z00z_storage/tests/test_hjmt_historical_proofs.rs` | E2E / proof continuity | Proves layered proof composition and historical compatibility rules. |
| `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs` | E2E / join | Proves standby-only join, owner activation after route generation `N+1`, and pre-activation reject rows. |
| `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs` | E2E / migration and carry-forward | Proves route-generation transfer, crash-recovery continuity, and byte-identical carry-forward rows. |
| `crates/z00z_simulator/tests/test_scenario_settlement.rs` and `test_scenario1_stage_surface.rs` | E2E / scenario and evidence | Prove `SIM-5A7S-PUB`, continuation of the Phase 056 multi-aggregator simulation lane, historical contract continuity with `test_hjmt_multi_aggregator_sim.rs`, publication trace-pack completeness, design sync, and YAML-driven topology transitions. |
| validator and watcher acceptance suites in `crates/z00z_runtime/validators/tests/` and `crates/z00z_runtime/watchers/tests/` | E2E / downstream acceptance | Prove shared publication digest, shared verdict mapping, and fail-closed downstream reject rows. |

### Skip Targets

| Item | Why It Is Skipped |
| --- | --- |
| `.planning/phases/057-HJMT-multi-aggregator/*.md` | Planning artifacts are inputs, not runtime test seams. |
| Any second publication digest lane outside the live runtime handoff path | Explicitly forbidden by the phase boundary. |
| Any second simulator, validator, or watcher evidence authority lane | Explicitly forbidden by the phase boundary. |
| Vendor code under `crates/z00z_crypto/tari/**` | Read-only vendor area. |
| Final readiness score claims | Final score closure belongs to the next audit phase. |

## Live Test Homes To Reuse First

| Area | Live owner homes | What they must prove |
| --- | --- | --- |
| Runtime guardrails | `crates/z00z_runtime/aggregators/tests/test_live_guardrails.rs` | No duplicate publication authority, no planner bypasses, and no alternate digest lanes. |
| Storage guardrails | `crates/z00z_storage/tests/test_live_guardrails.rs` | Storage remains the only owner of committed shard-root and proof truth. |
| Simulator evidence | `crates/z00z_simulator/tests/test_scenario_settlement.rs`, `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` | Publication trace-pack completeness, scenario sync, and artifact drift rejection. |
| Rollup-node integration | `crates/z00z_rollup_node/tests/` | Publication handoff validation, topology metadata, and composition-root publication status. |
| Settlement recovery seam | `crates/z00z_storage/src/settlement/test_live_recovery.rs` | Recovery lineage, crash continuity, and existing storage-side fail-closed recovery checks. |
| Proof-family compatibility | `crates/z00z_storage/tests/test_hjmt_live_proof_families.rs`, `test_hjmt_batch_proof.rs` | Existing proof-family semantics remain stable while public proof composition is added above them. |

## Phase 057 Test Files

| Planned File | Status | Purpose |
| --- | --- | --- |
| `crates/z00z_storage/tests/test_hjmt_root_generation.rs` | verified live | Root-generation, `ShardRootLeafV1`, and `CheckpointPublicationV1` manifest coverage plus tamper rows. |
| `crates/z00z_runtime/aggregators/tests/test_hjmt_publish.rs` | verified live | Canonical runtime publication assembly, route binding, and checkpoint continuity coverage. |
| `crates/z00z_storage/tests/test_hjmt_historical_proofs.rs` | verified live | Layered public proof and historical compatibility coverage. |
| `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs` | verified live | Join-as-standby versus join-as-owner coverage. |
| `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs` | verified live | Transfer, carry-forward, and crash-recovery publication coverage. |
| `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs` | verified live | Validator acceptance of canonical checkpoint publication and reject rows. |
| `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs` | verified live | Watcher digest/export sameness and fail-closed watcher-side reject rows. |
| `crates/z00z_storage/tests/test_hjmt_scope_birth.rs` | verified live | Dynamic-scope publication continuity for first-seen births. |
| `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs` | verified live | Byte-identical carry-forward and same-lineage failover evidence. |
| `crates/z00z_simulator/tests/test_hjmt_runtime_config.rs` | verified live | YAML-defined topology transitions and publication-profile contract coverage. |

## Config Artifact Homes

| Contract Surface | Home Status | Test Responsibility |
| --- | --- | --- |
| `config/hjmt_runtime/sim_5a7s/aggregators/agg-*/aggregator-config.yaml` | verified live | Tests must prove owner, standby, endpoint, and publication-role settings materially change observed publication behavior or fail closed. |
| `config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml` | verified live | Tests must prove route source, cadence, and route-generation boundaries materially change activation and publication planning behavior. |
| `config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml` | verified live | Tests must prove backend, journal, cache, and recovery settings materially change publication startup or carry-forward recovery behavior. |
| `crates/z00z_simulator/src/scenario_1/scenario_config.yaml` | verified live | Tests must prove old/new topology, route-generation boundaries, owner/standby roles, planned join mode, transfer target, activation point, and lawful `old_aggregator_count -> new_aggregator_count` cardinality changes are loaded from disk and affect behavior. |
| `crates/z00z_simulator/src/scenario_1/scenario_design.yaml` | verified live | Tests must prove executable publication stages and user-facing design documentation remain synchronized. |

## Test File Placement

| Scenario ID | Test File Path | Execution Status | Why This Is The Correct Home |
| --- | --- | --- | --- |
| `057-SC-01` | `crates/z00z_storage/tests/test_hjmt_root_generation.rs` | verified live | Root-generation behavior and exact `ShardRootLeafV1` bytes belong to the storage-owned committed root seam. |
| `057-SC-02` | `crates/z00z_runtime/aggregators/tests/test_hjmt_publish.rs` and `crates/z00z_storage/tests/test_hjmt_root_generation.rs` | verified live | Canonical `CheckpointPublicationV1` ordering, route binding, and prior-root continuity are split between runtime publication assembly and manifest-backed contract fixtures. |
| `057-SC-03` | `crates/z00z_storage/tests/test_hjmt_historical_proofs.rs` and `crates/z00z_storage/tests/test_hjmt_live_proof_families.rs` | verified live and extended | Two-layer proof composition stays anchored at the storage proof seam that already owns proof truth. |
| `057-SC-04` | `crates/z00z_storage/tests/test_hjmt_historical_proofs.rs` and `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs` | verified live | Compatibility and reject rows remain visible both at the proof seam and at downstream acceptance. |
| `057-SC-05` | `crates/z00z_simulator/tests/test_hjmt_runtime_config.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_publish.rs`, and `crates/z00z_simulator/tests/test_scenario_settlement.rs` | live and extended | `SIM-5A7S-PUB` integration spans live config loading, runtime publication, and simulator acceptance. |
| `057-SC-06` | `crates/z00z_simulator/tests/test_scenario_settlement.rs` and `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` | extended | Publication trace-pack completeness and design/runtime sync already belong to the simulator evidence seams. |
| `057-SC-07` | `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs` | verified live | Standby-only join belongs to runtime-owned publication continuity and must not leak into storage or watcher ownership. |
| `057-SC-08` | `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs` and `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs` | live and extended | Owner activation requires both runtime transition logic and composition-root handoff validation, including pre-activation rejects. |
| `057-SC-09` | `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs` and `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs` | live and extended | Route-generation transfer is runtime-owned, but the node must validate activation checkpoint and publication handoff metadata. |
| `057-SC-10` | `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`, and `crates/z00z_storage/src/settlement/test_live_recovery.rs` | live and extended | Carry-forward and crash recovery need runtime transition logic plus storage-side recovery and same-lineage continuity evidence. |
| `057-SC-11` | `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`, `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`, `crates/z00z_storage/tests/test_hjmt_scope_birth.rs`, and `crates/z00z_simulator/tests/test_scenario_settlement.rs` | live and extended | Downstream digest sameness and first-scope continuity require validator/watcher acceptance plus storage birth and trace linkage evidence. |
| `057-SC-12` | `crates/z00z_runtime/aggregators/tests/test_live_guardrails.rs`, `crates/z00z_storage/tests/test_live_guardrails.rs`, and `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` | extended | Anti-drift assertions belong in the seams that already police duplicate authority and artifact drift. |
| `057-SC-13` | `crates/z00z_storage/tests/test_bench_lanes.rs`, `crates/z00z_storage/benches/settlement_hjmt.rs`, `crates/z00z_storage/benches/settlement_shard.rs`, `crates/z00z_storage/benches/settlement_nested.rs`, and `crates/z00z_storage/benches/settlement_proofs.rs` | extended | Bench-home truth and no-second-harness protection already live in the current storage bench and guardrail homes. |

## Gate Coverage Map

| Gate | Primary proof homes |
| --- | --- |
| `057-G1` root-generation behavior | `057-SC-01`, `SRL-G-*`, compatibility traces |
| `057-G2` exact `ShardRootLeafV1` contract | `057-SC-01`, `SRL-T-*`, storage guardrails |
| `057-G3` exact `CheckpointPublicationV1` contract | `057-SC-02`, `CPP-G-*`, `CPP-T-*` |
| `057-G4` layered proof composition and compatibility | `057-SC-03`, `057-SC-04` |
| `057-G5` real `SIM-5A7S-PUB` publication lane | `057-SC-05`, `057-SC-06` |
| `057-G6` join-as-standby versus join-as-owner | `057-SC-07`, `057-SC-08` |
| `057-G7` route-generation-bound transfer | `057-SC-08`, `057-SC-09` |
| `057-G8` byte-identical carry-forward | `057-SC-10`, `FOV-G-002`..`FOV-G-004` |
| `057-G9` validator and watcher contract sameness | `057-SC-11` |
| `057-G10` dynamic-scope publication continuity | `057-SC-11` |
| `057-G11` lineage and scenario continuity honesty | `057-SC-06`, `057-SC-11`, `057-SC-12`, `057-SC-13` |

## TODO Scenario Coverage Crosswalk

| TODO required scenario coverage bullet | Scenario IDs | Primary proof homes |
| --- | --- | --- |
| pre-shard versus post-shard generation confusion reject | `057-SC-01` | `test_hjmt_root_generation.rs` |
| exact shard-leaf set coverage | `057-SC-01`, `057-SC-05` | `test_hjmt_root_generation.rs`, `test_hjmt_publish.rs`, simulator publication lane |
| carried-forward leaf byte identity | `057-SC-10` | `test_hjmt_migrate.rs`, failover and recovery seams |
| route-table digest binding at the publication layer | `057-SC-02`, `057-SC-05`, `057-SC-09` | `test_hjmt_publish.rs`, simulator traces, transfer tests |
| prior public root continuity | `057-SC-02` | `test_hjmt_publish.rs`, `CPP-G-*` |
| worked example replay for `6.8.1` and `6.8.2` | `057-SC-03` | `test_hjmt_historical_proofs.rs` |
| `6.8.3` cross-shard counterexample reject | `057-SC-04` | `test_hjmt_historical_proofs.rs`, validator acceptance suite |
| `6.8.4` join-as-standby and join-as-owner continuity | `057-SC-07`, `057-SC-08` | `test_hjmt_join.rs` |
| `5x7 -> 6x7` standby-only addition without public owner change | `057-SC-07` | `test_hjmt_join.rs` |
| `5x7 -> 6x7` owner activation after route generation `N+1` | `057-SC-08` | `test_hjmt_join.rs`, `test_hjmt_preflight.rs` |
| `6.8.5` mid-window failure containment | `057-SC-10` | `test_hjmt_migrate.rs`, `test_hjmt_failover_same_lineage.rs` |
| 5x7 checkpoint publication | `057-SC-02`, `057-SC-05` | `test_hjmt_publish.rs`, simulator publication lane |
| route-generation shard transfer to a remaining aggregator and to a new aggregator | `057-SC-09` | `test_hjmt_migrate.rs`, `test_hjmt_preflight.rs` |
| route-migration crash recovery | `057-SC-10` | `test_hjmt_migrate.rs`, storage recovery seam |
| wrong-lineage or pre-activation publication reject | `057-SC-04`, `057-SC-08`, `057-SC-10` | proofs, join activation rejects, recovery rejects |
| first-seen semantic scope immediately before standby takeover | `057-SC-11` | validator/watcher suites plus `test_hjmt_scope_birth.rs` |
| first-seen semantic scope immediately before carry-forward | `057-SC-11` | validator/watcher suites plus simulator traces |
| validator and watcher evidence continuity from the same runtime lineage | `057-SC-11` | validator and watcher publication-contract suites |
| config digests, process ids, journal paths, owner/standby assignments, process exit/restart verdicts, and runtime on/off matrix join-or-leave rows | `057-SC-05`, `057-SC-06`, `057-SC-13` | simulator evidence packet, guardrails, closeout matrix |

## Required End-To-End Behaviors

| Behavior | Requirement | Primary Path | Pass Signal | Fail Signal |
| --- | --- | --- | --- | --- |
| Root-generation boundary is explicit and fail-closed | `057-G1`, `057-G2` | committed root -> generation tag -> `ShardRootLeafV1` bytes -> vector replay | Last pre-shard root, first route table, first shard leaf, and confusion reject rows all behave exactly as specified. | Generation confusion or wrong-era leaf bytes are accepted. |
| Public truth is one canonical ordered leaf set and one digest story | `057-G2`, `057-G3` | ordered leaves -> `CheckpointPublicationV1` -> prior-root linkage -> digest | Ascending `ShardId` order, stable bytes, monotonic continuity, and one canonical digest all hold. | Different encodings, reordered leaves, or alternate digest stories are accepted. |
| Publication proof stays layered above shard-local proof truth | `057-G4` | shard-local proof -> public shard-leaf inclusion -> historical verification | Worked examples `6.8.1` and `6.8.2` pass while cross-shard or wrong-lineage counterexamples reject. | Publication collapses proof families into one ambiguous surface or accepts wrong-lineage proofs. |
| `SIM-5A7S-PUB` is real but topology-generic | `057-G5`, `057-G11` | YAML config -> runtime publication -> simulator evidence -> trace verification | Canonical 5x7 publication passes and at least one additional positive YAML transition changes behavior without code edits, while keeping the inherited `SIM-5A7S` packet as the acceptance baseline and preserving lawful `aggregator_count > 0` and `shard_count > 0` constraints. | `5x7` is hard-coded as the only supported topology, cardinality constraints are not honored, or traces are detached from the executed config. |
| Join-as-standby and join-as-owner are distinct protocol states | `057-G6`, `057-G7` | old topology -> standby add -> route generation `N+1` -> owner activation checkpoint | Standby state mirrors lineage without new authority; owner state activates only after committed generation advance. | Pre-activation owner publication succeeds or standby path creates public authority early. |
| Transfer is route-generation-bound and crash recovery has one lawful outcome | `057-G7`, `057-G8` | old route table -> new route table -> activation checkpoint -> crash/replay -> resumed publication | Historical proofs stay continuous, transfer binds to committed generation change, and crash recovery converges to one lawful checkpoint outcome. | Silent reroute, ambiguous activation, or multiple lawful post-crash publications appear. |
| Mid-window failure containment preserves unchanged bytes | `057-G8` | failure mid-window -> carry-forward leaf -> unaffected new leaves -> publication digest | Failed shard leaf bytes remain byte-identical while unaffected shards may advance. | Carried-forward bytes change or failure containment mutates unrelated shard leaves. |
| Validators and watchers share one digest and one verdict map | `057-G9`, `057-G10` | canonical publication -> validator acceptance -> watcher export -> trace linkage | Validator and watcher surfaces expose the same publication digest and same pass/reject meaning. | Watchers derive a local digest or validators and watchers disagree on verdict meaning. |
| First-seen scope birth surfaces only through lawful publication outputs | `057-G10` | storage scope birth -> touched shard root -> touched leaf -> touched checkpoint digest -> downstream evidence | New scope birth changes only the touched shard root, leaf, and checkpoint digest, with no extra public registry. | A second publication registry appears or scope birth is not visible downstream. |
| Config/process evidence and inherited runtime on/off matrix stay attached to publication truth | `057-G5`, `057-G11` | YAML config -> process topology -> publication traces -> closeout matrix | The same evidence packet records config digests, process ids, journal paths, owner/standby assignments, exit/restart verdicts, and join-or-leave runtime on/off rows. | Publication passes without the supporting runtime evidence, or a second matrix or detached evidence packet appears. |
| Trace pack, design sync, and bench closeout stay honest | `057-G11` | scenario execution -> publication traces -> design docs -> bench lanes -> guardrails | Every publication trace links back to Phase 056 lineage, `scenario_design.yaml` stays synced, and benches remain in existing homes only. | Traces are stale, design docs drift, or a second bench harness or evidence lane appears. |

## Critical Integration Paths

1. `z00z_storage` committed root generation -> `ShardRootLeafV1` bytes ->
   `z00z_runtime/aggregators` publication request -> `CheckpointPublicationV1`.
2. Phase 056 config, tx, route, planner, journal, scope, process, and
   recovery lineage ->
   `z00z_rollup_node` publication handoff validation ->
   `z00z_simulator` publication trace pack.
3. shard-local proof truth -> public shard-leaf inclusion ->
   historical verification at storage and validator seams.
4. YAML old/new topology and role config -> standby join ->
   route generation `N+1` -> owner activation checkpoint.
5. old route table plus new route table -> transfer to a remaining aggregator
   or transfer to a new aggregator -> carry-forward or crash recovery -> one
   lawful publication outcome.
6. first-seen scope birth -> touched shard root -> touched shard leaf ->
   touched checkpoint digest -> validator and watcher continuity evidence.
7. storage bench homes -> bench-lane guard tests -> simulator and guardrail
   closeout -> no second harness and no second authority layer.

## Realistic Examples To Implement

| Example ID | Scenario IDs | Test Home | Journey | Pass Condition | Failure Condition |
| --- | --- | --- | --- | --- | --- |
| `057-EX-01` | `057-SC-01`, `057-SC-02` | `test_hjmt_root_generation.rs`, `test_hjmt_publish.rs` | Start from the last pre-shard root and the first shard route table, generate the first lawful `ShardRootLeafV1`, assemble the canonical 5x7 ordered leaf set, and publish one `CheckpointPublicationV1` linked to the prior public root. | Canonical bytes, canonical digest, prior-root continuity, monotonicity, and route binding all hold. | Generation confusion, tampered leaf bytes, reordered leaves, or tampered prior-root linkage are accepted. |
| `057-EX-02` | `057-SC-03`, `057-SC-04` | `test_hjmt_historical_proofs.rs`, validator publication suite | Reconstruct the worked two-layer public membership proof for one shard-local batch, then mutate shard identity, lineage, or cross-shard placement to replay the `6.8.3` counterexample. | Positive two-layer proof verifies and historical compatibility rows pass exactly when lineage permits. | A cross-shard or wrong-lineage proof verifies, or layered proof data is flattened into an ambiguous object. |
| `057-EX-03` | `057-SC-05`, `057-SC-06` | `test_hjmt_runtime_config.rs`, `test_scenario_settlement.rs`, `test_scenario1_stage_surface.rs` | Run canonical `SIM-5A7S-PUB`, then run one additional positive YAML-loaded topology transition without code edits, and collect `leaf_flow.json` through `watch_flow.json` linked back to the full Phase 056 trace pack and the same config/process evidence packet. Treat the checked-in `scenario_1` publication lane as the live successor of `test_hjmt_multi_aggregator_sim.rs`. | Seven canonical leaves publish for 5x7, the second topology changes behavior lawfully, every publication trace resolves back to one lineage packet, and the evidence packet carries config digests, process ids, journal paths, owner/standby assignments, exit/restart verdicts, and runtime on/off rows. | The code path hard-codes `5x7`, omits a required trace or runtime evidence row, or lets `scenario_design.yaml` drift. |
| `057-EX-04` | `057-SC-07`, `057-SC-08` | `test_hjmt_join.rs`, `test_hjmt_preflight.rs` | Load a YAML transition from `5x7` to `6x7`, first as standby-only with no public owner change, then as owner activation after route generation `N+1` with one explicit activation checkpoint. | Standby row mirrors lineage without new public authority; owner row activates only after committed generation advance; pre-activation rows reject. | Standby path can publish as owner, or owner activation succeeds without route generation `N+1`. |
| `057-EX-05` | `057-SC-09`, `057-SC-10` | `test_hjmt_migrate.rs`, `test_hjmt_failover_same_lineage.rs`, `test_live_recovery.rs` | Execute one route-generation-bound transfer to a remaining aggregator and one route-generation-bound transfer to a new aggregator, then inject a mid-window owner failure and replay recovery so one shard is carried forward byte-for-byte while other shards advance. | Old/new route tables, both transfer-target exemplars, activation checkpoint, historical-proof continuity, and one lawful recovered publication outcome are all explicit. | Silent reroute, a missing transfer-target branch, changed carry-forward bytes, or multiple lawful post-crash checkpoints appear. |
| `057-EX-06` | `057-SC-11` | validator and watcher publication suites, `test_hjmt_scope_birth.rs`, simulator traces | Execute a first-seen semantic scope birth of a `definition_id` or `serial_id` immediately before standby takeover and immediately before carry-forward, then assert validator and watcher evidence bind to the same publication digest and same verdict mapping. | Touched shard root, touched leaf, and touched checkpoint digest change lawfully; validator and watcher outputs match exactly. | Scope birth disappears downstream, causes an extra registry lane, or yields diverging validator/watcher digests. |
| `057-EX-07` | `057-SC-12`, `057-SC-13` | guardrails, `test_bench_lanes.rs`, storage bench homes | Compile or execute the required bench lanes on current storage bench homes only, then introduce duplicate-authority or second-harness drift and assert the guardrails fail. | Required lanes remain on accepted homes, `SIM-BATCH-1000` stays reserved-only, and guardrails reject second authority patterns. | Bench or evidence work moves into a second harness, or duplicate truth lanes slip through source-shape tests. |

## Input Fixtures And Preconditions

| Scenario ID | Inputs | Preconditions | Fixture Source |
| --- | --- | --- | --- |
| `057-SC-01` | current HJMT root, first shard route table, `SRL-G-*`, `SRL-T-*` | storage root and generation types already exist and Phase 056 route lineage is available | verified `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_root_leaf_v1/` plus Phase 056 route packet |
| `057-SC-02` | canonical seven-leaf set, prior public root, route digest, `CPP-G-*`, `CPP-T-*` | `057-SC-01` leaf bytes are frozen | verified `crates/z00z_storage/tests/fixtures/hjmt_upgrade/checkpoint_publication_v1/` plus Phase 056 manifest |
| `057-SC-03` | shard-local proofs, publication leaf bytes, worked example path data | positive leaf and checkpoint vectors exist | Phase 055 proof vectors plus `057-SC-01` and `057-SC-02` fixtures |
| `057-SC-04` | wrong-shard, wrong-lineage, cross-shard, and pre-activation reject rows | positive proof composition exists for contrast | mutation helpers derived from `057-SC-03` vectors |
| `057-SC-05` | live runtime YAMLs, route-table bytes, publication-role settings, config digests, process ids, journal paths, owner/standby assignments | simulator and runtime can launch the inherited Phase 056 packet | `config/hjmt_runtime/sim_5a7s/**`, `scenario_config.yaml` |
| `057-SC-06` | `leaf_flow.json`, `proof_flow.json`, `pub_flow.json`, `val_flow.json`, `watch_flow.json`, and inherited runtime on/off matrix join-or-leave rows | `057-SC-05` execution emits publication traces and runtime evidence rows | phase-owned trace names plus Phase 056 trace pack |
| `057-SC-07` | old 5x7 topology, new 6x7 topology, standby role, same route generation | YAML transition fields are loaded from disk and live join seam exists | live config surfaces and join helpers |
| `057-SC-08` | old 5x7 topology, new 6x7 topology, owner role, route generation `N+1`, activation checkpoint | standby join case exists for contrast | live config surfaces, join helpers, activation-checkpoint vectors |
| `057-SC-09` | old route table, new route table, transfer target to a remaining aggregator, transfer target to a new aggregator, old and new shard roots | committed route-generation change path exists | transfer fixtures derived from live route tables and publication vectors |
| `057-SC-10` | failure timeline, `FOV-G-002`..`FOV-G-004`, carry-forward expectations, recovery metadata | transfer and publication continuity seams exist | verified `crates/z00z_runtime/aggregators/tests/fixtures/failover_v1/` plus RedB recovery state |
| `057-SC-11` | first-scope batch, first-seen `definition_id` or `serial_id`, publication digest, validator verdict mapping, watcher verdict mapping | canonical publication digest already exists and storage scope-birth seam is live | `test_hjmt_scope_birth.rs` helpers plus publication-trace packet |
| `057-SC-12` | source-shape strings, duplicate lane mutations, alternate digest-lane probes | repo source is present locally | `include_str!` guard tests and `rg` checks |
| `057-SC-13` | bench lane registrations, execution profiles, reserved benchmark profile | current storage bench homes compile | `settlement_hjmt.rs`, `settlement_shard.rs`, `settlement_nested.rs`, `settlement_proofs.rs` |

## Expected Outputs And Produced Artifacts

| Scenario ID | Expected Output | Persisted Artifact | Observable Signal |
| --- | --- | --- | --- |
| `057-SC-01` | canonical root-generation and leaf contract proof | `SRL-G-*`, `SRL-T-*` | exact bytes, generation tag, `policy_set_digest`, and reject verdict |
| `057-SC-02` | canonical checkpoint publication proof | `CPP-G-*`, `CPP-T-*` | canonical ordered leaves, prior-root linkage, publication digest, monotonicity verdict |
| `057-SC-03` | positive layered proof continuity proof | worked-example proof vectors | shard-local proof remains semantic truth and public layer verifies above it |
| `057-SC-04` | reject matrix for wrong lineage and cross-shard rows | negative proof evidence rows | explicit reject class and no ambiguous partial acceptance |
| `057-SC-05` | truthful `SIM-5A7S-PUB` publication run plus one second positive topology | `SIM-5A7S-PUB` fixture manifest | seven leaves in canonical order, route digest, config digests, publication digest |
| `057-SC-06` | complete publication trace pack linked to Phase 056 plus runtime evidence rows | `leaf_flow.json`, `proof_flow.json`, `pub_flow.json`, `val_flow.json`, `watch_flow.json` | every trace links to one config-digest set, one tx lineage, one route digest, one planner lineage, one journal lineage, one scope lineage, one process-topology view, one recovery lineage, and the same runtime on/off matrix join-or-leave rows |
| `057-SC-07` | standby-only join proof | join evidence packet | no new public owner, separate verdict, lineage mirror only |
| `057-SC-08` | owner-activation proof and pre-activation rejects | join activation packet | one activation checkpoint after route generation `N+1`, explicit reject rows before activation |
| `057-SC-09` | route-generation transfer proof for remaining and new aggregator targets | transfer evidence packet | old route table, new route table, both transfer-target branches, activation checkpoint, and continuity verdicts |
| `057-SC-10` | mid-window failure containment and crash recovery proof | `FOV-G-002`..`FOV-G-004` plus recovery evidence rows | byte-identical carry-forward and one lawful recovered publication outcome |
| `057-SC-11` | downstream digest sameness and scope-continuity proof | validator/watcher evidence rows | same publication digest, same verdict mapping, lawful first-scope continuity for first-seen `definition_id` and `serial_id` births |
| `057-SC-12` | anti-drift guard proof | none required | source-shape tests fail on second publication, proof, validator, watcher, or simulator truth lane |
| `057-SC-13` | bench-lane and profile closeout proof | bench compile outputs and measured reports when executed | required lanes exist on accepted homes only, `SIM-BATCH-1000` remains reserved-only, and the closeout matrix includes config/process evidence plus runtime on/off rows |

## Cryptographic And Security Invariants To Observe

| Invariant | Why It Matters | Assertion Shape |
| --- | --- | --- |
| `ShardRootLeafV1` bytes are exact and canonical | Prevents publication ambiguity and hidden alternate leaf encodings | Decode -> canonical re-encode -> bytes must be stable for `SRL-G-*`; any mutation must reject or alter the digest exactly as expected. |
| `policy_set_digest` semantics and `policy_digest` naming stay fixed | Prevents semantic drift in leaf binding and future compatibility | Accepted rows must expose the expected digest field and reject mislabeled or semantically wrong digest material. |
| `CheckpointPublicationV1` uses ascending `ShardId` order only | Prevents alternate leaf ordering from becoming a second truth lane | Canonical order must produce one stable publication digest; reordered or duplicated leaves must reject. |
| Prior public root continuity and monotonicity hold | Prevents checkpoint forks and non-canonical publication timelines | Accepted rows must bind to the expected prior root and advance monotonically; stale or skipped-link rows must reject. |
| Public proof remains layered above shard-local proof truth | Prevents publication from replacing or flattening storage proof semantics | Positive rows must prove a shard-local proof and a public leaf inclusion proof separately; cross-shard and wrong-lineage rows must reject. |
| Route digest and route generation bind joins and transfers | Prevents silent reroute or stale activation | Join and transfer rows must include old/new route data and reject on wrong generation or mismatched digest. |
| Carry-forward leaves are byte-identical | Prevents partial-failure recovery from mutating unchanged truth | Compare carried-forward leaf bytes and digest directly against the last lawful published leaf. |
| Validator and watcher outputs bind to the same digest and verdict map | Prevents local reinterpretation of public truth | The same accepted publication must yield one digest and one pass/reject meaning across both downstream consumers. |
| First-scope continuity adds no extra public registry lane | Prevents scope birth from becoming a second publication authority path | Only the touched shard root, touched leaf, and touched checkpoint digest may change; no extra registry object may appear. |
| Trace artifacts are evidence only | Prevents logs or JSON outputs from becoming semantic truth | Every publication trace must be cross-checked against config digests, process ids, journal paths, route digest, lineage, owner/standby assignments, and runtime on/off matrix rows. |

## Benchmark Owner Homes

| Logical lane | Live owner home | Required evidence |
| --- | --- | --- |
| publication and root-of-roots benchmark lane | `crates/z00z_storage/benches/settlement_hjmt.rs` or accepted direct successor | Bench compile and execution evidence for canonical publication and root-of-roots cost on the live storage harness only. |
| integrated shard-parallel commit lane | `crates/z00z_storage/benches/settlement_shard.rs` | Publication-aware shard-parallel commit evidence on the current shard bench home. |
| nested topology and carry-forward support | `crates/z00z_storage/benches/settlement_nested.rs` | Nested or transition-heavy support rows that corroborate carry-forward and topology-sensitive paths without creating a second harness. |
| proof-layer support | `crates/z00z_storage/benches/settlement_proofs.rs` | Proof-layer cost evidence that remains compatible with Phase 055 proof benchmarks. |
| bench-lane drift guard | `crates/z00z_storage/tests/test_bench_lanes.rs` | Lane registration, home ownership, docs linkage, and `SIM-BATCH-1000` reserved-only enforcement. |

## Anti-Drift Guardrails

- Do not place publication digest truth in storage tests unless they are
  consuming runtime-emitted publication artifacts.
- Do not place proof-authority assertions in runtime tests unless they are
  proving runtime does not overreach into storage proof ownership.
- Do not let validator or watcher tests invent a second canonical digest,
  signature, root, or verdict lane.
- Do not let simulator traces become semantic truth or bypass live route,
  checkpoint, or proof seams.
- Do not add a second publication bench harness or a second simulator lane just
  to make coverage look broader.
- Do not claim a proposed fixture or trace path is already live before
  execution verifies the exact home.

## Fixture And Trace Homes

| Contract Surface | Home Status | Test Responsibility |
| --- | --- | --- |
| `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_root_leaf_v1/` | verified live | Holds `SRL-G-*` and `SRL-T-*` exact bytes, expected verdicts, regeneration commands, and evidence pointers. |
| `crates/z00z_storage/tests/fixtures/hjmt_upgrade/checkpoint_publication_v1/` | verified live | Holds `CPP-G-*` and `CPP-T-*` exact bytes, expected verdicts, regeneration commands, and evidence pointers. |
| `crates/z00z_runtime/aggregators/tests/fixtures/failover_v1/` | verified live | Holds `FOV-G-002`..`FOV-G-004`, lineage state, process map, expected verdicts, and crash-recovery linkage. |
| `leaf_flow.json`, `proof_flow.json`, `pub_flow.json`, `val_flow.json`, and `watch_flow.json` | phase-owned contract names | Tests must verify exact linkage back to the Phase 056 `cfg_flow.json`, `tx_flow.json`, `route_flow.json`, `plan_flow.json`, `journal_flow.json`, `scope_flow.json`, `proc_flow.json`, and `recovery_flow.json` packet once execution lands their live homes. |

## Owned Fixture And Evidence Ledger

| Class | Exact owned ids or names | Minimum proof |
| --- | --- | --- |
| Shard-root golden vectors | `SRL-G-001`, `SRL-G-002`, `SRL-G-003`, `SRL-G-004` | Exact bytes, expected digest, regeneration command, and verdict. |
| Shard-root tamper vectors | `SRL-T-001`, `SRL-T-002`, `SRL-T-003`, `SRL-T-004`, `SRL-T-005`, `SRL-T-006` | One exact mutation point, reject stage, regeneration command, and verdict. |
| Checkpoint-publication golden vectors | `CPP-G-001`, `CPP-G-002`, `CPP-G-003`, `CPP-G-004`, `CPP-G-005` | Canonical ordered leaves, prior-root linkage, publication digest, regeneration command, and verdict. |
| Checkpoint-publication tamper vectors | `CPP-T-001`, `CPP-T-002`, `CPP-T-003`, `CPP-T-004`, `CPP-T-005`, `CPP-T-006`, `CPP-T-007` | One exact mutation point, reject stage, regeneration command, and verdict. |
| Carry-forward and crash vectors | `FOV-G-002`, `FOV-G-003`, `FOV-G-004` | Byte-identity proof, lineage state, recovery path, and one lawful outcome. |
| Upgrade `12.1` evidence classes | `Current HJMT root set`, `5x7 checkpoint-publication evidence`, `join-as-standby evidence`, `join-as-owner evidence`, `shard-transfer evidence`, `root-generation migration evidence` | Evidence pointer, regeneration command, expected verdict, and exact scenario anchor. |

## Definition Of Done

Phase 057 test implementation is not done until all of the following are true:

1. Every gate `057-G1` through `057-G11` has at least one primary passing test
   home and one explicit evidence artifact.
2. `SIM-SMALL`, `SIM-MEDIUM`, and `SIM-CACHE-EDGE` are all proven by runtime
   or simulator coverage, and `SIM-BATCH-1000` remains reserved-only.
3. `aggregator-config.yaml`, `planner-config.yaml`, `storage-config.yaml`, and
   `scenario_config.yaml` are all loaded from disk and proven behavior-changing
   through explicit assertions and config digests.
4. `SRL-G-001`..`SRL-G-004`, `SRL-T-001`..`SRL-T-006`,
   `CPP-G-001`..`CPP-G-005`, `CPP-T-001`..`CPP-T-007`, `FOV-G-002`,
   `FOV-G-003`, `FOV-G-004`, `Current HJMT root set`,
   `5x7 checkpoint-publication evidence`, `join-as-standby evidence`,
   `join-as-owner evidence`, `shard-transfer evidence`, and
   `root-generation migration evidence` all exist with regeneration
   instructions and explicit verdicts.
5. `leaf_flow.json`, `proof_flow.json`, `pub_flow.json`, `val_flow.json`, and
   `watch_flow.json` all link to one `cfg_flow.json`, `tx_flow.json`,
   `route_flow.json`, `plan_flow.json`, `journal_flow.json`,
   `scope_flow.json`, `proc_flow.json`, and `recovery_flow.json` packet plus
   one checkpoint digest story.
6. Remaining-aggregator transfer and new-aggregator transfer both have passing
   route-generation-bound exemplars, and the closeout matrix carries config
   digests, process ids, journal paths, owner/standby assignments,
   process exit/restart verdicts, and runtime on/off matrix join-or-leave rows.
7. Guardrail tests fail if a second planner truth path, second proof truth
   path, second publication digest path, or second simulator evidence lane
   appears.
