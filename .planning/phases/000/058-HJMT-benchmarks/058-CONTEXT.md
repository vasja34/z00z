# Phase 058: HJMT Benchmarks - Context

**Gathered:** 2026-06-14
**Status:** ready-for-execution-planning
**Source:** deep read of `058-TODO.md`, the referenced HJMT upgrade and design
packet, the completed Phase 056 runtime packet, the completed Phase 057
publication packet, and the live storage, runtime, simulator, validator,
watcher, and bench seams already present in the repository

<domain>

## 🎯 Phase Boundary

Phase 058 is the evidence, benchmark, and readiness-closeout phase for the
HJMT sharding upgrade.

It does not invent new routing semantics above Phase 056 or new publication
semantics above Phase 057. It verifies, measures, classifies, and closes the
upgrade honestly under release-style conditions.

Phase 058 owns:

- one cross-phase evidence ledger with source-backed claim discipline;
- the release-mode simulator as the canonical end-to-end observability lane;
- YAML config realism and at least one additional positive non-`5x7`
  topology change loaded from disk;
- final `SIM-5A7S`, `SIM-5A7S-PUB`, and `SIM-BATCH-1000` readiness packets;
- benchmark closure for mutation, query, search, proof bytes, verify time,
  shard scaling, recovery, and adaptive policy lanes;
- persistence, import/export, restart, and startup fail-closed readiness;
- dynamic-scope and wallet-observability closure;
- honest proof-compression status;
- full fixture-family and evidence-gap closure;
- the final repository verdict for the HJMT upgrade.

Phase 058 must not create:

- a second route, planner, publication, validator, watcher, or simulator truth
  lane;
- a debug-only acceptance story that replaces the public release lane;
- a second benchmark harness when live owner homes already exist;
- false repository facts by treating TODO contract names as already-live paths;
- a release-ready verdict that outruns the checked evidence.

</domain>

<review_findings>

## 🚨 Review Findings That Forced This Packet

### S1-HIGH: The phase was registered but had no executable planning packet

- The existing folder contained only `058-TODO.md`.
- There was no `058-CONTEXT.md`, `058-SOURCE-AUDIT.md`, numbered `058-*-PLAN`
  packet, or phase-local test planning packet.
- Result: the repository had a registered readiness phase with no frozen owner
  map, no gate routing, and no honest live-versus-proposed path ledger.

### S1-HIGH: `058-TODO.md` mixes exact live anchors with contract names

- `scenario_config.yaml`, `scenario_design.yaml`, the current bench homes, and
  several HJMT tests are exact live anchors.
- `run_meta.json`, `wallet_scan.json`, `sim_summary.md`, `hist_flow.json`, and
  `occ_flow.json` now have exact live public release-packet homes on the
  `scenario_1` lane, and `test_hjmt_batch_commit.rs`,
  `test_hjmt_batch_recovery.rs`, `test_hjmt_transition_proofs.rs`,
  `test_hjmt_privacy_regression.rs`, and `test_hjmt_e2e.rs` now also have
  exact live repo homes; meanwhile `asset_flow.json` and `right_flow.json`
  remain pending exact homes on that same public packet.
- Result: the plan must keep only the remaining pending exact homes labeled
  honestly.

### S1-HIGH: Release-mode observability is the real closeout boundary

- The simulator already owns the checked-in trace packet from `cfg_flow.json`
  through `watch_flow.json`, but the TODO phase requires a larger release packet
  and a proof that optional private lanes do not become mandatory.
- Result: the plan must freeze one public release lane and keep private debug
  surfaces optional and non-gating.

### S2-MEDIUM: Benchmark-home wording had drift between TODO language and the live repo

- Older Phase 058 wording referenced
  `crates/z00z_storage/outputs/assets/`.
- The checked evidence home is
  `crates/z00z_storage/outputs/settlement/`.
- Result: closeout retires the stale alias and keeps one canonical measured
  archive path.

### S2-MEDIUM: Some acceptance lanes began as successors and now need explicit closeout binding

- Historical proofs, adaptive policy transitions, and occupancy privacy already
  have live test anchors, but not all under the exact filenames listed in the
  TODO contract.
- Batch-commit is now closed on an exact fixture manifest and exact owner test
  home, while batch-recovery stays an explicit operational-readiness owner seam.
- Result: the planning packet must distinguish `verified live`, `successor
  live`, and `proposed`.

