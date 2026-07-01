# Phase 058 TODO - Evidence Closure, Benchmarks, and Readiness Gates

Date: 2026-06-10
Status: Mandatory execution contract

Rule:
Every unchecked item in this file is an implementation gate for Phase 058.
Nothing here is optional polish. This phase decides which claims survive
evidence review and which claims must remain unmade.

## 🎯 Mission

Phase 058 is the closure phase for the HJMT sharding upgrade.

This phase does not invent core routing or publication behavior. Instead it:

- assembles cross-phase evidence into one auditable ledger;
- runs the release-mode simulator as the canonical end-to-end observability
  path;
- proves that YAML configs are real, consumed, and behavior-changing;
- proves restart, failover, migration, import, export, and dynamic scope birth
  under release conditions;
- executes the benchmark matrix under explicit and comparable settings;
- decides whether the repository has only a contract, a prototype, a verified
  slice, an integrated upgrade, or a release-ready implementation.

Naming status:

- Contract names such as `aggregator-config.yaml`,
  `planner-config.yaml`, `storage-config.yaml`,
  `test_hjmt_multi_aggregator_sim.rs`, `leaf_flow.json`, `proof_flow.json`,
  `pub_flow.json`, `val_flow.json`, and `watch_flow.json` denote phase-owned
  surfaces unless an exact live repo path is given.
- The currently live simulator config surfaces are
  `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`,
  `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`, and the current
  sync gate `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`.
- The currently live HJMT bench homes are
  `crates/z00z_storage/benches/settlement_hjmt.rs`,
  `crates/z00z_storage/benches/settlement_proofs.rs`,
  `crates/z00z_storage/benches/settlement_shard.rs`,
  `crates/z00z_storage/benches/settlement_nested.rs`, and
  `crates/z00z_storage/benches/adaptive_policy_bench.rs`.

## 🧭 Phase Boundary

### ✅ This phase owns

- cross-phase evidence mapping and claim discipline
- release-mode simulator closure
- final `SIM-5A7S`, `SIM-5A7S-PUB`, and `SIM-BATCH-1000` evidence packets
- query and search benchmark closure
- import/export and persistence readiness closure
- startup fail-closed readiness closure
- dynamic-scope plus wallet-observability closure
- proof-compression policy verdict
- final fixture-family and evidence-gap closure

### 🚫 This phase does not own

- inventing routing semantics that Phase 056 did not implement
- inventing publication semantics that Phase 057 did not implement
- using debug-only evidence as release evidence
- treating a single benchmark run as proof of readiness

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
  `13. Required Decisions And Fail-Closed Rules`,
  `13.1 Fail-Closed Discipline`,
  `Appendix A. Normative Upgrade Requirements`,
  `Appendix E.4 Review Checklist For Implementation PRs`,
  `Appendix E.5 Evidence Needed For Conformance-Safe Execution`
- [Z00Z-HJMT-Fixture-Checklist.md](../../../docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md):
  `Completion Contract`,
  `Release Gate`

### 🧱 Inputs inherited from Phases 056 and 057

- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md):
  `5. Upgrade 3: Stable Shard Layer Above Buckets`,
  `6. Upgrade 4: Root-Of-Shard-Roots Publication`,
  `6.8 Worked Example: Three Aggregators, Eight Assets, One Shard-Local Batch`,
  `Appendix E.6 Cross-Crate Module Ownership`,
  `Appendix E.7 Cross-Crate Execution Order`

Phase 058 verifies those deliveries. It does not re-own them.

### ⚙️ Primary upgrade sections owned by Phase 058

- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md):
  `9. Scorecard And Measurement Plan`,
  `9.1 Benchmark Matrix`,
  `9.2 Claim Gate`,
  `9.3 Score Claim Discipline`,
  `10. Correctness, Security, And Privacy Checklist`,
  `10.1 Evidence Mapping Discipline`,
  `12. Test And Benchmark Plan`,
  `12.1 Evidence Gaps`,
  `14. Readiness Definition`,
  `14.1 Completion Discipline`,
  `Appendix B. Repository Evidence Map`,
  `Appendix C. Design Artifact Requirements`,
  `Appendix E.4 Review Checklist For Implementation PRs`,
  `Appendix E.5 Evidence Needed For Conformance-Safe Execution`,
  `Appendix F. Discussion Coverage Matrix`,
  `Appendix F.1 Traceability For Sharding And Storage Recommendations`

### 🔄 Required cross-read sections

- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md):
  `2. Upgrade Principles`,
  `7. Upgrade 5: Local Adaptive Transitions`,
  `7.1 Transition Families`,
  `7.2 Split Proof`,
  `7.3 Merge Proof`,
  `7.4 Occupancy Privacy`,
  `7.5 Historical Proof Rule`,
  `7.6 Transition Safety Requirements`,
  `7.7 Implementation Guidance`,
  `7.8 Mermaid State View: Adaptive Transition Lifecycle`,
  `11. Implementation Roadmap`,
  `Appendix A.1 Mermaid Requirement Trace View`,
  `Appendix D. Implementation Skeletons`,
  `Appendix D.5 Stable Path And Asset Leaf Proof Skeleton`,
  `Appendix E. Implementation Guidelines`
- [Z00Z-HJMT-Design.md](../../../docs/tech-papers/Z00Z-HJMT-Design.md):
  `9.2 Benchmark Plan`,
  `9.3 Acceptance Criteria`,
  `13. Testing And Verification Strategy`,
  `13.1 Equivalence Tests`,
  `13.2 Crash Tests`,
  `13.3 Proof Tests`,
  `13.4 Performance Tests`

### 🧪 Fixture ownership for this phase

- [Z00Z-HJMT-Fixture-Checklist.md](../../../docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md):
  full closure over
  `1. ShardRouteTableV1 Golden Vectors`,
  `2. ShardRouteTableV1 Tamper Vectors`,
  `3. ShardRootLeafV1 Golden Vectors`,
  `4. ShardRootLeafV1 Tamper Vectors`,
  `5. CheckpointPublicationV1 Golden Vectors`,
  `6. CheckpointPublicationV1 Tamper Vectors`,
  `7. Failover, Carry-Forward, And Crash Vectors`,
  `8. BatchProofBlobV1 Golden Vectors`,
  `9. BatchProofBlobV1 Tamper Vectors`,
  `Completion Contract`,
  `Release Gate`
- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md):
  `12.1 Evidence Gaps` full closure over:
  `Current HJMT root set`,
  `Independent proof batch`,
  `Shared proof vector`,
  `Tampered shared proof set`,
  `Bucket commit fixture`,
  `Backend conformance fixture`,
  `Route migration fixture`,
  `Failover fixture`,
  `Historical proof fixture`,
  `Occupancy fixture`

## 🧱 Embedded audit contract

This file now embeds the final evidence, benchmark, and readiness requirements
that were previously held in the HJMT audit checklist.

### 🔒 Audit-derived source rules for Phase 058

- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md)
  `9.1 Benchmark Matrix` requires shard-scaling evidence for `1`, `2`, `4`,
  `8`, and `16` shard workers, including hot-shard ratio, global cadence,
  shard TPS, global TPS, and blocked time.
- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md)
  `9.2 Claim Gate` requires durable-root-published timing rather than
  worker-only throughput and requires visible proof-byte growth evidence for
  clustered `128` and `1024` path profiles before score claims can stand.
- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md)
  `Appendix C. Design Artifact Requirements` requires the one-machine
  multi-aggregator simulation plan, deployment-shape notes, local-versus-
  replicated journal notes, RAID-like topology notes, implementation-options
  notes, and benchmark templates to survive as checked artifacts.
- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md)
  `Appendix E.5 Evidence Needed For Conformance-Safe Execution` keeps the
  current RedB-backed local journal as the active evidence baseline. Ordered
  WAL or replicated-log crates stay future adapters behind `JournalBackend`
  conformance and equal-durability gates.
