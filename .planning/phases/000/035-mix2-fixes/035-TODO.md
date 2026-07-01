# 035-TODO

Canonical design sources:

- [035-a1-deferred](./035-a1-deferred.md)
- [035-a2-suffixes](./035-a2-suffixes.md)
- [035-a3-garbage-filter](./035-a3-garbage-filter.md)
- [035-a4-fix-spec](./035-a4-fix-spec.md)
- [035-a5-fix-spec](./035-a5-fix-spec.md)
- [035-a6-renames](./035-a6-renames.md)

Execution rules:

- execute this file in order unless a dependency note says otherwise;
- treat this file as a composed Phase 035 execution backlog with these fixed
  source ranges:
  - `035-01..035-07` -> `035-a1-deferred.md`
  - `035-08..035-14` -> `035-a2-suffixes.md`
  - `035-15..035-21` -> `035-a3-garbage-filter.md`
  - `035-22..035-31` -> `035-a4-fix-spec.md`
  - `035-32..035-40` -> `035-a5-fix-spec.md`
  - `035-41..035-49` -> `035-a6-renames.md`
- treat `035-a1-deferred.md` as normative for deferred-intake meaning and
  phase-boundary honesty;
- treat `035-TODO.md` plus the six mapped source documents as the only
  execution authority surface for Phase 035;
- treat `035-a2-suffixes.md` as normative for suffix inventory, workspace-backed
  production-head interpretation, and cleanup guidance;
- treat `035-a3-garbage-filter.md` as normative for immediate garbage versus
  compatibility-live keep-set classification;
- treat `035-a4-fix-spec.md` and `035-a5-fix-spec.md` as the substantive sender
  and stealth implementation sources for Phase 035;
- treat `035-a6-renames.md` as normative for curated rename scope and approved
  rename targets;
- keep `035-a1-deferred.md` as a boundary artifact only; it must not replace
  the live sender and stealth specs as the substantive implementation
  authority;
- do not reopen historical deferred ledgers from earlier phases during Phase
  035 execution unless `035-a1-deferred.md` is updated first;
- if historical-deferred intake is widened, update order is fixed:
  `035-a1-deferred.md` -> `035-TODO.md` -> `035-CONTEXT.md` -> affected
  closeout artifacts;
- do not import resolved, stale, vendor-doctest, or release-gate leftovers as
  hidden Phase 035 work;
- do not let `keep_path(...)` complexity cleanup appear as sender-workflow,
  stealth-hardening, or semantic Phase 035 closure unless it is explicitly
  attached as an opportunistic sidecar;
- if execution discovers a new deferred item that genuinely belongs in Phase
  035, update `035-a1-deferred.md` first, then this backlog, then the affected
  closeout artifacts;
- before starting any numbered task, complete its `MANDATORY pre-read` block.

## 🎯 Decision Summary

The execution baseline for Phase 035 deferred intake is:

1. import no historical deferred item by default;
2. keep Phase 035 substantive work anchored only to the live sender and stealth
  specs in `035-a4-fix-spec.md` and `035-a5-fix-spec.md`;
3. treat the historical deferred note as a scope-boundary and triage artifact,
   not as a second implementation spec;
4. keep the lone surviving `keep_path(...)` complexity item outside semantic
   Phase 035 closure unless it is explicitly attached as opportunistic
   housekeeping;
5. reject stale vendor-doctest wording and already-resolved release or
   validation leftovers as Phase 035 obligations.

## 🔗 Dependency Chain

Execution dependency chain:

1. `035-01` canonical deferred-intake freeze
2. `035-02` live phase-source binding
3. `035-03` historical triage lock-in
4. `035-04` optional `keep_path(...)` sidecar gate
5. `035-05` phase closeout honesty rules
6. `035-06` deferred-consistency validation wave
7. `035-07` optional sidecar validation gate

Hard dependencies:

- `035-02` depends on `035-01`
- `035-03` depends on `035-01`
- `035-04` depends on `035-03`
- `035-05` depends on `035-02` and `035-03`
- `035-06` depends on `035-01` through `035-05`
- `035-07` depends on `035-04` and `035-06`

## 🗂️ File-First Implementation Order

Edit order by file cluster:

1. `.planning/phases/035-mix2-fixes/035-a1-deferred.md`
2. `.planning/phases/035-mix2-fixes/035-a2-suffixes.md`
3. `.planning/phases/035-mix2-fixes/035-a3-garbage-filter.md`
4. `.planning/phases/035-mix2-fixes/035-a4-fix-spec.md`
5. `.planning/phases/035-mix2-fixes/035-a5-fix-spec.md`
6. `.planning/phases/035-mix2-fixes/035-a6-renames.md`
7. `.planning/phases/035-mix2-fixes/035-TODO.md`
8. only if explicitly activated:
  `crates/z00z_storage/src/assets/store_internal/store_query.rs`
9. phase validation and closeout artifacts after implementation work begins

## ✅ Validation Matrix

This table proves that the instructions from `035-a1-deferred.md` have been
converted into execution-ordered work without inventing a second Phase 035
scope.

| 035-a1-deferred source section | Required theme | TODO coverage | Status |
| --- | --- | --- | --- |
| `Purpose` | this document is a self-contained deferred-intake note for Phase 035 | execution rules; `035-01`; `035-03` | Validated mapped |
| `Intake Decision` | no required historical deferred item is inherited by default | decision summary 1; `035-01`; `035-05`; `Completion Gate` | Validated mapped |
| `Canonical Triage Of Historical Deferred Sources` | resolved and stale sources remain out of Phase 035 execution and the lone Phase 032 survivor stays optional-only | `035-03`; `035-06`; explicit phase boundary; `Completion Gate` | Validated mapped |
| `Recommended Phase 035 Scope Boundary` | Phase 035 substantive implementation stays sender or stealth scoped inside the canonical `035-TODO.md` inventory plus the six mapped source documents | execution rules; `035-02`; explicit phase boundary | Validated mapped |
| `Why Phase 035 Should Stay Clean` | unrelated storage cleanup must not dilute sender or stealth closure | decision summary 4-5; `035-04`; `035-05` | Validated mapped |
| `Optional Exception Rule` | `keep_path(...)` may exist only as opportunistic housekeeping | `035-04`; `035-07`; explicit phase boundary | Validated mapped |
| `Final Recommendation` | Phase 035 keeps substantive implementation focused on `035-4-fix-spec.md` and `035-5-fix-spec.md` inside the canonical `035-TODO.md` inventory plus the six mapped source documents | execution rules; `035-02`; `Completion Gate` | Validated mapped |

## 🔗 Master Source Coverage

This table proves that the full restored backlog now covers every live Phase
035 source file, not only the deferred-intake front block.

| Phase 035 source | Scope lane | Task coverage | Status |
| --- | --- | --- | --- |
| `035-a1-deferred.md` | deferred-intake boundary and scope honesty | `035-01..035-07` | Fully mapped |
| `035-a2-suffixes.md` | suffix inventory, production-head cleanup guidance, and rename handoff | `035-08..035-14` | Fully mapped |
| `035-a3-garbage-filter.md` | garbage-removal lane versus compatibility-live keep-set | `035-15..035-21` | Fully mapped |
| `035-a4-fix-spec.md` | sender workflow canonicalization | `035-22..035-31` | Fully mapped |
| `035-a5-fix-spec.md` | stealth gap triage and canonical additions | `035-32..035-40` | Fully mapped |
| `035-a6-renames.md` | curated file and signature rename execution | `035-41..035-49` | Fully mapped |

## 🚫 Explicit Phase Boundary

The following topics are intentionally out of scope for default Phase 035
execution:

- any live required deferred import from Phases 015, 026, 027, 030, or 031,
  because those historical carry-forward items are already resolved;
- any live required deferred import from Phases 029 or 033, because their
  remaining blocker wording is stale or superseded rather than execution-
  authoritative;
- treating the surviving Phase 032 `keep_path(...)` complexity candidate as
  anything other than a weak, non-native, optional housekeeping sidecar;
- old vendor-doctest caveats or superseded blocker wording from stale deferred
  notes;
- already-resolved release-gate, validation, or continuation leftovers;
- treating `crates/z00z_storage/src/assets/store_internal/store_query.rs`
  `keep_path(...)` complexity cleanup as sender canonicalization, stealth gap
  closure, or a mandatory Phase 035 acceptance item.

## ⚙️ Concrete Execution Tasks

### 035-01 Canonical Deferred-Intake Freeze

Spec references:

- `Purpose`
- `Intake Decision`
- `Final Recommendation`

MANDATORY pre-read in `035-1-deferred.md`:

- section `Purpose`
- section `Intake Decision`
- section `Final Recommendation`

- [x] Freeze the Phase 035 default rule that no historical deferred item is
  imported automatically.
- [x] Make the zero-import rule explicit in the Phase 035 execution baseline so
  later work does not reopen earlier deferred ledgers by habit.
- [x] Record that any new historical-deferred intake requires a source update to
  `035-1-deferred.md` before implementation or closeout artifacts change.

Files:

- `.planning/phases/035-mix2-fixes/035-1-deferred.md`
- `.planning/phases/035-mix2-fixes/035-CONTEXT.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`
- `.planning/STATE.md`

Tests:

- [x] review `035-TODO.md` to confirm the execution rules, dependency chain,
  and completion gate do not claim any required inherited deferred item
- [x] verify no numbered task in `035-TODO.md` requires reopening older
  `deferred-items.md` artifacts

Exit condition:

- Phase 035 can start without consulting any historical deferred ledger beyond
  `035-1-deferred.md`.

### 035-02 Live Phase-Source Binding

Spec references:

- `Recommended Phase 035 Scope Boundary`
- `Why Phase 035 Should Stay Clean`
- `Final Recommendation`

MANDATORY pre-read in `035-1-deferred.md`:

- section `Recommended Phase 035 Scope Boundary`
- section `Why Phase 035 Should Stay Clean`
- section `Final Recommendation`

MANDATORY supporting pre-read:

- `035-4-fix-spec.md` sections `Objective`, `Verified Gaps To Fix`, and
  `Target Architecture`
- `035-5-fix-spec.md` sections `Objective`, `Verified short additions worth
  doing now`, and `Verified non-short additions`

- [x] Bind every non-optional Phase 035 task to the canonical `035-TODO.md`
  inventory plus the six mapped Phase 035 source documents, and bind
  substantive implementation tasks to the live sender or stealth source
  documents instead of to historical deferred notes.
- [x] Keep the deferred-intake document as a boundary artifact only, while
  `035-4-fix-spec.md` and `035-5-fix-spec.md` remain the substantive phase
  implementation authorities inside that fixed source surface.
- [x] Reject any task wording that tries to smuggle unrelated storage,
  doctest, or release residue into sender or stealth closure or into the fixed
  canonical `035-TODO.md` inventory plus the six mapped source documents.

Files:

- `.planning/phases/035-mix2-fixes/035-1-deferred.md`
- `.planning/phases/035-mix2-fixes/035-CONTEXT.md`
- `.planning/phases/035-mix2-fixes/035-4-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-5-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] review all non-optional task headings in `035-TODO.md` to confirm they
  map either to one of the six canonical Phase 035 source documents or, for
  substantive implementation tasks, to sender canonicalization or stealth
  additions from the live fix specs
- [x] verify no non-optional Phase 035 task cites a historical deferred source
  as its implementation authority

Exit condition:

- all mandatory Phase 035 work is traceable to the canonical `035-TODO.md`
  inventory plus the six mapped Phase 035 source documents, and all
  substantive implementation work is traceable only to the live Phase 035
  sender and stealth specs.

### 035-03 Historical Triage Lock-In

Spec references:

- `Canonical Triage Of Historical Deferred Sources`
- `Recommended Phase 035 Scope Boundary`

MANDATORY pre-read in `035-1-deferred.md`:

- section `Canonical Triage Of Historical Deferred Sources`
- section `Recommended Phase 035 Scope Boundary`

- [x] Lock the triage truth that Phases 015, 026, 027, 029, 030, 031, and 033
  contribute no live required deferred item to Phase 035.
- [x] Preserve the classification that the surviving Phase 032 item is only a
  weak, non-native complexity debt candidate.
- [x] Keep the validation matrix and explicit phase boundary aligned with the
  triage table so later closeout does not overstate inherited scope.

Files:

- `.planning/phases/035-mix2-fixes/035-1-deferred.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] review the validation matrix and explicit phase boundary to confirm every
  historical source is either excluded or labeled optional-only
- [x] verify there is no unresolved historical-source ambiguity left in the
  backlog text

Exit condition:

- the historical deferred triage is fully encoded in the Phase 035 execution
  plan and no source remains semantically ambiguous.

### 035-04 Optional `keep_path(...)` Sidecar Gate

Spec references:

- `Optional Exception Rule`
- `Why Phase 035 Should Stay Clean`

MANDATORY pre-read in `035-1-deferred.md`:

- section `Optional Exception Rule`
- section `Why Phase 035 Should Stay Clean`

- [x] Keep `keep_path(...)` outside the default Phase 035 execution lane.
- [x] If a future planning pass explicitly opts in, attach `keep_path(...)`
  only as opportunistic housekeeping with separate wording and separate closeout
  semantics.
- [x] Do not let the optional sidecar compete with sender-workflow or
  stealth-hardening tasks for phase acceptance.

Files:

- `.planning/phases/035-mix2-fixes/035-TODO.md`
- `crates/z00z_storage/src/assets/store_internal/store_query.rs`

Tests:

- [x] if the sidecar is not activated, verify no mandatory task or completion
  rule depends on `keep_path(...)`
- [x] if the sidecar is activated, verify it is labeled optional or
  housekeeping-only in all affected planning artifacts

Exit condition:

- `keep_path(...)` is either absent from Phase 035 execution or isolated as a
  clearly optional sidecar that cannot be mistaken for semantic closure.

### 035-05 Phase Closeout Honesty Rules

Spec references:

- `Intake Decision`
- `Why Phase 035 Should Stay Clean`
- `Final Recommendation`

MANDATORY pre-read in `035-1-deferred.md`:

- section `Intake Decision`
- section `Why Phase 035 Should Stay Clean`
- section `Final Recommendation`

- [x] Define closeout wording that counts sender and stealth work from the live
  fix specs as the substantive implementation completion signal while keeping
  the other canonical Phase 035 lanes bound to their own six-source controls.
- [x] Keep optional housekeeping, stale-doctest caveats, and resolved legacy
  leftovers out of mandatory acceptance language.
- [x] Require future summary and validation artifacts to preserve the same
  no-import and optional-sidecar truth.

Files:

- `.planning/phases/035-mix2-fixes/035-TODO.md`
- `.planning/phases/035-mix2-fixes/035-1-deferred.md`

Tests:

- [x] review the completion gate and task exit conditions to confirm they do
  not count optional `keep_path(...)` work toward Phase 035 semantic closure
- [x] verify closeout language names sender and stealth work as the only
  substantive implementation lane while preserving the fixed six-source phase
  authority surface

