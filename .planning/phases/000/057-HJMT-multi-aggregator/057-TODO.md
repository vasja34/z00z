# Phase 057 TODO - Root-of-Shard-Roots Publication, Join, and Migration

Date: 2026-06-10
Status: Mandatory execution contract

Rule:
Every unchecked item in this file is an implementation gate for Phase 057.
Nothing in this file is advisory. Phase 057 consumes the lawful runtime lineage
from Phase 056 and turns it into public checkpoint truth.

## 🎯 Mission

Phase 057 owns the publication boundary above shard-local execution.

This phase must make the following executable:

- `ShardRootLeafV1`
- `CheckpointPublicationV1`
- root-generation transitions
- carry-forward under partial failure
- join-as-standby behavior
- join-as-owner behavior after route generation `N+1`
- shard-transfer publication continuity
- validator and watcher acceptance of the same public contract

This phase must not reopen runtime routing truth, planner truth, or storage
truth. It publishes the output of those layers; it does not replace them.

Naming status:

- Contract names such as `aggregator-config.yaml`,
  `planner-config.yaml`, `storage-config.yaml`, `leaf_flow.json`,
  `proof_flow.json`, `pub_flow.json`, `val_flow.json`, `watch_flow.json`, and
  `test_hjmt_multi_aggregator_sim.rs` denote phase-owned surfaces unless an
  exact live repo path is given.
- The currently live simulator config surfaces are
  `crates/z00z_simulator/src/scenario_1/scenario_config.yaml` and
  `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`.
- The currently live storage bench homes remain
  `crates/z00z_storage/benches/settlement_hjmt.rs`,
  `crates/z00z_storage/benches/settlement_shard.rs`,
  `crates/z00z_storage/benches/settlement_nested.rs`, and
  `crates/z00z_storage/benches/settlement_proofs.rs`.

## 🧭 Phase Boundary

### ✅ This phase owns

- publication-layer canonical bytes and digests
- public shard-leaf ordering, bindings, and monotonicity
- carry-forward semantics
- join and migration rules at the publication boundary
- proof composition across shard-local and public layers
- validator and watcher evidence bound to publication truth
- public continuity for first-seen scope birth

### 🚫 This phase does not own

- new runtime placement rules
- alternate planner authority
- storage subtree lifecycle rules
- final benchmark score closure
- final readiness or release verdict

### 🔄 Phase handoff

- Phase 056 provides process, routing, planner, journal, and scope lineage.
- Phase 057 turns that lineage into public checkpoint evidence.
- Phase 058 audits the resulting runtime plus publication system and decides
  what claims are justified.

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

### 🧱 Inputs inherited from Phase 056 and consumed here

- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md):
  `5.2.1 Exact Codec Contract For ShardRouteTableV1`,
  `5.2.2 Protocol Truth Versus Runtime Placement`,
  `5.3 Shard Split And Migration`,
  `5.4.3 Runtime Placement Objects And Lawful Failover`,
  `5.4.4 C4 Dynamic View: Lawful Failover Versus Silent Reroute`,
  `Appendix D.3 Shard Route And Root Skeleton`,
  `Appendix E.6 Cross-Crate Module Ownership`,
  `Appendix E.7 Cross-Crate Execution Order`

Phase 057 consumes these execution results. It does not re-own them.

### ⚙️ Primary upgrade sections owned by Phase 057

- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md):
  `6. Upgrade 4: Root-Of-Shard-Roots Publication`,
  `6.1 Root Generations`,
  `6.1.1 Exact Codec Contract For ShardRootLeafV1`,
  `6.1.2 policy_set_digest Semantics`,
  `6.1.3 Naming Rule For policy_digest`,
  `6.2 Proof Composition`,
  `6.2.1 Worked Example: Two-Layer Public Membership Proof`,
  `6.3 Publication Cadence`,
  `6.3.1 Canonical Checkpoint Publication Object`,
  `6.3.2 Monotonicity Rules`,
  `6.3.3 Protection, Failure Containment, And Recovery Layers`,
  `6.3.4 Operational Summary And Source-Of-Truth Rules`,
  `6.4 Compatibility`,
  `6.5 Root Publication Requirements`,
  `6.6 Implementation Guidance`,
  `6.7 C4 Component View: Shard Root Publication`,
  `6.8 Worked Example: Three Aggregators, Eight Assets, One Shard-Local Batch`,
  `6.8.1 Shard-Local Trees Under One Public Root`,
  `6.8.2 One Batch Inside Shard B`,
  `6.8.3 Cross-Shard Counterexample`,
  `6.8.4 Worked Lifecycle: Adding Aggregator D`,
  `6.8.5 Worked Timeline: Aggregator B Fails Mid-Window`,
  `Appendix D.3 Shard Route And Root Skeleton`,
  `Appendix D.5 Stable Path And Asset Leaf Proof Skeleton`,
  `Appendix E.6 Cross-Crate Module Ownership`,
  `Appendix E.7 Cross-Crate Execution Order`

### 🔄 Required cross-read sections

- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md):
  `5.2.1 Exact Codec Contract For ShardRouteTableV1`,
  `5.3 Shard Split And Migration`,
  `5.4.3 Runtime Placement Objects And Lawful Failover`,
  `5.4.4 C4 Dynamic View: Lawful Failover Versus Silent Reroute`,
  `9.1 Benchmark Matrix`,
  `9.2 Claim Gate`,
  `9.3 Score Claim Discipline`,
  `12. Test And Benchmark Plan`,
  `12.1 Evidence Gaps`,
  `14. Readiness Definition`,
  `14.1 Completion Discipline`
- [Z00Z-HJMT-Design.md](../../../docs/tech-papers/Z00Z-HJMT-Design.md):
  `5.2 Root Taxonomy`,
  `8.3 Checkpoint Evidence`,
  `9.2 Benchmark Plan`,
  `13. Testing And Verification Strategy`,
  `13.3 Proof Tests`

### 🧪 Fixture ownership for this phase

- [Z00Z-HJMT-Fixture-Checklist.md](../../../docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md):
  `3. ShardRootLeafV1 Golden Vectors`,
  `4. ShardRootLeafV1 Tamper Vectors`,
  `5. CheckpointPublicationV1 Golden Vectors`,
  `6. CheckpointPublicationV1 Tamper Vectors`,
  `7. Failover, Carry-Forward, And Crash Vectors` with primary ownership of:
  `FOV-G-002`, `FOV-G-003`, `FOV-G-004`
- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md):
  `12.1 Evidence Gaps` with primary ownership of:
  `Current HJMT root set`,
  `5x7 checkpoint-publication evidence`,
  `join-as-standby evidence`,
  `join-as-owner evidence`,
  `shard-transfer evidence`,
  `root-generation migration evidence`

## 🧱 Embedded audit contract

This file now embeds the Phase 057 publication and migration requirements that
were previously kept in the HJMT audit checklist.

### 🔒 Audit-derived source rules for Phase 057

- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md)
  `5.4.2 Distributed-Tree Design Tradeoff And Aggregator Failure Domain`
  allows continued publication only by carrying forward the failed shard's last
  published `ShardRootLeafV1`. Silent reroute remains forbidden.
- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md)
  `6.3.1 Canonical Checkpoint Publication Object`,
  `6.3.2 Monotonicity Rules`, and
  `6.3.4 Operational Summary And Source-Of-Truth Rules` make public truth a
  canonical ordered leaf set with one digest story.
- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md)
  `6.8 Worked Example: Three Aggregators, Eight Assets, One Shard-Local Batch`
  is illustrative only. Phase 057 must prove the same concepts under the
  inherited five-aggregator, seven-shard fixture instead of relying on the
  smaller narrative example.
- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md)
  `6.4 Compatibility` requires continuity across the last pre-shard root, the
  first shard route table, the first shard leaf, and the first
  root-of-shard-roots publication.