- [Z00Z-HJMT-Design.md](../../../docs/tech-papers/Z00Z-HJMT-Design.md)
  `9.2 Benchmark Plan` and `13. Testing And Verification Strategy` require the
  insert, delete, proof, recovery, equivalence, crash-interruption, and
  performance lanes to be measured or rejected explicitly rather than implied.

### 🚨 Final release blockers absorbed by Phase 058

| Blocker | Phase 058 ownership |
| --- | --- |
| No exact 5x7 topology evidence packet | Final closure owner. |
| No separate-process aggregator simulation evidence in release mode | Final closure owner. |
| No 1000-operation heavy workload profile | Primary owner. |
| No query or search TPS benchmark | Primary owner. |
| No release-mode startup self-test gate evidence | Co-owner with Phase 056 for final proof. |
| No import or export lane for public byte formats and journal manifests | Co-owner with Phases 056 and 057 for final closure. |
| No proof-compression policy verdict | Primary owner. |
| No one-packet acceptance ledger tying runtime, publication, and benchmark claims together | Primary owner. |

### ✅ Integrated-upgrade acceptance rule

Do not mark the sharding upgrade as `Integrated upgrade` or `Release-ready`
until one evidence packet proves all of the following from repository
commands:

- `SIM-5A7S` fixture generation and deterministic re-encoding;
- five independent aggregator OS processes with no shared-memory dependency;
- explicit `aggregator-config.yaml`, `planner-config.yaml`,
  `storage-config.yaml`, and `scenario_config.yaml` evidence with config
  digests;
- central planner and per-aggregator planner equivalence;
- same-lineage failover and wrong-lineage rejection;
- aggregator join as standby and as route-generation owner;
- carry-forward publication while one shard is unavailable;
- route migration crash recovery;
- 1000-operation insert, delete, search or query, proof, and mixed workloads;
- durable-root-published TPS separated from worker-local TPS;
- proof bytes and verify time for `128`, `1000`, and `1024` path profiles;
- to-disk or from-disk restart, import or export roundtrip, and corrupted
  import reject;
- at least one additional non-`5x7` positive topology loaded from YAML changes
  runtime and publication behavior without code edits, proving that `5x7` is a
  canonical fixture rather than a hard-coded system ceiling;
- startup preflight failure on wrong route digest, wrong journal lineage, wrong
  codec bytes, unsupported root or backend generation, and wrong proof-family
  tag.

## ⚙️ Mandatory implementation gates

