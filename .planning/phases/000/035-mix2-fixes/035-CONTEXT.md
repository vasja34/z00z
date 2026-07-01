# Phase 035: Mix2 Fixes - Context

**Gathered:** 2026-04-11
**Status:** Completed execution baseline with `035-01-PLAN.md` through `035-19-PLAN.md` closed; this file remains the phase-local boundary reference

## Phase Boundary

Phase 035 is a completed execution phase rooted in the pre-existing
`035-mix2-fixes` directory. `035-TODO.md` remains the canonical Phase 035
execution inventory.

No historical deferred ledger is imported by default, and any later widening of
historical intake must flow through the fixed deferred-source update order
before implementation or closeout artifacts change.

The live execution authority surface is fixed to `035-TODO.md` plus the six
phase-local source documents named there. No alternate backlog or planner-
invented layer may compete with that surface, and `035-a1-deferred.md` remains
a scope-boundary artifact rather than the substantive implementation authority
for sender or stealth work.

Downstream execution must preserve the canonical Phase 035 tasks exactly as
they already exist in `035-TODO.md`, in the same sequence, one canonical task
after another. This phase is about landing that inventory without rewriting
backlog semantics, renaming task titles, excluding tasks, creating a parallel
implementation layer, or introducing concept drift relative to the existing
codebase and phase-local sources.

Any future historical-deferred intake must update `035-a1-deferred.md` first,
then `035-TODO.md`, then this context file before implementation or closeout
artifacts widen scope.

## Decision Summary Mirror

The Phase 035 execution baseline copied from `035-TODO.md` is:

1. import no historical deferred item by default;
2. keep substantive Phase 035 work anchored only to the live sender and
  stealth specs in `035-a4-fix-spec.md` and `035-a5-fix-spec.md`;
3. treat `035-1-deferred.md` as a scope-boundary and triage artifact rather
  than as a second implementation spec;
4. keep the lone surviving `keep_path(...)` item outside semantic Phase 035
  closure unless it is explicitly attached as opportunistic housekeeping;
5. reject stale vendor-doctest wording and already-resolved release or
  validation leftovers as Phase 035 obligations.

## File-First Implementation Order Mirror

The canonical file-first order copied from `035-TODO.md` is:

1. `.planning/phases/035-mix2-fixes/035-a1-deferred.md`
2. `.planning/phases/035-mix2-fixes/035-a2-suffixes.md`
3. `.planning/phases/035-mix2-fixes/035-a3-garbage-filter.md`
4. `.planning/phases/035-mix2-fixes/035-a4-fix-spec.md`
5. `.planning/phases/035-mix2-fixes/035-a5-fix-spec.md`
6. `.planning/phases/035-mix2-fixes/035-a6-renames.md`
7. `.planning/phases/035-mix2-fixes/035-TODO.md`
8. `crates/z00z_storage/src/assets/store_internal/store_query.rs` only if the
  optional sidecar is explicitly activated
9. phase validation and closeout artifacts only after implementation work
  begins

## Implementation Decisions

### Canonical planning inventory

- **D-01:** `.planning/phases/035-mix2-fixes/035-TODO.md` is the canonical
  planning inventory for Phase 035.
- **D-02:** The planner must cover all canonical Phase 035 tasks listed in
  `035-TODO.md`.
- **D-03:** Planning must proceed sequentially, one canonical task after
  another, in the order already defined by `035-TODO.md`.

### Task identity and wording

- **D-04:** Task titles must remain exactly as written in `035-TODO.md`.
- **D-05:** Task wording must remain exactly as written in `035-TODO.md`.
- **D-06:** The planner may add dependency notes, execution detail, and
  traceability, but must not merge, normalize, reinterpret, rename, or reorder
  the canonical task labels.

### Coverage and blocker handling

- **D-07:** No task in the canonical Phase 035 table may be excluded from
  planning under normal conditions.
- **D-08:** Only an extreme principle-level blocker that cannot be bypassed may
  justify recording a planning stop or exception.
- **D-09:** If such a blocker appears, it must be recorded explicitly in the
  final planning report rather than used to silently weaken, drop, or skip
  canonical tasks.

