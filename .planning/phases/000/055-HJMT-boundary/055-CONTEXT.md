# Phase 055: HJMT Boundary - Context

**Gathered:** 2026-06-09
**Status:** ready-for-execution-planning
**Source:** deep read of `055-TODO.md`, `Z00Z-HJMT-Upgrade.md`,
`Z00Z-HJMT-Fixture-Checklist.md`, `Z00Z-HJMT-Key-Terms.md`, and the live
repository seams that already own settlement proofs, storage backends, runtime
planning, and `scenario_1` evidence

<domain>

## Phase Boundary

Phase 055 freezes and then implements the batch-proof boundary on top of the
already-live generalized settlement backend shipped by Phase 053 and the
storage/runtime seam split hardened by Phase 054. The packet has two distinct
responsibilities:

- Phase 1 of `055-TODO.md` is satisfied by this planning packet itself:
  `055-CONTEXT.md`, `055-SOURCE-AUDIT.md`, and `055-TEST-SPEC.md` together
  freeze ownership, compatibility, fixture layout, benchmark ownership, and
  acceptance vocabulary before feature code lands.
- Phase 2 of `055-TODO.md` is executed by the numbered plans
  `055-01-PLAN.md` through `055-04-PLAN.md`. Those plans must deliver live
  production batch-proof code, deterministic fixtures, fail-closed verifier
  behavior, benchmark evidence, and strengthened Stage 13 scenario checks.

Phase 055 is not a sharding rollout phase. It must reserve the contract space
required by the HJMT upgrade paper without pretending that root-of-shard-roots
publication, route-table migration, multi-aggregator failover, or replicated
journal behavior are already live. Those areas are inventory and evidence
anchors only unless a later execution packet promotes them with real code and
tests.

</domain>

<decisions>

## Implementation Decisions

### D-01: The packet itself is the Phase 1 contract-freeze deliverable
- Do not create a second "plan for planning" layer.
- Treat `055-CONTEXT.md`, `055-SOURCE-AUDIT.md`, and `055-TEST-SPEC.md` as the
  frozen artifact pack required by `055-TODO.md` Phase 1.
- Treat the numbered plans only as the Phase 2 live-code execution slices.

### D-02: `ProofBlob` stays unchanged and remains the single-path authority
- `ProofBlob` keeps its current bytes, decode rules, verification semantics,
  and public meaning.
- `BatchProofBlobV1` is additive. It must not silently widen, rename, or
  replace `ProofBlob`.

### D-03: `BatchProofBlobV1` is storage-owned, positional, and exact
- The canonical batch proof contract lives in `z00z_storage`.
- Version 1 uses exact positional binary encoding. It must not use maps, TLV
  bags, optional field reordering, or backend-specific encodings.
- Parser limits are part of the public contract and must run before
  allocation-heavy work.

### D-04: Use an explicit wire-to-live family mapping layer
- The upgrade paper uses `asset/right` vocabulary for batch-proof wire tags.
- The live repository currently uses `SettlementLeafFamily::{Terminal, Right}`.
- Phase 055 must add an explicit wire-tag mapping layer instead of triggering a
  repo-wide rename wave inside the batch-proof phase.
- The wire contract may use `asset/right` tags while the live semantic mapping
  continues to target `Terminal/Right`.
- The harness-family rename boundary is frozen separately in
  `055-RENAME-MAP.md`: rename only family-level settlement bench homes and
  output surfaces there, and do not use that map to justify renaming workload
  labels such as `inclusion_asset` or fixture helpers such as `asset_seed`.

### D-05: Root-generation support is live-only in Phase 055
- The current live code exposes `RootGeneration::SettlementV1`.
- Batch-proof version 1 must reserve the upgrade paper's generation and
  shard-context fields, but Phase 055 may only emit and accept the current
  non-sharded live generation.
- Any partial shard context, unsupported generation, or future
  root-of-shard-roots claim must fail closed.

