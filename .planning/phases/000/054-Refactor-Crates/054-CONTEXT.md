# Phase 054: Refactor Crates - Context

**Gathered:** 2026-06-08
**Status:** ready-for-execution-planning
**Source:** Deep analysis of `054-TODO.md` plus the referenced HJMT, storage, and runtime documents

<domain>

## Phase Boundary

Phase 054 realigns `z00z_rollup_node`, `z00z_runtime/*`, and `z00z_storage`
around a storage backend seam, a runtime-owned planner-authority split, and a
delayed rename wave. The phase must preserve the live Phase 053 settlement
contracts, keep `z00z_storage` as the semantic truth layer, keep
`z00z_rollup_node` as the orchestration root, and avoid creating any second
authority plane for routing, proofs, checkpoints, or backend metadata.

The phase must reuse `.planning/phases/054-Refactor-Crates/` as the only Phase
054 directory. `054-TODO.md` remains the canonical backlog and execution-order
seed, while the numbered plan packet converts that backlog into bounded
execution slices.

</domain>

<decisions>

## Implementation Decisions

### D-01: Fixed four-wave architecture
- Preserve the TODO-defined wave ordering as a hard boundary:
  backend seam first, adapter move second, planner-authority split third,
  rename wave fourth. Documentation closeout follows the landed topology last.

### D-02: Proof-surface compatibility gate comes first
- Before structural storage moves, treat
  `crates/z00z_storage/benches/assets_proofs.rs` plus the existing storage test
  suites as the compatibility gate for the public proof and settlement API
  surface.

### D-03: Backend seam stays below the semantic facade
- Introduce `z00z_storage::backend` as a low-level durable seam with
  `StorageBackend`, `JournalBackend`, and txn-oriented contracts, but keep it
  strictly below the existing semantic facade `SettlementTreeBackend`.
- `SettlementStore` must remain the storage-owned semantic and proof facade.

### D-04: Phase 054 backend scope is `redb` plus `memory`
- Phase 054 may introduce `backend/redb/*`, `backend/common/*`, and
  `backend/memory/*`.
- Do not add a `rocksdb` stub in this phase. A backend name without a real
  implementation and tests is forbidden scope.

### D-05: `StoreBackendError` moves, but its live symbol stays stable
- Move backend-specific error definitions out of `crates/z00z_storage/src/error.rs`
  into `backend/error.rs`.
- Keep `StoreBackendError` as the live symbol name until the seam, downstream
  callers, tests, and docs are stable.

### D-06: `SettlementStore` keeps semantic ownership
- `SettlementStore` may stop storing direct `RedbBackend` fields and may pivot
  to the new low-level backend seam, but it must keep ownership of settlement
  semantics, proof issuance, deterministic replay, and the live
  `SettlementTreeBackend` contract.

### D-07: Planner authority moves selectively into runtime
- Runtime-owned planner authority covers canonicalization, route targeting,
  single-shard admission, canonical op digest generation, and `BatchPlanned`
  inputs.
- Do not move all of `tx_plan` wholesale out of storage.

### D-08: Store-side `tx_plan` helpers stay in storage
- Keep storage-local precheck, dry-run, rollback, `SettlementModel`, and
  store-scoped semantic helpers inside `z00z_storage` when they still depend on
  internal settlement truth.

### D-09: Runtime placement metadata is operational, not public truth
- `AggregatorId`, `ShardPlacementTable`, standby metadata, and `ShardExecutor`
  are runtime placement objects.
- Validators and watchers may consume runtime surfaces, but they must not
  become alternate planner authority or verifier-visible truth owners.

### D-10: `z00z_rollup_node` remains the only orchestration root
- `z00z_rollup_node` stays the composition and orchestration root for runtime
  services and DA adapters.
- The phase must not introduce a new super-aggregator layer above it.

### D-11: Rename wave is strictly post-stabilization
- File and module renames happen only after backend seam and planner split are
  stable.
