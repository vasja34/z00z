---
phase: 052-HJMT-Backend
artifact: planning-context
status: ready-for-execution-planning
updated: 2026-05-28
source: 052-TODO.md plus repository-backed JMT design, Phase 051 handoff, and live storage or simulator references
---

# 052 HJMT Backend Context

## Source Inputs

This phase is planned from these repository-backed sources:

- `.planning/phases/052-HJMT-Backend/052-TODO.md`
- `docs/Z00Z-JMT-Design.md`
- `.planning/phases/000/051-HJMT-Facade/051-TODO.md`
- `.planning/phases/000/051-HJMT-Facade/051-CONTEXT.md`
- `.planning/phases/000/051-HJMT-Facade/051-SUMMARY.md`
- `crates/z00z_storage/tests/test_phase051_golden_corpus.rs`
- `crates/z00z_storage/tests/test_phase051_guardrails.rs`
- `crates/z00z_storage/benches/assets/shard.rs`
- `crates/z00z_storage/benches/assets/nested.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/storage_view.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_preparation_core.rs`
- `crates/z00z_simulator/src/scenario_1/stage_5_utils/transfer_lane_impl.rs`
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/test_bundle_lane_impl_suite.rs`
- `crates/z00z_simulator/src/scenario_1/stage_7.rs`
- `crates/z00z_simulator/src/scenario_1/stage_11_utils/jmt_wallet_scan.rs`
- `crates/z00z_simulator/src/scenario_1/stage_12.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_utils/storage.rs`
- `crates/z00z_simulator/src/scenario_1/runner_verify.rs`

## Phase Boundary

Phase 052 implements the real bucketed root-chained HJMT forest backend behind
the Phase 051 storage facade. It keeps `AssetTreeBackend`, `AssetStore`,
`AssetStateRoot`, `CheckRoot`, `ProofBlob`, `chk_blob`, and storage-owned
checkpoint contracts as the only caller-facing semantic authority seam.

The phase is complete only when the real forest backend exists behind that
seam, compatibility and forest modes are semantically equivalent on the
required corpus, deletion and non-existence proofs either verify fail-closed
or remain explicitly unsupported, reload or checkpoint or path-index rebuild
behavior is green, benchmark evidence exists for async multi-insert and
multi-delete plus proof-size obligations, and simulator `scenario_1` still
consumes storage semantics without learning physical layout authority.

## Scope Reconciliation

`052-TODO.md` names twelve backend execution tasks plus five deferred
first-class follow-up tasks, along with the design-coverage, test-matrix,
benchmark, and simulator obligations needed to turn the HJMT design into an
implementation phase. This planning packet preserves every TODO task and
reconciles them into eleven sequential waves:

1. backend mode selection plus fixed bucket policy;
2. physical forest tree store plus deterministic batch planner;
3. forest commit journal plus reload or path-index rebuild;
4. storage-owned proof envelope plus deletion or non-existence proof families;
5. dual-backend equivalence plus checkpoint or downstream guardrails;
6. rollout configuration plus benchmark evidence plus final verification;
7. green-state audit for the implemented fixed-bucket backend work;
8. adaptive bucket split, merge, and migration proof follow-up candidate;
9. bucket occupancy metadata privacy follow-up candidate;
10. generalized settlement-root migration follow-up candidate;
11. `RightLeaf` plus `FeeEnvelope` protocol follow-up candidate.

Numbering note: `052-TODO.md` task ids and `052-*-PLAN.md` file ids are
separate ledgers after backend plan `052-06`. TODO tasks `052-01` through
`052-12` define the backend implementation work and map into plan files
`052-01` through `052-06`. TODO tasks `052-13` through `052-17` map into plan
files `052-07` through `052-11` for green-state audit and first-class follow-up
candidates. When a table says TODO task `052-07`, it means deletion and
non-existence proof semantics, not `052-07-PLAN.md`.

This packet is intentionally full-backend scope. Unlike Phase 051, it does not
stop at the facade or compatibility oracle. It implements the real forest
backend while preserving the Phase 051 boundary decisions.

`052-TODO.md` remains the normative coverage ledger for design-validation,
test-matrix, simulator-matrix, and phase-boundary obligations. This context
mirrors that ledger through `TODO Task Coverage`, `Source Coverage Matrix`,
`JMT Requirement Coverage`, `Test And Benchmark Coverage`, and
`Scenario 1 Surface Coverage`, while the numbered plans turn the same ledger
into executable slices.

Plans `052-07` through `052-11` are not live runtime expansion. They preserve
first-class future protocol work after the fixed-bucket forest backend is
green, so the project does not forget adaptive buckets, occupancy metadata,
generalized root migration, `RightLeaf`, or `FeeEnvelope` while Phase 052
stays narrowly implementable.

## Codebase Fit And Non-Duplication Rules

- Extend the existing `AssetTreeBackend`, `AssetStore`, `proof.rs`,
  `store_internal`, `redb_backend*`, and benchmark harness seams before adding
  any new module.
- File targets named `forest_config.rs`, `forest_policy.rs`,
  `forest_store.rs`, `forest_plan.rs`, `forest_commit.rs`,
  `forest_journal.rs`, `forest_proof.rs`, and
  `test_phase052_*.rs` are proposed file homes, not claims that those files
  already exist in the current codebase. If an earlier execution slice lands a
  better-fitting extension point, later plans must extend that landed module
  and update the planning packet instead of creating a duplicate file or logic
  lane.
- Do not duplicate compatibility planner, mutation, proof, reload, or
  checkpoint logic behind renamed forest modules. Forest code must own real
  bucketed semantics, while compatibility remains the migration oracle only.
- Do not create a second proof decoder, checkpoint verifier, or simulator
  authority surface outside storage. Prevent concept drift by keeping semantic
  authority on the Phase 051 facade and storage-owned contracts only.

## Locked Decisions

### Public Semantic Contract

- `AssetStateRoot` remains the only live public asset-state root in this asset-
  centric generation.
- `CheckRoot` remains checkpoint-facing evidence derived from the semantic
  asset root.
- `AssetPath { definition_id, serial_id, asset_id }` remains the caller-facing
  path contract. Buckets are derived internally.
- `ProofBlob`, `chk_blob`, and any forest proof successor remain storage-owned
  proof authority surfaces.
- `SettlementStateRoot`, `RightLeaf`, and `FeeEnvelope` remain future-only
  terms. Phase 052 must not export them as live storage contracts.

### Backend Boundary And Authority

- `AssetTreeBackend` and `AssetStore` remain the only public semantic entry
  point for wallet, validator, runtime, checkpoint, and simulator consumers.
- `CompatibilityBackend` remains the migration oracle and rollback reference,
  not a second long-lived public authority lane.
- `AssetBackendMode` may expose compatibility, forest, and dual-verify modes,
  but mode selection remains storage-owned and configuration-gated.
- No public API may accept `TreeId`, namespace bytes, bucket ids, branch
  ordering, physical key layout, raw backend roots, or raw bucket proofs as
  authority inputs.

### Fixed Bucket Policy

- Phase 052 uses deterministic fixed buckets only.
- `BucketPolicy` is versioned, verifier-visible, and limited to fields needed
  for recomputation and proof decoding.
- `BucketId` is internal to storage APIs except when the storage-owned verifier
  recomputes the derived bucket from `AssetPath` plus committed policy.
- Adaptive bucket split or merge, bucket migration proofs, and public occupancy
  counters remain out of live Phase 052 runtime scope.
- Adaptive bucket migration is captured by `052-08` as future work after
  fixed-bucket benchmark and proof evidence exists.
- Proof-visible bucket occupancy metadata is captured by `052-09` as future
  work that requires design update and privacy review first.

### Forest Runtime And Recovery

- Physical forest commits must happen through independent child JMT state, then
  deterministic parent-root publication.
- Forest updates must persist a durable journal with `Prepared`,
  `ChildrenCommitted`, `ParentsCommitted`, and `RootPublished` states.
- No parent root or exported `AssetStateRoot` may become visible unless child
  roots and journal digests are durable and replay-valid.
- The path index remains rebuildable internal lookup state rather than public
  root truth.

### Proof Contract And Absence Semantics

- The target forest proof envelope is storage-owned and versioned inside
  `z00z_storage`.
- Inclusion proofs must verify definition, serial, bucket, and terminal
  segments against chained semantic roots and committed bucket policy.
- Deletion proofs must bind prior root, deleted path, next root, and affected
  parent-root updates or remain unsupported.
- Non-existence proofs must use canonical default commitments rather than a
  node-local `not found` answer.
- Proof-size evidence is measured from real encodings and is implementation
  evidence only. Phase 052 must not freeze measured byte counts as protocol
  constants.

### Rollout And Benchmark Obligations

- Compatibility remains the default backend until equivalence, guardrails,
  checkpoint, reload, and proof gates are green.
- Forest and dual-verify modes are explicit storage-owned rollout options for
  local validation and CI.
- Benchmark evidence must cover broad and hot-definition and hot-serial
  workloads, multithread async `multi-insert` and `multi-delete`, proof-heavy
  and delete-heavy paths, bucket-width comparison, recovery replay, and proof-
  size reporting for inclusion and non-existence.

### Simulator And Downstream Consumers

- `scenario_1` remains a storage-semantics consumer only.
- Stage 4, Stage 6, Stage 7, Stage 11, Stage 12, Stage 13, and
  `runner_verify` must continue to validate semantic roots, storage-owned
  proof checks, and checkpoint-owned evidence without decoding physical forest
  internals as authority.
- Downstream source-shape guards remain required so wallet, validator, runtime,
  and simulator crates do not become a second proof or checkpoint authority
  lane.

## Current Baseline

The current repository already contains the Phase 051 seam that Phase 052 must
extend rather than replace:

- `crates/z00z_storage/src/assets/store.rs` already defines
  `AssetTreeBackend`, `CompatibilityBackend`, and facade-owned `backend_name`,
  root, check-root, lookup, mutation, proof, and reload hooks.
- `crates/z00z_storage/src/assets/proof.rs` already owns `ProofBlob`,
  `chk_blob`, `chk_blob_item`, compatibility proof-family checks, and the
  semantic/backend-root binding.
- `crates/z00z_storage/src/assets/store_internal/tree_id.rs` and
  `tree_store.rs` already keep logical tree identities and shared JMT storage
  private to storage.
- `crates/z00z_storage/tests/test_phase051_golden_corpus.rs` already provides
  one executable compatibility backend case plus a reserved
  `future-real-forest` slot that Phase 052 must replace with a real backend.
- `crates/z00z_storage/tests/test_phase051_guardrails.rs` already scans
  validators, wallets, and simulator files for forbidden physical-layout
  authority shapes.
- `crates/z00z_storage/benches/assets/shard.rs` and `nested.rs` already expose
  current benchmark lanes, timing probes, and plan/root mode toggles that
  Phase 052 should extend rather than bypass with a disconnected benchmark
  harness.
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_preparation_core.rs`,
  `stage_5_utils/transfer_lane_impl.rs`, `stage_7.rs`,
  `stage_11_utils/jmt_wallet_scan.rs`, `stage_12.rs`,
  `stage_13_utils/storage.rs`, and `runner_verify.rs` already consume storage
  roots and proof checks through storage-owned seams.