### D-06: Version 1 batch proofs are one family per envelope and atomic
- A single `BatchProofBlobV1` may contain only one `HjmtProofFamily`.
- Verification is all-or-nothing. The verifier may report the first failure
  class, but it must not return a partially accepted path set.

### D-07: The first builder must reuse current `ProofBlob` truth
- The first production builder must derive batch proofs from the current
  storage-owned single-path proof contexts that are already proven by
  `ProofBlob`.
- Witness reuse is allowed only when the reused byte segments are exactly
  equivalent under the canonical encoding.
- Phase 055 must not introduce a second independent proof engine.

### D-08: Runtime placement and routing stay runtime-owned
- `z00z_runtime/aggregators` continues to own routing, placement, `ShardId`,
  `BatchRoute`, `BatchPlanned`, and executor placement metadata.
- `z00z_storage` may carry optional shard-context fields in the batch-proof
  wire contract, but it must not import runtime planner types or become the
  routing authority.

### D-09: `StorageBackend` and `JournalBackend` are already the only durable seam
- Phase 055 must freeze and use the existing `crates/z00z_storage/src/backend`
  seam.
- Do not invent another backend abstraction, another recovery authority, or a
  parallel journal interface inside the batch-proof work.

### D-10: The batch-proof owner homes are storage-local and narrow
- Public wire contract live owner home:
  `crates/z00z_storage/src/settlement/proof_batch.rs`.
- Verifier live owner home:
  `crates/z00z_storage/src/settlement/proof_batch_verify.rs`.
- Builder and store integration live owner home:
  `crates/z00z_storage/src/settlement/hjmt_batch_proof.rs`.
- If later refactoring co-locates some of this logic, it must preserve the
  same responsibility split and keep runtime crates out of proof parsing,
  verification, and batch construction.
- These files now exist as live Phase 055 owner homes. The authoritative live
  settlement surface is `settlement/mod.rs`, `proof.rs`, `proof_batch.rs`,
  `proof_batch_verify.rs`, `store.rs`, `hjmt_proof.rs`, and
  `hjmt_batch_proof.rs`.

### D-11: Keep the current independent batch baseline as a first-class comparator
- `SettlementStore::settlement_proof_blobs(&[SettlementPath]) -> Vec<ProofBlob>`
  remains live and unchanged.
- Add a new explicit batch API for `BatchProofBlobV1` instead of overloading
  the current independent-proof helper.
- Every benchmark and scenario report must compare:
  one `ProofBlob`, current `Vec<ProofBlob>`, and the new `BatchProofBlobV1`.

### D-12: Fixture bytes are live artifacts, not handwritten decoration
- Positive fixtures must be generated from the live builder plus the live
  canonical encoder.
- Tamper fixtures must record one exact mutation point, the exact reject stage,
  and the exact regeneration command.
- Do not create empty placeholder fixture files just to reserve names.

### D-13: Stage 13 is the batch-proof evidence extension point
- Reuse `scenario_1` Stage 13 and its current artifact pack.
- Extend `hjmt_settlement_examples.json`, `hjmt_tamper_report.json`,
  `hjmt_proof_size_report.json`, and runner verification in place.
- Do not create a second scenario lane, a second simulator binary, or a
  parallel "batch proof demo" authority.

### D-14: Bench ownership stays in the existing consolidated bench homes
- The logical benchmark lanes `hjmt_batch_proof_bytes` and
  `hjmt_batch_verify` belong in the live bench homes
  `crates/z00z_storage/benches/settlement_proofs.rs` and
  `crates/z00z_storage/benches/settlement_hjmt.rs`.
- Do not create standalone bench crates only to match paper-friendly names.
- `test_bench_lanes.rs` remains the source-shape guard for lane presence.

### D-15: Live command and feature names beat stale planning drift
- The current live fast-test feature is `test-params-fast`, not `test-fast`.
- The current simulator debug feature is `wallet_debug_tools`, not
  `wallet_debug_dump`.
