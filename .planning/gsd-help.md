# GSD Command Reference

**GSD** (Get Shit Done) creates hierarchical project plans optimized for solo agentic development with Claude Code.

## Quick Start

1. `/gsd-new-project` - Initialize project (includes research, requirements, roadmap)
2. `/gsd-plan-phase 1` - Create detailed plan for first phase
3. `/gsd-execute-phase 1` - Execute the phase

## Staying Updated

GSD evolves fast. Update periodically:

```bash
npx get-shit-done-cc@latest
```

## Overview

📌 This file is a local GSD help sheet built from the skill inventory in
`/home/vadim/.copilot/skills`.

📌 Each row captures the documented skill description plus the visible command
arguments or settings from `SKILL.md` frontmatter.

## Canonical Reference

📌 Canonical command reference source: `~/.copilot/get-shit-done/workflows/help.md`.

📌 Core workflow: `/gsd-new-project -> /gsd-plan-phase -> /gsd-execute-phase -> repeat`.

📌 Quick start:

1. `/gsd-new-project`
2. `/gsd-plan-phase 1`
3. `/gsd-execute-phase 1`

## Skills Inventory

| Skill                        | Description                                                  | Args or settings                                             | Agent         | Allowed tools                                                |
| ---------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------ | ------------- | ------------------------------------------------------------ |
| `gsd-add-backlog`            | Add an idea to the backlog parking lot (999.x numbering)     | `description`                                                | -             | `Read, Write, Bash`                                          |
| `gsd-add-phase`              | Add phase to end of current milestone in roadmap             | `description`                                                | -             | `Read, Write, Bash`                                          |
| `gsd-add-tests`              | Generate tests for a completed phase based on UAT criteria and implementation | `<phase> [additional instructions]`                          | -             | `Read, Write, Edit, Bash, Glob, Grep, Task, AskUserQuestion` |
| `gsd-add-todo`               | Capture idea or task as todo from current conversation context | `[optional description]`                                     | -             | `Read, Write, Bash, AskUserQuestion`                         |
| `gsd-audit-milestone`        | Audit milestone completion against original intent before archiving | `[version]`                                                  | -             | `Read, Glob, Grep, Bash, Task, Write`                        |
| `gsd-audit-uat`              | Cross-phase audit of all outstanding UAT and verification items | -                                                            | -             | `Read, Glob, Grep, Bash`                                     |
| `gsd-autonomous`             | Run all remaining phases autonomously — discuss→plan→execute per phase | `[--from N]`                                                 | -             | `Read, Write, Bash, Glob, Grep, AskUserQuestion, Task`       |
| `gsd-check-todos`            | List pending todos and select one to work on                 | `[area filter]`                                              | -             | `Read, Write, Bash, AskUserQuestion`                         |
| `gsd-cleanup`                | Archive accumulated phase directories from completed milestones | -                                                            | -             | -                                                            |
| `gsd-complete-milestone`     | Archive completed milestone and prepare for next version     | `<version>`                                                  | -             | `Read, Write, Bash`                                          |
| `gsd-debug`                  | Systematic debugging with persistent state across context resets | `[issue description]`                                        | -             | `Read, Bash, Task, AskUserQuestion`                          |
| `gsd-discuss-phase`          | Gather phase context through adaptive questioning before planning. Use `--auto` to skip interactive questions. | `<phase> [--auto] [--batch] [--analyze] [--text]`            | -             | `Read, Write, Bash, Glob, Grep, AskUserQuestion, Task, mcp__context7__resolve-library-id, mcp__context7__query-docs` |
| `gsd-do`                     | Route freeform text to the right GSD command automatically   | `<description of what you want to do>`                       | -             | `Read, Bash, AskUserQuestion`                                |
| `gsd-execute-phase`          | Execute all plans in a phase with wave-based parallelization | `<phase-number> [--wave N] [--gaps-only] [--interactive]`    | -             | `Read, Write, Edit, Glob, Grep, Bash, Task, TodoWrite, AskUserQuestion` |
| `gsd-fast`                   | Execute a trivial task inline — no subagents, no planning overhead | `[task description]`                                         | -             | `Read, Write, Edit, Bash, Grep, Glob`                        |
| `gsd-forensics`              | Post-mortem investigation for failed GSD workflows — analyzes git history, artifacts, and state to diagnose what went wrong | `[problem description]`                                      | -             | `Read, Write, Bash, Grep, Glob`                              |
| `gsd-health`                 | Diagnose planning directory health and optionally repair issues | `[--repair]`                                                 | -             | `Read, Bash, Write, AskUserQuestion`                         |
| `gsd-help`                   | Show available GSD commands and usage guide                  | -                                                            | -             | -                                                            |
| `gsd-insert-phase`           | Insert urgent work as decimal phase (e.g., 72.1) between existing phases | `after description`                                          | -             | `Read, Write, Bash`                                          |
| `gsd-join-discord`           | Join the GSD Discord community                               | -                                                            | -             | -                                                            |
| `gsd-list-phase-assumptions` | Surface the agent's assumptions about a phase approach before planning | `[phase]`                                                    | -             | `Read, Bash, Grep, Glob`                                     |
| `gsd-list-workspaces`        | List active GSD workspaces and their status                  | -                                                            | -             | `Bash, Read`                                                 |
| `gsd-manager`                | Interactive command center for managing multiple phases from one terminal | -                                                            | -             | `Read, Write, Bash, Glob, Grep, AskUserQuestion, Task`       |
| `gsd-map-codebase`           | Analyze codebase with parallel mapper agents to prccccccoduce `.planning/codebase/` documents | `[optional: specific area to map, e.g., 'api' or 'auth']`    | -             | `Read, Bash, Glob, Grep, Write, Task`                        |
| `gsd-milestone-summary`      | Generate a comprehensive project summary from milestone artifacts for team onboarding and review | `[version]`                                                  | -             | `Read, Write, Bash, Grep, Glob`                              |
| `gsd-new-milestone`          | Start a new milestone cycle — update `PROJECT.md` and route to requirements | `[milestone name, e.g., 'v1.1 Notifications']`               | -             | `Read, Write, Bash, Task, AskUserQuestion`                   |
| `gsd-new-project`            | Initialize a new project with deep context gathering and `PROJECT.md` | `[--auto]`                                                   | -             | `Read, Bash, Write, Task, AskUserQuestion`                   |
| `gsd-new-workspace`          | Create an isolated workspace with repo copies and independent `.planning/` | `--name NAME [--repos repo1,repo2] [--path /target] [--strategy worktree-or-clone] [--branch name] [--auto]` | -             | `Read, Bash, Write, AskUserQuestion`                         |
| `gsd-next`                   | Automatically advance to the next logical step in the GSD workflow | -                                                            | -             | `Read, Bash, Grep, Glob, SlashCommand`                       |
| `gsd-note`                   | Zero-friction idea capture. Append, list, or promote notes to todos. | `text` or `list` or `promote N [--global]`                   | -             | `Read, Write, Glob, Grep`                                    |
| `gsd-pause-work`             | Create context handoff when pausing work mid-phase           | -                                                            | -             | `Read, Write, Bash`                                          |
| `gsd-plan-milestone-gaps`    | Create phases to close all gaps identified by milestone audit | -                                                            | -             | `Read, Write, Bash, Glob, Grep, AskUserQuestion`             |
| `gsd-plan-phase`             | Create detailed phase plan (`PLAN.md`) with verification loop | `[phase] [--auto] [--research] [--skip-research] [--gaps] [--skip-verify] [--prd FILE] [--reviews] [--text]` | `gsd-planner` | `Read, Write, Bash, Glob, Grep, Task, WebFetch, mcp__context7__*` |
| `gsd-plant-seed`             | Capture a forward-looking idea with trigger conditions — surfaces automatically at the right milestone | `[idea summary]`                                             | -             | `Read, Write, Edit, Bash, AskUserQuestion`                   |
| `gsd-pr-branch`              | Create a clean PR branch by filtering out `.planning/` commits — ready for code review | `[target branch, default: main]`                             | -             | `Bash, Read, AskUserQuestion`                                |
| `gsd-profile-user`           | Generate developer behavioral profile and create Claude-discoverable artifacts | `[--questionnaire] [--refresh]`                              | -             | `Read, Write, Bash, Glob, Grep, AskUserQuestion, Task`       |
| `gsd-progress`               | Check project progress, show context, and route to next action (execute or plan) | -                                                            | -             | `Read, Bash, Grep, Glob, SlashCommand`                       |
| `gsd-quick`                  | Execute a quick task with GSD guarantees (atomic commits, state tracking) but skip optional agents | `[--full] [--discuss] [--research]`                          | -             | `Read, Write, Edit, Glob, Grep, Bash, Task, AskUserQuestion` |
| `gsd-reapply-patches`        | Reapply local modifications after a GSD update               | -                                                            | -             | -                                                            |
| `gsd-remove-phase`           | Remove a future phase from roadmap and renumber subsequent phases | `phase-number`                                               | -             | `Read, Write, Bash, Glob`                                    |
| `gsd-remove-workspace`       | Remove a GSD workspace and clean up worktrees                | `workspace-name`                                             | -             | `Bash, Read, AskUserQuestion`                                |
| `gsd-research-phase`         | Research how to implement a phase (standalone - usually use `/gsd-plan-phase` instead) | `[phase]`                                                    | -             | `Read, Bash, Task`                                           |
| `gsd-resume-work`            | Resume work from previous session with full context restoration | -                                                            | -             | `Read, Bash, Write, AskUserQuestion, SlashCommand`           |
| `gsd-review`                 | Request cross-AI peer review of phase plans from external AI CLIs | `--phase N [--gemini] [--claude] [--codex] [--all]`          | -             | `Read, Write, Bash, Glob, Grep`                              |
| `gsd-review-backlog`         | Review and promote backlog items to active milestone         | -                                                            | -             | `Read, Write, Bash`                                          |
| `gsd-session-report`         | Generate a session report with token usage estimates, work summary, and outcomes | -                                                            | -             | `Read, Bash, Write`                                          |
| `gsd-set-profile`            | Switch model profile for GSD agents (`quality`, `balanced`, `budget`, `inherit`) | `profile: quality, balanced, budget, inherit`                | -             | `Bash`                                                       |
| `gsd-settings`               | Configure GSD workflow toggles and model profile             | -                                                            | -             | `Read, Write, Bash, AskUserQuestion`                         |
| `gsd-ship`                   | Create PR, run review, and prepare for merge after verification passes | `[phase number or milestone, e.g., '4' or 'v1.0']`           | -             | `Read, Bash, Grep, Glob, Write, AskUserQuestion`             |
| `gsd-stats`                  | Display project statistics — phases, plans, requirements, git metrics, and timeline | -                                                            | -             | `Read, Bash`                                                 |
| `gsd-thread`                 | Manage persistent context threads for cross-session work     | `[name or description]`                                      | -             | `Read, Write, Bash`                                          |
| `gsd-ui-phase`               | Generate UI design contract (`UI-SPEC.md`) for frontend phases | `[phase]`                                                    | -             | `Read, Write, Bash, Glob, Grep, Task, WebFetch, AskUserQuestion, mcp__context7__*` |
| `gsd-ui-review`              | Retroactive 6-pillar visual audit of implemented frontend code | `[phase]`                                                    | -             | `Read, Write, Bash, Glob, Grep, Task, AskUserQuestion`       |
| `gsd-update`                 | Update GSD to latest version with changelog display          | -                                                            | -             | `Bash, AskUserQuestion`                                      |
| `gsd-validate-phase`         | Retroactively audit and fill Nyquist validation gaps for a completed phase | `[phase number]`                                             | -             | `Read, Write, Edit, Bash, Glob, Grep, Task, AskUserQuestion` |
| `gsd-verify-work`            | Validate built features through conversational UAT           | `[phase number, e.g., '4']`                                  | -             | `Read, Bash, Glob, Grep, Edit, Write, Task`                  |
| `gsd-workstreams`            | Manage parallel workstreams — list, create, switch, status, progress, complete, and resume | `list, create NAME, status NAME, switch NAME, progress, complete NAME, resume NAME` | -             | -                                                            |

