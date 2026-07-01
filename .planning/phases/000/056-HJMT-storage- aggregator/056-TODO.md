# Phase 056 TODO - Runtime Routing, Process Topology, and Storage Handoff

Date: 2026-06-10
Status: Mandatory execution contract

Rule:
Every unchecked item in this file is an implementation gate for Phase 056.
Nothing in this file is a recommendation or a future nice-to-have unless it is
explicitly marked as a carry-forward dependency for Phase 057 or Phase 058.

## 🎯 Mission

Phase 056 is the first phase that must make the sharded runtime real.

This phase owns:

- the first canonical `SIM-5A7S` execution topology;
- independent aggregator OS processes;
- runtime-owned routing and planner behavior;
- real YAML-driven runtime configuration;
- runtime-to-storage handoff for semantic settlement work;
- lawful same-lineage failover;
- startup preflight and restart safety;
- simulator traces that prove the above.

This phase does not own public checkpoint publication or final benchmark claims.
Those are handed to Phase 057 and Phase 058 after the runtime plane is proven
to be lawful and reproducible.

Naming status:

- Contract names such as `aggregator-config.yaml`,
  `planner-config.yaml`, `storage-config.yaml`, `scope_flow.json`, and
  `test_hjmt_multi_aggregator_sim.rs` denote phase-owned surfaces unless an
  exact live repo path is given.
- The currently live simulator config surfaces are
  `crates/z00z_simulator/src/scenario_1/scenario_config.yaml` and
  `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`.
- The currently live HJMT bench homes are
  `crates/z00z_storage/benches/settlement_hjmt.rs`,
  `crates/z00z_storage/benches/settlement_shard.rs`,
  `crates/z00z_storage/benches/settlement_nested.rs`, and
  `crates/z00z_storage/benches/settlement_proofs.rs`.

## 🧭 Phase Boundary

### ✅ This phase owns

- `ShardId` routing from committed `ShardRouteTableV1`
- batch planning and cross-shard rejection in the runtime layer
- aggregator process lifecycle and one-machine multi-process orchestration
- `aggregator-config.yaml`, `planner-config.yaml`, `storage-config.yaml`, and
  `scenario_config.yaml` loading, validation, and application
- startup fail-closed preflight
- runtime evidence for journal lineage, restart, and lawful failover
- semantic runtime-to-storage handoff, including first-seen scope birth
- simulator stage and trace coverage through the runtime boundary

### 🚫 This phase does not own

- `ShardRootLeafV1` public publication as protocol truth
- `CheckpointPublicationV1`
- final validator and watcher publication closure
- final readiness score or release verdict
- versioned proof-byte compression experiments

### 🔄 Phase handoff

- Phase 057 receives a lawful execution lineage and turns it into public
  checkpoint publication.
- Phase 058 receives runtime and publication evidence and decides what claims
  are actually justified.

## 🔗 Source Map

### 📚 Global upgrade rules active in every HJMT phase

- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md):
  `Key Terms Used In This Paper`,
  `1.1 Inherited Base Constraints`,
  `1.2 Prohibited Changes`,
  `1.3 Verified Current Baseline`,
  `2.1 HJMT Remains The State Core`,
  `2.2 Optimize Inside The Existing Paradigm`,
  `2.3 Fail Closed`,
  `2.4 Narrow Versioned Contracts`,
  `2.5 Commitment Boundary`,
  `2.6 Contract Discipline`,
  `10. Correctness, Security, And Privacy Checklist`,
  `10.1 Evidence Mapping Discipline`,
  `13. Required Decisions And Fail-Closed Rules`,
  `13.1 Fail-Closed Discipline`,
  `14. Readiness Definition`,
  `14.1 Completion Discipline`,
  `Appendix A. Normative Upgrade Requirements`,
  `Appendix E.4 Review Checklist For Implementation PRs`,
  `Appendix E.5 Evidence Needed For Conformance-Safe Execution`
