---
phase: 057-HJMT-multi-aggregator
artifact: tests-tasks
status: execution-backed
source:
  - 057-TEST-SPEC.md
  - 057-TODO.md
  - 057-CONTEXT.md
  - 057-01-PLAN.md
  - 057-02-PLAN.md
  - 057-03-PLAN.md
  - 057-04-PLAN.md
  - 057-05-PLAN.md
  - 057-06-PLAN.md
  - 057-07-PLAN.md
  - 057-07-SUMMARY.md
updated: 2026-06-14
---

# Phase 057 Test Tasks

**Phase:** `057-HJMT-multi-aggregator`
**Status:** execution-backed testing artifact
**Companion spec:** `057-TEST-SPEC.md`

## Goal

This file turns the Phase 057 packet into an engineer-ready test work order.
It exists so execution can proceed without guessing:

- which live file home owns each publication scenario;
- which fixtures, traces, and config surfaces are required;
- which assertions prove correctness;
- which join, transfer, carry-forward, and proof rows must reject;
- which execution profiles and bench lanes must exist;
- which anti-drift checks must block duplicate authority layers.

## Scope Inputs

- `057-TODO.md` is the canonical source for required tests, benchmarks,
  fixture ids, scenario rows, and exit criteria.
- `057-CONTEXT.md` is the canonical source for decisions `D-02` through
  `D-16`, especially no duplicate authority layers, one digest story,
  topology-generic YAML control, and trace lineage honesty.
- `057-TEST-SPEC.md` freezes the scenario ledger, realistic examples,
  cryptographic invariants, and pass/fail oracles for implementation.
- `057-01-PLAN.md` through `057-07-PLAN.md` freeze the execution order and the
  narrow owner homes for each slice.
- Live anchors already exist in current storage, runtime, node, validator,
  watcher, simulator, and bench seams; proposed homes are marked explicitly in
  the companion spec.

## Execution Strategy

- This artifact is sequenced to match the six ordered execution core slices
  frozen in `057-01-PLAN.md` through `057-06-PLAN.md`, plus the explicit
  `057-07` post-closeout continuation that re-pins one canonical
  publication-binding path.
- Wave 1 must land before Wave 2 because proof composition is not trustworthy
  until leaf and checkpoint bytes are frozen.
- Wave 2 must land before Wave 3 because `SIM-5A7S-PUB` evidence is not honest
  until publication consumes the real layered proof surface.
- Wave 3 must land before Wave 4 because join, transfer, and carry-forward
  tests need one live publication lane and one trace packet to build on.
- Wave 4 must land before Wave 5 because validators and watchers must consume
  the canonical publication contract, not an intermediate transition draft.
- Wave 5 must land before Wave 6 because bench closeout and guardrail closure
  require the full publication digest story to be present.
- Wave 6 must land before Wave 7 because `057-07` only closes after the
  guardrail, bench, and evidence matrix are already green and the packet can
  be renormalized honestly instead of creating a parallel closeout lane.

## Hard Rules

- Reuse existing live owner homes whenever they already own the behavior.
- Do not create a second publication registry, second proof engine, second
  digest lane, second validator truth lane, second watcher truth lane, or
  second simulator evidence lane.
- Do not treat `SIM-5A7S-PUB` as the only supported topology; every test wave
  that touches config must preserve topology-generic YAML loading from the
  inherited `SIM-5A7S` packet, including lawful `aggregator_count > 0`,
  `shard_count > 0`, and `old_aggregator_count -> new_aggregator_count`
  transitions.
- Keep public technical content in English.
- When a commit is needed, use `/z00z-git-versioning`.

## Verify Block Template

Every Rust or test-affecting change in Phase 057 must verify in this order:

```bash
./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh
```

If bootstrap fails, stop, fix the regression, and rerun bootstrap before any
broader validation.

Then run the wave-specific targeted commands below, and close with:

```bash
cargo test --release
```

After the targeted commands, run
`/.github/prompts/gsd-review-tasks-execution.prompt.md`
(` /GSD-Review-Tasks-Execution `) in YOLO mode at least three times and
continue until at least two consecutive runs report no significant issues.

