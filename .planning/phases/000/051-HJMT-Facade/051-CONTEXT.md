---
phase: 051-HJMT-Facade
artifact: planning-context
status: ready-for-execution-planning
updated: 2026-05-28
source: 051-TODO.md plus repository-backed JMT design and state-management references
---

# 051 HJMT Facade Context

## Source Inputs

This phase is planned from these repository-backed sources:

- `.planning/phases/051-HJMT-Facade/051-TODO.md`
- `docs/Z00Z-JMT-Design.md`
- `.planning/phases/055-State Management-1/045-NEW-State-Management-Spec.md`
- `.planning/phases/055-State Management-1/045-TODO.md`
- Live storage, checkpoint, wallet, and simulator code under `crates/`

Reference drift note: `051-TODO.md` points at historical `050-state-mgmt`
planning references, but the current worktree does not contain a
`.planning/phases/050-state-mgmt/` directory. The available maintained copy of
that state-management material is under `.planning/phases/055-State
Management-1/`. Execution must use the current maintained copy and must not
restore or recreate the missing historical path.

## Phase Boundary

Phase 051 establishes the HJMT migration facade and compatibility reference
backend. It does not implement the production bucketed forest backend. Later
HJMT phases may add fixed bucket policy, physical forest commits, commit
journals, recovery, and dual-backend rollout behind the boundary created here.

The phase is complete when downstream crates consume storage through one
semantic facade, the current shared namespaced JMT is behind that facade as the
compatibility backend, root and proof semantics are fail-closed, and a golden
corpus proves the compatibility backend remains the semantic reference.

## Scope Reconciliation

`051-TODO.md` names both the immediate `0.1 Boundary And Compatibility Facade`
gate and the broader forest rollout sequence from `docs/Z00Z-JMT-Design.md`
section 9.1. This planning packet treats Phase 051 as the facade phase named by
the existing `051-HJMT-Facade` folder: `051-01` through `051-04` implement the
executable boundary, compatibility, proof, guardrail, and corpus work, while
`051-05` records the fixed-bucket, physical forest, journal, recovery,
dual-backend, and configuration-switch handoff. If execution scope expands to
ship forest rollout phases in this same numbered phase, this context must be
updated first and additional numbered plans must be added before code changes.
That explicit update is required to avoid concept drift or a parallel storage
authority layer.

## Locked Decisions

### Public Roots

- `AssetStateRoot` remains the only live public asset-state root for this
  asset-centric generation.
- `CheckRoot` remains a checkpoint-facing evidence type derived from
  `AssetStateRoot` where checkpoint APIs need a state-transition root.
- `SettlementStateRoot` remains future terminology only. Phase 051 must not
  export it, simulate it as live, or let tests imply that the current runtime is
  already a generalized settlement-root generation.
- `RightLeaf` remains future generalized-rights terminology only. Phase 051
  must not add a live `RightLeaf` runtime type, must not widen `AssetLeaf` into
  a fake generalized-rights object, and must keep any future `RightLeaf` /
  `FeeEnvelope` relationship documented as distinct future contract families.
- Physical backend roots, including current `ProofBlob::backend_root()`, remain
  proof-local or diagnostic data. They must never become authority roots.

### Backend Boundary

- The current `AssetStore` public behavior must remain source-compatible for
  callers while its implementation moves behind one storage-owned backend
  trait or equivalent facade.
- The current shared namespaced JMT implementation is the compatibility backend
  and semantic reference, not the target performance architecture.
- The facade must reuse the existing `AssetModel`, `TreeStore`, `RedbBackend`,
  `ProofBlob`, checkpoint modules, and path-index machinery where those are the
  current source of truth. Phase 051 must not create a duplicate semantic
  model, duplicate checkpoint verifier, duplicate proof decoder, duplicate
  path-index authority, or dummy forest backend.
- Callers must continue to address state by `AssetPath`, `StoreItem`,
  `StoreOp`, and storage-owned proof types. They must not provide bucket ids,
  raw `TreeId` values, namespace prefixes, branch ordering, or physical key
  layout.
- Future forest commit journal, fixed bucket policy, physical forest backend,
  crash-safe recovery, and configuration-gated rollout work must enter through
  this facade and compatibility corpus. They are reflected in Phase 051 as
  explicit handoff constraints unless a later plan updates this context before
  execution.

### Proof Envelope

- The compatibility proof envelope must be explicitly versioned and
  storage-owned.