| Gate | Requirement | Why this phase owns it | Minimum evidence |
| --- | --- | --- | --- |
| `058-G1` | Build one cross-phase evidence ledger. | Without one ledger, correctness and performance claims remain disconnected and unreviewable. | Machine-readable and human-readable claim map with commands, digests, verdicts, and source requirements. |
| `058-G2` | Run the canonical simulator in `--release`. | Release readiness cannot depend on debug-only observability. | Release-mode run metadata, config digests, process map, trace set, summary report, and proof that any optional debug-only lane such as `wallet_debug_tools` does not replace the public release lane. |
| `058-G3` | Keep simulator stages and docs synchronized. | The simulator is the user-facing explanation surface as well as the execution surface. | `scenario_config.yaml` drives runtime, `scenario_design.yaml` matches stage inventory, sync gate proves same-PR updates. |
| `058-G4` | Prove YAML configs are real and behavior-changing. | Configuration files that do not change runtime behavior are scaffolding, not implementation. | Changed YAML values alter ports, placement, planner mode, journal paths, workload profile, and failover schedule. |
| `058-G5` | Close the final `SIM-5A7S` packet. | The checklist requires one canonical 5x7 operational profile. | Routes, processes, failover, restart, scope birth, and config evidence in one packet. |
| `058-G6` | Close the final `SIM-5A7S-PUB` packet. | Publication claims must be tied to the same canonical topology. | Seven public shard leaves, publication digest, join, transfer, carry-forward, and validator/watcher evidence. |
| `058-G7` | Use `SIM-BATCH-1000` only for heavy readiness and benchmark lanes. | 1000-op batches are valuable for stress and TPS evidence, but too heavy to be the only correctness lane. | Explicit workload profile, release-mode reports, and proof that correctness still uses smaller deterministic profiles. |
| `058-G8` | Measure query, search, and shard-scaling lanes explicitly. | Mutation TPS alone cannot justify system-level performance claims. | Path lookup, terminal lookup, proof lookup, absent-proof generation, post-reload lookup metrics, a `1/2/4/8/16` shard-worker matrix plus canonical 5x7 profile, and proof-byte or verify-time evidence for `128`, `1000`, and `1024` path profiles. |
| `058-G9` | Close persistence and import/export readiness. | Restart and artifact portability are part of operational truth, not optional extras. | Roundtrip and tamper-reject evidence for route, publication, proof, and journal artifacts. |
| `058-G10` | Close startup fail-closed readiness. | Unsafe startup must remain impossible under release conditions. | Bad-config, bad-route, bad-lineage, bad-codec, bad-generation, and corrupted-state reject evidence. |
| `058-G11` | Close dynamic scope and wallet observability. | New economic activity is a live use case and must survive proof, publication, scan, and final state promotion. | Full tx-to-wallet trace proving proof-before-ownership and final spendable or confirmed-right state. |
| `058-G12` | Decide proof-compression status honestly. | The repository must not smuggle in unversioned compression claims. | Canonical `BatchProofBlobV1` closure first; optional compression lane clearly versioned and marked non-release-gating unless fully evidenced. |
| `058-G13` | Close all required fixture families and evidence gaps. | Readiness cannot be claimed while fixture families or evidence-gap classes remain open. | Full fixture closure table with exact artifacts, verdicts, and regeneration commands. |

## 🛠️ Workstreams

### 🧩 Workstream 1 - Evidence mapping and claim discipline

Build the final evidence ledger required by:

- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md):
  `10.1 Evidence Mapping Discipline`,
  `12.1 Evidence Gaps`,
  `Appendix B. Repository Evidence Map`,
  `Appendix F. Discussion Coverage Matrix`,
  `Appendix F.1 Traceability For Sharding And Storage Recommendations`

Required outcomes:

- every claim points to a command, report path, digest set, and verdict;
- every fixture class points to its checked artifact and regeneration path;
- every Appendix C design artifact points to its checked artifact, command, and
  verdict;
- every readiness decision can be traced back to a source requirement;
- unsupported claims are explicitly marked unsupported, not implied.

### 🧩 Workstream 2 - Release-mode simulator observability

Required outcomes:

- the simulator is a release-mode observability lane, not a scaffold;
- the tx path remains visible from ingress to planner admission, journal
  staging, scope birth, leaf formation, publication, validator verdict, watcher
  export, wallet scan, proof verification, and final spendable or
  confirmed-right promotion;
- `scenario_config.yaml` is the executable runtime config surface;
- `scenario_design.yaml` remains aligned with the executed stage set and
  scenario explanations.
- secret-bearing debug artifacts are excluded from the public release packet;
- if extra private observability is enabled through `wallet_debug_tools`, or a
  direct successor, it must stay on a private lane and must not become a
  release-gating dependency.

Required artifact inventory:

- `run_meta.json`
- `cfg_flow.json`
- `tx_flow.json`
- `route_flow.json`
- `plan_flow.json`
- `journal_flow.json`
- `scope_flow.json`
- `leaf_flow.json`
- `proof_flow.json`
- `pub_flow.json`
- `val_flow.json`
- `watch_flow.json`
- `wallet_scan.json`
- `asset_flow.json`
- `right_flow.json`
- `hist_flow.json`
- `occ_flow.json`
- `recovery_flow.json`
- `sim_summary.md`

### 🧩 Workstream 3 - Benchmark matrix and heavy workload profiles

