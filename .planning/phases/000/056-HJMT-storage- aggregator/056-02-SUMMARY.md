---
phase: 056-HJMT-storage-aggregator
plan: 056-02
status: complete
completed_at: 2026-06-11
next_plan: 056-03
requirements-completed:
  - 056-G4
summary_artifact_for: .planning/phases/056-HJMT-storage- aggregator/056-02-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 056-02 Summary: Route-Table Contract, Planner Truth, And Cross-Shard Reject

## Completed Scope

`056-02` is complete for the live Phase 056 planner-truth freeze slice.

The runtime-owned `ShardRouteTableV1` contract is now canonical and
fail-closed under one byte-stable encoding, one digest label, one routing
generation contract, and one full-range lookup surface. Route-table truth now
lives in `z00z_aggregators`, not in storage, validators, or simulator code.
The runtime also owns `PlannerMode` vocabulary directly, and rollup-node
config reuses that type instead of defining a second planner-mode authority.

The planner boundary now rejects invalid route tables, stale or wrong routing
generation, and cross-shard batches before semantic execution begins. Central
and per-aggregator planner modes are proven equivalent on accepted workload
profiles and on rejected rows. The checked-in fixture corpus now freezes the
golden route vectors `SRT-G-001` through `SRT-G-004` and the tamper vectors
`SRT-T-001` through `SRT-T-008` as the live byte-authoritative route evidence.

## Files Changed

- `.planning/STATE.md`
- `.planning/ROADMAP.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-02-SUMMARY.md`
- `crates/z00z_rollup_node/src/config.rs`
- `crates/z00z_runtime/aggregators/Cargo.toml`
- `crates/z00z_runtime/aggregators/README.md`
- `crates/z00z_runtime/aggregators/src/batch_planner.rs`
- `crates/z00z_runtime/aggregators/src/lib.rs`
- `crates/z00z_runtime/aggregators/src/types.rs`
- `crates/z00z_runtime/aggregators/tests/common/mod.rs` removed in favor of
  `tests/test_common.rs`
- `crates/z00z_runtime/aggregators/tests/test_common.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_planner.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_shard_routing.rs`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_route_table_v1/README.md`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_route_table_v1/manifest.json`

## Boundary Kept Intact

- Planner truth stayed runtime-owned; no storage-owned, validator-owned, or
  simulator-owned alternate route authority was introduced.
- `PlannerMode` now has one canonical owner under
  `z00z_runtime/aggregators`, so config code no longer carries a parallel
  planner-mode definition.
- Cross-shard work still fails closed at planner admission and never reaches
  semantic storage handoff.
- The route-table fixture corpus is evidentiary only; it freezes the runtime
  contract without creating a second planner or route recomputation layer.
- This slice did not pull subtree lifecycle, proof validation, or storage
  scope-creation truth into the runtime layer; those remain owned by later
  slices.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md`
was used because the slash prompt is not a callable tool in this environment.

- Pass 1 found one significant issue: the new aggregator integration tests
  were using direct `serde_json` instead of the repository codec abstraction.
  The tests were moved to `z00z_utils::codec::{Codec, JsonCodec}`.
- Pass 2 found two remaining contract gaps: the accepted workload label drifted
  from the required literal `mixed present or absent`, and route-codec unit
  coverage still lacked explicit truncated/trailing-byte reject rows. Both
  issues were fixed.
- Pass 3 found one remaining hygiene failure through the broader release path:
  the helper file `tests/common/mod.rs` tripped the rename guard. The helper
  was renamed to `tests/test_common.rs`, and imports were updated.
- Pass 4 reran the exact-string and plan-coverage audit against
  `056-TODO.md`, `056-CONTEXT.md`, and `056-02-PLAN.md`, including route
  fixture ids, planner-mode profiles, wrong-generation rejection, and
  cross-shard admission failure. No significant issues remained.
- Pass 5 repeated the residue scan for the old route-digest label, the stale
  mixed-profile spelling, the removed helper path, and direct `serde_json`
  usage. No significant issues remained.

Two consecutive clean review passes were achieved on passes 4 and 5 after the
Pass 1 through Pass 3 fixes.

## Validation

All Rust validation for this plan was rerun after the final code changes.

- `cargo fmt --all` completed.
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  as the mandatory fail-fast gate.
- `cargo test -p z00z_aggregators --release --features test-params-fast -- --nocapture`
  passed.
- `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_shard_routing -- --nocapture`
  passed.
- `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_planner -- --nocapture`
  passed.
- `cargo test -p z00z_aggregators --release --features test-params-fast --test test_live_guardrails -- --nocapture`
  passed.
- `cargo test -p z00z_rollup_node --release --features test-params-fast -- --nocapture`
  passed.
- `cargo test -p z00z_wallets --release --features test-params-fast --test test_rename_guards -- --nocapture`
  passed.
- `cargo test --release` passed for the workspace.
- `git diff --check` is clean.

## Result

`056-02` is complete. Phase 056 now advances to `056-03-PLAN.md` for the
semantic runtime-to-storage handoff and dynamic scope-birth slice.

This summary does not claim storage-owned subtree lifecycle handoff, first-seen
scope commit behavior, journal lineage continuity, lawful failover, YAML
startup preflight, or simulator closeout evidence; those remain owned by
`056-03` through `056-07`.