## Settings-Focused Commands

📌 Commands with explicit configuration semantics:

| Command             | Main settings or toggles                                     |
| ------------------- | ------------------------------------------------------------ |
| `gsd-settings`      | `model`, `research`, `plan_check`, `verifier`, `branching`   |
| `gsd-set-profile`   | `quality`, `balanced`, `budget`, `inherit`                   |
| `gsd-quick`         | `--full`, `--discuss`, `--research`                          |
| `gsd-plan-phase`    | `--auto`, `--research`, `--skip-research`, `--gaps`, `--skip-verify`, `--prd FILE`, `--reviews`, `--text` |
| `gsd-execute-phase` | `--wave N`, `--gaps-only`, `--interactive`                   |
| `gsd-discuss-phase` | `--auto`, `--batch`, `--analyze`, `--text`                   |
| `gsd-new-workspace` | `--name`, `--repos`, `--path`, `--strategy`, `--branch`, `--auto` |
| `gsd-review`        | `--phase`, `--gemini`, `--claude`, `--codex`, `--all`        |
| `gsd-profile-user`  | `--questionnaire`, `--refresh`                               |
| `gsd-health`        | `--repair`                                                   |
| `gsd-new-project`   | `--auto`                                                     |
| `gsd-new-milestone` | `--reset-phase-numbers`                                      |