</review_findings>

<decisions>

## 🔑 Implementation Decisions

### D-01: `058-TODO.md` remains the canonical backlog and scope authority

- This context, the source audit, the numbered plans, and the test packet
  schedule the TODO.
- They do not replace it.

### D-02: Phase 058 verifies inherited runtime and publication truth in place

- Phase 056 remains the runtime, route, process, and failover baseline.
- Phase 057 remains the publication, validator, and watcher digest baseline.
- Phase 058 closes evidence and readiness on top of those baselines instead of
  reopening their semantics.

### D-03: Live, successor, and proposed homes must stay separated

- `verified live` means the exact file exists today and already owns the seam.
- `successor live` means the exact TODO filename is absent, but a current live
  acceptance seam already covers the requirement under another exact filename.
- `proposed` means no exact live acceptance home is yet verified.

### D-04: The evidence ledger is a first-class execution artifact

- Every accepted claim must point to one command, one artifact path, one digest
  or result set, and one verdict.
- Unsupported claims must stay explicit and unsupported.
- Appendix C design-artifact rows stay literal in the ledger: the
  one-machine multi-aggregator simulation plan, deployment-shape notes,
  local-versus-replicated journal notes, RAID-like topology notes,
  implementation-options notes, and benchmark templates must each resolve to a
  checked artifact, command, and verdict row.

### D-05: The release-mode simulator is the canonical end-to-end lane

- `cargo run --release -p z00z_simulator --bin scenario_1` or its exact
  successor is the public end-to-end gate.
- Debug-only or private observability must not become the only acceptance path.
- Missing, stale, redaction-violating, or disconnected public trace artifacts
  fail the release lane instead of becoming soft follow-up work.

### D-06: The Phase 056/057 trace lineage remains inherited authority

- `cfg_flow.json`, `tx_flow.json`, `route_flow.json`, `plan_flow.json`,
  `journal_flow.json`, `scope_flow.json`, `proc_flow.json`, and
  `recovery_flow.json` remain the inherited lineage packet.
- Phase 058 may extend that packet, but it must not detach new evidence from it.

### D-07: `scenario_design.yaml` must stay synchronized with executable stages

- Any stage-surface or artifact-surface drift in `scenario_1` must update
  `scenario_design.yaml` in the same slice.
- A hidden release lane is not acceptable.

### D-08: The live YAML config anchors are frozen up front

- Aggregator config home:
  `config/hjmt_runtime/sim_5a7s/aggregators/agg-*/aggregator-config.yaml`
- Planner config home:
  `config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml`
- Storage config home:
  `config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml`
- Live simulator config home:
  `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`

### D-09: `SIM-5A7S` and `SIM-5A7S-PUB` remain the canonical acceptance fixtures

- They are mandatory readiness fixtures.
- They do not imply `5x7` is the only supported topology.

### D-10: `SIM-BATCH-1000` is heavy-only evidence

- It is required for stress, benchmark, and readiness evidence.
- It must not replace smaller deterministic correctness profiles
  `SIM-SMALL`, `SIM-MEDIUM`, and `SIM-CACHE-EDGE`.
- `SIM-SMALL` remains the fast deterministic correctness profile in the
  `16-64` operation range.
- `SIM-MEDIUM` remains the integration-correctness profile in the `128-256`
  operation range.
- `SIM-CACHE-EDGE` remains the capacity-relative cache-validation profile using
  `cap - 1`, `cap`, `cap + 1`, and `2 * cap`.

### D-11: Search, query, and shard-scaling claims require explicit reports

- Mutation TPS alone is not enough.
- Path lookup, terminal lookup, proof lookup, absent-proof generation,
  post-reload lookup, proof-size, verify-time, and shard-scaling claims must
  be measured and archived explicitly.
- Shard-scaling rows must preserve hot-shard ratio, global cadence, shard TPS,
  global TPS, and blocked time for the `1/2/4/8/16` worker matrix plus the
  canonical `SIM-5A7S` profile.
- The Design benchmark lanes `insert_many_definitions`,
  `insert_many_serials`, `insert_many_hot_serial`,
  `delete_many_definitions`, `delete_many_hot_serial`,
  `prove_many_assets`, `commit_recovery_replay`, and
  `compat_equivalence_random_ops` must map to executed reports or exact
  deterministic conformance replacements with explicit verdict rows.