- [Z00Z-HJMT-Fixture-Checklist.md](../../../docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md)
  requires exact bytes, expected digests or roots, explicit verdicts,
  regeneration commands, and evidence pointers for route, shard-root,
  publication, failover, and crash fixtures.

### 📐 Canonical `SIM-5A7S-PUB` publication profile

`SIM-5A7S-PUB` is the canonical publication acceptance fixture for this phase.
It is not the only supported public topology. Publication, join, and transfer
behavior must be derived from YAML-loaded topology and route state for any
positive old or new topology that satisfies the same public invariants.

| Field | Required value |
| --- | --- |
| Topology status | Canonical acceptance fixture only. Real publication topology remains config-driven for any `aggregator_count > 0` and any `shard_count > 0` that passes the required invariants. |
| Runtime inheritance | Uses the exact `SIM-5A7S` process-isolated topology, route-table digest, planner lineage, and config-digest set emitted by Phase 056. |
| Public leaf set | Seven active `ShardRootLeafV1` records in ascending `ShardId` order. |
| Checkpoint object | One canonical `CheckpointPublicationV1` with ordered leaf set, route binding, prior-root linkage, and monotonicity proof. |
| Join scenarios | `5x7 -> 6x7` standby-only join and `5x7 -> 6x7` owner activation after route generation `N+1` are mandatory exemplars, but the join engine must accept any positive `old_aggregator_count -> new_aggregator_count` transition loaded from YAML if the route-generation and lineage invariants hold. |
| Transfer scenarios | Shard transfer to a remaining aggregator and shard transfer to a new aggregator are mandatory exemplars, but transfer planning and publication must remain config-driven for any positive old or new topology loaded from YAML. |
| Failure scenarios | Mid-window owner failure, byte-identical carry-forward, wrong-lineage reject, and route-migration crash recovery. |
| Scope continuity | First-seen scope birth from Phase 056 must be visible in shard-leaf and checkpoint outputs without any extra public registry. |
| Evidence | Publication digest, leaf bytes, activation checkpoint, join or transfer verdicts, historical-proof verdicts, validator verdicts, watcher verdicts, and trace linkage back to the Phase 056 runtime packet. |

### 🚨 Release blockers owned or co-owned by Phase 057

| Blocker | Phase 057 ownership |
| --- | --- |
| No 5x7 checkpoint-publication evidence | Primary owner. |
| No join-as-standby evidence | Primary owner. |
| No join-as-owner evidence | Primary owner. |
| No route-generation shard-transfer evidence | Primary owner. |
| No byte-identical carry-forward proof under partial failure | Primary owner. |
| No publication continuity for first-seen scope birth | Primary owner. |
| No 5x7 join or leave rows attached to the runtime on or off matrix | Co-owner with Phase 056. |
| No publication-stage evidence packet feeding final readiness claims | Handoff owner to Phase 058. |

### ✅ Phase-owned acceptance subset

Phase 057 is not done until repository commands prove all of the following:

- `SIM-5A7S-PUB` publishes exactly seven active shard leaves in canonical
  order;
- at least one additional positive topology transition loaded from YAML changes
  join, transfer, or publication behavior without code edits while preserving
  route-generation and lineage invariants;
- join-as-standby and join-as-owner are separate protocol states with separate
  evidence packets;
- route-generation transfer to a remaining or new aggregator preserves
  historical-proof continuity and has one lawful activation checkpoint;
- mid-window failure carries forward the failed shard leaf byte-for-byte while
  unaffected shards publish new leaves;
- validator and watcher evidence binds to the same publication digest;
- publication traces remain linked to the exact route, journal, scope, config,
  and process lineage inherited from Phase 056.

## ⚙️ Mandatory implementation gates