- [Z00Z-HJMT-Fixture-Checklist.md](../../../docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md):
  `Completion Contract`,
  `Release Gate`

### 🧱 Inputs inherited from Phase 055 and consumed here

- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md):
  `4. Upgrade 2: Bucket-Local Batch Commit Engine`,
  `4.1 Commit Flow`,
  `4.2 Delta Records`,
  `4.3 Canonical Operation Semantics`,
  `4.4 Journal States`,
  `4.5 Cache And Index Rules`,
  `4.6 Required Metrics`,
  `4.7 Durability And Publication Requirements`,
  `4.8 Implementation Guidance`,
  `4.9 C4 Dynamic View: Durable Batch Commit`,
  `8. Upgrade 6: Inside-Tree Versus Outside-Tree Data Boundary`,
  `8.1 Auxiliary Boundary Requirements`,
  `8.2 Storage Backend Boundary`,
  `8.3 Access Rules For Wallet, Aggregator, Settlement, And Storage`,
  `8.4 Protocol Types Versus Storage Types`,
  `8.5 Public Export Hygiene For Backend Wrappers`,
  `8.5.1 C4 Component View: Storage Boundary And Access Rules`,
  `8.6 Implementation Guidance`,
  `Appendix D.4 Commit Delta And Journal Skeleton`,
  `Appendix D.6 Runtime Placement And Storage Boundary Skeletons`

These sections remain normative inputs. Phase 056 consumes them and must not
redefine them.

### ⚙️ Primary upgrade sections owned by Phase 056

- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md):
  `5. Upgrade 3: Stable Shard Layer Above Buckets`,
  `5.1 Concepts`,
  `5.2 Routing Model`,
  `5.2.1 Exact Codec Contract For ShardRouteTableV1`,
  `5.2.2 Protocol Truth Versus Runtime Placement`,
  `5.3 Shard Split And Migration`,
  `5.4 Per-Shard Journal And Queue Rules`,
  `5.4.1 Planner Ownership And Batch Formation`,
  `5.4.2 Distributed-Tree Design Tradeoff And Aggregator Failure Domain`,
  `5.4.3 Runtime Placement Objects And Lawful Failover`,
  `5.4.4 C4 Dynamic View: Lawful Failover Versus Silent Reroute`,
  `5.5 Routing Safety Requirements`,
  `5.6 Implementation Guidance`,
  `Appendix D.3 Shard Route And Root Skeleton`,
  `Appendix D.6 Runtime Placement And Storage Boundary Skeletons`,
  `Appendix E.6 Cross-Crate Module Ownership`,
  `Appendix E.7 Cross-Crate Execution Order`

### 🔄 Required cross-read sections

- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md):
  `1.7 Whole-System Structure View`,
  `6.1 Root Generations`,
  `6.4 Compatibility`,
  `9.1 Benchmark Matrix`,
  `9.2 Claim Gate`,
  `9.3 Score Claim Discipline`,
  `12. Test And Benchmark Plan`,
  `12.1 Evidence Gaps`,
  `14. Readiness Definition`,
  `14.1 Completion Discipline`
- [Z00Z-HJMT-Design.md](../../../docs/tech-papers/Z00Z-HJMT-Design.md):
  `9.2 Benchmark Plan`,
  `13. Testing And Verification Strategy`,
  `13.1 Equivalence Tests`,
  `13.2 Crash Tests`,
  `13.4 Performance Tests`

### 🧪 Fixture ownership for this phase

- [Z00Z-HJMT-Fixture-Checklist.md](../../../docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md):
  `1. ShardRouteTableV1 Golden Vectors`,
  `2. ShardRouteTableV1 Tamper Vectors`,
  `7. Failover, Carry-Forward, And Crash Vectors` with primary ownership of:
  `FOV-001`, `FOV-T-001`, `FOV-T-002`
- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md):
  `12.1 Evidence Gaps` with primary ownership of:
  `Route migration fixture`,
  `Failover fixture`

## 🧱 Embedded audit contract

This file now embeds the Phase 056 requirements that were previously tracked in
the HJMT audit checklist. Nothing in that retired checklist remains normative
after this file is updated.

### 🔒 Audit-derived source rules for Phase 056

- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md)
  `5.4 Per-Shard Journal And Queue Rules` means each shard keeps its own queue,
  journal sequence, local root, shard epoch, routing generation, durability
  checkpoint, and recovery records. Cross-shard operations stay rejected.
- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md)
  `5.4.1 Planner Ownership And Batch Formation` means planner truth is
  runtime-owned and based on committed route-table lookup, canonical operation
  order, and `BatchPlanned`, not local load heuristics.
- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md)
  `5.4.2 Distributed-Tree Design Tradeoff And Aggregator Failure Domain` means
  failed shards must not be silently rerouted. Lawful takeover requires the
  same shard lineage; public carry-forward is Phase 057 work.
- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md)
  `6.8 Worked Example: Three Aggregators, Eight Assets, One Shard-Local Batch`
  is illustrative only. Phase 056 must widen this into one canonical
  five-aggregator, seven-shard execution fixture.
- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md)
  `Appendix C. Design Artifact Requirements` requires the one-machine
  multi-aggregator simulation plan, deployment-shape notes, local-versus-
  replicated journal stance, RAID-like topology notes, implementation options,
  and benchmark handoff context to become executable repository artifacts.
- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md)
  `Appendix E.5 Evidence Needed For Conformance-Safe Execution` keeps the
  current RedB-backed local journal and storage path as the active evidence
  baseline until later adapters pass backend conformance and equal-durability
  gates.

### 📐 Canonical `SIM-5A7S` runtime profile

`SIM-5A7S` is the canonical acceptance fixture for this phase. It is not an
architecture limit, not a hard-coded topology, and not the only valid runtime
shape. The runtime must accept any positive topology loaded from YAML config
files as long as the route, placement, lineage, and fail-closed invariants are
satisfied.

| Field | Required value |
| --- | --- |
| Topology status | Canonical acceptance fixture only. Real runtime topology remains config-driven for any `aggregator_count > 0` and any `shard_count > 0` that passes the required invariants. |
| Aggregators | `AggregatorId(0)..AggregatorId(4)` |
| Process model | Each aggregator is a separate OS process. The simulation must not model aggregators as threads, tasks, actors, or in-process handles sharing memory. |
| Process isolation | Every aggregator has its own data directory, journal files, network or listener config, log path, and startup or shutdown lifecycle. |
| Aggregator config | Every aggregator process starts from an explicit `aggregator-config.yaml` path. |
| Planner config | Every planner starts from an explicit `planner-config.yaml` path, including centralized and per-aggregator planner modes. |
| Storage config | Every backend starts from an explicit `storage-config.yaml` path when the backend has settings. |
| Shards | `ShardId(0)..ShardId(6)` |
| Placement | Seven primary assignments spread across five aggregators, with at least one aggregator owning two shards. |
| Standby | Every shard has at least one standby aggregator with expected journal lineage. |
| Route table | Gap-free full route-hash coverage, canonical shard set, canonical range order, and one expected `route_table_digest`. |
| Public-state handoff | Phase 056 must emit the runtime lineage needed for Phase 057 to publish seven `ShardRootLeafV1` records in ascending `ShardId` order. |
| Batch profiles | broad, hot-shard, hot-serial, delete-heavy, search-heavy, proof-heavy, mixed present or absent, and rejected cross-shard. |
| Failure profiles | primary down, standby down, stale restart, wrong lineage, split-brain, route migration during crash, and carry-forward publication handoff. |
| Evidence | Fixture manifest, regeneration command, test command, accepted or rejected verdict, root or digest output, and report path. |