Implement the measurement contract from:

- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md):
  `9.1 Benchmark Matrix`,
  `9.2 Claim Gate`,
  `9.3 Score Claim Discipline`
- [Z00Z-HJMT-Design.md](../../../docs/tech-papers/Z00Z-HJMT-Design.md):
  `9.2 Benchmark Plan`,
  `13.4 Performance Tests`

Required outcomes:

- durable-root-published TPS is separated from worker-local TPS;
- benchmark settings are explicit, equal, and reproducible;
- search and query lanes are measured, not inferred;
- shard scaling covers `1`, `2`, `4`, `8`, and `16` shard workers plus the
  canonical `SIM-5A7S` profile;
- scaling reports include hot-shard ratio, cadence, shard TPS, global TPS, and
  blocked time;
- proof bytes and verify time are measured for clustered and wide sets;
- `SIM-BATCH-1000` is used for heavy benchmark and readiness evidence only,
  while `SIM-SMALL`, `SIM-MEDIUM`, and `SIM-CACHE-EDGE` remain the lighter
  correctness and integration profiles;
- `SIM-BATCH-1000` includes insert-heavy, delete-heavy, search-heavy,
  proof-heavy, mixed present or absent, hot-shard, hot-serial, and rejected
  cross-shard workloads;
- `SIM-BATCH-1000` also includes mixed workloads where some operations touch
  existing hot scopes while others create first-seen semantic scopes;
- the Design baseline lanes `insert_many_definitions`, `insert_many_serials`,
  `insert_many_hot_serial`, `delete_many_definitions`,
  `delete_many_hot_serial`, `prove_many_assets`,
  `commit_recovery_replay`, and `compat_equivalence_random_ops` are mapped to
  executed reports or exact deterministic conformance artifacts and honest
  score claims;
- accepted measured reports are archived under
  `crates/z00z_storage/outputs/settlement/`;
- score packets preserve raw timing slices for planning, child commit, parent
  commit, journal sync, recovery replay, search or query time, and proof time
  instead of relying on one aggregate median;
- proof-byte and verify-time closure includes the `128`, `1000`, and `1024`
  path profiles required by the checklist;
- final score claims are explicitly accepted, rejected, or marked unsupported.

### 🧩 Workstream 4 - Runtime config, restart, import, and fail-closed startup

Required outcomes:

- `aggregator-config.yaml`, `planner-config.yaml`, `storage-config.yaml`, and
  `scenario_config.yaml` are proven to change live behavior;
- changed YAML values are visible in ports, placement, planner mode, journal
  paths, workload selection, and failover schedule;
- changed YAML topology values are visible in aggregator count, shard count,
  join mode, transfer target, publication leaf set size, and route-generation
  activation behavior;
- release evidence must include at least one additional positive
  non-`SIM-5A7S` topology loaded from YAML to prove that the implementation is
  parameterized rather than fixture-bound;
- import/export coverage includes:
  `ShardRouteTableV1`,
  `ShardRootLeafV1`,
  `CheckpointPublicationV1`,
  `BatchProofBlobV1`,
  journal checkpoint manifests;
- tampered import rejects;
- wrong startup state rejects;
- persisted-state corruption is detected before live work begins.

### 🧩 Workstream 5 - Dynamic scope, wallet closure, and final verdict

Required outcomes:

- first-seen `definition_id` birth exists in the release evidence lane;
- first-seen `serial_id` birth exists in the release evidence lane;
- mixed `TerminalLeaf` and `RightLeaf` creation exists in the release evidence
  lane;
- restart and failover around scope birth are covered;
- wallet scanning follows proof-before-ownership;
- final wallet-state transition into spendable assets or confirmed rights is
  proven after the proof boundary, not before it;
- historical proof, split or merge transition, and occupancy-privacy closure
  are traced back to the imported route and publication artifacts rather than
  synthetic roots;
- historical playback records `hist_flow.json` and `occ_flow.json` with old and
  new route generations, old and new root generations, historical proof
  verdicts, occupancy-disclosure verdicts, and imported-artifact validation
  verdicts;
