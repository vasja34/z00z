# Phase 057: HJMT Multi Aggregator - Context

**Gathered:** 2026-06-13
**Status:** ready-for-execution-planning
**Source:** deep read of `057-TODO.md`,
`docs/tech-papers/Z00Z-HJMT-Upgrade.md`,
`docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md`,
`docs/tech-papers/Z00Z-HJMT-Design.md`, the completed Phase 056 packet, and
the live runtime, storage, validator, watcher, node, and simulator seams that
already own routing lineage, semantic settlement, and runtime evidence

<domain>

## 🎯 Phase Boundary

Phase 057 is the publication packet above the executable multi-aggregator
runtime from Phase 056. It consumes the lawful route, planner, journal, scope,
and process lineage that Phase 056 already made executable and turns that
lineage into public checkpoint truth.

Phase 057 owns:

- executable pre-shard versus post-shard root-generation behavior;
- one exact `ShardRootLeafV1` contract;
- one exact `CheckpointPublicationV1` contract;
- ordered public leaf-set bindings, monotonicity, and prior-root continuity;
- layered public proof composition above shard-local proof truth;
- join-as-standby versus join-as-owner publication behavior;
- route-generation-bound shard transfer at the publication boundary;
- byte-identical carry-forward under partial failure;
- validator and watcher acceptance of the same publication digest;
- public continuity for first-seen scope birth without a second registry.

Phase 057 must extend the current live seams in place. It must not create:

- a second planner authority beside `crates/z00z_runtime/aggregators`;
- a second semantic or proof authority beside `crates/z00z_storage`;
- a second publication digest story beside the canonical checkpoint object;
- a validator-local or watcher-local reinterpretation of public truth;
- a second simulator evidence lane beside `scenario_1`.

Phase 057 must also preserve the already-closed Phase 056 runtime contract:
publication is a new boundary, not permission to reopen runtime routing truth,
planner truth, or storage truth.

</domain>

<review_findings>

## 🚨 Review Findings That Forced This Packet Rewrite

### S1-HIGH: The phase had authority but no executable planning packet
- The folder contained `057-TODO.md` and no numbered `057-*-PLAN.md` files.
- There was no `057-CONTEXT.md` or `057-SOURCE-AUDIT.md`.
- Result: root-generation, publication, join, transfer, and downstream
  acceptance scope could drift during execution.

### S1-HIGH: The TODO mixes verified live paths with phase-owned contract names
- `aggregator-config.yaml`, `planner-config.yaml`, `storage-config.yaml`,
  `leaf_flow.json`, `proof_flow.json`, `pub_flow.json`, `val_flow.json`,
  `watch_flow.json`, and `test_hjmt_multi_aggregator_sim.rs` are contract
  names unless exact repo paths are given.
- The verified live simulator anchors today are still
  `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`,
  `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`,
  `crates/z00z_simulator/tests/test_scenario_settlement.rs`, and
  `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`.
- Result: the packet must keep proposed homes labeled honestly until the
  execution slices land them.

### S1-HIGH: Concept drift is likely if publication, proof, validator, and watcher roles blur
- `crates/z00z_runtime/aggregators/src/service.rs` already owns the runtime
  publication request/record seam.
- `crates/z00z_storage/src/settlement/*` already owns committed shard-local
  roots and proof truth.
- `crates/z00z_runtime/validators` and `crates/z00z_runtime/watchers` already
  consume published state from downstream seams.
- Result: the packet must freeze the owner map before adding publication logic.

### S2-MEDIUM: Current publication surfaces are intentionally narrow
- `PublicationRequest`, `PublishedBatch`, and `PublicationRecord` exist today,
  but they do not yet prove exact `ShardRootLeafV1` or
  `CheckpointPublicationV1` contracts.
- `z00z_rollup_node/src/config.rs` already checks publication-handoff coverage,
  generation, route digest, and shard ordering, which makes it a strong live
  anchor for Phase 057 rather than a blank slate.