- Insert, delete, proof, recovery, equivalence, crash-interruption, and
  performance lanes must be measured or rejected explicitly instead of being
  implied by a smaller subset of reports.
- Score packets must preserve raw timing slices for planning, child commit,
  parent commit, journal sync, recovery replay, search or query time, and
  proof time instead of collapsing everything into one aggregate median.

### D-12: Persistence, import/export, and startup fail-closed are core readiness

- Restart, artifact portability, corrupted-state rejection, wrong-lineage
  rejection, and wrong-generation rejection are not polish items.
- Startup fail-closed coverage also includes stale-route rejection and wrong
  proof-family-tag rejection on the checked startup surface.
- The current RedB-backed local journal remains the active evidence baseline
  for Phase 058. Ordered WAL and replicated-log directions stay future adapter
  surfaces behind `JournalBackend` conformance and equal-durability gates
  rather than becoming a second active truth lane inside this phase.

### D-13: Dynamic scope closure must prove proof-before-ownership

- First-seen semantic scope birth must remain visible through the release lane.
- Wallet state may become spendable or confirmed only after the proof boundary
  is satisfied.
- Split and merge transition closure must stay bound to imported route and
  publication artifacts instead of synthetic roots or current-policy replay.
- Route migration near a scope-creating batch must stay explicit across the
  runtime, historical-proof, and wallet-closure stories.
- `hist_flow.json` and `occ_flow.json` closure must record old and new route
  generations, old and new root generations, historical-proof verdicts,
  occupancy-disclosure verdicts, and imported-artifact validation verdicts.

### D-14: Proof compression status must stay honest

- `BatchProofBlobV1` closure comes first.
- Any further compression lane must be clearly versioned and must stay
  non-release-gating unless full evidence lands.

### D-15: Bench owner homes stay narrow

- `settlement_proofs.rs`, `settlement_hjmt.rs`, `settlement_shard.rs`,
  `settlement_nested.rs`, and `adaptive_policy_bench.rs` remain the live bench
  owner homes.
- Do not create a second HJMT benchmark crate or shadow harness.

### D-16: The current bench evidence path mismatch is phase-owned

- `crates/z00z_storage/outputs/settlement/` is live today.
- `crates/z00z_storage/outputs/assets/` is retired TODO-level wording.
- Closeout keeps `crates/z00z_storage/outputs/settlement/` as the one canonical
  evidence home.

### D-17: Full fixture closure may reuse existing manifests, but not partial truth

- Existing `SRT-*`, `SRL-*`, `CPP-*`, `FOV-*`, and `BPB-*` manifests are useful
  anchors.
- Phase 058 is still responsible for closing every required family and `12.1`
  evidence gap with exact regeneration and verdict data.

### D-18: The execution packet is seven ordered slices

1. `058-01`: evidence ledger, live/proposed path honesty, and design-artifact
   map
2. `058-02`: release-mode simulator observability and stage-sync closure
3. `058-03`: YAML config realism, extra topology, import/export, restart, and
   startup fail-closed readiness
4. `058-04`: final `SIM-5A7S` and `SIM-5A7S-PUB` closure on the real release
   lane
5. `058-05`: benchmark matrix, `SIM-BATCH-1000`, shard scaling, score
   discipline, and compression verdict
6. `058-06`: dynamic scope, wallet closure, historical playback, and occupancy
   privacy
7. `058-07`: full fixture-family closure, evidence-gap closure, final verdict,
   and planning-state sync

### D-19: Mandatory verification order applies to every Rust or test-affecting slice

- Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  first.
- If bootstrap fails, stop, fix the regression, and rerun it before broader
  validation.
- Then run the slice-local targeted tests or benches.
- Then run `cargo test --release`.
- Then run `cargo doc --no-deps` when public docs or APIs change.
- Then run `/.github/prompts/gsd-review-tasks-execution.prompt.md`
  (`/GSD-Review-Tasks-Execution`) in YOLO mode at least three times and
  continue until at least two consecutive runs show no significant issues.

### D-20: Commit and review flows must stay local to `.github/`

- If a slice needs a version or release-flow commit, use `/z00z-git-versioning`.
- Nested prompts, skills, scripts, and review loops resolve from `.github/`.

### D-21: The final verdict vocabulary is fixed

- `contract only`
- `prototype`
- `verified slice`
- `integrated upgrade`
- `release-ready`

No other final marketing label is allowed.