## Requirements

| Requirement | Meaning | Planned In |
| --- | --- | --- |
| `PH52-BACKEND-MODE` | Add internal backend mode selection for compatibility, forest, and dual-verify execution without widening the public semantic facade. | `052-01` |
| `PH52-BUCKET-POLICY` | Add deterministic fixed bucket policy, `BucketId`, and `BucketRootLeaf` with verifier-visible encoding and public guardrails. | `052-01` |
| `PH52-FOREST-LAYOUT` | Add the private physical forest tree layout and child-tree storage behind `store_internal`. | `052-02` |
| `PH52-BATCH-PLANNER` | Implement deterministic insert or delete planning across definition, serial, and derived bucket groups with compatibility-equivalent semantics. | `052-02` |
| `PH52-JOURNAL-RECOVERY` | Implement the forest commit journal, child-before-parent publication, crash recovery, and digest mismatch rejection. | `052-03` |
| `PH52-RELOAD-INDEX` | Rebuild the path index from committed leaves and validate reload or checkpoint state from durable forest rows. | `052-03` |
| `PH52-PROOF-ENVELOPE` | Implement the storage-owned forest inclusion proof envelope, chained verification, and reject classes. | `052-04` |
| `PH52-ABSENCE-PROOFS` | Implement or explicitly reject deletion and non-existence proofs fail-closed, including absence-default commitments and replay consistency. | `052-04` |
| `PH52-EQUIVALENCE` | Run compatibility and forest modes through one golden semantic corpus and dual-verify mismatch gate. | `052-05` |
| `PH52-CHECKPOINT-GUARDRAILS` | Keep checkpoint, wallet, validator, runtime, and simulator consumers on storage-owned semantic authority rather than forest layout details. | `052-05` |
| `PH52-ROLLOUT-BENCHMARKS` | Keep compatibility default while adding forest or dual mode plus benchmark evidence for async workloads, recovery, and proof-size obligations. | `052-06` |
| `PH52-CLOSEOUT` | Run the mandatory verification order, scenario validation, and review loops, then record honest follow-ups and closeout evidence. | `052-06` |
| `PH52-GREEN-AUDIT` | Prove plans `052-01` through `052-06` are green by executed evidence and contain no placeholder forest behavior or parallel authority drift. | `052-07` |
| `PH52-ADAPTIVE-BUCKETS-FOLLOWUP` | Capture adaptive split, merge, migration proofs, bucket epochs, historical proofs, and replay rules as a future phase candidate. | `052-08` |
| `PH52-OCCUPANCY-METADATA-FOLLOWUP` | Keep proof-visible bucket occupancy metadata blocked until design update, privacy review, and fail-closed tests exist. | `052-09` |
| `PH52-GENERALIZED-ROOT-FOLLOWUP` | Capture `AssetStateRoot` to `SettlementStateRoot` migration as a separate future protocol phase with generation and rollback rules. | `052-10` |
| `PH52-RIGHTLEAF-FEEENVELOPE-FOLLOWUP` | Capture `RightLeaf` and `FeeEnvelope` protocol work as future scope while preserving their separation. | `052-11` |

