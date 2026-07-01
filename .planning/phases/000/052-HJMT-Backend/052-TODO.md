---
phase: 052-HJMT-Backend
status: complete
source: 051-06-readiness-handoff
updated: 2026-05-29
owner: Z00Z Storage
---

<!-- markdownlint-disable MD060 -->

# Phase 052 TODO: Full HJMT Backend Behind The Facade

## 📌 Phase Implementation Brief

**Goal:**

- Implement the real bucketed root-chained HJMT forest backend behind the
  existing Phase 051 storage facade.
- Keep `CompatibilityBackend` as the semantic oracle while proving that the
  forest backend produces identical live `AssetStateRoot`, checkpoint, reload,
  path-index, and proof outcomes for the required corpus.
- Make the backend rollout configuration-gated, with compatibility as the
  default until forest and dual-verify validation are green.
- Ensure storage tests and simulator `scenario_1` pass through the same
  storage-owned facade without learning physical forest layout.

**Source:**

- primary design source: [Z00Z-JMT-Design](../../../docs/Z00Z-JMT-Design.md);
- migration seam source:
  [Phase 051 TODO](../000/051-HJMT-Facade/051-TODO.md);
- executable semantic oracle:
  `crates/z00z_storage/tests/test_phase051_golden_corpus.rs`;
- downstream guardrail oracle:
  `crates/z00z_storage/tests/test_phase051_guardrails.rs`;
- simulator authority consumers under
  `crates/z00z_simulator/src/scenario_1/`.

**Included work:**

- one real physical forest backend behind `AssetTreeBackend`;
- explicit backend mode selection for compatibility, forest, and dual-verify
  execution;
- fixed bucket derivation and committed verifier-visible bucket policy;
- private forest tree layout for definition, serial, bucket, and terminal
  asset JMTs;
- child-before-parent forest commits;
- durable forest commit journal and crash-safe reload validation;
- storage-owned forest proof envelope, verifier checks, and reject classes;
- deletion and non-existence proof families only where the live verifier can
  validate them fail-closed;
- benchmark evidence for multithread async `multi-insert` and `multi-delete`
  execution plus inclusion proof-size reporting and explicit non-existence
  unsupported fail-closed status;
- rebuildable path-index validation;
- dual-backend equivalence mode against the Phase 051 corpus;
- checkpoint seal and reload coverage for compatibility and forest backends;
- simulator `scenario_1` storage path coverage through the facade.
- a green-state audit proving plans `052-01` through `052-06` are complete by
  executed evidence, not planning intent;
- first-class deferred follow-up candidates for adaptive bucket migration,
  bucket occupancy metadata privacy, generalized settlement-root migration,
  and `RightLeaf` plus `FeeEnvelope` protocol work.

**Excluded work:**

- fake or placeholder forest backend;
- duplicated public storage facade or proof authority;
- public bucket id, `TreeId`, namespace prefix, branch ordering, physical key
  layout, or backend-root authority;
- adaptive bucket split, merge, or migration proofs;
- promotion of `SettlementStateRoot`, `RightLeaf`, or `FeeEnvelope` into live
  storage exports;
- generalized rights runtime work;
- adaptive bucket runtime behavior, split proofs, merge proofs, migration
  proofs, or bucket epoch replay in live Phase 052 code;
- proof-visible bucket occupancy counters without a repository-backed design
  update and privacy review;
- public performance claims without recorded benchmark evidence;
- simulator-owned checkpoint, proof, or storage-root authority.

**Implementation-relevant requirements:**

- `JMT-REQ-001`: keep `AssetStateRoot` as the live public asset root; covered
  by `052-01`, `052-10`, and `052-12`.
- `JMT-REQ-002`: preserve the three-part `AssetPath`; covered by `052-02`,
  `052-04`, `052-08`, and simulator Stage 4 and Stage 11 checks.
- `JMT-REQ-003`: derive fixed buckets from path identity and committed policy;
  covered by `052-02` and `052-06`.
- `JMT-REQ-004`: keep bucket ids internal except proof recomputation; covered
  by `052-02`, `052-03`, and `052-10`.
- `JMT-REQ-005`: support batch insert and delete across physical bucket JMTs;
  covered by `052-03`, `052-04`, and `052-11`.
- `JMT-REQ-006`: commit parent roots only after child roots are durable;
  covered by `052-05`.
- `JMT-REQ-007`: maintain a forest commit journal; covered by `052-05` and
  `052-08`.
- `JMT-REQ-008`: provide inclusion, deletion, and non-existence proof
  envelopes where verifiable; covered by `052-06` and `052-07`.
- `JMT-REQ-009`: keep path index rebuildable and internal; covered by
  `052-08`.
- `JMT-REQ-010`: keep compatibility backend for equivalence; covered by
  `052-09`.
- `JMT-REQ-011`: fail closed on policy, root, journal, or proof mismatch;
  covered by `052-05`, `052-06`, `052-07`, and `052-09`.
- `JMT-REQ-012`: never expose backend roots as `AssetStateRoot`; covered by
  `052-10` and `052-16`.
- `JMT-REQ-013`: treat `RightLeaf` as future-only; covered by `052-10` and
  `052-17`.
- `JMT-REQ-014`: keep future `RightLeaf` and `FeeEnvelope` separate; covered by
  `052-17`.

**Implementation boundary:**

- All physical forest mechanics belong inside `z00z_storage`.
- `AssetStore` and `AssetTreeBackend` remain the semantic entrypoint for
  wallet, validator, runtime, checkpoint, and simulator consumers.
- `CompatibilityBackend` remains the reference implementation for equivalence
  checks and migration rollback reasoning.
- `ProofBlob`, `chk_blob`, and any forest proof successor stay storage-owned.
- Checkpoint evidence binds prior and next `AssetStateRoot`; forest physical
  roots remain private or diagnostic.
- Simulator `scenario_1` may exercise storage behavior only through the live
  facade and storage-owned proof checks.

**Ordered internal gates:**

1. Gate A: backend selection and forest skeleton are present, with
   compatibility still default.
2. Gate B: fixed bucket policy and bucket root types are deterministic and
   private to storage authority.
3. Gate C: physical forest tree commits work for insert, delete, and no-op
   workloads.
4. Gate D: forest journal recovery proves child-before-parent publication.
5. Gate E: forest proof verification and reject classes are fail-closed.
6. Gate F: deletion and non-existence proofs are either validly implemented or
   explicitly unsupported.
7. Gate G: reload, checkpoint, and path-index rebuild pass for forest mode.
8. Gate H: dual-backend equivalence passes the full Phase 051 plus Phase 052
   corpus.
9. Gate I: simulator `scenario_1` passes in compatibility, forest, and
   dual-verify backend modes once those modes exist.
10. Gate J: benchmark and closeout evidence are recorded.
11. Gate K: plans `052-01` through `052-06` are audited green and contain no
    placeholder forest behavior, copied compatibility lane, or downstream
    physical-layout authority.
12. Gate L: adaptive bucket and migration proof work is captured as a future
    phase candidate with fixed-bucket evidence as a hard entry condition.
13. Gate M: bucket occupancy metadata remains private or diagnostic unless a
    future design update and privacy review explicitly make it proof-visible.
14. Gate N: `SettlementStateRoot` migration is captured as a separate future
    protocol phase and does not replace the Phase 052 `AssetStateRoot` oracle.
15. Gate O: `RightLeaf` and `FeeEnvelope` remain separate future protocol
    contracts with explicit test duties before live export.

**Implementation tasks:**

Numbering note: numbered TODO task ids and numbered plan-file ids are separate
ledgers after backend plan `052-06`. TODO tasks `052-01` through `052-12`
define the backend implementation work and map into plan files `052-01` through
`052-06`. TODO tasks `052-13` through `052-17` map into plan files `052-07`
through `052-11` for green-state audit and first-class follow-up candidates.

- `052-01`: backend selection and forest skeleton.
- `052-02`: fixed bucket policy and root leaf types.
- `052-03`: forest tree store and physical key layout.
- `052-04`: forest batch planner for inserts and deletes.
- `052-05`: forest commit journal and recovery state.
- `052-06`: forest proof envelope and verifier checks.
- `052-07`: deletion and non-existence proof semantics.
- `052-08`: reload validation and path-index rebuild.
- `052-09`: dual-backend equivalence corpus.
- `052-10`: checkpoint and downstream guardrail closure.
- `052-11`: rollout configuration and benchmark evidence.
- `052-12`: verification closeout.
- `052-13`: green-state audit for implemented plans `052-01` through
  `052-06`.
- `052-14`: adaptive bucket split, merge, and migration proof phase
  candidate.
- `052-15`: proof-visible bucket occupancy metadata privacy phase candidate.
- `052-16`: generalized settlement-root migration phase candidate.
- `052-17`: `RightLeaf` and `FeeEnvelope` protocol phase candidate.

**Tests and simulation:**

- Unit tests must cover deterministic bucket derivation, invalid bucket policy,
  bucket-root encoding, private tree ids, planner reject paths, journal status
  transitions, proof reject classes, deletion proof behavior, non-existence
  proof behavior, reload validation, and backend-mode parsing.
- Integration tests must extend the Phase 051 corpus for compatibility,
  forest, and dual-verify modes.
- Benchmark harnesses must cover multithread async `multi-insert` and
  `multi-delete` workloads, inclusion prove and verify time, serialized proof
  size, shared-parent transaction proof size, and explicit unsupported
  fail-closed status for non-existence proof-size reporting until absent-key
  proofs are live.
- Redb reload tests must cover forest persistence, journal recovery, path-index
  rebuild, checkpoint seal, checkpoint reload, and claim replay rows.
- Guardrail tests must keep wallet, validator, runtime, and simulator code away
  from physical layout authority.