### Reuse and anti-drift rules

- **D-10:** Do not duplicate the existing codebase or its logic.
- **D-11:** Do not introduce a parallel layer while planning or implementing
  Phase 035 work.
- **D-12:** Prevent codebase concept drift by keeping Phase 035 planning tied
  to the existing live seams and the canonical phase-local sources.
- **D-13:** Treat `035-a1-deferred.md`, `035-a2-suffixes.md`,
  `035-a3-garbage-filter.md`, `035-a4-fix-spec.md`, `035-a5-fix-spec.md`, and
  `035-a6-renames.md` as the authoritative Phase 035 source set behind the
  backlog ranges already mapped in `035-TODO.md`.

### Execution discipline and transfer rules

- **D-14:** The canonical task-to-source mapping is fixed and must stay intact:
  `035-01..035-07` -> `035-a1-deferred.md`, `035-08..035-14` ->
  `035-a2-suffixes.md`, `035-15..035-21` -> `035-a3-garbage-filter.md`,
  `035-22..035-31` -> `035-a4-fix-spec.md`, `035-32..035-40` ->
  `035-a5-fix-spec.md`, and `035-41..035-49` -> `035-a6-renames.md`.
- **D-15:** Every numbered task carries forward its TODO-level `MANDATORY
  pre-read`, declared file list, task-local tests, exit condition, and any
  lane-local dependency note. Downstream planning may refine execution detail
  but may not strip those controls.
- **D-16:** Validation waves, readiness gates, acceptance gates, and completion
  gates from `035-TODO.md` are mandatory execution controls rather than
  optional documentation. Planning is incomplete if any of them is omitted.
- **D-17:** The Phase 035 planning set must preserve the active artifact
    discipline from `.planning/STATE.md`: logs and manifests are evidentiary
    limiters only and must not replace semantic source of truth.

### Cryptographic non-regression guardrails

- **D-18:** The canonical sender-construction seam remains wallet-owned under
  `crates/z00z_wallets/src/core/stealth/`. Planning must not drift sender
  derivation or output-construction ownership back into `z00z_core::tx` or any
  new adapter-only façade.
- **D-19:** Sender planning must preserve the current live formula contracts
  and request-bound semantics around `owner_tag`, `tag16`, `leaf_ad`,
  `enc_pack`, hedged-`r`, duplicate-`R`, and the live `ZkPack` binding. These
  are non-regression invariants, not planner discretion.
- **D-20:** Sender semantic acceptance requires the dedicated validated
  card-only entrypoint, downstream adapter convergence, regression coverage,
  and documentation correction wave already encoded in `035-TODO.md`.
- **D-21:** The stealth lane is intentionally fenced to the documented Phase 035
  additions only: receiver-secret narrowing, derivation-vector freeze plus
  regression confirmation, and the V2 memo contract plus receive-path
  enablement in the exact TODO order.
- **D-22:** Memo work must remain wallet-private decrypted metadata; V1-stable
  behavior and side-by-side compatibility must remain explicit where the live
  Phase 035 source set says so.
- **D-23:** Out of scope for default Phase 035 stealth execution are PIR,
  OPRF, bucket-routing expansions, Poseidon2 `ZkPack` migration, Poseidon3-only
  unification, and recursive checkpoint-backend work unless a canonical source
  is updated first.

### Deletion, rename, rollback, and parallelization safety

- **D-24:** Immediate garbage-removal planning is limited to rows explicitly
  classified by the canonical garbage filter as `FALSE`, `GARBAGE`, or
  `DEBUG-ONLY`. Rows with `InProduction = TRUE` remain in the compatibility or
  migration keep-set until the canonical source itself is updated.
- **D-25:** The `only current production-path` interpretation from the garbage
  lane is a source-drift review aid, not standalone authority to delete live
  compatibility or migration surfaces.
- **D-26:** Suffix and rename work must remain declaration-backed and
  path-backed, with separate lanes for non-test rows, test-only rows, filename
  rows, and explicit exclusions. The raw matrix inventory is not an automatic
  rename queue.