- Verification must bind semantic root, path context, parent-root leaves,
  terminal leaf, leaf hash, proof bytes, and the semantic/backend-root binding.
- Unsupported envelope versions, malformed bytes, wrong semantic roots, wrong
  path context, wrong checkpoint binding, wrong bucket-policy metadata, wrong
  branch proofs, and detached payloads must reject fail-closed.
- In compatibility mode, bucket-policy metadata is either absent or explicitly
  marked as not applicable; unexpected bucket metadata is a reject condition.
- Compatibility mode must not fake deletion or non-existence proof support. If
  those proof families are not implemented for the current compatibility
  backend, their envelope forms must be explicit unsupported-version or
  unsupported-proof-family rejects, and future forest work must add the real
  semantics behind the same verifier boundary.

### Checkpoint And Reload Authority

- Checkpoint proof authority stays inside `z00z_storage::checkpoint`.
- Reload validation stays storage-owned and fail-closed before persisted
  checkpoint metadata is accepted.
- Validator, wallet, and simulator code must consume storage-owned checkpoint
  and proof contracts instead of inventing a second proof verifier or artifact
  schema.

## Current Baseline

The live code already has most semantic nouns needed for the facade:

- `crates/z00z_storage/src/assets/types_identity.rs` defines
  `DefinitionId`, `SerialId`, `AssetId`, `AssetPath`, `AssetStateRoot`,
  `ClaimSourceRoot`, `CheckRoot`, and `TxDigest`.
- `crates/z00z_storage/src/assets/types_record.rs` defines `RootApi`,
  `CompatRoot`, `DefinitionRootLeaf`, `SerialRootLeaf`, `StoreItem`,
  `SnapItem`, and `ProofItem`.
- `crates/z00z_storage/src/assets/model.rs` is the deterministic semantic
  reference model and computes `AssetStateRoot` from definition root leaves.
- `crates/z00z_storage/src/assets/store.rs` owns `AssetStore`, `RedbBackend`,
  a shared `MemTreeStore`, the semantic model, `TreeRoots`, path index state,
  and version history.
- `crates/z00z_storage/src/assets/store_internal/tree_store.rs` namespaces
  logical trees into one physical `Sha256Jmt` and commits through one
  `put_value_set` plus one `apply_batch` boundary.
- `crates/z00z_storage/src/assets/store_internal/tree_id.rs` already keeps
  `TreeId`, `TreeRootRef`, and `PathIndexRec` crate-private.
- `crates/z00z_storage/src/assets/proof.rs` defines the storage-owned
  `ProofBlob`, `ProofScanOut`, `chk_item`, `chk_blob`, `backend_root`, and
  `root_bind` behavior.
- `crates/z00z_storage/src/checkpoint/build.rs`,
  `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs`, and
  `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs`
  are the checkpoint-facing root and reload authority surfaces.

## Requirements

| Requirement | Meaning | Planned In |
| --- | --- | --- |
| `PH51-BACKEND-FACADE` | Introduce one storage-owned semantic backend trait or equivalent facade covering root queries, item lookup, batch put/delete, proof queries, checkpoint-facing reads, reload validation hooks, and storage-owned wallet or simulator state. | `051-01` |
| `PH51-COMPAT-BACKEND` | Move the current shared namespaced JMT implementation behind the facade as the compatibility backend and reference oracle. | `051-01`, `051-04` |
| `PH51-ROOT-TAXONOMY` | Codify `AssetStateRoot`, `CheckRoot`, absent `SettlementStateRoot`, and proof-local `backend_root` semantics in types, docs, and tests. | `051-02`, `051-03` |
| `PH51-PROOF-ENVELOPE` | Define and enforce the compatibility proof-envelope version boundary and fail-closed reject classes. | `051-02`, `051-04` |
| `PH51-GUARDRAILS` | Prevent downstream crates from depending on raw `backend_root`, raw `TreeId`, namespace prefixes, branch ordering, physical key layout, or duplicate checkpoint proof formulas as authority. | `051-03` |
| `PH51-EQUIVALENCE` | Add golden compatibility corpus for insert, delete, replay, proof verification, checkpoint seal/reload, and path-index rebuild semantics. | `051-04` |
| `PH51-CHECKPOINT-RELOAD` | Keep checkpoint-facing reads and reload validation on storage-owned authority and verify them through the facade. | `051-01`, `051-04` |
| `PH51-ROLLOUT-HANDOFF` | Record the forest-backend handoff boundary without shipping the forest backend in Phase 051. | `051-05` |

