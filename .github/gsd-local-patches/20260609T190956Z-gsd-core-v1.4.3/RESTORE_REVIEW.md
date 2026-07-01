# GSD Core v1.4.3 Restore Review

## Chosen Migration

- Migrate the repo-local runtime from `.github/get-shit-done` to `.github/gsd-core`.
- Keep the repository on the new upstream version line `v1.4.3` from `https://github.com/open-gsd/gsd-core`.
- Do not keep `get-shit-done` as the live runtime root after migration.

## Backup Summary

- Backup root: `.github/gsd-local-patches/20260609T190956Z-gsd-core-v1.4.3/`
- Current local files snapshot: `current/`
- Upstream `v1.4.3` comparison snapshot: `upstream-v1.4.3/`
- Preserved file list: `preserved_files.txt`
- Backup manifest integrity: `BACKUP_MISSING=0`

## Global Migration Notes

- Upstream Copilot local install emits `.github/gsd-core`, `.github/skills`, `.github/agents`, `.github/hooks`, and `.github/copilot-instructions.md`.
- Upstream skill execution contexts currently point at `@~/.copilot/gsd-core/...`; for this repo-local install they should be normalized to `@.github/gsd-core/...`.
- Blanket file replacement is unsafe for files below because local repo behavior diverges from upstream.
- Replacing `.github/copilot-instructions.md` with upstream is unsafe; keep the repo-specific file.

## Restore Candidates

| Path in new layout | Type | At-risk local behavior | Recommended mode | Structure changed upstream? | Recommendation |
|---|---|---|---|---|---|
| `.github/gsd-core/workflows/execute-plan.md` | Mandatory | Plan-end `plan_review_gate` with auto review + auto-fix via `/gsd-code-review-fix ${PHASE} --all --auto` | Surgical merge | Yes | Restore |
| `.github/gsd-core/workflows/execute-phase.md` | Mandatory | Phase-level `code_review_gate` auto-fix behavior instead of advisory-only review | Surgical merge | Yes | Restore |
| `.github/skills/gsd-add-tests/SKILL.md` | Mandatory | Repo-local workflow path, richer output contract, existing add-tests behavior | Surgical merge | Yes | Restore |
| `.github/skills/gsd-execute-phase/SKILL.md` | Support | Repo-local execution context path for execute-phase skill | Surgical merge | No | Restore |
| `.github/gsd-core/bin/lib/graphify.cjs` | Optional | Local graphify preflight/version/build/status behavior | Surgical merge | Yes | Restore |
| `.github/skills/gsd-graphify/SKILL.md` | Optional | Repo-local graphify invocation flow and inline build chain | Surgical merge | Yes | Restore |
| `.github/gsd-core/workflows/update.md` | Optional | Old repo-local update flow from previous line | Skip or minimal path-only merge | Yes | Skip by default |
| `.github/gsd-core/workflows/code-review.md` | Optional support | Local review behavior compatibility for restored auto-fix gates | Surgical merge only if required by gate port | Yes | Review if gate port needs it |
| `.github/gsd-core/workflows/code-review-fix.md` | Optional support | Local fix workflow compatibility for restored auto-fix gates | Surgical merge only if required by gate port | Yes | Review if gate port needs it |
| `.github/gsd-core/bin/lib/config.cjs` | Optional support | Local config compatibility for restored gate behavior | Surgical merge only if required by gate port | Yes | Review if gate port needs it |
| `.github/copilot-instructions.md` | Protected repo-local | Repo-specific instructions for this repository | Keep current file | N/A | Keep local |

## Key Evidence

### `execute-plan.md`

- Current local file contains a full `plan_review_gate` block that upstream `v1.4.3` no longer has.
- Local behavior auto-runs scoped review and then auto-runs `gsd-code-review-fix` for non-clean results.
- Upstream file changed heavily in unrelated worktree/orchestrator sections, so full-file rollback is unsafe.

### `execute-phase.md`

- Current local file requires `code_review_gate` and auto-invokes `gsd-code-review-fix`.
- Upstream `v1.4.3` converted this to advisory-only review with a suggestion to run `/gsd-code-review --fix`.
- Upstream file also changed heavily in unrelated execution and worktree sections, so only targeted carry-forward is safe.

### `gsd-add-tests`

- Current local skill uses repo-local `.github/get-shit-done/...` execution context and a broader output contract.
- Upstream local Copilot install points the skill at `@~/.copilot/gsd-core/...`, which is wrong for this repo-local installation and must be normalized anyway.

### `gsd-graphify`

- Current local graphify skill and runtime code are explicitly customized for this repository's graph workflow.
- Upstream `v1.4.3` changed the command text, launcher style, and artifact handling.
- This looks like a good candidate for targeted carry-forward after the `gsd-core` path migration.

## Proposed Restore Set

### Recommended `RESTORE`

- `.github/gsd-core/workflows/execute-plan.md`
- `.github/gsd-core/workflows/execute-phase.md`
- `.github/skills/gsd-add-tests/SKILL.md`
- `.github/skills/gsd-execute-phase/SKILL.md`
- `.github/gsd-core/bin/lib/graphify.cjs`
- `.github/skills/gsd-graphify/SKILL.md`
- Keep current `.github/copilot-instructions.md`

### Recommended `SKIP`

- `.github/gsd-core/workflows/update.md` as a full carry-forward

### Conditional `REVIEW_IF_NEEDED`

- `.github/gsd-core/workflows/code-review.md`
- `.github/gsd-core/workflows/code-review-fix.md`
- `.github/gsd-core/bin/lib/config.cjs`

## Pending User Choice

Choose one of these restore sets before live migration:

1. `Recommended`
2. `Recommended + Conditional`
3. `Minimal mandatory only`
