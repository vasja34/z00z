# Phase 036: Embedded Versioning - Context

**Gathered:** 2026-04-16  
**Status:** Execution baseline synchronized to `036-a1-versioning-spec.md`; `036-TODO-2.md` is the current execution backlog

## 🎯 Phase Boundary

Phase 036 is the embedded-versioning cleanup wave rooted in the existing
`.planning/phases/036-rename/` directory.

The authority chain for this phase is fixed:

- `.planning/phases/036-rename/036-a1-versioning-spec.md` is the canonical
  design source for row meaning, classification, and allowed action.
- `.planning/phases/036-rename/036-TODO-2.md` is the canonical execution
  backlog for task order, file clusters, tests, and phase closure.
- The `Raw Inventory Appendix` in `036-a1-versioning-spec.md` remains the only
  task-generation surface for this phase.

Do not import execution meaning from `036-a1-suffixes-spec.md`,
`036-TODO-1.md`, or stale notes. Those are historical artifacts, not the live
authority surface.

This phase is closed-world at raw-row granularity. If execution discovers a
new design constraint, a missed caller, or a missed producer surface, update
`036-a1-versioning-spec.md` first, then `036-TODO-2.md`, and only then update
this context or implementation artifacts.

## 🔑 Decision Summary Mirror

The current execution baseline derived from `036-a1-versioning-spec.md` is:

1. Freeze explicit wire discriminants, coexisting live lanes, future-reserved
   helpers, and literal or error contracts whose current version marker is part
   of the live contract rather than naming noise.
2. Preserve compatibility shims and compatibility read-import lanes until
   code-backed retirement proof exists.
3. Rename only the Step 2 internal wiring and diagnostic rows whose enclosing
   scope already carries the live version boundary.
4. Rename only the Step 3 single-version internal or persisted identifiers at
   the Rust symbol layer while keeping encoded bytes, literal payloads,
   persisted numeric values, and schema-string contents unchanged.
5. Hold paired legacy or outward-facing contracts until the paired migration or
   publication proof is explicitly approved.
6. Review local and test-only residue only after the production row decisions
   are stable, and preserve explicit version-scenario helpers that still carry
   compatibility evidence.
7. Close the phase only when Steps 0 through 5 remain row-exact, the residual
   scan is rerun, and no second task-generation layer or speculative contract
   rename was introduced.

## ⚙️ File-First Implementation Order Mirror

The live file-first execution order mirrored from `036-TODO-2.md` is:

1. Storage and checkpoint contract files.
2. Claim and crypto contract files.
3. Live asset-lane files.
4. Compatibility-hold and internal-rename wallet files.
5. Single-version identifier files.
6. Outward-contract hold files.
7. Local and test cleanup files named in the local/test residue appendix.
8. Validation and residual-scan artifacts only after touched production and
   test files are settled.

## ⛓️ Dependency Chain Mirror

The global execution dependency chain mirrored from `036-TODO-2.md` is fixed
and non-parallel:

1. `036-01` must close before `036-02` starts.
2. `036-02` must close before `036-03` starts.
3. `036-03` must close before `036-04` starts.
4. `036-04` must close before `036-05` starts.
5. `036-05` must close before `036-06` starts.
6. `036-06` must close before `036-07` starts.

## 📏 Execution Rules Mirror

The execution rules mirrored from `036-TODO-2.md` are fixed for every Phase 036
wave:

1. Execute the phase strictly in the order owned by `Execution Order By Row
  Range`; no parallel reinterpretation of steps is allowed.
2. Treat `036-a1-versioning-spec.md` as normative for requirement meaning and
  `036-TODO-2.md` as normative for execution order.
3. Treat the `Raw Inventory Appendix` as the only task-generation surface.
4. Do not import execution meaning from `036-a1-suffixes-spec.md`,
  `036-TODO-1.md`, or stale notes.
5. Preserve one task owner per step and one path-specific interpretation per
  raw row; identical spellings across files do not authorize row merge.
6. Keep literal-backed contracts frozen unless the canonical source spec
  explicitly upgrades them into a protocol-migration task.
7. Treat the `Compact Patch-Plan By Disputed Row Group` as a grouping aid only;
  if it ever disagrees with a raw row, the raw row wins.
8. If execution discovers a new design constraint, a missed caller, or a
  missed producer surface, update the canonical spec first, then the backlog,
  and only then resume planning or execution.
9. Before any numbered task starts, complete its mandatory pre-read and
  revalidate that task-local files, tests, acceptance rules, and exit
  condition still match the canonical raw-row ownership.
10. In every task verify pass, run deterministic gates first and use AI review
   as one evidence pass only; do not rerun AI review until it becomes green.

## ✅ Validation Matrix Mirror

The validation themes mirrored from `036-TODO-2.md` remain mandatory across the
phase:

1. Freeze rows must preserve explicit wire, schema, RPC, and literal-backed
  contracts byte-for-byte.