## Source Coverage Matrix

Every source bullet from `051-TODO.md` is covered as follows:

| Source item | Context coverage | Plan coverage |
| --- | --- | --- |
| Establish one storage migration gate before more correctness work binds to current shared-backend internals. | `Phase Boundary`, `Backend Boundary` | `051-01`, `051-03`, `051-04` |
| Make the three ordered steps explicit: boundary first, authority slices second, forest backend as early infrastructure lane. | `Backend Boundary`, `Non-Goals`, `Reference Requirement Coverage` | `051-01` through `051-04` for boundary/authority readiness; `051-05` for forest handoff |
| Current semantic storage vocabulary is mostly correct. | `Current Baseline`, `Public Roots` | `051-01`, `051-02` |
| Shared physical commit boundary is the bottleneck. | `Current Baseline`, `Backend Boundary` | `051-01`, `051-05` |
| Migration must be staged behind a compatibility backend. | `Backend Boundary`, `Requirements` | `051-01`, `051-04` |
| Stable public contract keeps `AssetPath`, `AssetLeaf`, and `AssetStateRoot`. | `Public Roots`, `Backend Boundary` | `051-01`, `051-02`, `051-03` |
| Do not promote `SettlementStateRoot` or expose `backend_root` as authority. | `Public Roots`, `Non-Goals` | `051-02`, `051-03`, `051-05` |
| Storage owns semantic-root to physical-proof binding, envelope versioning, and fail-closed verification. | `Proof Envelope` | `051-02`, `051-04` |
| Downstream crates consume typed proof results, not backend layout details. | `Backend Boundary`, `Checkpoint And Reload Authority` | `051-03`, `051-04` |
| One backend trait, forest commit journal, crash-safe publication rules, stable semantic API. | `Backend Boundary`, `Non-Goals` | `051-01` for stable API; `051-05` for journal/recovery handoff |
| Ordered rollout: boundary and compatibility backend first. | `Execution Plan Map` | `051-01` |
| Ordered rollout: fixed bucket policy and forest backend second. | `Backend Boundary`, `Non-Goals` | `051-05` handoff only; not executable in Phase 051 unless this context is updated |
| Ordered rollout: dual-backend equivalence and configuration switch last. | `Backend Boundary`, `Success Criteria` | `051-04` compatibility harness; `051-05` configuration switch handoff |
| Use state-management decisions only to place downstream authority slices after facade stability. | `Checkpoint And Reload Authority`, `State Management Reference Coverage` | `051-03`, `051-04`, `051-05` |
| Locality gate: local implementation and verification work only. | `Phase Boundary`, `Non-Goals` | All plans; no live network/operator setup |
| In-scope trait design and compatibility wrapper. | `Requirements` | `051-01` |
| In-scope semantic root taxonomy and proof-envelope contract. | `Requirements`, `Proof Envelope` | `051-02` |
| In-scope root-binding rules and API/type guardrails. | `Public Roots`, `Backend Boundary` | `051-02`, `051-03` |
| In-scope dual-backend equivalence tests and authority slices consuming stable facade. | `Success Criteria` | `051-03`, `051-04`; no dummy forest backend |
| Broader implementation-boundary forest items from the TODO: fixed bucket policy types, forest backend, commit journal, crash recovery, and config enablement. | `Scope Reconciliation`, `Backend Boundary`, `Reference Requirement Coverage`, `Non-Goals` | `051-05` explicit future handoff; not executable in this facade packet unless this context is updated first |
| Out-of-scope live rollout, benchmark claims, generalized root export, public protocol root changes, wallet logic moved into storage, simulator business rules, and backend leakage. | `Non-Goals` | All plans, especially `051-03` and `051-05` |
| Gate 0.1 boundary and compatibility facade. | `Execution Plan Map` | `051-01` through `051-04` |
| Six implementation tasks under 0.1. | `Requirements` | `051-01` task 1-2; `051-02` task 3-4; `051-03` task 5; `051-04` task 6 |
| Trait-compatibility tests across compatibility and forest backends. | `Success Criteria` | `051-04` harness with compatibility case and future forest insertion point; no fake forest backend |
| Root-taxonomy tests. | `Public Roots` | `051-02`, `051-03` |
| Proof-envelope tests. | `Proof Envelope` | `051-02`, `051-04` |
| Golden-equivalence tests. | `Success Criteria` | `051-04` |
| Done when downstream crates depend on one semantic facade and compatibility backend remains green reference. | `Phase Boundary`, `Success Criteria` | `051-01` through `051-04`; `051-05` closeout evidence |