- Simulator `scenario_1` must pass through storage facade checks for:
  - Stage 4 pre-transaction storage view and `chk_blob_item` preparation;
  - Stage 6 checkpoint bundle and storage handoff;
  - Stage 11 post-transaction storage apply and `proof_blob` plus `chk_blob`
    wallet scan;
  - Stage 13 storage replay, root binding report, and tamper checks;
  - `runner_verify` storage contract validation.
- Full scenario validation must include:
  - `cargo test -p z00z_simulator --release --features wallet_debug_dump
    scenario_1`;
  - `cargo run --release -p z00z_simulator --bin scenario_1 --features
    wallet_debug_dump`;
  - the same scenario path with the Phase 052 compatibility, forest, and
    dual-verify backend modes once `052-01` defines the exact config surface.

Canonical design source:

- [Z00Z-JMT-Design](../../../docs/Z00Z-JMT-Design.md)
- [Phase 051 TODO](../000/051-HJMT-Facade/051-TODO.md)
- `crates/z00z_storage/tests/test_phase051_golden_corpus.rs`
- `crates/z00z_storage/tests/test_phase051_guardrails.rs`

Execution rules:

- execute this file in order unless a dependency note says otherwise;
- treat `docs/Z00Z-JMT-Design.md` as normative for HJMT requirement meaning
  and this file as normative for Phase 052 execution order;
- treat Phase 051 as the migration seam: do not reopen, duplicate, or replace
  `AssetTreeBackend`, `CompatibilityBackend`, `AssetStateRoot`, `CheckRoot`,
  `ProofBlob`, `chk_blob`, or the storage-owned checkpoint contracts;
- keep `CompatibilityBackend` as the migration oracle and semantic reference
  only, not as a second long-lived public authority lane;
- keep the forest backend hidden behind `AssetTreeBackend` and the existing
  `AssetStore` facade; downstream crates must not learn physical tree layout;
- keep `AssetPath`, `AssetLeaf`, and `AssetStateRoot` as the live asset-centric
  vocabulary; do not export `SettlementStateRoot`, `RightLeaf`, or
  `FeeEnvelope` in Phase 052;
- keep bucket ids internal to storage APIs except where the storage-owned proof
  verifier needs committed metadata to recompute the derived bucket;
- do not accept placeholder deletion or non-existence proofs; if a proof family
  cannot validate fail-closed through the live wire format, it must remain
  explicitly unsupported;
- when execution discovers a new protocol or proof constraint, update
  `docs/Z00Z-JMT-Design.md` first, then this backlog, then the affected tests;
- before starting any numbered task, complete its `MANDATORY pre-read` block;
- repository artifacts, code, comments, documentation, commit messages, logs,
  and technical content stay in English.

Mandatory verification order:

1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
   first as the fail-fast gate for every Rust or test-affecting task.
2. If bootstrap fails, stop, fix, and rerun bootstrap before broader tests.
3. Run relevant focused release tests for the touched storage, checkpoint,
   wallet, validator, or simulator surface.
4. Run `cargo test --release --features test-fast --features wallet_debug_dump`
   when broad validation is relevant.
5. Run `.github/prompts/gsd-review-tasks-execution.prompt.md` in YOLO mode at
   least three times and stop only after at least two consecutive runs report
   no significant issues.

## 🚩 Deferred But First-Class Follow-Ups

These items are not live Phase 052 runtime scope, but they are first-class
future work. They must remain visible in the phase packet so the project can
finish the fixed-bucket forest safely, then promote the next protocol slice
with clean diff, clear rollback, and independent tests.

Future phase candidates:

- `HJMT-Adaptive-Buckets-And-Migration-Proofs`
  - depends on fixed-bucket forest benchmarks, proof-size evidence, recovery
    evidence, privacy review, and split/merge/migration proof design;
- `HJMT-Bucket-Occupancy-Metadata-Privacy`
  - depends on repository-backed design update before any proof-visible
    `leaf_count`, `bucket_occupancy`, or equivalent counter ships;
- `Generalized-Rights-Root-Model`
  - depends on a separate protocol migration for `AssetStateRoot` to
    `SettlementStateRoot`, with generation metadata, checkpoint migration,
    proof migration, and rollback rules;
- `RightLeaf-FeeEnvelope-Protocol`
  - depends on generalized-rights runtime semantics and keeps terminal right
    semantics separate from fee or processing support.

Phase 052 must preserve extension points and guardrails for these candidates,
but must not export them as live storage contracts.

## 🎯 Decision Summary

The execution baseline for Phase 052 is:

### Decision 1: One Physical Forest Backend Behind One Facade

Choice:

- add one real bucketed root-chained forest backend behind the existing
  `AssetTreeBackend` facade.

Chosen direction:

- do not add a fake forest backend, a second public store API, a second proof
  decoder outside storage, or a downstream layout authority lane.

### Decision 2: Keep Compatibility As Oracle, Not Authority Fork

Choice:

- use `CompatibilityBackend` and the Phase 051 golden corpus as the semantic
  oracle for equality, reject behavior, reload, checkpoint, and proof shape.

Chosen direction:

- forest output must match compatibility semantic outcomes before any rollout
  flag can make forest the selected backend.

### Decision 3: Add Fixed Buckets Before Parallel Commits

Choice:

- implement deterministic fixed bucket derivation and verifier-visible policy
  metadata before implementing physical bucket commits.

Chosen direction:

- do not implement adaptive split or merge, dynamic bucket migration, or
  proof-visible occupancy counts in Phase 052.

### Decision 4: Publish Parents Only After Durable Children

Choice:

- add a forest commit journal with `Prepared`, `ChildrenCommitted`,
  `ParentsCommitted`, and `RootPublished` states.

Chosen direction:

- no parent root or exported `AssetStateRoot` may be visible unless child roots
  and recorded digests are durable and replay-valid.

### Decision 5: Extend Proofs Through Storage-Owned Versions Only

Choice:

- keep the storage proof envelope owner in `z00z_storage`, add the target
  forest proof version there, and reject unsupported families fail-closed.

Chosen direction:

- no wallet, validator, simulator, or runtime crate may decode bucket proofs,
  branch order, physical roots, or raw JMT layout as authority.

### Decision 6: Roll Out Behind Configuration Gates

Choice:

- keep compatibility as the default backend until dual-backend equivalence is
  proven by the Phase 051 corpus plus Phase 052 extensions.

Chosen direction:

- allow forest and dual-verify modes only through an explicit storage-owned
  backend mode, with guardrails that reject unknown modes.

## 🔗 Dependency Chain

Execution dependency chain:

1. `052-01` backend selection and forest skeleton
2. `052-02` fixed bucket policy and root leaf types
3. `052-03` forest tree store and physical key layout
4. `052-04` forest batch planner for inserts and deletes
5. `052-05` forest commit journal and recovery state
6. `052-06` forest proof envelope and verifier checks
7. `052-07` deletion and non-existence proof semantics
8. `052-08` reload validation and path-index rebuild
9. `052-09` dual-backend equivalence corpus
10. `052-10` checkpoint and downstream guardrail closure
11. `052-11` rollout configuration and benchmark evidence
12. `052-12` verification closeout

Hard dependencies:

- `052-02` depends on `052-01`
- `052-03` depends on `052-02`
- `052-04` depends on `052-02` and `052-03`
- `052-05` depends on `052-03` and `052-04`
- `052-06` depends on `052-02`, `052-03`, and `052-05`
- `052-07` depends on `052-06`
- `052-08` depends on `052-04` and `052-05`
- `052-09` depends on `052-04`, `052-06`, `052-07`, and `052-08`
- `052-10` depends on `052-05`, `052-08`, and `052-09`
- `052-11` depends on `052-09` and `052-10`
- `052-12` depends on `052-01` through `052-11`

## 🧱 Structures And Interfaces

The following structures and interfaces are implementation targets, not public
protocol authority names.

Storage facade:

- keep `AssetTreeBackend` as the only semantic backend trait;
- keep `AssetStore` as the caller-facing store type;
- add internal backend selection with `AssetBackendMode` values such as
  `Compatibility`, `Forest`, and `DualVerify`;
- keep `CompatibilityBackend::NAME` as `"compatibility"`;
- add one internal forest adapter with a short name such as `ForestBackend`;
- keep `backend_name()` diagnostic and never make it a root authority.

Fixed bucket policy:

- add `BucketId` as an internal fixed-width storage identity;
- add `BucketPolicy` with `policy_id`, `bucket_bits`, hash domain, canonical
  encoding version, bucket id width, minimum bucket count, target leaf count,
  and compatibility generation;
- add `BucketRootLeaf` with `definition_id`, `serial_id`, `bucket_id`,
  `asset_jmt_root`, and `bucket_policy_id`;
- derive buckets through a domain-separated hash equivalent to
  `z00z.storage.asset.bucket.v1`;
- commit only policy fields needed by verifiers; keep operational counters
  private unless the design source is updated.

Forest tree layout:

- add a private logical `TreeId::Bucket(DefinitionId, SerialId)` or equivalent
  internal tree identity;
- add a private bucket asset tree identity scoped by definition, serial, and
  bucket id;
- keep `TreeId`, namespace prefixes, branch ordering, and physical roots
  crate-private;
- preserve deterministic parent ordering through sorted definition, serial,
  bucket, and asset keys.

Forest journal:

- add `ForestCommitJournalEntry`;
- add `ForestCommitStatus` with `Prepared`, `ChildrenCommitted`,
  `ParentsCommitted`, and `RootPublished`;
- record previous semantic root, next semantic root, touched definitions,
  touched serials, touched buckets, child digests, parent digest, version, and
  status;
- persist journal rows through the storage backend before exposing parent roots.

Proof envelope:

- keep `ProofBlob` and `chk_blob` as storage-owned compatibility proof
  authority;
- add a versioned forest proof path inside storage, either by extending the
  storage-owned proof envelope version boundary or by adding a new storage-owned
  forest payload that is decoded only by storage;
- include bucket policy, `BucketRootLeaf`, bucket proof, terminal proof, and
  the live `AssetStateRoot`;
