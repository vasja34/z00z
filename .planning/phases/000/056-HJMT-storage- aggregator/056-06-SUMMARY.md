---
phase: 056-HJMT-storage-aggregator
plan: 056-06
status: complete
completed_at: 2026-06-12
next_plan: 056-07
requirements-completed:
  - 056-G10
summary_artifact_for: .planning/phases/056-HJMT-storage- aggregator/056-06-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 056-06 Summary: Simulator Stage Sync And Runtime Evidence

## Completed Scope

`056-06` is complete for the simulator runtime-evidence slice.

`scenario_1` now proves the live Phase 056 runtime plane instead of hiding it
behind a simulator-only side model. The executable
`scenario_config.yaml` surface now carries one `hjmt_runtime` home reference
plus one `runtime_observability` contract with the required correctness
profiles `SIM-SMALL`, `SIM-MEDIUM`, and `SIM-CACHE-EDGE`, while keeping
`SIM-BATCH-1000` explicitly reserved-only. The synchronized
`scenario_design.yaml` surface now states the same runtime-profile and
trace-pack contract in user-facing form.

The runner now loads the checked-in runtime home through the live rollup-node
config seam, derives one canonical config-digest set, one route-table digest,
one journal-lineage digest, and one process-topology digest, and emits the
required trace pack:
`cfg_flow.json`, `tx_flow.json`, `route_flow.json`, `plan_flow.json`,
`journal_flow.json`, `scope_flow.json`, `proc_flow.json`, and
`recovery_flow.json`. These artifacts are explicitly evidentiary only: they
point back to the runtime, planner, and storage owner seams without replacing
runtime or storage semantic truth.

Validation now fails closed for missing traces, stale path anchors, stale
scenario or design references, digest drift, and attempts to execute the
reserved benchmark-only profile. The shared Stage 13 cache root is stabilized
onto its final promoted path before reuse so cached simulator evidence no
longer carries temporary-root drift. The deterministic-profile regression is
also isolated onto per-test ad hoc roots, removing the shared-root race that
previously made the full stage-surface suite flaky under parallel execution.

## Files Changed

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-06-SUMMARY.md`
- `Cargo.lock`
- `crates/z00z_simulator/Cargo.toml`
- `crates/z00z_simulator/src/config.rs`
- `crates/z00z_simulator/src/config_accessors.rs`
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`
- `crates/z00z_simulator/src/scenario_1/runner.rs`
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
- `crates/z00z_simulator/src/test_support/stage13_shared_cases.rs`
- `crates/z00z_simulator/tests/test_hjmt_runtime_config.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- `crates/z00z_simulator/tests/test_scenario_settlement.rs`

## Boundary Kept Intact

- The simulator now observes the live runtime plane through existing runtime
  and storage owner seams only; no second authority layer or parallel semantic
  model was introduced.
- `scope_flow.json` and `recovery_flow.json` stay linked-owner evidence that
  points back to canonical storage and failover owner homes instead of
  duplicating semantic ownership.
- Config digests, route digests, lineage digests, and topology digests remain
  evidence anchors only; planner truth stays runtime-owned and subtree or proof
  truth stays storage-owned.
- The shared cache stabilization fixes path drift at the cache seam, not by
  relaxing the trace validator or widening the simulator sandbox.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md`
was used because the slash prompt is not a callable tool in this environment.

- Pass 1 found a significant issue: the full
  `test_scenario1_stage_surface` suite was flaky because the deterministic
  profile test rewrote the same shared Stage 13 cache root that the live
  surface assertions read concurrently, and the shared cache could also retain
  temporary-root path anchors after promotion. The fix moved deterministic
  profile replays onto isolated ad hoc roots and stabilized the shared cache
  root onto its final promoted path before reuse.
- Pass 2 reran the full `056-06` validation contract against the plan, live
  config and design files, `runtime_observability.rs`, and the simulator
  tests. No significant issues remained.
- Pass 3 reran a narrow regression packet on runtime-config accessors and the
  shared-cache reset contract, then completed a workspace-first doublecheck of
  the material claims for profiles, trace-pack names, reserved-only rejection,
  and path-anchor validation against local repository evidence. No significant
  issues remained.

Two consecutive clean review passes were achieved on passes 2 and 3 after the
final shared-root race fix.

## Validation

Rust validation for this plan completed on the live tree before closeout.

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  as the mandatory fail-fast gate.
- `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario_settlement -- --nocapture`
  passed.
- `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario1_stage_surface -- --nocapture`
  passed.
- `cargo test --release` passed for the workspace.
- `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_hjmt_runtime_config -- --nocapture`
  passed.
- `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools test_support::stage13_shared_cases::tests::reset_stage13_attempt_root_removes_stale_outputs -- --exact --nocapture`
  passed.
- `git diff --check` is clean.

## Result

`056-06` is complete. Phase 056 now advances to `056-07-PLAN.md` for the
fixture, benchmark, validation, and closeout-sync slice.

This summary does not claim final fixture-matrix closeout, benchmark-home
proof, or phase completion; those remain owned by `056-07`.