- Phase 055 verification blocks must use the live feature names from the
  current manifests instead of copying stale names from older packets.

### D-16: No scaffolding, placeholder suites, or fake closeout claims
- Do not create empty test files, fake implementation shells, or paper-only
  authority layers for the owner-home names frozen by Phase 1.
- Live code claims in Phase 055 are not limited to the literal new files named
  by Phase 2; every whitepaper or design requirement that constrains the
  batch-proof boundary, migration reject behavior, backend boundary, benchmark
  evidence, or Stage 13 evidence remains live scope and must be discharged on
  the current owner seams.

### D-17: Mandatory verification order for every Rust or test-affecting auto task
- Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  first.
- Then run the broad release cargo gate `cargo test --release`.
- Then run the narrow storage, simulator, bench-compile, or scenario commands
  that correspond to the current slice.
- Then run `/GSD-Review-Tasks-Execution` in YOLO mode at least three times and
  continue until at least two consecutive runs show no significant issues.

### D-18: Path drift in the upgrade paper must resolve to current live paths
- Treat the paper's evidence-map paths as informative when they differ from the
  live worktree.
- Current live owners are:
  `z00z_rollup_node/src/runtime.rs`,
  `z00z_runtime/aggregators/src/{lib.rs,batch_planner.rs,placement.rs,shard_exec.rs,types.rs}`,
  `z00z_storage/src/settlement/store.rs`,
  `z00z_storage/src/settlement/hjmt_*.rs`,
  and `z00z_storage/src/backend/{mod.rs,memory.rs,redb/mod.rs}`.
- Do not create mirror files that only exist to satisfy a stale paper path.

### D-19: Sharding, failover, backend-boundary, and migration statements stay live authority in Phase 055
- `test_hjmt_batch_commit.rs`, `test_hjmt_batch_recovery.rs`,
  `test_hjmt_storage_boundary.rs`, `test_hjmt_backend_conformance.rs`,
  `test_hjmt_shard_routing.rs`, `test_hjmt_failover_same_lineage.rs`,
  `test_hjmt_split_brain_fencing.rs`, `test_hjmt_multi_aggregator_sim.rs`,
  `test_hjmt_root_generation.rs`, `test_hjmt_historical_proofs.rs`,
  `test_hjmt_transition_proofs.rs`, and `test_hjmt_privacy_regression.rs` are
  mandatory inventory anchors from Phase 1.
- They are not authorized to become empty placeholder suites, parallel harnesses,
  or second authority paths in Phase 055.
- When Phase 055 touches those concerns, satisfy them through the current
  storage, runtime, simulator, bench, or guardrail seams in this packet. Do not
  relabel them as a separate future-only backlog inside Phase 055.

### D-20: Every dash-list bullet class in `055-TODO.md` remains normative
- Phase 055 review must treat every dash-list bullet class in `055-TODO.md` as
  in-scope planning authority, not only the unchecked implementation bullets.
- The current strict review pass counts `77` dash-list bullet classes in
  `055-TODO.md`; this count is review evidence, not a new source of truth.
- If `055-TODO.md` changes, the strict coverage evidence in
  `055-SOURCE-AUDIT.md`, the review artifact, and the numbered-plan coverage
  contracts must be refreshed in the same change set.

### D-21: No duplicate code path, mirror abstraction, or parallel authority layer
- Phase 055 must extend the current storage, simulator, runtime, and bench
  seams in place when they already own the required responsibility.
- Do not plan or introduce a mirror proof engine, a second storage proof API
  that duplicates existing semantics, a second scenario lane, a second bench
  harness, a second durable seam, or a paper-only compatibility layer when the
  live codebase already has an owner home.
- If a target file or split is only proposed rather than verified in the live
  codebase, label it as proposed and keep the existing live owner authoritative
  until implementation proves the move.

### The agent's Discretion
- Exact helper names, the final split between `proof_batch.rs` and
  `proof_batch_verify.rs`, the precise fixture metadata format, and the final
  report schema field names are implementation choices as long as they preserve
  the decisions above, the HJMT upgrade paper, and the live storage/runtime
  boundaries already shipped in Phases 053 and 054.

