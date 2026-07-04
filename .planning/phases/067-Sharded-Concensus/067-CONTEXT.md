# Phase 067: Sharded Concensus - Context

**Gathered:** 2026-07-03
**Status:** Reviewed for plan coverage
**Source:** PRD Express Path (`.planning/phases/067-Sharded-Concensus/067-TODO.md`)

<verdict>
## 🎯 Verdict

`067-TODO.md` is the normative Phase 067 authority for this planning pass.
Literal `TASK-NNN` rows do not exist in the current artifact, so planning must
map the required implementation groups from `067-TODO.md` Section 14
(`14.1` through `14.9`) to numbered `067-NN-PLAN.md` files without inventing a
parallel task ledger.

The addendum line that names
`.planning/phases/067-Sharded-Concensus/Agg-Concensus-Spec.md` as canonical
authority conflicts with the live workspace and with the explicit user
instruction that `067-TODO.md` is normative. Planning therefore treats that
missing file reference as stale internal drift and must not create a second
authority plane to satisfy it.

`wiki -results.md` is supporting non-canonical context only.

`scenario_11` in `.planning/phases/090-New-Scenarios/066-TODO.md` is mandatory
planning input for the full local harness, evidence artifacts, and verification
anchors, but it does not replace `067-TODO.md` as the Phase 067 authority.
</verdict>

<domain>
## ⚙️ Phase Boundary

Phase 067 hardens the existing shard-local aggregator seam inside
`crates/z00z_runtime/aggregators` into a real local quorum-certificate pipeline.
The immediate target is not live network BFT and not live Celestia. The
immediate target is:

1. wallet-style package ingress normalization;
2. deterministic route-table shard selection;
3. primary execution subject construction;
4. independent secondary replay before voting;
5. per-shard quorum certificate formation;
6. local DA binding of the certificate digest;
7. validator acceptance or rejection against the same subject;
8. independent `scenario_11` evidence and fault-matrix coverage.

External transport, external DA transport, and devnet/provider behavior may be
simulated locally, but no plan may close on placeholder DTOs, no-op runners,
docs-only claims, or compile-only proof for runtime behavior.
</domain>

<decisions>
## 🔑 Implementation Decisions

### D-067-01 Normative Authority And Drift Handling
- `067-TODO.md` is normative.
- No plan may create or imply a second Phase 067 authority file.
- The missing `Agg-Concensus-Spec.md` reference is treated as stale drift unless
  a later explicit repository task recreates it as the same authority text.

### D-067-02 Required Group Inventory
- Unique literal `TASK-NNN` count is `0`.
- Required plan groups are the nine implementation groups in
  `067-TODO.md` Section 14:
  `PHASE-0` through `PHASE-8`.
- Each required group maps to exactly one `067-NN-PLAN.md`.

### D-067-03 Existing Runtime Seam First
- Implementation starts inside `crates/z00z_runtime/aggregators`.
- Public exports go through the `z00z_aggregators` crate root.
- Do not create a separate production consensus crate before the local seam is
  proven through tests and `scenario_11`.

### D-067-04 Honest Consensus Vocabulary
- The live system is local deterministic CFT quorum until stronger proof exists.
- Protocol prose uses `secondary aggregator`.
- Active code, config, fixtures, tests, and docs must remove live `standby`
  naming without aliases or compatibility shims.

### D-067-05 First-Class Quorum Artifacts
- Add `CommitSubject`, `ShardVote`, `ShardQuorumCertificate`, and
  `SecondaryReplayVerifier`.
- Digest material must use stable binary encoding with explicit domain and
  version bytes.
- Membership digest is the ordered set `{primary_id} + ready secondary
  aggregators` for one shard and one placement generation only.

### D-067-06 Real Project Primitives Only
- Routing uses real `IngressBoundary`, `WorkItem`, `ShardRouteTable`, and
  `BatchPlanner`.
- Recovery uses real `RecoveryBoundary`, `ShardRecoveryRecord`, and live
  journal-lineage and root metadata.