- Do not combine file/module rename with public symbol rename for
  `AggregatorService`, `ValidatorService`, `WatcherService`,
  `SettlementTreeBackend`, or `StoreBackendError` in the same wave.

### D-12: Storage canonical-module cleanup is scoped and semantic-first
- After the seam stabilizes, replace path-attribute bridge wiring in the
  active storage hot spots first. The landed canonical roots are now
  `settlement/store/mod.rs`, `backend/redb/mod.rs`, and the narrower
  `settlement/tx_plan*.rs` helpers that stayed semantic-first inside storage.
- Remove the duplicate `serialization/build_temp_tree.rs` only after verifying
  the canonical helper in `serialization/build/*`.
- Keep checkpoint/snapshot/serialization as distinct surface areas; do not fold
  them into a generic backup layer.

### D-13: Legacy source-shape rules are carry-over guardrails, not blind authority
- Carry over the legacy closeout gates for any source-shape slice that lands in
  this phase: zero `#[path = ...]` outside approved exceptions, zero
  module-body `include!("...")` outside approved exceptions, and full `fmt`,
  `clippy`, `test`, and `doc` closeout.
- Carry over the legacy ordering controls: land guardrails first, move one
  crate family at a time, keep mechanical cleanup separate from behavior moves,
  and reshape broad integration harnesses after production modules.

### D-14: Generated-data exception stays explicit
- Treat
  `crates/z00z_simulator/src/scenario_1/runner_contract.rs` and its
  `include!(concat!(...))` usage as a generated-data exception until a later
  intentional replacement lands. Do not silently normalize it during Phase 054.

### D-15: Current-worktree fallback for the legacy spec
- `legacy-refactor-spec.md` is a historical source, not the live authority.
- If `legacy-refactor-spec.md` is missing in the current worktree, use
  `054-TODO.md` section `18) Verified carry-over from legacy-refactor-spec.md`
  as the extracted live authority and do not recreate the file just to satisfy
  a reference.

### D-16: Mandatory validation order for every Rust or test-affecting auto task
- Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  first as the fail-fast gate.
- If it fails, stop, fix, and rerun it before broader validation.
- Then run `cargo test --release` when the slice changes Rust or tests.
- Then run `/GSD-Review-Tasks-Execution` in YOLO mode at least three times and
  continue until at least two consecutive runs report no significant issues.

### D-17: Git versioning discipline
- If a plan slice needs a commit, use `/z00z-git-versioning`.
- Do not create ad-hoc git/version flows outside the repository versioning
  workflow.

### D-18: No duplicate or parallel implementation layer
- Extend or replace existing seams in place.
- Do not park old runtime lanes under new names, do not create a parallel
  backend authority, and do not preserve stale doc language that treats
  target-state vocabulary as already-live exports.

### D-19: Keep rows are validation anchors, not ignorable prose
- Every TODO row marked `keep`, every placeholder cleanup row, every
  bench/fuzz/script/test row, and every manifest row remains mandatory
  planning authority even when the current slice does not expect a code edit
  in that file.
- If a row is not a real edit target for the current slice, the slice must
  still treat it as a validation anchor, an explicit non-goal, or a deferred
  follow-up target. No TODO row may be silently dropped as “informational”.

### D-20: `z00z_runtime` is a crate family, not one monolith
- Phase 054 treats `crates/z00z_runtime/aggregators`,
  `crates/z00z_runtime/validators`, and `crates/z00z_runtime/watchers` as
  three separate crates under one namespace folder.
- Do not plan or describe the migration as if one monolithic `z00z_runtime`
  crate exists today.

### D-21: Design vocabulary is live scope and must resolve to one canonical path
- Paper target names from `Z00Z-HJMT-Upgrade.md` such as `BatchPlanned`,
  `AggregatorId`, `ShardPlacementTable`, `ShardExecutor`, `StorageBackend`,
  and `JournalBackend` are live Phase 054 requirement vocabulary. Treat the
  referenced design and whitepaper corpus as authority for current scope, not
  as future-only prose.