</decisions>

<canonical_refs>

## Canonical References

Downstream executors MUST read these before implementing the relevant slices.

### Phase authority
- `.planning/phases/055-HJMT-boundary/055-TODO.md` - canonical Phase 055
  backlog, section scope, expected deliverables, required tests, fixture IDs,
  completion contract, and release gate.
- `.planning/phases/055-HJMT-boundary/055-CONTEXT.md` - locked Phase 055
  decisions, live-path corrections, ownership map, and coverage contract.
- `.planning/phases/055-HJMT-boundary/055-SOURCE-AUDIT.md` - source-to-plan
  coverage map for this packet.
- `.planning/phases/055-HJMT-boundary/055-TEST-SPEC.md` - frozen test,
  benchmark, fixture, and Stage 13 evidence inventory for this phase.
- `.planning/ROADMAP.md` and `.planning/STATE.md` - active planning state and
  roadmap positioning.

### Normative paper and glossary authority
- `docs/tech-papers/Z00Z-HJMT-Upgrade.md` - global rules, batch-proof contract,
  verifier requirements, benchmark matrix, evidence discipline, and module
  suggestions.
- `docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md` - `BPB-G-*`,
  `BPB-T-*`, completion contract, and release gate.
- `docs/tech-papers/Z00Z-HJMT-Key-Terms.md` - naming authority for
  `BatchProofBlob`, `ProofBlob`, `StorageBackend`, `JournalBackend`,
  route-table terminology, and versioned wire names.
- `docs/tech-papers/Z00Z-HJMT-Design.md` - live settlement semantics inherited
  from the production generalized backend.

### Predecessor phase anchors
- `.planning/phases/000/053-HJMT-Backend/053-CONTEXT.md` - live settlement
  contracts, proofs, cache, scheduler, journal, reload, and scenario authority.
- `.planning/phases/000/053-HJMT-Backend/053-TEST-SPEC.md` - current live test
  home ownership that Phase 055 must extend instead of duplicating.
- `.planning/phases/000/054-Refactor-Crates/054-CONTEXT.md` - current storage
  backend seam and runtime ownership split already accepted by the repo.

### Live storage anchors
- `crates/z00z_storage/src/settlement/mod.rs` - exported settlement surface.
- `crates/z00z_storage/src/settlement/proof.rs` - current `ProofBlob` and
  single-path proof family semantics.
- `crates/z00z_storage/src/settlement/store.rs` - storage semantic facade.
- `crates/z00z_storage/src/settlement/hjmt_proof.rs` - current
  `settlement_proof_blob` and `settlement_proof_blobs` baseline.
- `crates/z00z_storage/src/settlement/hjmt_commit.rs` - commit ordering
  and recovery-adjacent write path.
- `crates/z00z_storage/src/settlement/hjmt_journal.rs` - live journal
  rows and root-generation/proof-version bindings.
- `crates/z00z_storage/src/settlement/hjmt_scheduler.rs` - bounded
  scheduler metrics and queue semantics.
- `crates/z00z_storage/src/backend/mod.rs` - current `StorageBackend` and
  `JournalBackend` seam.
- `crates/z00z_storage/src/backend/memory.rs` and
  `crates/z00z_storage/src/backend/redb/mod.rs` - live backend implementations.
- `crates/z00z_storage/src/settlement/README.md` and
  `crates/z00z_storage/src/settlement/root_types.md` - public contract and root
  taxonomy docs that must stay consistent.

### Live runtime and node anchors
- `crates/z00z_runtime/aggregators/src/lib.rs` - runtime aggregation exports.
- `crates/z00z_runtime/aggregators/src/types.rs` - live `ShardId`,
  `BatchRoute`, `BatchPlanned`, and route-related runtime types.
- `crates/z00z_runtime/aggregators/src/batch_planner.rs` - routing and
  planner-authority ownership.