- Result: the packet must extend narrow live seams instead of inventing a new
  publication subsystem.

### S2-MEDIUM: Runtime evidence exists, but publication evidence does not yet have a frozen packet
- Phase 056 already emits or verifies `cfg_flow.json`, `tx_flow.json`,
  `route_flow.json`, `plan_flow.json`, `journal_flow.json`, `scope_flow.json`,
  `proc_flow.json`, and `recovery_flow.json`.
- Phase 057 must add `leaf_flow.json`, `proof_flow.json`, `pub_flow.json`,
  `val_flow.json`, and `watch_flow.json` without detaching them from the Phase
  056 lineage.
- Result: scenario and trace sync need their own explicit slices.

</review_findings>

<decisions>

## 🔑 Implementation Decisions

### D-01: `057-TODO.md` remains the canonical backlog and source authority
- This context, the source audit, and the numbered plans schedule the TODO.
- They do not replace it.
- If `057-TODO.md` changes, refresh this file, `057-SOURCE-AUDIT.md`, and
  every numbered plan in the same change set.

### D-02: No duplicate authority layer and no concept drift
- Extend the current runtime, storage, validator, watcher, node, and simulator
  seams in place.
- Do not create a second publication registry, second planner digest path, or
  second proof truth path.
- Do not promote validator or watcher interpretations into alternate protocol
  truth.

### D-03: The live owner map is frozen before execution
- `crates/z00z_runtime/aggregators` owns route lineage, placement lineage,
  runtime publication-request composition, and publication continuity at the
  runtime boundary.
- `crates/z00z_storage/src/settlement/*` owns committed shard roots,
  shard-local proof truth, and scope birth semantics.
- `crates/z00z_rollup_node` owns composition, startup wiring, and publication
  handoff validation.
- `crates/z00z_runtime/validators` owns validator acceptance of published
  artifacts.
- `crates/z00z_runtime/watchers` owns exported operational evidence and alerts
  over published artifacts.
- `crates/z00z_simulator` owns the executable scenario and evidence packet.

### D-04: Publication truth is one canonical ordered leaf set and one digest story
- `ShardRootLeafV1` is the only allowed public leaf contract for this phase.
- `CheckpointPublicationV1` is the only allowed public checkpoint object for
  this phase.
- Public truth requires canonical ascending `ShardId` order, one digest story,
  and one prior-root continuity story.
- Placement metadata remains operational; it does not become a second protocol
  truth source.

### D-05: Root-generation behavior must stay explicit
- Phase 057 must distinguish pre-shard and post-shard generations explicitly.
- Compatibility coverage must bridge the last pre-shard root, the first shard
  route table, the first shard leaf, and the first root-of-shard-roots
  publication.
- Generation confusion must reject fail-closed.

### D-06: Layered proof composition remains mandatory
- Shard-local proofs remain the semantic truth component.
- Publication adds a public shard-leaf inclusion layer.
- Historical proof continuity must remain valid only when protocol lineage
  says it should; publication must not flatten the proof family into one
  ambiguous object.

### D-07: `SIM-5A7S-PUB` is a canonical acceptance fixture only
- `SIM-5A7S-PUB` is the mandatory acceptance profile for this phase.
- The implementation must still accept any positive topology transition loaded
  from YAML when route-generation, lineage, and publication invariants hold.
- No code path may silently hard-code `5x7` as the only supported public
  topology.

### D-08: Join-as-standby and join-as-owner are distinct protocol states
- Standby join mirrors lineage and prepares takeover capacity only.
- Owner activation requires committed route generation `N+1` plus one explicit
  activation checkpoint.
- Pre-activation publication as owner must reject.

### D-09: Shard transfer is route-generation-bound
- Ownership changes are lawful only through committed route-table generation
  changes.
- Transfer evidence must include old route table, new route table, activation
  checkpoint, continuity verdicts, and historical-proof continuity.