### ⚙️ Process and config contract

| Requirement | Acceptance target |
| --- | --- |
| Separate process | Every aggregator runs as a separate OS process, even in the one-machine simulation. |
| No shared memory | Aggregators communicate only through configured network, storage, journal, or exported evidence surfaces. |
| Independent lifecycle | Tests can start, stop, kill, restart, and replace each aggregator independently. |
| Aggregator config | `aggregator-config.yaml` owns aggregator id, route-table path or digest, placement, network endpoints, journal path, data path, startup checks, evidence output, and local runtime limits. |
| Planner config | `planner-config.yaml` owns planner mode, route-table path or digest, batch limits, shard-local admission policy, cross-shard reject policy, planning cadence, and output paths. |
| Storage config | `storage-config.yaml` owns backend selection and backend-specific data path, cache, flush, sync, lock, generation, import or export, compression, and journal settings. |
| Scenario config | `scenario_config.yaml` owns executable simulation parameters and workload selection. |
| No hard-coded topology | Tests may provide config templates and fixture bytes, but production and simulation code must not hard-code aggregator count, shard count, route ranges, data paths, journal paths, ports, or benchmark batch sizes. |
| Positive topology domain | YAML config must allow any `aggregator_count > 0` and any `shard_count > 0`. Startup must reject zero, negative, missing, or internally inconsistent topology values. |
| Placement invariants | For every config-selected topology, each shard must have one lawful active owner, zero or more lawful standbys, full route coverage, no overlapping active ownership, and generation-bound placement transitions. |
| Config evidence | Every test and benchmark report records the exact config file paths, config digests, route-table digest, process ids, data directories, and journal paths used for the run. |

### 🧾 Journal and WAL decision captured by this phase

1. Keep the current RedB-backed local journal as the V1 baseline.
2. Treat `JournalBackend` as the abstraction boundary for future journal
   replacements.
3. Do not synchronize one shared WAL across all aggregators as protocol truth.
4. For standby failover, require the same `ShardId`, the same
   `routing_generation`, and the same journal lineage.
5. If replicated journaling is added later, it must arrive behind an explicit
   `ShardGroupId` scope, a dedicated conformance suite, a failure matrix, and
   equal-durability benchmarks before any readiness claim.
6. External ordered-WAL or replicated-log crates are allowed only as future
   adapters behind `JournalBackend`. They must not leak into proof, route,
   planner, checkpoint, or wallet semantics.

### 🛑 Startup self-test gate owned by this phase

Before an aggregator, planner, journal owner, or publication worker begins
accepting live work, startup must run the following fail-closed preflight:

| Check | Required failure behavior |
| --- | --- |
| Config load | Reject startup if any required `aggregator-config.yaml`, `planner-config.yaml`, `storage-config.yaml`, or `scenario_config.yaml` value is missing, unsupported, or inconsistent with the route table. |
| Route-table codec | Reject startup if canonical re-encode changes bytes or digest. |
| Shard placement | Reject startup if a primary or standby references an unknown `ShardId`, a wrong routing generation, or an impossible owner or standby relation. |
| Journal lineage | Reject startup if persisted lineage does not match placement expectation. |
| Backend generation | Reject startup if durable state has unsupported generation or stale root metadata. |
| Proof codec | Reject startup if `BatchProofBlobV1` vectors do not decode, re-encode, and verify as expected. |
| Checkpoint handoff readiness | Reject startup if the runtime cannot produce the shard-leaf inputs Phase 057 expects, or if handoff metadata is missing, unordered, duplicated, or bound to the wrong route digest. |
| Crypto or hash domains | Reject startup if hash-domain version constants, proof-family tags, or root-generation tags do not match compiled expectations. |

### 🚨 Release blockers owned or co-owned by Phase 056