- reject compatibility proofs where forest proof fields are required, and
  reject forest proofs where bucket policy, bucket id, child roots, terminal
  leaf hash, or semantic root binding drift.

Deletion and absence:

- add explicit proof family handling for inclusion, deletion, and
  non-existence;
- prove non-existence by opening the derived terminal slot to the canonical
  default commitment, not by returning a node-local `not found`;
- prove deletion only when the transition binds prior root, deleted path, next
  root, and affected parent roots;
- keep unsupported families fail-closed until the verifier can validate them.

## 🗂️ File-First Implementation Order

Edit order by file cluster:

1. `crates/z00z_storage/src/assets/store.rs`
2. `crates/z00z_storage/src/assets/mod.rs`
3. `crates/z00z_storage/src/assets/types.rs`
4. `crates/z00z_storage/src/assets/types_identity.rs`
5. `crates/z00z_storage/src/assets/types_record.rs`
6. new `crates/z00z_storage/src/assets/store_internal/forest_config.rs`
7. new `crates/z00z_storage/src/assets/store_internal/forest_policy.rs`
8. keep bucket and bucket-root types in
   `crates/z00z_storage/src/assets/types_identity.rs` and
   `crates/z00z_storage/src/assets/types_record.rs`; do not add a separate
   `crates/z00z_storage/src/assets/store_internal/forest_types.rs`
9. `crates/z00z_storage/src/assets/store_internal/tree_id.rs`
10. new `crates/z00z_storage/src/assets/store_internal/forest_store.rs`
11. new `crates/z00z_storage/src/assets/store_internal/forest_plan.rs`
12. new `crates/z00z_storage/src/assets/store_internal/forest_commit.rs`
13. new `crates/z00z_storage/src/assets/store_internal/forest_journal.rs`
14. `crates/z00z_storage/src/assets/store_internal/redb_backend_state.rs`
15. `crates/z00z_storage/src/assets/store_internal/redb_backend_helpers.rs`
16. `crates/z00z_storage/src/assets/store_internal/redb_backend.rs`
17. `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs`
18. `crates/z00z_storage/src/assets/model.rs`
19. `crates/z00z_storage/src/assets/proof.rs`
20. new `crates/z00z_storage/src/assets/store_internal/forest_proof.rs`
21. `crates/z00z_storage/src/assets/store_internal/proof_help.rs`
22. `crates/z00z_storage/src/assets/store_internal/store_query.rs`
23. `crates/z00z_storage/src/assets/store_internal/store_rows.rs`
24. `crates/z00z_storage/tests/assets/test_backend_facade_contract.rs`
25. `crates/z00z_storage/tests/test_phase051_golden_corpus.rs`
26. `crates/z00z_storage/tests/test_phase051_guardrails.rs`
27. new `crates/z00z_storage/tests/test_phase052_forest_backend.rs`
28. new `crates/z00z_storage/tests/test_phase052_forest_proofs.rs`
29. new `crates/z00z_storage/tests/test_phase052_recovery.rs`
30. new `crates/z00z_storage/tests/test_phase052_guardrails.rs`
31. `crates/z00z_storage/tests/test_redb_rehydrate.rs`
32. `crates/z00z_storage/tests/test_checkpoint_root_binding.rs`
33. `crates/z00z_storage/benches/assets/shard.rs`
34. `crates/z00z_storage/benches/assets/nested.rs`
35. `crates/z00z_storage/Cargo.toml` only if a new benchmark target is needed

## ✅ Validation Matrix

| Source section | Required theme | TODO coverage | Status |
| --- | --- | --- | --- |
| `1.2 Live Asset Terms Versus Generalized Rights Terms` | keep `AssetLeaf` and `AssetStateRoot` as live code-backed nouns while capturing generalized-rights follow-ups | execution rules; `052-10`; `052-12`; `052-16`; `052-17` | Mapped |
| `3.2 Physical Bottleneck To Remove` | replace one shared physical commit wall with real physical forest commits | `052-03`; `052-04`; `052-11` | Mapped |
| `3.3 Compatibility Backend Requirement` | use compatibility as migration reference and rollback guard | `052-09`; `052-12` | Mapped |
| `4.1 Topology Overview` | add definition, serial, bucket, and asset JMT layers behind stable semantics | `052-02`; `052-03`; `052-04` | Mapped |
| `4.2 Public Contract` | callers use `AssetPath`, not bucket ids or physical layout | `052-01`; `052-10` | Mapped |
| `4.3 Fixed Bucket Derivation` | deterministic bucket id from path and committed policy | `052-02`; `052-06` | Mapped |
| `4.4 Why Fixed Buckets` | forbid adaptive split and merge in the first backend while preserving adaptive migration as future work | explicit phase boundary; `052-14` | Mapped |
| `5.1 Asset Path Proof Shape` | include bucket-policy recomputation and bucket proof segment | `052-06`; `052-07` | Mapped |
| `5.1.1 Current Compatibility Envelope vs Target HJMT Envelope` | measure encoded inclusion-proof shell and bucket-layer delta without freezing estimates as protocol constants | `052-06`; `052-11` | Mapped |
| `5.1.1 Transaction-Scale Proof Size Example` | record measured shared-parent proof-size evidence versus compatibility baseline | `052-11`; completion gate | Mapped |
| `5.2 Root Taxonomy` | keep semantic roots and backend roots distinct | `052-01`; `052-10`; `052-12` | Mapped |
| `5.3 Inclusion, Deletion, And Non-Existence` | implement or explicitly reject each proof family fail-closed | `052-06`; `052-07` | Mapped |
| `5.4 Path Index Boundary` | keep path index internal and rebuildable | `052-08`; `052-10` | Mapped |
| `6.1 Insert Flow` | group by definition, serial, and bucket; commit children before parents | `052-04`; `052-05` | Mapped |
| `6.2 Delete Flow` | delete terminal leaves, prune parents deterministically, and publish safely | `052-04`; `052-05`; `052-07` | Mapped |
| `7.1 Forest Commit Journal` | durable journal status lifecycle and fail-closed recovery | `052-05`; `052-08` | Mapped |
| `7.2 Backend Interface Boundary` | forest enters through `AssetTreeBackend` responsibilities | `052-01`; `052-10` | Mapped |
| `9.1 Rollout Phases` | compatibility default, fixed buckets, forest, journal, equivalence, config gate, green audit, and deferred candidates | `052-01` through `052-17` | Mapped |
| `9.2 Benchmark Plan` | benchmark broad, hot-serial, proof, and recovery workloads before public claims | `052-11`; `052-12` | Mapped |
| `9.3 Acceptance Criteria` | prove roots, proofs, crash safety, path rebuild, and no public bucket API | completion gate | Mapped |
| `12 Normative Requirement Summary` | JMT-REQ-001 through JMT-REQ-014 | all numbered tasks | Mapped |
| `13 Testing And Verification Strategy` | equivalence, crash, proof, and performance tests | dedicated test waves | Mapped |
| `13.4 Performance Tests` | compare workload and bucket-width benchmark matrix under realistic execution modes | `052-11`; `052-12` | Mapped |

## 🔍 Design Coverage Doublecheck

Coverage result:

- every implementation-relevant design feature has a numbered task, test
  target, or explicit exclusion;
- every future-only design feature is captured as a boundary rule instead of
  being silently implemented in Phase 052;
- simulator `scenario_1` is included as an e2e validation consumer through the
  storage facade.