- current-config, current-route, or current-policy reinterpretation of old
  proofs is rejected;
- the final verdict honestly classifies the repository as one of:
  contract only,
  prototype,
  verified slice,
  integrated upgrade,
  release-ready.

## 🧪 Required tests and benchmarks

### ✅ Required test coverage

Contract names below describe required coverage lanes. Exact live file homes may
land in successor suites if the repository keeps the same acceptance surface
under different filenames.

- `test_hjmt_batch_proof.rs`
- `test_hjmt_batch_proof_negative.rs`
- `test_hjmt_batch_commit.rs`
- `test_hjmt_batch_recovery.rs`
- `test_hjmt_storage_boundary.rs`
- `test_hjmt_backend_conformance.rs`
- config-surface coverage lane for runtime YAML loading and drift rejects
- journal-baseline and WAL-boundary coverage lane
- restart and persistence coverage lane
- `test_hjmt_import_export.rs`
- `test_hjmt_shard_routing.rs`
- `test_hjmt_topology.rs`
- `test_hjmt_process.rs`
- `test_hjmt_planner.rs`
- `test_hjmt_preflight.rs`
- `test_hjmt_failover_same_lineage.rs`
- `test_hjmt_split_brain_fencing.rs`
- continuation of the multi-aggregator simulation lane from Phases 056 and 057
- `test_hjmt_root_generation.rs`
- `test_hjmt_join.rs`
- `test_hjmt_publish.rs`
- `test_hjmt_migrate.rs`
- `test_hjmt_historical_proofs.rs`
- `test_hjmt_transition_proofs.rs`
- `test_hjmt_privacy_regression.rs`
- `test_hjmt_e2e.rs`
- `test_scenario1_stage_surface.rs` or the exact successor sync gate

### ✅ Required benchmark coverage

- batch-proof bytes and verify lanes in
  `crates/z00z_storage/benches/settlement_proofs.rs`
- mutation, query, scheduler, and cache lanes in
  `crates/z00z_storage/benches/settlement_hjmt.rs`
- shard-parallel commit and shard-scaling lanes in
  `crates/z00z_storage/benches/settlement_shard.rs`
- recovery and nested-history lanes in
  `crates/z00z_storage/benches/settlement_nested.rs`
- adaptive split/merge and occupancy lanes in
  `crates/z00z_storage/benches/adaptive_policy_bench.rs`
- benchmark evidence contract tracked in
  `crates/z00z_storage/benches/settlement_benches.md`

### ✅ Required execution profiles

- `SIM-SMALL` for fast deterministic correctness in the `16-64` operation range
- `SIM-MEDIUM` for integration correctness in the `128-256` operation range
- `SIM-CACHE-EDGE` for capacity-relative cache validation using the configured
  cache capacity as `cap - 1`, `cap`, `cap + 1`, and `2 * cap`
- `SIM-BATCH-1000` for heavy benchmark and readiness evidence only

### ✅ Required scenario coverage

- full tx-to-planner-to-journal-to-scope-to-leaf-to-publication-to-validator-
  to-watcher-to-wallet chain in `--release`
- first-seen `definition_id` birth
- first-seen `serial_id` birth
- mixed `TerminalLeaf` and `RightLeaf` creation
- restart after scope birth
- failover around scope birth
- route migration near a scope-creating batch
- historical proof acceptance under retained historical metadata
- current-config, current-route, and current-policy reinterpretation reject
- imported Phase 057 publication artifact playback
- `hist_flow.json` and `occ_flow.json` completeness
- stale route reject
- wrong journal lineage reject
- cross-shard batch reject
- corrupted persisted state reject
- import/export roundtrip and tampered-import reject
- positive and negative wallet proof-before-ownership cases
- redaction compliance for public evidence
- lineage disagreement reject when `scope_flow.json`, `plan_flow.json`,
  `journal_flow.json`, `leaf_flow.json`, `pub_flow.json`, or `wallet_scan.json`
  disagree about the tx or batch that created the first live object under a
  semantic scope