</decisions>

<canonical_refs>

## 📚 Canonical References

### Phase authority

- `.planning/phases/058-HJMT-benchmarks/058-TODO.md`
- `.planning/phases/058-HJMT-benchmarks/058-CONTEXT.md`
- `.planning/phases/058-HJMT-benchmarks/058-SOURCE-AUDIT.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`

### Normative docs

- `docs/tech-papers/Z00Z-HJMT-Upgrade.md`
- `docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md`
- `docs/tech-papers/Z00Z-HJMT-Design.md`

### Predecessor packet anchors

- `.planning/phases/000/056-HJMT-storage- aggregator/056-CONTEXT.md`
- `.planning/phases/000/056-HJMT-storage- aggregator/056-SOURCE-AUDIT.md`
- `.planning/phases/000/056-HJMT-storage- aggregator/056-TEST-SPEC.md`
- `.planning/phases/000/057-HJMT-multi-aggregator/057-CONTEXT.md`
- `.planning/phases/000/057-HJMT-multi-aggregator/057-SOURCE-AUDIT.md`
- `.planning/phases/000/057-HJMT-multi-aggregator/057-TEST-SPEC.md`
- `.planning/phases/000/057-HJMT-multi-aggregator/057-TESTS-TASKS.md`

### Live runtime and node anchors

- `config/hjmt_runtime/sim_5a7s/manifest.json`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-*/aggregator-config.yaml`
- `config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml`
- `config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_planner.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_publish.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_split_brain_fencing.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_shard_routing.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_process.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs`

### Live simulator anchors

- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
- `crates/z00z_simulator/src/scenario_1/runner.rs`
- `crates/z00z_simulator/src/scenario_1/runner_verify.rs`
- `crates/z00z_simulator/tests/test_hjmt_runtime_config.rs`
- `crates/z00z_simulator/tests/test_scenario_settlement.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- `crates/z00z_simulator/tests/test_stage7_jmt_wallet_scan.rs`

### Live storage, fixture, and bench anchors

- `crates/z00z_storage/tests/test_hjmt_batch_proof.rs`
- `crates/z00z_storage/tests/test_hjmt_batch_proof_negative.rs`
- `crates/z00z_storage/tests/test_hjmt_root_generation.rs`
- `crates/z00z_storage/tests/test_hjmt_historical_proofs.rs`
- `crates/z00z_storage/tests/test_hjmt_scope_birth.rs`
- `crates/z00z_storage/tests/test_hjmt_adaptive_policy_proofs.rs`
- `crates/z00z_storage/tests/test_occupancy_privacy.rs`
- `crates/z00z_storage/tests/test_occupancy_evidence.rs`
- `crates/z00z_storage/tests/test_live_guardrails.rs`
- `crates/z00z_storage/tests/test_downstream_guardrails.rs`
- `crates/z00z_storage/src/settlement/test_live_recovery.rs`
- `crates/z00z_storage/tests/test_redb_reload.rs`
- `crates/z00z_storage/tests/test_bench_lanes.rs`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_positive/manifest.json`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_negative/manifest.json`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/root_generation_migration/manifest.json`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_route_table_v1/manifest.json`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_root_leaf_v1/manifest.json`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/checkpoint_publication_v1/manifest.json`
- `crates/z00z_storage/benches/settlement_hjmt.rs`
- `crates/z00z_storage/benches/settlement_proofs.rs`
- `crates/z00z_storage/benches/settlement_shard.rs`
- `crates/z00z_storage/benches/settlement_nested.rs`
- `crates/z00z_storage/benches/adaptive_policy_bench.rs`
- `crates/z00z_storage/benches/settlement_benches.md`
- `crates/z00z_storage/scripts/run_storage_settlement_bench.py`

### Live downstream acceptance anchors

- `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`
- `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`

</canonical_refs>

<ownership_map>

## 🧭 Cross-Crate Ownership Map

