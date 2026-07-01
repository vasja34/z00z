# Phase 035 Concept Drift Report

## Scope And Frame

- Requested baseline ref: `feat(v2.37.0)`
- Resolved baseline anchor: Git tag `v2.37.0`
- Current review target: workspace `HEAD` on `z00z-dev`
- Scope: `.planning/phases/035-mix2-fixes`
- Focus: `all`
- Baseline access mode: object-level read-only Git access only (`git show`, `git ls-tree`, `git diff`, `git grep`)

The literal input `feat(v2.37.0)` did not resolve as a Git revision. The only
matching repository anchor was tag `v2.37.0`, and the scoped Phase 035 tree
exists at that tag, so this report uses `v2.37.0` as the effective baseline.

## Baseline Concept Inventory

At tag `v2.37.0`, the scoped Phase 035 surface was a compact seven-file packet:

- `035-TODO.md`
- `035-1-deferred.md`
- `035-2-suffixes.md`
- `035-3-garbage-filter.md`
- `035-4-fix-spec.md`
- `035-5-fix-spec.md`
- `035-6-renames.md`

The baseline concepts encoded by that packet were:

- Phase 035 was a bounded execution backlog rooted in deferred-intake control,
  sender fixes, stealth fixes, and later rename follow-through.
- `035-TODO.md` was the scheduling surface.
- `035-1-deferred.md` was the deferred-intake boundary artifact.
- `035-4-fix-spec.md` and `035-5-fix-spec.md` were the only substantive
  implementation authorities for sender and stealth work.
- `keep_path(...)` stayed out of semantic closure unless explicitly admitted as
  opportunistic housekeeping.
- No validation, eval-review, review-chain, or closeout summary stack existed
  yet inside the scoped directory.

## Current Concept Inventory Summary

The current scoped Phase 035 surface has evolved into a full execution and
closeout chain with:

- canonical a-prefixed phase source files `035-a1` through `035-a6`
- a 49-task `035-TODO.md` mapped across all six source lanes
- plan and summary files `035-01` through `035-19`
- bounded review artifacts for late plans
- `035-VALIDATION.md`, `035-UAT.md`, `035-FULL-AUDIT.md`, and
  `035-EVAL-REVIEW.md`
- phase-local closeout wording that preserves a partial-validation verdict for
  the full phase while treating AI evaluation as not applicable

The major semantic shift is not from code behavior to different code behavior.
It is from a compact planning packet to a full audit-grade phase history with
explicit execution authority, reviews, and closeout artifacts.

## Classification Table

| ID | Candidate | Outcome | Why |
| --- | --- | --- | --- |
| C1 | Phase 035 authority surface widened from the baseline seven-file packet to a six-lane a-prefixed execution model with full plan/summary/review artifacts | `justified_change` | The widening is explicit, internally mapped, and backed by completed plan summaries rather than silently inferred. |
| C2 | Rename closeout wording now distinguishes curated acceptance from broader residue outside the active lane | `expected_evolution` | Current Plan 19 artifacts explicitly narrow the closure claim and correct earlier over-broad wording. |
| C3 | Validation and eval artifacts were added after baseline without upgrading full-phase truth beyond partial validation | `justified_change` | Current validation still says `partial`, and the AI eval report is explicitly scoped as applicability-only. |
| C4 | Active authority surfaces still contain stale references to non-existent pre-rename files `035-1/2/3/4/5/6-*.md` | `suspicious_drift` | This is no longer historical prose only; it affects live pre-read, authority, and manual-verification instructions. |
| C5 | `035-EVAL-REVIEW.md` headline says `PRODUCTION READY` while full-phase validation remains partial | `ambiguous` | The body scopes the verdict correctly to AI-eval applicability, but the headline can be skim-read as broader closure. |

## Confirmed Findings

### Finding C4 - Suspicious stale-reference drift in active authority surfaces

**Classification:** `suspicious_drift`

#### Why This Is Not Normal Historical Residue

The current directory contains only the a-prefixed canonical source files:

- `035-a1-deferred.md`
- `035-a2-suffixes.md`
- `035-a3-garbage-filter.md`
- `035-a4-fix-spec.md`
- `035-a5-fix-spec.md`
- `035-a6-renames.md`

However, active authority artifacts still instruct readers to use the old
non-existent filenames.

#### Repository Evidence

- `035-TODO.md` correctly defines the live canonical design sources as
  `035-a1` through `035-a6`, but it still sends readers into stale file names
  in multiple active sections:
  - validation matrix wording for the deferred lane
  - `MANDATORY pre-read` blocks
  - file lists for concrete execution tasks
  - suffix, garbage, sender, stealth, and rename lane headings and references
- `035-CONTEXT.md` correctly names the a-prefixed live authority set in the
  phase boundary and file-first order mirrors, but its lane headings and parts
  of the decision mirror still preserve `035-1-deferred.md` through
  `035-6-renames.md` spellings.
- `035-VALIDATION.md` manual-only verification instructions still tell the
  reader to inspect old filenames such as `035-2-suffixes.md` and
  `035-3-garbage-filter.md`.

#### Why It Matters