## Scenario To Plan Crosswalk

| Plan slice | Test scenarios from `057-TEST-SPEC.md` | Main owner homes |
| --- | --- | --- |
| `057-01` | `057-SC-01`, `057-SC-02` | `test_hjmt_root_generation.rs`, `test_hjmt_publish.rs` |
| `057-02` | `057-SC-03`, `057-SC-04` | `test_hjmt_historical_proofs.rs`, proof-family anchors, validator acceptance suite |
| `057-03` | `057-SC-05`, `057-SC-06` | `test_hjmt_runtime_config.rs`, `test_scenario_settlement.rs`, `test_scenario1_stage_surface.rs`, `test_hjmt_publish.rs` |
| `057-04` | `057-SC-07`, `057-SC-08`, `057-SC-09`, `057-SC-10` | `test_hjmt_join.rs`, `test_hjmt_migrate.rs`, `test_hjmt_preflight.rs`, failover and recovery seams |
| `057-05` | `057-SC-11` | validator and watcher publication suites, `test_hjmt_scope_birth.rs`, simulator traces |
| `057-06` | `057-SC-12`, `057-SC-13` | guardrails, `test_bench_lanes.rs`, storage bench homes |
| `057-07` | `057-SC-12`, `057-SC-13` continuation | live guardrails plus packet sync on `057-TEST-SPEC.md` and `057-TESTS-TASKS.md` |

## Wave 1: `057-01` Root Generations And Publication Objects

**Purpose:** freeze the exact root-generation, leaf, and checkpoint test
surface before later slices depend on it.

**Files to create or extend**

- `crates/z00z_storage/tests/test_hjmt_root_generation.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_publish.rs`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_root_leaf_v1/`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/checkpoint_publication_v1/`

**Implementation tasks**

- Create `SRL-G-*` and `SRL-T-*` vectors from live code, not handwritten byte
  blobs.
- Assert explicit pre-shard versus post-shard generation handling and reject
  confusion rows fail-closed.
- Assert exact `ShardRootLeafV1` field ordering, route binding,
  `policy_set_digest` semantics, and `policy_digest` naming discipline through
  canonical bytes and digest equality.
- Assert exact `CheckpointPublicationV1` bytes, canonical ascending `ShardId`
  ordering, prior-root continuity, and monotonicity.
- Tie every positive vector to one regeneration command, one expected digest,
  and one expected verdict.

**Success conditions**

- `057-SC-01` and `057-SC-02` pass with stable bytes, stable digests, and
  explicit tamper rejects.
- `057-G1`, `057-G2`, and `057-G3` are closed without reopening planner or
  storage ownership boundaries.

**Targeted commands**

```bash
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_root_generation -- --nocapture
cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_publish -- --nocapture
```

## Wave 2: `057-02` Public Proof Composition And Compatibility

**Purpose:** prove publication stays layered above shard-local proof truth and
keeps historical compatibility semantics.

**Files to create or extend**

- `crates/z00z_storage/tests/test_hjmt_historical_proofs.rs`
- `crates/z00z_storage/tests/test_hjmt_live_proof_families.rs`
- `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`

**Implementation tasks**

- Reconstruct the worked two-layer public membership proof from Upgrade
  `6.2.1`, `6.8.1`, and `6.8.2`.
- Assert publication proof remains additive above shard-local proof truth, not
  a replacement for it.
- Add wrong-shard, wrong-lineage, cross-shard, and historical-compatibility
  negative rows that prove the `6.8.3` counterexample rejects.
- Keep Phase 055 proof-family behavior visible through the current proof-family
  anchors instead of creating a second verifier.
- Add validator-side acceptance rows that confirm downstream verification
  depends on the same canonical proof and checkpoint surfaces.

**Success conditions**

- `057-SC-03` and `057-SC-04` pass.
- `057-G4` is closed with both positive compatibility evidence and explicit
  reject rows.

**Targeted commands**

```bash
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_historical_proofs -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_live_proof_families -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof -- --nocapture
cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_stage8_proof_path -- --nocapture
```

