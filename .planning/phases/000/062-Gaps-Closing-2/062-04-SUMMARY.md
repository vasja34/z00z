---
phase: 062-Gaps-Closing-2
plan: 062-04
status: complete
completed_at: 2026-06-25
next_plan: 062-05
summary_artifact_for: .planning/phases/062-Gaps-Closing-2/062-04-PLAN.md
---

# 062-04 Summary: Benchmark, Proof-Size, And Measurement Guardrails

## Outcome

`062-04` is complete. The grouped plan contract `PLAN-062-G04` now resolves
through the renamed `062-04-PLAN.md` packet with one explicit measurement
closure mode: Phase 27 is `Closed by Stage13 evidence`.

No standalone measurement sidecar file was introduced. The canonical local
guardrail stays on the existing Stage 13 proof-size, cache, replay, and
tamper artifact packet plus the live `test_scenario1_stage_surface` checks.
This keeps proof-size and benchmark evidence on one existing simulator/storage
axis and avoids inventing a second measurement authority plane.

Benchmark claim discipline is now explicit on the live storage bench path.
Only `durable_root_published_tps` may support user-facing throughput claims.
Worker-local throughput, cache-only throughput, compression-win wording, and
synthetic proof-size rows are now rejected or unsupported in the canonical
bench document and locked by `test_bench_lanes`.

Stage 13 proof-size validation is stronger on the live `scenario_1` packet.
The exact stage-surface test now fails if proof entries lose verified status,
lose non-zero proof bytes or verify time, or if replayed roots drift away from
the live settlement root after reload.

The implementation stayed on existing bench-doc and simulator-report paths.
No new sidecar runtime, no second publication or proof authority plane, and no
parallel storage measurement layer were introduced.

## Files Changed