## TODO Task Coverage

This matrix is the direct `052-TODO.md` numbered-task audit trail for the
execution packet. Every numbered TODO task must map to one concrete plan task
or an explicit gate in the eleven-plan packet.

| TODO task | Plan coverage | Execution note |
| --- | --- | --- |
| `052-01` Backend selection and forest skeleton | `052-01` task 1 | Add storage-owned backend mode routing without widening the public facade. |
| `052-02` Fixed bucket policy and root leaf types | `052-01` task 2 | Freeze deterministic `BucketPolicy`, `BucketId`, and `BucketRootLeaf`. |
| `052-03` Forest tree store and physical layout | `052-02` task 1 | Build private tree identities and forest storage helpers behind `store_internal`. |
| `052-04` Forest batch planner for inserts and deletes | `052-02` task 2 | Implement real deterministic forest planning and mutation semantics. |
| `052-05` Forest commit journal and recovery state | `052-03` task 1 | Persist journal lifecycle, child-before-parent publication, and replay validation. |
| `052-06` Forest proof envelope and verifier checks | `052-04` task 1 | Add one storage-owned forest inclusion-proof decoder and reject matrix. |
| `052-07` Deletion and non-existence proof semantics | `052-04` task 2 | Implement or explicitly reject fail-closed absence families. |
| `052-08` Reload validation and path-index rebuild | `052-03` task 2 | Rebuild internal path index and validate persisted forest rows. |
| `052-09` Dual-backend equivalence corpus | `052-05` task 1 | Convert the Phase 051 corpus into compatibility, forest, and dual-verify execution. |
| `052-10` Checkpoint and downstream guardrail closure | `052-05` task 2 | Keep checkpoint, wallet, validator, runtime, and simulator on semantic authority only. |
| `052-11` Rollout configuration and benchmark evidence | `052-06` task 1 | Keep compatibility default and record async benchmark plus proof-size evidence. |
| `052-12` Verification closeout | `052-06` task 2 | Run bootstrap-first validation, scenario runs, review loops, and honest closeout evidence. |
| `052-13` Green-state audit for plans `052-01` through `052-06` | `052-07` tasks 1-2 | Verify implemented backend work is green before promoting deferred candidates. |
| `052-14` Adaptive bucket split, merge, and migration proof candidate | `052-08` tasks 1-2 | Record future adaptive migration scope, entry conditions, and test duties. |
| `052-15` Bucket occupancy metadata privacy candidate | `052-09` tasks 1-2 | Keep counters out of proof metadata until design and privacy gates are complete. |
| `052-16` Generalized settlement-root migration candidate | `052-10` tasks 1-2 | Preserve `AssetStateRoot` in Phase 052 and plan root-generation migration separately. |
| `052-17` `RightLeaf` and `FeeEnvelope` protocol candidate | `052-11` tasks 1-2 | Keep future terminal-right and fee-support contracts distinct before live export. |