- Silent reroute remains forbidden.

### D-10: Carry-forward must preserve unchanged shard-leaf bytes exactly
- Mid-window owner failure is contained by carrying forward the failed shard's
  last lawful `ShardRootLeafV1`.
- Unaffected shards may publish new leaves.
- Changed bytes for an unchanged carried-forward leaf are a hard failure.

### D-11: Dynamic scope birth stays storage-owned but must surface through lawful publication
- First-seen `definition_id` and `serial_id` birth remains storage-owned.
- Phase 057 must prove that the touched shard root, touched shard leaf, and
  touched checkpoint digest change lawfully without any extra public
  registration lane.
- Publication begins from committed shard-local results only.

### D-12: Validators and watchers consume the same publication contract
- Validators accept only canonical leaf sets, bindings, and checkpoint
  continuity.
- Watchers export evidence for the same digest and same verdict mapping.
- Neither layer may derive a second local checkpoint truth.

### D-13: Simulator traces are evidence, not semantic truth
- `leaf_flow.json`, `proof_flow.json`, `pub_flow.json`, `val_flow.json`, and
  `watch_flow.json` describe execution outcomes only.
- They must resolve back to Phase 056 `cfg_flow.json`, `tx_flow.json`,
  `route_flow.json`, `plan_flow.json`, `journal_flow.json`, `scope_flow.json`,
  `proc_flow.json`, and `recovery_flow.json`.
- Missing, stale, or cross-linked traces fail the phase.

### D-14: Live YAML anchors and proposed publication homes must stay separated
- Verified live YAML anchors today are
  `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`,
  `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`, and the checked
  in runtime home under `config/hjmt_runtime/`.
- Verified live storage bench homes today are
  `crates/z00z_storage/benches/settlement_hjmt.rs`,
  `crates/z00z_storage/benches/settlement_shard.rs`,
  `crates/z00z_storage/benches/settlement_nested.rs`, and
  `crates/z00z_storage/benches/settlement_proofs.rs`.
- `aggregator-config.yaml`, `planner-config.yaml`, and
  `storage-config.yaml` resolve through the checked-in runtime home under
  `config/hjmt_runtime/`.
- `leaf_flow.json`, `proof_flow.json`, `pub_flow.json`, `val_flow.json`, and
  `watch_flow.json` are verified on the live simulator path: filenames are
  declared in `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`,
  emitted by `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`,
  and rechecked by `crates/z00z_simulator/tests/test_scenario_settlement.rs`
  plus `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`.
- The old TODO contract name `test_hjmt_multi_aggregator_sim.rs` resolves to
  `crates/z00z_simulator/tests/test_scenario_settlement.rs` and
  `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`.
- No planning artifact may claim a proposed path is already live.

### D-15: `scenario_design.yaml` must stay synchronized with executable stage behavior
- If `scenario_1` stage boundaries or publication flows change, update
  `scenario_design.yaml` in the same slice.
- Do not accept a hidden publication lane that is not described by the design
  packet.

### D-16: The canonical packet began as six ordered execution slices and is
explicitly renormalized to a seventh closeout continuation when post-closeout
authority work lands
- `057-01`: root-generation behavior plus canonical `ShardRootLeafV1` and
  `CheckpointPublicationV1` contracts.
- `057-02`: public proof composition and historical compatibility.
- `057-03`: `SIM-5A7S-PUB` publication integration and publication trace
  packet.
- `057-04`: join, transfer, carry-forward, and crash recovery.
- `057-05`: validator/watcher binding, dynamic-scope continuity, and scenario
  sync.
- `057-06`: fixtures, benchmarks, validation closeout, and planning-state
  sync.
- `057-07`: canonical publication-binding guardrails, post-closeout
  authority-drift closure, and renormalized ledger sync.

