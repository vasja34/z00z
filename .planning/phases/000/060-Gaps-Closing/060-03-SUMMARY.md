---
phase: 060-Gaps-Closing
plan: 060-03
status: complete
completed_at: 2026-06-20
next_plan: 060-04
summary_artifact_for: .planning/phases/060-Gaps-Closing/060-03-PLAN.md
---

# 060-03 Summary: HJMT Process Model And YAML Shard Mapping Contract

## Completed Scope

`060-03` is complete for the Phase 060 HJMT topology-contract slice.

The repository now tells one explicit live process-model story across checked-in
runtime config, config parsing, topology tests, and simulator evidence:
`aggregator_owned` remains the production-default mapping, one aggregator still
equals one OS process, and a process may own multiple primary shards under that
default. The checked-in `SIM-5A7S` home now states that contract literally in
YAML and in `manifest.json`, so the operator-facing default is no longer
implicit or guessable.

This slice also adds the opt-in `shard_process` switch without widening the
authority surface. The new branch is local to `AggExecutionCfg` plus
`ShardMapping`, defaults fail closed to `aggregator_owned`, rejects mixed
mapping inside one HJMT home, and rejects any `shard_process` aggregator that
tries to own more than one primary shard. The existing same-lineage
process-scoped journal contract remains enforced for `aggregator_owned`, which
preserves the current lawful failover model instead of silently redefining it.

Finally, the selected mapping is now carried into the simulator observability
packet and the checked-in manifest, so runtime evidence, checked fixtures, and
config parsing all speak the same topology vocabulary. That keeps later `B6`
A/B performance work honest: `shard_process` exists as an experimental switch,
but the repository does not pretend that it is already the live production
contract.

## Files Changed

- `.planning/phases/060-Gaps-Closing/060-03-SUMMARY.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-0/aggregator-config.yaml`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-1/aggregator-config.yaml`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-2/aggregator-config.yaml`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-3/aggregator-config.yaml`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-4/aggregator-config.yaml`
- `config/hjmt_runtime/sim_5a7s/manifest.json`
- `crates/z00z_rollup_node/src/config.rs`
- `crates/z00z_rollup_node/src/lib.rs`
- `crates/z00z_rollup_node/src/runtime.rs`
- `crates/z00z_rollup_node/tests/support/test_hjmt_home.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_node_lifecycle.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_process.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- `crates/z00z_simulator/tests/test_scenario_settlement.rs`

## Boundary Kept

- No second checked-in runtime home, no second topology authority path, and no
  default promotion to `shard_process` were introduced.
- The production-default checked fixture remains `SIM-5A7S` on
  `aggregator_owned`; the new mapping branch is opt-in only.
- The switch is localized to config and evidence seams instead of duplicating
  planner, runtime, or simulator topology logic.
- No parallel requirements layer, alias topology contract, or concept-drift
  wording was introduced.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md` was
used because the slash prompt is not a callable tool in this environment.

- Pass 1 found that the new mapping switch could not stay honest if it lived
  only in parser logic. The checked-in `SIM-5A7S` YAMLs, checked-in
  `manifest.json`, and topology tests all needed the explicit
  `aggregator_owned` marker so the live default was visible on disk instead of
  only inside Rust defaults. Those surfaces were synchronized.
- Pass 2 found that config-level coverage alone was still too narrow. The
  simulator observability packet and settlement-facing tests needed explicit
  `shard_mapping` assertions so runtime evidence could not drift from config and
  manifest truth. Those evidence assertions were added.
- Pass 3 reran the mandatory bootstrap gate, targeted release tests for
  rollup-node and simulator topology surfaces, and the full
  `cargo test --release` broad gate on the final tree. No significant issues
  remained.
- Pass 4 repeated post-gate concept-drift checks on the final tree with
  `git diff --check` and direct source review of the config, manifest, helper,
  and runtime-evidence seams. No significant issues remained.

Two consecutive clean review passes were achieved on passes 3 and 4.

## Validation

- Mandatory bootstrap gate passed:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_rollup_node --release --test test_hjmt_process -- --nocapture`
  passed.
- `cargo test -p z00z_rollup_node --release --test test_hjmt_topology -- --nocapture`
  passed.
- `cargo test -p z00z_rollup_node --release --test test_hjmt_node_lifecycle -- --nocapture`
  passed.
- `cargo test -p z00z_simulator --release --test test_scenario_settlement -- --nocapture`
  passed.
- `cargo test --release` passed on the final tree.
- `rg -n "shard_mapping|AggExecutionCfg|ShardMapping" crates/z00z_rollup_node crates/z00z_simulator config/hjmt_runtime/sim_5a7s`
  confirms one canonical mapping path across config, checked fixtures, tests,
  and runtime evidence.
- `git diff --check` is clean for the files changed in this slice.

## Result

`060-03` is complete. Phase 060 advances to `060-04-PLAN.md` for the rights
owner move, shim demotion, and dual-authority YAML closure slice.