- `crates/z00z_runtime/aggregators/src/placement.rs` and
  `crates/z00z_runtime/aggregators/src/shard_exec.rs` - placement and executor
  ownership that storage must not absorb.
- `crates/z00z_rollup_node/src/runtime.rs` and
  `crates/z00z_rollup_node/README.md` - orchestration root and dependency flow.

### Live simulator and report anchors
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml` - Stage 13 output
  location and scenario configuration.
- `crates/z00z_simulator/src/scenario_1/stage_13_utils/hjmt_examples.rs` -
  current Stage 13 example generation.
- `crates/z00z_simulator/src/scenario_1/stage_13_utils/report.rs` - current
  report schemas to extend.
- `crates/z00z_simulator/src/scenario_1/stage_13_utils/tamper.rs` - current
  tamper evidence owner home.
- `crates/z00z_simulator/src/scenario_1/runner_verify.rs` - current artifact
  verification contract.

### Live tests and benches to extend
- `crates/z00z_storage/tests/test_live_guardrails.rs`
- `crates/z00z_storage/tests/test_hjmt_live_proof_families.rs`
- `crates/z00z_storage/tests/test_hjmt_proofs.rs`
- `crates/z00z_storage/tests/test_bench_lanes.rs`
- `crates/z00z_simulator/tests/test_scenario_settlement.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- `crates/z00z_storage/benches/settlement_proofs.rs`
- `crates/z00z_storage/benches/settlement_hjmt.rs`
- `crates/z00z_storage/benches/settlement_benches.md`
- `crates/z00z_storage/scripts/run_storage_settlement_bench.py`

</canonical_refs>

<phase1_artifact_pack>

## Phase 1 Artifact Pack

### Acceptance Vocabulary

| Term | Meaning in Phase 055 |
| --- | --- |
| `Specified contract` | The wire contract, owner homes, fixture layout, and verification obligations are frozen in packet docs, but production code is not yet landed. |
| `Prototype` | Code compiles and demonstrates local behavior, but fixture authority, benchmark evidence, or scenario evidence is not yet complete. |
| `Verified slice` | The relevant narrow tests, broad cargo gate, and review loop are green for one numbered slice. |
| `Integrated upgrade` | The full `BatchProofBlobV1` path from builder through verifier, fixtures, benches, and Stage 13 evidence is wired end to end without breaking `ProofBlob`. |
| `Release-ready` | Completion contract, release gate, deterministic fixtures, scenario evidence, benchmark evidence, and guardrail/docs updates are all green with no placeholder authority left. |

### Cross-Crate Ownership Map

| Concern | Primary owner | Explicit non-owner rule |
| --- | --- | --- |
| `BatchProofBlobV1` wire bytes, parser limits, verifier | `z00z_storage` | Runtime, node, wallets, and simulator must not own proof parsing or acceptance semantics. |
| Current single-path `ProofBlob` compatibility | `z00z_storage` | Phase 055 must not reinterpret `ProofBlob` in runtime or simulator code. |
| Route planning, placement, and `ShardId` authority | `z00z_runtime/aggregators` | Storage may carry reserved shard fields, but it is not the route planner. |
| Orchestration and service wiring | `z00z_rollup_node` | Storage must not become the node composition root. |
| Stage 13 evidence generation and artifact verification | `z00z_simulator` | It consumes live storage APIs but is not the proof-truth owner. |
| Durable KV and journal seam | `z00z_storage::backend` | No second seam or vendor-leak public API is allowed in Phase 055. |

### `BatchProofBlobV1` Layout Plan