- Migration text must still distinguish landed code from required scope while
  the move is in progress: every design-owned term must either exist on one
  canonical live path or be explicitly mapped to the live symbols that satisfy
  it, such as `AggregatorService`, `ValidatorService`, `WatcherService`,
  `WatcherBoundary`, `ValidatorBoundary`, `IngressBoundary`,
  `OrderingBoundary`, `RecoveryBoundary`, `SchedulerBoundary`,
  `SettlementTreeBackend`, and `StoreBackendError`.

### D-22: Wave-mixing prohibitions are execution rules, not advice
- Do not mix backend seam extraction with the rename wave.
- Do not mix file or module rename with public symbol rename for runtime
  facades or the storage semantic facade.
- Do not mix planner-authority relocation with validator or watcher naming
  cleanup.
- Do not rewire `SettlementStore` to the new backend while breaking the public
  proof surface in the same slice.
- Do not add a `rocksdb` stub in the seam-introduction wave.
- Do not move all of `tx_plan` out of storage as one move-only change.

### D-23: Legacy follow-up extraction must use verified live baselines
- Do not copy old inventory counts from the legacy spec as current authority;
  use the verified baseline attached to `054-TODO.md` section 18.
- Do not copy exact legacy paths that no longer exist in the live tree, such
  as `crates/z00z_storage/src/assets/store.rs`.
- Do not promote the old “fewer than 3 Rust files per directory” heuristic
  into a repository-wide hard gate. It remains a local refactor heuristic only.

### D-24: Phase 054 is explicitly guarding three drift classes
- Guard semantic drift: `z00z_storage` must stay the owner of settlement
  semantics, proof surfaces, and deterministic replay rules.
- Guard API drift: benches and tests remain the proof/public API compatibility
  oracle while structural moves land.
- Guard review drift: structural moves and naming cleanup must stay in separate
  waves so review evidence remains attributable.

### the agent's Discretion
- Exact helper names, the exact split between `backend/common/*` and
  `backend/memory/*`, targeted regression additions, and the precise shape of
  runtime internal types are implementation choices as long as they preserve
  the decisions above, the Design Foundation, and the Phase 053 live
  settlement contract.

</decisions>

<canonical_refs>

## Canonical References

Downstream executors MUST read these before implementing the relevant slices.

### Phase authority
- `.planning/phases/054-Refactor-Crates/054-TODO.md` - canonical backlog,
  migration rules, target topology tables, hard execution order, and carry-over
  source-shape guidance.
- `.planning/phases/054-Refactor-Crates/054-CONTEXT.md` - locked Phase 054
  boundaries, decisions, and coverage contract.
- `.planning/phases/054-Refactor-Crates/054-SOURCE-AUDIT.md` - plan-to-source
  coverage map for this packet.
- `.planning/ROADMAP.md` - active Phase 054 goal, plan list, and roadmap state.
- `.planning/STATE.md` - active planning status and prior decisions.

### Predecessor phase and design authority
- `.planning/phases/000/053-HJMT-Backend/053-CONTEXT.md` - live settlement
  contract, cache/scheduler/journal/reload, and downstream authority boundaries
  shipped by Phase 053.
- `.planning/phases/000/053-HJMT-Backend/053-SUMMARY.md` - closeout evidence
  that Phase 053 generalized settlement is already live and must not be
  downgraded by refactor work.
- `docs/tech-papers/Z00Z-HJMT-Upgrade.md` - normative upgrade rules for runtime
  placement, planner authority, shard routing, `StorageBackend`,
  `JournalBackend`, and orchestration/storage boundaries.
- `docs/tech-papers/Z00Z-HJMT-Key-Terms.md` - cross-phase naming authority for
  semantic terms, exact `V1` names, runtime-only placement names, storage seam
  names, and archived compatibility aliases.