Exit condition:

- future Phase 035 closeout cannot honestly pass unless it stays aligned with
  the deferred-intake boundary, the canonical `035-TODO.md` inventory plus the
  six mapped source documents, and the live fix specs.

## 🧪 Concrete Test Execution Tasks

### 035-06 Deferred-Consistency Validation Wave

Spec references:

- `Canonical Triage Of Historical Deferred Sources`
- `Final Recommendation`

MANDATORY pre-read in `035-1-deferred.md`:

- section `Canonical Triage Of Historical Deferred Sources`
- section `Final Recommendation`

- [x] Run a planning-artifact consistency sweep across `035-1-deferred.md`,
  `035-4-fix-spec.md`, `035-5-fix-spec.md`, and `035-TODO.md`.
- [x] Confirm Phase 035 wording does not claim inherited historical deferred
  work beyond the explicit optional-sidecar exception.
- [x] Confirm the backlog still routes substantive execution only through the
  live sender and stealth specs.

Files:

- `.planning/phases/035-mix2-fixes/035-1-deferred.md`
- `.planning/phases/035-mix2-fixes/035-4-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-5-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] `rg -n "Phase 015 deferred|Phase 026 deferred|Phase 027 deferred|Phase 029 deferred|Phase 030 deferred|Phase 031 deferred|Phase 033 deferred|vendor-doctest|keep_path" .planning/phases/035-mix2-fixes`
- [x] manually review hits to verify they only appear in triage, boundary, or
  optional-sidecar contexts

Exit condition:

- the Phase 035 planning set contains no hidden historical deferred import and
  no stale-deferred wording masquerades as live work.

### 035-07 Optional Sidecar Validation Gate

Spec references:

- `Optional Exception Rule`

MANDATORY pre-read in `035-1-deferred.md`:

- section `Optional Exception Rule`

- Active sidecar branch applies in the current live tree because staged
  `store_query.rs` and `test_search_api.rs` changes already carry the optional
  `keep_path(...)` cleanup.
- [x] If `keep_path(...)` is activated, confirm every affected artifact labels
  it housekeeping-only and excludes it from stealth-gap or sender-gap closure.
- [x] Confirm no sender or stealth regression suite depends on the optional
  sidecar for a green closeout story.

Files:

- `.planning/phases/035-mix2-fixes/035-TODO.md`
- `crates/z00z_storage/src/assets/store_internal/store_query.rs`

Tests:

- The inactive-sidecar checklist is not the truthful branch for this validation
  wave because the optional local refactor is already attached in the live
  tree.
- [x] if the sidecar is active, verify `store_query.rs` changes are referenced
  only from optional-sidecar language and not from mandatory sender or stealth
  tasks

Exit condition:

- the optional sidecar state is unambiguous and cannot distort Phase 035
  semantic acceptance.

## ✅ Completion Gate

This backlog is complete only when all of the following hold:

- every mandatory Phase 035 task stays anchored to the canonical
  `035-TODO.md` inventory plus the six mapped Phase 035 source documents, and
  every substantive implementation task stays anchored to `035-4-fix-spec.md`
  or `035-5-fix-spec.md` rather than to historical deferred ledgers;
- sender workflow completion from `035-4-fix-spec.md` and stealth completion
  from `035-5-fix-spec.md` remain the only substantive implementation signals
  that later validation or closeout may count toward semantic Phase 035
  closure;
- `035-1-deferred.md`, `035-TODO.md`, and the future Phase 035 closeout
  artifacts still agree that no historical deferred item was imported by
  default;
- any `keep_path(...)` work is either absent or explicitly labeled as
  opportunistic housekeeping outside semantic Phase 035 closure;
- no stale vendor-doctest or resolved legacy blocker wording is promoted as
  live Phase 035 scope.

## ➕ Appended Backlog - 035-2 Suffix Inventory

Canonical design source:

- [035-2-suffixes](./035-2-suffixes.md)

Execution rules:

- treat `035-2-suffixes.md` as the canonical authority for this suffix block:
  syntax-first inventory, workspace-backed `production-current` versus
  `reserved-future` interpretation, and merged production-head cleanup
  guidance;
- keep declaration ownership explicit; do not group rows by bare identifier
  alone when distinct modules or types define the same suffix-bearing name;
- keep production/API rows, test-only rows, filename rows, and exclusion-only
  grep noise in separate lanes;
- target one default cleanup outcome: canonical surviving Rust-facing
  `production-current` surfaces should converge to unsuffixed names on the
  default path;
- keep suffix-bearing compatibility rows only when the canonical source still
  marks them as `reserved-future` support or as explicit review candidates;
- route concrete code renames and retirement decisions through the curated
  rename lane in `035-a6-renames.md`; do not treat raw inventory presence as an
  auto-approved rename order;
- before starting any numbered task in this appended block, complete its
  `MANDATORY pre-read` block.

### 🎯 Appended Decision Summary

The execution baseline for the appended suffix-inventory block is:

1. keep `035-2-suffixes.md` as the single canonical source for suffix
   inventory, production-head interpretation, and cleanup guidance;
2. preserve declaration-backed grouping and path-backed provenance so repeated
   names such as `VERSION_V1` or `VERSION_V2` are not merged across unrelated
   modules or types;
3. keep test-only rows, filename rows, and explicit exclusions visible instead
   of flattening them into one rename bucket;
4. converge default `production-current` Rust-facing survivors to unsuffixed
   canonical names;
5. keep `reserved-future` compatibility rows only when the canonical source
   still proves they are live or deliberately under review.

### 🔗 Appended Dependency Chain

Execution dependency chain:

1. `035-08` suffix authority freeze
2. `035-09` declaration-backed inventory lock-in
3. `035-10` production-head cleanup target
4. `035-11` filename and exclusion hygiene
5. `035-12` curated rename and retirement handoff
6. `035-13` suffix inventory validation wave
7. `035-14` suffix cleanup readiness gate

Hard dependencies:

- `035-09` depends on `035-08`
- `035-10` depends on `035-08` and `035-09`
- `035-11` depends on `035-09`
- `035-12` depends on `035-10` and `035-11`
- `035-13` depends on `035-08` through `035-12`
- `035-14` depends on `035-12` and `035-13`

### 🗂️ Appended File-First Implementation Order

Edit order by file cluster:

1. `.planning/phases/035-mix2-fixes/035-2-suffixes.md`
2. `.planning/phases/035-mix2-fixes/035-a6-renames.md`
3. `.planning/phases/035-mix2-fixes/035-TODO.md`
4. only after explicit rename activation: repository files selected by the
   canonical suffix inventory and curated rename plan

### ✅ Appended Validation Matrix

This table proves that the suffix-inventory source has been translated into an
execution backlog without reintroducing the older split-authority drift.
For `035-08..035-14`, `035-2-suffixes.md` is the sole suffix-lane execution
authority; older split-authority notes remain historical context only.

| 035-2-suffixes source section | Required theme | TODO coverage | Status |
| --- | --- | --- | --- |
| `Objective` | collect a source-backed inventory of suffix-bearing signature and filename surfaces, plus cleanup guidance | execution rules; `035-08`; `035-09`; `035-11` | Validated mapped |
| `Collection Rules` | inventory remains declaration-backed and excludes usage-only noise | decision summary 2-3; `035-09`; `035-11`; `035-13` | Validated mapped |
| `Non-Test Signature Surfaces` | live signature rows need durable declaration ownership and path-backed provenance | `035-09`; `035-10`; `035-13` | Validated mapped |
| `Test-Only Signature Surfaces` | test-only rows stay visible without becoming default production rename targets | `035-09`; `035-13`; explicit phase boundary | Validated mapped |
| `Fixed Table` | `production-current` rows converge toward unsuffixed canonical names while `reserved-future` rows stay justified or retired intentionally | execution rules; `035-08`; `035-10`; `035-12`; `035-14` | Validated mapped |
| `Cleanup Interpretation And Production-Head Guidance` | cleanup intent, guardrails, family rules, and corrected-row quarantine rules remain part of execution authority | `035-08`; `035-10`; `035-12`; `035-13`; `035-14` | Validated mapped |
| `Stale Or Corrected Inventory Rows` | corrected rows stay outside suffix-lane authority and cannot drift back into rename scope | `035-09`; `035-13`; explicit phase boundary | Validated mapped |
| `Filename Surfaces` | filename inventory remains a separate lane from signature rows | `035-11`; `035-13` | Validated mapped |
| `Explicit Exclusions` | usage-only, comment-only, temporary, and hidden-path artifacts stay out of primary rows | execution rules; `035-11`; explicit phase boundary | Validated mapped |
| `Bottom Line` | suffix inventory, production-head interpretation, and cleanup guidance now live in one canonical document | decision summary 1; `035-08`; `035-12`; `035-14`; appended completion gate | Validated mapped |

### 🚫 Appended Explicit Phase Boundary

The following topics are intentionally out of scope for this appended suffix
block unless a later source explicitly widens scope:

- blind deletion of compatibility readers, import paths, migration helpers, or
  backup wire structs that the canonical source still marks as live support or
  review candidates;
- collapsing generic names such as `VERSION_V1` or `VERSION_V2` across modules
  without declaration-backed ownership proof;
- reintroducing corrected rows such as `Argon2idParamsV1`, `ClaimStmtV2`,
  `ClaimAuthoritySigV2`, or `ClaimSourceProof` into primary suffix or rename
  authority after the live code already moved to unsuffixed surfaces;
- promoting usage-only grep hits, local temporaries, comment labels, or hidden
  planning-path artifacts into primary suffix inventory rows;
- treating the suffix inventory itself as proof that any semantic wallet,
  storage, checkpoint, claim, or backup hardening is complete.

### ⚙️ Appended Concrete Execution Tasks

### 035-08 Suffix Authority Freeze

Spec references:

- `Objective`
- `Fixed Table`
- `Cleanup Interpretation And Production-Head Guidance`
- `Bottom Line`

MANDATORY pre-read in `035-2-suffixes.md`:

- section `Objective`
- section `Fixed Table`
- section `Cleanup Interpretation And Production-Head Guidance`
- section `Bottom Line`

- [x] Freeze the rule that `035-2-suffixes.md` is the canonical authority for
  suffix inventory, production-head interpretation, and cleanup guidance.
- [x] Make that authority explicit anywhere the Phase 035 backlog refers to
  suffix work so later cleanup does not point back to superseded semantic
  split documents.
- [x] Preserve the distinction between syntax-first collection, workspace-backed
  classification, and curated rename execution.

Files:

- `.planning/phases/035-mix2-fixes/035-2-suffixes.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] review `035-TODO.md` to confirm no suffix-retirement or rename wording
  still points to any superseded split-authority suffix source
- [x] verify the appended suffix block now describes `035-2-suffixes.md` as
  the canonical authority for this lane

Exit condition:

- the appended suffix backlog cannot be misread as an older split-authority
  cleanup plan.

### 035-09 Declaration-Backed Inventory Lock-In

Spec references:

- `Collection Rules`
- `Non-Test Signature Surfaces`
- `Test-Only Signature Surfaces`
- `Stale Or Corrected Inventory Rows`

MANDATORY pre-read in `035-2-suffixes.md`:

- section `Collection Rules`
- section `Non-Test Signature Surfaces`
- section `Test-Only Signature Surfaces`
- subsection `Stale Or Corrected Inventory Rows`

- [x] Keep signature rows declaration-backed and path-backed rather than grouped
  by bare symbol name alone.
- [x] Preserve separate visibility for non-test rows and test-only rows without
  flattening them into the same semantic bucket.
- [x] Recheck repeated names such as `VERSION_V1`, `VERSION_V2`, and similar
  constants against their owning type or module path.
- [x] Keep corrected rows such as `Argon2idParamsV1` and the unsuffixed claim
  structs out of suffix-lane execution authority when the live code already
  moved.

Files:

- `.planning/phases/035-mix2-fixes/035-2-suffixes.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] review repeated generic rows in `035-2-suffixes.md` and confirm each one
  still has declaration-backed path ownership
- [x] verify no corrected row is silently reintroduced into rename scope by the
  appended backlog wording

Exit condition:

- the suffix inventory remains declaration-backed, path-backed, and immune to
  generic-name conflation or corrected-row drift.

### 035-10 Production-Head Cleanup Target

Spec references:

- `Fixed Table`
- `Cleanup Interpretation And Production-Head Guidance`

MANDATORY pre-read in `035-2-suffixes.md`:

- section `Fixed Table`
- section `Cleanup Interpretation And Production-Head Guidance`

- [x] Preserve the suffix-status split between `production-current`,
  `reserved-future`, and `single surviving V1 review` rows exactly as the
  canonical source now states it.
- [x] Plan the cleanup target so `production-current` Rust-facing surfaces land
  on unsuffixed canonical names on the default path.
- [x] Keep rows such as backup wire structs, compatibility readers, and legacy
  import helpers honest when they are still part of live read, open, import, or
  migration support.
- [x] Retire or demote suffix-bearing rows only when the canonical source now
  classifies them as non-default ballast rather than live compatibility support
  or explicit review candidates.

Files:

- `.planning/phases/035-mix2-fixes/035-2-suffixes.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] review a representative set of `production-current`, `reserved-future`,
  and single surviving `V1` rows to confirm the appended backlog preserves the
  current canonical distinction
- [x] verify the appended suffix block states that default canonical survivors
  should be unsuffixed `production-current` surfaces

Exit condition:

- later suffix cleanup work has a source-backed end state: canonical survivors
  are unsuffixed `production-current` surfaces, and any retained suffix-bearing
  rows are compatibility-only, explicit review candidates, or explicit
  public-contract exceptions.

### 035-11 Filename And Exclusion Hygiene

Spec references:

- `Filename Surfaces`
- `Explicit Exclusions`

MANDATORY pre-read in `035-2-suffixes.md`:

- section `Filename Surfaces`
- section `Explicit Exclusions`

- [x] Preserve filename surfaces as a dedicated inventory lane separate from
  Rust signature definitions.
- [x] Keep hidden paths, usage-only tokens, temporary values, comment labels,
  and non-owning string fragments excluded from primary inventory rows unless a
  later source widens scope.
- [x] Ensure the explicit RPC method string remains the only promoted
  string-contract exception unless the source adds another one.

Files:

