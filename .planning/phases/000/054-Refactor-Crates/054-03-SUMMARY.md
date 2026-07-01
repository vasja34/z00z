---
phase: 054-Refactor-Crates
plan: 054-03
status: complete
completed_at: 2026-06-08
next_plan: 054-04
requirements-completed: [PH54-03]
---

# 054-03 Summary

## Outcome

Phase `054-03` is complete.

Runtime planner authority now has an explicit home in
`crates/z00z_runtime/aggregators/src/batch_planner.rs`, while storage
`tx_plan` is narrowed back to store-local precheck, snapshot, and semantic
dry-run machinery. The new runtime planner owns canonical item ordering,
single-shard route admission, one-shard route-table targeting, and canonical
plan digest generation without turning runtime metadata into storage truth.

## Landed Changes

- Added `crates/z00z_runtime/aggregators/src/batch_planner.rs` with:
  - `ShardRouteTable`
  - `RouteRangeRule`
  - `BatchPlanner`
  - `RouteErr`
  - canonical route-table digest and batch-plan digest generation
  - current one-shard compatibility admission and multi-shard rejection
- Extended `crates/z00z_runtime/aggregators/src/agg_types.rs` with runtime
  planner-owned metadata:
  - `ShardId`
  - `PlanDigest`
  - `BatchRoute`
  - `BatchPlanned`
  - `OrderedBatch.planned`
- Reworked `crates/z00z_runtime/aggregators/src/agg_ordering.rs` so
  `OrderingBoundary` delegates batch planning and batch construction through
  `BatchPlanner` instead of remaining a passive item copier.
- Re-exported the new runtime planning surface from
  `crates/z00z_runtime/aggregators/src/lib.rs` without renaming the existing
  public runtime facade traits.
- Narrowed storage planning helpers:
  - `crates/z00z_storage/src/settlement/tx_plan_help.rs` now returns touched
    paths plus `NextState` instead of exposing planner-scoped `PlanScope`.
  - `crates/z00z_storage/src/settlement/tx_plan_types.rs` now keeps only the
    store-local `StoreSnap`, `SeenOps`, and `NextState` machinery.
  - `crates/z00z_storage/src/settlement/hjmt_plan.rs` now keeps the
    HJMT-internal `PlanScope`, `ShardKey`, and `ShardItem` names private to the
    storage commit engine instead of leaving them in the broader `tx_plan`
    helper surface.

## Validation

Executed and passed on the current tree:

- `cargo fmt --all`
- `cargo test -p z00z_aggregators --release --no-run`
- `cargo test -p z00z_storage --release --no-run`
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_aggregators --release`
- `cargo test -p z00z_storage --release --features test-params-fast`

The plan-mandated broad workspace command is stale against the live manifest on
this repository state:

- `cargo test --release --features test-fast --features wallet_debug_dump`

Observed failure:

- `error: none of the selected packages contains these features: test-fast, wallet_debug_dump`

The live-equivalent release evidence for this slice is therefore the
crate-scoped release validation above, plus the green bootstrap gate.

## Review Loop

- Review pass 1 inspected the runtime and storage diff to confirm that planner
  authority moved into the runtime crate while storage kept semantic truth
  ownership; no significant issues remained.
- Review pass 2 checked source-shape constraints and confirmed that
  `PlanScope`, `ShardKey`, and `ShardItem` no longer appear in the storage
  `tx_plan` helper files; no significant issues remained.
- Review pass 3 checked for runtime leakage of storage seam symbols and ran
  `git diff --check`; no significant issues remained.
- Review pass 4 rechecked downstream drift boundaries and confirmed that the
  new planner-only runtime types did not leak into validators, watchers, or
  rollup node code; no significant issues remained.

## Closeout

The runtime batch planner split is now summary-backed complete. Phase `054-04`
is the active next lane for placement and shard-execution surfaces plus bounded
validator, watcher, and node adoption.
