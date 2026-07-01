# TASK005 - Create create-tests Skill

**Status:** Completed
**Added:** 2026-04-05
**Updated:** 2026-04-05

## Original Request

Create a workspace skill named `create-tests` that can derive a phase-local
E2E or unit test specification document from GSD planning artifacts and then
guide approval-backed unit, integration, and end-to-end test generation. The
skill must explicitly use `brainstorming` together with `crypto-architect`
and/or `security-audit` when crypto or security-sensitive scenarios are in
scope.

## Thought Process

The repository already had `gsd-add-tests` as a command-shaped wrapper, but the
new request was broader: create a reusable lower-level skill that starts from a
phase folder, produces a durable `*-TEST-SPEC.md` artifact, expands internal
and edge-case scenarios deliberately, and only then moves into test generation.
After the first implementation, a second repo-grounded audit showed that the
skill still needed a more detailed implementation-order planning artifact for
GSD-style execution, plus richer sections for scenario inputs, outputs,
artifacts, file placement, diagrams, and code-shape clarification.

## Implementation Plan

- Inspect existing skill-builder rules and adjacent GSD test-generation
  patterns
- Author `.github/skills/create-tests/SKILL.md` with the requested workflow and
  trigger language
- Add compact support files for artifact precedence and reusable templates
- Validate the skill and record the result in the memory bank

## Progress Tracking

**Overall Status:** Completed - 100%

### Subtasks

| ID | Description | Status | Updated | Notes |
| --- | --- | --- | --- | --- |
| 5.1 | Inspect existing patterns and requirements | Complete | 2026-04-05 | Reviewed skill-builder, brainstorming, crypto-architect, security-audit, gsd-add-tests, and existing phase `*-TEST-SPEC.md` artifacts |
| 5.2 | Create the new skill files | Complete | 2026-04-05 | Added `SKILL.md`, `REFERENCE.md`, and `FORMS.md` under `.github/skills/create-tests/` |
| 5.3 | Upgrade the skill after compliance audit | Complete | 2026-04-05 | Added `*-TESTS-TASKS.md` planning support, optional `*-TEST-PLAN.md` compatibility alias, detailed scenario fields, Mermaid guidance, and exact test placement requirements |
| 5.4 | Validate and synchronize continuity docs | Complete | 2026-04-05 | Fixed markdown lint notes, ran Codacy, reran the skill validator, updated GSD add-tests docs, and refreshed memory-bank dashboard files |

## Progress Log

### 2026-04-05

- Confirmed that `.github/skills/create-tests/` already existed as an empty
  directory, so the task was to define the workflow rather than create a new
  location
- Grounded the new skill in the repository's existing `*-TEST-SPEC.md` pattern
  so phase-local test contracts keep the same structure and vocabulary already
  used in Phases 019, 029, and 032
- Made `brainstorming` mandatory for wide scenario discovery and required
  `crypto-architect` plus `security-audit` escalation for proof, commitment,
  signature, secret, and trust-boundary cases
- Added support templates so another engineer or agent can create the test spec
  and approval plan without guessing headings, fields, or pass-oracle shape
- Ran Codacy on all edited skill files and validated the final structure with
  `python3 .github/skills/skill-builder/scripts/validate-skill.py .github/skills/create-tests`
- Performed a follow-up compliance review against the richer assignment and
  aligned the skill with the repository's existing `029-TESTS-TASKS.md`
  pattern instead of inventing an isolated planning artifact
- Expanded the skill and forms so the planning contract now explicitly covers
  what is tested, how it is tested, exact inputs, expected outputs, produced
  artifacts, exact test-file placement, Mermaid flow, and short clarifying code
  snippets when needed
- Updated `.github/skills/gsd-add-tests/SKILL.md` and
  `.github/get-shit-done/workflows/add-tests.md` so GSD-facing instructions now
  mention `*-TEST-SPEC.md`, `*-TESTS-TASKS.md`, and the optional compatibility
  `*-TEST-PLAN.md` alias
- Closed the remaining documentation gaps called out by a follow-up independent
  audit: prefixed and unprefixed artifact names are now described consistently,
  TODO and every `*-PLAN.md` are part of the advertised contract, and the GSD
  workflow wording no longer treats end-to-end coverage as browser-only
- Synced the remaining repo evidence so the Phase 029 exemplar and roadmap now
  use the real `/000/029-crypto-audit-wallets/` path and the unified
  `test(phase-{N}): add unit and E2E tests` commit message
- Normalized the legacy source-path block in
  `.planning/phases/000/029-crypto-audit-wallets/029-TEST-SPEC.md` so the
  exemplar spec, tests-tasks document, and roadmap all reference the same
  canonical Phase 029 directory
- Resolved the final contract drift between the base skill and the GSD wrapper:
  test commits are now clearly opt-in, downstream workflow text accepts the
  full prefixed or unprefixed artifact set, and the last stale panic-inventory
  path inside the Phase 029 spec was corrected
- Reconciled the remaining Phase 029 scenario-ownership drift so
  `029-E2E-03` now names the same backup restore-identity coverage in both the
  spec and the tests-tasks execution artifact
- Adjusted the public usage surface in `.github/skills/create-tests/SKILL.md`
  so the compact example prompt now uses `/create-spec phase=...`, treats the
  phase parameter as sufficient context, and documents a default target of
  roughly 10 to 20 brainstormed scenarios unless the phase size justifies a
  different scope
- Closed the last discoverability gap after the compact prompt rewrite by
  adding `create-spec` and `phase=` to the skill description trigger surface
  and documenting that `/create-spec phase=...` is an accepted spec-first entry
  point even though the canonical skill name remains `create-tests`
- Tightened the compact examples so they no longer weaken the contract: the
  examples now say that `phase=...` alone is sufficient input and explicitly
  name `brainstorming` first, followed by `crypto-architect` and or
  `security-audit` for crypto or security-sensitive phases