## Reference Requirement Coverage

| JMT requirement | Phase 051 treatment |
| --- | --- |
| `JMT-REQ-001` | `AssetStateRoot` is live public root; `SettlementStateRoot` is future-only. Covered by `051-02`, `051-03`, `051-05`. |
| `JMT-REQ-002` | `AssetPath { definition_id, serial_id, asset_id }` remains the canonical path contract. Covered by `051-01`, `051-04`. |
| `JMT-REQ-003` | Fixed buckets are future forest work. Phase 051 records deterministic bucket policy as a verifier-visible handoff and rejects unexpected bucket metadata in compatibility mode. Covered by `051-02`, `051-05`. |
| `JMT-REQ-004` | Bucket identifiers stay internal unless proof verification needs recomputation. Covered by `051-03`, `051-05`. |
| `JMT-REQ-005` | Batch insert/delete remains semantic facade behavior in Phase 051; parallel physical bucket JMTs are future work behind the facade. Covered by `051-01`, `051-05`. |
| `JMT-REQ-006` | Child-root durability before parent publication is future forest commit behavior. Phase 051 records it as a handoff invariant. Covered by `051-05`. |
| `JMT-REQ-007` | Forest commit journal is future runtime work. Phase 051 must not create a fake journal; it records the journal/recovery contract for later implementation. Covered by `051-05`. |
| `JMT-REQ-008` | Inclusion proof envelopes are compatibility-mode work; deletion and non-existence proof families must either reject as unsupported or be implemented explicitly later. Covered by `051-02`, `051-04`, `051-05`. |
| `JMT-REQ-009` | Path index is rebuildable internal lookup state, not public root truth. Covered by `051-01`, `051-04`. |
| `JMT-REQ-010` | Compatibility backend remains semantic reference. Covered by `051-01`, `051-04`. |
| `JMT-REQ-011` | Fail-closed mismatch handling is required for proof envelope and future bucket/journal mismatches. Covered by `051-02`, `051-04`, `051-05`. |
| `JMT-REQ-012` | Backend roots must not substitute for `AssetStateRoot`. Covered by `051-02`, `051-03`. |
| `JMT-REQ-013` | `RightLeaf` is target-only until live runtime widens. Covered by `Public Roots`, `051-02`, `051-05`. |
| `JMT-REQ-014` | Future `RightLeaf` and fee support remain distinct contract families. Covered by `Public Roots`, `051-05`. |

## State Management Reference Coverage

`051-TODO.md` references historical `050-state-mgmt` anchors for execution
baseline decisions and required execution order. The maintained repository copy
is `.planning/phases/055-State Management-1/`. Phase 051 uses those references
only as downstream authority-order guardrails, not as a source for HJMT backend
topology, bucket policy, or proof-envelope design.

| State-management source item | Phase 051 treatment |
| --- | --- |
| Keep one truthful authority path for claim-source, checkpoint, scan, and nullifier work. | Reuse storage-owned claim-source, checkpoint, scan, and nullifier seams; do not add parallel carriers, verifier formulas, cursor models, replay registries, import pipelines, or persistence paths. Covered by `051-01`, `051-03`, `051-05`. |
| Keep `ClaimSourceProof`, `CheckpointStore::seal_artifact(...)`, `ScanChunk`, `ScanStatePayload`, `recv_range(...)`, `recv_claim_asset(...)`, and existing verdict or reject vocabulary as live authority surfaces unless the spec is updated first. | Treat these as existing downstream authority boundaries. Phase 051 guardrails must extend or consume them instead of inventing replacements. Covered by `051-03`, `051-05`. |
| Do not create a second checkpoint verifier outside `z00z_storage::checkpoint`. | Validator, wallet, and simulator code must consume storage-owned checkpoint and proof contracts. Covered by `051-03`, `051-04`, `051-05`. |
| Required execution order puts checkpoint/storage authority before validator flow and simulator closure last. | Phase 051 keeps facade and compatibility work before downstream cutover, and keeps simulator/closeout evidence after storage guardrails. Covered by `051-01` through `051-05`. |
| Validator checkpoint flow consumes storage-owned checkpoint authority and existing verdict or reject vocabulary. | `051-03` must include validator source-shape guardrails so validators do not freeze a second artifact schema or proof formula. |
| Wallet receive and scan stay on `ScanChunk` plus `ScanStatePayload`, not a new scan cursor model or direct runtime DTO persistence lane. | `051-03` and `051-05` record this as an anti-drift guardrail; Phase 051 must not redesign wallet receive persistence. |
| Nullifier reserved-to-spent transition keeps storage authoritative for final spent state. | Phase 051 treats nullifier state as downstream authority ordering context only; it must not introduce a second replay registry or generic spend-replay surface. Covered by `051-05` handoff if touched. |
| State-management canonical commands remain phase-local to that packet. | Phase 051 uses the user-mandated `bootstrap_tests.sh`, broad cargo command, and repeated `/GSD-Review-Tasks-Execution` gates; state-management commands may be referenced only when a touched downstream file makes them relevant. |