- **D-27:** `No-change` rows from the curated rename lane stay frozen until the
  canonical rename source widens scope. Vendor or protected-tree rows remain
  excluded.
- **D-28:** Parallel execution is unsafe across tasks that mutate the same
  public facade root, compatibility lane, or acceptance gate. If duplicate
  public entrypoints, hidden boundary drift, or path churn appear, planning
  must fall back to the last stable seam and update the canonical source set
  before resuming.

### the agent's Discretion

- The agent may choose how to format per-task planning artifacts and
  traceability notes.
- The agent may decide how much implementation detail to attach to each task,
  as long as all canonical tasks are preserved in order and the anti-drift
  rules remain intact.

## Specific Ideas

- `035-TODO.md` is the only canonical scheduling surface for Phase 035.
- The full canonical task inventory must be planned in order.
- Task titles and task wording are frozen.
- No duplicate logic, no parallel layer, and no concept drift are allowed.
- If an extreme principle blocker appears, record it explicitly instead of
  silently dropping or weakening the affected task.

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase boundary and live state

- `.planning/ROADMAP.md` §Phase 035: Mix2 Fixes — defines the canonical phase
  surface and dependency on Phase 034.
- `.planning/STATE.md` — confirms that Phase 035 is the active execution phase
  with `035-03-PLAN.md` closed and `035-04-PLAN.md` as the next active plan.

### Canonical Phase 035 inventory

- `.planning/phases/035-mix2-fixes/035-TODO.md` — canonical execution
  inventory, order, dependency chain, validation matrix, and completion gates.

### Phase-local source set

- `.planning/phases/035-mix2-fixes/035-a1-deferred.md` — deferred-intake
  boundary and scope-honesty authority.
- `.planning/phases/035-mix2-fixes/035-a2-suffixes.md` — suffix inventory,
  production-head interpretation, and cleanup guidance.
- `.planning/phases/035-mix2-fixes/035-a3-garbage-filter.md` — immediate
  garbage versus compatibility-live keep-set classification.
- `.planning/phases/035-mix2-fixes/035-a4-fix-spec.md` — sender workflow
  canonicalization source.
- `.planning/phases/035-mix2-fixes/035-a5-fix-spec.md` — stealth additions and
  triage source.
- `.planning/phases/035-mix2-fixes/035-a6-renames.md` — curated rename scope
  and approved rename targets.

### Repository constraints

- `.github/copilot-instructions.md` — repository-local anti-duplication,
  no-parallel-layer, and safe-workflow constraints.
- `.github/requirements/Z00Z_DESIGN_FOUNDATION.md` — primary architectural
  authority for one-source-of-truth boundaries and duplicate-abstraction
  avoidance.

## Existing Code Insights

### Reusable Assets

- Existing production seams in the live codebase must be extended rather than
  shadowed by fresh compatibility or parallel layers.
- The phase-local source documents already split the work into deferred,
  suffix, garbage-filter, sender, stealth, and rename lanes; planning should
  reuse that structure instead of inventing a second taxonomy.

### Verified Integration Anchors

- Verified sender anchors:
  `crates/z00z_wallets/src/core/stealth/output_build.rs`,
  `crates/z00z_wallets/src/core/stealth/output.rs`,
  `crates/z00z_wallets/src/core/tx/builder.rs`, and
  `crates/z00z_wallets/src/core/tx/output_flow.rs`.
- Verified stealth anchors:
  `crates/z00z_wallets/src/core/key/stealth_keys_receiver.rs` plus the
  currently verified stealth-facing test surface already present under
  `crates/z00z_wallets/tests/`. Task-specific additions named by
  `035-TODO.md` remain proposed until they exist in the repository.
- Verified garbage and compatibility anchors:
  `crates/z00z_storage/src/assets/store_internal/store_query.rs` (optional
  sidecar only), `crates/z00z_wallets/src/db/redb_wallet_store_debug_export.rs`,
  `crates/z00z_crypto/src/claim/statement.rs`, and
  `crates/z00z_wallets/src/core/backup/wallet_backup.rs`.
- Verified planning anchors: the six phase-local source documents and
  `035-TODO.md` itself.