## Notes

📌 `Args or settings` in the table above are taken from `argument-hint` in
frontmatter when present.

📌 If a skill has `-` in `Args or settings`, its `SKILL.md` either documents no
frontmatter arguments or uses workflow-local interactive input instead.

📌 `Allowed tools` and `Agent` are copied only when explicitly declared in the
skill metadata.

----

---

## Core Workflow

```
/gsd-new-project → /gsd-plan-phase → /gsd-execute-phase → repeat
```

### Project Initialization

**`/gsd-new-project`**
Initialize new project through unified flow.

One command takes you from idea to ready-for-planning:
- Deep questioning to understand what you're building
- Optional domain research (spawns 4 parallel researcher agents)
- Requirements definition with v1/v2/out-of-scope scoping
- Roadmap creation with phase breakdown and success criteria

Creates all `.planning/` artifacts:
- `PROJECT.md` — vision and requirements
- `config.json` — workflow mode (interactive/yolo)
- `research/` — domain research (if selected)
- `REQUIREMENTS.md` — scoped requirements with REQ-IDs
- `ROADMAP.md` — phases mapped to requirements
- `STATE.md` — project memory

Usage: `/gsd-new-project`

**`/gsd-map-codebase`**
Map an existing codebase for brownfield projects.

- Analyzes codebase with parallel Explore agents
- Creates `.planning/codebase/` with 7 focused documents
- Covers stack, architecture, structure, conventions, testing, integrations, concerns
- Use before `/gsd-new-project` on existing codebases