## Ordered Internal Gates

This table mirrors the ordered gate list from `052-TODO.md` so execution can
prove the phase in the same order the backlog requires.

| TODO gate | Meaning | Plan coverage |
| --- | --- | --- |
| Gate A | Backend selection and forest skeleton exist while compatibility stays default. | `052-01` task 1 |
| Gate B | Fixed bucket policy and bucket-root metadata are deterministic and storage-owned. | `052-01` task 2 |
| Gate C | Physical forest commits work for insert, delete, and no-op workloads. | `052-02` tasks 1-2 |
| Gate D | Journal recovery proves child-before-parent publication. | `052-03` task 1 |
| Gate E | Forest proof verification and reject classes are fail-closed. | `052-04` task 1 |
| Gate F | Deletion and non-existence proofs are validly implemented or explicitly unsupported. | `052-04` task 2 |
| Gate G | Reload, checkpoint, and path-index rebuild pass for forest mode. | `052-03` task 2; `052-05` task 2 |
| Gate H | Dual-backend equivalence passes the Phase 051 plus Phase 052 corpus. | `052-05` task 1 |
| Gate I | Simulator `scenario_1` passes in compatibility, forest, and dual-verify modes once those modes exist. | `052-05` task 2; `052-06` task 2 |
| Gate J | Benchmark and closeout evidence are recorded. | `052-06` tasks 1-2 |
| Gate K | Plans `052-01` through `052-06` are green and no placeholder forest behavior remains. | `052-07` task 1 |
| Gate L | Adaptive bucket split, merge, and migration proof work is future-ready but not live. | `052-08` tasks 1-2 |
| Gate M | Bucket occupancy metadata remains private or diagnostic unless future privacy and design gates pass. | `052-09` tasks 1-2 |
| Gate N | `SettlementStateRoot` migration is separate from the Phase 052 `AssetStateRoot` oracle. | `052-10` tasks 1-2 |
| Gate O | `RightLeaf` and `FeeEnvelope` protocol candidates are separate and future-only. | `052-11` tasks 1-2 |

## Source Coverage Matrix

Every major source obligation is routed into executable plans:

| Source obligation | Context coverage | Plan coverage |
| --- | --- | --- |
| One real forest backend behind the Phase 051 facade | `Phase Boundary`, `Backend Boundary And Authority` | `052-01`, `052-02` |
| Fixed bucket derivation and committed policy metadata | `Fixed Bucket Policy` | `052-01`, `052-04` |
| Physical child commits without public bucket authority | `Backend Boundary And Authority`, `Forest Runtime And Recovery` | `052-02`, `052-03` |
| Child-before-parent journaled publication and crash recovery | `Forest Runtime And Recovery` | `052-03` |
| Storage-owned proof envelope and fail-closed proof families | `Proof Contract And Absence Semantics` | `052-04` |
| Dual-backend equivalence using the Phase 051 corpus | `Backend Boundary And Authority`, `Current Baseline` | `052-05` |
| Checkpoint, wallet, validator, and simulator guardrails remain storage-owned | `Simulator And Downstream Consumers` | `052-05` |
| Compatibility default plus explicit forest rollout switch | `Rollout And Benchmark Obligations` | `052-06` |
| Async benchmark evidence and proof-size reporting | `Rollout And Benchmark Obligations`, `Proof Contract And Absence Semantics` | `052-04`, `052-06` |
| Full verification order plus scenario_1 coverage | `Simulator And Downstream Consumers` | `052-05`, `052-06` |
| Green-state audit before future promotion | `Change-Control Rules`, `Success Criteria` | `052-07` |
| Adaptive buckets and migration proofs as future work | `Fixed Bucket Policy`, `Non-Goals` | `052-08` |
| Proof-visible occupancy metadata as privacy-sensitive future work | `Fixed Bucket Policy`, `Non-Goals` | `052-09` |
| Generalized root migration separate from backend swap | `Public Semantic Contract`, `Non-Goals` | `052-10` |
| `RightLeaf` and `FeeEnvelope` separate future protocol contracts | `Public Semantic Contract`, `Non-Goals` | `052-11` |

