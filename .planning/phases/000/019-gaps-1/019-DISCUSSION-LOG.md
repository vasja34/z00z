<!-- markdownlint-disable MD001 MD022 MD032 MD033 MD047 -->
# Phase 019: 019-gaps-1 - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution
> agents. Decisions are captured in `019-CONTEXT.md`.

**Date:** 2026-03-24
**Phase:** 019-gaps-1
**Areas discussed:** Nullifier boundary, Receive taxonomy contract, Backup
restore target, Auxiliary cleanup guardrail

---

## Nullifier boundary

| Option | Description | Selected |
| ------ | ----------- | -------- |
| Storage-owned canonical namespace | Move nullifier state toward a storage-owned canonical contract. | ✓ |
| Checkpoint-coupled adjunct state | Keep nullifier state separate from root, but checkpoint-bound. | |
| Document current local semantics only | Formalize the current local contract only. | |
| You decide | Leave the choice to planner discretion. | |

**User's choice:** Storage-owned canonical namespace.
**Notes:** The mandatory invariant was also locked as atomic asset-plus-
nullifier commit.

---

## Receive taxonomy contract

| Option | Description | Selected |
| ------ | ----------- | -------- |
| Report-first runtime API | Public runtime receive path returns explicit taxonomy. | ✓ |
| Service-only protection | Keep low-level scanner semantics and rely on service precheck. | |
| Canonical-only report surface | Tighten only the canonical leaf path. | |
| You decide | Leave the choice to planner discretion. | |

**User's choice:** Report-first runtime API.
**Notes:** The user also locked that malformed runtime input must not degrade
 into `NotMine` or `MaybeMine` on the public path.

---

## Backup restore target

| Option | Description | Selected |
| ------ | ----------- | -------- |
| Full restore on `WalletExportPack` | Public backup converges on the existing full export/import path. | ✓ |
| Snapshot plus rescan contract | Restore snapshot state, then rescan missing runtime tails. | |
| Metadata-only legacy as norm | Keep the public backup contract limited. | |

**User's choice:** Full restore on `WalletExportPack`.
**Notes:** Metadata-only backup behavior remains legacy/read-compatible, not
 the long-term promise.

---

## Auxiliary cleanup guardrail

| Option | Description | Selected |
| ------ | ----------- | -------- |
| Only three primary gaps | No adjacent cleanup inside the phase. | |
| Only tightly related refactors | Allow narrow refactors strictly needed by the three gaps. | ✓ |
| Allow broader cleanup | Also take RNG and docs cleanup in the same phase. | |

**User's choice:** Only tightly related refactors.
**Notes:** Broader RNG-provider and documentation cleanup remains deferred
 unless directly required by one of the primary contracts.

---