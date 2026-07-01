# Phase 056: HJMT Storage Aggregator - Context

**Gathered:** 2026-06-11
**Status:** ready-for-execution-planning
**Source:** deep read of `056-TODO.md`, `Z00Z-HJMT-Upgrade.md`,
`Z00Z-HJMT-Fixture-Checklist.md`, `Z00Z-HJMT-Design.md`, and the live runtime,
storage, node, and simulator seams that already own planner truth, durable
storage, and `scenario_1` evidence

<domain>

## 🎯 Phase Boundary

Phase 056 is the first packet that turns the sharded runtime from design
inventory into executable repository scope. It does not own publication truth
or release-final score claims. It owns the runtime plane that later phases must
consume without reinterpretation:

- one canonical `SIM-5A7S` acceptance fixture;
- real aggregator process topology;
- runtime-owned routing and planner truth;
- YAML-driven runtime behavior;
- semantic runtime-to-storage handoff;
- same-lineage failover;
- startup preflight;
- simulator traces that prove the above.

Phase 056 must extend the current live seams in place. It must not duplicate
the current codebase, invent a second planning or storage authority layer, or
recreate logic already owned by:

- `crates/z00z_runtime/aggregators` for routing, planner truth, placement, and
  shard-execution metadata;
- `crates/z00z_storage` for semantic settlement ownership, durable commit
  stages, subtree lifecycle, proof truth, and the low-level
  `StorageBackend`/`JournalBackend` seam;
- `crates/z00z_rollup_node` for runtime composition and service attachment;
- `crates/z00z_simulator` for `scenario_config.yaml`,
  `scenario_design.yaml`, runner verification, and evidence artifacts.

Phase 056 must also preserve the already-closed Phase 055 batch-proof authority
surface. Route, lineage, process, and configuration work may consume that
surface, but they must not reopen it or build a parallel proof or storage path.

</domain>

<review_findings>

## 🚨 Review Findings That Forced This Packet Rewrite

### S1-HIGH: The phase had no executable planning packet
- The folder contained only `056-TODO.md`.
- No `056-CONTEXT.md` existed.
- No numbered `056-*-PLAN.md` packet existed.
- Result: there was no execution-safe mapping from TODO scope to codebase
  owner homes, validation gates, or sequencing.

### S1-HIGH: The TODO uses contract names that were not yet grounded to live paths
- `aggregator-config.yaml`, `planner-config.yaml`, and
  `storage-config.yaml` are phase-owned contract names in `056-TODO.md`.
- Only `crates/z00z_simulator/src/scenario_1/scenario_config.yaml` and
  `crates/z00z_simulator/src/scenario_1/scenario_design.yaml` are verified live
  YAML anchors today.
- Result: the packet must distinguish live anchors from proposed new config
  homes instead of claiming those paths already exist.

### S1-HIGH: Concept drift risk is real if storage, runtime, and simulator roles blur
- The current codebase already has a split owner map:
  `batch_planner.rs`, `placement.rs`, `recovery.rs`, `shard_exec.rs`,
  `StorageBackend`, `JournalBackend`, `scenario_config.yaml`, and
  `scenario_design.yaml`.
- Any Phase 056 plan that reimplements planner truth in storage, semantic
  storage truth in runtime, or trace truth in a new side harness would create a
  second authority plane.

### S2-MEDIUM: Current config surfaces are too narrow for the TODO contract
- `NodeConfig` is currently minimal.
- `hjmt_config.rs` is environment-oriented, not yet the full YAML runtime
  contract demanded by Phase 056.
- The packet must schedule additive config-surface expansion instead of
  assuming the current code already satisfies the YAML/runtime contract.

### S2-MEDIUM: Current runtime helpers are intentionally narrow
- `SchedulerBoundary::plan_waves(...)` is one-wave only today.
- `RecoveryBoundary` currently records handoff only.
- The packet must plan explicit deltas for lawful failover, restart proof,
  startup preflight, and process isolation instead of pretending those
  behaviors are already live.

</review_findings>

<decisions>

## 🔑 Implementation Decisions