| Concern | Primary owner | Explicit non-owner rule |
| --- | --- | --- |
| Runtime topology, route generation, planner behavior, join/transfer/failover intent | `z00z_runtime/aggregators` plus `z00z_rollup_node` config wiring | Storage, validators, watchers, and simulator must not become alternate route or process truth owners. |
| Committed roots, proofs, historical validation, and adaptive transition semantics | `z00z_storage` | Runtime or simulator evidence must not replace storage proof truth. |
| Public checkpoint digest, validator verdict, and watcher verdict continuity | inherited Phase 057 runtime + validator + watcher seams | Phase 058 must consume and measure this digest story, not redefine it. |
| Release-mode trace packet and stage synchronization | `z00z_simulator` | Trace evidence is observability only, not semantic truth. |
| Bench harnesses and archived measured reports | `z00z_storage/benches/*` plus the bench runner | Do not add a second benchmark authority path. |
| Final evidence ledger and verdict classification | Phase 058 planning artifacts | The ledger summarizes repository evidence; it does not invent new code authority. |

</ownership_map>

<plan_packet>

## ⚙️ Numbered Plan Packet

1. `058-01-PLAN.md` — evidence ledger, source-honest path mapping, and design-artifact closure map
2. `058-02-PLAN.md` — release-mode simulator observability and stage-sync closure
3. `058-03-PLAN.md` — YAML config realism, extra topology, import/export, restart, and startup fail-closed readiness
4. `058-04-PLAN.md` — final `SIM-5A7S` and `SIM-5A7S-PUB` release packets
5. `058-05-PLAN.md` — benchmark matrix, `SIM-BATCH-1000`, shard scaling, score discipline, and compression verdict
6. `058-06-PLAN.md` — dynamic scope, wallet closure, historical playback, and occupancy privacy
7. `058-07-PLAN.md` — fixture-family closure, evidence-gap closure, final verdict, and planning-state sync

</plan_packet>

<todo_coverage_contract>

## ✅ TODO Coverage Contract

### Section-to-owner transfer

| `058-TODO.md` section | Packet owner | Coverage rule |
| --- | --- | --- |
| `Mission`, `Phase Boundary`, `This phase owns`, `This phase does not own` | `058-CONTEXT.md` plus all numbered plans | Boundary language is frozen here and must remain consistent across every slice. |
| `Source Map` and all referenced papers | `058-CONTEXT.md` plus every plan `coverage_contract` | Global rules remain active across the full packet. |
| `Embedded audit contract` | `058-CONTEXT.md`, `058-01`, `058-02`, `058-03`, `058-04`, `058-05`, `058-06`, `058-07` | Release blockers, benchmark discipline, and evidence rules remain live scope. |
| `Mandatory implementation gates` | gate map below | Every gate has one primary owner plan. |
| `Workstream 1` | `058-01` | The evidence ledger and claim discipline must freeze before later slices claim closure. |
| `Workstream 2` | `058-02` | Release-mode observability and simulator/doc sync close together. |
| `Workstream 3` | `058-05` | Benchmark matrix, heavy profile, and score discipline belong to one measurement slice. |
| `Workstream 4` | `058-03` | YAML realism, restart, import/export, and startup fail-closed readiness must close together before final packet claims. |
| `Workstream 5` | `058-06` and `058-07` | Dynamic scope, wallet closure, and the final verdict close after the runtime and benchmark packet are honest. |
| `Required tests and benchmarks`, `Required execution profiles`, `Required scenario coverage` | `058-TEST-SPEC.md`, `058-TESTS-TASKS.md`, and `058-05` through `058-07` | Cross-cutting test and bench closure must stay explicit and source-backed. |
| `Required artifacts`, `Fixture closure`, and `Exit criteria` | `058-07` with dependencies on all previous slices | Final closure must prove the whole matrix together. |

### Gate-to-plan map

| Gate | Primary owner plan | Why |
| --- | --- | --- |
| `058-G1` | `058-01` | One evidence ledger must exist before readiness claims can be reviewed honestly. |
| `058-G2` and `058-G3` | `058-02` | Release-mode simulator closure and stage/doc sync are one observability packet. |
| `058-G4`, `058-G9`, and `058-G10` | `058-03` | Config realism, import/export, restart, and startup fail-closed readiness close together. |
| `058-G5` and `058-G6` | `058-04` | The canonical runtime and publication packets must be closed on the same release lane. |
| `058-G7`, `058-G8`, and `058-G12` | `058-05` | Heavy profile discipline, measurement closure, and honest compression verdict are one benchmark packet. |
| `058-G11` | `058-06` | Dynamic scope, wallet proof-before-ownership, and historical playback close together. |
| `058-G13` and final exit-criteria sync | `058-07` | Full fixture-family closure and the final repository verdict must aggregate every earlier slice. |

### Literal bullet preservation map