- `.planning/phases/Z00Z-IMPL-PHASES.md`
- `crates/z00z_storage/benches/settlement_benches.md`
- `crates/z00z_storage/tests/test_bench_lanes.rs`
- `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs`
- `.planning/phases/062-Gaps-Closing-2/062-04-SUMMARY.md`

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_storage --test test_bench_lanes`
- `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_stage_surface::test_scenario1_stage_surface -- --exact`
- `cargo test --release -p z00z_rollup_node --test test_hjmt_node_lifecycle`
- `cargo test --release`
- `rg -n "Closed by Stage13 evidence|durable_root_published_tps|cache-only throughput|synthetic proof-size numbers|publication_latency_us|worker-local throughput may stand in" .planning/phases/Z00Z-IMPL-PHASES.md crates/z00z_storage/benches/settlement_benches.md crates/z00z_storage/tests/test_bench_lanes.rs crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs`
- Result: green

## Manual Review Passes

Because `/GSD-Review-Tasks-Execution` is not callable as a tool here, the
required review loop was executed manually against the same scope.

- Pass 1
  - `git diff -- .planning/phases/Z00Z-IMPL-PHASES.md crates/z00z_storage/benches/settlement_benches.md crates/z00z_storage/tests/test_bench_lanes.rs crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs`
  - `cargo test --release -p z00z_storage --test test_bench_lanes`
  - `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_stage_surface::test_scenario1_stage_surface -- --exact`
  - `cargo test --release -p z00z_rollup_node --test test_hjmt_node_lifecycle`
  - Result: clean; the existing Stage 13 path was sufficient, so no standalone sidecar or new measurement owner was added.
- Pass 2
  - `rg -n "Standalone sidecar required|measure_guard|Closed by Stage13 evidence|proof_size_bytes|verify_time_us|reloaded_settlement_state_root_hex" .planning/phases/Z00Z-IMPL-PHASES.md crates/z00z_storage/benches/settlement_benches.md crates/z00z_storage/tests/test_bench_lanes.rs crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs`
  - `git diff --check .planning/phases/Z00Z-IMPL-PHASES.md crates/z00z_storage/benches/settlement_benches.md crates/z00z_storage/tests/test_bench_lanes.rs crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs`
  - Result: clean
- Pass 3
  - `cargo test --release`
  - `git diff --check`
  - Result: clean

Passes 2 and 3 were consecutive clean runs.

## Completion Notes

Completion:
- Date: 2026-06-25
- Task: TASK-010
- Files changed:
  - `.planning/phases/Z00Z-IMPL-PHASES.md`
  - `crates/z00z_storage/benches/settlement_benches.md`
  - `crates/z00z_storage/tests/test_bench_lanes.rs`
  - `.planning/phases/062-Gaps-Closing-2/062-04-SUMMARY.md`
- Tests run:
  - `cargo test --release -p z00z_storage --test test_bench_lanes` -> passed
  - `cargo test --release` -> passed
- Closeout evidence:
  - `crates/z00z_storage/benches/settlement_benches.md`
  - `crates/z00z_storage/tests/test_bench_lanes.rs`
  - `.planning/phases/Z00Z-IMPL-PHASES.md`

Completion:
- Date: 2026-06-25
- Task: TASK-011
- Files changed:
  - `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs`
  - `.planning/phases/062-Gaps-Closing-2/062-04-SUMMARY.md`
- Tests run:
  - `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_stage_surface::test_scenario1_stage_surface -- --exact` -> passed
  - `cargo test --release` -> passed
- Closeout evidence:
  - `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs`
  - `crates/z00z_simulator/src/scenario_1/runner_verify.rs`

Completion:
- Date: 2026-06-25
- Task: TASK-012
- Files changed:
  - `.planning/phases/Z00Z-IMPL-PHASES.md`
  - `crates/z00z_storage/benches/settlement_benches.md`
  - `crates/z00z_storage/tests/test_bench_lanes.rs`
  - `.planning/phases/062-Gaps-Closing-2/062-04-SUMMARY.md`
- Tests run:
  - `cargo test --release -p z00z_storage --test test_bench_lanes` -> passed
  - `rg -n "Closed by Stage13 evidence" .planning/phases/Z00Z-IMPL-PHASES.md crates/z00z_storage/benches/settlement_benches.md crates/z00z_storage/tests/test_bench_lanes.rs` -> passed
  - `cargo test --release` -> passed
- Closeout evidence:
  - `.planning/phases/Z00Z-IMPL-PHASES.md`
  - `crates/z00z_storage/benches/settlement_benches.md`
  - `crates/z00z_storage/tests/test_bench_lanes.rs`

Completion:
- Date: 2026-06-25
- Task: TASK-013
- Files changed:
  - `.planning/phases/Z00Z-IMPL-PHASES.md`
  - `crates/z00z_storage/benches/settlement_benches.md`
  - `crates/z00z_storage/tests/test_bench_lanes.rs`
  - `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs`
  - `.planning/phases/062-Gaps-Closing-2/062-04-SUMMARY.md`
- Tests run:
  - `cargo test --release -p z00z_storage --test test_bench_lanes` -> passed
  - `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_stage_surface::test_scenario1_stage_surface -- --exact` -> passed
  - `cargo test --release` -> passed
- Closeout evidence:
  - `.planning/phases/Z00Z-IMPL-PHASES.md`
  - `crates/z00z_storage/benches/settlement_benches.md`
  - `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs`

Completion:
- Date: 2026-06-25
- Task: TASK-103
- Files changed:
  - `.planning/phases/Z00Z-IMPL-PHASES.md`
  - `crates/z00z_storage/benches/settlement_benches.md`
  - `crates/z00z_storage/tests/test_bench_lanes.rs`
  - `.planning/phases/062-Gaps-Closing-2/062-04-SUMMARY.md`
- Tests run:
  - `cargo test --release -p z00z_storage --test test_bench_lanes` -> passed
  - `cargo test --release -p z00z_rollup_node --test test_hjmt_node_lifecycle` -> passed
  - `cargo test --release` -> passed
- Closeout evidence:
  - `crates/z00z_storage/benches/settlement_benches.md`
  - `crates/z00z_storage/tests/test_bench_lanes.rs`
  - `.planning/phases/Z00Z-IMPL-PHASES.md`
