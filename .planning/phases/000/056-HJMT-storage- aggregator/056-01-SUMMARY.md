---
phase: 056-HJMT-storage-aggregator
plan: 056-01
status: complete
completed_at: 2026-06-11
next_plan: 056-02
requirements-completed:
  - 056-G1
  - 056-G2
summary_artifact_for: .planning/phases/056-HJMT-storage- aggregator/056-01-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 056-01 Summary: Runtime Topology, Composition Root, And Config-Home Freeze

## Completed Scope

`056-01` is complete for the live Phase 056 topology-freeze slice.

The repository now has one checked-in `config/hjmt_runtime/sim_5a7s/` runtime
home with explicit `aggregator-config.yaml`, `planner-config.yaml`, and
`storage-config.yaml` files, plus simulator-side `hjmt_runtime` anchoring in
`scenario_config.yaml`. `z00z_rollup_node::NodeConfig::from_hjmt_home(...)`
now loads this topology packet generically from YAML, projects topology into
runtime status, and validates the separate-OS-process contract fail-closed.

The landed guardrails keep `SIM-5A7S` as the canonical acceptance fixture while
still accepting positive non-`5x7` topologies. They also enforce one routing
generation, unique per-aggregator config/data/journal/log/listen paths,
standby coverage for every shard, explicit lifecycle references to
aggregator/planner/storage config paths, and the exact `AggregatorId(0)..(4)`
plus `ShardId(0)..(6)` fixture ranges for `SIM-5A7S`.

The phase packet itself is also synchronized with the new mandatory verify
contract: every `056-0N-PLAN.md` verify block now requires bootstrap first,
the broad release gate, and at least three `/GSD-Review-Tasks-Execution`
passes with two consecutive clean runs before closeout.

## Files Changed

- `.planning/STATE.md`
- `.planning/ROADMAP.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-CONTEXT.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-01-PLAN.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-02-PLAN.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-03-PLAN.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-04-PLAN.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-05-PLAN.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-06-PLAN.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-07-PLAN.md`
- `config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml`
- `config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-0/aggregator-config.yaml`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-1/aggregator-config.yaml`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-2/aggregator-config.yaml`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-3/aggregator-config.yaml`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-4/aggregator-config.yaml`
- `crates/z00z_rollup_node/src/config.rs`
- `crates/z00z_rollup_node/src/runtime.rs`
- `crates/z00z_rollup_node/src/status.rs`
- `crates/z00z_rollup_node/src/lib.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_process.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_node_lifecycle.rs`
- `crates/z00z_simulator/src/config.rs`
- `crates/z00z_simulator/src/lib.rs`
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`

## Boundary Kept Intact

- `SIM-5A7S` is acceptance evidence only; runtime topology remains YAML-driven
  for positive non-`5x7` shapes.
- The accepted multi-aggregator model is separate OS processes only; no
  shared-memory mesh, actor-only shortcut, or in-process orchestration root
  was introduced.
- Planner truth stayed on the runtime side and semantic settlement truth stayed
  storage-owned; this slice did not create a second planner/storage authority
  layer.
- The checked-in runtime home lives under one canonical `config/hjmt_runtime/`
  path and is referenced from the existing simulator surface instead of a new
  side harness.
- Runtime status projection is additive only; it reports topology metadata
  without redefining placement, storage, or publication truth.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md`
was used because the slash prompt is not a callable tool in this environment.

- Pass 1 found four significant issues: direct `std::fs::read_dir` in the
  rollup-node config loader despite the repo I/O abstraction rule; test-side
  raw filesystem writes reintroduced by the new YAML fixture helpers; missing
  fail-closed validation that lifecycle commands reference explicit planner and
  storage config paths; and missing exact `SIM-5A7S`
  `AggregatorId(0)..AggregatorId(4)` / `ShardId(0)..ShardId(6)` enforcement.
  All were fixed and covered by new negative tests.
- Pass 2 found one remaining drift issue: the runtime status unit test still
  labeled a `2`-aggregator / `3`-shard sample as `SIM-5A7S` and used
  non-contract lifecycle command strings. The sample was renamed to
  `SIM-2A3S` and updated to the explicit config-trio lifecycle shape.
- Pass 3 reran the phase-local claim audit against the live files, including
  config-home anchors, topology-status projection, exact fixture range
  enforcement, lifecycle path enforcement, and diff hygiene. No significant
  issues remained.
- Pass 4 repeated the same evidence-only audit after the final test reruns.
  No significant issues remained.

Two consecutive clean review passes were achieved on passes 3 and 4 after the
Pass 1 and Pass 2 fixes.

## Validation

All Rust validation for this plan was rerun after the final code change.

- `cargo fmt --all` completed.
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  as the mandatory fail-fast gate.
- `cargo test -p z00z_rollup_node --release --features test-params-fast -- --nocapture`
  passed: rollup-node unit tests plus the new `test_hjmt_topology.rs`,
  `test_hjmt_process.rs`, and `test_hjmt_node_lifecycle.rs` coverage are
  green.
- `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools -- --nocapture`
  passed for the full simulator crate, including the `scenario_config.yaml` /
  `scenario_design.yaml` stage-surface guards and the long Stage 13 replay
  tail.
- `cargo test --release` passed for the workspace.
- `git diff --check` is clean.

## Result

`056-01` is complete. Phase 056 now advances to `056-02-PLAN.md` for the
route-table contract, planner-truth, and cross-shard reject slice.

This summary does not claim semantic storage handoff, dynamic scope birth,
lawful failover, YAML startup preflight, or final benchmark/evidence closeout;
those remain owned by `056-03` through `056-07`.