| Design area | Coverage decision | Task or gate |
| --- | --- | --- |
| Key terms | Preserve live terms and future-only terms separately | execution rules; `052-10`; `052-16`; `052-17` |
| `1.1 Design Thesis` | Implement bucketed root-chained JMT forest | `052-02` through `052-05` |
| `1.2 Live Asset Terms Versus Generalized Rights Terms` | Keep `AssetLeaf` and `AssetStateRoot` live; keep `RightLeaf` and `SettlementStateRoot` out while planning future migrations | `052-10`; `052-16`; `052-17`; completion gate |
| `1.3 What This Design Does Not Claim` | Do not claim current repository already has forest backend; implement it behind facade | explicit phase boundary; `052-01` |
| `2.1 Main Protocol Requirements` | Keep leaf-oriented, checkpointed, path-local settlement evidence | `052-06`; `052-10`; simulator Stage 11 and Stage 13 |
| `2.2 Cross-Chain And External-Right Requirements` | Preserve definition-scoped semantics and no global serial authority | `052-02`; `052-04`; golden corpus cross-definition tests |
| `2.3 Machine And Agent Economy Requirements` | Hot-serial and high-volume workloads require fixed buckets | `052-04`; `052-11`; benchmark matrix |
| `2.4 RightLeaf As The Generalized Terminal Object` | Keep `RightLeaf` future-only, do not export it, and capture protocol duties separately | explicit phase boundary; `052-17` |
| `2.4.1 Legal-Defensibility Value Of RightLeaf` | Keep legal and governance widening future-only; Phase 052 preserves the storage boundary without pretending those contracts already exist | explicit phase boundary; `052-17` |
| `2.5 Linked Liability Requirements` | Keep path-local and family-local proof discipline | `052-06`; `052-07` |
| `2.6 OnionNet And Publication Requirements` | Keep transport, publication, and settlement separated | `052-10`; simulator guardrails |
| `2.7 Roadmap And Maturity Requirements` | Use staged rollout and compatibility oracle | `052-01`; `052-09`; `052-11` |
| `2.8 Uniqueness And Use-Case Requirements` | Optimize storage for rights mobility and selective evidence | `052-06`; `052-11`; simulation coverage |
| `3.1 Semantic Strengths Already Present` | Reuse current typed IDs, parent leaves, root taxonomy, and path index | `052-01`; `052-08`; `052-10` |
| `3.2 Physical Bottleneck To Remove` | Remove the single shared physical commit wall | `052-03`; `052-04`; benchmarks |
| `3.3 Compatibility Backend Requirement` | Compatibility remains oracle and rollback reference | `052-09` |
| `4.1 Topology Overview` | Add definition, serial, bucket, and terminal tree layers | `052-02`; `052-03`; `052-04` |
| `4.2 Public Contract` | Public callers remain on `AssetPath` and semantic roots | `052-01`; `052-10` |
| `4.3 Fixed Bucket Derivation` | Implement committed fixed bucket policy | `052-02`; `052-06` |
| `4.4 Why Fixed Buckets` | Exclude adaptive buckets from live Phase 052 and capture migration-proof follow-up | explicit phase boundary; `052-14` |
| `5.1 Asset Path Proof Shape` | Add bucket policy, bucket root leaf, bucket proof, and terminal proof checks | `052-06` |
| `5.1.1 Current Compatibility Envelope vs Target HJMT Envelope` | Measure encoded proof delta and keep estimates non-normative | `052-06`; `052-11` |
| `5.1.1 Transaction-Scale Proof Size Example` | Record measured shared-parent inclusion proof-size evidence against compatibility | `052-11`; completion gate |
| `5.2 Root Taxonomy` | Prevent backend root substitution | `052-10`; guardrail tests |
| `5.3 Inclusion, Deletion, And Non-Existence` | Implement or explicitly reject proof families fail-closed | `052-06`; `052-07` |
| `5.4 Path Index Boundary` | Keep path index rebuildable and internal | `052-08` |
| `6.1 Insert Flow` | Validate, group, derive bucket, journal, child commits, parent commits, publish root | `052-04`; `052-05` |
| `6.2 Delete Flow` | Resolve, derive bucket, delete terminal leaves, prune parents, publish safely | `052-04`; `052-05`; `052-07` |
| `6.3 Expected Performance By Workload` | Benchmark definitions, serials, hot serials, delete-heavy, proof-heavy, recovery | `052-11` |
| `6.4 Score Target` | Capture performance versus risk tradeoff; avoid overclaiming | `052-11`; completion gate |
| `7.1 Forest Commit Journal` | Add durable journal lifecycle and recovery | `052-05`; `052-08` |
| `7.2 Backend Interface Boundary` | Keep forest behind `AssetTreeBackend` | `052-01`; `052-10` |
| `7.3 Main Implementation Risks` | Map mismatch risks to fail-closed tests | `052-05` through `052-09` |
| `8.1 Public Visibility` | Do not publish bucket activity counters or physical layout as business meaning | explicit phase boundary; `052-10`; `052-15` |
| `8.2 Selective Disclosure` | Keep proofs path-local | `052-06`; simulator Stage 11 |
| `8.3 Checkpoint Evidence` | Bind prior and next semantic roots, not forest internals | `052-10`; simulator Stage 13 |
| `9.1 Rollout Phases` | Implement backend trait, buckets, forest, journal, equivalence, config gate, green audit, and deferred candidates | `052-01` through `052-17` |
| `9.2 Benchmark Plan` | Add benchmark evidence before performance claims | `052-11` |
| `9.3 Acceptance Criteria` | Make completion gate explicit | completion gate |
| `13.4 Performance Tests` | Run realistic workload, bucket-width, and proof benchmark matrix | `052-11`; `052-12` |
| `10 Design Rationale` | Preserve rejected alternatives as exclusions | explicit phase boundary |
| `11 Relationship To Z00Z Use Cases` | Validate private cash, external asset, audit, machine, liability, and publication flows through storage facade | golden corpus; simulator `scenario_1` |
| `12 Normative Requirement Summary` | Map JMT-REQ-001 through JMT-REQ-014 | phase implementation brief; validation matrix |
| `13 Testing And Verification Strategy` | Add equivalence, crash, proof, performance, and simulator tests | required test matrix; `052-12` |
| `14 Open Questions` | Keep dynamic buckets, public path index, global root registry, occupancy metadata, and generalized rights as future decisions | explicit phase boundary; `052-14`; `052-15`; `052-16`; `052-17` |
| `15 Conclusion` | Implement the balanced forest design without broad global state expansion | all gates |
| Appendix A and B | Preserve glossary and compact design summary as vocabulary guardrails | execution rules; `052-10`; `052-16`; `052-17` |

## 🚫 Explicit Phase Boundary

The following topics are intentionally out of scope for Phase 052:

- a fake forest backend;
- copied compatibility logic pretending to be physical forest implementation;
- a second public asset store facade;
- public APIs that accept `TreeId`, `ns_key`, namespace prefixes, branch
  ordering, physical key layout, raw backend roots, or bucket ids as authority;
- replacing `AssetStateRoot` with a generalized settlement root;
- exporting `RightLeaf` or `FeeEnvelope`;
- adaptive bucket split, merge, or migration proof machinery;
- proof-visible bucket occupancy counters unless the design source is updated;
- a second checkpoint verifier outside storage;
- a second proof decoder or branch-proof authority in wallet, validator,
  runtime, or simulator crates;
- benchmark-backed public performance claims before the Phase 052 benchmark
  matrix is green.

The future-only items above are not dropped. They are captured by
`052-14` through `052-17` as first-class follow-up candidates with separate
entry conditions and test duties.

## ⚙️ Concrete Execution Tasks

### 052-01 Backend Selection And Forest Skeleton

Spec references:

- `3.3 Compatibility Backend Requirement`
- `4.2 Public Contract`
- `7.2 Backend Interface Boundary`
- `9.1 Rollout Phases`

MANDATORY pre-read in `docs/Z00Z-JMT-Design.md`:

- section `3.3 Compatibility Backend Requirement`
- section `4.2 Public Contract`
- section `7.2 Backend Interface Boundary`
- section `9.1 Rollout Phases`

- [x] Add internal backend selection to `AssetStore` without changing the
  public `AssetTreeBackend` trait.
- [x] Add `AssetBackendMode` with compatibility default, forest mode, and
  dual-verify mode.
- [x] Add an internal `ForestBackend` skeleton that returns explicit
  unsupported-backend errors until its concrete operations land.
- [x] Route `backend_name()` through the selected mode as diagnostics only.
- [x] Reject unknown backend config values fail-closed.
- [x] Keep all existing callers on `AssetStore` and `AssetTreeBackend`.

Files:

- `crates/z00z_storage/src/assets/store.rs`
- `crates/z00z_storage/src/assets/store_internal/forest_config.rs`
- `crates/z00z_storage/tests/assets/test_backend_facade_contract.rs`

Unit tests:

- [x] add `test_backend_mode_default`
  - compatibility remains the default;
  - `AssetTreeBackend::backend_name(&store)` returns compatibility by default.
- [x] add `test_backend_mode_rejects_unknown`
  - unknown backend mode rejects instead of silently falling back.
- [x] extend `test_facade_hides_tree_shape`
  - facade still does not expose `TreeId`, `RootHash`, `KeyHash`, or `ns_key`.

E2E or integration tests:

- [x] extend `crates/z00z_storage/tests/test_phase051_guardrails.rs`
  - downstream crates still do not import physical layout names.

Exit condition:

- Forest selection exists behind the Phase 051 facade, compatibility remains
  default, and no downstream code has a new physical authority path.

### 052-02 Fixed Bucket Policy And Root Leaf Types

Spec references:

- `4.1 Topology Overview`
- `4.3 Fixed Bucket Derivation`
- `4.4 Why Fixed Buckets Instead Of Adaptive Buckets`
- `12 Normative Requirement Summary`

MANDATORY pre-read in `docs/Z00Z-JMT-Design.md`:

- section `4.1 Topology Overview`
- section `4.3 Fixed Bucket Derivation`
- section `12 Normative Requirement Summary`

- [x] Add `BucketId` with fixed serialized width and stable ordering.
- [x] Add `BucketPolicy` with versioned, verifier-visible fields only.
- [x] Add `BucketRootLeaf` that binds definition, serial, bucket id, asset JMT
  root, and bucket policy id.
- [x] Implement deterministic bucket derivation from `definition_id`,
  `serial_id`, `asset_id`, and policy id through domain-separated hashing.
- [x] Freeze endian and canonical byte encoding in tests.
- [x] Reject invalid `bucket_bits`, invalid width, and single-bucket policy
  where the minimum bucket count forbids it.
- [x] Keep bucket ids outside normal put, delete, lookup, list, and checkpoint
  APIs.

Files:

- `crates/z00z_storage/src/assets/types_identity.rs`
- `crates/z00z_storage/src/assets/types_record.rs`
- `crates/z00z_storage/src/assets/types.rs`
- `crates/z00z_storage/src/assets/store_internal/forest_policy.rs`
- `crates/z00z_storage/src/assets/mod.rs`

Unit tests:

- [x] add `test_bucket_policy_derives_stably`
  - same path and policy derive the same bucket id;
  - different definition, serial, asset, or policy id changes the bucket id.
- [x] add `test_bucket_policy_checks_bounds`
  - invalid `bucket_bits`, width, and minimum count reject.
- [x] add `test_bucket_leaf_encodes_stably`
  - `BucketRootLeaf` encoding is deterministic and deny-unknown-fields where
    serialized.

E2E or integration tests:

- [x] add `test_phase052_bucket_guardrails`
  - normal asset APIs never require or accept `BucketId`;
  - proof-only bucket metadata remains storage-owned.

Exit condition:

- Fixed bucket policy and bucket root types exist with deterministic encoding,
  but no caller outside storage can use bucket ids as write authority.

### 052-03 Forest Tree Store And Physical Layout

Spec references:

- `3.2 Physical Bottleneck To Remove`
- `4.1 Topology Overview`
- `6.1 Insert Flow`
- `6.2 Delete Flow`

MANDATORY pre-read in `docs/Z00Z-JMT-Design.md`:

- section `3.2 Physical Bottleneck To Remove`
- section `4.1 Topology Overview`
- section `6.1 Insert Flow`
- section `6.2 Delete Flow`

- [x] Add private tree identities for bucket and bucket-local asset trees.
- [x] Add a forest tree store that can commit independent child JMTs before
  parent JMTs.