## Wave 3: `057-03` `SIM-5A7S-PUB` Integration And Trace Packet

**Purpose:** make the real Phase 056 runtime packet publish canonical public
checkpoint truth and emit a truthful trace pack.

**Files to create or extend**

- `crates/z00z_simulator/tests/test_hjmt_runtime_config.rs`
- `crates/z00z_simulator/tests/test_scenario_settlement.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_publish.rs`

**Implementation tasks**

- Extend the current runtime packet into `SIM-5A7S-PUB` with seven active
  leaves in canonical order and one canonical publication digest.
- Add at least one additional positive topology transition loaded entirely
  from YAML without code edits.
- Treat the checked-in `scenario_1` publication lane as the live successor of
  the historical `test_hjmt_multi_aggregator_sim.rs` contract, not as a new
  parallel simulator authority.
- Assert `scenario_config.yaml` controls old/new topology, route-generation
  boundaries, owner/standby roles, planned join mode, transfer target,
  activation point, and lawful `old_aggregator_count -> new_aggregator_count`
  transitions.
- Emit `leaf_flow.json`, `proof_flow.json`, `pub_flow.json`, `val_flow.json`,
  and `watch_flow.json` as evidence linked back to the full Phase 056
  `cfg_flow.json`, `tx_flow.json`, `route_flow.json`, `plan_flow.json`,
  `journal_flow.json`, `scope_flow.json`, `proc_flow.json`, and
  `recovery_flow.json` trace pack.
- Keep config digests, process ids, journal paths, owner/standby assignments,
  process exit/restart verdicts, and runtime on/off matrix join-or-leave rows
  in the same evidence packet as the publication traces.
- Fail if `scenario_design.yaml` drifts from the executed stage graph.

**Success conditions**

- `057-SC-05` and `057-SC-06` pass.
- `057-G5` is closed with one truthful 5x7 publication lane and one second
  positive YAML-driven topology row plus one attached config/process/runtime
  evidence packet.

**Targeted commands**

```bash
cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_preflight -- --nocapture
cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_hjmt_runtime_config -- --nocapture
cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario_settlement -- --nocapture
cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario1_stage_surface -- --nocapture
cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_publish -- --nocapture
```

## Wave 4: `057-04` Join, Transfer, Carry-Forward, And Crash Recovery

**Purpose:** prove lawful publication transitions under join, transfer, partial
failure, and recovery.

**Files to create or extend**

- `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs`
- `crates/z00z_storage/src/settlement/test_live_recovery.rs`
- `crates/z00z_runtime/aggregators/tests/fixtures/failover_v1/`

**Implementation tasks**

- Add `5x7 -> 6x7` standby-only join and `5x7 -> 6x7` owner activation after
  route generation `N+1` as mandatory exemplars.
- Add at least one additional positive topology transition loaded from YAML
  that preserves lineage and route invariants.
- Assert pre-activation owner publication rejects.
- Assert transfer to a remaining aggregator and transfer to a new aggregator as
  separate mandatory exemplars.
- Assert transfer evidence includes old route table, new route table, transfer
  target, activation checkpoint, old shard roots, new shard roots, and
  historical-proof continuity.
- Add `FOV-G-002`..`FOV-G-004` rows for mid-window failure containment,
  byte-identical carry-forward, and route-migration crash recovery.
- Assert crash recovery has one lawful publication outcome only.

**Success conditions**

- `057-SC-07` through `057-SC-10` pass.
- `057-G6`, `057-G7`, and `057-G8` are closed with separate evidence packets
  and explicit reject rows.

**Targeted commands**

```bash
cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_join -- --nocapture
cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_migrate -- --nocapture
cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_failover_same_lineage -- --nocapture
cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_preflight -- --nocapture
```

## Wave 5: `057-05` Downstream Acceptance And Dynamic Scope Continuity

**Purpose:** bind validators, watchers, and first-scope continuity to one
publication digest story.

**Files to create or extend**

- `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`
- `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`
- `crates/z00z_storage/tests/test_hjmt_scope_birth.rs`
- `crates/z00z_simulator/tests/test_scenario_settlement.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`