| Bullet class | Explicit packet owners |
| --- | --- |
| Claim ledger with commands, digests, verdicts, and unsupported rows | `058-01`, closed by `058-07` |
| Release-mode public simulator lane, stage sync, inherited trace lineage including `proc_flow.json` and `recovery_flow.json`, and private-lane non-dependence | `058-02`, rechecked by `058-04` and `058-07` |
| Literal simulator observability inventory `run_meta.json`, `cfg_flow.json`, `tx_flow.json`, `route_flow.json`, `plan_flow.json`, `journal_flow.json`, `scope_flow.json`, `leaf_flow.json`, `proof_flow.json`, `pub_flow.json`, `val_flow.json`, `watch_flow.json`, `wallet_scan.json`, `asset_flow.json`, `right_flow.json`, `hist_flow.json`, `occ_flow.json`, `recovery_flow.json`, and `sim_summary.md` | `058-02`, cross-checked by `058-TEST-SPEC.md` and `058-TESTS-TASKS.md` |
| Public-evidence redaction compliance plus missing/stale/disconnected artifact failure rules | `058-02`, `058-06`, and `058-07` |
| YAML-defined config behavior, publication leaf-set changes, extra topology, import/export, restart, stale-route reject, and startup fail-closed rows including wrong proof-family-tag reject | `058-03`, rechecked by `058-04` and `058-07` |
| Final `SIM-5A7S` and `SIM-5A7S-PUB` packets with deterministic re-encoding, independent-process proof, planner equivalence, same-lineage failover, wrong-lineage rejection, join/transfer/carry-forward, and validator/watcher evidence | `058-04`, closed by `058-07` |
| Runtime and publication evidence inventory including process ids, ports, data directories, journal paths, route-table digest, publication digest, join verdicts, migration verdicts, failover verdicts, and restart verdicts | `058-04`, closed by `058-07` |
| `SIM-BATCH-1000`, `SIM-SMALL`, `SIM-MEDIUM`, `SIM-CACHE-EDGE`, query/search/shard-scaling metrics, Design baseline lanes, cross-shard batch reject coverage, `128/1000/1024` proof-size rows, raw timing slices, archive-home honesty, and score classification | `058-05`, closed by `058-07` |
| First-seen scope birth, split or merge transition closure, route migration near a scope-creating batch, positive and negative wallet proof-before-ownership cases, final promotion, imported Phase 057 playback, historical replay, occupancy closure, and lineage-disagreement reject | `058-06`, closed by `058-07` |
| Full fixture-family closure over `SRT-G-001..004`, `SRT-T-001..008`, `SRL-G-001..004`, `SRL-T-001..006`, `CPP-G-001..005`, `CPP-T-001..007`, `FOV-001`, `FOV-T-001`, `FOV-T-002`, `FOV-G-002`, `FOV-G-003`, `FOV-G-004`, `BPB-G-001..005`, and `BPB-T-001..008`, plus `12.1` evidence-gap closure | `058-07` |

### Required test-lane owner routing

| Required lane from `058-TODO.md` | Primary plan owner | Routing rule |
| --- | --- | --- |
| `test_hjmt_batch_commit.rs`, `test_hjmt_batch_recovery.rs`, config-surface coverage lane, journal-baseline and WAL-boundary coverage lane, restart and persistence coverage lane, `test_hjmt_import_export.rs`, `test_hjmt_storage_boundary.rs`, `test_hjmt_backend_conformance.rs` | `058-03` | Config realism, journal baseline, restart safety, import/export, and fail-closed startup stay in one operational-readiness slice. |
| Continuation of the multi-aggregator simulation lane from Phases 056 and 057 | `058-02` and `058-04` | The simulator lineage stays single-path: `058-02` owns release-mode continuation and `058-04` owns final packet closure on that same lane. |
| `test_hjmt_transition_proofs.rs`, `test_hjmt_privacy_regression.rs`, and `test_hjmt_e2e.rs` | `058-06` | Historical-proof, occupancy-privacy, wallet-closure, and end-to-end user-visible readiness stay in one proof-boundary slice. |

### Required artifact inventory freeze

- Core evidence packet:
  `SIM-5A7S`, `SIM-5A7S-PUB`, `SIM-BATCH-1000`, one release-mode simulator
  packet, one evidence ledger, and accepted measured reports under
  `crates/z00z_storage/outputs/settlement/`.