2. Compatibility rows must retain live read-import or decode behavior until
  retirement proof is complete.
3. Rename-now rows may change Rust identifiers only within their owned row set
  and only while frozen value contracts stay unchanged.
4. Hold rows remain explicitly blocked until a later migration proof changes
  their disposition.
5. Local and test cleanup may simplify only rows marked `rename now`; it must
  preserve intentional version-scenario evidence marked `keep`.
6. Closure must prove that no second task-generation layer, speculative row
  merge, or uncovered phase-owned residue was introduced.

## ✅ Implementation Decisions

### 🔑 Canonical authority

- **D-01:** `.planning/phases/036-rename/036-a1-versioning-spec.md` is the
  canonical design source for Phase 036.
- **D-02:** `.planning/phases/036-rename/036-TODO-2.md` is the canonical
  execution backlog for Phase 036.
- **D-03:** The `Raw Inventory Appendix` is the only task-generation surface.
  Compact patch-plan groupings, comments, and historical TODO files do not own
  execution meaning.
- **D-04:** `Current classification`, `Action now`, and `Future survivor
  target` are different authority fields. `Future survivor target` is never
  rename-now authority by itself.

### ⚙️ Step-bound execution rules

- **D-05:** Step 0 rows are freeze-only in this wave. They carry explicit
  wire, schema, public RPC, current-lane, coexistence-lane, or future-reserved
  meaning and must not be renamed or deleted here.
- **D-06:** Step 1 rows are compatibility-owned. They must not be renamed or
  deleted until their linked compatibility read, import, decode, or public shim
  window is closed by code-backed retirement proof.
- **D-07:** Step 2 rename authority is limited to non-test rows `22-23` and
  `28`. Only Rust symbol names and their internal call sites may change; public
  RPC strings, transport literals, and outward claim or RPC contracts must stay
  byte-for-byte unchanged.
- **D-08:** Step 3 rename authority is limited to non-test rows `36-43`,
  `45-47`, and `52`. Only Rust identifiers may simplify; encoded bytes,
  persisted numeric values, schema-string contents, and literal payloads must
  remain unchanged.
- **D-09:** Step 4 rows `19` and `44` are blocked-by-contract hold rows. No
  rename is allowed until the paired legacy lane or outward publication
  contract is explicitly migrated.
- **D-10:** Step 5 local and test-only cleanup is downstream of Steps 0 through
  4. Cleanup may rename only rows marked `rename now`; explicit legacy, V1,
  V2, V3, or V4 scenario helpers and tests marked `keep` remain evidence and do
  not become cleanup targets.

### 🚫 Anti-drift guards

- **D-11:** Do not rename literal-backed contracts, error strings, schema or
  wire discriminants, public RPC strings, or future-reserved address helpers in
  this wave unless the source spec itself is upgraded first.
- **D-12:** Do not collapse one signature text across multiple files into one
  synthetic owner. Phase 036 stays path-specific: one raw row, one owned
  interpretation.
- **D-13:** Do not widen execution into bench, example, bin output, generic
  logging text, or comments. `Manual Broad-Scan Residual Disposition` already
  filters these out of the phase surface.
- **D-14:** Do not create a parallel planning register, duplicate ownership
  table, extra rename manifest, or planner-invented abstraction layer outside
  the canonical spec and backlog pair.
- **D-15:** If execution finds a missed caller, missed producer, or new design
  constraint, fail closed: update the versioning spec first, then the backlog,
  and only then resume implementation.
- **D-16:** If a wave edits `036-a1-versioning-spec.md` or `036-TODO-2.md`,
  stop that wave immediately. Re-review and re-plan before any further code
  edits, test closure, or closeout claim in the same wave.
- **D-17:** A retirement proof is complete only when it includes the active
  caller or export inventory, persisted-read or decode proof, the required
  compatibility and negative old-lane tests, and explicit approver
  disposition. Without that full proof set, a row remains hold or keep.
- **D-18:** After Step 3, `build_aad` remains the current alias for the
  existing v1-framed AAD contract only. Any future AAD lane must introduce a
  new helper plus a new domain value rather than overloading the current
  helper.
- **D-19:** Broad token scans are discovery hints only. No scan hit may
  authorize rename, keep, or closure state until it is re-mapped to a concrete
  raw row in the canonical spec.
- **D-20:** Historical artifact strings may appear only as explicit
  non-authority warnings. Mere text presence of `036-TODO-1.md` or
  `036-a1-suffixes-spec.md` is never proof of authority import.

## 📌 Task Transfer Matrix