- Publication uses real `PublicationRequest`, `PublishedBatch`, and
  `LocalDaAdapter`.
- Validation uses real theorem and validator boundaries.
- Signature and digest work must use current `z00z_crypto` domain-separation and
  signature primitives when real signatures are added.
- HJMT, state-root, checkpoint, publication, validator, crypto, and utility
  behavior must reuse existing `z00z_storage`, `z00z_core`, `z00z_crypto`, and
  `z00z_utils` primitives rather than introducing a parallel implementation
  layer.

### D-067-07 Independent Scenario Ownership
- `scenario_11` is the owner of the end-to-end quorum-certificate harness.
- `scenario_1` remains reference-only and must not gain new
  quorum-certificate-specific stages or observability fields.

### D-067-08 External Layers Stay Behind Local Proof
- Network BFT, HotStuff, libp2p, and Celestia are valid design targets only
  after the local harness and local certificate semantics are proven.
- Local deterministic simulation may fake the external transport boundary or DA
  provider boundary, but it must still use real routing, planning, execution,
  replay, storage, publication, theorem, and validator logic.
- Dependency candidates listed in `067-TODO.md` Section `8.1` are deferred
  adapter options, not blanket approval to add third-party crates while an
  equivalent repository-owned primitive already exists.

### D-067-09 Anti-Placeholder Rule
- No plan may close on placeholder, scaffold-only, TODO-only, constant-digest,
  hard-coded vote, string-only, docs-only, or compile-only behavior.
- Acceptable implementation depths are `full`, `simulated-full`, and
  `live-claim-removed`.

### D-067-10 Strict TODO Section Lock
- All `19` H2 sections and all `55` H3 sections from `067-TODO.md` remain
  normative for the packet.
- Exact line-range, bullet-count, and plan-owner traceability is locked in
  `.planning/phases/067-Sharded-Concensus/067-SOURCE-AUDIT.md`.
- No review or execution step may silently drop Sections `1` through `19`,
  collapse Section `16` into an untracked note, or ignore Section `17` code
  anchors and Section `19` addendum rules.

### D-067-11 No Graphify Authority And No Parallel Layer
- Graphify may be used only for codebase orientation and must never be used as
  evidence for coverage, acceptance, or implementation truth.
- No duplicate codebase logic, mirror abstraction, or parallel planning
  authority may be introduced where the live codebase already has an owner.
- New file, module, test, config, or doc homes named in the numbered plans are
  proposed targets when absent in the current worktree; implementation must
  prefer a tighter existing owner if one already exists.
</decisions>

<constraints>
## 🛑 Requirement Gate Contract

- `REQ-067-001`: `067-TODO.md` stays normative for planning and execution
  scoping.
- `REQ-067-002`: The plan corpus must not create a second authority layer for
  Phase 067.
- `REQ-067-003`: Existing runtime work stays inside
  `crates/z00z_runtime/aggregators` and the `z00z_aggregators` facade until the
  local seam is proven.
- `REQ-067-004`: Active protocol-facing `standby` naming must be removed from
  live code, config, fixtures, tests, and docs without aliases.
- `REQ-067-005`: Quorum artifacts must use canonical binary encoding with
  explicit domain/version bytes and deterministic digest tests.
- `REQ-067-006`: Membership digest includes only the live primary and ready
  secondary aggregators for one shard and one placement generation.
- `REQ-067-007`: Validator and local DA acceptance must reject missing,
  detached, stale, mixed, or mismatched certificate binding once the gate is
  enabled.
- `REQ-067-008`: `scenario_11` must be an independent scenario target and must
  not modify `scenario_1` ownership.
- `REQ-067-009`: Fake only external transport, remote process, external DA
  transport, wall-clock, or unavailable third-party network behavior. Use real
  project primitives everywhere else.
- `REQ-067-010`: Every plan must contain explicit artifacts, tests, expected
  results, simulation requirements, evidence gates, and anti-placeholder proof.
- `REQ-067-011`: External BFT and Celestia work may close only through local
  deterministic simulation and 3f+1 or 2f+1 proof, not through future-only
  prose.