- Required execution profiles:
  `SIM-SMALL` for `16-64` deterministic operations, `SIM-MEDIUM` for
  `128-256` integration operations, `SIM-CACHE-EDGE` for `cap - 1`, `cap`,
  `cap + 1`, and `2 * cap` cache-relative validation, and `SIM-BATCH-1000` for
  heavy-only readiness and benchmark evidence.
- Config evidence:
  aggregator YAML surface, planner YAML surface, storage YAML surface,
  `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`, config digests,
  and proof that changed values change live behavior.
- Runtime and publication evidence:
  process ids, ports, data directories, journal paths, route-table digest,
  publication digest, join verdicts, migration verdicts, failover verdicts, and
  restart verdicts.
- Simulator observability artifacts:
  `run_meta.json`, `cfg_flow.json`, `tx_flow.json`, `route_flow.json`,
  `plan_flow.json`, `journal_flow.json`, `scope_flow.json`, `leaf_flow.json`,
  `proof_flow.json`, `pub_flow.json`, `val_flow.json`, `watch_flow.json`,
  `wallet_scan.json`, `asset_flow.json`, `right_flow.json`, `hist_flow.json`,
  `occ_flow.json`, `recovery_flow.json`, and `sim_summary.md`.
- Fixture-family closure set:
  `SRT-G-001..004`, `SRT-T-001..008`, `SRL-G-001..004`, `SRL-T-001..006`,
  `CPP-G-001..005`, `CPP-T-001..007`, `FOV-001`, `FOV-T-001`, `FOV-T-002`,
  `FOV-G-002`, `FOV-G-003`, `FOV-G-004`, `BPB-G-001..005`, and
  `BPB-T-001..008`.

### Mandatory global cross-read before any implementation

From `docs/tech-papers/Z00Z-HJMT-Upgrade.md`:

- `2. Upgrade Principles`
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
- `9. Scorecard And Measurement Plan`
- `9.1 Benchmark Matrix`
- `9.2 Claim Gate`
- `9.3 Score Claim Discipline`
- `10. Correctness, Security, And Privacy Checklist`
- `10.1 Evidence Mapping Discipline`
- `12. Test And Benchmark Plan`
- `12.1 Evidence Gaps`
- `13. Required Decisions And Fail-Closed Rules`
- `13.1 Fail-Closed Discipline`
- `14. Readiness Definition`
- `14.1 Completion Discipline`
- `Appendix A. Normative Upgrade Requirements`
- `Appendix B. Repository Evidence Map`
- `Appendix C. Design Artifact Requirements`
- `Appendix E.4 Review Checklist For Implementation PRs`
- `Appendix E.5 Evidence Needed For Conformance-Safe Execution`
- `Appendix F. Discussion Coverage Matrix`
- `Appendix F.1 Traceability For Sharding And Storage Recommendations`

From `docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md`:

- `Completion Contract`
- `Release Gate`

From `docs/tech-papers/Z00Z-HJMT-Design.md`:

- `9.2 Benchmark Plan`
- `9.3 Acceptance Criteria`
- `13. Testing And Verification Strategy`
- `13.1 Equivalence Tests`
- `13.2 Crash Tests`
- `13.3 Proof Tests`
- `13.4 Performance Tests`

### Mandatory Phase 058 cross-read that every numbered plan must keep active

- `5. Upgrade 3: Stable Shard Layer Above Buckets`
- `6. Upgrade 4: Root-Of-Shard-Roots Publication`
- `6.8 Worked Example: Three Aggregators, Eight Assets, One Shard-Local Batch`
- `7. Upgrade 5: Local Adaptive Transitions`
- `7.1 Transition Families`
- `7.2 Split Proof`
- `7.3 Merge Proof`
- `7.4 Occupancy Privacy`
- `7.5 Historical Proof Rule`
- `7.6 Transition Safety Requirements`
- `7.7 Implementation Guidance`
- `7.8 Mermaid State View: Adaptive Transition Lifecycle`
- `11. Implementation Roadmap`
- `Appendix A.1 Mermaid Requirement Trace View`
- `Appendix D. Implementation Skeletons`
- `Appendix D.5 Stable Path And Asset Leaf Proof Skeleton`
- `Appendix E. Implementation Guidelines`
- `Appendix E.6 Cross-Crate Module Ownership`
- `Appendix E.7 Cross-Crate Execution Order`

</todo_coverage_contract>