## Design Corpus Coverage

This matrix answers the strict review question "do the plans implement the
design corpus?" in the only honest way available before code lands:
implementation-relevant HJMT backend sections must map to concrete plan work,
while future-only generalized-rights or governance-expansion sections must map
to explicit phase boundaries instead of being silently dropped or falsely
claimed as shipped in Phase 052.

| Design section | Phase 052 treatment | Plan or boundary |
| --- | --- | --- |
| `1.1 Design Thesis` | Implement the bucketed root-chained HJMT forest behind the facade. | `052-01` through `052-05` |
| `1.2 Live Asset Terms Versus Generalized Rights Terms` | Keep `AssetLeaf` and `AssetStateRoot` live; keep generalized-rights nouns future-only while planning a separate root migration. | `Phase Boundary`; `052-05`; `052-10`; `Non-Goals` |
| `1.3 What This Design Does Not Claim` | Do not claim the full forest already exists; implement it for real in this phase. | `Phase Boundary`; `052-02`; `052-03` |
| `2.1 Main Protocol Requirements` | Keep leaf-oriented, checkpoint-bound, path-local settlement evidence. | `052-04`; `052-05`; `052-06` |
| `2.2 Cross-Chain And External-Right Requirements` | Preserve definition-scoped semantics and non-global serial authority. | `052-02`; `052-05` |
| `2.3 Machine And Agent Economy Requirements` | Cover hot-serial and burst workloads with fixed buckets and benchmarks. | `052-02`; `052-06` |
| `2.4 RightLeaf As The Generalized Terminal Object` | Preserve as future-only design vocabulary and capture a separate protocol candidate. | `Non-Goals`; `052-11`; `Codebase Fit And Non-Duplication Rules` |
| `2.4.1 Legal-Defensibility Value Of RightLeaf` | Preserve as future governance or legal-architecture work with explicit protocol test duties, not backend scope for this phase. | `Non-Goals`; `052-11`; `Phase Boundary` |
| `2.5 Linked Liability Requirements` | Keep proofs path-local and typed; no global authority widening. | `052-04`; `052-05` |
| `2.6 OnionNet And Publication Requirements` | Keep transport or publication concerns outside storage semantic authority. | `052-05`; `Phase Boundary` |
| `2.7 Roadmap And Maturity Requirements` | Use staged rollout with compatibility as oracle and default. | `052-01`; `052-05`; `052-06` |
| `2.8 Uniqueness And Use-Case Requirements` | Validate selective evidence and workload shape through proofs, benches, and simulator flows. | `052-04`; `052-06` |
| `3.1 Semantic Strengths Already Present` | Reuse existing semantic vocabulary and tree identities instead of replacing them. | `Current Baseline`; `052-01`; `052-02` |
| `3.2 Physical Bottleneck To Remove` | Replace the shared physical commit wall with real forest commits. | `052-02`; `052-03` |
| `3.3 Compatibility Backend Requirement` | Keep compatibility as oracle, rollback reference, and dual-verify partner. | `052-01`; `052-05`; `052-06` |
| `4.1 Topology Overview` | Implement definition, serial, bucket, and terminal-tree layering privately behind storage. | `052-02`; `052-03`; `052-04` |
| `4.2 Public Contract` | Keep public callers on `AssetPath` and semantic roots only. | `052-01`; `052-05` |
| `4.3 Fixed Bucket Derivation` | Freeze deterministic committed bucket policy and verifier-visible metadata. | `052-01`; `052-04` |
| `4.4 Why Fixed Buckets Instead Of Adaptive Buckets` | Exclude adaptive split or merge from live Phase 052 and capture a future migration-proof candidate. | `Fixed Bucket Policy`; `052-08`; `Non-Goals` |
| `5.1 Asset Path Proof Shape` | Implement chained inclusion proof semantics through one storage-owned decoder. | `052-04` |
| `5.1.1 Current Compatibility Envelope vs Target HJMT Envelope` | Measure encoded proof delta and keep byte counts non-normative. | `052-04`; `052-06` |
| `5.1.1 Transaction-Scale Proof Size Example` | Record shared-parent multi-leaf proof-size evidence from real encodings. | `052-04`; `052-06` |
| `5.2 Root Taxonomy` | Keep semantic roots and backend roots distinct; prevent substitution. | `052-04`; `052-05` |
| `5.3 Inclusion, Deletion, And Non-Existence` | Implement or explicitly reject fail-closed proof families. | `052-04` |
| `5.4 Path Index Boundary` | Keep path index rebuildable and internal. | `052-03`; `052-05` |
| `6.1 Insert Flow` | Real deterministic child updates before parent publication. | `052-02`; `052-03` |
| `6.2 Delete Flow` | Deterministic deletes, parent pruning, and safe publication. | `052-02`; `052-03`; `052-04` |
| `6.3 Expected Performance By Workload` | Benchmark broad, hot-definition, hot-serial, delete-heavy, proof-heavy, and recovery workloads. | `052-06` |
| `6.4 Score Target` | Record evidence honestly; do not overclaim performance. | `052-06`; `Success Criteria` |
| `7.1 Forest Commit Journal` | Persist durable journal lifecycle and recovery. | `052-03` |
| `7.2 Backend Interface Boundary` | Keep forest behind `AssetTreeBackend` and storage-owned mode routing. | `052-01`; `Codebase Fit And Non-Duplication Rules` |
| `7.3 Main Implementation Risks` | Route mismatch, drift, and partial-publication hazards into fail-closed tests. | `052-03`; `052-04`; `052-05` |
| `8.1 Public Visibility` | Do not turn buckets, occupancy counters, or physical layout into public business meaning. | `052-01`; `052-05`; `052-09`; `Non-Goals` |
| `8.2 Selective Disclosure` | Keep proofs path-local and storage-owned. | `052-04`; `052-05` |
| `8.3 Checkpoint Evidence` | Bind checkpoint evidence to semantic roots, not forest internals. | `052-05` |
| `9.1 Rollout Phases` | Use the eleven-plan packet as staged backend implementation plus first-class follow-up capture. | `Execution Plan Map`; `052-01` through `052-11` |
| `9.2 Benchmark Plan` | Extend the landed benchmark harness before making performance claims. | `052-06` |
| `9.3 Acceptance Criteria` | Mirror the design acceptance gate in phase success criteria and closeout rules. | `Success Criteria`; `052-06` |
| `10 Design Rationale` | Preserve rejected alternatives as explicit exclusions rather than reintroducing them. | `Codebase Fit And Non-Duplication Rules`; `Non-Goals` |
| `11 Relationship To Z00Z Use Cases` | Validate private cash, external asset, audit, machine, liability, and publication consumers through semantic-root flows. | `052-04`; `052-05`; `052-06` |
| `12 Normative Requirement Summary` | Map JMT-REQ-001 through JMT-REQ-014 into phase requirements and plan coverage. | `Requirements`; `JMT Requirement Coverage` |
| `13.1 Equivalence Tests` | Run compatibility, forest, and dual-verify semantic corpus. | `052-05` |
| `13.2 Crash Tests` | Run journal interruption, recovery, and reload validation matrix. | `052-03` |
| `13.3 Proof Tests` | Run inclusion, deletion, absence, and reject matrix tests. | `052-04`; `052-05` |
| `13.4 Performance Tests` | Run realistic async benchmark and proof-size matrix. | `052-06` |
| `14 Open Questions` | Preserve dynamic buckets, public path index, occupancy metadata, generalized rights widening, and other unresolved expansions as future work with named candidates. | `052-08`; `052-09`; `052-10`; `052-11`; `Non-Goals`; `Change-Control Rules` |
| `15 Conclusion` | Implement the balanced HJMT backend without widening public authority. | `Success Criteria` |
| `Appendix A` and `Appendix B` | Treat glossary and compact summary as vocabulary and boundary guardrails. | `Public Semantic Contract`; `Backend Boundary And Authority` |

