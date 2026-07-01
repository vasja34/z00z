# Phase 030: refactor-long-files - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `030-CONTEXT.md` — this log preserves the alternatives considered.

**Date:** 2026-03-31
**Phase:** 030-refactor-long-files
**Areas discussed:** Split Policy, Refactor Sequencing, Public Path Cleanup, Verification Gates, Documentation And YAML Sync

---

## Split Policy

| Option | Description | Selected |
| --- | --- | --- |
| Strict policy | Root file stays a thin facade around ~300 lines; helpers and stateful logic move out aggressively. | |
| Moderate policy | Facade may stay around ~500 lines when dominated by rustdoc, re-exports, or orchestration. | |
| Soft policy | Main goal is to break extreme monoliths; facades may stay large if still readable. | ✓ |

**User's choice:** Soft policy.
**Follow-up:** The facade cap was clarified to `up to 500 lines`, not an unbounded soft policy.

## Refactor Sequencing

| Option | Description | Selected |
| --- | --- | --- |
| Dependency-first by crate | Start from seams that unlock downstream splits inside each crate. | ✓ |
| Strict top-5 order | Follow the `030-todo.md` backlog in exactly the listed order. | |
| Wallets-first sweep | Finish the `z00z_wallets` giant files first, then move outward. | |

**User's choice:** Dependency-first by crate.
**Notes:** The `First Five Candidates` list remains the seed backlog, but not a hard global execution order.

## Public Path Cleanup

| Option | Description | Selected |
| --- | --- | --- |
| Preserve public paths | Prefer facades and re-exports so external paths stay stable. | |
| Limited cleanup | Allow selective path cleanup where the value is high. | |
| Full cleanup | Normalize the structure fully even if public paths must change. | ✓ |

**User's choice:** Full cleanup.
**Follow-up:** Public path breaks may be handled as a one-wave mass rename without compatibility shims.

## Verification Gates

| Option | Description | Selected |
| --- | --- | --- |
| Hybrid gate | Bootstrap fail-fast, then targeted crate tests, then broader release-style gates per wave or per risky split. | ✓ |
| Maximum strictness everywhere | Run bootstrap plus wide release-style gates after every major split step. | |
| Local gate only | Use mostly targeted crate tests and defer wide gates to the end of the phase. | |

**User's choice:** Hybrid gate.
**Notes:** The bootstrap gate remains mandatory for every implementation wave.

## Documentation And YAML Sync

| Option | Description | Selected |
| --- | --- | --- |
| Same-wave sync required | Docs, YAML, planning refs, and structure changes ship together. | ✓ |
| End-of-phase sync | Collect doc and YAML cleanup in a final dedicated plan. | |
| Runtime-facing sync only | Update only user-facing and executable docs immediately; defer the rest. | |

**User's choice:** Same-wave sync required.
**Notes:** A split wave is not considered done if its module naming or path changes are not reflected in affected docs or YAML.

## the agent's Discretion

- 📌 Exact helper module names.
- 📌 Whether a seam becomes sibling files or a directory facade.
- 📌 Exact plan granularity, as long as dependency-first sequencing and hybrid verification remain intact.

## Deferred Ideas

- None.