## JMT Test Corpus Coverage

The JMT design also defines test-strategy material beyond the normative
requirements. Phase 051 routes that material as follows:

| JMT test section | Phase 051 treatment |
| --- | --- |
| `13.1 Equivalence Tests` | `051-04` builds the compatibility golden corpus with a compatibility backend case and a future real forest-backend insertion point. It must not introduce a copied compatibility backend or dummy forest backend. |
| `13.2 Crash Tests` | Full forest crash and recovery tests are future forest work because Phase 051 does not ship the physical forest or commit journal. `051-04` covers compatibility reload-after-crash and checkpoint reload behavior; `051-05` records forest crash/recovery handoff. |
| `13.3 Proof Tests` | `051-02` and `051-04` cover compatibility proof-envelope reject classes now, including unsupported deletion/non-existence families if those semantics are not implemented; full target-forest proof families remain behind the same verifier boundary. |

## Execution Plan Map

| Plan | Objective | Requirements |
| --- | --- | --- |
| `051-01-PLAN.md` | Add the storage facade and wrap the current shared JMT implementation as the compatibility backend. | `PH51-BACKEND-FACADE`, `PH51-COMPAT-BACKEND`, `PH51-CHECKPOINT-RELOAD` |
| `051-02-PLAN.md` | Lock root taxonomy and proof-envelope v1 reject semantics. | `PH51-ROOT-TAXONOMY`, `PH51-PROOF-ENVELOPE` |
| `051-03-PLAN.md` | Add public API and downstream guardrails against backend-layout coupling. | `PH51-GUARDRAILS`, `PH51-ROOT-TAXONOMY` |
| `051-04-PLAN.md` | Build the compatibility/golden semantic corpus, including checkpoint and reload cases. | `PH51-EQUIVALENCE`, `PH51-COMPAT-BACKEND`, `PH51-PROOF-ENVELOPE`, `PH51-CHECKPOINT-RELOAD` |
| `051-05-PLAN.md` | Close docs, roadmap/state evidence, and handoff notes for future forest rollout. | `PH51-ROLLOUT-HANDOFF`, all Phase 051 requirements |

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
4. If committing is needed, use `/z00z-git-versioning` and the repository-owned
   `.github/skills/z00z-git-versioning/scripts/version-manager.sh` flow.

## Non-Goals

- Do not implement the production forest backend in Phase 051.
- Do not add adaptive buckets, bucket split or merge logic, or a commit journal
  as live runtime behavior in Phase 051.
- Do not expose `TreeId`, namespace bytes, or backend branch ordering outside
  storage-owned modules.
- Do not claim that `backend_root` is a substitute for `AssetStateRoot`.
- Do not introduce a second checkpoint verifier, checkpoint artifact schema,
  wallet scan cursor model, or storage authority plane.

## Success Criteria

1. Downstream crates can depend on one semantic storage facade rather than the
   current physical layout.
2. The compatibility backend is the current shared namespaced JMT behavior and
   remains the semantic reference backend.
3. `AssetStateRoot`, `CheckRoot`, and `backend_root` have explicit non-
   interchangeable semantics enforced by source-shape and behavior tests.
4. The proof envelope rejects unsupported versions, malformed bytes, wrong
   semantic roots, wrong paths, wrong checkpoint bindings, wrong bucket-policy
   metadata, wrong branch proofs, and detached payloads.
5. Golden compatibility tests cover insert-many, delete-many, hot-serial,
   cross-definition, duplicate path, delete-missing, reload-after-crash,
   checkpoint seal/reload, and path-index rebuild.
6. Future forest backend work has a documented handoff that uses the facade
   without widening Phase 051 scope.