- [x] Keep physical key derivation private to `store_internal`.
- [x] Preserve deterministic sort order for definition, serial, bucket, and
  asset updates.
- [x] Keep path-index writes separate from the public semantic root.
- [x] Avoid reusing the compatibility shared `flat_root` as forest authority.

Files:

- `crates/z00z_storage/src/assets/store_internal/tree_id.rs`
- `crates/z00z_storage/src/assets/store_internal/forest_store.rs`
- `crates/z00z_storage/src/assets/store_internal/tree_store.rs`
- `crates/z00z_storage/src/assets/store_internal/store_roots.rs`

Unit tests:

- [x] add `test_forest_tree_ids_private`
  - bucket tree identities are crate-private;
  - assets public module does not export physical tree names.
- [x] add `test_forest_tree_order_stable`
  - committing identical rows in different order produces identical child and
    semantic roots.
- [x] add `test_forest_child_roots_distinct`
  - definition, serial, bucket, and terminal roots cannot collide by namespace.

E2E or integration tests:

- [x] add `test_phase052_layout_hidden`
  - source scans prove no wallet, validator, runtime, or simulator file imports
    forest tree ids, namespace helpers, or raw backend root authority.

Exit condition:

- The target physical tree layout exists as private storage implementation
  state and cannot be consumed as public authority.

### 052-04 Forest Batch Planner For Inserts And Deletes

Spec references:

- `6.1 Insert Flow`
- `6.2 Delete Flow`
- `6.3 Expected Performance By Workload`
- `9.2 Benchmark Plan`

MANDATORY pre-read in `docs/Z00Z-JMT-Design.md`:

- section `6.1 Insert Flow`
- section `6.2 Delete Flow`
- section `6.3 Expected Performance By Workload`

- [x] Add a forest planner that validates every `StoreOp` before mutation.
- [x] Group puts and deletes by definition, serial, and derived bucket.
- [x] Detect duplicate canonical paths before building child commits.
- [x] Reject missing deletes before mutating any child tree.
- [x] Commit bucket-local terminal leaves in deterministic batches.
- [x] Update bucket, serial, and definition parent leaves in dependency order.
- [x] Preserve no-op and idempotent put behavior from the Phase 051 corpus.
- [x] Keep exported `AssetStateRoot` semantically equal to compatibility for
  the same operation sequence.

Files:

- `crates/z00z_storage/src/assets/store_internal/forest_plan.rs`
- `crates/z00z_storage/src/assets/store_internal/forest_commit.rs`
- `crates/z00z_storage/src/assets/store_internal/tx_plan_types.rs`
- `crates/z00z_storage/src/assets/store_internal/tx_plan_engine.rs`
- `crates/z00z_storage/src/assets/model.rs`
- `crates/z00z_storage/src/assets/store_internal/store_query.rs`

Unit tests:

- [x] add `test_forest_insert_many`
  - multiple definitions, serials, and buckets commit successfully.
- [x] add `test_forest_delete_many`
  - deletes update bucket and parent roots deterministically.
- [x] add `test_forest_hot_serial`
  - one definition and serial distributes across fixed buckets.
- [x] add `test_forest_rejects_duplicate`
  - duplicate path rejects with state preserved.
- [x] add `test_forest_rejects_missing`
  - missing delete rejects with state preserved.
- [x] add `test_forest_noop_root`
  - empty operation set returns the current root.

E2E or integration tests:

- [x] extend `test_phase051_golden_corpus.rs` or add
  `test_phase052_forest_backend.rs`
  - run insert-many, delete-many, hot-serial, cross-definition, duplicate path,
    delete-missing, reorder-stable roots, and no-op root against forest mode.

Exit condition:

- Forest batch mutation is real physical forest work, not copied compatibility
  mutation, and it preserves Phase 051 semantic outcomes.

### 052-05 Forest Commit Journal And Recovery State

Spec references:

- `7.1 Forest Commit Journal`
- `7.3 Main Implementation Risks`
- `9.3 Acceptance Criteria`
- `13.2 Crash Tests`

MANDATORY pre-read in `docs/Z00Z-JMT-Design.md`:

- section `7.1 Forest Commit Journal`
- section `7.3 Main Implementation Risks`
- section `13.2 Crash Tests`

- [x] Add `ForestCommitJournalEntry` and `ForestCommitStatus`.
- [x] Persist `Prepared` before child commits are made durable.
- [x] Persist child commit digests before parent roots are committed.
- [x] Persist parent commit digest before publishing the semantic root.
- [x] Persist `RootPublished` only after the exported `AssetStateRoot` is safe.
- [x] Recover by completing or rolling back to the last published semantic root.
- [x] Fail closed on child digest mismatch, parent digest mismatch, or journal
  status regression.
- [x] Keep in-memory rollback snapshots as a local guard, but do not treat them
  as durable forest recovery.

Files:

- `crates/z00z_storage/src/assets/store_internal/forest_journal.rs`
- `crates/z00z_storage/src/assets/store_internal/forest_commit.rs`
- `crates/z00z_storage/src/assets/store_internal/redb_backend_state.rs`
- `crates/z00z_storage/src/assets/store_internal/redb_backend_helpers.rs`
- `crates/z00z_storage/src/assets/store_internal/redb_backend.rs`
- `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs`
- `crates/z00z_storage/src/assets/store_internal/tx_plan_batches.rs`

Unit tests:

- [x] add `test_forest_journal_order`
  - status transitions can only move forward.
- [x] add `test_forest_rejects_child_mismatch`
  - mismatched child digest fails closed.
- [x] add `test_forest_rejects_parent_mismatch`
  - mismatched parent digest fails closed.
- [x] add `test_forest_rolls_back_prepared`
  - interruption before child commit restores previous root.
- [x] add `test_forest_publishes_once`
  - repeated recovery does not republish divergent roots.

E2E or integration tests:

- [x] add `crates/z00z_storage/tests/test_phase052_recovery.rs`
  - simulate interruption before child commit;
  - simulate interruption after all child commits before parent commit
    (`ChildrenCommitted`);
  - record that partial per-child durable state is not representable because
    child rows commit as one child stage;
  - simulate interruption after parent commit before publication;
  - simulate reload after publication.

Exit condition:

- Forest recovery never exposes a parent root whose child roots are missing,
  stale, or unverified by the journal.

### 052-06 Forest Proof Envelope And Verifier Checks

Spec references:

- `5.1 Asset Path Proof Shape`
- `5.2 Root Taxonomy`
- `5.3 Inclusion, Deletion, And Non-Existence`
- `12 Normative Requirement Summary`

MANDATORY pre-read in `docs/Z00Z-JMT-Design.md`:

- section `5.1 Asset Path Proof Shape`
- section `5.2 Root Taxonomy`
- section `5.3 Inclusion, Deletion, And Non-Existence`

- [x] Add forest proof encoding under storage ownership.
- [x] Include `AssetStateRoot`, `AssetPath`, `BucketPolicy`,
  `DefinitionRootLeaf`, `SerialRootLeaf`, `BucketRootLeaf`, terminal leaf, and
  all required branch proof bytes.
- [x] Verify bucket id by recomputing it from path and committed policy.
- [x] Verify definition, serial, bucket, and terminal proof segments against
  the expected chained roots.
- [x] Keep `backend_root` diagnostic only and never allow it to replace
  `AssetStateRoot`.
- [x] Keep compatibility proofs inclusion-only unless explicitly upgraded by
  storage-owned version handling.
- [x] Add explicit reject variants for bucket policy mismatch, bucket id
  mismatch, bucket root mismatch, terminal proof mismatch, and unsupported
  forest version.
- [x] Measure representative serialized inclusion-proof size for single-leaf
  and shared-parent multi-leaf envelopes and record the evidence without
  freezing the measured byte counts as protocol constants.

Plan 04 evidence note:

- `test_phase052_forest_proofs.rs` records live encoded-size samples for
  single-leaf and shared-parent inclusion proofs. The samples are regression
  evidence only, not normative byte-count constants. Broader benchmark and
  proof-size evidence remains part of `052-11`.

Files:

- `crates/z00z_storage/src/assets/proof.rs`
- `crates/z00z_storage/src/assets/store_internal/forest_proof.rs`
- `crates/z00z_storage/src/assets/store_internal/proof_help.rs`
- `crates/z00z_storage/src/assets/store_internal/store_query.rs`
- `crates/z00z_storage/src/assets/types_record.rs`

Unit tests:

- [x] add `test_forest_proof_accepts_inclusion`
  - valid forest proof accepts under the live semantic root.
- [x] add `test_forest_rejects_policy`
  - tampered bucket policy rejects.
- [x] add `test_forest_rejects_bucket`
  - tampered bucket id or bucket root leaf rejects.
- [x] add `test_forest_rejects_branch`
  - wrong definition, serial, bucket, or terminal branch rejects.
- [x] add `test_forest_rejects_root`
  - wrong semantic root and wrong root binding reject.
- [x] add `test_forest_rejects_version`
  - unsupported proof envelope version rejects.

E2E or integration tests:

- [x] add `crates/z00z_storage/tests/test_phase052_forest_proofs.rs`
  - run proof verification success and failure matrix for compatibility and
    forest modes;
  - prove state preservation after every rejecting workload.

Exit condition:

- Storage can verify forest inclusion proofs through one storage-owned decoder
  and rejects every tampered bucket, root, path, leaf, and branch component.

### 052-07 Deletion And Non-Existence Proof Semantics

Spec references:

- `5.3 Inclusion, Deletion, And Non-Existence`
- `6.2 Delete Flow`
- `13.3 Proof Tests`

MANDATORY pre-read in `docs/Z00Z-JMT-Design.md`:

- section `5.3 Inclusion, Deletion, And Non-Existence`
- section `6.2 Delete Flow`
- section `13.3 Proof Tests`

- [x] Defer live deletion proof fields because prior root, deleted path, next
  root, and affected parent-root updates cannot yet be validated fail-closed by
  the live wire format.
