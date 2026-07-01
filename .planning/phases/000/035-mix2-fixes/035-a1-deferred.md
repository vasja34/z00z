# Phase 035 Deferred Intake

## Purpose

This document is the self-contained deferred-work intake note for Phase 035.
It exists so Phase 035 planning can decide what to absorb without reopening the
older `deferred-items.md` files from earlier phases.

Phase 035 already carries its substantive implementation work from:

- `035-a4-fix-spec.md`
- `035-a5-fix-spec.md`

This document covers only the residual small technical debt candidates from the
older deferred ledger.

## Execution Freeze

Phase 035 must not consult or inherit any historical deferred ledger by
default beyond the truth already frozen in this document.

If a future execution pass discovers a historical deferred item that genuinely
belongs in Phase 035, the update order is fixed:

1. update `035-a1-deferred.md` first;
2. then update `035-TODO.md`;
3. then update `035-CONTEXT.md` and any affected closeout artifacts.

No implementation or closeout artifact may widen historical intake before that
source-order update happens.

## Intake Decision

Phase 035 should **not** inherit any required item from the historical deferred
list.

The reviewed deferred ledger does not contain any live residual that naturally
belongs to the Phase 035 substantive sender or stealth backlog.

The only still-live tiny debt found in the older deferred set is unrelated to
the Phase 035 stealth scope:

- `crates/z00z_storage/src/assets/store_internal/store_query.rs`
  - function: `keep_path(...)`
  - status: pre-existing cyclomatic complexity warning
  - relevance to Phase 035: weak and non-native

Because Phase 035 keeps sender workflow canonicalization and narrow stealth
additions as its substantive implementation lanes inside a wider six-source
authority surface, that storage-query cleanup should stay outside the default
Phase 035 scope.

## Canonical Triage Of Historical Deferred Sources

| Historical source | Current truth | Pull into Phase 035? | Why |
| --- | --- | --- | --- |
| Phase 015 deferred | Resolved | No | Already closed and validated. |
| Phase 026 deferred | Resolved | No | No active deferred item remains. |
| Phase 027 deferred | Resolved | No | No active deferred blocker remains. |
| Phase 029 deferred | Stale | No | Old vendor-doctest blocker was superseded by later planning truth. |
| Phase 030 deferred | Resolved or stale | No | Follow-ups were closed in continuation. |
| Phase 031 deferred | Resolved | No | No deferred item remains. |
| Phase 032 deferred | Live tiny debt, but unrelated | No by default | `keep_path(...)` complexity does not advance sender or stealth work. |
| Phase 033 deferred | Stale | No | Old vendor or doctest blocker wording is no longer the latest planning truth. |

## Validation And Closeout Binding

Future Phase 035 validation, acceptance, and closeout artifacts must preserve
the same triage truth without compression or reinterpretation.

- Phases 015, 026, 027, 030, and 031 contribute no live required deferred
  item because their old carry-forward notes are already resolved.
- Phases 029 and 033 contribute no live required deferred item because their
  remaining wording is stale or superseded rather than execution-authoritative.
- Phase 032 contributes only one weak, non-native optional sidecar candidate:
  `keep_path(...)` complexity cleanup.
- Sender workflow work from `035-a4-fix-spec.md` and stealth additions from
  `035-a5-fix-spec.md` remain the only substantive implementation lanes that may
  count toward semantic Phase 035 completion.
- Optional housekeeping, stale vendor-doctest caveats, and resolved legacy
  leftovers must stay outside mandatory acceptance language.

## Recommended Phase 035 Scope Boundary

### Include by default

- nothing from the historical deferred ledger

### Keep explicitly out of scope

- storage-query complexity cleanup in `keep_path(...)`
- old vendor-doctest caveats from stale deferred notes
- already-resolved release-gate or validation leftovers

## Why Phase 035 Should Stay Clean

Phase 035 already has a coherent internal theme:

- canonical sender workflow
- legacy sender-path convergence
- validated card-only entrypoint
- narrow stealth hardening additions
- receiver-secret exposure boundary narrowing
- golden-vector expansion
- explicit `V2Memo` implement-or-defer decision

Importing unrelated storage-query cleanup into this phase would weaken the
semantic boundary of the phase and make closure less honest.

## Optional Exception Rule

If a future planning pass wants one tiny opportunistic cleanup attached to a
Phase 035 execution wave, the only candidate from the historical deferred list
is still the `keep_path(...)` complexity refactor. Even then:

- treat it as opportunistic housekeeping only
- do not let it appear as a stealth-gap closure item
- do not let it compete with sender-workflow or stealth-hardening tasks

Current validation note for the live tree:

- the optional sidecar is currently attached as a staged local refactor in
  `crates/z00z_storage/src/assets/store_internal/store_query.rs`
- the paired staged coverage in `crates/z00z_storage/tests/test_search_api.rs`
  keeps the change scoped to search-behavior preservation
- the attached sidecar remains opportunistic housekeeping only and does not
  count toward sender or stealth semantic closure

## Final Recommendation

For Phase 035, the correct default is:

- import no historical deferred item
- keep substantive implementation focused on `035-a4-fix-spec.md` and
  `035-a5-fix-spec.md` inside the fixed six-source Phase 035 authority surface
- leave the one surviving tiny debt (`keep_path(...)` complexity) outside the
  phase unless it is explicitly attached as opportunistic cleanup

## Live Authority Binding

The live Phase 035 authority surface is limited to:

- `035-TODO.md`
- `035-a1-deferred.md`
- `035-a2-suffixes.md`
- `035-a3-garbage-filter.md`
- `035-a4-fix-spec.md`
- `035-a5-fix-spec.md`
- `035-a6-renames.md`

Within that surface, this document remains a boundary and triage authority
only. Mandatory substantive Phase 035 implementation stays anchored to the live
sender and stealth specs in `035-a4-fix-spec.md` and `035-a5-fix-spec.md`, while
the other canonical lanes keep their own mapped source ownership.

## 🔗 TODO One-To-One Mapping

| 035-1 section | Task coverage | Mapping note |
| --- | --- | --- |
| `Purpose` | `035-01`; `035-03` | freezes deferred-intake semantics and keeps this file boundary-only |
| `Intake Decision` | `035-01`; `035-05`; `035-06` | encodes the no-import default and closeout honesty rules |
| `Canonical Triage Of Historical Deferred Sources` | `035-03`; `035-06` | keeps resolved, stale, and superseded historical items out |
| `Recommended Phase 035 Scope Boundary` | `035-02`; `035-05` | binds the fixed six-source phase surface and keeps substantive implementation on live sender and stealth specs |
| `Include by default` | `035-02` | maps the live in-scope source set |
| `Keep explicitly out of scope` | `035-03`; `035-05` | blocks historical overflow and hidden scope creep |
| `Why Phase 035 Should Stay Clean` | `035-04`; `035-05`; `035-07` | prevents `keep_path(...)` cleanup from becoming fake semantic closure |
| `Optional Exception Rule` | `035-04`; `035-07` | preserves the optional sidecar-only treatment of `keep_path(...)` |
| `Final Recommendation` | `035-01`; `035-02`; `035-07` | closes the deferred-intake lane on a clean scope boundary |