This weakens reproducibility and authority hygiene in exactly the documents that
claim to be authoritative. A reader following the current execution or manual
verification instructions literally will be pointed at files that do not exist.
That is concept drift in the authority surface, not just benign historical
provenance.

#### What Keeps This From Being Critical

- The top-level authority map itself is already corrected to the a-prefixed
  files.
- Late closeout artifacts such as `035-19-SUMMARY.md` and `035-19-REVIEW.md`
  correctly describe the active rename lane in terms of `035-a6-renames.md`.
- The issue is documentation-authority drift, not evidence of code-path or
  crypto-path regression.

#### Recommended Narrow Repair For Ambiguity

- Normalize the remaining active/normative references in `035-TODO.md`,
  `035-CONTEXT.md`, and `035-VALIDATION.md` from `035-1/2/3/4/5/6-*` to the
  corresponding `035-a1/2/3/4/5/6-*` files.
- Leave historical plan and summary prose alone unless those files claim to be
  current execution authority.

## Cleared Healthy Evolution

### C1 - Widened six-lane execution authority is documented, not silent drift

**Classification:** `justified_change`

Baseline `035-TODO.md` treated `035-1-deferred.md` as the canonical design
source and `035-4-fix-spec.md` plus `035-5-fix-spec.md` as the substantive
implementation sources. The current `035-TODO.md` explicitly widens Phase 035
into six mapped source lanes and records the fixed task ranges `035-01..035-49`
across `035-a1` through `035-a6`.

This is a real concept change, but it is explicit and mirrored by
`035-CONTEXT.md`, the plan chain, and the summary chain. It is not a silent
fork of meaning.

### C2 - Rename closeout wording narrowed instead of overclaiming

**Classification:** `expected_evolution`

Current late closeout artifacts explicitly say that the Plan 19 closure is
bounded to curated rename authority and does not widen into historical docs,
older plan artifacts, or non-curated compatibility seams. That is the correct
direction of travel and reduces the earlier risk of saying residue existed only
in docs/examples.

### C3 - Validation and eval surfaces were added without erasing partial truth

**Classification:** `justified_change`

The current phase has new validation, UAT, full-audit, and eval-review
artifacts that did not exist at baseline. This is healthy evolution because the
new material does not silently promote the whole phase to fully validated.

The strongest full-phase truth remains:

- `035-VALIDATION.md`: `status: partial`, `nyquist_compliant: false`
- `035-FULL-AUDIT.md`: repeatedly preserves the partial-validation boundary

## Ambiguous Item

### C5 - `PRODUCTION READY` in `035-EVAL-REVIEW.md`

**Classification:** `ambiguous`

This is not a proven contradiction, but it is a wording-level ambiguity.

#### Why It Likely Is Not A Real Drift Bug

`035-EVAL-REVIEW.md` repeatedly states that:

- Phase 035 is a non-AI phase.
- The verdict is an AI-eval applicability verdict only.
- Normal Rust, UAT, security, and Nyquist truth still belongs to
  `035-VALIDATION.md` and related artifacts.

#### Why It Is Still Worth Recording

The headline `PRODUCTION READY` plus `100/100` can be skimmed out of context by
someone who does not read the applicability caveats. That is a presentation
ambiguity, not a state contradiction.

#### Recommended Narrow Repair

- Consider renaming the headline verdict to something like
  `AI-EVAL NOT APPLICABLE - READY` or `PRODUCTION READY FOR AI-EVAL
  APPLICABILITY` if the project wants to reduce skim-risk.

## Doublecheck Ledger

| Candidate | Doublecheck outcome | Result |
| --- | --- | --- |
| C4 stale-reference drift in active authority docs | Confirmed that the problem is present in active/normative surfaces, not only in historical review prose | Keep as `suspicious_drift` |
| C5 `PRODUCTION READY` vs partial validation | Confirmed no hard contradiction, but headline-level skim ambiguity remains | Keep as `ambiguous` |

## Findings First

1. `suspicious_drift`: active Phase 035 authority and manual-verification
   documents still contain stale references to non-existent pre-rename source
   files. This is the only confirmed material concept drift in scope.
2. `ambiguous`: `035-EVAL-REVIEW.md` headline wording can be skim-read too
   broadly, even though the body scopes it correctly to AI-eval applicability.

## Follow-Up Recommendations

1. Repair the stale active references in `035-TODO.md`, `035-CONTEXT.md`, and
   `035-VALIDATION.md` first. That is the highest-signal cleanup because those
   files still claim authority.
2. Leave older historical plan prose untouched unless it still claims to be
   the live authority surface.
3. Optionally tighten the `035-EVAL-REVIEW.md` verdict headline so it cannot be
   skimmed as a full phase-production verdict.

## Final Verdict

Compared with `v2.37.0`, the Phase 035 planning scope shows mostly healthy and
well-documented evolution from a compact backlog packet into a full execution,
review, and closeout chain. The only confirmed material concept drift is stale
file-reference residue inside current authority documents after the move to the
a-prefixed source set. No code-path, crypto-path, or trust-boundary regression
was proven in this scoped audit.