### D-01: `056-TODO.md` remains the canonical backlog and source authority
- This context and the numbered plans interpret and schedule the TODO.
- They do not replace it.
- If `056-TODO.md` changes, refresh this context, `056-SOURCE-AUDIT.md`, and
  every `056-*-PLAN.md` coverage contract in the same change set.

### D-02: No duplicate code path, no parallel layer, no concept drift
- Extend the existing runtime, storage, node, and simulator seams in place.
- Do not add a second planner authority path beside
  `crates/z00z_runtime/aggregators`.
- Do not add a second semantic storage authority path beside
  `crates/z00z_storage`.
- Do not add a second simulator or trace lane beside the current `scenario_1`
  config/design/runner surfaces.

### D-03: Current live owner map is frozen before execution
- `crates/z00z_runtime/aggregators/src/batch_planner.rs` owns committed route
  lookup and `BatchPlanned` construction.
- `crates/z00z_runtime/aggregators/src/placement.rs` owns operational shard
  ownership and standby metadata.
- `crates/z00z_runtime/aggregators/src/shard_exec.rs` owns runtime execution
  tickets and placement-aware shard admission.
- `crates/z00z_storage/src/backend/mod.rs` owns the low-level durable seam.
- `crates/z00z_storage/src/settlement/*` owns semantic settlement truth,
  durable commit, journal semantics, proof truth, and scope birth.