| Blocker | Phase 056 ownership |
| --- | --- |
| No exact 5x7 topology fixture | Primary owner of the first executable fixture and route digest. |
| No separate-process aggregator simulation | Primary owner of process isolation and lifecycle evidence. |
| No external YAML config contract for aggregator, planner, and backend settings | Primary owner. |
| No planner placement equivalence test | Primary owner. |
| No 5x7 aggregator on or off matrix | Primary owner of runtime failover rows; Phase 057 extends this into publication join or leave rows. |
| No startup self-test gate | Primary owner of runtime preflight behavior; Phase 058 closes release-mode proof. |
| No import or export lane for route, journal, and backend runtime artifacts | Primary owner of first roundtrip implementation; Phase 058 closes final evidence. |
| No explicit local-versus-replicated journal decision in TODO acceptance language | Primary owner. |

### ✅ Phase-owned acceptance subset

Phase 056 is not done until repository commands prove all of the following:

- `SIM-5A7S` fixture generation and deterministic re-encoding;
- five independent aggregator OS processes with no shared-memory dependency;
- explicit `aggregator-config.yaml`, `planner-config.yaml`,
  `storage-config.yaml`, and `scenario_config.yaml` evidence with config
  digests;
- at least one additional non-`5x7` positive topology loaded from YAML passes
  startup, routing, and failover invariants without code edits;
- central planner and per-aggregator planner equivalence;
- same-lineage failover acceptance and wrong-lineage, wrong-generation,
  stale-root, stale-restart, and split-brain rejection;
- to-disk or from-disk restart plus route, journal, and backend import or
  export roundtrip for the artifacts owned here;
- startup preflight failure on wrong route digest, wrong journal lineage, bad
  codec bytes, unsupported backend generation, and invalid placement wiring.

## ⚙️ Mandatory implementation gates

| Gate | Requirement | Why this phase owns it | Minimum evidence |
| --- | --- | --- | --- |
| `056-G1` | Build canonical `SIM-5A7S`. | All later publication and benchmark work depends on one fixed 5-aggregator / 7-shard runtime profile. | Fixture manifest, canonical route bytes, `route_table_digest`, placement map, regeneration command. |
| `056-G2` | Run each aggregator as a separate OS process. | Runtime placement is operational truth, so process independence must be real. | PID map, ports, data dirs, journal paths, logs, lifecycle commands, proof that tests do not rely on shared memory. |
| `056-G3` | Make runtime configuration YAML-driven and behavior-changing. | The user requires real config ownership; hidden hard-coded topology is forbidden. | `aggregator-config.yaml`, `planner-config.yaml`, `storage-config.yaml`, `scenario_config.yaml`, config digests, change-impact evidence. |
| `056-G4` | Keep planner truth in the runtime layer and prove planner-mode equivalence. | `BatchPlanned` belongs to runtime, not storage; central and per-aggregator planner modes must not diverge semantically. | Matching digest and reject matrix for both planner placement modes on the same input set. |
| `056-G5` | Implement semantic runtime-to-storage handoff without duplicating storage truth. | Runtime must submit semantic work; storage must remain sole owner of bucket derivation, subtree lifecycle, and proof truth. | Execution trace from ingress to commit, API boundary proof, and no second tree registry in runtime code. |
| `056-G6` | Preserve dynamic scope birth as a live runtime case. | New `definition_id`, `serial_id`, and first terminal/right objects are normal economic events. | `scope_flow.json`, first-seen scope tests, restart coverage around the first scope-creating batch. |
| `056-G7` | Fix journal/WAL baseline and persistence contract. | Failover and restart need one baseline before any future replicated-journal discussion. | Explicit RedB local-journal statement, `JournalBackend` seam, restart roundtrip, import/export contract for phase-owned artifacts. |
| `056-G8` | Enforce lawful failover and split-brain fencing. | Same-lineage takeover is a runtime responsibility; silent reroute is forbidden. | Accept/reject matrix for primary down, standby down, stale restart, wrong lineage, wrong generation, stale local root, and dual-owner attempts. |
| `056-G9` | Fail closed at startup. | Invalid config, route, lineage, or codec state must stop the node before live work is accepted. | `test_hjmt_preflight.rs`, preflight report, explicit failure classes. |
| `056-G10` | Make the simulator a real runtime observability lane. | The simulator must prove the runtime plane instead of hiding it behind scaffolding. | Real config loading, real process topology, trace set, and scenario-stage sync evidence. |