Usage: `/gsd-map-codebase`

### Phase Planning

**`/gsd-discuss-phase <number>`**
Help articulate your vision for a phase before planning.

- Captures how you imagine this phase working
- Creates CONTEXT.md with your vision, essentials, and boundaries
- Use when you have ideas about how something should look/feel
- Optional `--batch` asks 2-5 related questions at a time instead of one-by-one

Usage: `/gsd-discuss-phase 2`
Usage: `/gsd-discuss-phase 2 --batch`
Usage: `/gsd-discuss-phase 2 --batch=3`

**`/gsd-research-phase <number>`**
Comprehensive ecosystem research for niche/complex domains.

- Discovers standard stack, architecture patterns, pitfalls
- Creates RESEARCH.md with "how experts build this" knowledge
- Use for 3D, games, audio, shaders, ML, and other specialized domains
- Goes beyond "which library" to ecosystem knowledge

Usage: `/gsd-research-phase 3`

**`/gsd-list-phase-assumptions <number>`**
See what the agent is planning to do before it starts.

- Shows the agent's intended approach for a phase
- Lets you course-correct if the agent misunderstood your vision
- No files created - conversational output only

Usage: `/gsd-list-phase-assumptions 3`

**`/gsd-plan-phase <number>`**
Create detailed execution plan for a specific phase.