- `crates/z00z_rollup_node/src/runtime.rs` owns service composition and
  status projection.
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml` and
  `scenario_design.yaml` remain the live simulator config and design anchors.

### D-04: Existing YAML anchors and proposed YAML homes must stay clearly separated
- Verified live YAML anchors today:
  `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`,
  `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`,
  and the top-level `config/` tree.
- Proposed new Phase 056 runtime config home:
  `config/hjmt_runtime/`.
- `aggregator-config.yaml`, `planner-config.yaml`, and
  `storage-config.yaml` remain contract names until `056-01` verifies the exact
  file layout under that proposed home.
- No document in this packet may present those proposed paths as already-live
  repository facts before the first execution slice lands them.

### D-05: `SIM-5A7S` is an acceptance fixture, not a hard-coded architecture limit
- `SIM-5A7S` is the canonical acceptance topology for this phase.
- The production runtime must still accept any positive
  `aggregator_count > 0` and any positive `shard_count > 0` that satisfies the
  invariants.
- No Rust constant may silently freeze 5 aggregators or 7 shards as the only
  supported topology.

### D-06: Aggregators must be modeled as separate OS processes
- The accepted runtime shape is process-based.
- Tests may orchestrate those processes on one machine, but they must not model
  the canonical multi-aggregator topology as one in-memory thread or task mesh
  sharing state.

### D-07: Planner truth stays runtime-owned and digest-bound
- Route-table lookup, route-table digest, routing generation, intake ids, and
  `BatchPlanned` all remain runtime authority.
- Storage may validate the handoff, but it must not recalculate route truth as
  an alternate planner lane.

### D-08: Cross-shard work remains rejected in the runtime layer
- Cross-shard work must reject before semantic execution.
- Storage must not become a fallback path that silently accepts mixed-shard
  batches after runtime rejection.

### D-09: Runtime-to-storage handoff is semantic-only
- Runtime may submit semantic work items and committed routing context.
- Runtime must not submit backend tree inventory, subtree registry authority,
  or proof truth as if it owned them.
- Storage remains the only owner of bucket derivation, subtree creation, parent
  recomposition, durable root transitions, and proof truth.

### D-10: Dynamic scope birth stays storage-owned but is Phase 056 live scope
- First-seen `definition_id`, first-seen `serial_id`, and first terminal/right
  objects are normal runtime cases in this phase.
- The runtime must preserve them through semantic handoff and restart/failover.
- `scope_flow.json` is mandatory acceptance evidence for this behavior.

### D-11: The journal baseline is the current RedB local journal behind `JournalBackend`
- RedB-backed local journaling remains the V1 durability baseline.
- `JournalBackend` remains the extension seam for future backends.
- A shared cross-aggregator WAL is explicitly forbidden as current protocol
  truth in this phase.

### D-12: Lawful failover requires same-lineage takeover
- Same `ShardId`.
- Same `routing_generation`.
- Same expected journal lineage.
- Coherent local root / backend generation state.
- No silent reroute, no stale restart acceptance, no split-brain acceptance.

### D-13: Startup preflight must fail closed
- Invalid config, route bytes, route digest, placement relations, journal
  lineage, backend generation, proof codec assumptions, or publication-handoff
  metadata must stop startup before live work is accepted.

### D-14: Simulator artifacts remain evidentiary, not semantic truth
- `cfg_flow.json`, `tx_flow.json`, `route_flow.json`, `plan_flow.json`,
  `journal_flow.json`, `scope_flow.json`, `proc_flow.json`, and
  `recovery_flow.json` are required runtime evidence.
- They may describe the runtime; they must not replace runtime or storage
  semantic truth.

### D-15: `scenario_design.yaml` must stay in lockstep with executable stage behavior
- Any stage addition, reorder, or runtime-plane expansion in `scenario_1` must
  update `scenario_design.yaml` in the same slice.
- Do not accept a hidden runtime lane that is not described by the design doc.

### D-16: The packet is seven ordered execution slices
- `056-01` topology and config ownership freeze.
- `056-02` route table and planner equivalence.
- `056-03` semantic storage handoff and scope birth.
- `056-04` journal lineage, restart, and lawful failover.
- `056-05` YAML materialization and startup preflight.
- `056-06` simulator stage sync and runtime evidence.
- `056-07` fixtures, benchmarks, validation closeout, and final evidence sync.

### D-17: Mandatory verification order for every Rust or test-affecting slice
- Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  first.
- Then run the slice-local targeted tests.
- Then run `cargo test --release` as the broad corroborating gate when Rust or
  tests changed.
- Then run `/GSD-Review-Tasks-Execution` in YOLO mode at least 3 times and
  continue until at least two consecutive review passes show no significant
  issues.

### D-18: Live feature names beat stale planning drift
- Use `test-params-fast` when fast release-style validation is appropriate.
- Use `wallet_debug_tools` when the simulator needs its current debug-only
  wallet dump feature.
- Do not use stale older aliases in this packet.

### D-19: Proposed targets must be labeled honestly
- If a file, YAML home, report schema, or trace owner is not yet verified in
  the current tree, label it as proposed.
- Do not upgrade a proposed path into a live fact just because the TODO names a
  contract.

### D-20: Phase 056 is not complete until the TODO exit criteria are fully closed
- All gates `056-G1` through `056-G10`.
- The required artifacts and traces.
- The required execution profiles.
- The required scenario coverage.
- The no-duplicate-layer rule.

</decisions>

<canonical_refs>

## 📚 Canonical References

### Phase authority
- `.planning/phases/056-HJMT-storage- aggregator/056-TODO.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-CONTEXT.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-SOURCE-AUDIT.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`

### Normative docs
- `docs/tech-papers/Z00Z-HJMT-Upgrade.md`
- `docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md`
- `docs/tech-papers/Z00Z-HJMT-Design.md`
- `docs/tech-papers/Z00Z-HJMT-Key-Terms.md`

### Predecessor packet anchors
- `.planning/phases/000/055-HJMT-boundary/055-CONTEXT.md`
- `.planning/phases/000/055-HJMT-boundary/055-TEST-SPEC.md`
- `.planning/phases/000/054-Refactor-Crates/054-CONTEXT.md`
- `.planning/phases/000/053-HJMT-Backend/053-CONTEXT.md`

### Live runtime anchors
- `crates/z00z_runtime/aggregators/README.md`
- `crates/z00z_runtime/aggregators/src/lib.rs`
- `crates/z00z_runtime/aggregators/src/ingress.rs`
- `crates/z00z_runtime/aggregators/src/batch_planner.rs`
- `crates/z00z_runtime/aggregators/src/placement.rs`
- `crates/z00z_runtime/aggregators/src/recovery.rs`
- `crates/z00z_runtime/aggregators/src/scheduler.rs`
- `crates/z00z_runtime/aggregators/src/service.rs`
- `crates/z00z_runtime/aggregators/src/shard_exec.rs`
- `crates/z00z_runtime/aggregators/src/types.rs`

### Live storage anchors
- `crates/z00z_storage/src/backend/mod.rs`
- `crates/z00z_storage/src/settlement/mod.rs`
- `crates/z00z_storage/src/settlement/store.rs`
- `crates/z00z_storage/src/settlement/hjmt_store.rs`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs`
- `crates/z00z_storage/src/settlement/hjmt_plan.rs`
- `crates/z00z_storage/src/settlement/hjmt_config.rs`
- `crates/z00z_storage/src/settlement/test_live_recovery.rs`
- `crates/z00z_storage/src/settlement/README.md`
- `crates/z00z_storage/src/settlement/root_types.md`