### D-17: Mandatory verification order applies to every Rust or test-affecting slice
- Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  first.
- If bootstrap fails, stop, fix the regression, and rerun bootstrap before any
  broader validation.
- Then run slice-local targeted tests.
- Then run `cargo test --release` as the broad corroborating gate.
- Then run `/.github/prompts/gsd-review-tasks-execution.prompt.md`
  (`/GSD-Review-Tasks-Execution`) in YOLO mode at least 3 times and continue
  until at least 2 consecutive runs show no significant issues.

### D-18: Commit and review flows must use the repository-local tooling packet
- If a slice needs a git version or release-flow commit, use
  `/z00z-git-versioning`.
- Nested prompts, skills, scripts, and instructions must resolve from
  `.github/`.
- Keep Phase 057 planning and execution local to the repository packet; do not
  route through external graph or generic workflows when the local packet
  already exists.

### D-19: Proposed targets must be labeled honestly
- If a file, fixture home, trace home, or report schema is not yet verified in
  the current tree, label it as proposed.
- Do not turn TODO contract names into false repository facts.

### D-20: Phase 057 is not complete until the full exit criteria close
- All gates `057-G1` through `057-G11`.
- All required fixtures, traces, execution profiles, and benchmark lanes.
- All route, root, checkpoint, join, migration, carry-forward, validator,
  watcher, and scope-continuity evidence.
- Final readiness and release claims remain Phase 058 handoff work; the Phase
  057 closeout slice must not overclaim them.

</decisions>

<canonical_refs>

## 📚 Canonical References

### Phase authority
- `.planning/phases/057-HJMT-multi-aggregator/057-TODO.md`
- `.planning/phases/057-HJMT-multi-aggregator/057-CONTEXT.md`
- `.planning/phases/057-HJMT-multi-aggregator/057-SOURCE-AUDIT.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`

### Normative docs
- `docs/tech-papers/Z00Z-HJMT-Upgrade.md`
- `docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md`
- `docs/tech-papers/Z00Z-HJMT-Design.md`

### Predecessor packet anchors
- `.planning/phases/056-HJMT-storage- aggregator/056-CONTEXT.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-SOURCE-AUDIT.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-TEST-SPEC.md`
- `.planning/phases/055-HJMT-boundary/055-CONTEXT.md`

### Live runtime anchors
- `crates/z00z_runtime/aggregators/README.md`
- `crates/z00z_runtime/aggregators/src/lib.rs`
- `crates/z00z_runtime/aggregators/src/batch_planner.rs`
- `crates/z00z_runtime/aggregators/src/placement.rs`
- `crates/z00z_runtime/aggregators/src/recovery.rs`
- `crates/z00z_runtime/aggregators/src/scheduler.rs`
- `crates/z00z_runtime/aggregators/src/service.rs`
- `crates/z00z_runtime/aggregators/src/shard_exec.rs`
- `crates/z00z_runtime/aggregators/src/types.rs`

