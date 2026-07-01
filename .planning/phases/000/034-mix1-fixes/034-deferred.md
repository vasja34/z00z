# Phase 034 Deferred Intake

## Purpose

This document is the self-contained deferred-work intake note for Phase 034.
It exists so Phase 034 planning can decide what to absorb without reopening the
older `deferred-items.md` files from earlier phases.

Phase 034 already carries the substantive closure work from:

- `033-FULL-AUDIT.md`
- `fix-spec-4.md`

This document covers only the residual small technical debt candidates from the
older deferred ledger.

## Intake Decision

Phase 034 should **not** inherit any major deferred item from the older phase
list.

Only one small live residual existed in the reviewed deferred ledger at intake:

- `crates/z00z_storage/src/assets/store_internal/store_query.rs`
  - function: `keep_path(...)`
  - status: pre-existing cyclomatic complexity warning
  - shape: small local refactor, not a semantic closure blocker

That item was **optional** for Phase 034 and should be treated only as a tiny
cleanup sidecar. It must not be mixed into the Phase 034 closure claims for:

- helper-owned claim-source continuity
- regular spend nullifier semantics
- authoritative checkpoint proof backend

## Canonical Triage Of Historical Deferred Sources

| Historical source | Current truth | Pull into Phase 034? | Why |
| --- | --- | --- | --- |
| Phase 015 deferred | Resolved | No | Old build blocker is already closed and validated. |
| Phase 026 deferred | Resolved | No | No active deferred item remains. |
| Phase 027 deferred | Resolved | No | No active deferred blocker remains. |
| Phase 029 deferred | Stale | No | Old vendor-doctest blocker was superseded by later planning truth. |
| Phase 030 deferred | Resolved or stale | No | Continuation pass closed the follow-ups. |
| Phase 031 deferred | Resolved | No | Wave 4 closeout left no deferred item. |
| Phase 032 deferred | Live tiny debt | Optional only | Pre-existing `keep_path(...)` complexity warning. |
| Phase 033 deferred | Stale | No | Old vendor or doctest blocker wording is no longer the latest planning truth. |

## Recommended Phase 034 Scope Boundary

### Include by default

- Nothing from the historical deferred ledger.

### Optional tiny sidecar only

- `STORQ-KEEP-PATH-COMPLEXITY`
  - Refactor `keep_path(...)` into smaller local predicates without changing
    behavior.
  - Keep this explicitly non-blocking and out of the semantic closure story.

### Live-tree status after execution

- `STORQ-KEEP-PATH-COMPLEXITY` is now executed on the live tree as a local
  behavior-preserving cleanup.
- The sidecar remains non-semantic and is not part of the Q63, Q64, Q65, or
  Q47 closure proof.

### Do not import into Phase 034

- any old vendor-doctest story from the earlier deferred notes
- any already-resolved release-gate or validation note
- any broad “cleanup everything” tail that would dilute the audit-gap closure
  chain

## Optional Tiny Debt Specification

### `STORQ-KEEP-PATH-COMPLEXITY`

**Location:** `crates/z00z_storage/src/assets/store_internal/store_query.rs`

**Problem:** `keep_path(...)` still carries a pre-existing cyclomatic
complexity warning. The function is small and understandable, but static
analysis still flags it as above the preferred threshold.

**Why this is safe to do:**

- it is local
- it does not require architecture changes
- it does not touch protected vendor code
- it can be proven with behavior-preserving tests

**Why this is not core Phase 034 work:**

- it does not close claim continuity
- it does not add spend nullifier semantics
- it does not add checkpoint backend authority

**Acceptance rule:**

- only do this if the main Phase 034 semantic blockers are already planned and
  clearly separated from this cleanup
- do not claim this cleanup as evidence that Phase 034 itself is semantically
  complete

## Final Recommendation

For Phase 034, the correct default is:

- keep the old deferred ledger out of scope
- absorb no historical deferred item as required Phase 034 work
- the optional `keep_path(...)` complexity cleanup was safe to attach as a
  tiny, explicitly non-semantic side task
