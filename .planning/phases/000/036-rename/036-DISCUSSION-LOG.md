# Phase 036: Rename - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-04-14
**Phase:** 036-rename
**Areas discussed:** canonical planning authority, task coverage, sequential execution, anti-drift constraints

---

## Canonical planning authority

| Option | Description | Selected |
| ------ | ----------- | -------- |
| Use `036-TODO-1.md` as the canonical planning inventory | Keep the phase anchored to the validated backlog already present in the pre-existing phase directory | ✓ |
| Use another TODO file or copied backlog | Introduce a competing planning source inside the same phase directory | |
| Let the planner synthesize a new planning surface | Create a planner-invented layer that competes with the existing canonical backlog | |

**User's choice:** Use `036-TODO-1.md` as the canonical planning inventory for Phase 036.
**Notes:** The user explicitly stated that `036-TODO-1.md` is canonical and that planning must schedule the specific tasks from that file.

---

## Task coverage and wording

| Option | Description | Selected |
| ------ | ----------- | -------- |
| Preserve every canonical task exactly as written | Keep full coverage, frozen task titles, and frozen wording | ✓ |
| Normalize or merge similar tasks | Reduce or reshape the backlog for convenience | |
| Exclude lower-priority tasks | Allow silent scope reduction during planning | |

**User's choice:** Preserve every canonical task exactly as written.
**Notes:** The user explicitly forbade changing task titles or wording and required that none of the table tasks be excluded except under an explicitly recorded principle-level blocker.

---

## Sequential execution discipline

| Option | Description | Selected |
| ------ | ----------- | -------- |
| Plan one canonical task after another in the existing order | Keep serial execution aligned with the canonical backlog | ✓ |
| Parallelize multiple canonical tasks | Change the execution discipline for convenience | |
| Reorder tasks by perceived implementation efficiency | Override the backlog's fixed sequence | |

**User's choice:** Plan one canonical task after another in the existing order.
**Notes:** The user explicitly required sequential planning and execution ordering, one canonical task after another.

---

## Anti-drift constraints

| Option | Description | Selected |
| ------ | ----------- | -------- |
| Preserve live seams and avoid duplicate layers | Keep planning tied to the existing codebase and existing phase-local sources | ✓ |
| Introduce a parallel planning or implementation layer | Create duplicate logic or competing abstractions | |
| Expand scope beyond the canonical phase inventory | Allow concept drift during planning | |

**User's choice:** Preserve live seams and avoid duplicate layers.
**Notes:** The user explicitly required no duplicate codebase logic, no parallel layer, and explicit prevention of codebase concept drift.

---

## the agent's Discretion

- Format of the future `036-*-PLAN.md` files.
- Amount of added traceability and validation detail, as long as canonical task wording and order remain unchanged.

## Deferred Ideas

None.