- Proposed rather than verified current-tree anchors must stay labeled as such.
  Example: `crates/z00z_wallets/tests/test_asset_pack_v2_memo.rs` is referenced
  by `035-TODO.md` but was not verified as an existing file during this context
  review, so downstream planning must treat it as a proposed test target unless
  it is created or separately verified.

### Established Patterns

- Recent phases use one canonical TODO inventory per phase as the scheduling
  authority.
- Planning truth must stay aligned with the phase-local backlog and must not
  overclaim closure or silently import unrelated leftovers.

### Integration Points

- Planning starts from `.planning/phases/035-mix2-fixes/035-TODO.md` and the
  six mapped source documents.
- Sequential planner output must feed directly into Phase 035 plan files
  without changing the canonical task wording.

## Canonical Task Transfer Matrix

This matrix confirms that every canonical task from `035-TODO.md` has been
transferred into this context as a required planning item. No task is dropped,
renamed, or demoted by this review. It is a transfer-proof artifact only, not
a task-completion dashboard or execution-status source.

### Deferred-Intake Lane (`035-1-deferred.md`)

| Task | Canonical title | Role | Transfer status |
| --- | --- | --- | --- |
| `035-01` | Canonical Deferred-Intake Freeze | Planning | Transferred and required |
| `035-02` | Live Phase-Source Binding | Planning | Transferred and required |
| `035-03` | Historical Triage Lock-In | Planning | Transferred and required |
| `035-04` | Optional `keep_path(...)` Sidecar Gate | Planning sidecar gate | Transferred and required |
| `035-05` | Phase Closeout Honesty Rules | Planning | Transferred and required |
| `035-06` | Deferred-Consistency Validation Wave | Validation wave | Transferred and required |
| `035-07` | Optional Sidecar Validation Gate | Acceptance gate | Transferred and required |

### Suffix Inventory Lane (`035-2-suffixes.md`)

| Task | Canonical title | Role | Transfer status |
| --- | --- | --- | --- |
| `035-08` | Suffix Authority Freeze | Planning | Transferred and required |
| `035-09` | Declaration-Backed Inventory Lock-In | Planning | Transferred and required |
| `035-10` | Production-Head Cleanup Target | Planning | Transferred and required |
| `035-11` | Filename And Exclusion Hygiene | Planning | Transferred and required |
| `035-12` | Curated Rename And Retirement Handoff | Planning | Transferred and required |
| `035-13` | Suffix Inventory Validation Wave | Validation wave | Transferred and required |
| `035-14` | Suffix Cleanup Readiness Gate | Acceptance gate | Transferred and required |

### Garbage-Filter Lane (`035-3-garbage-filter.md`)

| Task | Canonical title | Role | Transfer status |
| --- | --- | --- | --- |
| `035-15` | Garbage Classification Freeze | Planning | Transferred and required |
| `035-16` | Hard-Garbage Removal Cluster | Planning | Transferred and required |
| `035-17` | Debug-Dump Retirement Review | Planning | Transferred and required |
| `035-18` | Compatibility And Migration Keep-Set Freeze | Planning | Transferred and required |
| `035-19` | Current-Path-Only Source Drift Handoff | Planning | Transferred and required |
| `035-20` | Garbage-Filter Validation Wave | Validation wave | Transferred and required |
| `035-21` | Current-Path Closure Gate | Acceptance gate | Transferred and required |

### Sender Workflow Lane (`035-4-fix-spec.md`)

| Task | Canonical title | Role | Transfer status |
| --- | --- | --- | --- |
| `035-22` | Sender Seam Freeze | Planning | Transferred and required |
| `035-23` | Canonical Helper And Approval Extension | Planning | Transferred and required |
| `035-24` | Validated Card-Only Entrypoint | Planning | Transferred and required |
| `035-25` | Legacy Builder Adapter Convergence | Planning | Transferred and required |
| `035-26` | Replayable Bundle Adapter Convergence | Planning | Transferred and required |
| `035-27` | Stealth Export And Unit Coverage | Planning | Transferred and required |
| `035-28` | Downstream Adapter Regression Sweep | Planning | Transferred and required |
| `035-29` | Documentation Correction Wave | Planning | Transferred and required |
| `035-30` | Sender Workflow Validation Wave | Validation wave | Transferred and required |
| `035-31` | Sender Workflow Acceptance Gate | Acceptance gate | Transferred and required |