### Live storage anchors
- `crates/z00z_storage/src/settlement/README.md`
- `crates/z00z_storage/src/settlement/root_types.md`
- `crates/z00z_storage/src/settlement/store.rs`
- `crates/z00z_storage/src/settlement/hjmt_store.rs`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs`
- `crates/z00z_storage/src/settlement/hjmt_proof.rs`
- `crates/z00z_storage/src/settlement/proof.rs`
- `crates/z00z_storage/src/settlement/proof_batch.rs`
- `crates/z00z_storage/src/settlement/proof_batch_verify.rs`
- `crates/z00z_storage/src/settlement/test_live_recovery.rs`

### Live node anchors
- `crates/z00z_rollup_node/src/config.rs`
- `crates/z00z_rollup_node/src/runtime.rs`
- `crates/z00z_rollup_node/src/status.rs`
- `crates/z00z_rollup_node/src/da.rs`
- `crates/z00z_rollup_node/tests/support/test_hjmt_home.rs`

### Live validator and watcher anchors
- `crates/z00z_runtime/validators/src/checkpoint.rs`
- `crates/z00z_runtime/validators/src/engine.rs`
- `crates/z00z_runtime/validators/src/verdict.rs`
- `crates/z00z_runtime/watchers/README.md`
- `crates/z00z_runtime/watchers/src/publication.rs`
- `crates/z00z_runtime/watchers/src/evidence_export.rs`
- `crates/z00z_runtime/watchers/src/engine.rs`
- `crates/z00z_runtime/watchers/src/status.rs`

### Live simulator anchors
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
- `crates/z00z_simulator/src/scenario_1/runner.rs`
- `crates/z00z_simulator/src/scenario_1/runner_verify.rs`
- `crates/z00z_simulator/tests/test_scenario_settlement.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`

### Live bench and guardrail anchors
- `crates/z00z_storage/benches/settlement_shard.rs`
- `crates/z00z_storage/benches/settlement_hjmt.rs`
- `crates/z00z_storage/benches/settlement_nested.rs`
- `crates/z00z_storage/benches/settlement_proofs.rs`
- `crates/z00z_storage/tests/test_live_guardrails.rs`
- `crates/z00z_runtime/aggregators/tests/test_live_guardrails.rs`

</canonical_refs>

<ownership_map>

## 🧭 Cross-Crate Ownership Map

| Concern | Primary owner | Explicit non-owner rule |
| --- | --- | --- |
| Route-table bytes, route digest, routing generation, and planner lineage | `z00z_runtime/aggregators` | Storage, validators, watchers, and simulator must not become alternate route or planner truth owners. |
| Committed shard-local roots and shard-local proof truth | `z00z_storage` | Runtime publication, validators, and watchers must consume these outputs and must not recompute them as alternate truth. |
| Runtime publication request assembly and publication continuity state | `z00z_runtime/aggregators` | Storage must not become the publication orchestrator, and watchers must not become the digest authority. |
| Publication handoff validation and composition root wiring | `z00z_rollup_node` | Runtime, storage, validators, and simulator must not become a second node-composition root. |
| Validator acceptance and verdict mapping | `z00z_runtime/validators` | Validators must not widen into route truth or simulator truth ownership. |
| Watcher export, observation, and alert surfaces | `z00z_runtime/watchers` | Watchers must not derive a second local publication contract. |
| Scenario config, stage sync, and trace verification | `z00z_simulator` | Simulator evidence must not replace runtime, storage, or publication semantic truth. |

</ownership_map>

<plan_packet>

## ⚙️ Numbered Plan Packet

1. `057-01-PLAN.md` — root generation, shard leaf, and checkpoint publication
   contracts
2. `057-02-PLAN.md` — two-layer proof composition and historical
   compatibility
3. `057-03-PLAN.md` — `SIM-5A7S-PUB` publication integration and trace packet
4. `057-04-PLAN.md` — join, transfer, carry-forward, and crash recovery
5. `057-05-PLAN.md` — validator/watcher binding, scope continuity, and
   scenario sync
6. `057-06-PLAN.md` — fixture, benchmark, validation, and planning-state
   closeout
7. `057-07-PLAN.md` — canonical-authority guardrails and renormalized closeout
   sync

</plan_packet>

<todo_coverage_contract>

## ✅ TODO Coverage Contract

### Section-to-owner transfer

| `057-TODO.md` section | Packet owner | Coverage rule |
| --- | --- | --- |
| `Mission`, `Phase Boundary`, `This phase owns`, `This phase does not own`, `Phase handoff` | `057-CONTEXT.md` plus all numbered plans | Boundary language is frozen here and must remain consistent across every slice. |
| `Global upgrade rules active in every HJMT phase` | `057-CONTEXT.md` plus every plan `coverage_contract` | These rules remain normative for every execution slice. |
| `Inputs inherited from Phase 056 and consumed here` | `057-CONTEXT.md`, `057-01` through `057-06` | Phase 056 outputs are predecessor constraints, not optional historical notes. |
| `Primary upgrade sections owned by Phase 057` | `057-01` through `057-06` | Every numbered plan must cite the exact upgrade subsections it owns. |
| `Required cross-read sections` | `057-CONTEXT.md` plus every plan | These sections stay active authority for correctness, benchmark, and readiness language. |
| `Fixture ownership for this phase` | `057-SOURCE-AUDIT.md`, `057-01`, `057-02`, `057-04`, `057-06` | Shard-root, checkpoint-publication, and carry-forward fixture rows are execution-bound and cannot be dropped. |
| `Embedded audit contract` | `057-CONTEXT.md`, `057-01`, `057-02`, `057-03`, `057-04`, `057-05`, `057-06` | Silent-reroute bans, canonical ordered publication, and compatibility rows remain live scope. |
| `Canonical SIM-5A7S-PUB publication profile` | `057-03`, `057-04`, `057-05`, `057-06` | The fixture is acceptance authority, but the implementation must stay topology-generic. |
| `Release blockers owned or co-owned by Phase 057` | all plans, summarized in `057-SOURCE-AUDIT.md` | Every blocker row must map to at least one numbered plan and one validation artifact, including the co-owned runtime on/off matrix join-or-leave row. |
| `Phase-owned acceptance subset` | all plans, closed by `057-06` and rechecked by `057-07` | The subset is the execution-wide acceptance contract. |
| `Mandatory implementation gates` `057-G1`..`057-G11` | gate map below | Every gate has one primary owner plan. |
| `Workstream 1` | `057-01` | Root generations and publication object contracts. |
| `Workstream 2` | `057-02` | Public proof composition and compatibility. |
| `Workstream 3` | `057-04` | Join, transfer, and carry-forward. |
| `Workstream 4` | `057-05` | Dynamic-scope publication continuity. |
| `Workstream 5` | `057-03`, `057-05` | Publication integration plus validator, watcher, and scenario sync. |
| `Required tests and benchmark slices`, `Required execution profiles`, `Required scenario coverage`, `Required artifacts`, `Fixture ownership`, `Exit criteria` | `057-06` with dependencies on all previous plans plus `057-TEST-SPEC.md` and `057-TESTS-TASKS.md`, then `057-07` for continuation anti-drift and ledger honesty | Final validation and evidence sync must close these cross-cutting sections without reopening ownership or claiming Phase 058 readiness early. |

### Gate-to-plan map

| Gate | Primary owner plan | Why |
| --- | --- | --- |
| `057-G1`, `057-G2`, and `057-G3` | `057-01` | Root-generation behavior and both canonical publication objects must freeze together before any later slice is honest. |
| `057-G4` | `057-02` | Layered proof truth and historical continuity live here. |
| `057-G5` | `057-03` | The real 5x7 runtime lineage becomes the `SIM-5A7S-PUB` public checkpoint lane here. |
| `057-G6`, `057-G7`, and `057-G8` | `057-04` | Join, route-generation transfer, carry-forward, and crash containment are one transition packet. |
| `057-G9`, `057-G10`, and `057-G11` | `057-05` | Validator/watcher sameness, scope continuity, and lineage-trace honesty close together. |
| Final gate-to-evidence matrix | `057-06`, rechecked by `057-07` | The closeout slice proves every gate, fixture class, profile, and artifact together before the ledgers move, and the continuation slice keeps the final ledger story honest. |

### Literal bullet preservation map

The following TODO bullet classes must remain explicit across this packet.
Review must not rely on implication-only coverage.

| Bullet class | Explicit packet owners |
| --- | --- |
| Canonical root-generation, `ShardRootLeafV1`, `CheckpointPublicationV1`, monotonicity, and prior-root continuity | `057-01`, closed by `057-06` |
| Two-layer proof composition, worked examples `6.8.1` and `6.8.2`, and `6.8.3` reject continuity | `057-02`, closed by `057-06` |
| `SIM-5A7S-PUB`, topology-generic YAML proof, publication digest, and `leaf_flow.json` or `proof_flow.json` or `pub_flow.json` trace surfaces | `057-03`, closed by `057-06` |
| Join-as-standby, join-as-owner, route-generation transfer, byte-identical carry-forward, and crash recovery | `057-04`, closed by `057-06` |
| Validator/watcher digest sameness, `val_flow.json`, `watch_flow.json`, first-seen scope continuity, and `scenario_design.yaml` sync | `057-05`, closed by `057-06` |
| YAML-defined old/new topology, route-generation boundaries, join mode, transfer target, owner/standby roles, and publication activation point | `057-03`, `057-05`, closed by `057-06` |
| Config digests, process ids, journal paths, owner/standby assignments, process exit/restart verdicts, and inherited runtime on/off matrix rows | `057-03`, `057-04`, `057-05`, closed by `057-06` |
| Exit-criteria carryover for executable contracts, canonical seven-leaf publication, distinct join states, committed route generations, exact carry-forward, scenario/design synchronization, and no detached publication artifacts | `057-01`, `057-03`, `057-04`, `057-05`, closed by `057-06` |
| Required tests, benchmarks, execution profiles, fixtures, artifacts, and final exit criteria | `057-06`, rechecked by `057-07` |
| TODO contract-name honesty and verified live anchors for simulator config and the four current storage bench homes | `057-CONTEXT.md`, `057-SOURCE-AUDIT.md`, `057-03`, `057-06` |

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

From `docs/tech-papers/Z00Z-HJMT-Design.md`:

- `5.2 Root Taxonomy`
- `8.3 Checkpoint Evidence`
- `9.2 Benchmark Plan`
- `13. Testing And Verification Strategy`
- `13.3 Proof Tests`

### Mandatory Phase 057 cross-read that every numbered plan must keep active

- `5.2.1 Exact Codec Contract For ShardRouteTableV1`
- `5.2.2 Protocol Truth Versus Runtime Placement`
- `5.3 Shard Split And Migration`
- `5.4.2 Distributed-Tree Design Tradeoff And Aggregator Failure Domain`
- `5.4.3 Runtime Placement Objects And Lawful Failover`
- `5.4.4 C4 Dynamic View: Lawful Failover Versus Silent Reroute`
- `6. Upgrade 4: Root-Of-Shard-Roots Publication`
- `6.1 Root Generations`
- `6.1.1 Exact Codec Contract For ShardRootLeafV1`
- `6.1.2 policy_set_digest Semantics`
- `6.1.3 Naming Rule For policy_digest`
- `6.2 Proof Composition`
- `6.2.1 Worked Example: Two-Layer Public Membership Proof`
- `6.3 Publication Cadence`
- `6.3.1 Canonical Checkpoint Publication Object`
- `6.3.2 Monotonicity Rules`
- `6.3.3 Protection, Failure Containment, And Recovery Layers`
- `6.3.4 Operational Summary And Source-Of-Truth Rules`
- `6.4 Compatibility`
- `6.5 Root Publication Requirements`
- `6.6 Implementation Guidance`
- `6.7 C4 Component View: Shard Root Publication`
- `6.8 Worked Example: Three Aggregators, Eight Assets, One Shard-Local Batch`
- `6.8.1 Shard-Local Trees Under One Public Root`
- `6.8.2 One Batch Inside Shard B`
- `6.8.4 Worked Lifecycle: Adding Aggregator D`
- `6.8.5 Worked Timeline: Aggregator B Fails Mid-Window`
- `Appendix D.3 Shard Route And Root Skeleton`
- `Appendix D.5 Stable Path And Asset Leaf Proof Skeleton`
- `Appendix E.6 Cross-Crate Module Ownership`
- `Appendix E.7 Cross-Crate Execution Order`
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