| Gate | Requirement | Why this phase owns it | Minimum evidence |
| --- | --- | --- | --- |
| `057-G1` | Define executable root-generation behavior. | Public publication must separate the pre-shard and post-shard worlds without ambiguity. | Generation vectors, confusion rejects, compatibility traces. |
| `057-G2` | Make `ShardRootLeafV1` a real canonical contract. | One shard-local root becomes one public leaf only through exact bytes and bindings. | Golden vectors, tamper vectors, canonical digest checks, monotonicity checks. |
| `057-G3` | Make `CheckpointPublicationV1` a real canonical contract. | Public truth requires one ordered leaf set and one digest story. | Canonical bytes, canonical ordering, prior-root linkage, tamper rejects. |
| `057-G4` | Prove two-layer proof composition and historical compatibility. | Publication must compose with shard-local proofs instead of replacing them. | Historical proof vectors, proof-family tags, continuity checks across generations. |
| `057-G5` | Carry `SIM-5A7S` into `SIM-5A7S-PUB`. | Publication evidence must use the real 5x7 runtime topology rather than synthetic fixtures. | Seven public shard leaves, route digest, publication digest, replayable publication trace. |
| `057-G6` | Distinguish join-as-standby from join-as-owner. | Standby participation and public ownership are different protocol states. | Separate scenarios, separate verdicts, pre-activation reject, activation checkpoint. |
| `057-G7` | Make shard transfer route-generation-bound. | Ownership changes are lawful only through committed route generation changes. | Old route table, new route table, activation checkpoint, transfer continuity evidence. |
| `057-G8` | Enforce byte-identical carry-forward. | Partial failure containment must not mutate unchanged shard leaves. | Carry-forward vectors proving unchanged leaf bytes and digest identity. |
| `057-G9` | Bind validators and watchers to the same publication contract. | Publication is incomplete until downstream consumers accept exactly the same truth. | `val_flow.json`, `watch_flow.json`, shared digest and verdict mapping. |
| `057-G10` | Preserve dynamic-scope publication continuity. | First-seen scope birth must flow through publication without creating extra public registries. | Scope-to-leaf-to-publication trace continuity, changed shard root, changed publication digest. |
| `057-G11` | Keep scenario and lineage continuity honest. | Publication traces must stay connected to the Phase 056 runtime evidence chain. | Trace linkage to route, process, journal, and scope lineage plus scenario-doc sync. |

## 🛠️ Workstreams

### 🧩 Workstream 1 - Root generations and publication objects

Implement the publication object model from:

- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md):
  `6.1 Root Generations`,
  `6.1.1 Exact Codec Contract For ShardRootLeafV1`,
  `6.3.1 Canonical Checkpoint Publication Object`,
  `6.3.2 Monotonicity Rules`,
  `6.5 Root Publication Requirements`

Required outcomes:

- explicit pre-shard and post-shard generation handling;
- exact canonical `ShardRootLeafV1` bytes and bindings;
- exact canonical `CheckpointPublicationV1` bytes and digest;
- prior public root continuity;
- monotonicity enforcement;
- compatibility evidence covers the last pre-shard root, the first shard route
  table, the first shard leaf, and the first root-of-shard-roots publication;
- compatibility across the current-root to root-of-shard-roots transition.

### 🧩 Workstream 2 - Public proof composition and compatibility

Implement the verification path from:

- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md):
  `6.2 Proof Composition`,
  `6.2.1 Worked Example: Two-Layer Public Membership Proof`,
  `6.4 Compatibility`,
  `Appendix D.5 Stable Path And Asset Leaf Proof Skeleton`

Required outcomes:

- public proof composition stays layered, not collapsed;
- shard-local proofs remain the semantic truth component;
- publication proofs add the public shard-leaf layer;
- historical proofs remain valid across root-generation and route-generation
  changes when the protocol says they should.

### 🧩 Workstream 3 - Join, transfer, and carry-forward

Implement the publication behavior from:

- [Z00Z-HJMT-Upgrade.md](../../../docs/tech-papers/Z00Z-HJMT-Upgrade.md):
  `6.8.3 Cross-Shard Counterexample`,
  `6.8.4 Worked Lifecycle: Adding Aggregator D`,
  `6.8.5 Worked Timeline: Aggregator B Fails Mid-Window`,
  `5.3 Shard Split And Migration`