### Stealth Additions Lane (`035-5-fix-spec.md`)

| Task | Canonical title | Role | Transfer status |
| --- | --- | --- | --- |
| `035-32` | Stealth Scope Freeze | Planning | Transferred and required |
| `035-33` | Receiver-Secret Exposure Inventory | Planning | Transferred and required |
| `035-34` | Receiver-Secret Narrowing Seam | Planning | Transferred and required |
| `035-35` | Stealth Derivation Vector Freeze | Planning | Transferred and required |
| `035-36` | Derivation Drift Regression Sweep | Planning | Transferred and required |
| `035-37` | V2 Memo Contract Definition | Planning | Transferred and required |
| `035-38` | V2 Memo Receive-Path Enablement | Planning | Transferred and required |
| `035-39` | Stealth Additions Validation Wave | Validation wave | Transferred and required |
| `035-40` | Stealth Additions Acceptance Gate | Acceptance gate | Transferred and required |

### Rename Lane (`035-6-renames.md`)

| Task | Canonical title | Role | Transfer status |
| --- | --- | --- | --- |
| `035-41` | Rename Scope Freeze | Planning | Transferred and required |
| `035-42` | Live Rename Manifest And Lane Split | Planning | Transferred and required |
| `035-43` | File Rename Wave A - Test And Support Files | Planning | Transferred and required |
| `035-44` | File Rename Wave B - Wallet DB And Egui Canonical Files | Planning | Transferred and required |
| `035-45` | Signature Rename Wave A - Module, Path, And Include Mirrors | Planning | Transferred and required |
| `035-46` | Signature Rename Wave B - Types, Functions, And Methods | Planning | Transferred and required |
| `035-47` | Cross-File Reference Sweep And No-Change Guard | Planning | Transferred and required |
| `035-48` | Rename Validation Wave | Validation wave | Transferred and required |
| `035-49` | Rename Acceptance Gate | Acceptance gate | Transferred and required |

## TODO Coverage Confirmation

- The `035-TODO.md` execution rules are carried forward as mandatory planning
  discipline in this context.
- The `035-TODO.md` zero-import deferred rule, fixed six-source authority
  surface, sender and stealth substantive source binding, deferred-note
  boundary-only rule, optional `keep_path(...)` sidecar isolation, and stale-
  leftover rejection rules are mirrored above as explicit Phase 035 baseline
  decisions.
- The `035-TODO.md` decision summaries, dependency chains, file-first order,
  validation matrices, explicit phase boundaries, validation waves, and
  completion gates remain binding and may not be
  weakened downstream.
- Every canonical task keeps its TODO-local `MANDATORY pre-read`, declared
  file targets, task-local tests, and exit condition as binding execution
  controls rather than optional guidance.
- The deferred-intake, suffix, garbage-filter, sender, stealth, and rename
  lanes are all represented in this context with no exclusions.
- All suggestions and issue classes surfaced by the canonical TODO are covered
  here as planning controls: no historical deferred import by default, no
  hidden stale leftovers, optional `keep_path(...)` sidecar isolation,
  declaration-backed rename safety, sender/stealth non-regression invariants,
  and explicit validation plus acceptance gates.
- The sender lane preserves three distinct public surfaces from the canonical
  TODO: raw construction, request-bound validated construction with unchanged
  observable behavior, and the dedicated card-only validated entrypoint. The
  planner must not collapse these approval levels into a single "one live path"
  story.
- The stealth lane keeps Workstream A -> B -> C intact: receiver-secret caller
  inventory and narrowing first, derivation-vector freeze plus drift tests
  second, and the side-by-side V2 memo decode boundary plus receive-path
  enablement last.

## Deferred Ideas

None — this context intentionally locks Phase 035 to the existing canonical
task inventory and rejects hidden scope expansion.

---

*Phase: 035-mix2-fixes*
*Context gathered: 2026-04-11*
