---
phase: 064-Gaps-Closing-3
plan: 064-01
status: complete
completed_at: 2026-06-30
next_plan: 064-02
summary_artifact_for: .planning/phases/064-Gaps-Closing-3/064-01-PLAN.md
---

# 064-01 Summary: Simulator Final Truth And Packet Integrity

## Outcome

`064-01` is complete. `PLAN-064-G01` now closes `REC-064-P0-01`,
`REC-064-P0-02`, `REC-064-P0-03`, `REC-064-P0-08`, and `REC-064-P2-07`
through one truthful simulator-owned path.

Default `scenario_1` publication now seals final checkpoint evidence unless
the configuration explicitly requests `DraftOnly`. Shared draft fixtures keep
their earlier draft behavior through an explicit draft-only helper, so the
default release path no longer leaves final checkpoint fields `null` or
reports incomplete publication rows.

Canonical stage coverage also now fails closed instead of self-healing through
synthetic `step_stub` events. The stage-owned fallback writers error on
uncovered steps, the filtered-run regression proves the canonical runner path
stays hermetic, and the slice does not introduce any alias, shim, or second
closure lane.

`runtime_observability` now emits real `asset_flow.json`,
`voucher_flow.json`, and `right_flow.json` files, validates them as emitted
public packet anchors, and aligns the live config plus docs to one canonical
emitted path. The default release packet remains secret-clean, and the
simulator harness keeps importing through stable public facades.

## Files Changed

- `crates/z00z_simulator/src/config.rs`
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`
- `crates/z00z_simulator/src/scenario_1/stage_9/bundle_lane_impl.rs`
- `crates/z00z_simulator/src/scenario_1/stage_12/mod.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13/shared_cases.rs`
- `crates/z00z_simulator/src/scenario_1/support/checkpoint_shared_cases.rs`
- `crates/z00z_simulator/tests/scenario_1/main.rs`
- `crates/z00z_simulator/tests/scenario_1/test_hjmt_runtime_config.rs`
- `crates/z00z_simulator/tests/scenario_1/test_scenario1_filtered_runs.rs`
- `crates/z00z_simulator/tests/scenario_1/test_scenario1_object_flows.rs`
- `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs`
- `crates/z00z_simulator/tests/scenario_1/test_scenario_settlement.rs`
- `crates/z00z_simulator/README.md`
- `wiki/06-simulator-and-quality/scenario1-object-artifacts.md`
- `.planning/phases/064-Gaps-Closing-3/064-01-PLAN.md`
- `.planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md`
- `.planning/phases/064-Gaps-Closing-3/064-TESTS-TASKS.md`
- `.planning/phases/064-Gaps-Closing-3/064-01-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_simulator --test scenario_1 test_hjmt_runtime_config -- --nocapture`
- `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_filtered_runs -- --nocapture`
- `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_stage_surface -- --nocapture`
- `cargo test --release -p z00z_simulator --test scenario_1 test_scenario_settlement -- --nocapture`
- `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_object_flows -- --nocapture`
- `cargo test --release -p z00z_simulator --test scenario_1 test_stage2_secret_artifacts -- --nocapture`
- `cargo test --release`
- `rg -n "pending_public_files|pending_exact_home|step_stub" crates/z00z_simulator/src crates/z00z_simulator/tests wiki/06-simulator-and-quality crates/z00z_simulator/README.md .planning/phases/064-Gaps-Closing-3/064-01-PLAN.md .planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md .planning/phases/064-Gaps-Closing-3/064-TESTS-TASKS.md`
- `git diff --check -- crates/z00z_simulator/src/config.rs crates/z00z_simulator/src/scenario_1/runtime_observability.rs crates/z00z_simulator/src/scenario_1/scenario_config.yaml crates/z00z_simulator/src/scenario_1/stage_9/bundle_lane_impl.rs crates/z00z_simulator/src/scenario_1/stage_12/mod.rs crates/z00z_simulator/src/scenario_1/stage_13/shared_cases.rs crates/z00z_simulator/src/scenario_1/support/checkpoint_shared_cases.rs crates/z00z_simulator/tests/scenario_1/main.rs crates/z00z_simulator/tests/scenario_1/test_hjmt_runtime_config.rs crates/z00z_simulator/tests/scenario_1/test_scenario1_filtered_runs.rs crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs crates/z00z_simulator/tests/scenario_1/test_scenario_settlement.rs crates/z00z_simulator/tests/scenario_1/test_scenario1_object_flows.rs crates/z00z_simulator/README.md wiki/06-simulator-and-quality/scenario1-object-artifacts.md .planning/phases/064-Gaps-Closing-3/064-01-PLAN.md .planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md .planning/phases/064-Gaps-Closing-3/064-TESTS-TASKS.md .planning/STATE.md .planning/ROADMAP.md`

- Result:
  - The mandatory bootstrap gate passed.
  - All targeted `z00z_simulator` release-mode acceptance tests for
    `PLAN-064-G01` passed.
  - The broad workspace `cargo test --release` rerun reproduced current-tree
    `z00z_core` blockers outside the modified simulator-owned slice:
    `genesis::genesis_manifest::test_genesis_plan_rights_only_requires_policy_resolution_when_needed`
    failed with
    `ConfigParseFailed("wallet profile validator_mandate_lock_v1 references unknown locked_asset_id z00z")`,
    and `genesis::genesis_rights::test_genesis_rights_deterministic` reported
    the current rights snapshot drift rooted in
    `crates/z00z_core/configs/devnet_genesis_config.yaml`.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times
against this slice:

- Attempt 1
  - `timeout 90s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/064-Gaps-Closing-3/064-01-PLAN.md current_task="Simulator final truth and packet integrity"'`
  - Result: failed with `402 Prompt tokens limit exceeded: 84578 > 45156`