Required outcomes:

- join as standby mirrors lineage but does not create new public authority;
- join as owner requires route generation `N+1` and an activation checkpoint;
- `5x7 -> 6x7` standby join is covered as a no-new-authority case;
- `5x7 -> 6x7` owner activation is covered as a route-generation migration case;
- transfer scenarios include old route table, new route table, old shard roots,
  new shard roots, activation checkpoint, and historical-proof continuity;
- carry-forward preserves unchanged shard-leaf bytes exactly;
- route-migration crash recovery has one lawful publication outcome only.

### 🧩 Workstream 4 - Dynamic-scope publication continuity

Required outcomes:

- publication begins from committed shard-local results, never from synthetic
  leaf fixtures;
- a first-seen `definition_id` or `serial_id` birth changes the touched shard
  root and therefore the touched `ShardRootLeafV1` and
  `CheckpointPublicationV1` digest;
- no extra public "tree registration" side channel appears;
- `leaf_flow.json`, `proof_flow.json`, `pub_flow.json`, `val_flow.json`, and
  `watch_flow.json` resolve back to Phase 056 `scope_flow.json` when the run
  includes first-seen scope birth.

### 🧩 Workstream 5 - Validator, watcher, and scenario sync

Required outcomes:

- validators accept only canonical leaf sets with correct bindings;
- watchers export evidence for the same publication digest, not a derived local
  view;
- simulator continuation preserves the same config, process, and lineage model
  introduced in Phase 056;
- `scenario_config.yaml` continues to control runtime scenario parameters;
- YAML config must define the old topology, the new topology, route-generation
  boundaries, owner or standby roles, planned join mode, transfer target, and
  publication activation point;
- `5x7 -> 6x7` remains the mandatory exemplar, but the same publication layer
  must accept any positive topology transition loaded from YAML without code
  edits when invariants are satisfied;
- changing `aggregator-config.yaml` owner, standby, endpoint, or
  publication-role settings must change observed publication behavior or fail
  closed;
- if `scenario_1` stages change, `scenario_design.yaml` must be updated in the
  same PR so the user-facing explanation stays aligned with the executed stage
  graph.

Required trace set for this phase:

- `leaf_flow.json`
- `proof_flow.json`
- `pub_flow.json`
- `val_flow.json`
- `watch_flow.json`

These traces must join the Phase 056 lineage:

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

- `test_hjmt_root_generation.rs`
- `test_hjmt_historical_proofs.rs`
- `test_hjmt_join.rs`
- `test_hjmt_publish.rs`
- `test_hjmt_migrate.rs`
- continuation of the Phase 056 multi-aggregator simulation lane

### ✅ Required benchmark slices

- publication and root-of-roots benchmark lane in a Phase 057 bench home, or a
  direct successor
- integrated shard-parallel commit lane in
  `crates/z00z_storage/benches/settlement_shard.rs`, or a direct successor

### ✅ Required execution profiles

- `SIM-SMALL` for fast deterministic publication correctness
- `SIM-MEDIUM` for publication integration correctness
- `SIM-CACHE-EDGE` when cache or queue state can affect publication cadence
- `SIM-BATCH-1000` remains reserved for later benchmark and readiness evidence

### ✅ Required scenario coverage

- pre-shard versus post-shard generation confusion reject
- exact shard-leaf set coverage
- carried-forward leaf byte identity
- route-table digest binding at the publication layer
- prior public root continuity
- worked example replay for `6.8.1` and `6.8.2`
- the `6.8.3` cross-shard counterexample reject
- `6.8.4` join-as-standby and join-as-owner continuity
- `5x7 -> 6x7` standby-only addition without public owner change
- `5x7 -> 6x7` owner activation after route generation `N+1`
- `6.8.5` mid-window failure containment
- 5x7 checkpoint publication
- route-generation shard transfer
- route-migration crash recovery
- wrong-lineage or pre-activation publication reject
- first-seen semantic scope immediately before standby takeover
- first-seen semantic scope immediately before carry-forward
- validator and watcher evidence continuity from the same runtime lineage

