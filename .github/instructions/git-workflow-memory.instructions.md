---
description: 'Workspace memory for direct Z00Z git versioning shortcuts and execution defaults.'
applyTo: '**'
---

# Git Workflow Memory

Workspace memory for how direct Z00Z git versioning commands should be handled.

## Direct Minor Commit Shortcut

- Treat `/z00z-git-versioning minor-commit` as explicit authorization to run the canonical repository flow on the currently checked out branch:
  `./.github/skills/z00z-git-versioning/scripts/version-manager.sh minor --stage-all -f -b "$CURRENT_BRANCH" -m "<message>"`
- Do not ask for an extra confirmation only because the flow stages all changes, creates a version bump and tag, or force-pushes the current branch.
- When the user does not provide a commit message, use a neutral, honest message that matches the real mixed scope of the current worktree.
- Keep execution on the current branch, respect existing deletions, and do not introduce a PR flow.
- If a platform policy, approval gate, or external-execution restriction blocks that exact flow, report the blocker directly instead of silently downgrading to a safer alternative.

## Exact Stage-All Trigger

- Treat the exact command `/z00z-git-versioning minor-commit stage-all` as an execute-as-is instruction.
- Do not ask clarifying or confirmation questions before running the canonical stage-all minor versioning flow on the current branch.
- Preserve the exact `stage-all` intent instead of narrowing scope to selected files.
