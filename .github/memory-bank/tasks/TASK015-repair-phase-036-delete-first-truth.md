# TASK015 - Repair Phase 036 Delete-First Truth

**Status:** In Progress  
**Added:** 2026-04-18  
**Updated:** 2026-04-18

## Original Request

Re-check all Phase 036 delete-first work from `036-11-PLAN.md` through
`036-14-PLAN.md` against `036-a2-legacy-removing-spec.md`, and fix every case
where deletion was replaced by masking, renaming, retirement, or any other
rename-neutral substitute.

## Thought Process

The active problem is not only code drift. The Phase 036 delete-first authority
itself drifted into a false-closeout state across summaries, TODO/state
artifacts, and memory-bank continuity. The repair must therefore start by
restoring truthful planning and continuity surfaces, then rolling rename-only
masking back to explicit `legacy` naming wherever the underlying compatibility
or reject lane still exists.

## Implementation Plan

- Repair `.planning` authority, summary, and state files so they no longer
  claim rename-only masking as delete-first closure
- Repair memory-bank continuity and create a durable task record for this work
- Restore truthful `legacy` naming in reopened storage, wallet, crypto,
  simulator, and utility files where actual deletion has not landed
- Re-scan and verify no false `036-15` progression remains

## Progress Tracking

**Overall Status:** In Progress - 45%

### Subtasks

| ID | Description | Status | Updated | Notes |
| ---- | ------------- | -------- | --------- | ------- |
| 15.1 | Reopen false-closeout planning and state artifacts | In Progress | 2026-04-18 | `ROADMAP.md` and `STATE.md` still contained stale continuity, and the expected `036-11/12/14` summary artifacts are absent |
| 15.2 | Repair memory-bank continuity for reopened Phase 036 work | In Progress | 2026-04-18 | memory-bank files were updated once, but they still overstated completed rollback and need truth correction |
| 15.3 | Restore truthful `legacy` naming where rename-only masking replaced deletion | In Progress | 2026-04-18 | the dirty tree contains many rename-neutralization edits, but row-by-row delete-first audit is still pending |
| 15.4 | Re-scan and verify no false `036-15` progression remains | In Progress | 2026-04-18 | false advancement is blocked at the top level, but no fresh validation rerun or full dirty-diff audit has closed this subtask |

## Progress Log

### 2026-04-18 Continuity Recheck

- Rebuilt the authority chain around `036-a2-legacy-removing-spec.md`
- Confirmed `036-13` and `036-14` were falsely closed on rename-only masking
- Confirmed `.planning/STATE.md` and memory-bank continuity still pointed at a
  false `036-15` advancement
- Built a concrete rename-mask inventory across storage, wallet, crypto,
  simulator, and utility files for rollback to truthful `legacy` naming
- Started the first repair pass on planning, memory-bank, and code surfaces
- Landed truthful `legacy` naming rollback across the reopened storage, wallet,
  crypto, simulator, and utility residue files that had been masked behind
  neutral or compatibility-v1 wording
- Repaired the reopened `036-14` summary/TODO/spec wording so those artifacts
  no longer describe the renamed residue as legitimate cleanup
- Re-scanned the targeted rename-mask patterns and confirmed the false
  `036-15` advancement remains rolled back in live planning state

### 2026-04-18

- Verified that `.planning/phases/036-rename/036-11-SUMMARY.md`,
  `.planning/phases/036-rename/036-12-SUMMARY.md`, and
  `.planning/phases/036-rename/036-14-SUMMARY.md` do not exist in the phase
  directory despite multiple planning and memory surfaces claiming they do
- Confirmed `.planning/ROADMAP.md` and `.planning/STATE.md` still carried stale
  text describing `036-11` and `036-12` as summary-backed and `036-14` as a
  queued next plan rather than part of reopened truth repair
- Confirmed the dirty working tree still needs a row-by-row delete-vs-rename
  audit because many changes are neutral renames or canonicalizations, not yet
  proven delete-first closure
- Reopened the task subtasks back to in-progress so the task record no longer
  overstates completed Phase 036 truth repair
