# Memory Bank Help

## Summary

`memory-bank.instructions.md` is not code and not a plugin. It is a workflow
policy that tells the agent how to preserve project continuity across sessions.

## What It Defines

The policy defines:

- which memory-bank files must exist
- the required reading order
- when updates must happen
- how task history is preserved across sessions

## Main Entry Point

`activeContext.md` is the main memory-bank file.

Read it first on every task. It is the dashboard that tells you:

- what the last verified project truth is
- what is currently changing
- what is still open or risky
- what should happen next
- which files or repository artifacts must be reopened

It does not replace the rest of the memory bank. It routes the next read.

## Expected Memory-Bank Files

- `.github/memory-bank/projectbrief.md`
- `.github/memory-bank/productContext.md`
- `.github/memory-bank/activeContext.md`
- `.github/memory-bank/systemPatterns.md`
- `.github/memory-bank/techContext.md`
- `.github/memory-bank/progress.md`
- `.github/memory-bank/tasks/_index.md`
- `.github/memory-bank/tasks/TASK001-...md`

## Initialization

Initialization usually means:

1. create the `.github/memory-bank/` directory
2. create the required Markdown files
3. write the initial project continuity summary

This does not happen automatically at the Git, Cargo, or operating-system
level. It happens when the files are created manually or by the agent following
the workflow instructions.

## Manual Update

The direct manual trigger is:

- `update memory bank`

When that happens, the agent should:

1. re-read all memory-bank files
2. update `activeContext.md` first
3. reconcile `progress.md`, `tasks/_index.md`, and any relevant task file
4. verify that the dashboard matches real repository evidence

The files can also be edited manually as normal Markdown documents.

## Automatic Update Boundaries

### Agent-driven logical automation

If the agent follows the workflow correctly, memory-bank updates should happen
when:

- significant work is completed
- a new project pattern is discovered
- task status changes
- repository truth changes enough to make `activeContext.md` stale
- the user explicitly asks for `update memory bank`

### Not repository-level automation

The instruction file itself does not:

- run scripts
- watch the file system
- auto-save summaries
- update memory-bank files without agent or human action

It is a workflow contract, not an automation engine.

## Practical Reliability Rules

To keep the memory bank reliable:

1. keep `activeContext.md` short, current, and evidence-backed
2. never promote unfinished planning into verified baseline facts
3. always reconcile dashboard updates with `progress.md` and task history
4. treat `.planning`, validation files, summaries, and changed files as primary
   evidence when updating continuity
5. create a task file whenever important work should survive across sessions

## Bottom Line

- Initialization means creating and seeding the memory-bank files
- Manual update means editing the Markdown files or asking for
  `update memory bank`
- Agent-driven automatic behavior is possible only inside the chat workflow
- Full out-of-band automation would require separate scripts, hooks, or CI