- Attempt 2
  - `timeout 90s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/064-Gaps-Closing-3/064-01-PLAN.md current_task="Simulator final truth and packet integrity"'`
  - Result: failed with `402 Prompt tokens limit exceeded: 66818 > 45156`
- Attempt 3
  - `timeout 90s gsd --no-session -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/064-Gaps-Closing-3/064-01-PLAN.md current_task="Simulator final truth and packet integrity"'`
  - Result: failed with `402 Prompt tokens limit exceeded: 75149 > 45156`

Equivalent review passes were executed manually against the same scope.

- Pass 1
  - `git diff -- crates/z00z_simulator/src/config.rs crates/z00z_simulator/src/scenario_1/runtime_observability.rs crates/z00z_simulator/src/scenario_1/scenario_config.yaml crates/z00z_simulator/src/scenario_1/stage_9/bundle_lane_impl.rs crates/z00z_simulator/src/scenario_1/stage_12/mod.rs crates/z00z_simulator/src/scenario_1/stage_13/shared_cases.rs crates/z00z_simulator/src/scenario_1/support/checkpoint_shared_cases.rs crates/z00z_simulator/tests/scenario_1/main.rs crates/z00z_simulator/tests/scenario_1/test_hjmt_runtime_config.rs crates/z00z_simulator/tests/scenario_1/test_scenario1_filtered_runs.rs crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs crates/z00z_simulator/tests/scenario_1/test_scenario_settlement.rs crates/z00z_simulator/tests/scenario_1/test_scenario1_object_flows.rs crates/z00z_simulator/README.md wiki/06-simulator-and-quality/scenario1-object-artifacts.md`
  - `rg -n "pending_public_files|pending_exact_home|step_stub" crates/z00z_simulator/src crates/z00z_simulator/tests wiki/06-simulator-and-quality crates/z00z_simulator/README.md`
  - Result: the retired `pending_public_files` lane is gone from the live
    simulator slice, the remaining `pending_exact_home` and `step_stub`
    strings are limited to plan/test assertions, and no second packet-authority
    path was introduced
- Pass 2
  - `cargo test --release -p z00z_simulator --test scenario_1 test_hjmt_runtime_config -- --nocapture`
  - `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_filtered_runs -- --nocapture`
  - `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_object_flows -- --nocapture`
  - `git diff --check -- crates/z00z_simulator/src/config.rs crates/z00z_simulator/src/scenario_1/runtime_observability.rs crates/z00z_simulator/src/scenario_1/scenario_config.yaml crates/z00z_simulator/src/scenario_1/stage_9/bundle_lane_impl.rs crates/z00z_simulator/src/scenario_1/stage_12/mod.rs crates/z00z_simulator/src/scenario_1/stage_13/shared_cases.rs crates/z00z_simulator/src/scenario_1/support/checkpoint_shared_cases.rs crates/z00z_simulator/tests/scenario_1/main.rs crates/z00z_simulator/tests/scenario_1/test_hjmt_runtime_config.rs crates/z00z_simulator/tests/scenario_1/test_scenario1_filtered_runs.rs crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs crates/z00z_simulator/tests/scenario_1/test_scenario_settlement.rs crates/z00z_simulator/tests/scenario_1/test_scenario1_object_flows.rs crates/z00z_simulator/README.md wiki/06-simulator-and-quality/scenario1-object-artifacts.md`
  - Result: clean
- Pass 3
  - `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_stage_surface -- --nocapture`
  - `cargo test --release -p z00z_simulator --test scenario_1 test_scenario_settlement -- --nocapture`
  - `cargo test --release -p z00z_simulator --test scenario_1 test_stage2_secret_artifacts -- --nocapture`
  - `cargo test --release`
  - Result: no significant issues remained in the modified `064-01` slice;
    only the current-tree `z00z_core` genesis/config blockers outside the
    changed simulator files were reproduced

Passes 2 and 3 were consecutive clean manual review passes for the modified
scope.

## Completion Notes

- `064-01-SUMMARY.md` closes `PLAN-064-G01` and advances the active execution
  lane to `064-02-PLAN.md`.
- The live default simulator publication path is now final by default, while
  draft-only fixtures remain explicit.
- Public object-flow packet anchors are now emitted on one canonical path
  instead of a pending placeholder lane.
- The current broad workspace blocker remains in the in-progress
  `z00z_core` genesis/config surface, not in the `064-01` simulator-owned
  slice.
