# Phase 054 Source Audit

**Generated:** 2026-06-08
**Purpose:** prove that the Phase 054 plan packet covers the roadmap goal, the
live TODO authority, the referenced HJMT/storage/runtime documents, and the
locked context decisions without silently dropping required work.

## Coverage Matrix

| Source | ID | Feature / requirement | Plan | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| GOAL | - | Backend seam, planner-authority split, delayed rename wave, no duplicate authority surfaces | `054-01` to `054-07` | COVERED | Entire plan packet preserves the roadmap goal sequence. |
| TODO | W1 | Freeze compatibility gates before structural storage moves | `054-01` | COVERED | Anchored to `assets_proofs.rs` and storage suites first. |
| TODO | W2 | Add `backend/mod.rs` and `backend/error.rs` under `SettlementTreeBackend` | `054-01` | COVERED | Separate seam creation from downstream rewiring. |
| TODO | W3 | Move RedB-specific code into `backend/redb/*` | `054-02` | COVERED | Kept inside the storage seam wave. |
| TODO | W4 | Move shared helpers into `backend/common/*` and `backend/memory/*` | `054-02` | COVERED | Preserves `SettlementStore` semantics. |
| TODO | W5 | Rewire `SettlementStore` to the backend seam without proof/API drift | `054-02` | COVERED | Storage public contract stays stable until rename wave. |
| TODO | W6 | Move only planner-authority logic into runtime `batch_planner.rs` | `054-03` | COVERED | Split is semantic-first, not move-only. |
| TODO | W7 | Keep store-side `tx_plan` precheck and dry-run helpers in storage | `054-03` | COVERED | Explicitly retained in the storage truth layer. |
| TODO | W8 | Add runtime placement and shard executor surfaces | `054-04` | COVERED | `AggregatorId`, placement, and executor stay runtime-only. |
| TODO | W9 | Keep validators/watchers downstream-only, not planner authority | `054-04` | COVERED | Boundary preserved during runtime adoption. |
| TODO | W10 | Remove the duplicate serialization temp-tree helper | `054-05` | COVERED | Scoped to localized structural drift only. |
| TODO | W11 | Canonicalize active storage bridge debt after seam stabilization | `054-05` | COVERED | `store.rs`, `redb_backend.rs`, `tx_plan.rs`, and related helpers. |
| TODO | W12 | Run delayed rename wave only after semantic stabilization | `054-06` | COVERED | File/module renames stay separate from public symbol renames. |
| TODO | W13 | Update README, architecture docs, and migration tables last | `054-07` | COVERED | Landed topology only; no in-flight wording. |
| TODO | W14 | Carry forward legacy zero-`#[path]` / zero-`include!` closeout rules | `054-05`, `054-07` | COVERED | Active storage hot spots first, full phase closeout last. |
| TODO | W15 | Preserve generated-data exception for simulator runner contract | `054-07` | COVERED | Recorded as an explicit closeout exception, not silent debt. |
| TODO | M1 | `z00z_runtime` is a three-crate namespace folder, not one monolithic crate | `054-CONTEXT`, `054-04`, `054-06`, `054-07` | COVERED | Context and plans describe crate-family ownership explicitly. |
| TODO | M2 | HJMT paper target names are target-state vocabulary until live code lands | `054-CONTEXT`, `054-01`, `054-06`, `054-07` | COVERED | Plan packet distinguishes proposed target names from current live exports. |
| TODO | M3 | Migration text stays anchored to current live symbols during the move | `054-CONTEXT`, `054-01`, `054-06` | COVERED | Current runtime and storage public symbol anchors remain explicit across the packet. |
| TODO | M4 | Semantic drift, API drift, and review drift must all be prevented | `054-CONTEXT`, `054-01`, `054-05`, `054-06` | COVERED | Packet now carries all three drift classes as explicit review and execution guardrails. |
| TODO | S1 | Section 1 node table, including placeholder cleanup and theorem-test keep row | `054-04`, `054-06`, `054-07` | COVERED | Boundary adoption, rename/layout cleanup, and docs closeout jointly own the node rows. |
| TODO | S2 | Section 2 aggregators table, including keep rows and new planner or placement modules | `054-03`, `054-04`, `054-06`, `054-07` | COVERED | Planner split, runtime placement, rename normalization, and docs closeout jointly own every aggregator row. |
| TODO | S3 | Section 3 validators table, including keep rows and rename candidates | `054-04`, `054-06`, `054-07` | COVERED | Boundary adoption, rename normalization, and docs closeout jointly own every validator row. |
| TODO | S4 | Section 4 watchers table, including keep rows and rename candidates | `054-04`, `054-06`, `054-07` | COVERED | Boundary adoption, rename normalization, and docs closeout jointly own every watcher row. |
| TODO | S5 | Section 5 storage root/backend table, including `Cargo.toml` feature gating and no-`rocksdb`-stub rule | `054-01`, `054-02`, `054-07` | COVERED | Seam creation, manifest/error extraction, seam rewiring, and final docs jointly own the storage root/backend rows. |
| TODO | S6 | Section 6 checkpoint table | `054-05`, `054-07` | COVERED | Checkpoint split normalization stays inside storage canonicalization and closeout validation. |
| TODO | S7 | Section 7 snapshot table | `054-05`, `054-07` | COVERED | Snapshot remains a distinct surface area and is validated again at closeout. |
| TODO | S8 | Section 8 serialization table | `054-05`, `054-07` | COVERED | Duplicate helper removal and final docs jointly own serialization rows. |
| TODO | S9 | Section 9 settlement-root table, including `types_identity`, `types_query`, `types_record`, README casing, and `tx_plan` split | `054-02`, `054-03`, `054-05`, `054-06`, `054-07` | COVERED | Seam rewiring, selective planner split, canonicalization, delayed rename, and docs closeout jointly own every settlement-root row. |
| TODO | S10 | Section 10 HJMT keep-table and whitebox-removal row | `054-05`, `054-07` | COVERED | Internal HJMT seams remain validation anchors, and whitebox residue is owned by storage cleanup or closeout evidence. |
| TODO | S11 | Section 11 RedB backend move table | `054-02`, `054-05` | COVERED | Adapter extraction lands first; bridge cleanup lands after stabilization. |
| TODO | S12 | Section 12 store-helper move table | `054-02`, `054-05` | COVERED | Helper extraction lands first; canonical-module cleanup lands after stabilization. |
| TODO | S13 | Section 13 checkpoint support rows | `054-05`, `054-07` | COVERED | Storage cleanup and closeout validation jointly own the support-file rows. |
| TODO | S14 | Section 14 snapshot test rows | `054-07` | COVERED | Snapshot test rows are explicit closeout validation anchors. |
| TODO | S15 | Section 15 serialization nested-helper rows | `054-05`, `054-07` | COVERED | Duplicate-helper removal lands in storage cleanup; docs and validation close the loop. |
| TODO | S16 | Section 16 benches, fuzz, scripts rows | `054-01`, `054-07` | COVERED | `assets_proofs.rs` is the opening compatibility gate; the remaining bench/fuzz/script rows are explicit closeout anchors, and stale TODO examples map to the live `assets_benches.md`, `settlement_proofs.rs`, and `settlement_proofs/*` paths already in the repo. |
| TODO | S17 | Section 17 tests and proptest-regression rows | `054-01`, `054-05`, `054-07` | COVERED | Guardrails start here, whitebox residue cleanup lives in storage cleanup, and closeout uses the live root-level snapshot harness and corpus-fixture equivalents instead of recreating stale TODO folder examples. |
| DOC | U1 | `z00z_rollup_node` stays the orchestration root | `054-04`, `054-07` | COVERED | Bound to HJMT upgrade whole-system layering. |
| DOC | U2 | Runtime planner authority is operational, not public truth | `054-03`, `054-04` | COVERED | Planner split and placement surfaces are runtime-owned only. |
| DOC | U3 | `z00z_storage` remains semantic truth and proof owner | `054-01`, `054-02`, `054-03`, `054-05` | COVERED | Every storage slice preserves semantic ownership. |
| DOC | U4 | `StorageBackend` and `JournalBackend` are low-level durable seams | `054-01`, `054-02` | COVERED | Introduced beneath `SettlementTreeBackend`. |
| DOC | U5 | `Z00Z-HJMT-Key-Terms.md` is the cross-phase naming authority when term-selection conflicts appear | `054-CONTEXT`, `054-01`, `054-03`, `054-04`, `054-06`, `054-07` | COVERED | Naming-sensitive plans now anchor to the shared glossary to prevent terminology drift. |
| DOC | S1 | `SettlementStateRoot` remains the public semantic root | `054-01`, `054-02`, `054-07` | COVERED | No backend-root authority or public drift permitted. |
| DOC | S2 | `ProofBlob` and storage proof helpers remain canonical contracts | `054-01`, `054-02`, `054-07` | COVERED | Guarded by proof benches and storage tests. |
| DOC | S3 | `SettlementStore` is the storage facade and must stay storage-owned | `054-01`, `054-02`, `054-03` | COVERED | Rewiring cannot demote it into a thin transport adapter. |
| LIVE | L1 | `store.rs` is the current hot spot with `#[path]` bridge wiring | `054-02`, `054-05` | COVERED | First rewire, then canonicalize. |
| LIVE | L2 | `redb_backend.rs` is the current durable backend bridge hot spot | `054-02`, `054-05` | COVERED | First extract, then canonicalize. |
| LIVE | L3 | `tx_plan.rs` is the current split point between runtime planning and storage semantics | `054-03`, `054-05` | COVERED | First semantic split, then source-shape cleanup. |
| LIVE | L4 | Runtime crates already have zero `#[path]` debt | `054-04`, `054-06` | COVERED | Runtime work stays focused on boundaries and renames. |
| LIVE | L5 | TODO sections 16-17 contain some stale bench/fuzz/test path examples relative to the current repository | `054-07` | COVERED | Closeout binds to live equivalents and forbids creating a duplicate bench, fuzz, or test layer just to match stale text. |
| LIVE | L6 | TODO sections 10-15 contain nested target-shape examples, while the live repo currently uses flat `hjmt_*`, `redb_backend_*`, `store_*`, canonical nested checkpoint or snapshot modules, and `serialization/build/temp_tree.rs` for the temp-tree helper | `054-CONTEXT`, `054-05`, `054-07` | COVERED | Packet normalizes execution to live equivalents and forbids inventing shadow storage folders just to mirror stale text. |
| CONTEXT | D-01 | Fixed backend/adapters/planner/rename/doc wave order | `054-01` to `054-07` | COVERED | Encoded in plan dependencies and roadmap wave notes. |
| CONTEXT | D-02 | Proof/public API gate comes first | `054-01` | COVERED | First plan owns the compatibility baseline. |
| CONTEXT | D-03 | Backend seam stays below `SettlementTreeBackend` | `054-01`, `054-02` | COVERED | Split seam from semantic facade. |
| CONTEXT | D-04 | Scope limited to `redb` plus `memory` in this phase | `054-01`, `054-02` | COVERED | No `rocksdb` stub plan exists. |
| CONTEXT | D-05 | `StoreBackendError` symbol stability during seam work | `054-01`, `054-02` | COVERED | Error move is isolated from rename wave. |
| CONTEXT | D-06 | `SettlementStore` keeps semantic ownership | `054-02`, `054-03`, `054-05` | COVERED | No plan demotes it to a transport-only layer. |
| CONTEXT | D-07 | Planner authority moves selectively into runtime | `054-03` | COVERED | Explicitly bounded to runtime planner logic. |
| CONTEXT | D-08 | Store-side semantic helpers remain in storage | `054-03`, `054-05` | COVERED | `tx_plan` is split, not uprooted. |
| CONTEXT | D-09 | Placement metadata is operational only | `054-04`, `054-07` | COVERED | Docs and boundaries both preserve this. |
| CONTEXT | D-10 | `z00z_rollup_node` stays the orchestration root | `054-04`, `054-07` | COVERED | Runtime adoption cannot add a super-layer. |
| CONTEXT | D-11 | Rename wave happens strictly after stabilization | `054-06` | COVERED | No earlier plan carries rename work. |
| CONTEXT | D-12 | Storage canonical-module cleanup is scoped and semantic-first | `054-05` | COVERED | Cleanup follows seam stabilization. |
| CONTEXT | D-13 | Legacy source-shape rules are carry-over guardrails only | `054-05`, `054-07` | COVERED | Applied to active hot spots and closeout evidence. |
| CONTEXT | D-14 | Generated-data exception stays explicit | `054-07` | COVERED | Recorded during final docs/closeout. |
| CONTEXT | D-15 | If legacy spec is missing, use TODO section 18 as live authority | `054-01`, `054-07` | COVERED | Plans do not recreate the missing file. |
| CONTEXT | D-16 | Mandatory bootstrap -> broad cargo -> review-loop verify order | `054-01` to `054-07` | COVERED | Repeated in every auto-task verify block. |
| CONTEXT | D-17 | Use `/z00z-git-versioning` for commits | `054-01` to `054-07` | COVERED | Repeated in every auto-task verify block. |
| CONTEXT | D-18 | No duplicate or parallel implementation layer | `054-01` to `054-07` | COVERED | Applies to every slice. |
| CONTEXT | D-19 | Keep rows, manifest rows, and validation-only rows remain mandatory anchors | `054-01` to `054-07` | COVERED | Prevents TODO table drift where “keep” items would otherwise disappear from execution planning. |
| CONTEXT | D-20 | `z00z_runtime` is a crate family, not one monolithic crate | `054-01` to `054-07` | COVERED | Packet describes family-level planning and avoids monolith drift. |
| CONTEXT | D-21 | Target-state names stay proposed until live code and exports land | `054-01`, `054-03`, `054-04`, `054-06`, `054-07` | COVERED | Prevents docs or plans from overclaiming non-live APIs. |
| CONTEXT | D-22 | Wave-mixing prohibitions are hard execution rules | `054-01` to `054-06` | COVERED | Non-mixing constraints are enforced across the slice boundaries. |
| CONTEXT | D-23 | Legacy follow-up extraction must use verified live baselines, not stale inventories, dead paths, or repo-wide heuristics | `054-CONTEXT`, `054-07` | COVERED | Packet preserves the verified baseline and rejects verbatim carry-over drift from the deleted legacy spec. |
| CONTEXT | D-24 | Phase 054 explicitly guards semantic drift, API drift, and review drift | `054-01`, `054-05`, `054-06` | COVERED | Proof semantics, compatibility oracles, and wave separation are all now explicit planning constraints. |