- `.planning/phases/035-mix2-fixes/035-2-suffixes.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] verify the filename list stays limited to non-hidden repository paths
  carrying real suffix-bearing filenames
- [x] verify the exclusion rules still name usage-only, comment-only, local,
  temporary, and hidden-path artifacts as non-primary inventory material

Exit condition:

- filename inventory and exclusion policy remain explicit and cannot drift into
  grep-noise driven cleanup.

### 035-12 Curated Rename And Retirement Handoff

Spec references:

- `Cleanup Interpretation And Production-Head Guidance`
- `Bottom Line`

MANDATORY pre-read in `035-2-suffixes.md`:

- section `Cleanup Interpretation And Production-Head Guidance`
- section `Bottom Line`

MANDATORY supporting pre-read:

- `.planning/phases/035-mix2-fixes/035-a6-renames.md` sections `Summary`,
  `Suffix-Lane Handoff`, and `Doublecheck`

- [x] Keep suffix inventory maintenance separate from curated rename planning.
- [x] Route concrete `production-current` renames into `035-a6-renames.md`, with
  the explicit target that canonical surviving names no longer carry `V1..Vn`
  suffixes on the default path.
- [x] Route review-candidate or compatibility-lane retirement into the same
  curated lane only after the canonical source says the rows no longer need to
  survive as live support.
- [x] Preserve the rule that rename execution must be justified by canonical
  suffix guidance and curated review, not by raw inventory presence alone.

Files:

- `.planning/phases/035-mix2-fixes/035-2-suffixes.md`
- `.planning/phases/035-mix2-fixes/035-a6-renames.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] verify rename handoff wording targets unsuffixed canonical
  `production-current` surfaces rather than inventory-only preservation
- [x] verify suffix-bearing compatibility rows are still described as conditional
  keep-or-retire decisions rather than blanket deletion

Exit condition:

- suffix inventory now feeds a concrete cleanup target while still requiring a
  separate curated lane for actual renames and safe retirement.

### 🧪 Appended Concrete Test Execution Tasks

### 035-13 Suffix Inventory Validation Wave

Spec references:

- `Collection Rules`
- `Fixed Table`
- `Cleanup Interpretation And Production-Head Guidance`
- `Filename Surfaces`
- `Explicit Exclusions`

MANDATORY pre-read in `035-2-suffixes.md`:

- section `Collection Rules`
- section `Fixed Table`
- section `Cleanup Interpretation And Production-Head Guidance`
- section `Filename Surfaces`
- section `Explicit Exclusions`

- [x] Run a consistency sweep across `035-2-suffixes.md`, `035-a6-renames.md`,
  and the appended suffix block in `035-TODO.md`.
- [x] Confirm generic repeated symbols remain declaration-backed and are not
  conflated across modules or types.
- [x] Confirm filename rows, test-only rows, exclusion rules, and corrected-row
  removals are preserved as separate truths.
- [x] Confirm the appended block now encodes the correct cleanup target:
  unsuffixed canonical `production-current` survivors, with conditional keep or
  retire handling for suffix-bearing compatibility rows.

Files:

- `.planning/phases/035-mix2-fixes/035-2-suffixes.md`
- `.planning/phases/035-mix2-fixes/035-a6-renames.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] `rg -n "VERSION_V1|VERSION_V2|BackupContainerV1|BackupEncryptionV1|BackupAssociatedDataV1|BackupCompressionV1|wallet.key.export_public_material_v2" .planning/phases/035-mix2-fixes/035-2-suffixes.md .planning/phases/035-mix2-fixes/035-a6-renames.md .planning/phases/035-mix2-fixes/035-TODO.md`
- [x] manually review hits to verify repeated names remain declaration-backed
  and backup-wire V1 rows are not downgraded into legacy-unused wording without
  explicit canonical support

Exit condition:

- the suffix inventory and its cleanup target are explicitly validated as
  aligned, declaration-backed, and anti-drift.

### 035-14 Suffix Cleanup Readiness Gate

Spec references:

- `Bottom Line`

MANDATORY pre-read in `035-2-suffixes.md`:

- section `Bottom Line`

- [x] If no rename lane is activated, confirm the appended suffix block closes
  as inventory-plus-guidance planning work.
- [x] If a rename lane is activated later, confirm every suffix-lane candidate still routes
  through `035-2-suffixes.md` and `035-a6-renames.md` before code changes.
- [x] Confirm the rename-ready target is precise: canonical surviving
  Rust-facing surfaces should be unsuffixed `production-current` names, while
  any surviving suffix-bearing rows must be compatibility-only or explicit
  review candidates.
- [x] Confirm no semantic completion claim for sender, stealth, wallet,
  checkpoint, backup, or claim behavior depends on suffix cleanup alone.

Files:

- `.planning/phases/035-mix2-fixes/035-2-suffixes.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`
- `.planning/phases/035-mix2-fixes/035-a6-renames.md`

Tests:

- [x] if the rename lane is inactive, verify no execution checklist in the
  appended suffix block requires source-file renames
- [x] if the rename lane is active, verify every suffix-lane candidate cites the
  canonical suffix source and curated rename lane rather than raw inventory
  alone
- [x] verify no checklist claims that suffix-bearing compatibility rows must be
  deleted without a source-backed keep-or-retire decision

Exit condition:

- the suffix-inventory backlog cannot be mistaken for semantic completion or an
  auto-approved blanket deletion program.

### ✅ Appended Completion Gate

This appended suffix block is complete only when all of the following hold:

- `035-2-suffixes.md` remains the canonical authority for suffix inventory,
  workspace-backed production-head interpretation, and cleanup guidance;
- the cleanup target is explicit: canonical surviving Rust-facing surfaces are
  unsuffixed `production-current` names on the default path;
- declaration-backed ownership, filename inventory, corrected-row handling, and
  explicit exclusions all remain visible and uncollapsed in the planning story;
- any future rename, collapse, or retirement work is routed into the curated
  rename lane and is not implied by raw inventory presence alone;
- any remaining suffix-bearing row after cleanup is either a proven
  compatibility-live support surface, an explicit review candidate, or retired
  from the default path;
- no suffix cleanup claim is used as proof of semantic closure for unrelated
  wallet, storage, checkpoint, claim, or stealth work.

## ➕ Appended Backlog - 035-3 Garbage Filter

Canonical design source:

- [035-3-garbage-filter](./035-3-garbage-filter.md)

Execution rules:

- treat `035-3-garbage-filter.md` as the semantic authority for what belongs in
  the immediate garbage-removal lane;
- treat rows explicitly labeled `GARBAGE` as the only default immediate
  retirement candidates in this appended block;
- treat `DEBUG-ONLY` rows as a separate reviewed non-production cluster that
  requires an explicit cluster-level retirement decision before any delete
  wave;
- treat `InProduction = TRUE` rows as out of the immediate delete lane even
  when the names look legacy, because the source says they are still active
  current or compatibility/migration paths proven by live callers;
- do not silently delete live migration decoders, legacy wallet KDF ladders,
  backup compatibility readers, or legacy claim proof surfaces just because a
  stricter `current production-path only` target would prefer them gone;
- if the repository owner wants a tree that keeps only the default current
  production path, update the canonical source first so the affected
  compatibility or migration rows stop being authoritative keep-set entries,
  then regenerate this backlog block;
- before starting any numbered task in this appended block, complete its
  `MANDATORY pre-read` block.

### 🎯 Appended Garbage Decision Summary

The execution baseline for the appended garbage-filter block is:

1. `035-3-garbage-filter.md` already separates immediate garbage from
   compatibility-live rows through `InProduction` and comment labels;
2. the first deletion wave is limited to the narrow hard-garbage set:
  `LegacyProofBlob`, the top-level `ArtWire` shell in
  `crates/z00z_storage/src/checkpoint/ids.rs`, and
  `_keep_checkpoint_draft`;
3. the debug-dump pipeline (`debug_export_wallet`, `verify_debug_wallets`,
   `enrich_debug_dump_with_assets`) is non-production and may be retired only
   as one reviewed cluster so Scenario 1 verification fallout stays explicit;
4. live compatibility and migration surfaces such as `ClaimNullRecV0`,
   `derive_key_v1_repetition_padding`, `derive_wallet_keys_v1`,
   `GenesisClaimStatement`, `statement_hash`, `ClaimAuthoritySig`, and
   `GenesisClaimProof` remain outside the delete lane until a canonical source
   update demotes them;
5. the stronger target `leave only current production-path` is real design
   intent from the user input, but it is blocked by the current canonical table
   and therefore must be carried as source drift rather than smuggled into an
   immediate removal checklist.

### 🔗 Appended Garbage Dependency Chain

Execution dependency chain:

1. `035-15` garbage classification freeze
2. `035-16` hard-garbage removal cluster
3. `035-17` debug-dump retirement review
4. `035-18` compatibility and migration keep-set freeze
5. `035-19` current-path-only source drift handoff
6. `035-20` garbage-filter validation wave
7. `035-21` current-path closure gate

Hard dependencies:

- `035-16` depends on `035-15`
- `035-17` depends on `035-15`
- `035-18` depends on `035-15`
- `035-19` depends on `035-18`
- `035-20` depends on `035-16` through `035-19`
- `035-21` depends on `035-19` and `035-20`

### 🗂️ Appended Garbage File-First Implementation Order

Edit order by file cluster:

1. `crates/z00z_storage/src/assets/store_internal/test_whitebox_proofs.rs`
2. `crates/z00z_storage/src/checkpoint/ids.rs`
3. `crates/z00z_wallets/src/core/tx/state_checkpoint.rs`
4. `crates/z00z_wallets/src/db/redb_wallet_store_debug_export.rs`
5. `crates/z00z_simulator/src/scenario_1/runner_verify.rs`
6. `crates/z00z_simulator/src/scenario_1/stage_3_utils/post_claim.rs`
7. `crates/z00z_crypto/src/claim/statement.rs`
8. `crates/z00z_crypto/src/claim/proof.rs`
9. `crates/z00z_storage/src/assets/store_internal/redb_backend_state.rs`
10. `crates/z00z_wallets/src/core/backup/wallet_backup.rs`
11. `crates/z00z_wallets/src/db/redb_wallet_crypto_kdf_helpers.rs`
12. `.planning/phases/035-mix2-fixes/035-3-garbage-filter.md`
13. `.planning/phases/035-mix2-fixes/035-TODO.md`

### 📋 Appended Garbage Validation Matrix

| Source section | Implementation meaning | TODO coverage | Status |
| --- | --- | --- | --- |
| `Classification Rules` | only `FALSE` / `GARBAGE` rows enter the default immediate removal lane; `DEBUG-ONLY` stays a reviewed non-production cluster | execution rules; `035-15`; `035-16`; `035-17` | Validated mapped |
| table rows `LegacyProofBlob`, `ArtWire` (top-level `crates/z00z_storage/src/checkpoint/ids.rs` shell), `_keep_checkpoint_draft` | narrow hard-garbage deletion wave for the audited source-file shells only | `035-16`; `035-20`; `035-21` | Validated narrow hard-garbage cluster |
| table rows `debug_export_wallet`, `verify_debug_wallets`, `enrich_debug_dump_with_assets` | reviewed non-production pipeline retirement | `035-17`; `035-20`; `035-21` | Validated deferred simulator-backed cluster |
| table rows `GenesisClaimStatement`, `statement_hash`, `ClaimAuthoritySig`, `GenesisClaimProof`, `ClaimNullRecV0`, `BackupContainer`, `derive_key_with_kdf`, `Argon2idParams`, `derive_key_v1_repetition_padding`, `derive_wallet_keys_v1` | `InProduction = TRUE` compatibility or migration keep-set must not be misread as garbage | `035-18`; `035-19`; `035-20`; `035-21` | Validated explicit keep-set |
| table row `derive_key_legacy_v1` | support-only backup API lane needs explicit API review before any removal plan | `035-18`; `035-19`; `035-20`; `035-21` | Review-only support lane |
| `Filtering Notes` | no suffix-age-only or legacy-name-only deletion; strongest keep-set remains in storage and wallets | execution rules; `035-15`; `035-18`; `035-20` | Validated keep-set guardrail |
| user target `only current production-path` | blocked until canonical source demotes live compatibility or migration rows | `035-19`; `035-21` | Validated source-drift closure gate |

### 🚫 Appended Garbage Explicit Phase Boundary

The following topics are intentionally out of scope for this appended garbage
block unless the canonical source is updated first:

- blanket deletion of every legacy-named row in `035-3-garbage-filter.md`;
- retirement of `InProduction = TRUE` compatibility or migration rows such as
  `ClaimNullRecV0`, `derive_key_v1_repetition_padding`, or
  `derive_wallet_keys_v1` without a source update that demotes them;
- deletion of legacy claim proof surfaces while the canonical table still says
  they are exported or used by live prover or verifier paths;
- silent collapse of support-only APIs such as `derive_key_legacy_v1` into the
  garbage lane without the API review that the source explicitly calls for;
- using this block as proof that sender, stealth, checkpoint, claim, wallet,
  or migration semantics are fully modernized.

Exit condition:

- the appended garbage block cannot be misread as a blanket
  `current-production-only` deletion order while the canonical source still
  says otherwise.

### 035-15 Garbage Classification Freeze

Spec references:

- `Classification Rules`
- `Filtering Notes`

MANDATORY pre-read in `035-3-garbage-filter.md`:

- section `Classification Rules`
- section `Filtering Notes`

- [x] Freeze the meaning of `InProduction = TRUE` versus `InProduction = FALSE`
  for this phase so legacy-looking names do not get silently upgraded into the
  delete lane.
- [x] Record that `FALSE` plus `GARBAGE` drives default immediate removal,
  while `DEBUG-ONLY` stays a reviewed non-production cluster and `TRUE` drives
  keep-set review only.
- [x] Carry the user target `leave only current production-path` as explicit
  source drift rather than flattening it into a fake immediate-delete rule.

Files:

- `.planning/phases/035-mix2-fixes/035-3-garbage-filter.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] verify every row used in later tasks still matches the source-side
  `InProduction` value and comment classification
- [x] verify no later checklist says `delete because legacy-looking` without a
  matching `FALSE` plus `GARBAGE` basis or an explicit reviewed debug-cluster
  decision

Exit condition:

- the removal lane, keep-set lane, and source-drift lane are explicit and
  cannot be confused.

### 035-16 Hard-Garbage Removal Cluster

Spec references:

- table row `LegacyProofBlob`
- table row `ArtWire` (top-level `crates/z00z_storage/src/checkpoint/ids.rs` shell)
- table row `_keep_checkpoint_draft`

MANDATORY pre-read in `035-3-garbage-filter.md`:

- table row `LegacyProofBlob`
- table row `ArtWire` (top-level `crates/z00z_storage/src/checkpoint/ids.rs` shell)
- table row `_keep_checkpoint_draft`

- [x] Treat the audited source-file shells for `LegacyProofBlob`, the
  top-level `ArtWire` shell in `crates/z00z_storage/src/checkpoint/ids.rs`,
  and `_keep_checkpoint_draft` as the first hard-garbage deletion wave.
- [x] Remove or inline away these rows only after confirming they do not feed a
  live production path beyond local whitebox, ID-test, or empty keepalive
  shells.
- [x] Keep the deletion wave narrow; do not expand it with additional
  legacy-looking rows that the source has not already demoted.

Files:

- `crates/z00z_storage/src/assets/store_internal/test_whitebox_proofs.rs`
- `crates/z00z_storage/src/checkpoint/ids.rs`
- `crates/z00z_wallets/src/core/tx/state_checkpoint.rs`
- `.planning/phases/035-mix2-fixes/035-3-garbage-filter.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] `rg -n "LegacyProofBlob|ArtWire|_keep_checkpoint_draft" crates/z00z_storage/src/assets/store_internal/test_whitebox_proofs.rs crates/z00z_storage/src/checkpoint/ids.rs crates/z00z_wallets/src/core/tx/state_checkpoint.rs`
- [x] manually review hits to confirm only the audited source-file shells in
  `src/assets/store_internal/test_whitebox_proofs.rs`,
  `src/checkpoint/ids.rs`, and `src/core/tx/state_checkpoint.rs` enter this
  removal wave; in `src/checkpoint/ids.rs` the removed top-level `ArtWire`
  shell is replaced only by the local `UnsupportedVersionArtWire`, while
  same-name test helpers outside that wave remain out of scope

Exit condition:

- the hard-garbage cluster is fully enumerated and removed or inlined away
  without touching live compatibility code.

### 035-17 Debug-Dump Retirement Review

Spec references:

- table row `debug_export_wallet`
- table row `verify_debug_wallets`
- table row `enrich_debug_dump_with_assets`
- `Filtering Notes`

MANDATORY pre-read in `035-3-garbage-filter.md`:

- table row `debug_export_wallet`
- table row `verify_debug_wallets`
- table row `enrich_debug_dump_with_assets`
- section `Filtering Notes`

- [x] Review the debug-dump trio as one non-production cluster rather than as
  isolated helpers.
- [x] Confirm the default production path does not depend on the cluster and
  that any remaining use is limited to simulator debug-dump emission,
  simulator verification, or post-claim inspection.
- [x] Since the cluster is not retired in this slice, defer it explicitly as
  one simulator-backed cluster and keep any future retirement tied to a
  deliberate simulator verification rewrite.

Files:

- `crates/z00z_wallets/src/db/redb_wallet_store_debug_export.rs`
- `crates/z00z_simulator/src/scenario_1/runner_verify.rs`
- `crates/z00z_simulator/src/scenario_1/stage_3_utils/post_claim.rs`
- `.planning/phases/035-mix2-fixes/035-3-garbage-filter.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] `rg -n "debug_export_wallet|verify_debug_wallets|enrich_debug_dump_with_assets" crates/z00z_wallets crates/z00z_simulator`
- [x] manually verify that every remaining caller is debug-only before the
  cluster enters a delete wave; the live hits stay bounded to
  `wallet_debug_tools` or `wallet_debug_dump` surfaces plus simulator
  verification and post-claim inspection

Exit condition:

- the debug-only pipeline is either explicitly queued for retirement as one
  cluster or explicitly deferred with a source-backed reason.

### 035-18 Compatibility And Migration Keep-Set Freeze

Spec references:

- table row `GenesisClaimStatement`
- table row `statement_hash`
- table row `ClaimAuthoritySig`
- table row `GenesisClaimProof`
- table row `ClaimNullRecV0`
- table row `BackupContainer`
- table row `derive_key_with_kdf`
- table row `derive_key_legacy_v1`
- table row `Argon2idParams`
- table row `derive_key_v1_repetition_padding`
- table row `derive_wallet_keys_v1`
- `Filtering Notes`

MANDATORY pre-read in `035-3-garbage-filter.md`:

- the table rows listed above
- section `Filtering Notes`

- [x] Freeze every `InProduction = TRUE` compatibility or migration row as out
  of the immediate delete lane.
- [x] Keep the storage and wallet migration ladders visible as the strongest
  source-backed keep-set in this block.
- [x] Route `derive_key_legacy_v1` into explicit API-review language instead of
  silently treating it as guaranteed garbage.
- [x] If a future current-production-only cleanup still wants these rows gone,
  require a canonical source update first.

Files:

- `crates/z00z_crypto/src/claim/statement.rs`
- `crates/z00z_crypto/src/claim/proof.rs`
- `crates/z00z_crypto/src/claim/v2.rs`
- `crates/z00z_storage/src/assets/store_internal/redb_backend_state.rs`
- `crates/z00z_wallets/src/core/backup/backup_wire.rs`
- `crates/z00z_wallets/src/core/backup/wallet_backup.rs`
- `crates/z00z_wallets/src/core/key/seed_cipher_params.rs`
- `crates/z00z_wallets/src/db/redb_wallet_crypto_kdf_helpers.rs`
- `.planning/phases/035-mix2-fixes/035-3-garbage-filter.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] `rg -n "GenesisClaimStatement|statement_hash|ClaimAuthoritySig|GenesisClaimProof|ClaimNullRecV0|BackupContainer|derive_key_with_kdf|derive_key_legacy_v1|Argon2idParams|derive_key_v1_repetition_padding|derive_wallet_keys_v1" crates/z00z_crypto crates/z00z_storage crates/z00z_wallets`
- [x] manually review hits to confirm the keep-set still corresponds to live
  prover, verifier, importer, open-session, or migration callers

Exit condition:

- live compatibility and migration surfaces cannot be mistaken for immediate
  garbage.

### 035-19 Current-Path-Only Source Drift Handoff

Spec references:

- `Classification Rules`
- `Filtering Notes`

MANDATORY pre-read in `035-3-garbage-filter.md`:

- section `Classification Rules`
- section `Filtering Notes`

- [x] Record the exact drift between the user target `leave only current
  production-path` and the current canonical table that still marks some
  compatibility or migration rows as `InProduction = TRUE`.
- [x] List the rows that would need a source-side demotion before a stronger
  current-production-only delete wave can be generated honestly.
- [x] Keep the handoff explicit: update the canonical source first, then
  regenerate this appended backlog block.

Files:

- `.planning/phases/035-mix2-fixes/035-3-garbage-filter.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] verify the appended block never claims that live compatibility or
  migration rows are already approved for deletion
- [x] verify the stronger current-production-only target is labeled as blocked
  by source drift rather than as active execution truth

Exit condition:

- the stronger cleanup target is preserved without falsifying the canonical
  source.

### 🧪 Appended Garbage Test Execution Tasks

### 035-20 Garbage-Filter Validation Wave

Spec references:

- `Classification Rules`
- `Filtering Notes`
- the audited table rows used by `035-16` through `035-19`

MANDATORY pre-read in `035-3-garbage-filter.md`:

- section `Classification Rules`
- section `Filtering Notes`

- [x] Run a consistency sweep across `035-3-garbage-filter.md` and the appended
  garbage block in `035-TODO.md`.
- [x] Confirm the hard-garbage cluster is still narrow and source-backed.
- [x] Confirm the debug-only cluster is treated as non-production but not
  silently conflated with the hard-garbage cluster.
- [x] Confirm the compatibility and migration keep-set remains explicitly out of
  the delete lane.

Files:

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/035-mix2-fixes/035-09-PLAN.md`
- `.planning/phases/035-mix2-fixes/035-09-SUMMARY.md`
- `.planning/phases/035-mix2-fixes/035-3-garbage-filter.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- [x] `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump`
- [x] `rg -n "LegacyProofBlob|ArtWire|_keep_checkpoint_draft|debug_export_wallet|verify_debug_wallets|enrich_debug_dump_with_assets|GenesisClaimStatement|ClaimNullRecV0|derive_key_v1_repetition_padding|derive_wallet_keys_v1" .planning/phases/035-mix2-fixes/035-3-garbage-filter.md crates/z00z_crypto crates/z00z_storage/src/assets/store_internal/test_whitebox_proofs.rs crates/z00z_storage/src/checkpoint/ids.rs crates/z00z_wallets/src/core/tx/state_checkpoint.rs crates/z00z_wallets/src/db/redb_wallet_store_debug_export.rs crates/z00z_wallets/src/core/backup/wallet_backup.rs crates/z00z_wallets/src/db/redb_wallet_crypto_kdf_helpers.rs crates/z00z_simulator/src/scenario_1/runner_verify.rs crates/z00z_simulator/src/scenario_1/stage_3_utils/post_claim.rs`
- [x] manually verify the keep-set rows still have live-caller evidence and the
  garbage rows do not
- [x] run the repeated read-only review loop across `ROADMAP.md`, `STATE.md`,
  `035-09-PLAN.md`, `035-09-SUMMARY.md`, `035-3-garbage-filter.md`, and
  `035-TODO.md` until two consecutive clean passes are reached
- [x] preserve the review-loop evidence explicitly: passes 1-5 blocked on
  proof-surface drift, stale continuity metadata, missing command-gate
  evidence, missing `035-21` review-loop evidence, and missing preserved
  run-count history before the first clean pass was accepted
- [x] preserve the clean closure records explicitly: pass 6 was the first
  clean review pass and pass 7 was the second consecutive clean review pass on
  the same six-file surface

Exit condition:

- the appended garbage block stays aligned with the audited table and cannot be
  read as overbroad cleanup.

### 035-21 Current-Path Closure Gate

Spec references:

- `Classification Rules`
- `Filtering Notes`

MANDATORY pre-read in `035-3-garbage-filter.md`:

- section `Classification Rules`
- section `Filtering Notes`

- [x] If the canonical source is unchanged, confirm this appended block closes
  only on hard-garbage removal plus explicit debug-cluster deferral, not on
  blanket compatibility retirement.
- [x] If the canonical source is later updated to current-production-only
  semantics, regenerate this block instead of stretching the old one.
- [x] Confirm no phase-level closure statement claims that current production is
  the only remaining tree while compatibility or migration rows are still
  source-backed as live.

Files:

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/035-mix2-fixes/035-09-PLAN.md`
- `.planning/phases/035-mix2-fixes/035-09-SUMMARY.md`
- `.planning/phases/035-mix2-fixes/035-3-garbage-filter.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- [x] `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump`
- [x] run the repeated read-only review loop across `ROADMAP.md`, `STATE.md`,
  `035-09-PLAN.md`, `035-09-SUMMARY.md`, `035-3-garbage-filter.md`, and
  `035-TODO.md` until two consecutive clean passes are reached
- [x] preserve the same review-loop evidence for `035-21`: the loop already
  exceeded the minimum three-pass requirement, passes 1-5 blocked on planning
  drift while the closeout surface was corrected, pass 6 was the first clean
  review pass, pass 7 was the second consecutive clean review pass, and
  closure is accepted only on those two clean passes over that six-file
  surface
- [x] verify the appended completion criteria do not overstate removal scope
- [x] verify any future stronger cleanup is routed through a source update and
  backlog regeneration rather than through silent TODO drift

Exit condition:

- the garbage-filter backlog cannot be mistaken for proof that the repository
  already keeps only the default current production path.

### ✅ Appended Garbage Completion Gate

This appended garbage block is complete only when all of the following hold:

- `035-3-garbage-filter.md` remains the semantic authority for immediate
  garbage-removal scope;
- the first delete wave is limited to the source-backed hard-garbage cluster;
- the reviewed non-production debug cluster stays explicitly deferred outside
  the delete lane until a later canonical source update intentionally requeues
  it;
- live compatibility and migration surfaces remain outside the immediate delete
  lane until the canonical source explicitly demotes them;
- the stronger `only current production-path` target is preserved as explicit
  source drift and not misrepresented as already approved execution truth;
- no garbage cleanup claim is used as proof that sender, stealth, checkpoint,
  claim, wallet, or migration semantics are fully modernized.

## ➕ Appended Backlog - 035-4 Sender Workflow Canonicalization

Canonical design source:

- [035-4-fix-spec](./035-4-fix-spec.md)

Execution rules:

- treat `035-4-fix-spec.md` as the semantic and execution authority for sender
  workflow canonicalization;
- keep `crates/z00z_wallets/src/core/stealth/output_build.rs` as the single
  canonical sender-derivation seam unless the source itself is updated;
- do not move sender derivation into `z00z_core::tx` or recreate temp-spec
  architecture inside `core/tx` adapters;
- preserve current request-bound observable behavior unless an intentional,
  tested change is documented;
- do not change `tag16`, `owner_tag`, `leaf_ad`, `enc_pack`, or hedged-`r`
  formulas during adapter convergence;
- do not remove raw builders unless their compatibility signatures and current
  callers are first covered by explicit adapter or deprecation handling;
- keep wallet-local approval, sender self-check, and future public-verifier
  semantics as distinct layers in both code comments and temp-doc cleanup;
- before starting any numbered task in this appended block, complete its
  `MANDATORY pre-read` block.

### 🎯 Appended Sender Decision Summary

The execution baseline for the appended sender-canonicalization block is:

1. the canonical sender workflow already exists in
   `core/stealth/output.rs` and `output_build.rs`, so Phase 035 must converge
   legacy callers onto that seam instead of inventing a second engine;
2. the dedicated validated card-only sender entrypoint now exists and must stay
  explicit without changing the observable request-bound validated contract;
3. approval logic should be split into explicit small helpers, with
   `approve_card(...)` remaining wallet-local policy rather than a public
   verifier claim;
4. legacy builder and replayable output-bundle paths must become thin adapters
   over canonical stealth helpers while preserving public signatures where
   possible;
5. if full-leaf or replayable callers need variable `serial_id` or injected
   `r`, the preferred fix is to generalize the canonical stealth seam first;
   only if that fails cleanly may a canonical helper under `core/stealth/` be
   added, but not a duplicate derivation engine under `core/tx`;
6. documentation cleanup is part of semantic completion, because current docs
   still risk overstating guarantees or implying conceptual-only workflow.

### 🔗 Appended Sender Dependency Chain

Execution dependency chain:

1. `035-22` sender seam freeze
2. `035-23` canonical helper and approval extension
3. `035-24` validated card-only entrypoint
4. `035-25` legacy builder adapter convergence
5. `035-26` replayable bundle adapter convergence
6. `035-27` stealth export and unit coverage
7. `035-28` downstream adapter regression sweep
8. `035-29` documentation correction wave
9. `035-30` sender workflow validation wave
10. `035-31` sender workflow acceptance gate

Hard dependencies:

- `035-23` depends on `035-22`
- `035-24` depends on `035-22` and `035-23`
- `035-25` depends on `035-23` and `035-24`
- `035-26` depends on `035-23`
- `035-27` depends on `035-24`
- `035-28` depends on `035-25`, `035-26`, and `035-27`
- `035-29` depends on `035-24` through `035-28`
- `035-30` depends on `035-23` through `035-29`
- `035-31` depends on `035-30`

### 🗂️ Appended Sender File-First Implementation Order

Edit order by file cluster:

1. `crates/z00z_wallets/src/core/stealth/output_build.rs`
2. `crates/z00z_wallets/src/core/stealth/output.rs`
3. `crates/z00z_wallets/src/core/tx/builder.rs`
4. `crates/z00z_wallets/src/core/tx/output_flow.rs`
5. `crates/z00z_wallets/src/core/stealth/mod.rs`
6. `crates/z00z_wallets/src/core/stealth/test_output.rs`
7. `crates/z00z_wallets/src/core/stealth/test_output_extra.rs`
8. `crates/z00z_wallets/src/core/tx/`
9. `.planning/temp/Z00Z-ECC-SPEC_part1.md`
10. `.planning/temp/Z00Z-ECC-IDEAS.md`
11. `.planning/phases/035-mix2-fixes/035-TODO.md`