### Live node anchors
- `crates/z00z_rollup_node/src/config.rs`
- `crates/z00z_rollup_node/src/runtime.rs`
- `crates/z00z_rollup_node/src/status.rs`
- `crates/z00z_rollup_node/src/lib.rs`

### Live simulator anchors
- `crates/z00z_simulator/src/config.rs`
- `crates/z00z_simulator/src/design.rs`
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`
- `crates/z00z_simulator/src/scenario_1/runner.rs`
- `crates/z00z_simulator/src/scenario_1/runner_verify.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_utils/flow.rs`
- `crates/z00z_simulator/tests/test_scenario_settlement.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`

### Live bench and guardrail anchors
- `crates/z00z_storage/benches/settlement_hjmt.rs`
- `crates/z00z_storage/benches/settlement_shard.rs`
- `crates/z00z_storage/benches/settlement_nested.rs`
- `crates/z00z_storage/benches/settlement_proofs.rs`
- `crates/z00z_storage/tests/test_live_guardrails.rs`
- `crates/z00z_runtime/aggregators/tests/test_live_guardrails.rs`

</canonical_refs>

<ownership_map>

## 🧭 Cross-Crate Ownership Map

| Concern | Primary owner | Explicit non-owner rule |
| --- | --- | --- |
| Route-table bytes, digest, and committed lookup | `z00z_runtime/aggregators` | Storage, validators, watchers, and simulator must not become alternate planner truth owners. |
| Shard placement and standby metadata | `z00z_runtime/aggregators` | Placement is operational metadata, not storage truth and not checkpoint publication truth in this phase. |
| Semantic settlement execution and scope birth | `z00z_storage` | Runtime must hand off semantic work only and must not own subtree lifecycle or proof truth. |
| Durable journal and backend seam | `z00z_storage::backend` | Do not introduce a second durability seam or a shared WAL truth layer. |
| Service composition and dependency wiring | `z00z_rollup_node` | Storage and runtime crates must not become the composition root. |
| Simulator config, design sync, and artifact verification | `z00z_simulator` | Simulator evidence does not replace runtime or storage semantic truth. |

</ownership_map>

<plan_packet>

## ⚙️ Numbered Plan Packet

1. `056-01-PLAN.md` — topology, process model, and config-home freeze
2. `056-02-PLAN.md` — route-table codec, planner truth, and cross-shard reject
3. `056-03-PLAN.md` — semantic storage handoff and dynamic scope birth
4. `056-04-PLAN.md` — journal lineage, restart, and lawful failover
5. `056-05-PLAN.md` — YAML materialization and startup preflight
6. `056-06-PLAN.md` — simulator stage sync and runtime evidence
7. `056-07-PLAN.md` — fixtures, benchmarks, validation closeout, and final
   evidence sync

</plan_packet>

<todo_coverage_contract>

## ✅ TODO Coverage Contract

### Section-to-owner transfer

| `056-TODO.md` section | Packet owner | Coverage rule |
| --- | --- | --- |
| `Mission`, `Phase Boundary`, `This phase owns`, `This phase does not own`, `Phase handoff` | `056-CONTEXT.md` plus all numbered plans | Boundary language is frozen here and must stay consistent across every slice. |
| `Global upgrade rules active in every HJMT phase` | `056-CONTEXT.md` plus every plan `coverage_contract` | These rules remain normative for every execution slice, not only for the context. |
| `Inputs inherited from Phase 055 and consumed here` | `056-CONTEXT.md`, `056-02` through `056-07` | Phase 055 outputs are predecessor constraints, not optional historical notes. |
| `Primary upgrade sections owned by Phase 056` | `056-01` through `056-07` | Every numbered plan must cite the exact upgrade subsections it owns. |
| `Required cross-read sections` | `056-CONTEXT.md` plus every plan | These sections remain active authority for correctness, score claims, benchmarks, and readiness language. |
| `Fixture ownership for this phase` | `056-SOURCE-AUDIT.md`, `056-02`, `056-04`, `056-07` | Route, failover, and gap fixtures are execution-bound and cannot be dropped. |
| `Embedded audit contract` and `Audit-derived source rules` | `056-CONTEXT.md`, `056-02`, `056-04`, `056-05`, `056-06` | Planner truth, failover lawfulness, and runtime evidence rules remain live scope. |
| `Canonical SIM-5A7S runtime profile` | `056-01`, `056-04`, `056-06`, `056-07` | The fixture is acceptance authority, but the runtime must stay topology-generic. |
| `Process and config contract` | `056-01`, `056-05`, `056-06` | YAML ownership and separate OS process behavior are planned explicitly. |
| `Journal and WAL decision captured by this phase` | `056-04` | This decision is additive to the current RedB + `JournalBackend` seam and must stay fail-closed. |
| `Startup self-test gate` | `056-05` | Every listed failure mode must become explicit preflight behavior. |
| `Release blockers owned or co-owned by Phase 056` | all plans, summarized in `056-SOURCE-AUDIT.md` | Every blocker row must map to at least one numbered plan and one validation artifact. |
| `Phase-owned acceptance subset` | all plans, closed by `056-07` | The subset is the execution-wide acceptance contract. |
| `Mandatory implementation gates` `056-G1`..`056-G10` | gate map below | Every gate has one primary owner plan. |
| `Workstream 1` | `056-01` | Process topology and composition root. |
| `Workstream 2` | `056-02` | Routing, planner ownership, and cross-shard rejection. |
| `Workstream 3` | `056-03` | Storage handoff and dynamic scope birth. |
| `Workstream 4` | `056-04` | Journal, restart, and lawful failover. |
| `Workstream 5` | `056-05` | YAML configuration and startup preflight. |
| `Workstream 6` | `056-06` | Simulator stage sync and runtime evidence. |
| `Required tests and benchmark slices`, `Required execution profiles`, `Required scenario coverage`, `Required artifacts`, `Fixture ownership`, `Exit criteria` | `056-07` with dependencies on all previous plans | Final validation and evidence sync must close these cross-cutting sections without reopening ownership. |

### Gate-to-plan map

| Gate | Primary owner plan | Why |
| --- | --- | --- |
| `056-G1` Build canonical `SIM-5A7S` | `056-01` | The fixture and topology freeze must exist before route/failover work is trusted. |
| `056-G2` Separate OS process aggregators | `056-01` | Process model is a foundational topology rule. |
| `056-G3` YAML-driven runtime configuration | `056-05` | Config materialization and behavior change proof belong to the config slice. |
| `056-G4` Planner truth and planner-mode equivalence | `056-02` | Route-table and planner logic are runtime-owned. |
| `056-G5` Semantic runtime-to-storage handoff | `056-03` | The handoff contract is the core of the storage-boundary slice. |
| `056-G6` Dynamic scope birth | `056-03` | First-seen scope behavior is inseparable from semantic handoff. |
| `056-G7` Journal/WAL baseline and persistence contract | `056-04` | Journal lineage and restart semantics live here. |
| `056-G8` Lawful failover and split-brain fencing | `056-04` | Same-lineage failover and reject matrix live here. |
| `056-G9` Startup fail-closed preflight | `056-05` | Preflight is a config/startup gate, not a route-only concern. |
| `056-G10` Simulator as real runtime observability lane | `056-06` | Runtime traces and stage sync belong to the simulator slice. |

### Literal bullet preservation map

The following TODO bullet classes must remain explicit across this packet. They
are listed here so review does not rely on implication-only coverage.

#### `Canonical SIM-5A7S runtime profile` bullets

- Topology-generic acceptance rule: `056-01`, `056-05`, `056-07`
- Aggregators `AggregatorId(0)..AggregatorId(4)`: `056-01`
- Separate-OS-process model and isolated lifecycle/data dirs/logs/listeners:
  `056-01`, `056-06`
- Explicit `aggregator-config.yaml` path per process: `056-01`, `056-05`
- Explicit `planner-config.yaml` path per planner mode: `056-01`, `056-05`
- Explicit `storage-config.yaml` path per backend: `056-01`, `056-05`
- Shards `ShardId(0)..ShardId(6)`: `056-01`
- Placement spread with at least one dual-primary owner: `056-01`
- Every shard has at least one standby with expected lineage: `056-01`,
  `056-04`
- Gap-free route coverage, canonical order, and one `route_table_digest`:
  `056-02`, `056-07`
- Public-state handoff inputs for ascending `ShardRootLeafV1` publication:
  `056-04`, `056-05`, `056-07`
- Batch profiles `broad`, `hot-shard`, `hot-serial`, `delete-heavy`,
  `search-heavy`, `proof-heavy`, `mixed present or absent`, and rejected
  cross-shard: `056-02`, `056-03`, `056-07`
- Failure profiles `primary down`, `standby down`, `stale restart`,
  `wrong lineage`, `split-brain`, `route migration during crash`, and
  carry-forward publication handoff: `056-04`, `056-07`

#### `Required test coverage` bullets

- `test_hjmt_shard_routing.rs`: `056-02`, `056-07`
- `test_hjmt_topology.rs`: `056-01`, `056-07`
- `test_hjmt_process.rs`: `056-01`, `056-07`
- `test_hjmt_node_lifecycle.rs`: `056-01`, `056-07`
- Config-surface drift coverage lane: `056-05`, `056-07`
- `test_hjmt_planner.rs`: `056-02`, `056-07`
- `test_hjmt_preflight.rs`: `056-05`, `056-07`
- `test_hjmt_failover_same_lineage.rs`: `056-04`, `056-07`
- `test_hjmt_split_brain_fencing.rs`: `056-04`, `056-07`
- `test_hjmt_scope_birth.rs`: `056-03`, `056-07`
- Journal-baseline and WAL-boundary lane: `056-04`, `056-07`
- Restart and persistence lane: `056-04`, `056-07`
- Initial multi-aggregator simulation lane: `056-06`, `056-07`

#### `Required execution profiles` bullets

- `SIM-SMALL`: `056-06`, `056-07`
- `SIM-MEDIUM`: `056-06`, `056-07`
- `SIM-CACHE-EDGE`: `056-06`, `056-07`
- `SIM-BATCH-1000` reserved for later phases only and forbidden as default
  correctness profile here: `056-07`

#### `Required scenario coverage` bullets

- One-generation committed route lookup: `056-02`
- Stale/wrong generation reject: `056-02`, `056-04`
- Cross-shard batch reject at planner admission: `056-02`
- Central vs per-aggregator planner equivalence: `056-02`
- First-seen `definition_id` birth: `056-03`
- First-seen `serial_id` birth: `056-03`
- First terminal/right under routed shard: `056-03`
- Mixed existing-scope plus first-seen-scope batch: `056-03`
- Duplicate terminal id reject in one batch: `056-03`
- Path/leaf mismatch reject under live runtime admission: `056-03`
- Primary down, standby down, stale restart, wrong lineage, wrong generation,
  stale local root, and split-brain reject: `056-04`
- Crash after durable journal advance but before runtime ack: `056-04`
- Replay of the first scope-creating batch after restart: `056-03`, `056-04`
- Config change proof for ports, placement, planner mode, journal path,
  storage backend selection, and simulator scenario selection: `056-05`,
  `056-06`, `056-07`
- Deterministic `SIM-SMALL` and `SIM-MEDIUM`: `056-06`, `056-07`
- Capacity-relative `SIM-CACHE-EDGE`: `056-06`, `056-07`

#### `Required artifacts` bullets

- Aggregator YAML surface: `056-01`, `056-05`
- Planner YAML surface: `056-01`, `056-05`
- Storage YAML surface: `056-01`, `056-05`
- Live simulator config surface `scenario_config.yaml`: `056-01`, `056-05`,
  `056-06`
- Config digests for every used config surface: `056-05`, `056-06`, `056-07`
- `SIM-5A7S` fixture manifest: `056-01`, `056-07`
- Process ids, bound ports, data dirs, journal paths, log paths, startup
  commands, kill/restart commands, and explicit failover verdicts: `056-01`,
  `056-04`, `056-06`, `056-07`
- `route_table_digest`: `056-02`, `056-07`
- Trace artifacts `cfg_flow.json`, `tx_flow.json`, `route_flow.json`,
  `plan_flow.json`, `journal_flow.json`, `scope_flow.json`, `proc_flow.json`,
  and `recovery_flow.json`: `056-03`, `056-04`, `056-06`, `056-07`

#### `Fixture ownership` bullets

- Route fixtures `SRT-G-001` through `SRT-G-004`: `056-02`, `056-07`
- Route tamper fixtures `SRT-T-001` through `SRT-T-008`: `056-02`, `056-07`
- Failover fixtures `FOV-001`, `FOV-T-001`, and `FOV-T-002`: `056-04`,
  `056-07`
- Upgrade `12.1` gap fixtures `Route migration fixture` and `Failover fixture`:
  `056-02`, `056-04`, `056-07`

### Mandatory global cross-read before any implementation

From `docs/tech-papers/Z00Z-HJMT-Upgrade.md`:

- `Key Terms Used In This Paper`
- `1.1 Inherited Base Constraints`
- `1.2 Prohibited Changes`
- `1.3 Verified Current Baseline`
- `2.1 HJMT Remains The State Core`
- `2.2 Optimize Inside The Existing Paradigm`
- `2.3 Fail Closed`
- `2.4 Narrow Versioned Contracts`
- `2.5 Commitment Boundary`
- `2.6 Contract Discipline`
- `10. Correctness, Security, And Privacy Checklist`
- `10.1 Evidence Mapping Discipline`
- `13. Required Decisions And Fail-Closed Rules`
- `13.1 Fail-Closed Discipline`
- `14. Readiness Definition`
- `14.1 Completion Discipline`
- `Appendix A. Normative Upgrade Requirements`
- `Appendix E.4 Review Checklist For Implementation PRs`
- `Appendix E.5 Evidence Needed For Conformance-Safe Execution`

From `docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md`:

- `Completion Contract`
- `Release Gate`

### Mandatory Phase 056 cross-read that every numbered plan must keep active

- `5. Upgrade 3: Stable Shard Layer Above Buckets`
- `5.1 Concepts`
- `5.2 Routing Model`
- `5.2.1 Exact Codec Contract For ShardRouteTableV1`
- `5.2.2 Protocol Truth Versus Runtime Placement`
- `5.3 Shard Split And Migration`
- `5.4 Per-Shard Journal And Queue Rules`
- `5.4.1 Planner Ownership And Batch Formation`
- `5.4.2 Distributed-Tree Design Tradeoff And Aggregator Failure Domain`
- `5.4.3 Runtime Placement Objects And Lawful Failover`
- `5.4.4 C4 Dynamic View: Lawful Failover Versus Silent Reroute`
- `5.5 Routing Safety Requirements`
- `5.6 Implementation Guidance`
- `Appendix D.3 Shard Route And Root Skeleton`
- `Appendix D.6 Runtime Placement And Storage Boundary Skeletons`
- `Appendix E.6 Cross-Crate Module Ownership`
- `Appendix E.7 Cross-Crate Execution Order`
- `1.7 Whole-System Structure View`
- `6.1 Root Generations`
- `6.4 Compatibility`
- `9.1 Benchmark Matrix`
- `9.2 Claim Gate`
- `9.3 Score Claim Discipline`
- `12. Test And Benchmark Plan`
- `12.1 Evidence Gaps`

### Review discipline

- Every dash-list bullet class under the covered TODO sections remains
  normative, not decorative.
- No numbered plan may silently drop a bullet class as "implied by the
  section".
- If a plan depends on a proposed path or file target, it must label that fact
  explicitly instead of claiming the path already exists.

</todo_coverage_contract>