## JMT Requirement Coverage

| JMT requirement | Phase 052 treatment |
| --- | --- |
| `JMT-REQ-001` | Keep `AssetStateRoot` as the live semantic root and keep future generalized root terms out of live exports while planning the separate migration. Covered by `052-01`, `052-05`, `052-06`, `052-10`. |
| `JMT-REQ-002` | Keep the three-part `AssetPath` as the caller-facing path contract. Covered by `052-01`, `052-02`, `052-03`. |
| `JMT-REQ-003` | Derive fixed buckets from path identity and committed policy. Covered by `052-01`, `052-04`. |
| `JMT-REQ-004` | Keep bucket ids internal except where the storage-owned verifier recomputes them. Covered by `052-01`, `052-05`. |
| `JMT-REQ-005` | Support batch insert or delete across independent physical bucket JMTs. Covered by `052-02`, `052-06`. |
| `JMT-REQ-006` | Commit parent roots only after child roots are durable. Covered by `052-03`. |
| `JMT-REQ-007` | Maintain a forest commit journal for crash-safe recovery. Covered by `052-03`. |
| `JMT-REQ-008` | Provide inclusion, deletion, and non-existence proof envelopes where supported and fail-closed otherwise. Covered by `052-04`. |
| `JMT-REQ-009` | Keep the path index rebuildable and internal. Covered by `052-03`. |
| `JMT-REQ-010` | Keep compatibility as the semantic oracle for equivalence and migration testing. Covered by `052-05`. |
| `JMT-REQ-011` | Fail closed on policy mismatch, child-root mismatch, journal inconsistency, or proof-envelope mismatch. Covered by `052-03`, `052-04`, `052-05`. |
| `JMT-REQ-012` | Do not expose backend roots as substitutes for `AssetStateRoot`. Covered by `052-01`, `052-05`. |
| `JMT-REQ-013` | Treat `RightLeaf` as future-only rather than live current runtime truth, and capture the future protocol candidate. Covered by `052-05`, `052-11`, `Non-Goals`. |
| `JMT-REQ-014` | Keep future `RightLeaf` and `FeeEnvelope` distinct contract families with separate test duties. Covered by `052-05`, `052-11`, `Non-Goals`. |