</constraints>

<threat_model>
## 🔐 Threat Model And Trust Boundaries

Security-critical assets for this packet:

- package digests and route digests;
- placement and membership digests;
- recovery lineage, root generation, and proof metadata;
- commit-subject, vote, and certificate digests;
- publication bindings, theorem digests, and validator verdicts;
- payload bytes and any later blob commitments or settlement anchors.

Packet adversaries and fault sources:

- stale or unready secondary aggregators;
- removed members or mixed-generation voters;
- equivocal voters or conflicting same-term subjects;
- crashed primaries before or after local quorum;
- drifted route tables, plan digests, state roots, proof versions, or policy generations;
- detached or mismatched publication, theorem, certificate, or blob bindings;
- transport delay, reorder, partition, offline-member, restart, or external-DA outage conditions.

Trust boundaries that must remain explicit:

- wallet-style package ingress normalization;
- route-table and placement-generation ownership;
- recovery record and lineage truth;
- local DA publish or resolve boundaries;
- validator checkpoint and theorem verification boundaries;
- external transport and external DA provider boundaries, which may be simulated
  but may not replace real local runtime primitives.

Fail-closed review rule:

- Any plan that allows votes, certificates, publications, or validator
  acceptance across these boundaries without explicit digest, membership,
  lineage, and state checks is invalid.
</threat_model>

<source_corpus>
## 📚 Source Corpus

- `.planning/phases/067-Sharded-Concensus/067-TODO.md`
  - authoritative Phase 067 spec, implementation phases, failure semantics,
    local DA model, theorem requirements, and addendum constraints.
- `.planning/phases/090-New-Scenarios/066-TODO.md`
  - `scenario_11` target, artifact list, unit/integration/E2E tests, fault
    matrix, and verification anchors.
- `crates/z00z_runtime/aggregators/README.md`
  - live doc path named by the rename matrix in `067-TODO.md`.
- `.planning/phases/067-Sharded-Concensus/067-SOURCE-AUDIT.md`
  - exact section, bullet, and corpus traceability evidence for the Phase 067
    packet; evidence only, not a second authority layer.
- `.planning/phases/067-Sharded-Concensus/wiki -results.md`
  - supporting codebase Q&A; non-canonical.
- `.github/copilot-instructions.md`
  - English-only artifacts, protected Tari vendor rule, tone signal, naming,
    verification, and compact-output policy.
- `.github/requirements/Z00Z_DESIGN_FOUNDATION.md`
  - one-source-of-truth, trait injection, domain separation, vendor isolation,
    concurrency, error handling, API design, and NASA-style rules.
- `.github/instructions/rust.instructions.md`
  - Rust testing, module, documentation, error-handling, and naming rules.
</source_corpus>

<count_answer>
## 🔢 Count Answer

- Unique `TASK-NNN` identifiers in `067-TODO.md`: `0`
- Required GSD Plan Groups: `9`
- Required group source: `067-TODO.md` Sections `14.1` through `14.9`
- Coverage rule: each required group maps to exactly one `067-NN-PLAN.md`
- Duplicate group status: none at planning start
- Missing group status: none at planning start
- Planning fail condition: any missing group, duplicate mapping, dropped
  acceptance clause, or unbound verification anchor is phase-failing.
- Exact structure lock: `19` H2 sections, `55` H3 sections, `447` dash-list
  bullets, and `80` numbered-list items are traced in
  `.planning/phases/067-Sharded-Concensus/067-SOURCE-AUDIT.md`.
</count_answer>

<section_lock>
## 🧷 Strict TODO Section Lock

- `.planning/phases/067-Sharded-Concensus/067-SOURCE-AUDIT.md` records exact
  section-to-owner traceability for the full
  `.planning/phases/067-Sharded-Concensus/067-TODO.md` packet.