- `docs/tech-papers/Z00Z-HJMT-Design.md` - live settlement root, path, proof,
  and ownership semantics inherited from the Phase 053 backend.

### Storage anchors
- `crates/z00z_storage/Cargo.toml` - manifest boundary for backend feature
  gating and the explicit no-`rocksdb`-stub rule.
- `crates/z00z_storage/README.md` - top-level storage contract summary.
- `crates/z00z_storage/src/lib.rs` - current exported storage surface.
- `crates/z00z_storage/src/error.rs` - current checkpoint/serialization/backend
  error split, including `StoreBackendError`.
- `crates/z00z_storage/src/settlement/README.md` - canonical settlement storage
  contract and proof-surface notes.
- `crates/z00z_storage/src/settlement/root_types.md` - semantic root taxonomy
  and live cutover notes.
- `crates/z00z_storage/src/settlement/mod.rs` - live settlement export surface.
- `crates/z00z_storage/src/settlement/store/mod.rs` - current semantic facade
  with canonical in-crate submodule paths and storage-owned proof/state wiring.
- `crates/z00z_storage/src/backend/redb/mod.rs` - current durable RedB backend
  entrypoint and nested bridge wiring.
- `crates/z00z_storage/src/settlement/tx_plan.rs` - current planner/store-side
  transaction planning bridge.
- `crates/z00z_storage/src/serialization/build.rs` and
  `crates/z00z_storage/src/serialization/build/temp_tree.rs` - current
  canonical serialization build helper path.
- `crates/z00z_storage/benches/assets_proofs.rs` - proof/public API
  compatibility guard.
- `crates/z00z_storage/benches/assets_benches.md` - current benchmark
  evidence surface that must continue to describe the landed topology honestly.
- `crates/z00z_storage/tests` - integration test matrix and source-shape
  closeout anchors referenced by `054-TODO.md`.
- `crates/z00z_storage/fuzz` - fuzz-target and seed anchors that must remain in
  sync with any structural storage move.
- Live path drift note: some TODO sections use older or target-shape path
  examples. Phase 054 execution and closeout must bind to the live equivalents
  already present in the repository instead of recreating parallel folders or
  harnesses:
  TODO sections 10-15 use nested target-shape examples in several rows. The
  live repository currently uses flat equivalents:
  `crates/z00z_storage/src/settlement/store/hjmt_*.rs` for the store-private
  HJMT internals;
  `crates/z00z_storage/src/backend/redb/*.rs` for the durable backend;
  `crates/z00z_storage/src/backend/common/*.rs` and `backend/memory.rs` for
  backend-neutral helpers;
  `crates/z00z_storage/src/checkpoint/build/build_prepare.rs`,
  `checkpoint/build/build_state.rs`, `checkpoint/store/store_fs.rs`, and
  `checkpoint/{artifact,store}/tests.rs` for the canonical nested checkpoint
  examples;
  `crates/z00z_storage/src/snapshot/store/tests.rs` for the canonical
  `snapshot/store/tests.rs` example; and
  `crates/z00z_storage/src/serialization/build/temp_tree.rs` as the live
  canonical temp-tree helper.
  TODO sections 16-17 also use older bench, fuzz, and harness names. Bind them
  to the current live equivalents:
  `crates/z00z_storage/benches/assets_benches.md`,
  `crates/z00z_storage/fuzz/fuzz_targets/settlement_proofs.rs`,
  `crates/z00z_storage/fuzz/seeds/settlement_proofs/*`,
  `crates/z00z_storage/tests/test_snapshot_suite.rs` plus
  `crates/z00z_storage/tests/snapshot_suite/*`, and
  `crates/z00z_storage/src/test_support/settlement_corpus.rs` with
  `crates/z00z_storage/tests/fixtures/test_settlement_corpus_fixture.json`.

