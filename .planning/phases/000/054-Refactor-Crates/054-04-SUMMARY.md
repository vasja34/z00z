---
phase: 054-Refactor-Crates
plan: 054-04
status: complete
completed_at: 2026-06-08
next_plan: 054-05
requirements-completed: [PH54-04]
---

# 054-04 Summary

## Outcome

Phase `054-04` is complete.

Runtime placement and shard-execution authority now have explicit homes under
`crates/z00z_runtime/aggregators/src/placement.rs` and
`crates/z00z_runtime/aggregators/src/shard_exec.rs`. Validators, watchers, and
`z00z_rollup_node` consume those surfaces only as downstream operational
metadata. Placement state did not become verifier-visible truth, planner
authority, or a second orchestration root.

## Landed Changes

- Added `crates/z00z_runtime/aggregators/src/placement.rs` with:
  - `AggregatorId`
  - `StandbyState`
  - `ShardPlacement`
  - `ShardPlacementView`
  - `ShardPlacementTable`
- Added `crates/z00z_runtime/aggregators/src/shard_exec.rs` with:
  - `ShardExecState`
  - `ShardExecTicket`
  - `ShardExecutor`
- Extended `crates/z00z_runtime/aggregators/src/agg_types.rs` with
  `BatchId::from_bytes` so placement and executor metadata can rehydrate
  canonical batch identity without introducing new authority seams.
- Re-exported the runtime placement and shard-execution surfaces from
  `crates/z00z_runtime/aggregators/src/lib.rs`.
- Rebased validator, watcher, and node adoption onto the new runtime-owned
  surfaces without moving planner or truth ownership out of their existing
  boundaries.
- Canonicalized downstream status projection so placement state prefers the
  `exec_ticket` view when both independent placement and execution snapshots are
  present, removing contradictory public status drift.

## Validation

Executed and passed on the current tree:

- `cargo fmt --all`
- `cargo test -p z00z_aggregators --release`
- `cargo test -p z00z_validators --release`
- `cargo test -p z00z_watchers --release`
- `cargo test -p z00z_rollup_node --release`
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`

The plan-mandated broad workspace command is stale against the live manifest on
this repository state:

- `cargo test --release --features test-fast --features wallet_debug_dump`

Observed failure:

- `error: none of the selected packages contains these features: test-fast, wallet_debug_dump`

The live-equivalent release evidence for this slice is therefore the
crate-scoped release validation above, plus the green bootstrap gate.

## Review Loop

- Review pass 1 found contradictory independent placement and execution status
  exposure; the canonical downstream projection was tightened around
  `exec_ticket`.
- Review pass 2 checked validator, watcher, and node adoption boundaries and
  confirmed the new runtime surfaces did not become planner or truth authority.
- Review pass 3 reran the targeted release suites plus bootstrap and found no
  significant issues.
- Review pass 4 rechecked source shape, downstream status coherence, and
  `git diff --check`; no significant issues remained, giving the required
  consecutive clean closure.

## Closeout

The runtime placement and shard-execution boundary split is now summary-backed
complete. Phase `054-05` is the active next lane for storage canonical-module
cleanup and the remaining bridge-debt removal.