- The packet must preserve:
  - all direct-answer sections in `3.*`;
  - all live-model and `sim_5a7s` truth in `4.*` and `5`;
  - the Mermaid flow and lifecycle semantics in `6.*`;
  - all current-vs-external boundary rules in `7.*` and `8.*`;
  - the full target model and failure semantics in `9.*` and `10.*`;
  - the DA, theorem, and simulation contracts in `11` through `13`;
  - the exact phase mapping in `14.*`;
  - the rotation discipline, doublecheck matrix, source evidence map, bottom
    line, and addendum rules in `15` through `19`.
- Any future edit that changes the TODO section inventory, bullet counts, or
  source corpus must update
  `.planning/phases/067-Sharded-Concensus/067-SOURCE-AUDIT.md` before
  execution continues.
</section_lock>

<required_groups>
## 📌 Required GSD Plan Groups

| Required group | Source section | Planned packet | Purpose |
| --- | --- | --- | --- |
| `PHASE-0` | `14.1` | `067-01-PLAN.md` | Terminology and boundary cleanup |
| `PHASE-1` | `14.2` | `067-02-PLAN.md` | Commit subject and certificate types |
| `PHASE-2` | `14.3` | `067-03-PLAN.md` | Secondary replay verifier |
| `PHASE-3` | `14.4` | `067-04-PLAN.md` | Local quorum certificate integration |
| `PHASE-4` | `14.5` | `067-05-PLAN.md` | End-to-end `sim_5a7s` harness |
| `PHASE-5` | `14.6` | `067-06-PLAN.md` | Join, removal, and rotation simulation |
| `PHASE-6` | `14.7` | `067-07-PLAN.md` | Validator and theorem binding |
| `PHASE-7` | `14.8` | `067-08-PLAN.md` | Network and signature adapter |
| `PHASE-8` | `14.9` | `067-09-PLAN.md` | BFT and Celestia local backend |
</required_groups>

<pre_plan_blockers>
## ⚠️ Pre-Plan Blockers

- `067-CONTEXT.md` did not exist before this planning pass.
- `067-TODO.md` contains no literal `TASK-NNN` rows; the plan corpus must use
  required group ids instead of fabricating a task ledger.
- `.planning/phases/067-Sharded-Concensus/Agg-Concensus-Spec.md` is referenced
  but absent; planning must not create it as a second authority layer.
- `scenario_11` verification commands are normative only after the target and
  tests exist; each relevant plan must create those targets before claiming the
  commands are runnable.
- Live code, config, tests, and docs still expose `standby` naming in active
  paths, so Phase 067 must start with a breaking terminology cleanup.
</pre_plan_blockers>

<artifact_contract>
## 🧪 Artifact/Test/Result Proof Contract

Every numbered plan must include:

- `plan_id`
- `task_ids`
- copied source rows from the controlling section
- exact `source_refs`
- concrete `inputs`
- concrete `outputs`
- explicit `dependencies`
- executable `acceptance_tests`
- explicit `simulation_gate`
- explicit `negative_tests`
- `plan_artifacts`
- `plan_tests`
- `plan_results`
- per-group `task_artifacts`
- per-group `task_tests`
- per-group `task_results`
- `anti_placeholder_gate`
- `current_code_refs`
- `blockers`
- `evidence_gate`
- `not_recommendation_gate`

Per-group implementation depth must be one of:

- `full`
- `simulated-full`
- `live-claim-removed`
</artifact_contract>

<current_code_refs>
## 🔍 Current Code Evidence Anchors

### Aggregator Runtime
- `crates/z00z_runtime/aggregators/src/consensus_adapter.rs`
- `crates/z00z_runtime/aggregators/src/placement.rs`
- `crates/z00z_runtime/aggregators/src/batch_planner.rs`
- `crates/z00z_runtime/aggregators/src/ingress.rs`
- `crates/z00z_runtime/aggregators/src/recovery.rs`
- `crates/z00z_runtime/aggregators/src/dist_sim.rs`
- `crates/z00z_runtime/aggregators/src/dist_dispatch.rs`
- `crates/z00z_runtime/aggregators/src/dist_scheduler.rs`
- `crates/z00z_runtime/aggregators/src/service.rs`
- `crates/z00z_runtime/aggregators/src/types.rs`
- `crates/z00z_runtime/aggregators/src/lib.rs`