## Coverage Summary

- No roadmap-goal item is left without a plan owner.
- No live TODO execution step is left without a plan owner.
- No numbered TODO section table is left without a plan owner.
- No locked context decision is left without a plan owner.
- Top-level migration-safety bullets and non-mixing rules are carried as
  explicit context decisions, not only as implied section ownership.
- Legacy carry-over rules now include the “do not copy verbatim” constraints,
  so stale inventories, dead paths, and repo-wide heuristics are not revived by
  closeout docs.
- Live-path drift in TODO sections 16-17 is explicitly normalized to current
  repository anchors instead of being “fixed” by creating parallel folders.
- Live-path drift in TODO sections 10-15 is explicitly normalized to current
  flat storage anchors instead of being “fixed” by creating shadow nested
  folders.
- No separate plan exists for `rocksdb`, repo-wide wallet cleanup, or simulator
  cleanup because those items are explicitly deferred outside this packet.

## Deliberate Deferrals

These items remain intentionally out of the Phase 054 execution packet:

- wallet canonical-module cleanup outside the active storage/runtime seam;
- simulator canonical-module cleanup outside the generated-data exception note;
- `z00z_core`, `z00z_crypto`, and `z00z_utils` mechanical source-shape cleanup;
- a real `rocksdb` adapter implementation;
- any public symbol rename beyond the delayed file/module rename wave.
