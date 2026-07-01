---
phase: 063-Core-Update
plan: 063-01
status: complete
completed_at: 2026-06-28
next_plan: 063-02
summary_artifact_for: .planning/phases/063-Core-Update/063-01-PLAN.md
---

# 063-01 Summary: Genesis Execution Contract Restoration

## Outcome

`063-01` is complete. `PLAN-063-G01` now closes `REC-063-P0-01` with one
truthful local execution contract for genesis thread-count handling and one
exact snapshot ZIP contract rooted at `snapshot_export_path`.

`run_genesis(...)` now keeps Rayon scoped to a local thread pool, the snapshot
archive is emitted under `snapshot_export_path` instead of the timestamped
typed-artifact directory, and the live config docs now describe the exact
split between `assets_export_path` and `snapshot_export_path`. Regression
coverage proves both behaviors without introducing process-global
`build_global()` side effects or a parallel artifact authority path.

The implementation stayed on the existing current-tree surfaces only. No
duplicate bootstrap lane, no parallel snapshot path, and no second thread
execution authority were introduced.

## Files Changed

- `crates/z00z_core/src/genesis/genesis_config.rs`
- `crates/z00z_core/src/genesis/genesis_output_support.rs`
- `crates/z00z_core/src/genesis/genesis_run.rs`
- `crates/z00z_core/src/genesis/test_genesis_suite.rs`
- `crates/z00z_storage/tests/test_bench_lanes.rs`
- `crates/z00z_storage/tests/test_live_guardrails.rs`
- `.planning/phases/063-Core-Update/063-01-SUMMARY.md`

## Additional Workspace Unblockers

Broad `cargo test --release` on the current tree exposed stale planning-file
`include_str!(...)` anchors in `z00z_storage` tests. Those tests were updated
to the live Phase `062` and `064` workspace paths, and the `062-04` summary
assertion was narrowed to the actual summary outcome block so the release gate
uses the current canonical planning corpus instead of missing legacy paths.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_core --test test_genesis_manifest_refs -- --nocapture`
- `cargo test --release -p z00z_core --test test_genesis_manifest_goldens -- --nocapture`
- `cargo test --release -p z00z_core --test genesis_tests test_validation -- --nocapture`
- `cargo test --release -p z00z_storage --test test_bench_lanes -- --nocapture`
- `cargo test --release -p z00z_storage --test test_live_guardrails -- --nocapture`
- `cargo test --release`
- `rg -n "num_threads|snapshot_export_path|assets_export_path" crates/z00z_core/src/genesis crates/z00z_core/tests`
- `git diff --check -- crates/z00z_core/src/genesis/genesis_output_support.rs crates/z00z_core/src/genesis/genesis_run.rs crates/z00z_core/src/genesis/genesis_config.rs crates/z00z_core/src/genesis/test_genesis_suite.rs crates/z00z_storage/tests/test_bench_lanes.rs crates/z00z_storage/tests/test_live_guardrails.rs`
- Result: green

## Manual Review Passes

Because `/GSD-Review-Tasks-Execution` is not callable as a tool here, the
required review loop was executed manually against the same scope.

- Pass 1
  - `git diff -- crates/z00z_core/src/genesis/genesis_output_support.rs crates/z00z_core/src/genesis/genesis_run.rs crates/z00z_core/src/genesis/genesis_config.rs crates/z00z_core/src/genesis/test_genesis_suite.rs`
  - `rg -n "ThreadPoolBuilder::new|build_global|snapshot_export_path|assets_export_path" crates/z00z_core/src/genesis crates/z00z_core/src/genesis/test_genesis_suite.rs`
  - Result: found and then fixed the remaining snapshot ZIP root drift plus the timestamp expectation mismatch in the new regression test
- Pass 2
  - `git diff -- crates/z00z_core/src/genesis/genesis_output_support.rs crates/z00z_core/src/genesis/genesis_run.rs crates/z00z_core/src/genesis/genesis_config.rs crates/z00z_core/src/genesis/test_genesis_suite.rs crates/z00z_storage/tests/test_bench_lanes.rs crates/z00z_storage/tests/test_live_guardrails.rs`
  - `cargo test --release -p z00z_core --test test_genesis_manifest_refs -- --nocapture`
  - `cargo test --release -p z00z_core --test test_genesis_manifest_goldens -- --nocapture`
  - `cargo test --release -p z00z_core --test genesis_tests test_validation -- --nocapture`
  - `git diff --check -- crates/z00z_core/src/genesis/genesis_output_support.rs crates/z00z_core/src/genesis/genesis_run.rs crates/z00z_core/src/genesis/genesis_config.rs crates/z00z_core/src/genesis/test_genesis_suite.rs crates/z00z_storage/tests/test_bench_lanes.rs crates/z00z_storage/tests/test_live_guardrails.rs`
  - Result: clean
- Pass 3
  - `cargo test --release -p z00z_storage --test test_bench_lanes -- --nocapture`
  - `cargo test --release -p z00z_storage --test test_live_guardrails -- --nocapture`
  - `cargo test --release`
  - `git diff --check -- crates/z00z_core/src/genesis/genesis_output_support.rs crates/z00z_core/src/genesis/genesis_run.rs crates/z00z_core/src/genesis/genesis_config.rs crates/z00z_core/src/genesis/test_genesis_suite.rs crates/z00z_storage/tests/test_bench_lanes.rs crates/z00z_storage/tests/test_live_guardrails.rs`
  - Result: clean

Passes 2 and 3 were consecutive clean runs.

## Completion Notes

- `063-01-SUMMARY.md` closes `PLAN-063-G01` and advances the active execution
  lane to `063-02-PLAN.md`.
- `performance.num_threads` remains live because the code now enforces it
  through a local pool instead of a process-global Rayon mutation.
- `assets_export_path` is now the base directory for timestamped typed
  artifact outputs, while `snapshot_export_path` is the only live directory
  for snapshot ZIP archives.
