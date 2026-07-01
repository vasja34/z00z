---
phase: 056-HJMT-storage-aggregator
plan: 056-03
status: complete
completed_at: 2026-06-12
next_plan: 056-04
requirements-completed:
  - 056-G5
  - 056-G6
summary_artifact_for: .planning/phases/056-HJMT-storage- aggregator/056-03-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 056-03 Summary: Semantic Runtime-To-Storage Handoff And Dynamic Scope Birth

## Completed Scope

`056-03` is complete for the live semantic-handoff and dynamic-scope-birth
slice.

The runtime/storage boundary now exposes one semantic
`SettlementExecHandoff` path from `z00z_runtime/aggregators` into storage,
carrying committed batch id, shard id, routing generation, and
`route_table_digest` without promoting runtime code into subtree, bucket, or
proof authority. Storage remains the only owner of scope derivation, subtree
lifecycle, root transition, and checkpoint execution truth, and
`scope_flow.json` is now the required evidence surface for first-seen scope
creation and post-commit progression.

The landed storage path now proves first-seen `definition_id`, first-seen
`serial_id`, first terminal creation, first right creation, mixed
existing/new-scope batches, and reload continuity after the first
scope-creating batch. It also rejects duplicate terminal ids, path mismatch,
leaf mismatch, orphan checkpoint exec rows, and mixed terminal/non-terminal
batches that try to carry checkpoint exec evidence.

## Files Changed

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-03-SUMMARY.md`
- `crates/z00z_runtime/aggregators/src/service.rs`
- `crates/z00z_runtime/aggregators/src/types.rs`
- `crates/z00z_runtime/aggregators/tests/test_live_guardrails.rs`
- `crates/z00z_storage/src/backend/common/rows.rs`
- `crates/z00z_storage/src/settlement/README.md`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs`
- `crates/z00z_storage/src/settlement/mod.rs`
- `crates/z00z_storage/src/settlement/model.rs`
- `crates/z00z_storage/src/settlement/store.rs`
- `crates/z00z_storage/tests/test_hjmt_scope_birth.rs`
- `crates/z00z_storage/tests/test_live_guardrails.rs`

## Boundary Kept Intact

- Runtime submits semantic `StoreOp` rows plus committed route context only;
  it does not compute subtree ids, bucket derivation, or proof truth.
- Storage remains the only semantic/proof authority; no second planner/storage
  registry or mirror tree was introduced.
- `scope_flow.json` stays evidence-only and records semantic facts such as
  `batch_id`, `tx_id`, `shard_id`, routing generation, route digest,
  first-seen markers, and root progression without exporting `HjmtTreeId` as
  protocol truth.
- Checkpoint exec evidence stays tied to terminal-only settlement ops; mixed
  terminal/right semantic batches use the same handoff API but cannot silently
  downgrade attested exec rows into unchecked storage mutations.
- The runtime canonical path stays inside the existing aggregator/storage seams;
  no parallel staging or simulator-only handoff layer was added.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md`
was used because the slash prompt is not a callable tool in this environment.

- Pass 1 found two significant issues: mixed terminal/right batches with
  checkpoint exec rows could fall back to plain semantic apply without a
  fail-closed reject, and the new `scope_flow.json` test path had drifted to
  direct filesystem I/O instead of repository I/O helpers. Both were fixed.
- Pass 2 found two remaining issues: orphan checkpoint exec rows could be
  accepted without terminal settlement ops, and the `scope_flow.json`
  assertions still did not prove explicit `tx_id` and `route_table_digest`
  coverage. Both were fixed.
- Pass 3 reran a literal coverage scan against `056-TODO.md`,
  `056-03-PLAN.md`, runtime/storage README guards, and the phase-owned test
  matrix for first-seen birth, mixed scope batches, duplicate terminal
  rejection, path/leaf mismatch rejection, and reload continuity. No
  significant issues remained.
- Pass 4 repeated the residue scan for semantic-only handoff, first-seen
  singularity within one batch, `HjmtTreeId` privacy, and checkpoint exec
  fail-closed behavior. No significant issues remained.

Two consecutive clean review passes were achieved on passes 3 and 4 after the
Pass 1 and Pass 2 fixes.

## Validation

Rust validation for this plan completed on the live tree before closeout.

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  as the mandatory fail-fast gate.
- `cargo test -p z00z_aggregators --release --features test-params-fast`
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_scope_birth`
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_live_guardrails`
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast` passed.
- `cargo test --release` passed for the workspace.
- `git diff --check` is clean.

## Result

`056-03` is complete. Phase 056 now advances to `056-04-PLAN.md` for the
journal-lineage, restart, and lawful-failover slice.

This summary does not claim journal/WAL baseline closeout, same-lineage
failover acceptance, split-brain fencing, startup preflight, or simulator
evidence; those remain owned by `056-04` through `056-07`.