- Generates `.planning/phases/XX-phase-name/XX-YY-PLAN.md`
- Breaks phase into concrete, actionable tasks
- Includes verification criteria and success measures
- Multiple plans per phase supported (XX-01, XX-02, etc.)

Usage: `/gsd-plan-phase 1`
Result: Creates `.planning/phases/01-foundation/01-01-PLAN.md`

**PRD Express Path:** Pass `--prd path/to/requirements.md` to skip discuss-phase entirely. Your PRD becomes locked decisions in CONTEXT.md. Useful when you already have clear acceptance criteria.

### Execution

**`/gsd-execute-phase <phase-number>`**
Execute all plans in a phase, or run a specific wave.

- Groups plans by wave (from frontmatter), executes waves sequentially
- Plans within each wave run in parallel via Task tool
- Optional `--wave N` flag executes only Wave `N` and stops unless the phase is now fully complete
- Verifies phase goal after all plans complete
- Updates REQUIREMENTS.md, ROADMAP.md, STATE.md

Usage: `/gsd-execute-phase 5`
Usage: `/gsd-execute-phase 5 --wave 2`

### Smart Router

**`/gsd-do <description>`**
Route freeform text to the right GSD command automatically.

- Analyzes natural language input to find the best matching GSD command
- Acts as a dispatcher — never does the work itself
- Resolves ambiguity by asking you to pick between top matches
- Use when you know what you want but don't know which `/gsd-*` command to run

Usage: `/gsd-do fix the login button`
Usage: `/gsd-do refactor the auth system`
Usage: `/gsd-do I want to start a new milestone`

### Quick Mode

**`/gsd-quick [--full] [--discuss] [--research]`**
Execute small, ad-hoc tasks with GSD guarantees but skip optional agents.

Quick mode uses the same system with a shorter path:
- Spawns planner + executor (skips researcher, checker, verifier by default)
- Quick tasks live in `.planning/quick/` separate from planned phases
- Updates STATE.md tracking (not ROADMAP.md)

Flags enable additional quality steps:
- `--discuss` — Lightweight discussion to surface gray areas before planning
- `--research` — Focused research agent investigates approaches before planning
- `--full` — Adds plan-checking (max 2 iterations) and post-execution verification

Flags are composable: `--discuss --research --full` gives the complete quality pipeline for a single task.

Usage: `/gsd-quick`
Usage: `/gsd-quick --research --full`
Result: Creates `.planning/quick/NNN-slug/PLAN.md`, `.planning/quick/NNN-slug/SUMMARY.md`

---

**`/gsd-fast [description]`**
Execute a trivial task inline — no subagents, no planning files, no overhead.

For tasks too small to justify planning: typo fixes, config changes, forgotten commits, simple additions. Runs in the current context, makes the change, commits, and logs to STATE.md.

- No PLAN.md or SUMMARY.md created
- No subagent spawned (runs inline)
- ≤ 3 file edits — redirects to `/gsd-quick` if task is non-trivial
- Atomic commit with conventional message

Usage: `/gsd-fast "fix the typo in README"`
Usage: `/gsd-fast "add .env to gitignore"`

### Roadmap Management

**`/gsd-add-phase <description>`**
Add new phase to end of current milestone.

- Appends to ROADMAP.md
- Uses next sequential number
- Updates phase directory structure

Usage: `/gsd-add-phase "Add admin dashboard"`

**`/gsd-insert-phase <after> <description>`**
Insert urgent work as decimal phase between existing phases.

- Creates intermediate phase (e.g., 7.1 between 7 and 8)
- Useful for discovered work that must happen mid-milestone
- Maintains phase ordering

Usage: `/gsd-insert-phase 7 "Fix critical auth bug"`
Result: Creates Phase 7.1

**`/gsd-remove-phase <number>`**
Remove a future phase and renumber subsequent phases.

- Deletes phase directory and all references
- Renumbers all subsequent phases to close the gap
- Only works on future (unstarted) phases
- Git commit preserves historical record

Usage: `/gsd-remove-phase 17`
Result: Phase 17 deleted, phases 18-20 become 17-19