## Test And Benchmark Coverage

| Source test duty | Phase 052 treatment |
| --- | --- |
| Golden semantic corpus | `052-05` converts the Phase 051 harness from one executable backend case to compatibility plus real forest and dual-verify modes. |
| Crash and recovery matrix | `052-03` covers every journal interruption point and reload validation. |
| Proof reject matrix | `052-04` covers malformed envelope, wrong semantic root, wrong path, wrong definition leaf, wrong serial leaf, wrong bucket metadata, wrong bucket root, wrong terminal leaf, wrong terminal leaf hash, wrong branch proof, wrong root binding, wrong backend-root diagnostic bind, wrong checkpoint context, wrong deletion proof, wrong non-existence proof, default commitment, replay, present-key absence, and unsupported-version rejects with state preserved after rejecting workloads. |
| Checkpoint or reload or path-index rebuild | `052-03` and `052-05` extend `test_redb_rehydrate.rs`, `test_checkpoint_root_binding.rs`, and checkpoint-facing storage tests. |
| Async benchmark matrix | `052-06` extends the existing bench harness for broad, hot-definition, hot-serial, delete-heavy, proof-heavy, and recovery workloads, including multithread async `multi-insert` and `multi-delete`. |
| Proof-size reporting | `052-04` records encoded inclusion-proof evidence; `052-06` records shared-parent transaction proof-size and absent-key proof-size evidence. |
| Simulator `scenario_1` | `052-05` preserves storage-owned source shapes across Stage 4, Stage 6, Stage 7 transfer receive, Stage 11 wallet scan, Stage 12 checkpoint finalization, Stage 13 replay, and `runner_verify`; `052-06` runs compatibility, forest, and dual-verify scenario validation once those modes exist. |
| Green-state and no-drift audit | `052-07` proves `052-01` through `052-06` are complete by evidence and that deferred candidates are not implied shipped. |
| Adaptive bucket future tests | `052-08` defines future split, merge, migration, epoch, stale-proof, replay, recovery, benchmark, and simulator duties. |
| Occupancy metadata future tests | `052-09` defines future privacy, design-update, proof tamper, policy-generation, reload drift, and downstream non-authority duties. |
| Generalized-root future tests | `052-10` defines future old/new root generation, checkpoint, proof, downgrade, rollback, wallet, validator, and simulator duties. |
| `RightLeaf` and `FeeEnvelope` future tests | `052-11` defines future terminal-right schema, fee-support separation, tamper, replay, expiry, sponsor, binding, and state-preservation duties. |

## Scenario 1 Surface Coverage

| Scenario surface from `052-TODO.md` | Planned in | Guardrail |
| --- | --- | --- |
| Stage 4 pre-transaction storage view and `chk_blob_item` preparation | `052-05`, `052-06` | Storage-owned semantic root and proof APIs only |
| Stage 6 checkpoint bundle and storage handoff | `052-05`, `052-06` | `CheckRoot` and checkpoint inputs remain semantic-root based |
| Stage 7 transfer receive or post-apply storage transition | `052-05`, `052-06` | No physical layout, bucket, or backend-root authority appears in simulator logic |
| Stage 11 wallet scan using `proof_blob` and `chk_blob` | `052-04`, `052-05`, `052-06` | One storage-owned proof decoder boundary |
| Stage 12 checkpoint finalization artifacts | `052-05`, `052-06` | Finalized checkpoint artifacts stay bound to semantic roots |
| Stage 13 storage replay and tamper checks | `052-03`, `052-05`, `052-06` | Replay validation stays semantic-root based |
| `runner_verify` storage contract validation | `052-05`, `052-06` | Detects drift without learning forest layout authority |

## Execution Plan Map