**Implementation tasks**

- Add validator and watcher acceptance rows that consume the same canonical
  publication digest and the same verdict mapping.
- Add first-scope birth rows immediately before standby takeover and
  immediately before carry-forward, using first-seen `definition_id` and
  `serial_id` births, and assert only the touched shard root, touched leaf,
  and touched checkpoint digest change.
- Assert no extra public registry or watcher-local reinterpretation appears.
- Assert `val_flow.json` and `watch_flow.json` resolve back to the same
  `cfg_flow.json`, `tx_flow.json`, `route_flow.json`, `plan_flow.json`,
  `journal_flow.json`, `scope_flow.json`, `proc_flow.json`, and
  `recovery_flow.json` lineage packet.
- Assert the same downstream evidence packet carries config digests, process
  ids, journal paths, owner/standby assignments, process exit/restart
  verdicts, and runtime on/off matrix join-or-leave rows.
- Keep `scenario_design.yaml` synchronized whenever scenario stages or
  publication flow semantics change.

**Success conditions**

- `057-SC-11` passes.
- `057-G9`, `057-G10`, and the trace side of `057-G11` are closed with one
  digest story.

**Targeted commands**

```bash
cargo test -p z00z_validators --release
cargo test -p z00z_watchers --release
cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario_settlement -- --nocapture
cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario1_stage_surface -- --nocapture
```

## Wave 6: `057-06` Guardrails, Bench Lanes, And Closeout Matrix

**Purpose:** close the phase with explicit evidence that no second authority
lane or second benchmark harness appeared.

**Files to create or extend**

- `crates/z00z_runtime/aggregators/tests/test_live_guardrails.rs`
- `crates/z00z_storage/tests/test_live_guardrails.rs`
- `crates/z00z_storage/tests/test_bench_lanes.rs`
- `crates/z00z_storage/benches/settlement_hjmt.rs`
- `crates/z00z_storage/benches/settlement_shard.rs`
- `crates/z00z_storage/benches/settlement_nested.rs`
- `crates/z00z_storage/benches/settlement_proofs.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`

**Implementation tasks**

- Add anti-drift source-shape checks for no second publication digest lane, no
  second proof truth lane, no second validator or watcher truth lane, and no
  second simulator evidence lane.
- Wire publication or root-of-roots benchmark evidence into the accepted
  storage bench homes only.
- Wire the required shard-parallel, nested, or proof-support lanes into current
  bench homes or accepted direct successors only.
- Assert `SIM-SMALL`, `SIM-MEDIUM`, and `SIM-CACHE-EDGE` are covered, and
  assert `SIM-BATCH-1000` remains reserved-only.
- Close the config/process evidence packet and inherited runtime on/off matrix
  join-or-leave rows in the same closeout matrix that proves the bench lanes.
- Close the gate-to-evidence matrix without claiming final Phase 058 readiness
  or release judgment.

**Success conditions**

- `057-SC-12` and `057-SC-13` pass.
- Bench and guardrail evidence confirms there is no second harness and no
  duplicate authority layer.

**Targeted commands**

```bash
cargo test -p z00z_storage --release --features test-params-fast --test test_bench_lanes -- --nocapture
cargo bench -p z00z_storage --bench settlement_shard --no-run
cargo bench -p z00z_storage --bench settlement_hjmt --no-run
```

## Wave 7: `057-07` Canonical Publication-Binding Guardrails And Packet Sync

**Purpose:** keep the shared publication-binding path singular after closeout
and move the test packet from speculative planning wording to execution-backed
reality.

**Files to create or extend**

- `crates/z00z_runtime/aggregators/tests/test_live_guardrails.rs`
- `crates/z00z_storage/tests/test_live_guardrails.rs`
- `crates/z00z_runtime/aggregators/README.md`
- `057-TEST-SPEC.md`
- `057-TESTS-TASKS.md`

**Implementation tasks**

- Add source-shape checks that keep `bind_publication_contract(...)` as the
  only runtime-owned `PublicationBinding` constructor path.
- Assert validators, watchers, and simulator evidence reuse the same binding
  digest instead of constructing or hashing a local publication-binding lane.