## 🛠️ Workstreams

### 🧩 Workstream 1 - Process topology and composition root

Implement the ownership split from:

- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md):
  `1.7 Whole-System Structure View`,
  `5.2.2 Protocol Truth Versus Runtime Placement`,
  `5.4.3 Runtime Placement Objects And Lawful Failover`,
  `Appendix E.6 Cross-Crate Module Ownership`,
  `Appendix E.7 Cross-Crate Execution Order`

Required outcomes:

- `z00z_rollup_node` remains the composition root;
- each aggregator process is independently startable, stoppable, killable, and
  restartable;
- no accepted topology path uses threads, tasks, or hidden globals as the
  system model for multi-aggregator orchestration;
- `SIM-5A7S` defines exactly five `AggregatorId` values and seven `ShardId`
  values, with at least one aggregator owning two primaries and every shard
  having at least one standby.

Implementation anchors for this workstream:

- `crates/z00z_rollup_node/src/lifecycle.rs`, or a direct successor, owns
  service attachment and process lifecycle;
- `crates/z00z_runtime/aggregators/src/agg_ingress.rs`,
  `agg_scheduler.rs`, and `agg_recovery.rs`, or direct successors, own ingress
  normalization, route lookup, shard-local admission, lawful retry, and
  same-lineage resume.

### 🧩 Workstream 2 - Routing, planner ownership, and cross-shard rejection

Implement the runtime contract from:

- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md):
  `5.2 Routing Model`,
  `5.2.1 Exact Codec Contract For ShardRouteTableV1`,
  `5.4.1 Planner Ownership And Batch Formation`,
  `5.5 Routing Safety Requirements`

Required outcomes:

- route lookup is deterministic and committed;
- shard admission is based on route-table truth, not local load guesses;
- cross-shard work is rejected before execution;
- route migration is generation-bound and explicit;
- central-planner and per-aggregator-planner modes produce the same accepted
  `BatchPlanned` digests and the same reject verdicts.

### 🧩 Workstream 3 - Storage handoff and dynamic scope birth

This workstream exists because the storage layer already owns dynamic semantic
scope creation and runtime orchestration must preserve that behavior.

Required outcomes:

- runtime submits semantic work only, not backend tree inventory;
- storage remains sole owner of bucket derivation, subtree creation, parent
  recomposition, durable root transitions, and proof truth;
- the first live object under a new `definition_id` or `serial_id` is lawful;
- first terminal and first right creation inside an already routed shard is
  covered;
- `scope_flow.json` records:
  batch or tx id,
  `ShardId`,
  `routing_generation`,
  `definition_id`,
  `serial_id`,
  leaf family,
  first-seen versus already-live scope status,
  post-commit root progression.
- `scope_flow.json` must not promote storage-private `HjmtTreeId` internals to
  protocol truth.

### 🧩 Workstream 4 - Journal, restart, and lawful failover

Implement the runtime durability and failover contract from:

- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md):
  `5.4 Per-Shard Journal And Queue Rules`,
  `5.4.2 Distributed-Tree Design Tradeoff And Aggregator Failure Domain`,
  `5.4.3 Runtime Placement Objects And Lawful Failover`,
  `5.4.4 C4 Dynamic View: Lawful Failover Versus Silent Reroute`,
  `13. Required Decisions And Fail-Closed Rules`,
  `Appendix E.5 Evidence Needed For Conformance-Safe Execution`

Required outcomes:

- RedB-backed local journal is the active V1 baseline;
- `JournalBackend` is the extension seam for later backends;
- a shared cross-aggregator WAL is explicitly forbidden as current protocol
  truth;
- restart preserves route state, placement state, journal lineage, and backend
  generation metadata;
- same-lineage standby takeover is accepted;
- wrong lineage, wrong generation, stale local root, stale restart, and
  split-brain attempts are rejected;
- crash and replay around the first scope-creating batch are covered.

### 🧩 Workstream 5 - YAML configuration and startup preflight

Required outcomes:

- `aggregator-config.yaml` owns aggregator id, role, route source, placement,
  ports, data dir, journal path, log path, startup checks, and evidence output
  settings;
- `planner-config.yaml` owns planner mode, route source, batch limits,
  shard-local admission policy, cross-shard rejection policy, cadence, and
  output settings;
- `storage-config.yaml` owns backend selection and backend-specific path,
  cache, flush, sync, lock, generation, compression, journal, and import/export
  settings;
- `scenario_config.yaml` owns simulator runtime parameters and scenario
  selection;
- topology size, shard ids, owner or standby assignments, route ranges, join
  candidates, transfer targets, and failover schedule must be loaded from YAML
  rather than encoded in Rust constants;
- `SIM-5A7S` is one mandatory config profile, but the same runtime must also be
  able to load other positive topologies without code changes;
- no accepted implementation hard-codes aggregator count, shard count, route
  ranges, ports, data paths, journal paths, planner mode, or benchmark-relevant
  batch sizes;
- startup rejects missing config, inconsistent route digests, invalid placement,
  broken journal lineage, unsupported backend generation, or codec mismatch.

### 🧩 Workstream 6 - Simulator stage sync and runtime evidence

Required outcomes:

- the simulator launches real runtime components using YAML config files;
- `scenario_config.yaml` is the executable scenario configuration surface;
- `scenario_design.yaml` is the user-facing scenario explanation surface;
- if `scenario_1` stages are added, removed, or reordered, the same change must
  update `scenario_design.yaml` in the same PR so the documented stage model
  stays truthful;
- traces are mandatory acceptance evidence, not optional debug residue.

Required trace set:

- `cfg_flow.json`
- `tx_flow.json`
- `route_flow.json`
- `plan_flow.json`
- `journal_flow.json`
- `scope_flow.json`
- `proc_flow.json`
- `recovery_flow.json`

## 🧪 Required tests and benchmark slices

### ✅ Required test coverage

Contract names below describe required coverage lanes. Exact live file homes may
land in successor suites if the repository keeps the same acceptance surface
under different filenames.

- `test_hjmt_shard_routing.rs`
- `test_hjmt_topology.rs`
- `test_hjmt_process.rs`
- `test_hjmt_node_lifecycle.rs`
- config-surface coverage lane for runtime YAML loading and drift rejects
- `test_hjmt_planner.rs`
- `test_hjmt_preflight.rs`
- `test_hjmt_failover_same_lineage.rs`
- `test_hjmt_split_brain_fencing.rs`
- `test_hjmt_scope_birth.rs`
- journal-baseline and WAL-boundary coverage lane
- restart and persistence coverage lane
- initial multi-aggregator simulation lane owned by this phase

### ✅ Required benchmark slices

- shard-parallel commit lane in
  `crates/z00z_storage/benches/settlement_shard.rs`, or a direct successor
- initial shard-scaling lane in
  `crates/z00z_storage/benches/settlement_shard.rs`, or a direct successor

### ✅ Required execution profiles

- `SIM-SMALL` for fast deterministic correctness
- `SIM-MEDIUM` for integration correctness
- `SIM-CACHE-EDGE` for capacity-relative queue and cache validation using the
  configured cache capacity as `cap - 1`, `cap`, `cap + 1`, and `2 * cap`
- `SIM-BATCH-1000` is reserved for later benchmark and readiness phases and
  must not become the default correctness profile here