| Section | Planned owner | Contract notes |
| --- | --- | --- |
| Header | `proof_batch.rs` | `encoding_version`, transcript domain, proof family, root generation, public root, policy binding, parser limits; exact field order is frozen. |
| Path table | `proof_batch.rs` | Canonical path ordering, explicit leaf-family mapping, optional shard context reserved but unsupported for live generation. |
| Witness DAG | `proof_batch.rs` plus `proof_batch_verify.rs` | Deduplicated witness nodes with bounded indexes and domain checks; no implicit reconstruction shortcuts. |
| Opening table | `proof_batch.rs` | One frozen payload contract per opening kind; no mixed family envelope in V1. |
| Reference table | `proof_batch.rs` plus `proof_batch_verify.rs` | Bounded witness indexes, deterministic reuse, and fail-closed reference validation. |

### Compatibility Matrix

| Surface | Scope | Phase 055 rule | Evidence owner |
| --- | --- | --- | --- |
| `ProofBlob` | Single path | Must remain byte- and semantics-compatible. | Existing proof suites plus `test_live_guardrails.rs` |
| `Vec<ProofBlob>` | Current multi-path baseline | Must remain callable and benchmarked as the independent baseline. | `test_hjmt_proofs.rs`, `test_bench_lanes.rs`, `settlement_proofs.rs`, `settlement_hjmt.rs` |
| `BatchProofBlobV1` | New shared multiproof | Must be additive, deterministic, fail-closed, and benchmarked against the independent baseline. | New batch-proof suites plus Stage 13 and bench lanes |

### Root-Generation Migration Vector Plan

| Vector | Phase 055 behavior | Phase 055 evidence owner |
| --- | --- | --- |
| Last pre-shard live settlement root | Supported via current `RootGeneration::SettlementV1` mapping only. | `055-03` positive fixtures |
| First route-table generation | Live reject vector in Phase 055 V1; not accepted by the live verifier. | `055-03` migration vectors plus `055-04` Stage 13 tamper evidence |
| First shard-root leaf publication | Live reject vector in Phase 055 V1; not accepted by the live verifier. | `055-03` migration vectors plus `055-04` Stage 13 tamper evidence |
| First root-of-shard-roots public root | Explicit reject in Phase 055. | `055-03` migration vectors plus `055-04` Stage 13 tamper evidence |

### Backend Conformance Plan

- Shared semantic conformance remains anchored to the existing live seam:
  `StorageBackend`, `JournalBackend`, `memory`, and `redb`.
- Equal roots under equal operations, equal journal replay behavior, equal
  reject behavior, and no backend-vendor leakage in public proof APIs are live
  Phase 055 authority requirements.
- Prove those requirements by extending the current backend seams, guardrails,
  benches, fixtures, or runner evidence whenever the additive batch-proof path
  touches them. Do not scaffold them as empty files or satisfy them with a
  second backend abstraction.

### Multi-Aggregator Simulation Plan

- A one-machine multi-aggregator harness remains the canonical owner shape if a
  dedicated suite materializes, and it must compose `z00z_rollup_node`,
  runtime placement, and same-lineage journal checks.
- The required lawful cases are: standby takeover under the same shard lineage,
  shard-unavailable retry without silent reroute, and split-brain rejection.
- In Phase 055 these cases remain live authority on the current runtime and
  scenario seams: the new batch-proof surface must not hide silent reroute,
  split-brain acceptance, or lineage drift.

### Benchmark Report Template

Every measured batch-proof report must include:

- `proof_surface`: one `ProofBlob`, current `Vec<ProofBlob>`, or
  `BatchProofBlobV1`
- `path_count`
- `path_shape`: clustered or scattered
- `proof_family`
- `cache_mode`
- `persistence_mode`
- `serialized_bytes`
- `bytes_per_path`
- `prove_time_us`
- `verify_time_us`
- `peak_memory_bytes`

### Benchmark Home Plan And Current Owner Mapping

Phase 1 also freezes the benchmark-home inventory from `055-TODO.md`. These
names are logical benchmark identities, not permission to create standalone
bench files when the live consolidated harness already owns the concern.