### Milestone Management

**`/gsd-new-milestone <name>`**
Start a new milestone through unified flow.

- Deep questioning to understand what you're building next
- Optional domain research (spawns 4 parallel researcher agents)
- Requirements definition with scoping
- Roadmap creation with phase breakdown
- Optional `--reset-phase-numbers` flag restarts numbering at Phase 1 and archives old phase dirs first for safety

Mirrors `/gsd-new-project` flow for brownfield projects (existing PROJECT.md).

Usage: `/gsd-new-milestone "v2.0 Features"`
Usage: `/gsd-new-milestone --reset-phase-numbers "v2.0 Features"`

**`/gsd-complete-milestone <version>`**
Archive completed milestone and prepare for next version.

- Creates MILESTONES.md entry with stats
- Archives full details to milestones/ directory
- Creates git tag for the release
- Prepares workspace for next version

Usage: `/gsd-complete-milestone 1.0.0`

### Progress Tracking

**`/gsd-progress`**
Check project status and intelligently route to next action.

- Shows visual progress bar and completion percentage
- Summarizes recent work from SUMMARY files
- Displays current position and what's next
- Lists key decisions and open issues
- Offers to execute next plan or create it if missing
- Detects 100% milestone completion

Usage: `/gsd-progress`

### Session Management

**`/gsd-resume-work`**
Resume work from previous session with full context restoration.

- Reads STATE.md for project context
- Shows current position and recent progress
- Offers next actions based on project state

Usage: `/gsd-resume-work`

**`/gsd-pause-work`**
Create context handoff when pausing work mid-phase.

- Creates .continue-here file with current state
- Updates STATE.md session continuity section
- Captures in-progress work context

Usage: `/gsd-pause-work`

### Debugging

**`/gsd-debug [issue description]`**
Systematic debugging with persistent state across context resets.

- Gathers symptoms through adaptive questioning
- Creates `.planning/debug/[slug].md` to track investigation
- Investigates using scientific method (evidence → hypothesis → test)
- Survives `/clear` — run `/gsd-debug` with no args to resume
- Archives resolved issues to `.planning/debug/resolved/`

Usage: `/gsd-debug "login button doesn't work"`
Usage: `/gsd-debug` (resume active session)

### Quick Notes

**`/gsd-note <text>`**
Zero-friction idea capture — one command, instant save, no questions.

- Saves timestamped note to `.planning/notes/` (or `~/.copilot/notes/` globally)
- Three subcommands: **append (default), list, promote**
- Promote converts a note into a structured todo
- Works without a project (falls back to global scope)

Usage: `/gsd-note refactor the hook system`
Usage: `/gsd-note list`
Usage: `/gsd-note promote 3`
Usage: `/gsd-note --global cross-project idea`

### Todo Management

**`/gsd-add-todo [description]`**
Capture idea or task as todo from current conversation.

- Extracts context from conversation (or uses provided description)
- Creates structured todo file in `.planning/todos/pending/`
- Infers area from file paths for grouping
- Checks for duplicates before creating
- Updates STATE.md todo count

Usage: `/gsd-add-todo` (infers from conversation)
Usage: `/gsd-add-todo Add auth token refresh`

**`/gsd-check-todos [area]`**
List pending todos and select one to work on.

- Lists all pending todos with title, area, age
- Optional area filter (e.g., `/gsd-check-todos api`)
- Loads full context for selected todo
- Routes to appropriate action (work now, add to phase, brainstorm)
- Moves todo to done/ when work begins

Usage: `/gsd-check-todos`
Usage: `/gsd-check-todos api`

### User Acceptance Testing

**`/gsd-verify-work [phase]`**
Validate built features through conversational UAT.

- Extracts testable deliverables from SUMMARY.md files
- Presents tests one at a time (yes/no responses)
- Automatically diagnoses failures and creates fix plans
- Ready for re-execution if issues found

Usage: `/gsd-verify-work 3`

### Ship Work

**`/gsd-ship [phase]`**
Create a PR from completed phase work with an auto-generated body.

- Pushes branch to remote
- Creates PR with summary from SUMMARY.md, VERIFICATION.md, REQUIREMENTS.md
- Optionally requests code review
- Updates STATE.md with shipping status