### Runtime and node anchors
- `crates/z00z_runtime/aggregators/Cargo.toml` - runtime manifest anchor for
  the aggregators crate during planner and placement moves.
- `crates/z00z_runtime/aggregators/README.md` - runtime aggregation doc anchor.
- `crates/z00z_runtime/aggregators/src/lib.rs` - current public runtime
  aggregation exports and stable re-exports.
- `crates/z00z_runtime/validators/Cargo.toml` - validator manifest anchor.
- `crates/z00z_runtime/validators/README.md` - validator doc anchor.
- `crates/z00z_runtime/validators/src/lib.rs` - current validator public
  boundary exports.
- `crates/z00z_runtime/watchers/Cargo.toml` - watcher manifest anchor.
- `crates/z00z_runtime/watchers/README.md` - watcher doc anchor.
- `crates/z00z_runtime/watchers/src/lib.rs` - current watcher public boundary
  exports.
- `crates/z00z_rollup_node/Cargo.toml` - node manifest anchor.
- `crates/z00z_rollup_node/src/lib.rs` - current rollup-node orchestration and
  settlement theorem surface.
- `crates/z00z_rollup_node/README.md` - current node documentation placeholder
  that must only be updated after topology stabilizes.
- `crates/z00z_rollup_node/tests/test_settlement_theorem.rs` - node theorem
  validation anchor that must remain coherent while runtime and storage seams
  move.

</canonical_refs>

<todo_bullet_coverage>

## TODO Bullet Coverage Contract

The numbered plans own the complete matching source sections from
`054-TODO.md`, not only the summary prose copied into each plan:

| Source section or rule | Plan owner | Coverage rule |
| --- | --- | --- |
| Architectural Verdict, Migration Safety Rules, hard steps 1-3 | `054-01-PLAN.md` | Freeze public proof and backend-boundary guardrails before seam rewiring and introduce low-level backend contracts without changing the `SettlementStore` public API. |
| Section 1 `z00z_rollup_node` table | `054-04-PLAN.md`, `054-06-PLAN.md`, `054-07-PLAN.md` | Runtime boundary adoption, placeholder/layout cleanup, and final docs together own every node row. |
| Section 2 `z00z_runtime/aggregators` table | `054-03-PLAN.md`, `054-04-PLAN.md`, `054-06-PLAN.md`, `054-07-PLAN.md` | Planner split, placement/executor work, rename normalization, and docs closeout together own every aggregator row. |
| Section 3 `z00z_runtime/validators` table | `054-04-PLAN.md`, `054-06-PLAN.md`, `054-07-PLAN.md` | Boundary adoption, rename normalization, and final docs together own every validator row. |
| Section 4 `z00z_runtime/watchers` table | `054-04-PLAN.md`, `054-06-PLAN.md`, `054-07-PLAN.md` | Boundary adoption, rename normalization, and final docs together own every watcher row. |
| Section 5 `z00z_storage` root/backend table, hard steps 4-7 | `054-01-PLAN.md`, `054-02-PLAN.md`, `054-07-PLAN.md` | Backend seam creation, manifest/error extraction, seam rewiring, and doc closeout together own every storage root/backend row. |
| Section 6 checkpoint table | `054-05-PLAN.md`, `054-07-PLAN.md` | Storage cleanup and closeout validation own every checkpoint facade, support, and test-anchor row. |
| Section 7 snapshot table | `054-05-PLAN.md`, `054-07-PLAN.md` | Snapshot remains a separate surface area and is revalidated at closeout instead of being folded into a backup layer. |
| Section 8 serialization table | `054-05-PLAN.md`, `054-07-PLAN.md` | Duplicate temp-tree cleanup plus final docs and validation own every serialization row. |
| Section 9 settlement-root table | `054-02-PLAN.md`, `054-03-PLAN.md`, `054-05-PLAN.md`, `054-06-PLAN.md`, `054-07-PLAN.md` | Seam rewiring, selective planner split, canonicalization, delayed rename, and final docs jointly own every settlement-root row. |
| Section 10 HJMT keep-table and whitebox row | `054-05-PLAN.md`, `054-07-PLAN.md` | Internal HJMT keep rows remain validation anchors, and whitebox residue is resolved or documented without adding a parallel layer. |
| Section 11 RedB backend move table | `054-02-PLAN.md`, `054-05-PLAN.md` | Adapter extraction lands first; bridge cleanup lands only after stabilization. |
| Section 12 store-helper move table | `054-02-PLAN.md`, `054-05-PLAN.md` | Backend-common and memory helper moves land first; canonical source-shape cleanup follows after semantics stabilize. |
| Section 13 checkpoint support rows | `054-05-PLAN.md`, `054-07-PLAN.md` | Support-file rows remain explicit cleanup or validation anchors. |
| Section 14 snapshot test rows | `054-07-PLAN.md` | Snapshot validation anchors are owned explicitly at closeout, using live harness paths where TODO examples drifted. |
| Section 15 serialization nested-helper rows | `054-05-PLAN.md`, `054-07-PLAN.md` | Duplicate-helper removal lands in storage cleanup and is revalidated at closeout. |
| Section 16 benches, fuzz, scripts | `054-01-PLAN.md`, `054-07-PLAN.md` | Proof-surface guardrails start here; final closeout treats the rest as mandatory validation anchors and resolves any stale TODO path examples to live repo equivalents instead of duplicating harnesses. |
| Section 17 tests and proptest regressions | `054-01-PLAN.md`, `054-05-PLAN.md`, `054-07-PLAN.md` | Guardrails start here, storage cleanup may remove obsolete whitebox residue, and final closeout treats the matrix as mandatory evidence while using current live test-harness paths instead of spawning parallel copies. |
| Runtime aggregators tables plus hard steps 8-9 | `054-03-PLAN.md` | Split planner authority into runtime while preserving store-side semantic helpers in storage. |
| Runtime aggregators/validators/watchers/node tables plus hard steps 10-11 | `054-04-PLAN.md` | Add placement and shard-executor runtime surfaces, keep node orchestration authoritative, and prevent validator/watcher authority drift. |
| Rename-wave rows across runtime/storage/node tables plus hard step 13 | `054-06-PLAN.md` | Land file/module/doc casing renames without coupling them to public symbol renames or semantic moves. |
| Docs, migration tables, closeout gates, hard step 14, and legacy carry-over closeout rules | `054-07-PLAN.md` | Update landed-topology documentation and close the phase with explicit source-shape and validation evidence. |

Global TODO rules are mandatory for every applicable plan: the four-wave order,
the “do not mix in one wave” list, the carry-now legacy verification gates and
ordering rules, the proof/public API compatibility gate, and the verified
source-shape baseline counts from section 18. Per D-19, keep rows, manifest
rows, placeholder rows, and validation-only anchors remain mandatory even when
the current slice does not expect a code edit in that exact file.

### Hard-Step Ownership

- Hard step 1 -> `054-01-PLAN.md`
- Hard step 2 -> `054-01-PLAN.md`
- Hard step 3 -> `054-01-PLAN.md`
- Hard step 4 -> `054-02-PLAN.md`
- Hard step 5 -> `054-02-PLAN.md`
- Hard step 6 -> `054-02-PLAN.md`
- Hard step 7 -> `054-02-PLAN.md`
- Hard step 8 -> `054-03-PLAN.md`
- Hard step 9 -> `054-03-PLAN.md`
- Hard step 10 -> `054-04-PLAN.md`
- Hard step 11 -> `054-04-PLAN.md`
- Hard step 12 -> `054-05-PLAN.md`
- Hard step 13 -> `054-06-PLAN.md`
- Hard step 14 -> `054-07-PLAN.md`

### Non-Mixing Rules