| Task | Source step | Required meaning | Must remain true before close |
| --- | --- | --- | --- |
| `036-01` | Step 0 | Freeze explicit wire discriminants, live lanes, and literal or error contracts | No protocol, RPC, storage, or cryptographic contract rename is introduced |
| `036-02` | Step 1 | Preserve compatibility shims and compatibility read-import lanes | Every compatibility row is still blocked on explicit retirement proof |
| `036-03` | Step 2 | Rename internal wiring and diagnostic noise only | Only rows `22-23` and `28` change, and outward contracts remain unchanged |
| `036-04` | Step 3 | Rename single-version internal or persisted identifiers only | Encoded values, persisted bytes, and literal payloads remain unchanged |
| `036-05` | Step 4 | Hold paired legacy/public outward contracts | Rows `19` and `44` remain blocked until migration proof exists |
| `036-06` | Step 5 | Clean local and test-only residue after production naming stabilizes | Cleanup does not erase intentional legacy or version-scenario evidence, and every rename-or-keep choice is re-resolved against the canonical raw rows |
| `036-07` | Closure | Validate row coverage, regressions, and no-extra-surface closure | Spec, backlog, and executed code agree on row ownership and remaining holds, with intentionally untouched hold rows recorded explicitly |

## 🧮 Row-Step Coverage Mirror

This mirror restates the execution ownership ledger from `036-TODO-2.md`
without changing authority:

| Task ID | Source step | Raw rows owned by the task | Execution meaning |
| --- | --- | --- | --- |
| `036-01` | Step 0 | non-test `1-18`, `26`, `29-35`, `48-55`; literal `1-25` | freeze explicit wire discriminants, live lanes, and literal or error contracts |
| `036-02` | Step 1 | non-test `20-21`, `24-25`, `27` | preserve compatibility shims and compatibility read-import lanes |
| `036-03` | Step 2 | non-test `22-23`, `28` | rename internal wiring and diagnostic noise |
| `036-04` | Step 3 | non-test `36-43`, `45-47`, `52` | rename current single-version internal or persisted identifiers without changing encoded values |
| `036-05` | Step 4 | non-test `19`, `44` | hold paired legacy/public outward contracts |
| `036-06` | Step 5 | local/test `1-92` | clean rename-safe local/test residue after production naming stabilizes |
| `036-07` | Closure | Steps `0-5` as executed | validate row coverage, regressions, and no-extra-surface closure |

## 🚦 Completion Gate Mirror

No task is complete unless all of the following mirrored gate conditions remain
true:

1. Its owned raw-row set still matches the source spec exactly.
2. Its files, tests, and exit condition still describe the real touched seam.
3. Any hold-only or keep-only rows under the task remain explicitly justified
  rather than silently untouched.
4. Any rename-now rows under the task are closed without changing frozen
  literal, schema, RPC, or persisted-value contracts.
5. No new execution meaning was imported from `036-a1-suffixes-spec.md`,
  `036-TODO-1.md`, or stale notes.
6. `036-07` proves that the raw inventory is still the only task-generation
  surface and that no uncovered phase-owned version-bearing signature remains.
7. If the source spec or backlog changed during execution, the active wave was
  stopped and re-planned before any further implementation or closure work
  resumed.

## 🚩 Explicit Non-Goals Mirror

The following remain out of scope unless the canonical source is updated first:

- importing execution meaning from `036-a1-suffixes-spec.md`, `036-TODO-1.md`,
  or stale planning notes;
- inventing a parallel register, facade, migration tracker, or second
  ownership table;
- renaming wire or schema discriminants, literal-backed contracts, RPC strings,
  or error strings in this wave;
- collapsing coexisting V1/V2 production lanes, future-reserved address
  helpers, or live claim-v2 outer-lane markers into unsuffixed names;
- letting local or test cleanup lead production naming decisions;
- treating bench, example, bin output, generic log text, or comments as part
  of the execution surface.

## ⭐ Canonical References

**Downstream agents MUST read these before planning or implementing.**

### 🔑 Phase-local authority

- `.planning/phases/036-rename/036-a1-versioning-spec.md` - canonical design
  source for row meaning, action, and translation rules.
- `.planning/phases/036-rename/036-TODO-2.md` - canonical execution backlog,
  dependency chain, validation matrix, row-step coverage ledger, and completion
  gate.

### ⚙️ Repository policy constraints

- `.github/copilot-instructions.md` - repository-local workflow, safe-edit, and
  protected-surface rules.
- `.github/requirements/Z00Z_DESIGN_FOUNDATION.md` - primary architectural
  authority for one-source-of-truth, ownership boundaries, and duplicate-layer
  avoidance.

## 📍 Specific Ideas

- Phase 036 is now versioning-spec-backed, not suffixes-spec-backed.
- `036-TODO-2.md` is the live execution surface for this phase.
- The raw inventory remains the only truthful place to widen or narrow row
  ownership.
- Any future execution or closeout artifact that still treats `036-TODO-1.md`
  or `036-a1-suffixes-spec.md` as canonical is stale by definition.

## Deferred Ideas

None. This context intentionally stays inside the current versioning-spec
boundary.

---

*Phase: 036-rename*  
*Context gathered: 2026-04-16*