Prerequisites: Phase verified, `gh` CLI installed and authenticated.

Usage: `/gsd-ship 4` or `/gsd-ship 4 --draft`

---

**`/gsd-review --phase N [--gemini] [--claude] [--codex] [--all]`**
Cross-AI peer review — invoke external AI CLIs to independently review phase plans.

- Detects available CLIs (gemini, claude, codex)
- Each CLI reviews plans independently with the same structured prompt
- Produces REVIEWS.md with per-reviewer feedback and consensus summary
- Feed reviews back into planning: `/gsd-plan-phase N --reviews`

Usage: `/gsd-review --phase 3 --all`

---

**`/gsd-pr-branch [target]`**
Create a clean branch for pull requests by filtering out .planning/ commits.

- Classifies commits: code-only (include), planning-only (exclude), mixed (include sans .planning/)
- Cherry-picks code commits onto a clean branch
- Reviewers see only code changes, no GSD artifacts

Usage: `/gsd-pr-branch` or `/gsd-pr-branch main`

---

**`/gsd-plant-seed [idea]`**
Capture a forward-looking idea with trigger conditions for automatic surfacing.

- Seeds preserve WHY, WHEN to surface, and breadcrumbs to related code
- Auto-surfaces during `/gsd-new-milestone` when trigger conditions match
- Better than deferred items — triggers are checked, not forgotten

Usage: `/gsd-plant-seed "add real-time notifications when we build the events system"`

---

**`/gsd-audit-uat`**
Cross-phase audit of all outstanding UAT and verification items.
- Scans every phase for pending, skipped, blocked, and human_needed items
- Cross-references against codebase to detect stale documentation
- Produces prioritized human test plan grouped by testability
- Use before starting a new milestone to clear verification debt

Usage: `/gsd-audit-uat`

### Milestone Auditing

**`/gsd-audit-milestone [version]`**
Audit milestone completion against original intent.

- Reads all phase VERIFICATION.md files
- Checks requirements coverage
- Spawns integration checker for cross-phase wiring
- Creates MILESTONE-AUDIT.md with gaps and tech debt

Usage: `/gsd-audit-milestone`

**`/gsd-plan-milestone-gaps`**
Create phases to close gaps identified by audit.

- Reads MILESTONE-AUDIT.md and groups gaps into phases
- Prioritizes by requirement priority (must/should/nice)
- Adds gap closure phases to ROADMAP.md
- Ready for `/gsd-plan-phase` on new phases

Usage: `/gsd-plan-milestone-gaps`

### Configuration

**`/gsd-settings`**
Configure workflow toggles and model profile interactively.

- Toggle researcher, plan checker, verifier agents
- Select model profile (quality/balanced/budget/inherit)
- Updates `.planning/config.json`

Usage: `/gsd-settings`

**`/gsd-set-profile <profile>`**
Quick switch model profile for GSD agents.

- `quality` — Opus everywhere except verification
- `balanced` — Opus for planning, Sonnet for execution (default)
- `budget` — Sonnet for writing, Haiku for research/verification
- `inherit` — Use current session model for all agents (OpenCode `/model`)

Usage: `/gsd-set-profile budget`

### Utility Commands

**`/gsd-cleanup`**
Archive accumulated phase directories from completed milestones.

- Identifies phases from completed milestones still in `.planning/phases/`
- Shows dry-run summary before moving anything
- Moves phase dirs to `.planning/milestones/v{X.Y}-phases/`
- Use after multiple milestones to reduce `.planning/phases/` clutter

Usage: `/gsd-cleanup`

**`/gsd-help`**
Show this command reference.

**`/gsd-update`**
Update GSD to latest version with changelog preview.

- Shows installed vs latest version comparison
- Displays changelog entries for versions you've missed
- Highlights breaking changes
- Confirms before running install
- Better than raw `npx get-shit-done-cc`

Usage: `/gsd-update`

**`/gsd-join-discord`**
Join the GSD Discord community.

- Get help, share what you're building, stay updated
- Connect with other GSD users

Usage: `/gsd-join-discord`

