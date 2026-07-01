---
phase: 056-HJMT-storage-aggregator
plan: 056-04
status: complete
completed_at: 2026-06-12
next_plan: 056-05
requirements-completed:
  - 056-G7
  - 056-G8
summary_artifact_for: .planning/phases/056-HJMT-storage- aggregator/056-04-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 056-04 Summary: Journal Lineage, Restart, And Lawful Failover

## Completed Scope

`056-04` is complete for the live journal-lineage, restart, and lawful-failover
slice.

The runtime/storage seam now closes on one current durability baseline and one
lawful takeover rule. The RedB-backed local durable journal remains the V1
baseline behind `JournalBackend`, `StoragePlane` now implements that seam
directly, and the recovery surface exports one serializable
`SettlementRecoveryState` contract carrying route-compatible root, generation,
proof-policy, and journal-lineage metadata without creating a second protocol
truth layer.

The runtime failover boundary now preserves `ShardPlacement`,
`PublicationState`, checkpoint handoff metadata, journal lineage, and backend
generation truth across restart/import-export roundtrips. Same-lineage standby
takeover is accepted only when shard id, routing generation, local root, and
expected journal lineage all match. Wrong lineage, wrong generation, stale
local root, stale restart, split-brain, standby-down, and route migration
during crash all reject fail-closed, and carry-forward publication handoff
metadata is explicitly preserved across recovery export/import so Phase 057 can
build on one canonical lineage path rather than a parallel restart story.

The landed coverage also proves crash/restart continuity around the first
scope-creating batch on the live storage seam, keeps the journal baseline local
to the current backend authority, and records guardrails that forbid a shared
cross-aggregator WAL from becoming present-tense protocol truth.

## Files Changed

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-04-PLAN.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-04-SUMMARY.md`
- `crates/z00z_rollup_node/src/config.rs`
- `crates/z00z_rollup_node/src/runtime.rs`
- `crates/z00z_runtime/aggregators/Cargo.toml`
- `crates/z00z_runtime/aggregators/README.md`
- `crates/z00z_runtime/aggregators/src/lib.rs`
- `crates/z00z_runtime/aggregators/src/placement.rs`
- `crates/z00z_runtime/aggregators/src/recovery.rs`
- `crates/z00z_runtime/aggregators/src/types.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_split_brain_fencing.rs`
- `crates/z00z_runtime/aggregators/tests/test_live_guardrails.rs`
- `crates/z00z_runtime/aggregators/tests/test_recovery_common.rs`
- `crates/z00z_runtime/watchers/src/status.rs`
- `crates/z00z_storage/src/backend/mod.rs`
- `crates/z00z_storage/src/backend/redb/mod.rs`
- `crates/z00z_storage/src/settlement/README.md`
- `crates/z00z_storage/src/settlement/mod.rs`
- `crates/z00z_storage/src/settlement/store.rs`
- `crates/z00z_storage/src/settlement/test_live_recovery.rs`
- `crates/z00z_storage/tests/test_live_guardrails.rs`

## Boundary Kept Intact

- RedB local journal durability remains the active baseline; `JournalBackend`
  is an extension seam, not a second source of truth.
- Runtime recovery keeps placement, lineage, and publication handoff metadata
  on the existing aggregator/storage seam only; it does not compute new proof
  or publication truth.
- Same-lineage takeover is the only lawful failover path in this slice; silent
  reroute and split-brain fallback remain forbidden.
- Restart/import-export evidence stays additive to the live storage authority;
  no shared cross-aggregator WAL or replicated-truth layer was introduced.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md`
was used because the slash prompt is not a callable tool in this environment.

- Pass 1 found two significant issues: the new lineage field was still missing
  from downstream `ShardPlacementView` literals in `z00z_watchers` and
  `z00z_rollup_node`, and the full workspace gate therefore did not prove one
  canonical failover surface. Both were fixed.
- Pass 2 found one remaining evidence gap: the failover slice still did not
  prove carry-forward publication handoff metadata survives recovery
  export/import. The new publication handoff roundtrip test was added and the
  guardrail docs were updated.
- Pass 3 found one planning drift issue: `056-04-PLAN.md` did not explicitly
  preserve the literal TODO wording `carry-forward publication handoff`, even
  though `056-CONTEXT.md` mapped that requirement to `056-04` and `056-07`.
  The plan coverage contract and acceptance text were updated.
- Pass 4 reran a repo-first scan against `056-TODO.md`, `056-CONTEXT.md`,
  `056-04-PLAN.md`, the recovery/failover tests, and the runtime/storage
  guardrails. No significant issues remained.
- Pass 5 repeated the same residue scan for same-lineage takeover, reject
  matrix completeness, publication handoff continuity, and no-parallel-WAL
  drift. No significant issues remained.

Two consecutive clean review passes were achieved on passes 4 and 5 after the
Pass 1 through Pass 3 fixes.

## Validation

Rust validation for this plan completed on the live tree before closeout.

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  as the mandatory fail-fast gate.
- `cargo test -p z00z_aggregators --release --features test-params-fast`
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast` passed.
- `cargo test -p z00z_watchers --release` passed.
- `cargo test -p z00z_rollup_node --release` passed.
- `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_failover_same_lineage`
  passed.
- `cargo test --release` passed for the workspace.
- `git diff --check` is clean.

## Result

`056-04` is complete. Phase 056 now advances to `056-05-PLAN.md` for the
YAML-materialization and startup-preflight slice.

This summary does not claim multi-process topology closeout beyond the already
completed `056-01` slice, simulator runtime evidence closeout, or final
fixture/benchmark packet closure; those remain owned by `056-05` through
`056-07`.
