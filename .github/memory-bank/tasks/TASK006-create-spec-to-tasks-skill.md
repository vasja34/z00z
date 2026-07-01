# TASK006 - Create spec-to-tasks Skill

**Status:** Completed
**Added:** 2026-04-09
**Updated:** 2026-04-09

## Original Request

Create a workspace skill named `spec-to-tasks` that can take any spec or a
free-form design document and convert it into a repository-style `TODO.md`
artifact similar to `.planning/phases/045-state-mngmnt/045-TODO.md` and
`.planning/phases/050-offline-tx/050-TODO.md`. The skill must think through
the small details so the produced TODO files stay uniform and execution-ready.

## Thought Process

The repository already had strong examples of modern backlog shape in Phases
040, 045, and 050, plus a previous skill-creation precedent in
`.github/skills/create-tests/`. The new skill therefore needed to do more than
say "turn doc into tasks": it had to freeze one canonical backlog format,
define how a rough free-doc is normalized before task generation, decide how to
name prefixed and unprefixed task IDs, preserve traceability through a
validation matrix, and explicitly stop when ambiguity would otherwise create a
fake high-confidence backlog.

## Implementation Plan

- Inspect modern `*-TODO.md` exemplars and existing skill conventions
- Define the normalized output contract for repository-style TODO backlogs
- Create `.github/skills/spec-to-tasks/` with `SKILL.md`, `REFERENCE.md`, and
  `FORMS.md`
- Validate the skill and update memory-bank continuity files

## Progress Tracking

**Overall Status:** Completed - 100%

### Subtasks

| ID | Description | Status | Updated | Notes |
| --- | --- | --- | --- | --- |
| 6.1 | Inspect backlog exemplars and skill conventions | Complete | 2026-04-09 | Reviewed Phase 035, 040, 045, and 050 TODO artifacts plus skill-builder and create-tests skill patterns |
| 6.2 | Design the normalized backlog contract | Complete | 2026-04-09 | Fixed the modern section order, task identifier rules, ambiguity-stop behavior, and completion-gate expectations |
| 6.3 | Create the new skill files | Complete | 2026-04-09 | Added `SKILL.md`, `REFERENCE.md`, and `FORMS.md` under `.github/skills/spec-to-tasks/` |
| 6.4 | Validate and refresh continuity docs | Complete | 2026-04-09 | Ran the skill validator, completed Codacy on every edited file, and synced activeContext plus progress |

## Progress Log

### 2026-04-09

- Grounded the new skill in the modern phase backlog format instead of the
  older mandatory-task-only layout so generated TODO artifacts keep the same
  execution grammar as Phases 040, 045, and 050
- Defined two operating modes: `spec-backed` for structured sources and
  `free-doc normalization` for rough narrative documents that need decisions,
  invariants, non-goals, seams, and tests extracted before task drafting
- Added explicit output-naming rules so phase-backed sources emit
  `NNN-TODO.md`, while unprefixed sources emit `TODO.md` with `TODO-01` style
  identifiers unless the user supplies a custom prefix
- Made `Validation Matrix` and `Explicit Phase Boundary` mandatory parts of the
  modern output contract while treating the validation-wave heading and any
  phase-level closing section as exemplar-driven instead of universally fixed
- Added an ambiguity-report template and stop conditions to prevent the skill
  from fabricating dependencies, file seams, or pre-read line ranges when the
  source document does not support them
- Tightened the skill after a source-backed audit so it now describes the real
  shared denominator of `045-TODO.md` and `050-TODO.md`: dedicated validation
  waves remain optional and heading-specific, while `Completion Gate` is no
  longer presented as a universal section
- Synced the memory-bank dashboard and progress summary so future sessions know
  that the repository now has both a `create-tests` planning skill and a new
  `spec-to-tasks` backlog-generation skill
- Validated the new skill with
  `python3 .github/skills/skill-builder/scripts/validate-skill.py .github/skills/spec-to-tasks`
  and got a clean pass with no warnings
- Ran Codacy analysis for every edited skill and memory-bank file and got zero
  findings across the whole change set