| Inventory benchmark name | Canonical owner home | Current live anchor |
| --- | --- | --- |
| `hjmt_batch_proof_bytes.rs` | logical lane in `crates/z00z_storage/benches/settlement_proofs.rs` with `settlement_benches.md` and `test_bench_lanes.rs` | the existing proof bench harness already owns proof byte totals and comparison reporting |
| `hjmt_batch_verify.rs` | logical lane in `crates/z00z_storage/benches/settlement_proofs.rs` with `settlement_benches.md` and `test_bench_lanes.rs` | the existing proof bench harness already owns proof verification timing and reject-path reporting |
| `hjmt_bucket_delta_commit.rs` | logical lane in `crates/z00z_storage/benches/settlement_hjmt.rs` plus `settlement_benches.md` | `hjmt_commit.rs`, `settlement_hjmt.rs`, and the current storage bench runner already own bucket-local commit work |
| `hjmt_backend_boundary.rs` | cross-backend comparison lane in `crates/z00z_storage/benches/settlement_hjmt.rs` plus `run_storage_settlement_bench.py` and `settlement_benches.md` | `backend/{mod.rs,memory.rs,redb/mod.rs}` plus the current bench runner already own backend-mode and durability comparison plumbing |
| `hjmt_shard_parallel_commit.rs` | logical lane in `crates/z00z_storage/benches/settlement_shard.rs` plus `settlement_benches.md` | `settlement_shard.rs` already owns wide and shard-shaped storage workloads without introducing a second shard bench harness |
| `hjmt_root_of_roots_publish.rs` | logical lane in `crates/z00z_storage/benches/settlement_shard.rs` plus `settlement_benches.md` | `settlement_shard.rs` is the current bench family closest to shard-growth and publication-cost evidence |
| `hjmt_transition_locality.rs` | logical lane in `crates/z00z_storage/benches/adaptive_policy_bench.rs` plus `settlement_benches.md` | `adaptive_policy_bench.rs` already owns fixed baseline, split, merge, and policy-transition locality evidence |

### Fixture Layout Plan

```text
crates/z00z_storage/tests/fixtures/hjmt_upgrade/
  proof_blob_single/
  proof_blob_batch_independent/
  batch_proof_v1_positive/
  batch_proof_v1_negative/
  root_generation_migration/
```

Each fixture must carry scenario notes, regeneration instructions, expected
root, expected verdict, and the exact reject stage when it is negative.

</phase1_artifact_pack>

<todo_bullet_coverage>

## TODO Bullet Coverage Contract

| TODO section | Packet owner | Coverage rule |
| --- | --- | --- |
| `Phase 1. Contract Freeze And Artifact Pack` | `055-CONTEXT.md`, `055-SOURCE-AUDIT.md`, `055-TEST-SPEC.md` | All deliverable bullets, owner-home inventory bullets, completion-contract bullets, and release-gate bullets are satisfied by this packet and remain normative for every later execution slice. |
| `Phase 2. Batch Proof Contract And Shared Multiproof` | `055-01-PLAN.md` to `055-04-PLAN.md` | All codec, ordering, verifier, witness reuse, fixture, benchmark, compatibility, and test bullets are mandatory across the numbered plans. |

### Phase 1 Primary Upgrade Sections Locked By This Packet

The Phase 1 planning packet itself must keep these exact source sections active
and reflected in packet content:

- `1. Purpose And Upgrade Boundary`
- `1.4 Core Architecture Decision`
- `1.5 Parallel Shard Reality Check`
- `1.6 Upgrade Boundary And Evidence Discipline`
- `1.7 Whole-System Structure View`
- `1.7.1 C4 Component View: Whole-System Roles`
- `1.8 C4 Component Reading Map`
- `9. Scorecard And Measurement Plan`
- `11. Implementation Roadmap`
- `11.1 Roadmap Dependency Discipline`
- `11.1.1 Mermaid Flow View: Upgrade Dependency Chain`
- `12. Test And Benchmark Plan`
- `12.1 Evidence Gaps`
- `Appendix B. Repository Evidence Map`
- `Appendix C. Design Artifact Requirements`
- `Appendix E.1 Suggested Module Boundaries`
- `Appendix E.2 First Slice Implementation Order`
- `Appendix E.3 Test Vector Layout`
- `Appendix E.6 Cross-Crate Module Ownership`
- `Appendix E.7 Cross-Crate Execution Order`
- `Appendix F. Discussion Coverage Matrix`
- `Appendix F.1 Traceability For Sharding And Storage Recommendations`