- [x] Defer live non-existence proof fields because canonical default
  commitment semantics for the exact root and path are not live yet.
- [x] Keep `DEFAULT_VALUE_COMMITMENT` and `DEFAULT_CHILD_COMMITMENT`
  requirements as future proof-family gates; Phase 052 does not treat
  node-local `not found` results as proofs.
- [x] Reject present-key non-existence attempts by keeping the whole
  non-existence proof family explicitly unsupported fail-closed.
- [x] Reject tampered non-existence attempts by keeping the whole
  non-existence proof family explicitly unsupported fail-closed until default
  commitment semantics land.
- [x] Record non-existence proof-size status as unsupported fail-closed; defer
  prove time, verify time, serialized proof size, and aggregation throughput
  measurements until absent-key proofs are live.
- [x] Keep deletion and non-existence unsupported in compatibility mode unless
  compatibility receives explicit storage-owned semantics.
- [x] Do not introduce tombstones unless the design source is updated.
- [x] Keep forest deletion and non-existence proof families explicitly
  unsupported until canonical default-commitment proof semantics land.

Plan 04 evidence note:

- `ForestProofFamily::Deletion` and `ForestProofFamily::NonExistence` return
  `UnsupportedProofFamily`; inclusion remains the only live forest proof
  family.
- Real deletion proof fields, default commitments, present-key absence
  rejection, absence replay consistency, and absence proof benchmarks remain
  open until a later proof-family implementation can validate them honestly.

Files:

- `crates/z00z_storage/src/assets/proof.rs`
- `crates/z00z_storage/src/assets/store_internal/forest_proof.rs`
- `crates/z00z_storage/src/assets/store_internal/forest_policy.rs`
- `crates/z00z_storage/src/assets/store_internal/forest_commit.rs`

Unit tests:

- [x] add `test_forest_absence_and_deletion_families_remain_explicitly_unsupported`
  - inclusion is accepted;
  - deletion and non-existence families reject as unsupported.

- [x] Defer `test_forest_delete_proof_ok` to a future proof-family phase; Phase
  052 keeps deletion proofs unsupported fail-closed instead of accepting a
  placeholder transition proof.
- [x] Defer `test_forest_delete_rejects_root` to a future proof-family phase;
  Phase 052 rejects the deletion proof family before prior or next roots can
  become authority.
- [x] Defer `test_forest_nonexist_empty` to a future proof-family phase; Phase
  052 rejects empty-tree absence claims until default commitments are live.
- [x] Defer `test_forest_nonexist_random` to a future proof-family phase; Phase
  052 rejects absent-key proof claims until default commitments are live.
- [x] Defer `test_forest_nonexist_present` to a future proof-family phase;
  Phase 052 rejects the whole non-existence family, including present-key
  absence attempts.
- [x] Defer `test_forest_nonexist_replay_consistent` to a future proof-family
  phase; Phase 052 has no accepted absence payload shape to replay.
- [x] Defer `test_forest_nonexist_tamper` to a future proof-family phase; Phase
  052 rejects the whole non-existence family before tampered default, index,
  bucket, or root values can validate.

E2E or integration tests:

- [x] Extend `test_phase052_forest_proofs.rs` with explicit unsupported-family
  rejection for deletion and non-existence proofs; defer live deletion,
  non-existence replay consistency, and mixed inclusion plus non-existence
  success workloads until a future proof-family phase.

Exit condition:

- Deletion and non-existence proof families are either live and fail-closed or
  explicitly unsupported; no placeholder proof is accepted.

### 052-08 Reload Validation And Path-Index Rebuild

Spec references:

- `5.4 Path Index Boundary`
- `7.1 Forest Commit Journal`
- `9.3 Acceptance Criteria`
- `13.2 Crash Tests`

MANDATORY pre-read in `docs/Z00Z-JMT-Design.md`:

- section `5.4 Path Index Boundary`
- section `7.1 Forest Commit Journal`
- section `9.3 Acceptance Criteria`

- [x] Rebuild `asset_id -> AssetPath` from committed forest leaves during
  reload.
- [x] Validate rebuilt path index against forest terminal leaves.
- [x] Reject path index rows that bind an asset id to a different path.
- [x] Validate `AssetStateRoot` from committed forest parent leaves.
- [x] Validate checkpoint metadata against prior and next semantic roots.
- [x] Validate journal status before accepting persisted forest state.
- [x] Keep path-index roots private unless the design source changes.

Files:

- `crates/z00z_storage/src/assets/store_internal/store_rows.rs`
- `crates/z00z_storage/src/assets/store_internal/redb_backend.rs`
- `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs`
- `crates/z00z_storage/src/assets/store_internal/forest_journal.rs`
- `crates/z00z_storage/src/assets/store_internal/forest_store.rs`

Unit tests:

- [x] add `test_forest_rebuilds_index`
  - lookup by asset id works after rebuild.
- [x] add `test_forest_rejects_path_drift`
  - path drift rejects through forest child digest validation or rebuild
    comparison before state is accepted.
- [x] add `test_forest_reload_root`
  - reload recomputes the same semantic root.
- [x] add `test_forest_reload_journal`
  - invalid journal status rejects reload.

E2E or integration tests:

- [x] extend `crates/z00z_storage/tests/test_redb_rehydrate.rs`
  - checkpoint metadata reload validation now rejects prior-root, next-root,
    snapshot, exec, draft, checkpoint, statement, and proof drift on the live
    storage checkpoint path.
- [x] extend `crates/z00z_storage/tests/test_phase052_recovery.rs`
  - path-index rebuild after crash and recovery.
- [x] add forest checkpoint seal/reload coverage after forest proof snapshots
  and proof-envelope semantics land; forest checkpoint-attested execution now
  requires matching canonical tx rows and reloads through the forest journal.

Exit condition:

- A forest-backed store can reload from durable rows, rebuild internal lookup
  state, and reject any path-index or journal drift.

### 052-09 Dual-Backend Equivalence Corpus

Spec references:

- `3.3 Compatibility Backend Requirement`
- `9.1 Rollout Phases`
- `13.1 Equivalence Tests`

MANDATORY pre-read in `docs/Z00Z-JMT-Design.md`:

- section `3.3 Compatibility Backend Requirement`
- section `9.1 Rollout Phases`
- section `13.1 Equivalence Tests`

- [x] Convert the Phase 051 backend case table from one executable backend to
  compatibility plus forest once forest mode is implemented.
- [x] Add dual-verify mode that applies the same operation stream to
  compatibility and forest and compares semantic outputs.
- [x] Compare root, check root, get, lookup, list, proof result class, reload
  result, checkpoint result, and state preservation after rejects.
- [x] Keep mismatches as hard errors with typed context.
- [x] Keep compatibility as the oracle for semantic outcomes until a later
  protocol migration updates the design source.

Files:

- `crates/z00z_storage/tests/test_phase051_golden_corpus.rs`
- `crates/z00z_storage/tests/test_phase052_forest_backend.rs`
- `crates/z00z_storage/tests/assets/test_backend_facade_contract.rs`
- `crates/z00z_storage/src/assets/store.rs`
- `crates/z00z_storage/src/assets/store_internal/dual_verify.rs`
- `crates/z00z_storage/src/assets/store_internal/forest_config.rs`
- `crates/z00z_storage/src/assets/store_internal/forest_store.rs`
- `crates/z00z_storage/src/assets/store_internal/test_whitebox_state.rs`

Unit tests:

- [x] add `test_dual_verify_reports_drift`
  - injected mismatch reports a hard backend error.
- [x] add `test_dual_verify_keeps_state`
  - rejecting workload preserves both backend states.

E2E or integration tests:

- [x] run corpus for:
  - insert-many;
  - delete-many;
  - hot-serial;
  - cross-definition;
  - duplicate path;
  - delete-missing;
  - reorder-stable roots;
  - no-op root;
  - proof verification success and fail;
  - reload-after-crash;
  - checkpoint seal and reload;
  - path-index rebuild;
  - unsupported or malformed proof envelope rejection.

Exit condition:

- Compatibility and forest backends produce identical semantic outcomes for
  the golden corpus, and every mismatch fails the test that found it.

### 052-10 Checkpoint And Downstream Guardrail Closure

Spec references:

- `5.2 Root Taxonomy`
- `8.3 Checkpoint Evidence`
- `9.3 Acceptance Criteria`
- `12 Normative Requirement Summary`

MANDATORY pre-read in `docs/Z00Z-JMT-Design.md`:

- section `5.2 Root Taxonomy`
- section `8.3 Checkpoint Evidence`
- section `12 Normative Requirement Summary`

- [x] Keep checkpoint-facing evidence bound to prior and next
  `AssetStateRoot`.
- [x] Ensure `CheckRoot` remains checkpoint evidence and not a backend root.
- [x] Ensure forest backend root bytes never appear as public checkpoint root
  authority.
- [x] Ensure wallet, validator, runtime, and simulator code still consume
  storage-owned proof checks instead of decoding physical branch proofs.
- [x] Add guardrails for forbidden future-only terms in live storage exports.
- [x] Update storage documentation only if it remains explicit about private
  backend roots and public semantic roots; no storage documentation update was
  required for this slice, and existing diagnostic-backend-root wording stays
  source-guarded.

Files:

- `crates/z00z_storage/tests/test_phase052_guardrails.rs`
- `crates/z00z_storage/tests/test_checkpoint_root_binding.rs`
- `crates/z00z_storage/tests/test_phase051_guardrails.rs`
- `crates/z00z_runtime/validators/src/checkpoint_flow.rs`
- `crates/z00z_runtime/validators/src/verdicts.rs`
- `crates/z00z_wallets/src/tx/commit_audit.rs`
- `crates/z00z_wallets/src/tx/claim/claim_tx_verifier_impl_proof.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_preparation_core.rs`
- `crates/z00z_simulator/src/scenario_1/stage_11_utils/jmt_wallet_scan.rs`
- `crates/z00z_simulator/src/scenario_1/stage_11_utils/stage_11_apply.rs`
- `crates/z00z_simulator/src/scenario_1/stage_12.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_utils/storage.rs`
- `crates/z00z_simulator/src/scenario_1/stage_5_utils/transfer_lane_impl.rs`
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/test_bundle_lane_impl_suite.rs`
- `crates/z00z_simulator/src/scenario_1/stage_7.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/storage_view.rs`
- `crates/z00z_simulator/src/scenario_1/runner_verify.rs`

Unit tests:

- [x] add `test_forest_root_taxonomy`
  - `AssetStateRoot`, `CheckRoot`, and backend root roles stay distinct through
    `test_root_taxonomy_guard`, `test_backend_root_mix`, and
    `test_wrong_semantic_root_rejects`.
- [x] add `test_future_terms_hidden`
  - live storage exports do not introduce `SettlementStateRoot`, `RightLeaf`,
    or `FeeEnvelope`; covered by `test_future_terms_guard`.

E2E or integration tests:

- [x] extend checkpoint root binding tests
  - compatibility and forest modes seal and reload checkpoint evidence through
    the same semantic root contract.
- [x] extend downstream source guards
  - downstream crates still do not contain physical layout authority strings.
  - Stage 7 facade delegation, Stage 11 storage-backed apply, Stage 12
    checkpoint finalization, Stage 13 replay, and `runner_verify` have
    positive semantic-root or storage-proof source anchors.

Exit condition:

- Checkpoint and downstream consumers remain bound to storage semantics rather
  than forest physical layout.

### 052-11 Rollout Configuration And Benchmark Evidence

Spec references:

- `6.3 Expected Performance By Workload`
- `9.1 Rollout Phases`
- `9.2 Benchmark Plan`
- `9.3 Acceptance Criteria`

MANDATORY pre-read in `docs/Z00Z-JMT-Design.md`:

- section `6.3 Expected Performance By Workload`
- section `9.2 Benchmark Plan`
- section `9.3 Acceptance Criteria`

- [x] Keep compatibility default until `052-09` and `052-10` are green.
- [x] Add explicit forest mode for local validation.
- [x] Add dual-verify mode for local and CI equivalence checks where runtime
  cost is acceptable.
- [x] Add focused benchmark coverage for insert-many definitions, insert-many
  serials, hot serial, delete-heavy, proof-heavy, and recovery replay.
- [x] Add multithread async benchmark coverage for `multi-insert` and
  `multi-delete` execution across broad, hot-definition, and hot-serial
  workload shapes.
- [x] Compare compatibility baseline, bucket-width variants, and the forest
  backend on random, hot-definition, hot-serial, delete-heavy, proof-heavy,
  and recovery workloads; if a definition-sharded-only comparison lane is
  needed, keep it benchmark-only rather than a production backend.
- [x] Measure planning time, child commit time, parent commit time, journal
  time, proof time, proof size, and reload time separately.
- [x] Measure inclusion prove time, inclusion verify time, shared-parent
  transaction proof size, fail-closed non-existence prove attempt time,
  unsupported non-existence serialized proof-size status, and absence
  aggregation throughput.
- [x] Do not claim public performance wins unless benchmark output is attached
  to the phase closeout.

Files:

- `crates/z00z_storage/src/assets/store_internal/forest_config.rs`
- `crates/z00z_storage/benches/assets/shard.rs`
- `crates/z00z_storage/benches/assets/nested.rs`
- `crates/z00z_storage/Cargo.toml`

Unit tests:

- [x] add `test_forest_mode_enabled`
  - explicit forest mode selects forest backend.
- [x] add `test_dual_mode_enabled`
  - explicit dual mode compares backends.
- [x] add `test_backend_default_safe`
  - no config still selects compatibility.

E2E or integration tests:

- [x] run focused release tests for compatibility default, forest mode, and
  dual-verify mode.
- [x] run storage benchmarks with representative workload sizes, multithread
  async `multi-insert` and `multi-delete` execution, inclusion proof-size
  evidence, and explicit non-existence unsupported fail-closed status, then
  record the results in phase closeout notes.

Exit condition:

- Forest can be selected only through explicit configuration, compatibility is
  still the default, and benchmark evidence exists for the design's
  performance claims, async batch workloads, and proof-size checks.

### 052-12 Verification Closeout

Spec references:

- `9.3 Acceptance Criteria`
- `12 Normative Requirement Summary`
- `13 Testing And Verification Strategy`

MANDATORY pre-read in `docs/Z00Z-JMT-Design.md`:

- section `9.3 Acceptance Criteria`
- section `12 Normative Requirement Summary`
- section `13 Testing And Verification Strategy`

- [x] Run the mandatory verification order from this TODO.
- [x] Run storage-focused release tests after every task that changes storage
  logic or tests.
- [x] Run full release validation when forest mode, proof format, checkpoint
  behavior, or downstream guardrails change.
- [x] Run the YOLO review prompt at least three times and require at least two
  consecutive no-significant-issue runs before closeout.
- [x] Record exact commands, backend modes, and results in closeout notes.
- [x] Record any deferred proof family, benchmark, or protocol migration as a
  source-backed follow-up rather than hiding it inside this phase.

Files:

- `.planning/phases/052-HJMT-Backend/052-TODO.md`
- phase closeout notes chosen by the active GSD workflow

Unit tests:

- [x] all Phase 052 unit tests pass in the focused release set.

E2E or integration tests:

- [x] `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- [x] `cargo test -p z00z_storage --release --features test-fast`
- [x] `cargo test -p z00z_simulator --release --features wallet_debug_dump
  scenario_1`
- [x] `cargo run --release -p z00z_simulator --bin scenario_1 --features
  wallet_debug_dump`
- [x] `cargo test --release --features test-fast --features wallet_debug_dump`
- [x] `.github/prompts/gsd-review-tasks-execution.prompt.md` in YOLO mode,
  three runs minimum, with two consecutive no-significant-issue results.

Exit condition:

- The forest backend is implemented behind the Phase 051 facade, compatibility
  and forest are semantically equivalent across the required corpus, proof and
  recovery reject matrices are fail-closed, and no downstream crate uses
  physical layout as authority.

### 052-13 Green-State Audit For Plans 052-01 Through 052-06

Spec references:

- `9.3 Acceptance Criteria`
- `12 Normative Requirement Summary`
- `13 Testing And Verification Strategy`
- `14 Open Questions`

MANDATORY pre-read in `docs/Z00Z-JMT-Design.md`:

- section `9.3 Acceptance Criteria`
- section `13 Testing And Verification Strategy`
- section `14 Open Questions`

- [x] Verify every in-scope backend gate from plans `052-01` through
  `052-06` has executed evidence.
- [x] Verify no selected forest operation still returns skeleton,
  placeholder, copied compatibility, or fake success behavior.
- [x] Verify compatibility remains default until equivalence, guardrails,
  checkpoint, reload, proof, benchmark, and scenario evidence are green.
- [x] Verify no downstream crate uses bucket ids, tree ids, branch ordering,
  namespace bytes, backend roots, or physical layout as authority.
- [x] Verify deferred protocol work is recorded as follow-up scope rather than
  hidden inside a green Phase 052 summary.

Files:

- `.planning/phases/052-HJMT-Backend/052-06-SUMMARY.md`
- `.planning/phases/052-HJMT-Backend/052-SUMMARY.md`
- `.planning/STATE.md`

Unit tests:

- [x] no new unit tests are required for a docs-only audit, but missing
  in-scope evidence must route back to the owning implementation plan.

E2E or integration tests:

- [x] rerun any focused or broad command whose evidence is missing from the
  implementation summary.
- [x] run `git diff --check -- .planning/phases/052-HJMT-Backend
  .planning/STATE.md` for docs-only audit edits.

Exit condition:

- Phase 052 in-scope backend work is green by evidence, and future work is
  visibly deferred instead of implied shipped.

### 052-14 Adaptive Bucket Split, Merge, And Migration Proof Candidate

Spec references:

- `4.4 Why Fixed Buckets Instead Of Adaptive Buckets`
- `6.3 Expected Performance By Workload`
- `7.3 Main Implementation Risks`
- `8.1 Public Visibility`
- `13 Testing And Verification Strategy`
- `14 Open Questions`

MANDATORY pre-read in `docs/Z00Z-JMT-Design.md`:

- section `4.4 Why Fixed Buckets Instead Of Adaptive Buckets`
- section `8.1 Public Visibility`
- section `14 Open Questions`

- [x] Capture `HJMT-Adaptive-Buckets-And-Migration-Proofs` as a future phase
  candidate, not Phase 052 live runtime work.
- [x] Require fixed-bucket forest implementation, benchmarks, proof-size
  evidence, recovery evidence, and privacy review as entry conditions.
- [x] Define split proofs, merge proofs, migration proofs, bucket epochs,
  old-policy and new-policy compatibility, historical proof compatibility,
  and deterministic replay across policy changes.
- [x] Define crash recovery for split or merge interruptions and fail-closed
  verification for old-root or new-root confusion.
- [x] Define benchmark comparison against fixed buckets before rollout.

Files:

- `.planning/phases/052-HJMT-Backend/052-08-PLAN.md`
- future phase artifacts only if the active GSD workflow explicitly promotes
  the candidate.

Unit tests:

- [x] future tests must cover split, merge, migration, epoch binding, policy
  mismatch, stale proof, replay drift, and recovery interruption cases.

E2E or integration tests:

- [x] future simulator coverage must keep `scenario_1` on storage-owned APIs
  through any adaptive bucket migration.

Exit condition:

- Adaptive bucket work is first-class future scope with hard entry conditions
  and zero live Phase 052 runtime behavior.

### 052-15 Bucket Occupancy Metadata Privacy Candidate

Spec references:

- `4.1 Topology Overview`
- `4.3 Fixed Bucket Derivation`
- `8.1 Public Visibility`
- `12 Normative Requirement Summary`
- `14 Open Questions`
- Appendix A

