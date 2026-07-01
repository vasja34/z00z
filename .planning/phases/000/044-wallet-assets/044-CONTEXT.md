# 044-CONTEXT

## Phase Scope

Phase 044 is wallet-centric and reuses the pre-existing
`.planning/phases/044-wallet-assets/` directory. The phase authority is the
self-contained [044-TODO.md](./044-TODO.md); the spec and patch files remain
archived source inputs already embedded into that backlog.

## Source Inputs

- [044-TODO.md](./044-TODO.md)
- [044-wallets-assets-spec.md](./044-wallets-assets-spec.md)
- [044-wallets-patch.md](./044-wallets-patch.md)

## Locked Invariants

- `.wlt` stays wallet-state-only and must not become the tx package history
  store.
- `wallet_<stem>_tx_history.jsonl` is the canonical live tx-history store for
  the wallet stem.
- Phase 044 must not introduce a broad new wallet database for tx packages.
- Exact canonical tx package bytes must be preserved in history, backup,
  restore, import, export, and reconciliation flows.
- Legacy `wallet_<stem>_tx_history/` directories are migration input only.
- Report-only receive stays non-persistent.
- Simulated admission must remain explicit and trait-backed.
- Reconciliation must stay storage-backed and idempotent.
- No edits may touch `crates/z00z_crypto/tari/**`.

## Plan Map

| Plan | Task groups | Focus |
| --- | --- | --- |
| `044-01` | `044-01`, `044-02`, `044-03` | coverage ledger, asset lifecycle, sender build/send |
| `044-02` | `044-04`, `044-04A`, `044-04B` | tx journal, canonical JSONL authority, JSONL storage |
| `044-03` | `044-04C`, `044-04D`, `044-05` | backup/restore, migration guards, portable submission |
| `044-04` | `044-06`, `044-07`, `044-08` | admission, reconciliation, receiver finalization |
| `044-05` | `044-09`, `044-10` | balance views, regression matrix, source-shape guards |

## Execution Order

1. Build the coverage ledger and core wallet lifecycle foundation first.
2. Lock the tx journal and live JSONL authority second.
3. Add backup, restore, migration, and portable submission third.
4. Land explicit admission, storage-backed reconciliation, and receiver
   finalization fourth.
5. Finish with balance views, source-shape guards, coverage sync, and phase
   summary closeout.

## Cross-Cutting Rules

- Preserve exact task names and wording from [044-TODO.md](./044-TODO.md).
- Keep every plan at or below four TODO task groups.
- Keep each task executable on its own with concrete files, actions, and
  verification.
- Carry the task-local test impact matrix into the final regression wave.
- Keep `044-coverage.md` and `044-SUMMARY.md` as required closeout artifacts.

## TODO Coverage Crosswalk

This is a navigation crosswalk only. It does not introduce a second authority;
it shows where each top-level block in [044-TODO.md](./044-TODO.md) is
represented in the phase plan and test artifacts.

| `044-TODO.md` block | Reflected in context / plan |
| --- | --- |
| `Decision Summary` | `Locked Invariants`, `Drift Bars`, and the plan map keep the phase wallet-centric, JSONL-backed, and non-duplicative. |
| `Dependency Chain` | `Execution Order` and the plan dependencies in `044-01` through `044-05`. |
| `File-First Implementation Order` | `Plan Map` plus the per-task file lists in each `044-0N-PLAN.md`. |
| `Validation Matrix` | The verify blocks in each plan task, especially `044-01`, `044-05`, and the test artifacts linked below. |
| `Embedded Requirement Corpus` | `Cross-Cutting Rules`, `Drift Bars`, and the per-task requirement summaries embedded in the plan waves. |
| `Full Source Coverage Index` | `044-01-PLAN.md` through `044-05-PLAN.md`, plus `044-coverage.md` as the implementation-time ledger target. |
| `Existing Test Impact Matrix` | `044-05-PLAN.md` task 2, the `044-TEST-SPEC.md` existing-test appendix, and `044-TESTS-TASKS.md` preserve the existing-test audit trail. |
| `Explicit Phase Boundary` | `Locked Invariants` and `Drift Bars` forbid the out-of-phase shapes and duplicate authorities. |
| `Concrete Execution Tasks` | The task groups in `044-01-PLAN.md` through `044-05-PLAN.md`. |
| `Completion Gate` | `044-05-PLAN.md`, `044-TESTS-TASKS.md`, and the required `044-coverage.md` / `044-SUMMARY.md` closeout artifacts. |

## Drift Bars

- Do not introduce a parallel authority layer, duplicate tx schema, duplicate
  assembler, duplicate verifier, or duplicate receiver claim path.
- Keep the existing test impact matrix authoritative for the final regression
  wave; any uncovered existing test file must be handled as explicit no-change
  evidence rather than silently omitted.
- Treat `BuiltTxStub`, fake broadcast success, empty tx detail rows,
  `pending = 0` drift, report-only persistence drift, and per-tx JSON live
  storage regressions as forbidden shapes that the final wave must guard
  against.
- Preserve the TODO completion gate: coverage ledger, summary artifacts,
  canonical JSONL history, backup/restore byte preservation, and legacy
  live-store rejection semantics.

## Test Artifacts

- [044-TEST-SPEC.md](./044-TEST-SPEC.md)
- [044-TESTS-TASKS.md](./044-TESTS-TASKS.md)