### 📋 Appended Sender Validation Matrix

| Source section | Implementation meaning | TODO coverage | Status |
| --- | --- | --- | --- |
| `Verified Baseline` | current canonical seam already lives in `core/stealth`; raw and legacy caller surfaces still coexist | execution rules; `035-22`; `035-23`; `035-24` | Active |
| `Canonical public entrypoints` | preserve the current raw and request-bound validated entrypoints as canonical public surfaces while adding the dedicated card-only validated lane | `035-22`; `035-24`; `035-27`; `035-31` | Closed through `035-27` |
| `Canonical derivation path` plus `Canonical formula surfaces` and `Canonical ZkPack binding` | keep the helper-owned `build_output_state*` / `build_leaf_state*` seam, `select_r(...)`, `owner_tag`, `tag16`, `leaf_ad`, and the live `ZkPack` binding truth aligned with the wallet-owned stealth seam | `035-22`; `035-23`; `035-28`; `035-30`; `035-31` | Planned |
| `Gap 1: Legacy sender path divergence` | `sender_create_output_for(...)` now reuses canonical helper/formula ownership through an explicit stateless compatibility adapter | `035-25`; `035-28`; `035-31` | Closed in `035-25` |
| `Gap 2: Replayable output bundle path divergence` | `create_output_bundle_with_rng(...)` now reuses canonical stealth helpers through an explicit replayable/stateless compatibility lane | `035-26`; `035-28`; `035-30`; `035-31` | Closed in `035-26` |
| `Gap 3: Dedicated validated card-only entrypoint closure` | `build_card_stealth_output_validated(...)` now exists with wallet-local strict card approval and exported coverage | `035-23`; `035-24`; `035-27`; `035-30`; `035-31` | Closed through `035-27` |
| `Gap 4: Documentation mixes approval levels` | docs and comments must separate raw builder, validated wallet-local approval, sender self-check, and future verifier semantics | `035-29`; `035-30`; `035-31` | Closed in `035-29`; validation waves pending |
| `File-By-File Change Plan` | concrete edit clusters and ownership | `035-23` through `035-29` | Planned |
| `Testing Plan` and `Security and Correctness Requirements` | fail-closed validated entrypoints, no formula drift, no request-regression | `035-27`; `035-28`; `035-30`; `035-31` | `035-27` complete; downstream waves pending |
| `Migration Sequence` and `Acceptance Criteria` | closure must follow helper extension, adapter convergence, docs correction, and acceptance gate | `035-23` through `035-31` | Planned |

### 🚫 Appended Sender Explicit Phase Boundary

The following topics are intentionally out of scope for this appended sender
block unless the canonical source is updated first:

- moving sender derivation into `z00z_core::tx` or any new core-owned sender
  architecture;
- replacing current canonical wallet formulas with temp-doc variants;
- changing request-bound or card-bound `tag16` meaning as part of convergence;
- collapsing wallet-local approval into a public-verifier or consensus claim;
- removing raw builders without a compatibility plan for existing tests,
  examples, and simulator flows;
- bypassing existing hedged-`r` and duplicate-`R` protections to keep legacy
  callers working.

Exit condition:

- the appended sender block cannot be misread as permission to introduce a
  second sender engine, formula drift, or verifier-semantic overclaim.

### 035-22 Sender Seam Freeze

Spec references:

- `Objective`
- `Verified Baseline`
- `Anti-Drift Rules`
- `Canonical Current Workflow`
- `Canonical public entrypoints`
- `Canonical derivation path`
- `Canonical formula surfaces`
- `Canonical ZkPack binding`

MANDATORY pre-read in `035-4-fix-spec.md`:

- section `🎯 Objective`
- section `✅ Verified Baseline`
- section `🧭 Anti-Drift Rules`
- section `📌 Canonical Current Workflow`
- subsection `Canonical public entrypoints`
- subsection `Canonical derivation path`
- subsection `Canonical formula surfaces`
- subsection `Canonical ZkPack binding`

- [x] Freeze `core/stealth/output.rs` and `output_build.rs` as the only
  canonical sender-construction seam for this phase.
- [x] Record that raw builder warnings stay caller-owned and validated flows
  stay wallet-local rather than public-verifier semantics.
- [x] Record the live canonical entrypoints, derivation order, formula seams,
  and `ZkPack` binding truth explicitly so later closure can trace them
  directly instead of only through bundled anti-drift wording.
- [x] Carry the anti-drift rule set into every later task so convergence work
  does not rewrite formulas or architecture ownership.

Files:

- `crates/z00z_wallets/src/core/stealth/output.rs`
- `crates/z00z_wallets/src/core/stealth/output_build.rs`
- `.planning/phases/035-mix2-fixes/035-4-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] `rg -n "build_tx_stealth_output|build_tx_stealth_output_validated|derive_mat|select_r" crates/z00z_wallets/src/core/stealth`
- [x] manually verify no task wording routes canonical sender ownership into
  `z00z_core::tx`

Exit condition:

- the canonical sender seam and its anti-drift boundaries are explicit before
  helper or adapter work begins.

### 035-23 Canonical Helper And Approval Extension

Spec references:

- `Target structure`
- `crates/z00z_wallets/src/core/stealth/output_build.rs`
- `Migration Sequence` / `Phase A`

MANDATORY pre-read in `035-4-fix-spec.md`:

- section `🏗️ Target Architecture`
- subsection `2. crates/z00z_wallets/src/core/stealth/output_build.rs`
- section `🚦 Migration Sequence`

- [x] Add `approve_card(...)` as explicit wallet-local approval logic.
- [x] Keep the helper-owned `build_output_state*` / `build_leaf_state*` seam
  and `select_r(...)` as the canonical derivation path.
- [x] Prefer canonical seam generalization for variable `serial_id` and any
  replayable injected-`r` need; only add `build_output_ctx_with_r(...)` if the
  preferred generalization cannot preserve clean adapter reuse.
- [x] Keep all helper additions under `core/stealth/` and do not recreate
  independent sender derivation elsewhere.

Files:

- `crates/z00z_wallets/src/core/stealth/output_build.rs`
- `.planning/phases/035-mix2-fixes/035-4-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] verify helper additions still preserve the helper-owned build seam as the
  canonical material builder and do not bypass hedged `r`
- [x] verify any injected-`r` or variable-`serial_id` support is documented as
  canonical-helper reuse rather than duplicate derivation

Exit condition:

- canonical helper extension is ready for both validated card-only and legacy
  adapter convergence without creating a second engine.

### 035-24 Validated Card-Only Entrypoint

Spec references:

- `Gap 3: Dedicated validated card-only entrypoint closure`
- `crates/z00z_wallets/src/core/stealth/output.rs`
- `Migration Sequence` / `Phase A`

MANDATORY pre-read in `035-4-fix-spec.md`:

- subsection `Gap 3: Dedicated validated card-only entrypoint closure`
- subsection `1. crates/z00z_wallets/src/core/stealth/output.rs`
- section `🚦 Migration Sequence`

- [x] Add `build_card_stealth_output_validated(...)` in `output.rs`.
- [x] Route the new entrypoint through `approve_card(...)`, canonical output
  construction, and sender self-check.
- [x] Keep the request-bound validated constructor behavior unchanged at the
  observable API level.
- [x] Update local code comments so the new card-only path is explicitly
  wallet-local approval and not a public verifier claim.

Files:

- `crates/z00z_wallets/src/core/stealth/output.rs`
- `crates/z00z_wallets/src/core/stealth/output_build.rs`
- `.planning/phases/035-mix2-fixes/035-4-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] verify the new public signature matches the spec and stays within the
  repository identifier rules
- [x] verify request-bound validated entrypoint wording and call shape remain
  intact after the new card-only entrypoint is added

Exit condition:

- a dedicated validated card-only sender API exists without request-flow
  regression or verifier-semantic drift.

### 035-25 Legacy Builder Adapter Convergence

Spec references:

- `Gap 1: Legacy sender path divergence`
- `crates/z00z_wallets/src/core/tx/builder.rs`
- `Migration Sequence` / `Phase B`

MANDATORY pre-read in `035-4-fix-spec.md`:

- subsection `Gap 1: Legacy sender path divergence`
- subsection `3. crates/z00z_wallets/src/core/tx/builder.rs`
- section `🚦 Migration Sequence`

- [x] Refactor `sender_create_output_for(...)` into an adapter over canonical
  stealth helpers.
- [x] Preserve the public signature unless the repository proves downstream
  absence and an intentional change is documented.
- [x] Convert canonical stealth output into `AssetLeaf` in one owned adapter
  seam instead of keeping a second derivation path.
- [x] Resolve the `serial_id` requirement through canonical seam generalization
  before accepting any fallback helper.

Files:

- `crates/z00z_wallets/src/core/tx/builder.rs`
- `crates/z00z_wallets/src/core/stealth/output.rs`
- `crates/z00z_wallets/src/core/stealth/output_build.rs`
- `.planning/phases/035-mix2-fixes/035-4-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] verify the adapter no longer owns independent sender derivation logic
- [x] verify `serial_id` handling remains explicit and source-backed for
  full-leaf callers

Exit condition:

- `builder.rs` becomes an adapter seam over canonical stealth helpers instead
  of a second sender implementation.

### 035-26 Replayable Bundle Adapter Convergence

Spec references:

- `Gap 2: Replayable output bundle path divergence`
- `crates/z00z_wallets/src/core/tx/output_flow.rs`
- `Migration Sequence` / `Phase B`

MANDATORY pre-read in `035-4-fix-spec.md`:

- subsection `Gap 2: Replayable output bundle path divergence`
- subsection `4. crates/z00z_wallets/src/core/tx/output_flow.rs`
- section `🚦 Migration Sequence`

- [x] Refactor `create_output_bundle_with_rng(...)` so replayable bundle
  construction reuses canonical stealth helpers instead of hand-owning random
  `r` derivation.
- [x] Preserve deterministic or injected randomness for replayable callers
  through canonical helper seams only.
- [x] Keep `output_flow.rs` as a conversion adapter to `OutputBundle`, not as a
  second sender derivation engine.

Files:

- `crates/z00z_wallets/src/core/tx/output_flow.rs`
- `crates/z00z_wallets/src/core/stealth/output_build.rs`
- `.planning/phases/035-mix2-fixes/035-4-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] verify replayable callers still have deterministic or injected-r support
  where required
- [x] verify `output_flow.rs` no longer documents or owns a standalone random-scalar sender path

Exit condition:

- replayable bundle construction stays ergonomic without duplicating sender
  derivation semantics.

### 035-27 Stealth Export And Unit Coverage

Spec references:

- `crates/z00z_wallets/src/core/stealth/mod.rs`
- `crates/z00z_wallets/src/core/stealth/test_output.rs`
- `crates/z00z_wallets/src/core/stealth/test_output_extra.rs`
- `Testing Plan` / `Required unit coverage`

MANDATORY pre-read in `035-4-fix-spec.md`:

- subsection `5. crates/z00z_wallets/src/core/stealth/mod.rs`
- subsection `6. crates/z00z_wallets/src/core/stealth/test_output.rs`
- subsection `7. crates/z00z_wallets/src/core/stealth/test_output_extra.rs`
- section `🧪 Testing Plan`

- [x] Export the new validated card-only entrypoint from `stealth/mod.rs`.
- [x] Add success and fail-closed tests for card-only validated construction.
- [x] Add negative coverage for card validation failure paths and explicit
  request/card approval separation.
- [x] Keep existing request-bound validated tests green while adding the new
  card-only surface.

Files:

- `crates/z00z_wallets/src/core/stealth/mod.rs`
- `crates/z00z_wallets/src/core/stealth/test_output.rs`
- `crates/z00z_wallets/src/core/stealth/test_output_extra.rs`
- `.planning/phases/035-mix2-fixes/035-4-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] verify unit coverage exists for valid signed card, invalid signed card,
  parse failures where applicable, and sender self-check success on the new
  card-only path
- [x] verify current request-bound validated behavior remains green in the same
  test wave

Exit condition:

- the new public entrypoint is exported and covered by fail-closed unit tests.

### 035-28 Downstream Adapter Regression Sweep

Spec references:

- `crates/z00z_wallets/src/core/tx/ tests and adapters`
- `Testing Plan` / `Required regression coverage`
- `Migration Sequence` / `Phase B`

MANDATORY pre-read in `035-4-fix-spec.md`:

- subsection `8. crates/z00z_wallets/src/core/tx/ tests and adapters`
- section `🧪 Testing Plan`
- section `🚦 Migration Sequence`

- [x] Update downstream `core/tx/` tests and adapters so visible output
  behavior stays equivalent while canonical stealth helpers now own sender
  derivation.
- [x] Preserve old public signatures where the spec says compatibility should
  stay intact.
- [x] Update examples or simulator-facing callers when adapter convergence
  changes their internal path.

Files:

- `crates/z00z_wallets/src/core/tx/`
- `crates/z00z_wallets/src/core/stealth/`
- `.planning/phases/035-mix2-fixes/035-4-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] verify legacy adapters still produce equivalent observable outputs after
  migration
- [x] verify no regression in request-bound `tag16`, card-bound `tag16`,
  sender self-validation, hedged `r`, or duplicate-`R` cache behavior

Exit condition:

- downstream adapter callers are converged without observable contract drift.

### 035-29 Documentation Correction Wave

Spec references:

- `Gap 4: Documentation mixes approval levels`
- `Documentation targets`
- `Migration Sequence` / `Phase C`
- `Reference Snippets`

MANDATORY pre-read in `035-4-fix-spec.md`:

- subsection `Gap 4: Documentation mixes approval levels`
- subsection `9. Documentation targets`
- section `🚦 Migration Sequence`
- section `📚 Reference Snippets`

- [x] Rewrite stale temp planning text so it describes the current wallet-owned
  architecture honestly.
- [x] Remove or mark stale sender signatures as historical or superseded.
- [x] Make the approval-level split explicit in docs and code comments:
  raw sender construction, wallet-local validated construction, sender
  self-check, and future public-verifier semantics.

Files:

- `.planning/temp/Z00Z-ECC-SPEC_part1.md`
- `.planning/temp/Z00Z-ECC-IDEAS.md`
- `crates/z00z_wallets/src/core/stealth/output.rs`
- `crates/z00z_wallets/src/core/stealth/output_build.rs`
- `.planning/phases/035-mix2-fixes/035-4-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] verify no documentation still claims sender workflow is merely conceptual
- [x] verify docs and comments separate wallet-local approval and sender
  self-check from future public-verifier semantics

Exit condition:

- sender workflow docs are honest, current, and free of approval-level drift.

### 🧪 Appended Sender Test Execution Tasks

### 035-30 Sender Workflow Validation Wave

Spec references:

- `Testing Plan`
- `Security and Correctness Requirements`
- `Migration Sequence`