The fixture-scope sections locked by the Phase 1 packet are:

- `Completion Contract`
- `Release Gate`

### Mandatory Global Cross-Read Before Implementation

Downstream executors MUST read and keep these sections active before any Phase
055 implementation, verification, review, or summary update. This list is
exact on purpose so local task focus cannot cause concept drift away from the
source docs.

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

### Mandatory Phase 2 Cross-Read For Every Numbered Plan

Even when one numbered plan focuses on one narrow slice, the following Phase 2
sections stay active and MUST NOT be treated as out of scope just because a
local `read_first` block emphasizes a smaller subset:

- `3. Upgrade 1: Shared Hierarchical Multiproof`
- `3.1 Required Format`
- `3.1.1 Exact Codec Contract For BatchProofBlobV1`
- `3.1.2 Exact Codec Contract For Nested Batch Tables`
- `3.2 Canonical Ordering`
- `3.3 Verification Algorithm`
- `3.4 Witness Reuse Rules`
- `3.5 Acceptance Evidence`
- `3.6 Verifier Safety Requirements`
- `3.7 Implementation Guidance`
- `3.8 C4 Component View: Batch Proof Contract`
- `9.1 Benchmark Matrix`
- `9.2 Claim Gate`
- `9.3 Score Claim Discipline`
- `12. Test And Benchmark Plan`
- `12.1 Evidence Gaps`
- `Appendix D.1 Batch Proof Envelope Skeleton`
- `Appendix D.2 Fail-Closed Batch Verifier Skeleton`
- `Appendix E.2 First Slice Implementation Order`
- `Appendix E.3 Test Vector Layout`

These global and Phase 2 cross-read sets are mandatory for every applicable
plan:

- key terms and inherited base constraints;
- prohibited changes and verified current baseline;
- fail-closed rules, narrow versioned contracts, commitment boundary, and
  contract discipline;
- correctness/security/privacy checklist and evidence mapping discipline;
- readiness definition and completion discipline;
- appendix-level review and evidence checklists;
- fixture-checklist completion contract and release gate.

</todo_bullet_coverage>

<specifics>

## Specific Ideas

- Add `BatchProofBlobV1` as a public storage contract next to `ProofBlob`, not
  inside it.
- Keep the first builder simple and provable: derive from current single-path
  proof contexts and deduplicate only byte-identical witness material.
- Use dedicated batch-proof tests for canonical bytes and negative cases, but
  reuse existing guardrail, bench-lane, and Stage 13 suites for repo-wide
  safety.
- Extend Stage 13 with representative clustered and scattered batch checks, but
  keep the large benchmark matrix in the storage bench harness rather than
  bloating simulator runtime.
- Treat every shard-aware or failover-aware item as live authority on the
  current owner seams, not as permission to land a placeholder implementation
  or a second harness in Phase 055.

</specifics>

<live_boundary_limits>

## Live Boundary Limits

- live shard-route tables, shard-root leaves, and root-of-shard-roots proof
  acceptance remain explicit fail-closed reject surfaces in Phase 055 V1 until
  a later versioned contract adds support;
- same-lineage failover and split-brain behavior remain live guardrail
  requirements on current runtime and scenario seams, even where Phase 055 does
  not introduce a new dedicated harness;
- backend conformance remains a live semantic requirement on the current
  durable seam and must not be bypassed by the additive batch-proof path;
- replicated journal or WAL choices beyond the existing local durable baseline
  remain outside the V1 batch-proof acceptance contract.

</live_boundary_limits>

---

*Phase: 055-HJMT-boundary*
*Context gathered: 2026-06-09 via full local source read*