## 📦 Required artifacts

### 📁 Publication artifacts

- `SIM-5A7S-PUB` fixture manifest
- route-table digest
- publication digest
- old and new route generations
- activation checkpoint
- join verdicts
- migration verdicts
- carry-forward digest identity
- historical-proof continuity verdict

### 📁 Config and process evidence

- aggregator YAML surface
  (contract name `aggregator-config.yaml`; exact repo path is phase-owned)
- planner YAML surface
  (contract name `planner-config.yaml`; exact repo path is phase-owned)
- storage YAML surface
  (contract name `storage-config.yaml`; exact repo path is phase-owned)
- live simulator config surface:
  `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`
- config digests
- process ids
- journal paths
- owner and standby assignments
- process exit or restart verdicts

### 📁 Trace artifacts

- publication leaf trace surface
  (contract name `leaf_flow.json`; exact repo path is phase-owned)
- publication proof trace surface
  (contract name `proof_flow.json`; exact repo path is phase-owned)
- publication digest trace surface
  (contract name `pub_flow.json`; exact repo path is phase-owned)
- validator trace surface
  (contract name `val_flow.json`; exact repo path is phase-owned)
- watcher trace surface
  (contract name `watch_flow.json`; exact repo path is phase-owned)

Each publication trace must resolve back to the matching Phase 056 execution
lineage, especially when the run includes first-seen scope birth.
Missing, stale, or cross-linked publication traces fail the phase.

## 🧪 Fixture ownership

### 🔹 Shard-root fixtures

Phase 057 owns:

- `SRL-G-001`
- `SRL-G-002`
- `SRL-G-003`
- `SRL-G-004`
- `SRL-T-001`
- `SRL-T-002`
- `SRL-T-003`
- `SRL-T-004`
- `SRL-T-005`
- `SRL-T-006`

### 🔹 Checkpoint-publication fixtures

Phase 057 owns:

- `CPP-G-001`
- `CPP-G-002`
- `CPP-G-003`
- `CPP-G-004`
- `CPP-G-005`
- `CPP-T-001`
- `CPP-T-002`
- `CPP-T-003`
- `CPP-T-004`
- `CPP-T-005`
- `CPP-T-006`
- `CPP-T-007`

### 🔹 Carry-forward and crash fixtures

Phase 057 owns:

- `FOV-G-002`
- `FOV-G-003`
- `FOV-G-004`

### 🔹 Upgrade `12.1` fixture classes

Phase 057 owns:

- `Current HJMT root set`
- `5x7 checkpoint-publication evidence`
- `join-as-standby evidence`
- `join-as-owner evidence`
- `shard-transfer evidence`
- `root-generation migration evidence`

Every owned fixture must satisfy the
[Completion Contract](../../../docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md)
and the
[Release Gate](../../../docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md):
canonical bytes where applicable, expected digest or root, explicit verdict,
regeneration command, and evidence pointer.

## ✅ Exit criteria

Do not mark Phase 057 complete until all gates `057-G1` through `057-G11` are
closed and all of the following are true:

- `ShardRootLeafV1` and `CheckpointPublicationV1` are executable contracts, not
  merely named types;
- pre-shard and post-shard generation behavior is explicit and tested;
- `SIM-5A7S-PUB` exists and proves seven active shard leaves in canonical
  order;
- join-as-standby and join-as-owner are distinct and evidence-backed;
- route migration uses committed route generations and activation checkpoints;
- carry-forward preserves unchanged shard-leaf bytes exactly;
- publication of a first-seen semantic scope is proven to affect only lawful
  shard-root and checkpoint-digest outputs;
- validator and watcher evidence binds to the same publication contract;
- scenario code and `scenario_design.yaml` stay synchronized;
- no publication artifact is detached from Phase 056 route, journal, scope, or
  process lineage.