MANDATORY pre-read in `035-4-fix-spec.md`:

- section `🧪 Testing Plan`
- section `🔐 Security and Correctness Requirements`
- section `🚦 Migration Sequence`

- [x] Run the full sender-workflow consistency sweep across helper extension,
  validated card-only entrypoint, legacy adapters, replayable adapters, and
  documentation correction.
- [x] Confirm all validated entrypoints fail closed.
- [x] Confirm no path silently downgrades from hedged `r` to plain random `r`
  without an explicit replay or test contract.
- [x] Confirm no formula-level concept drift was introduced for `owner_tag`,
  `tag16`, `leaf_ad`, or `enc_pack` binding.
- [x] Confirm the canonical public entrypoints, derivation order, and live
  `ZkPack` binding language still match the source spec directly.

Files:

- `crates/z00z_wallets/src/core/stealth/`
- `crates/z00z_wallets/src/core/tx/`
- `.planning/temp/Z00Z-ECC-SPEC_part1.md`
- `.planning/temp/Z00Z-ECC-IDEAS.md`
- `.planning/phases/035-mix2-fixes/035-4-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump`
- [x] `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- [x] if `output_flow.rs` or simulator-facing callers change, run the release-style simulator verification path required by the workspace

Exit condition:

- sender canonicalization is validated on helper, adapter, formula, and
  documentation boundaries.

### 035-31 Sender Workflow Acceptance Gate

Spec references:

- `Acceptance Criteria`

MANDATORY pre-read in `035-4-fix-spec.md`:

- section `✅ Acceptance Criteria`

- [x] Confirm there is a dedicated validated card-only sender entrypoint.
- [x] Confirm legacy sender derivation in `builder.rs` is removed or reduced to
  an adapter over canonical stealth helpers.
- [x] Confirm legacy sender derivation in `output_flow.rs` is removed or
  reduced to an adapter over canonical stealth helpers.
- [x] Confirm request-bound validated behavior remains unchanged unless a
  documented and tested intentional change says otherwise.
- [x] Confirm docs no longer claim the sender workflow is merely conceptual and
  that approval-level separation is explicit.
- [x] Confirm no formula-level concept drift was introduced during
  implementation.

Files:

- `crates/z00z_wallets/src/core/stealth/output.rs`
- `crates/z00z_wallets/src/core/stealth/output_build.rs`
- `crates/z00z_wallets/src/core/tx/builder.rs`
- `crates/z00z_wallets/src/core/tx/output_flow.rs`
- `.planning/temp/Z00Z-ECC-SPEC_part1.md`
- `.planning/temp/Z00Z-ECC-IDEAS.md`
- `.planning/phases/035-mix2-fixes/035-4-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] verify every acceptance criterion has direct code or documentation
  evidence and is not inferred from partial adapter work
- [x] verify no closure wording overclaims verifier semantics or architecture
  movement beyond what the spec authorizes

Exit condition:

- Phase 035 sender workflow canonicalization closes only on source-backed
  helper convergence, adapter convergence, validation, and documentation truth.

### ✅ Appended Sender Completion Gate

This appended sender block is complete only when all of the following hold:

- `035-4-fix-spec.md` remains the authority for sender workflow meaning and
  execution order;
- canonical sender derivation still lives under `core/stealth/` and legacy
  caller paths have been reduced to adapters or explicitly documented
  compatibility shims;
- a dedicated validated card-only sender entrypoint exists and is covered by
  fail-closed tests;
- request-bound validated behavior remains observably stable unless an
  intentional change is documented and tested;
- no formula-level drift was introduced for `owner_tag`, `tag16`, `leaf_ad`,
  `enc_pack`, hedged `r`, or duplicate-`R` protection;
- documentation and code comments now separate wallet-local approval, sender
  self-check, and future public-verifier semantics honestly;
- no sender canonicalization claim is used as proof of unrelated stealth,
  checkpoint, or proof-backend closure.

## ➕ Appended Backlog - 035-5 Stealth Canonical Additions

Canonical design source:

- [035-5-fix-spec](./035-5-fix-spec.md)

Execution rules:

- treat `035-5-fix-spec.md` as the authority for which stealth gaps are still
  small canonical additions and which topics remain explicitly out of scope;
- execute only the three approved short additions from the spec: narrow
  `reveal_receiver_secret(...)`, freeze missing stealth derivation vectors, and
  define the V2 memo decode boundary;
- do not pull PIR, OPRF, bucket-routing, Poseidon2 `ZkPack` migration,
  Poseidon3-only unification, or a shipped recursive checkpoint backend into
  this appended block;
- preserve current canonical owners such as `z00z_wallets::core::stealth::*`,
  `z00z_wallets::core::key::*`, and `z00z_crypto::*`; do not move formulas or
  memo ownership to satisfy older temp-doc architecture;
- treat the misnamed subsection labels inside the source workstreams as heading
  drift only; use the enclosing workstream blocks plus `Recommended Execution
  Order` as the sequencing authority;
- keep V1 asset-pack behavior unchanged while adding any V2 memo lane side by
  side;
- before starting any numbered task in this appended block, complete its
  `MANDATORY pre-read` block.

### 🎯 Appended Stealth Decision Summary

The execution baseline for the appended stealth-canonical-additions block is:

1. only three short additions are authorized now: receiver-secret exposure
   narrowing, broader derivation-vector freezing, and a concrete V2 memo decode
   boundary;
2. `reveal_receiver_secret(...)` is a live internal-wallet seam whose public
   visibility should be narrowed only after callsites are inventoried and test
   callers are migrated or given an explicit test-only seam;
3. drift tests already exist and should be expanded to freeze the remaining
   canonical stealth derivation formulas rather than inventing new proof or
   routing architecture;
4. `AssetPackVersion::V2Memo` is already reserved in the version layer, so the
   correct addition is a side-by-side decoded V2 payload contract and strict
   receive support, not mutation of the live V1 wire contract;
5. the source contains subsection-label drift inside workstreams, but its
   actual workstream content and `Recommended Execution Order` still define a
   coherent A -> B -> C sequence;
6. Poseidon2 `ZkPack` migration, Poseidon3-only unification, and recursive
   checkpoint proof ambitions remain explicitly outside this block.

### 🔗 Appended Stealth Dependency Chain

Execution dependency chain:

1. `035-32` stealth-scope freeze
2. `035-33` receiver-secret exposure inventory
3. `035-34` receiver-secret narrowing seam
4. `035-35` stealth derivation vector freeze
5. `035-36` derivation drift regression sweep
6. `035-37` V2 memo contract definition
7. `035-38` V2 memo receive-path enablement
8. `035-39` stealth additions validation wave
9. `035-40` stealth additions acceptance gate

Hard dependencies:

- `035-33` depends on `035-32`
- `035-34` depends on `035-33`
- `035-35` depends on `035-32`
- `035-36` depends on `035-35`
- `035-37` depends on `035-32`
- `035-38` depends on `035-37`
- `035-39` depends on `035-34`, `035-36`, and `035-38`
- `035-40` depends on `035-39`

### 🗂️ Appended Stealth File-First Implementation Order

Edit order by file cluster:

1. `crates/z00z_wallets/src/core/key/stealth_keys_receiver.rs`
2. `crates/z00z_wallets/tests/test_view_key_contract.rs`
3. `crates/z00z_wallets/tests/test_e2e_send_scan.rs`
4. `crates/z00z_wallets/tests/test_scenario1_semantics.rs`
5. `crates/z00z_wallets/tests/test_spec_terms_guard.rs`
6. `crates/z00z_wallets/tests/test_serial_leaf_ad.rs`
7. `crates/z00z_wallets/tests/test_tx_drift.rs`
8. `crates/z00z_wallets/tests/test_stealth_kdf_vectors.rs`
9. `crates/z00z_wallets/tests/fixtures/stealth_kdf_vectors.yaml`
10. `crates/z00z_core/src/assets/leaf.rs`
11. `crates/z00z_core/src/assets/version.rs`
12. `crates/z00z_core/src/assets/mod.rs`
13. `crates/z00z_wallets/src/core/address/stealth_scan_support.rs`
14. `crates/z00z_wallets/src/core/address/stealth_scanner.rs`
15. `crates/z00z_wallets/tests/test_asset_pack_v2_memo.rs`
16. `.planning/phases/035-mix2-fixes/035-TODO.md`

### 📋 Appended Stealth Validation Matrix

| Source section | Implementation meaning | TODO coverage | Status |
| --- | --- | --- | --- |
| `Verified short additions worth doing now` | only Workstreams A, B, and C are in scope | execution rules; `035-32`; `035-40` | Active |
| `Verified non-short additions` and `Poseidon3-only unification is not part of this spec` | block larger routing, proof, and hash-migration branches | execution rules; `035-32`; `035-40` | Active |
| `Corrective note: Poseidon2` | keep the layer model explicit: Poseidon2 is a building block, Plonky3 is a proof toolkit, and stealth is not reduced to a Poseidon2-only claim | `035-32`; `035-39`; `035-40` | Completed |
| `What Memo Path Means In This Repository` | keep memo semantics wallet-private and post-decrypt, not a public-address, routing, or proof-layer feature | `035-32`; `035-37`; `035-38`; `035-39`; `035-40` | Completed |
| `Workstream A: Narrow reveal_receiver_secret(...)` | inventory current callers, then narrow visibility through an internal or test-only seam | `035-33`; `035-34`; `035-39`; `035-40` | Completed |
| `Workstream B: Expand Golden Vectors For Stealth Derivations` | freeze missing canonical derivation formulas with fixture-backed vectors and drift tests | `035-35`; `035-36`; `035-39`; `035-40` | Completed |
| `Workstream C: Define And Implement The V2 Memo Decode Boundary` | add a side-by-side V2 decoded payload contract and enable strict wallet receive support | `035-37`; `035-38`; `035-39`; `035-40` | Completed |
| `Recommended Execution Order` | A then B then C is the sequencing authority despite subsection-label drift | `035-33` through `035-40` | Completed |
| `Source Map` | file clusters stay anchored to current wallet and core owners | file-first order; task file lists | Active |

### 🚫 Appended Stealth Explicit Phase Boundary

The following topics are intentionally out of scope for this appended stealth
block unless the canonical source is updated first:

- PIR inbox or private retrieval networks;
- OPRF-based helper routing or bucket-routing layers;
- a full Poseidon2 `ZkPack` migration;
- Poseidon3-only unification or proof-engine rewrites;
- a fully shipped proof-native OWF or authoritative recursive checkpoint
  backend;
- moving stealth formulas or memo ownership out of their current canonical
  wallet or crypto owners;
- widening `AssetPackPlain` V1 in place instead of adding V2 side by side.

Exit condition:

- this appended stealth block cannot be misread as approval for broader proof,
  routing, or hash-stack architecture changes.

### 035-32 Stealth Scope Freeze

Spec references:

- `Objective`
- `Verified Claim Resolution`
- `Corrective note: Poseidon2`
- `Anti-Drift Rules`
- `What Memo Path Means In This Repository`
- `Recommended Execution Order`

MANDATORY pre-read in `035-5-fix-spec.md`:

- section `🎯 Objective`
- section `🔍 Verified Claim Resolution`
- subsection `🚨 Corrective note: Poseidon2,`
- section `🧭 Anti-Drift Rules`
- section `📌 What Memo Path Means In This Repository`
- section `✅ Recommended Execution Order`

- [x] Freeze the scope of this block to Workstreams A, B, and C only.
- [x] Record the explicit exclusion of routing, proof-backend, and `ZkPack`
  migration branches.
- [x] Record the corrective layer model explicitly so later closure cannot
  drift into `stealth = Poseidon2 only` or `Plonky3 = Poseidon3` wording.
- [x] Record the memo-path meaning explicitly so V2 memo work stays wallet-
  private decrypted metadata rather than a public or proof-layer feature.
- [x] Record that `Recommended Execution Order` governs sequencing even where
  nested subsection labels drift.

Files:

- `.planning/phases/035-mix2-fixes/035-5-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] verify no task in this block requires PIR, OPRF, route-key, Poseidon3,
  or recursive checkpoint work
- [x] verify the dependency chain still follows Workstream A -> B -> C

Exit condition:

- the stealth-additions block has a hard scope fence and a stable execution
  order.

### 035-33 Receiver-Secret Exposure Inventory

Spec references:

- `Workstream A: Narrow reveal_receiver_secret(...)`
- `Verified Baseline`

MANDATORY pre-read in `035-5-fix-spec.md`:

- subsection `Workstream A: Narrow reveal_receiver_secret(...)`
- section `✅ Verified Baseline`

- [x] Inventory every `reveal_receiver_secret(...)` callsite.
- [x] Classify each callsite as production-internal, crate-internal helper, or
  test-only.
- [x] Keep the current public method unchanged until the caller inventory is
  complete.

Files:

- `crates/z00z_wallets/src/core/key/stealth_keys_receiver.rs`
- `crates/z00z_simulator/src/scenario_1/stage_2_utils/flow.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime.rs`
- `crates/z00z_wallets/tests/test_view_key_contract.rs`
- `crates/z00z_wallets/tests/test_e2e_send_scan.rs`
- `crates/z00z_wallets/tests/test_scenario1_semantics.rs`
- `crates/z00z_simulator/tests/test_e2e_phase4.rs`
- `.planning/phases/035-mix2-fixes/035-5-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] `rg -n "reveal_receiver_secret|receiver_secret_hex|ReceiverSecret::from_bytes" crates/z00z_wallets/src crates/z00z_wallets/tests crates/z00z_simulator/src crates/z00z_simulator/tests`
- [x] manually classify each live wallet and simulator consumer as internal production, simulator compatibility, helper, or test-only

Exit condition:

- the receiver-secret caller set is complete enough to narrow visibility
  without breaking intentional test coverage blindly.

### 035-34 Receiver-Secret Narrowing Seam

Spec references:

- `Workstream A: Narrow reveal_receiver_secret(...)`
- `Anti-Drift Rules`

MANDATORY pre-read in `035-5-fix-spec.md`:

- subsection `Workstream A: Narrow reveal_receiver_secret(...)`
- section `🧭 Anti-Drift Rules`

- [x] Narrow `reveal_receiver_secret(...)` to an internal seam once the caller
  inventory proves which consumers are production-internal.
- [x] Migrate external tests toward direct `ReceiverSecret::from_bytes(...)`
  reconstruction where that preserves intent.
- [x] If a test seam still remains necessary, keep it explicit and clearly
  test-only instead of preserving a broad public convenience method.

Files:

- `crates/z00z_wallets/src/core/key/stealth_keys_receiver.rs`
- `crates/z00z_wallets/tests/test_view_key_contract.rs`
- `crates/z00z_simulator/src/scenario_1/stage_2_utils/flow.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime.rs`
- `crates/z00z_simulator/tests/test_e2e_phase4.rs`
- `crates/z00z_wallets/tests/test_e2e_send_scan.rs`
- `crates/z00z_wallets/tests/test_scenario1_semantics.rs`
- `.planning/phases/035-mix2-fixes/035-5-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] verify production callers still have the internal access they require
- [x] verify external tests no longer depend on the general public method unless
  a clearly named test-only seam is intentionally kept

Exit condition:

- receiver-secret exposure is narrower and no longer exported as a broad public
  convenience method.

### 035-35 Stealth Derivation Vector Freeze

Spec references:

- `Workstream B: Expand Golden Vectors For Stealth Derivations`
- `Target formulas to freeze`
- `Files to modify or add`
- `Fixture format recommendation`

MANDATORY pre-read in `035-5-fix-spec.md`:

- subsection `Workstream B: Expand Golden Vectors For Stealth Derivations`
- subsection `Target formulas to freeze`
- subsection `Files to modify or add`
- subsection `Fixture format recommendation`

- [x] Freeze fixture-backed vectors for `derive_k_dh(...)`,
  `derive_k_dh_with_req(...)`, `derive_s_out(...)`, `compute_owner_tag(...)`,
  `compute_tag16(...)`, `compute_tag16_with_req(...)`, and
  `compute_leaf_ad(...)`.
- [x] Reuse existing fixture style rather than inventing a parallel vector
  format.
- [x] Add the dedicated vector test and fixture files if the current test files
  cannot truthfully carry the new matrix.

Files:

- `crates/z00z_wallets/tests/test_spec_terms_guard.rs`
- `crates/z00z_wallets/tests/test_serial_leaf_ad.rs`
- `crates/z00z_wallets/tests/test_tx_drift.rs`
- `crates/z00z_wallets/tests/test_stealth_kdf_vectors.rs`
- `crates/z00z_wallets/tests/fixtures/stealth_kdf_vectors.yaml`
- `.planning/phases/035-mix2-fixes/035-5-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] verify every canonical formula listed in the spec has at least one
  frozen fixture-backed vector
- [x] verify fixture format stays aligned with the existing `leaf_ad` vector
  style

Exit condition:

- the missing derivation surfaces are frozen in a reusable fixture-backed form.

### 035-36 Derivation Drift Regression Sweep

Spec references:

- `Workstream B: Expand Golden Vectors For Stealth Derivations`
- `Test shape recommendation`
- `Acceptance Checks For Workstream C`

MANDATORY pre-read in `035-5-fix-spec.md`:

- subsection `Workstream B: Expand Golden Vectors For Stealth Derivations`
- subsection `Test shape recommendation`
- subsection `Acceptance Checks For Workstream C`

- [x] Add negative drift checks where domain strings, argument order, or
  request-bound versus card-bound contexts are intentionally wrong.
- [x] Prove the different context families do not collapse into one result.
- [x] Keep drift coverage near the existing parity and drift tests instead of
  inventing a new proof-oriented verification harness.

Files:

- `crates/z00z_wallets/tests/test_spec_terms_guard.rs`
- `crates/z00z_wallets/tests/test_tx_drift.rs`
- `crates/z00z_wallets/tests/test_stealth_kdf_vectors.rs`
- `.planning/phases/035-mix2-fixes/035-5-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] verify at least one negative drift test exists per formula family named
  in the spec
- [x] verify request-bound and card-bound contexts produce distinct outcomes
  where the spec expects divergence

Exit condition:

- canonical stealth formulas are frozen both positively and negatively against
  drift.

### 035-37 V2 Memo Contract Definition

Spec references:

- `Workstream C: Define And Implement The V2 Memo Decode Boundary`
- `Meaningful target state`
- `Files to modify`
- `Recommended implementation shape`
- `Memo payload rules`

MANDATORY pre-read in `035-5-fix-spec.md`:

- subsection `Workstream C: Define And Implement The V2 Memo Decode Boundary`
- subsection `Meaningful target state`
- subsection `Files to modify`
- subsection `Recommended implementation shape`
- subsection `Memo payload rules`

- [x] Keep V1 stable and add a side-by-side V2 memo-capable decoded payload
  contract.
- [x] Add strict version-aware decode entrypoints instead of mutating the live
  V1 decode path in place.
- [x] Keep memo encrypted inside `enc_pack`, bounded, optional, and wallet-
  facing rather than public consensus metadata.
- [x] Keep the memo-path explanation aligned with the spec section that says
  memo is a wallet-consumed post-decrypt lane rather than a public-address
  feature.

Files:

- `crates/z00z_core/src/assets/leaf.rs`
- `crates/z00z_core/src/assets/version.rs`
- `crates/z00z_core/src/assets/mod.rs`
- `.planning/phases/035-mix2-fixes/035-5-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] verify V1 asset-pack behavior remains unchanged while V2 is introduced
  side by side
- [x] verify the decoded V2 lane is represented by a concrete type or enum
  branch rather than a lingering TODO-only placeholder

Exit condition:

- the V2 memo lane has a concrete decoded contract without destabilizing V1.

### 035-38 V2 Memo Receive-Path Enablement

Spec references:

- `Workstream C: Define And Implement The V2 Memo Decode Boundary`
- `Who should consume the memo after decryption`
- `Acceptance checks`

MANDATORY pre-read in `035-a5-fix-spec.md`:

- subsection `Workstream C: Define And Implement The V2 Memo Decode Boundary`
- subsection `Who should consume the memo after decryption`
- subsection `Acceptance checks`

- [x] Replace the current automatic `V2Memo` rejection path once the canonical
  decoder is complete.
- [x] Wire wallet receive support so V2 memo stays private decrypted metadata
  and is not copied into public leaf fields.
- [x] Keep malformed or oversized memo payloads fail-closed.

Files:

- `crates/z00z_wallets/src/core/address/stealth_scan_support.rs`
- `crates/z00z_wallets/src/core/address/stealth_scanner.rs`
- proposed test target: `crates/z00z_wallets/tests/test_asset_pack_v2_memo.rs`
- `.planning/phases/035-mix2-fixes/035-a5-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] verify receive path can classify and decode V2 memo outputs without
  leaking memo into public metadata
- [x] verify malformed memo payloads fail closed

Exit condition:

- the wallet receive boundary can classify and decode the V2 memo lane safely.

### 🧪 Appended Stealth Test Execution Tasks

### 035-39 Stealth Additions Validation Wave

Spec references:

- `Verified Baseline`
- `Anti-Drift Rules`
- `Recommended Execution Order`

MANDATORY pre-read in `035-a5-fix-spec.md`:

- section `✅ Verified Baseline`
- section `🧭 Anti-Drift Rules`
- section `✅ Recommended Execution Order`

- [x] Run the full consistency sweep across receiver-secret narrowing,
  derivation-vector freezing, and V2 memo decode enablement.
- [x] Confirm the execution order stayed A -> B -> C.
- [x] Confirm no workstream imported broader proof, routing, or Poseidon-stack
  rewrites.
- [x] Confirm canonical wallet and crypto owners still own the formulas and the
  memo path.
- [x] Confirm the corrective Poseidon2 note and memo-path meaning are still
  represented directly in the planning story and not only implied indirectly.

Files:

- `crates/z00z_wallets/src/core/key/stealth_keys_receiver.rs`
- `crates/z00z_wallets/src/core/address/stealth_scan_support.rs`
- `crates/z00z_wallets/src/core/address/stealth_scanner.rs`
- `crates/z00z_wallets/tests/`
- `crates/z00z_core/src/assets/`
- `.planning/phases/035-mix2-fixes/035-a5-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump`
- [x] `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- [x] if V2 memo receive support touches cross-crate behavior, run a targeted named test binary rather than a plain filter to avoid false-green `running 0 tests` output

Exit condition:

- the three approved stealth additions are validated without scope creep or
  owner drift.

### 035-40 Stealth Additions Acceptance Gate

Spec references:

- `Recommended Execution Order`
- `Source Map`

MANDATORY pre-read in `035-a5-fix-spec.md`:

- section `✅ Recommended Execution Order`
- section `📎 Source Map`

- [x] Confirm Workstream A reduced receiver-secret exposure without breaking the
  intended internal-wallet seam.
- [x] Confirm Workstream B froze every listed canonical derivation formula with
  fixture-backed or equivalent drift coverage.
- [x] Confirm Workstream C added a concrete V2 memo decode boundary while
  keeping V1 behavior unchanged.
- [x] Confirm no out-of-scope routing, proof, or Poseidon-stack branch was
  smuggled into the implementation story.

Files:

- `crates/z00z_wallets/src/core/key/stealth_keys_receiver.rs`
- `crates/z00z_wallets/src/core/address/stealth_scan_support.rs`
- `crates/z00z_wallets/src/core/address/stealth_scanner.rs`
- `crates/z00z_wallets/tests/`
- `crates/z00z_core/src/assets/`
- `.planning/phases/035-mix2-fixes/035-a5-fix-spec.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] verify each acceptance claim maps back to one of the three approved short
  additions and not to an unrelated future branch
- [x] verify closure wording does not overclaim a shipped recursive proof,
  Poseidon3 migration, or public-address memo feature

Exit condition:

- Phase 035 stealth canonical additions close only on the three source-backed
  short additions and nothing broader.

### ✅ Appended Stealth Completion Gate

This appended stealth block is complete only when all of the following hold:

- `035-a5-fix-spec.md` remains the authority for which stealth gaps are short
  canonical additions now;
- Workstream A narrowed receiver-secret exposure through an internal or
  explicit test-only seam instead of leaving a broad public convenience method;
- Workstream B froze the remaining canonical stealth derivation formulas with
  reusable vector and drift coverage;
- Workstream C added a concrete V2 memo decode boundary and wallet receive
  support while keeping V1 stable and memo private;
- no out-of-scope routing, proof, Poseidon2 migration, Poseidon3-only, or
  recursive checkpoint backend work was imported into the block;
- no stealth-additions claim is used as proof of unrelated sender,
  checkpoint, or proof-backend closure.

## ➕ Appended Backlog - 035-6 Curated Rename Convergence

Canonical design source:

- `.planning/phases/035-mix2-fixes/035-a6-renames.md`

Execution rules:

- `035-a6-renames.md` is the authority for rename intent and execution order for
  this block.
- The recovered `Old-Name` -> `Suggested-Name` table and the `High-Confidence
  Delta` block are implementation authority for this append block.
- The `Raw Matrix Of All 814 Over-5-Word Names` is inventory only and must not
  be auto-implemented as if every suggested name were already approved.
- Re-resolve every targeted row against the live declaration before editing any
  file or signature, especially for append rows and reused helper names.
- Rename files first, then rename module declarations, `#[path]` attributes,
  `include!` targets, types, functions, methods, and callsites inside files.
- Keep vendor Tari code excluded.
- Keep declaration-backed rename rows path-specific; do not collapse versioned
  or repeated names by bare identifier alone.

### 🎯 Appended Rename Decision Summary

1. This append block closes only curated rename rows from the recovered
   baseline table and the `High-Confidence Delta` block.
2. Raw-matrix-only candidates remain audit inventory until they receive the
   same explicit curated approval level as the existing rename tables.
3. File-system rename work must land before inside-file signature work so
   module declarations, `#[path]` attributes, and `include!` targets can be
   updated against the new canonical filenames.
4. Rows that were explicitly doublechecked as `no-change` remain frozen and
   must not be renamed indirectly during cleanup.
5. Where the same rename appears in multiple split files, the implementation
   must converge on one canonical spelling in all live declarations and
   callsites.

### 🔗 Appended Rename Dependency Chain

- `035-41` freeze rename scope and authority boundaries.
- `035-42` build the live declaration-backed rename manifest and split it into
  file-rename-first and signature-after lanes.
- `035-43` and `035-44` execute file renames.
- `035-45` and `035-46` update module, include, type, and function signatures
  against the new filenames.
- `035-47` performs the cross-file callsite and no-change guard sweep.
- `035-48` runs the rename validation wave.
- `035-49` closes the rename acceptance gate.

### 📂 Appended Rename File-First Implementation Order

1. `.planning/phases/035-mix2-fixes/035-a6-renames.md`
2. `.planning/phases/035-mix2-fixes/035-TODO.md`
3. file-backed test/module rename clusters across:
   - `crates/z00z_core/src/`
   - `crates/z00z_crypto/src/`
   - `crates/z00z_crypto/tests/`
   - `crates/z00z_simulator/src/`
   - `crates/z00z_storage/src/`
   - `crates/z00z_utils/src/`
   - `crates/z00z_wallets/src/`
4. module/include target files that must mirror renamed files:
   - `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_tests.rs`
   - `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_tests.rs`
   - `crates/z00z_wallets/src/core/key/bip32_manager.rs`
   - `crates/z00z_wallets/src/core/key/bip32_manager_tests.inc.rs`
   - `crates/z00z_wallets/src/db/mod.rs`
5. declaration and callsite clusters for curated symbol renames:
   - `crates/z00z_core/src/assets/wire_pkg_serde_parse.rs`
   - `crates/z00z_simulator/src/claim_pkg_consumer.rs`
   - `crates/z00z_simulator/examples/simulator_interop/`
   - `crates/z00z_simulator/tests/`
   - `crates/z00z_storage/src/serialization/build_temp_tree.rs`
   - `crates/z00z_wallets/src/db/`
   - `crates/z00z_wallets/src/services/`
   - `crates/z00z_wallets/src/wasm/storage_backend.rs`

### 📋 Appended Rename Validation Matrix

| Source section | Implementation meaning | TODO coverage | Status |
| --- | --- | --- | --- |
| `Rename Plan - 2026-04-09` and recovered rename table | execute the original curated `Old-Name` -> `Suggested-Name` rows in deterministic order | `035-41` through `035-49` | Planned |
| `Recovered Provenance` | preserve the recovered baseline snapshot and append snapshot as explicit provenance for the curated rename authority | `035-41`; `035-42`; `035-49` | Planned |
| `Summary` and `Doublecheck` | preserve recovered provenance and deterministic crate/path ordering | `035-41`; `035-42`; `035-49` | Planned |
| `Long-Name / Bad-Name Audit` scope and snapshot | keep vendor exclusion and declaration-only discipline | `035-41`; `035-42`; `035-47`; `035-49` | Planned |
| `Revalidated Existing Rows Already Recorded Above` | keep previously curated repeat rows aligned with their earlier approved names | `035-42`; `035-46`; `035-47`; `035-49` | Planned |
| `Doublechecked No-Change Calls` | freeze explicit no-change rows and prevent accidental rename drift | `035-41`; `035-47`; `035-49` | Planned |
| `Rename Plan - 2026-04-10 - High-Confidence Delta` | implement the newly curated 25-row declaration rename set | `035-42`; `035-46`; `035-47`; `035-49` | Planned |
| `Raw Matrix Of All 814 Over-5-Word Names` | treat raw inventory as non-authoritative backlog input only | `035-41`; `035-49` | Planned |

### 🚫 Appended Rename Explicit Phase Boundary

The following actions are out of scope for this appended rename block unless
`035-a6-renames.md` is updated first:

- auto-applying every row from the 814-entry raw matrix;
- inventing new rename targets not present in the curated rename tables;
- folding repeated names across distinct declaration paths by bare identifier
  only;
- renaming vendor Tari files or symbols;
- mixing functional refactors, logic rewrites, or architecture moves into the
  rename pass;
- changing `Doublechecked No-Change Calls` into rename work;
- changing execution order so inside-file signatures are renamed before the
  referenced files themselves.

Exit condition:

- this rename block cannot be misread as approval for the raw matrix or for
  non-rename refactors.

### 035-41 Rename Scope Freeze

Spec references:

- `Rename Plan - 2026-04-09`
- `Recovered Provenance`
- `Summary`
- `Doublecheck`
- `Doublechecked No-Change Calls`
- `Raw Matrix Of All 814 Over-5-Word Names`

MANDATORY pre-read in `035-a6-renames.md`:

- section `Rename Plan - 2026-04-09`
- section `Recovered Provenance`
- section `Summary`
- section `Doublecheck`
- subsection `Doublechecked No-Change Calls`
- section `Raw Matrix Of All 814 Over-5-Word Names`

- [x] Freeze implementation authority to the recovered rename table plus the
  curated `High-Confidence Delta` block.
- [x] Preserve the recovered baseline snapshot and append snapshot as explicit
  provenance metadata for the curated rename authority.
- [x] Mark the raw 814-row matrix as inventory only.
- [x] Freeze the explicit `no-change` rows so they cannot be renamed by grep
  drift during later waves.
- [x] Freeze the required execution order: files first, signatures after.

Files:

- `.planning/phases/035-mix2-fixes/035-a6-renames.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] manually confirm the raw matrix is not referenced as direct execution
  authority anywhere in this block
- [x] manually confirm the `no-change` rows remain explicit protected
  non-goals for implementation

Exit condition:

- the rename block has a hard authority fence and cannot drift into raw-matrix
  or refactor work.

### 035-42 Live Rename Manifest And Lane Split

Spec references:

- `Rename Plan - 2026-04-09`
- `Revalidated Existing Rows Already Recorded Above`
- `Rename Plan - 2026-04-10 - High-Confidence Delta`

MANDATORY pre-read in `035-a6-renames.md`:

- section `Rename Plan - 2026-04-09`
- subsection `Revalidated Existing Rows Already Recorded Above`
- section `Rename Plan - 2026-04-10 - High-Confidence Delta`

- [x] Re-resolve each targeted rename row against the live declaration or file
  path before editing.
- [x] Build one manifest lane for file renames and a second lane for
  inside-file signature and callsite renames.
- [x] Keep repeated helper names path-qualified so versioned or split symbols
  are not merged incorrectly.
- [x] Mark rows that are path/module/include mirrors and must move in lockstep.

Files:

- `.planning/phases/035-mix2-fixes/035-a6-renames.md`
- `crates/z00z_core/src/`
- `crates/z00z_crypto/src/`
- `crates/z00z_crypto/tests/`
- `crates/z00z_simulator/src/`
- `crates/z00z_simulator/examples/`
- `crates/z00z_simulator/tests/`
- `crates/z00z_storage/src/`
- `crates/z00z_utils/src/`
- `crates/z00z_wallets/src/`

Tests:

- [x] verify every targeted append-row `Old Name` still maps to a live file or
  declaration before edit work starts
- [x] verify file-rename rows and signature rows are split into separate waves
- [x] verify repeated names are path-specific rather than grouped by bare
  identifier only

Exit condition:

- a declaration-backed rename manifest exists and is ordered into file-first
  and signature-after waves.

### 035-43 File Rename Wave A - Test And Support Files

Spec references:

- `Rename Plan - 2026-04-09`
- `Rename Plan - 2026-04-10 - High-Confidence Delta`

MANDATORY pre-read in `035-a6-renames.md`:

- section `Rename Plan - 2026-04-09`
- section `Rename Plan - 2026-04-10 - High-Confidence Delta`

- [x] Rename the curated test and support files that only need canonical file
  names before inside-file symbol edits.
- [x] Keep crate/path ordering stable while processing the recovered table.
- [x] Include simulator, storage, utils, and wallet test-file renames from the
  curated table.

Files:

- `crates/z00z_core/src/assets/*.rs`
- `crates/z00z_core/src/genesis/*.rs`
- `crates/z00z_crypto/src/*.rs`
- `crates/z00z_crypto/tests/*.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/*.rs`
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/*.rs`
- `crates/z00z_storage/src/checkpoint/*.rs`
- `crates/z00z_storage/src/snapshot/*.rs`
- `crates/z00z_utils/src/io/*.rs`
- `crates/z00z_utils/src/os_hardening/*.rs`
- `crates/z00z_wallets/src/**/*.rs`

Tests:

- [x] verify renamed files exist at the suggested paths before any symbol
  rewrite wave starts
- [x] verify no file rename row from the curated table was skipped silently

Exit condition:

- curated test/support file renames are complete and the filesystem is ready
  for inside-file mirror updates.

### 035-44 File Rename Wave B - Wallet DB And Egui Canonical Files

Spec references:

- `Rename Plan - 2026-04-09`
- `Rename Plan - 2026-04-10 - High-Confidence Delta`

MANDATORY pre-read in `035-a6-renames.md`:

- section `Rename Plan - 2026-04-09`
- section `Rename Plan - 2026-04-10 - High-Confidence Delta`

- [x] Rename the curated wallet DB files carrying the `wlt` abbreviation.
- [x] Rename the curated egui tab files carrying unexplained `_1` suffixes.
- [x] Keep these file renames separate from later type and function renames so
  module and callsite rewrites can target final filenames.

Files:

- `crates/z00z_wallets/src/db/wlt_io.rs`
- `crates/z00z_wallets/src/db/wlt_store.rs`
- `crates/z00z_wallets/src/db/wlt_validate.rs`
- `crates/z00z_wallets/src/egui_views/add_wallet_tab_1.rs`
- `crates/z00z_wallets/src/egui_views/app_create_wallet_tab_1.rs`
- `crates/z00z_wallets/src/egui_views/app_logout_tab_1.rs`
- `crates/z00z_wallets/src/egui_views/app_logs_tab_1.rs`
- `crates/z00z_wallets/src/egui_views/app_settings_tab_1.rs`
- `crates/z00z_wallets/src/egui_views/network_*_tab_1.rs`

Tests:

- [x] verify the renamed wallet DB and egui files resolve to the suggested
  canonical filenames before signature updates begin
- [x] verify no `_1` or `wlt_` file row from the curated table remains only
  partially renamed

Exit condition:

- the curated wallet DB and egui file renames are complete and stable.

### 035-45 Signature Rename Wave A - Module, Path, And Include Mirrors

Spec references:

- `Rename Plan - 2026-04-09`

MANDATORY pre-read in `035-a6-renames.md`:

- section `Rename Plan - 2026-04-09`

- [x] Update `mod` declarations to match renamed files.
- [x] Update `#[path]` attributes to match renamed support files.
- [x] Update `include!` targets to match renamed `.inc.rs` files.
- [x] Keep mirror rewrites exact so they only reflect the new file names.

Files:

- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_tests.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_tests.rs`
- `crates/z00z_wallets/src/core/key/bip32_manager.rs`
- `crates/z00z_wallets/src/core/key/bip32_manager_tests.inc.rs`
- `crates/z00z_wallets/src/db/mod.rs`

Tests:

- [x] verify every renamed file with an explicit module/include mirror has its
  corresponding path string updated
- [x] verify no old filename remains in `mod`, `#[path]`, or `include!`
  declarations

Exit condition:

- all file-to-code mirror declarations resolve to the new canonical filenames.

### 035-46 Signature Rename Wave B - Types, Functions, And Methods

Spec references:

- `Revalidated Existing Rows Already Recorded Above`
- `Rename Plan - 2026-04-10 - High-Confidence Delta`

MANDATORY pre-read in `035-a6-renames.md`:

- subsection `Revalidated Existing Rows Already Recorded Above`
- section `Rename Plan - 2026-04-10 - High-Confidence Delta`

- [x] Rename the curated type, function, and method declarations to their
  approved `Suggested-Name` values.
- [x] Keep revalidated earlier rows and the 25-row high-confidence delta
  consistent where the same naming rule appears in multiple split files.
- [x] Preserve the curated pros/cons outcome for long-name fixes instead of
  improvising shorter variants during implementation.

Files:

- `crates/z00z_core/src/assets/wire_pkg_serde_parse.rs`
- `crates/z00z_simulator/src/claim_pkg_consumer.rs`
- `crates/z00z_simulator/examples/simulator_interop.rs`
- `crates/z00z_simulator/examples/simulator_interop/support.rs`
- `crates/z00z_simulator/tests/*.rs`
- `crates/z00z_storage/src/serialization/build_temp_tree.rs`
- `crates/z00z_wallets/src/db/*.rs`
- `crates/z00z_wallets/src/services/*.rs`
- `crates/z00z_wallets/src/wasm/storage_backend.rs`

Tests:

- [x] verify each renamed declaration still resolves to the exact curated
  `Suggested-Name`
- [x] verify repeated split-file helpers converge on one canonical spelling
  across all live declarations
- [x] verify curated long-name replacements still satisfy the `<=5` word rule

Exit condition:

- all curated declaration renames land on the approved canonical spellings.

### 035-47 Cross-File Reference Sweep And No-Change Guard

Spec references:

- `Long-Name / Bad-Name Audit`
- `Doublechecked No-Change Calls`
- `Append Summary - 2026-04-10`

MANDATORY pre-read in `035-a6-renames.md`:

- subsection `Long-Name / Bad-Name Audit`
- subsection `Doublechecked No-Change Calls`
- section `Append Summary - 2026-04-10`

- [x] Update all curated callsites, imports, re-exports, and test references to
  the new file and declaration names.
- [x] Re-run a live symbol recheck so no stale old names remain in active code
  paths.
- [x] Confirm the explicit `no-change` rows still keep their original spelling.
- [x] Confirm repeated versioned or path-split identifiers were updated by
  declaration-backed path, not by naive global grouping.

Files:

- `crates/z00z_core/`
- `crates/z00z_crypto/`
- `crates/z00z_simulator/`
- `crates/z00z_storage/`
- `crates/z00z_utils/`
- `crates/z00z_wallets/`

Tests:

- [x] verify curated old-name residue only on the declaration-backed file list from `035-a6-renames.md`; do not treat unrelated `wlt_path`, `.wlt`, `WltStore`, or raw-matrix-only hits as blockers
- [x] verify explicit `no-change` rows still remain unchanged in the live tree
- [x] verify no curated old name remains in active declarations or references

Exit condition:

- cross-file references match the curated rename plan and no protected
  no-change row was modified.

### 🧪 Appended Rename Test Execution Tasks

### 035-48 Rename Validation Wave

Spec references:

- `Summary`
- `Doublecheck`
- `Append Summary - 2026-04-10`

MANDATORY pre-read in `035-a6-renames.md`:

- section `Summary`
- section `Doublecheck`
- section `Append Summary - 2026-04-10`

- [x] Run the workspace checks needed to prove file moves and symbol renames did
  not break module resolution, imports, or tests.
- [x] Reconfirm that the curated append-block names still satisfy the five-word
  limit after live implementation.
- [x] Reconfirm that this block only implemented curated rename rows.

Files:

- `crates/z00z_core/`
- `crates/z00z_crypto/`
- `crates/z00z_simulator/`
- `crates/z00z_storage/`
- `crates/z00z_utils/`
- `crates/z00z_wallets/`
- `.planning/phases/035-mix2-fixes/035-a6-renames.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`

Tests:

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --all-targets --all-features`
- [x] `cargo test --release --features test-fast --features wallet_debug_dump`
- [x] `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`

Exit condition:

- the curated rename waves compile, resolve, and test cleanly enough to close
  the rename block.

### 035-49 Rename Acceptance Gate

Spec references:

- `Rename Plan - 2026-04-09`
- `Recovered Provenance`
- `Rename Plan - 2026-04-10 - High-Confidence Delta`
- `Doublechecked No-Change Calls`
- `Raw Matrix Of All 814 Over-5-Word Names`

MANDATORY pre-read in `035-a6-renames.md`:

- section `Rename Plan - 2026-04-09`
- section `Recovered Provenance`
- section `Rename Plan - 2026-04-10 - High-Confidence Delta`
- subsection `Doublechecked No-Change Calls`
- section `Raw Matrix Of All 814 Over-5-Word Names`

- [x] Confirm all curated file rename rows landed before their mirrored path and
  signature rewrites.
- [x] Confirm all curated declaration rename rows landed on the approved
  `Suggested-Name` spellings.
- [x] Confirm the recovered baseline snapshot and append snapshot remain named
  as provenance for the curated rename scope.
- [x] Confirm explicit `no-change` rows remain unchanged.
- [x] Confirm the raw matrix was not treated as automatic implementation scope.
- [x] Confirm closure wording still describes a rename-only block rather than a
  semantic refactor or architecture rewrite.

Files:

- `.planning/phases/035-mix2-fixes/035-a6-renames.md`
- `.planning/phases/035-mix2-fixes/035-TODO.md`
- `crates/`

Tests:

- [x] verify every acceptance claim maps to curated rename authority instead of
  raw-matrix inventory
- [x] verify the file-first then signature-after rule is still visible in the
  final change set

Exit condition:

- Phase 035 rename convergence closes only on curated file and signature
  renames, with raw-matrix inventory and no-change rows preserved correctly.

### ✅ Appended Rename Completion Gate

This appended rename block is complete only when all of the following hold:

- `035-a6-renames.md` remains the authority for rename scope and approved target
  names;
- curated file renames landed before any mirrored in-file path or signature
  rewrites;
- curated declaration renames from both the recovered table and the
  high-confidence delta now use their approved `Suggested-Name` spellings;
- repeated or versioned identifiers were updated by live declaration path, not
  by bare-name grouping;
- explicit `Doublechecked No-Change Calls` remain unchanged;
- the raw 814-row matrix remains inventory only and was not promoted into
  automatic execution authority;
- no logic rewrite, vendor edit, or architecture refactor was smuggled into the
  rename work.

## ✅ Full Phase 035 Implementation Readiness Gate

`035-TODO.md` is ready for implementation only when all of the following stay
true at the master-file level:

- all six live Phase 035 source files remain mapped to concrete task ranges in
  this backlog;
- every source file also carries its own local one-to-one spec-to-task table so
  no section is left orphaned;
- task numbering remains continuous from `035-01` through `035-49` with no
  silent gaps, shadow ranges, or backup-only obligations;
- the restored `035-2` block no longer depends on superseded `034` semantic
  split wording;
- deferred-intake, suffix cleanup, garbage removal, sender canonicalization,
  stealth additions, and curated renames remain separated into explicit lanes
  with their own completion gates;
- no block claims semantic closure for another lane without that other lane's
  own acceptance gate passing.