## Files & Structure

```
.planning/
├── PROJECT.md            # Project vision
├── ROADMAP.md            # Current phase breakdown
├── STATE.md              # Project memory & context
├── RETROSPECTIVE.md      # Living retrospective (updated per milestone)
├── config.json           # Workflow mode & gates
├── todos/                # Captured ideas and tasks
│   ├── pending/          # Todos waiting to be worked on
│   └── done/             # Completed todos
├── debug/                # Active debug sessions
│   └── resolved/         # Archived resolved issues
├── milestones/
│   ├── v1.0-ROADMAP.md       # Archived roadmap snapshot
│   ├── v1.0-REQUIREMENTS.md  # Archived requirements
│   └── v1.0-phases/          # Archived phase dirs (via /gsd-cleanup or --archive-phases)
│       ├── 01-foundation/
│       └── 02-core-features/
├── codebase/             # Codebase map (brownfield projects)
│   ├── STACK.md          # Languages, frameworks, dependencies
│   ├── ARCHITECTURE.md   # Patterns, layers, data flow
│   ├── STRUCTURE.md      # Directory layout, key files
│   ├── CONVENTIONS.md    # Coding standards, naming
│   ├── TESTING.md        # Test setup, patterns
│   ├── INTEGRATIONS.md   # External services, APIs
│   └── CONCERNS.md       # Tech debt, known issues
└── phases/
    ├── 01-foundation/
    │   ├── 01-01-PLAN.md
    │   └── 01-01-SUMMARY.md
    └── 02-core-features/
        ├── 02-01-PLAN.md
        └── 02-01-SUMMARY.md
```

## Workflow Modes

Set during `/gsd-new-project`:

**Interactive Mode**

- Confirms each major decision
- Pauses at checkpoints for approval
- More guidance throughout

**YOLO Mode**

- Auto-approves most decisions
- Executes plans without confirmation
- Only stops for critical checkpoints

Change anytime by editing `.planning/config.json`

## Planning Configuration

Configure how planning artifacts are managed in `.planning/config.json`:

**`planning.commit_docs`** (default: `true`)
- `true`: Planning artifacts committed to git (standard workflow)
- `false`: Planning artifacts kept local-only, not committed

When `commit_docs: false`:
- Add `.planning/` to your `.gitignore`
- Useful for OSS contributions, client projects, or keeping planning private
- All planning files still work normally, just not tracked in git

**`planning.search_gitignored`** (default: `false`)
- `true`: Add `--no-ignore` to broad ripgrep searches
- Only needed when `.planning/` is gitignored and you want project-wide searches to include it

Example config:
```json
{
  "planning": {
    "commit_docs": false,
    "search_gitignored": true
  }
}
```

## Common Workflows

**Starting a new project:**

```
/gsd-new-project        # Unified flow: questioning → research → requirements → roadmap
/clear
/gsd-plan-phase 1       # Create plans for first phase
/clear
/gsd-execute-phase 1    # Execute all plans in phase
```

**Resuming work after a break:**

```
/gsd-progress  # See where you left off and continue
```

**Adding urgent mid-milestone work:**

```
/gsd-insert-phase 5 "Critical security fix"
/gsd-plan-phase 5.1
/gsd-execute-phase 5.1
```

**Completing a milestone:**

```
/gsd-complete-milestone 1.0.0
/clear
/gsd-new-milestone  # Start next milestone (questioning → research → requirements → roadmap)
```

**Capturing ideas during work:**

```
/gsd-add-todo                    # Capture from conversation context
/gsd-add-todo Fix modal z-index  # Capture with explicit description
/gsd-check-todos                 # Review and work on todos
/gsd-check-todos api             # Filter by area
```

**Debugging an issue:**

```
/gsd-debug "form submission fails silently"  # Start debug session
# ... investigation happens, context fills up ...
/clear
/gsd-debug                                    # Resume from where you left off
```

## Getting Help

- Read `.planning/PROJECT.md` for project vision
- Read `.planning/STATE.md` for current context
- Check `.planning/ROADMAP.md` for phase status
- Run `/gsd-progress` to check where you're up to
</reference>