MANDATORY pre-read in `docs/Z00Z-JMT-Design.md`:

- section `8.1 Public Visibility`
- section `14 Open Questions`
- Appendix A glossary entries for `leaf_count` and bucket policy fields

- [x] Capture `HJMT-Bucket-Occupancy-Metadata-Privacy` as a future phase
  candidate or review gate.
- [x] Keep proof-visible `leaf_count`, `bucket_occupancy`, and equivalent
  counters out of Phase 052 proof metadata.
- [x] Allow local operational metrics only when they remain diagnostic and
  non-authoritative.
- [x] Require design update, proof-version bump, privacy review, and
  fail-closed tests before any occupancy counter becomes verifier-visible.
- [x] Review exact counts, ranges, thresholds, sparse-bucket hints, policy
  generation changes, and cross-proof correlation risk.

Files:

- `.planning/phases/052-HJMT-Backend/052-09-PLAN.md`
- future phase artifacts only if the active GSD workflow explicitly promotes
  the candidate.

Unit tests:

- [x] future guardrails must fail if proof-visible occupancy fields appear
  before the design source authorizes them.

E2E or integration tests:

- [x] future proof tests must reject tampered counter metadata, wrong policy
  generation, wrong root binding, reload drift, and downstream authority use.

Exit condition:

- Bucket occupancy counters are treated as privacy-sensitive future metadata,
  not as default Phase 052 proof fields.

### 052-16 Generalized Settlement-Root Migration Candidate

Spec references:

- `1.2 Live Asset Terms Versus Generalized Rights Terms`
- `5.2 Root Taxonomy`
- `8.3 Checkpoint Evidence`
- `12 Normative Requirement Summary`
- `14 Open Questions`

MANDATORY pre-read in `docs/Z00Z-JMT-Design.md`:

- section `1.2 Live Asset Terms Versus Generalized Rights Terms`
- section `5.2 Root Taxonomy`
- section `14 Open Questions`

- [x] Capture `Generalized-Rights-Root-Model` as a future protocol migration
  candidate.
- [x] Keep `AssetStateRoot` as the live Phase 052 oracle and public asset root.
- [x] Require root-generation metadata, compatibility adapters, checkpoint
  statement migration, proof-envelope versioning, old-root and new-root
  coexistence rules, downgrade rejection, and rollback rules.
- [x] Define a new oracle for the generalized root migration rather than
  reusing backend equivalence as proof of root-vocabulary correctness.
- [x] Define wallet, validator, simulator, checkpoint, reload, and
  `scenario_1` migration impact.

Files:

- `.planning/phases/052-HJMT-Backend/052-10-PLAN.md`
- future phase artifacts only if the active GSD workflow explicitly promotes
  the candidate.

Unit tests:

- [x] future tests must cover old root, new root, mixed generation, downgrade,
  and wrong-generation rejection with state preservation.

E2E or integration tests:

- [x] future checkpoint and simulator tests must prove semantic root migration
  without allowing downstream root-vocabulary authority.

Exit condition:

- `SettlementStateRoot` is planned as a separate protocol migration and does
  not replace `AssetStateRoot` inside Phase 052.

### 052-17 RightLeaf And FeeEnvelope Protocol Candidate

Spec references:

- `1.2 Live Asset Terms Versus Generalized Rights Terms`
- `2.4 RightLeaf As The Generalized Terminal Object`
- `2.4.1 Legal-Defensibility Value Of RightLeaf`
- `5.1 Asset Path Proof Shape`
- `11 Relationship To Z00Z Use Cases`
- `12 Normative Requirement Summary`
- `14 Open Questions`

MANDATORY pre-read in `docs/Z00Z-JMT-Design.md`:

- section `2.4 RightLeaf As The Generalized Terminal Object`
- section `2.4.1 Legal-Defensibility Value Of RightLeaf`
- section `14 Open Questions`

- [x] Capture `RightLeaf-FeeEnvelope-Protocol` as a future protocol candidate.
- [x] Keep `RightLeaf` out of Phase 052 live exports until generalized-rights
  runtime semantics exist.
- [x] Define future `RightLeaf` scope: terminal family marker, bounded right
  type, issuer or provider scope, holder or capability binding, expiry,
  one-time use, revocation or transition semantics, selective disclosure, and
  checkpoint interaction.
- [x] Keep `FeeEnvelope` separate from terminal right semantics.
- [x] Define future `FeeEnvelope` scope: payer or sponsor binding, processing
  guarantee, verification or relay budget, expiry, replay protection, fee
  credit or reserve interaction, and failure modes.
- [x] Define tests proving invalid fees reject before right transitions and
  fee support does not prove right ownership.

Files:

- `.planning/phases/052-HJMT-Backend/052-11-PLAN.md`
- future phase artifacts only if the active GSD workflow explicitly promotes
  the candidate.

Unit tests:

- [x] future tests must cover `RightLeaf` schema validation, proof-family
  marker, transition rules, fee tamper, fee replay, expired support,
  insufficient support, wrong sponsor, wrong right binding, and reject-state
  preservation.

E2E or integration tests:

- [x] future wallet, validator, checkpoint, and simulator tests must prove
  rights semantics and fee support remain separate contract families.

Exit condition:

- `RightLeaf` and `FeeEnvelope` are first-class future protocol work, and
  Phase 052 remains a fixed-bucket HJMT backend implementation.

## 🧪 Required Test Matrix

The Phase 052 test matrix must reuse and extend the Phase 051 corpus:

- insert-many;
- delete-many;
- hot-serial;
- cross-definition;
- duplicate path;
- delete-missing;
- reorder-stable roots;
- no-op root;
- proof verification success and fail;
- reload-after-crash;
- checkpoint seal and reload;
- path-index rebuild;
- unsupported or malformed proof envelope rejection;
- wrong semantic root rejection;
- wrong path rejection;
- wrong definition leaf rejection;
- wrong serial leaf rejection;
- wrong terminal leaf rejection;
- wrong terminal leaf hash rejection;
- wrong root bind rejection;
- wrong backend root rejection;
- wrong branch proof rejection;
- wrong checkpoint context rejection;
- wrong bucket metadata rejection;
- wrong deletion proof rejection;
- wrong non-existence proof rejection;
- present-key non-existence rejection;
- non-existence replay consistency;
- multithread async `multi-insert` benchmark evidence;
- multithread async `multi-delete` benchmark evidence;
- inclusion proof-size measurement and regression review;
- non-existence proof-size unsupported-status review, with live measurement and
  aggregation-throughput review deferred until absent-key proofs are live;
- state preservation after every rejecting workload;
- green-state audit for plans `052-01` through `052-06`;
- source-shape guardrails proving adaptive buckets, proof-visible occupancy
  counters, `SettlementStateRoot`, `RightLeaf`, and `FeeEnvelope` are not live
  Phase 052 exports unless a later promoted phase changes the design source.

## 🎬 Scenario 1 Simulation Matrix

Simulator `scenario_1` must remain a consumer of storage semantics, not a
physical layout authority.

| Scenario surface | Required Phase 052 validation | Task coverage |
| --- | --- | --- |
| Stage 4 pre-transaction storage view | pre-transaction root, prep snapshot, `chk_blob_item`, and storage view export still use storage-owned proof APIs | `052-10`; `052-12` |
| Stage 6 checkpoint bundle handoff | bundle and exec input paths still bind `CheckRoot` and storage-owned checkpoint data without backend roots | `052-10`; `052-12` |
| Stage 7 or post-apply storage transition | storage-backed apply still seals the expected semantic state root | `052-10`; `052-12` |
| Stage 11 wallet scan | `proof_blob` plus `chk_blob` still runs before ownership detection and does not decode forest layout in simulator code | `052-06`; `052-10`; `052-12` |
| Stage 12 checkpoint finalization | finalized checkpoint artifacts continue to bind semantic roots | `052-10`; `052-12` |
| Stage 13 storage replay | pre and post replay stores, storage contract, root binding report, and tamper checks remain semantic-root based | `052-08`; `052-10`; `052-12` |
| `runner_verify` | storage contract validation catches drift in report roots, storage paths, and replay outputs | `052-10`; `052-12` |

Required simulator runs after forest mode exists:

- compatibility mode:
  `cargo run --release -p z00z_simulator --bin scenario_1 --features
  wallet_debug_dump`;
- forest mode: same scenario command with the Phase 052 forest backend mode
  selected;
- dual-verify mode: same scenario command with the Phase 052 dual-verify mode
  selected;
- simulator test target:
  `cargo test -p z00z_simulator --release --features wallet_debug_dump
  scenario_1`.

## ✅ Completion Gate

This phase is complete only when all of the following hold:

- the forest backend is the only new physical backend path and is hidden behind
  the Phase 051 facade;
- compatibility and forest backends produce identical semantic outcomes for
  the golden corpus;
- root vocabulary, proof ownership, checkpoint contracts, and downstream
  authority guardrails remain unchanged unless the design source records a
  repository-backed protocol migration;
- fixed bucket derivation and verifier-visible bucket metadata are stable,
  versioned, and covered by reject tests;
- child-before-parent publication is durable through the forest commit journal;
- reload, checkpoint, path-index rebuild, and crash-recovery corpora pass for
  compatibility and forest backends;
- inclusion, deletion, and non-existence proof families either verify
  fail-closed or remain explicitly unsupported;
- benchmark evidence includes multithread async `multi-insert` and
  `multi-delete` workloads plus measured inclusion and non-existence proof
  sizes;
- no downstream crate uses physical layout as authority;
- simulator `scenario_1` passes through the storage facade in compatibility,
  forest, and dual-verify modes once those modes exist;
- mandatory verification order has been run and recorded;
- plans `052-01` through `052-06` have a green-state audit before any future
  protocol candidate is promoted;
- adaptive buckets, bucket occupancy metadata, generalized settlement roots,
  `RightLeaf`, and `FeeEnvelope` are captured as first-class follow-ups without
  becoming live Phase 052 exports.