| Plan | Objective | Requirements |
| --- | --- | --- |
| `052-01-PLAN.md` | Add backend selection, forest skeleton, and fixed bucket policy types behind the facade. | `PH52-BACKEND-MODE`, `PH52-BUCKET-POLICY` |
| `052-02-PLAN.md` | Build the private forest tree store and deterministic batch planner for real insert or delete work. | `PH52-FOREST-LAYOUT`, `PH52-BATCH-PLANNER` |
| `052-03-PLAN.md` | Add forest commit journal, crash recovery, reload validation, and path-index rebuild. | `PH52-JOURNAL-RECOVERY`, `PH52-RELOAD-INDEX` |
| `052-04-PLAN.md` | Implement the forest proof envelope plus deletion or non-existence proof families and reject matrix. | `PH52-PROOF-ENVELOPE`, `PH52-ABSENCE-PROOFS` |
| `052-05-PLAN.md` | Extend the dual-backend equivalence corpus and close checkpoint or downstream authority guardrails. | `PH52-EQUIVALENCE`, `PH52-CHECKPOINT-GUARDRAILS` |
| `052-06-PLAN.md` | Gate rollout, extend benchmarks, run final validation, and record closeout evidence. | `PH52-ROLLOUT-BENCHMARKS`, `PH52-CLOSEOUT` |
| `052-07-PLAN.md` | Audit implemented backend plans green and record deferred follow-up ledger. | `PH52-GREEN-AUDIT` |
| `052-08-PLAN.md` | Define the future adaptive bucket split, merge, and migration proof candidate. | `PH52-ADAPTIVE-BUCKETS-FOLLOWUP` |
| `052-09-PLAN.md` | Define the future proof-visible bucket occupancy privacy candidate. | `PH52-OCCUPANCY-METADATA-FOLLOWUP` |
| `052-10-PLAN.md` | Define the future generalized settlement-root migration candidate. | `PH52-GENERALIZED-ROOT-FOLLOWUP` |
| `052-11-PLAN.md` | Define the future `RightLeaf` and `FeeEnvelope` protocol candidate. | `PH52-RIGHTLEAF-FEEENVELOPE-FOLLOWUP` |

## Mandatory Verification Policy

Every `task type="auto"` verify section in this phase must include the user
requested execution review loop:

1. For Rust or test-affecting changes, run
   `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` first as
   a fail-fast gate. If it fails, stop, fix, and rerun it before broader
   validation.
2. Then run `cargo test --release --features test-fast --features
   wallet_debug_dump` whenever relevant to the task.
3. Run `.github/prompts/gsd-review-tasks-execution.prompt.md` through
   `/GSD-Review-Tasks-Execution` at least 3 times in YOLO mode, fix all issues
   and warnings, and stop only after at least 2 consecutive runs report no
   significant code issues.
4. If committing is needed, use `/z00z-git-versioning` and the repository-
   owned `.github/skills/z00z-git-versioning/scripts/version-manager.sh`
   flow.

## Change-Control Rules

- If implementation discovers a new protocol, proof, checkpoint, or semantic
  constraint that changes the HJMT design contract, update
  `docs/Z00Z-JMT-Design.md` first, then `052-TODO.md`, then the affected
  context or plan files, and only then the affected tests.
- `052-TODO.md` remains the phase backlog authority for scope and coverage.
  The context and plan packet must be updated to stay in lockstep with it.
- Repository artifacts, code, comments, documentation, commit messages, logs,
  and technical content remain English.

## Non-Goals

- Do not create a fake forest backend or a copied compatibility branch
  pretending to be the real forest implementation.
- Do not expose `TreeId`, namespace bytes, bucket ids, branch ordering, or raw
  backend roots as public authority inputs.
- Do not promote `SettlementStateRoot`, `RightLeaf`, or `FeeEnvelope` into
  live storage exports; plans `052-10` and `052-11` only capture future
  protocol candidates.
- Do not add adaptive bucket split or merge, migration proofs, or public bucket
  occupancy counters in live Phase 052 runtime; plans `052-08` and `052-09`
  only capture future candidates and privacy gates.
- Do not add a second proof decoder, checkpoint verifier, or storage authority
  lane outside `z00z_storage`.
- Do not claim public performance wins without recorded benchmark evidence from
  the landed bench harness.

## Success Criteria

1. The real HJMT forest backend exists behind the Phase 051 facade and is the
   only new physical backend path.
2. Compatibility and forest modes produce identical semantic outcomes for the
   required corpus, and dual-verify turns any mismatch into a hard failure.
3. Fixed bucket policy and bucket metadata are deterministic, versioned,
   verifier-visible where needed, and hidden from public write authority.
4. Forest commit journal and reload validation never expose a parent root whose
   child roots are missing, stale, or digest-mismatched.
5. Forest inclusion proofs validate through one storage-owned verifier and
   deletion or non-existence proofs either verify fail-closed or remain
   explicitly unsupported.
6. Path-index rebuild, checkpoint seal or reload, and reject matrices are green
   for compatibility and forest modes.
7. Downstream crates, including simulator `scenario_1`, continue to consume
   storage semantics and do not become a second layout authority surface.
8. Compatibility remains the default backend until rollout, guardrail, and
   benchmark obligations are satisfied.
9. Async benchmark evidence exists for multithread `multi-insert` and
   `multi-delete`, along with measured inclusion proof-size evidence and
   explicit unsupported fail-closed status for non-existence proof-size
   reporting.
10. The mandatory verification order and repeated
    `/GSD-Review-Tasks-Execution` loop are recorded in closeout evidence.
11. Plans `052-07` through `052-11` record green-state audit and first-class
    future candidates without claiming adaptive buckets, occupancy counters,
    `SettlementStateRoot`, `RightLeaf`, or `FeeEnvelope` shipped in Phase 052.