- Renormalize the phase-local test packet from `planning-ready` and
  `proposed` wording to execution-backed status once the live tests, fixtures,
  and guardrails are confirmed in the repository.
- Keep `057-07` as the only post-closeout continuation for Phase 057; do not
  fork a second test closeout lane or a duplicate authority note.

**Success conditions**

- The live guardrails fail if a second publication-binding constructor or
  digest lane appears.
- `057-TEST-SPEC.md` and `057-TESTS-TASKS.md` reflect the current live test
  homes, fixture homes, and `057-07` continuation honestly.

**Targeted commands**

```bash
cargo test -p z00z_aggregators --release --features test-params-fast --test test_live_guardrails -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_live_guardrails -- --nocapture
cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario1_stage_surface -- --nocapture
```

## Task Ledger

| Task ID | Scenario IDs | File Homes | Action | Done When |
| --- | --- | --- | --- | --- |
| `057-TT-01` | `057-SC-01` | `crates/z00z_storage/tests/test_hjmt_root_generation.rs`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_root_leaf_v1/` | Create pre-shard versus post-shard generation tests plus `SRL-G-*` and `SRL-T-*` fixtures. | Root generations are explicit, exact bytes are canonical, and confusion rows reject. |
| `057-TT-02` | `057-SC-02` | `crates/z00z_runtime/aggregators/tests/test_hjmt_publish.rs`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/checkpoint_publication_v1/` | Create ordered checkpoint publication tests plus `CPP-G-*` and `CPP-T-*` fixtures. | Canonical order, prior-root continuity, route binding, and monotonicity are proven and tamper rows reject. |
| `057-TT-03` | `057-SC-03`, `057-SC-04` | `crates/z00z_storage/tests/test_hjmt_historical_proofs.rs`, `crates/z00z_storage/tests/test_hjmt_live_proof_families.rs`, `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs` | Create layered proof, worked-example replay, and historical-compatibility coverage. | Public inclusion remains layered above shard-local proof truth and wrong-lineage or cross-shard counterexamples reject. |
| `057-TT-04` | `057-SC-05`, `057-SC-06` | `crates/z00z_simulator/tests/test_hjmt_runtime_config.rs`, `crates/z00z_simulator/tests/test_scenario_settlement.rs`, `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` | Extend the simulator packet for `SIM-5A7S-PUB`, continuation of the Phase 056 multi-aggregator simulation lane, publication traces, YAML-driven second topology, scope-to-publication continuity, and the attached config/process/runtime evidence packet. | Seven-leaf publication evidence is replayable, a second positive topology runs from YAML, trace linkage back to the full Phase 056 packet is explicit, and runtime evidence rows stay attached. |
| `057-TT-05` | `057-SC-07`, `057-SC-08` | `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs`, `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs` | Create standby-only join and owner-activation coverage for `5x7 -> 6x7` and one additional positive topology transition. | Join-as-standby and join-as-owner remain separate protocol states with separate verdicts and pre-activation rows reject. |
| `057-TT-06` | `057-SC-09`, `057-SC-10`, `057-SC-11` | `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs`, `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`, `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`, `crates/z00z_storage/src/settlement/test_live_recovery.rs`, `crates/z00z_storage/tests/test_hjmt_scope_birth.rs` | Create transfer, crash-recovery, carry-forward, validator, watcher, and first-scope continuity coverage. | Transfer to a remaining aggregator and to a new aggregator are both route-generation-bound, carry-forward is byte-identical, crash recovery has one lawful publication outcome, and downstream consumers share one digest story plus one runtime evidence packet. |
| `057-TT-07` | `057-SC-12` | `crates/z00z_storage/tests/test_live_guardrails.rs`, `crates/z00z_runtime/aggregators/tests/test_live_guardrails.rs`, `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` | Add anti-drift checks for no second publication lane, no second proof lane, no second validator or watcher truth lane, and no second simulator evidence lane. | Any duplicate-authority pattern causes guardrail failure. |
| `057-TT-08` | `057-SC-13` | `crates/z00z_storage/tests/test_bench_lanes.rs`, `crates/z00z_storage/benches/settlement_hjmt.rs`, `crates/z00z_storage/benches/settlement_shard.rs`, `crates/z00z_storage/benches/settlement_nested.rs`, `crates/z00z_storage/benches/settlement_proofs.rs` | Extend the required benchmark lanes without inventing a second harness and keep `SIM-BATCH-1000` reserved-only. | Required bench evidence exists in the accepted owner homes only, the reserved profile remains unclaimed, and the closeout matrix carries config/process/runtime on-off evidence. |
| `057-TT-09` | `057-SC-12`, `057-SC-13` continuation | `crates/z00z_runtime/aggregators/tests/test_live_guardrails.rs`, `crates/z00z_storage/tests/test_live_guardrails.rs`, `057-TEST-SPEC.md`, `057-TESTS-TASKS.md` | Add one canonical publication-binding guardrail path and renormalize the test packet to execution-backed wording. | The shared binding path is source-shaped and the phase-local test packet matches the live repository state without speculative `proposed` drift. |