- Backend seam extraction and rename work must stay in separate waves.
- File or module rename and public symbol rename must stay in separate waves.
- Planner-authority relocation and validator or watcher naming cleanup must
  stay in separate waves.
- `SettlementStore` backend rewiring and proof-surface breakage must never
  share one slice.
- `rocksdb` stub introduction is forbidden in the seam-introduction wave.
- Whole-`tx_plan` move-only rewrites are forbidden; the split must stay
  semantic-first.

</todo_bullet_coverage>

<specifics>

## Specific Ideas

- Live path-attribute and module-body include debt counts verified on
  2026-06-08 after `054-07` closeout:
  `wallets=254`, `simulator=182`, `core=92`, `storage=0`, `crypto=26`,
  `utils=11`, `runtime/*=0`.
- `crates/z00z_storage/src/settlement/store/mod.rs` now owns the live
  `SettlementTreeBackend` facade through canonical submodule paths.
- `crates/z00z_storage/src/backend/redb/mod.rs` now owns the durable RedB
  backend tree directly.
- `crates/z00z_storage/src/test_support/*` and
  `crates/z00z_storage/tests/snapshot_suite/*` replaced the prior storage test
  support shims with one canonical helper and suite layout.
- `crates/z00z_storage/Cargo.toml` currently exposes no backend feature split,
  so the plan packet must treat manifest gating as an explicit seam-boundary
  target rather than an implied side effect.
- Runtime crates currently have zero canonical-module debt, so Phase 054
  runtime work is about authority placement and rename/layout cleanup rather
  than `#[path]` elimination.
- The TODO table rows that say `keep` are not filler. They are validation
  anchors used to prevent concept drift and to ensure the refactor extends the
  existing codebase instead of building a parallel layer.
- The phase-wide proof/public API guard is stronger than naming cleanup. Any
  structural move that destabilizes `assets_proofs.rs`, storage integration
  tests, or the semantic settlement facade is a regression even if the target
  topology looks cleaner.

## Plan Map

| Plan | Focus | Wave |
| --- | --- | --- |
| `054-01` | guardrails, backend contracts, and backend-error boundary freeze | 1 |
| `054-02` | RedB/common/memory extraction and `SettlementStore` seam rewiring | 2 |
| `054-03` | runtime batch planner split and storage-local `tx_plan` retention | 3 |
| `054-04` | runtime placement/executor surfaces plus validator/watcher/node boundaries | 3 |
| `054-05` | storage canonical-module cleanup and duplicate helper removal | 4 |
| `054-06` | delayed rename wave and placeholder/layout cleanup | 5 |
| `054-07` | landed-topology docs, migration tables, and full closeout evidence | 6 |

</specifics>

<deferred>

## Deferred Ideas

- Repository-wide wallet canonical-module cleanup after the storage/runtime
  semantic wave.
  Verified hot spots: `src/adapters/rpc/methods`, `src/services`, `src/key`,
  `src/receiver`, and `tests/test_common` plus related `.inc` support
  patterns.
- Repository-wide simulator canonical-module cleanup after production modules
  stabilize.
  Verified hot spots: `src/scenario_1/*`, `tests/*`, and stage utility or
  support-bridge folders.
- `z00z_core`, `z00z_crypto`, and `z00z_utils` canonical-module cleanup as
  follow-up mechanical slices, not part of this semantic migration packet.
  Verified core hot spots: `src/assets/*` and `src/genesis/*`.
  Verified crypto hot spots: `src/hash/*`, `src/aead/*`, `src/types/*`, and
  `src/protocol/ecdh`.
  Verified utils hot spots: `src/io/fs*` and `src/os_hardening*`.
- A real `rocksdb` backend adapter, but only when a tested implementation
  exists.
- Any later public symbol rename beyond the file/module rename wave in this
  packet.

</deferred>

---

*Phase: 054-Refactor-Crates*
*Context gathered: 2026-06-08 via TODO-driven analysis*