## 📦 Required artifacts

### 📁 Core evidence packet

- one final `SIM-5A7S` packet
- one final `SIM-5A7S-PUB` packet
- one final `SIM-BATCH-1000` packet
- one release-mode simulator packet
- one evidence ledger linking all commands, digests, verdicts, and source rules
- archived accepted measured reports under
  `crates/z00z_storage/outputs/settlement/`

### 📁 Config evidence

- aggregator YAML surface
  (contract name `aggregator-config.yaml`; exact repo path is phase-owned)
- planner YAML surface
  (contract name `planner-config.yaml`; exact repo path is phase-owned)
- storage YAML surface
  (contract name `storage-config.yaml`; exact repo path is phase-owned)
- live simulator config surface:
  `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`
- config digests
- proof that changed values change live behavior

### 📁 Runtime and publication evidence

- process ids
- ports
- data directories
- journal paths
- route-table digest
- publication digest
- join verdicts
- migration verdicts
- failover verdicts
- restart verdicts

### 📁 Simulator observability artifacts

- `run_meta.json`
- `cfg_flow.json`
- `tx_flow.json`
- `route_flow.json`
- `plan_flow.json`
- `journal_flow.json`
- `scope_flow.json`
- `leaf_flow.json`
- `proof_flow.json`
- `pub_flow.json`
- `val_flow.json`
- `watch_flow.json`
- `wallet_scan.json`
- `asset_flow.json`
- `right_flow.json`
- `hist_flow.json`
- `occ_flow.json`
- `recovery_flow.json`
- `sim_summary.md`

Missing, stale, redaction-violating, or disconnected artifacts fail the phase.

## 🧪 Fixture closure

### 🔹 Full fixture-family closure

Phase 058 closes:

- `SRT-G-001..004`
- `SRT-T-001..008`
- `SRL-G-001..004`
- `SRL-T-001..006`
- `CPP-G-001..005`
- `CPP-T-001..007`
- `FOV-001`
- `FOV-T-001`
- `FOV-T-002`
- `FOV-G-002`
- `FOV-G-003`
- `FOV-G-004`
- `BPB-G-001..005`
- `BPB-T-001..008`

### 🔹 Full upgrade `12.1` fixture-class closure

Phase 058 closes:

- `Current HJMT root set`
- `Independent proof batch`
- `Shared proof vector`
- `Tampered shared proof set`
- `Bucket commit fixture`
- `Backend conformance fixture`
- `Route migration fixture`
- `Failover fixture`
- `Historical proof fixture`
- `Occupancy fixture`

Every closure item must satisfy the
[Completion Contract](../../../docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md)
and the
[Release Gate](../../../docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md):
canonical bytes where applicable, expected digest or root, explicit verdict,
exact mutation point for tamper cases, exact reject stage, regeneration command,
and evidence pointer.

## ✅ Exit criteria

Do not mark Phase 058 complete until all gates `058-G1` through `058-G13` are
closed and all of the following are true:

- the final evidence ledger maps every live claim to concrete repository
  evidence;
- the release-mode simulator, not a debug-only run, is the canonical end-to-end
  proof lane;
- YAML configs are proven real and behavior-changing;
- scenario code and `scenario_design.yaml` stay synchronized;
- dynamic scope birth is present in the release evidence lane;
- wallet proof-before-ownership and final state promotion are both proven;
- persistence, restart, import, and export are all evidenced;
- heavy 1000-op batches are used where they add signal, without replacing the
  smaller deterministic correctness profiles;
- benchmark claims are mapped to reports and honestly rated;
- proof compression is either closed with versioned evidence or explicitly
  marked non-release-gating;
- every required fixture family and `12.1` fixture class is closed;
- the final verdict honestly states whether the repository is only a contract,
  a prototype, a verified slice, an integrated upgrade, or release-ready.