## Required Fixture Tasks

| Fixture Task | Owner Home | Minimum Content |
| --- | --- | --- |
| Shard-root fixtures | `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_root_leaf_v1/` | Canonical bytes, digest, generation, route binding, regeneration command, and expected verdict for `SRL-G-*` and `SRL-T-*`. |
| Checkpoint-publication fixtures | `crates/z00z_storage/tests/fixtures/hjmt_upgrade/checkpoint_publication_v1/` | Canonical ordered leaves, prior-root linkage, monotonicity metadata, regeneration command, and expected verdict for `CPP-G-*` and `CPP-T-*`. |
| Carry-forward fixtures | `crates/z00z_runtime/aggregators/tests/fixtures/failover_v1/` | `FOV-G-002`..`FOV-G-004`, lineage state, process map, expected verdict, and trace linkage. |
| Trace manifests | phase-owned publication evidence outputs | Config digests, route digest, process ids, journal paths, owner/standby assignments, process exit/restart verdicts, runtime on/off matrix join-or-leave rows, trace paths, topology transition id, and publication digest. |

## Owned Fixture Id Ledger

- `SRL-G-001`, `SRL-G-002`, `SRL-G-003`, `SRL-G-004`
- `SRL-T-001`, `SRL-T-002`, `SRL-T-003`, `SRL-T-004`, `SRL-T-005`,
  `SRL-T-006`
- `CPP-G-001`, `CPP-G-002`, `CPP-G-003`, `CPP-G-004`, `CPP-G-005`
- `CPP-T-001`, `CPP-T-002`, `CPP-T-003`, `CPP-T-004`, `CPP-T-005`,
  `CPP-T-006`, `CPP-T-007`
- `FOV-G-002`, `FOV-G-003`, `FOV-G-004`
- `Current HJMT root set`
- `5x7 checkpoint-publication evidence`
- `join-as-standby evidence`
- `join-as-owner evidence`
- `shard-transfer evidence`
- `root-generation migration evidence`

## Mandatory Review Loop

Every execution closeout for this packet must run
`/.github/prompts/gsd-review-tasks-execution.prompt.md`
(` /GSD-Review-Tasks-Execution `) in YOLO mode at least three times.

Do not stop the review loop until at least two consecutive runs report no
significant issues, and fix every material warning or finding before claiming
completion.

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
   `watch_flow.json` are all linked to one `cfg_flow.json`, `tx_flow.json`,
   `route_flow.json`, `plan_flow.json`, `journal_flow.json`,
   `scope_flow.json`, `proc_flow.json`, and `recovery_flow.json` lineage
   packet and one digest story.
6. Remaining-aggregator transfer and new-aggregator transfer both have passing
   route-generation-bound exemplars, and the same closeout matrix carries
   config digests, process ids, journal paths, owner/standby assignments,
   process exit/restart verdicts, and runtime on/off matrix join-or-leave rows.
7. Guardrail tests fail if a second planner truth path, second proof truth
   path, second publication digest path, or second simulator evidence lane
   appears.
