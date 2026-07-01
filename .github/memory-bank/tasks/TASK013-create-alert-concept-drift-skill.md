# TASK013 - Create alert-concept-drift Skill

**Status:** Completed
**Added:** 2026-04-14
**Updated:** 2026-04-14

## Original Request

Create a workspace skill named `alert-concept-drift` that compares the current
repository to a selected historical Git commit and produces a detailed report
that separates healthy concept evolution from suspicious drift. The skill must
focus on security, API contracts, duplication, architecture, and cryptography,
and it must route suspicious or ambiguous findings through `doublecheck`
instead of treating every diff as a real regression.

## Thought Process

The request was explicitly not for a raw diff helper. The repository already
had adjacent skills for spec compliance, security review, crypto reasoning, and
doublecheck, so the new skill needed to orchestrate those capabilities into a
single historical-anchor workflow. The key design constraint was classification:
the skill had to distinguish normal evolution, documented concept changes, real
suspicious drift, critical regressions, and evidence gaps, with a report shape
that makes the cleared items visible instead of collapsing everything into a
flat findings list.

## Implementation Plan

- Review skill-builder rules, agent skill guidance, and adjacent repository
  skills that already handle compliance, crypto, security, and doublecheck
- Design the concept-drift workflow around baseline concept extraction rather
  than raw code diffing
- Add `.github/skills/alert-concept-drift/` with `SKILL.md`, `REFERENCE.md`,
  and `FORMS.md`
- Validate the new skill and synchronize memory-bank continuity files

## Progress Tracking

**Overall Status:** Completed - 100%

### Subtasks

| ID | Description | Status | Updated | Notes |
| --- | --- | --- | --- | --- |
| 13.1 | Inspect local skill rules and adjacent patterns | Complete | 2026-04-14 | Reviewed skill-builder, agent-skills guidance, doublecheck, spec-to-code-compliance, and code-reviewer patterns |
| 13.2 | Design the concept-drift classification model | Complete | 2026-04-14 | Defined baseline concept reconstruction, drift dimensions, and the five-way classification model |
| 13.3 | Create the new skill files | Complete | 2026-04-14 | Added `SKILL.md`, `REFERENCE.md`, and `FORMS.md` under `.github/skills/alert-concept-drift/` |
| 13.4 | Validate and sync continuity docs | Complete | 2026-04-14 | Ran `get_errors`, passed the skill validator cleanly, attempted Codacy on edited Markdown files, and refreshed memory-bank files |
| 13.5 | Harden safe-mode git comparison rules | Complete | 2026-04-14 | Added explicit object-level Git access first, detached worktree escalation, and forbidden live-worktree mutation rules |
| 13.6 | Add concrete safe audit command patterns | Complete | 2026-04-14 | Added reproducible object-level, detached worktree, and safe cleanup command patterns to the skill |
| 13.7 | Normalize report anchor naming and add argument hints | Complete | 2026-04-14 | Added `argument-hint` frontmatter plus normalized `<anchor-slug>` guidance so refs with `/` stay in one report file path |

## Progress Log

### 2026-04-14

- Interpreted the request as a semantic and invariant drift audit, not a line
  diff helper, and shaped the workflow around concept extraction from a
  historical Git anchor
- Reused the repository's existing strength areas instead of inventing a new
  isolated review engine: `spec-to-code-compliance` for claim mapping,
  `security-audit` for trust-boundary review, `crypto-architect` for crypto
  invariant analysis, `code-reviewer` for duplication and architecture drift,
  and `doublecheck` for adversarial confirmation of suspicious items
- Added a five-way classification model:
  `expected_evolution`, `justified_change`, `suspicious_drift`,
  `critical_regression`, and `ambiguous`
- Wrote support reference material that defines evidence precedence,
  dimension-specific drift checks, healthy-evolution heuristics, severity
  guidance, and hard rules for when suspicious items must be downgraded due to
  missing evidence
- Added reusable forms for a detailed concept-drift report, per-finding cards,
  cleared-evolution cards, and a compact chat summary table
- Ran
  `python .github/skills/skill-builder/scripts/validate-skill.py .github/skills/alert-concept-drift`
  and got a clean pass with no warnings
- Attempted Codacy analysis on the edited Markdown skill files as required by
  repository policy; the configured Codacy toolset reported that Markdown files
  were unsupported or had no applicable analyzers, so validation fell back to
  repository markdown diagnostics plus the local skill validator
- Added an explicit safe-mode Git comparison model after follow-up review:
  the skill now forbids switching the active worktree to the baseline branch,
  forbids reset or clean style operations, defaults to read-only `git show` and
  related object-level access, and allows a detached isolated worktree only as
  a bounded read-only escalation path when a materialized historical tree is
  genuinely needed
- Added explicit recommended command patterns so the skill now shows the safe
  operational sequence directly: object-level reads first, detached scratch
  worktree only when necessary, dirty-state refusal before cleanup, and final
  removal via `git worktree remove` instead of unsafe workspace mutation
- Added a final hardening pass on invocation ergonomics and report naming: the
  skill now exposes a concrete `argument-hint` with `baseline_ref`, `scope`,
  `focus`, and `report_path`, and the recommended report filename now uses a
  normalized `<anchor-slug>` so refs like `origin/main` do not create nested
  directories under `reports/concept-drift/`