### Local DA And Validator Boundaries
- `crates/z00z_rollup_node/src/da.rs`
- `crates/z00z_runtime/validators/src/checkpoint.rs`
- `crates/z00z_runtime/validators/src/engine.rs`
- `crates/z00z_runtime/validators/src/verdict.rs`

### Simulator And Topology
- `config/hjmt_runtime/sim_5a7s/manifest.json`
- `config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml`
- `crates/z00z_simulator/src/config.rs`
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
- `crates/z00z_rollup_node/src/config.rs`

### Current Test Anchors
- `crates/z00z_runtime/aggregators/tests/test_hjmt_consensus.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_dist_journal.rs`
- `crates/z00z_runtime/aggregators/tests/test_recovery_failover.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_route_rollout.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_dispatch.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_shard_routing.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_publish.rs`
- `crates/z00z_runtime/aggregators/tests/test_publication_binding.rs`
- `crates/z00z_rollup_node/tests/test_da_local_sim.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`
- `crates/z00z_rollup_node/tests/test_rollup_theorem_guard.rs`
</current_code_refs>

<plan_waves>
## 🌊 Plan Waves

- Wave 1: `067-01` through `067-04`
  - terminology cleanup
  - first-class quorum artifacts
  - replay verifier
  - certificate-producing commit integration
- Wave 2: `067-05` through `067-07`
  - independent `scenario_11`
  - join/removal/rotation and failover matrix
  - validator/theorem/local-DA certificate binding
- Wave 3: `067-08` through `067-09`
  - real signature seam and transport trait
  - local simulated BFT/Celestia backend behind the proven subject interface
</plan_waves>

<task_inventory>
## 🗂️ Canonical Task Inventory

- `PHASE-0`: prevent concept drift and rename live `standby` debt.
- `PHASE-1`: make commit subjects and certificates first-class artifacts.
- `PHASE-2`: make secondary votes meaningful through independent replay.
- `PHASE-3`: connect the current consensus seam to the new artifact model.
- `PHASE-4`: prove the end-to-end local package-to-validator path in
  `sim_5a7s`.
- `PHASE-5`: make join/removal/rotation and takeover safe under the new
  certificate model.
- `PHASE-6`: require the quorum artifact in local DA publication and validator
  acceptance.
- `PHASE-7`: add real signature and transport seams without bypassing local
  replay.
- `PHASE-8`: add simulated network-BFT and Celestia-style local backends behind
  the already-proven subject interface.
</task_inventory>

<simulation_register>
## 🔬 Local Full-System Simulation Closure Register

Local deterministic simulation is mandatory for:

- replication and quorum formation;
- conflict detection and same-term freeze;
- secondary catch-up and stale-secondary rejection;
- route rollout and dispatch ownership;
- membership join/removal and planned rotation;
- restart and publication resume after crash;
- partition, heal, and offline-member behavior;
- divergent roots, lineage drift, and digest drift;
- local DA binding, resolve, and validator theorem alignment;
- fault telemetry and honest scenario reporting.

The only allowed simulated boundaries are:

- external transport;
- remote process boundary;
- external DA transport;
- wall-clock or fault scheduler;
- unavailable third-party network.
</simulation_register>

<verification_checklist>
## ✅ Verification Checklist

Each numbered plan verify block must require:

1. `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
2. relevant targeted `cargo test` commands
3. `cargo test --release` when Rust or test-affecting changes are relevant
4. `/GSD-Review-Tasks-Execution` at least 3 times in YOLO mode, or an explicit
   workspace-first fallback note if the runner is unavailable
5. nested use of `doublecheck`, smart tests, spec-to-code compliance, and Z00Z
   verification gates where relevant

Planning is invalid if any required group lacks inputs, outputs, artifacts,
tests, results, acceptance tests, negative tests, simulation gate, or evidence
gate.
</verification_checklist>