### ✅ Required scenario coverage

- committed route lookup under one generation
- stale or wrong generation reject
- cross-shard batch reject at planner admission
- central-planner versus per-aggregator-planner equivalence
- first-seen `definition_id` birth
- first-seen `serial_id` birth
- first terminal or right object under an already routed shard
- mixed existing-scope and first-seen-scope batch
- duplicate terminal id in one batch reject
- path or leaf mismatch reject under live runtime admission
- primary down, standby down, stale restart, wrong lineage, wrong generation,
  stale local root, and split-brain reject
- crash after durable journal advance but before runtime ack
- replay of the first scope-creating batch after restart
- config change proof for ports, placement, planner mode, journal path, storage
  backend selection, and simulator scenario selection
- deterministic `SIM-SMALL` and `SIM-MEDIUM`
- capacity-relative `SIM-CACHE-EDGE`

## 📦 Required artifacts

### 📁 Config artifacts

- aggregator YAML surface
  (contract name `aggregator-config.yaml`; exact repo path is phase-owned)
- planner YAML surface
  (contract name `planner-config.yaml`; exact repo path is phase-owned)
- storage YAML surface
  (contract name `storage-config.yaml`; exact repo path is phase-owned)
- live simulator config surface:
  `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`
- config digests for every config surface used by the run

### 📁 Runtime evidence artifacts

- `SIM-5A7S` fixture manifest
- process ids
- bound ports
- data directories
- journal paths
- log paths
- route-table digest
- startup commands
- kill and restart commands
- explicit accept or reject verdict per failover row

### 📁 Trace artifacts

- `cfg_flow.json`
- `tx_flow.json`
- `route_flow.json`
- `plan_flow.json`
- `journal_flow.json`
- `scope_flow.json`
- `proc_flow.json`
- `recovery_flow.json`

Every trace must resolve back to one config-digest set, one route-table digest,
one journal-lineage view, and one process-topology view.
Missing, stale, or cross-linked runtime traces fail the phase.

## 🧪 Fixture ownership

### 🔹 Route-table fixtures

Phase 056 owns:

- `SRT-G-001`
- `SRT-G-002`
- `SRT-G-003`
- `SRT-G-004`
- `SRT-T-001`
- `SRT-T-002`
- `SRT-T-003`
- `SRT-T-004`
- `SRT-T-005`
- `SRT-T-006`
- `SRT-T-007`
- `SRT-T-008`

### 🔹 Failover fixtures

Phase 056 owns:

- `FOV-001`
- `FOV-T-001`
- `FOV-T-002`

### 🔹 Upgrade `12.1` fixture classes

Phase 056 owns:

- `Route migration fixture`
- `Failover fixture`

Every owned fixture must include exact canonical bytes where applicable,
expected digest or root, explicit verdict, regeneration command, and evidence
pointer as required by the
[Completion Contract](../../../docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md)
and the
[Release Gate](../../../docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md).

## ✅ Exit criteria

Do not mark Phase 056 complete until all gates `056-G1` through `056-G10` are
closed and all of the following are true:

- `SIM-5A7S` exists and is reproducible.
- every aggregator in `SIM-5A7S` runs as a separate OS process;
- all four YAML config surfaces are loaded from disk and materially affect
  runtime behavior;
- runtime placement is proven separate from protocol truth;
- planner ownership is proven to live in the runtime layer;
- storage remains the only owner of subtree lifecycle, durable commit stages,
  and proof truth;
- first-seen semantic scope birth is treated as a normal runtime case and is
  evidenced through `scope_flow.json`;
- same-lineage failover works and illegal failover states reject;
- startup preflight fails closed on the required error classes;
- persistence and import/export coverage for Phase 056-owned scope exist;
- scenario code and `scenario_design.yaml` stay synchronized;
- no required artifact or trace is missing, stale, or detached from the
  committed runtime lineage.
