---
phase: 058-HJMT-benchmarks
plan: 058-05
status: complete
completed_at: 2026-06-15
next_plan: 058-06
requirements-completed:
  - 058-G7
  - 058-G8
  - 058-G12
summary_artifact_for: .planning/phases/058-HJMT-benchmarks/058-05-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 058-05 Summary: Benchmark Matrix, `SIM-BATCH-1000`, And Honest Score Discipline

## Completed Scope

`058-05` is complete for the benchmark matrix, heavy-workload profile, and
score-discipline slice.

This slice did not create a second storage benchmark harness, a second
simulator profile authority, or a second archive-home truth path. Instead it
closed the live measurement contract inside the existing owners:
`settlement_hjmt.rs`, `settlement_proofs.rs`, `settlement_shard.rs`,
`settlement_benches.md`, `run_storage_settlement_bench.py`, and the
release-mode simulator observability contract.

`SIM-BATCH-1000` is now a live heavy-only benchmark and readiness profile
instead of a reserved-only placeholder. The runtime config keeps
`SIM-SMALL`, `SIM-MEDIUM`, and `SIM-CACHE-EDGE` as the deterministic or
bounded correctness profiles while exposing `SIM-BATCH-1000` explicitly
through `supported_profiles`, `heavy_only_profiles`, the stage-surface tests,
and the release trace packet.

The storage benchmark packet now closes the `1/2/4/8/16` shard-scaling
matrix, preserves `sim_5a7s` as the canonical comparison lane, and records the
required shard-scaling columns `hot_shard_ratio`, `global_cadence_us`,
`shard_tps`, `global_tps`, `worker_local_tps`,
`durable_root_published_tps`, and `blocked_time_us`. The proof comparison
matrix now carries the explicit `128/1000/1024` rows, and the live bench
ledger classifies baseline-lane, score, compression, and archive-home claims
as `accepted`, `rejected`, or `unsupported` instead of implying closure.

The measured-report path also now preserves stage-owned timing slices without
introducing a new harness. `run_storage_settlement_bench.py` writes sibling
`*.timing.tsv` traces, archives them on the same report path, and inlines an
`Internal Stage Timing Slices` section so mutation-facing HJMT reports keep
separate `hjmt_plan_ops`, `hjmt_child_commit`, `hjmt_parent_commit`, and
`hjmt_journal_sync` evidence instead of collapsing them into one commit median.

## Files Changed

- `.planning/phases/058-HJMT-benchmarks/058-05-SUMMARY.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `crates/z00z_simulator/src/config.rs`
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
- `crates/z00z_simulator/tests/test_hjmt_runtime_config.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- `crates/z00z_simulator/tests/test_scenario_settlement.rs`
- `crates/z00z_storage/benches/settlement_benches.md`
- `crates/z00z_storage/benches/settlement_hjmt.rs`
- `crates/z00z_storage/benches/settlement_proofs.rs`
- `crates/z00z_storage/benches/settlement_shard.rs`
- `crates/z00z_storage/scripts/run_storage_settlement_bench.py`
- `crates/z00z_storage/tests/test_bench_lanes.rs`

## Boundary Kept Intact

- Phase 058 still measures and classifies the inherited Phase 056 runtime and
  Phase 057 publication system in place; it did not invent new routing,
  publication, or proof semantics.
- `SIM-BATCH-1000` is live scope, but it remains heavy-only readiness evidence
  and does not replace the smaller deterministic correctness profiles.
- `crates/z00z_storage/outputs/settlement/` remains the only live benchmark
  archive home. No `outputs/assets` live truth path was introduced.
- The timing-slice fix stays inside the existing helper and report path. No
  parallel benchmark tool, sidecar crate, or detached evidence store was added.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md`
was used because the slash prompt is not a callable tool in this environment.

- Pass 1 found one material closure gap: the live bench reports still lacked
  explicit stage-owned raw timing slices for `hjmt_plan_ops`,
  `hjmt_child_commit`, `hjmt_parent_commit`, and `hjmt_journal_sync`, which
  meant the score packet could still collapse commit timing too coarsely.
  The fix was landed on the existing `run_storage_settlement_bench.py`
  report path plus the bench doc and guard tests.
- Pass 2 re-audited `058-TODO.md`, `058-05-PLAN.md`, the simulator runtime
  contract, the storage bench doc, the runner, and the updated test anchors.
  No significant issues remained.
- Pass 3 repeated the same audit after the final validation packet on the
  finished code tree. No significant issues remained.

Two consecutive clean review passes were achieved on passes 2 and 3.

## Validation

All validation for this slice is green on the final code tree.

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  as the mandatory fail-fast gate on the final tree.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_bench_lanes -- --nocapture`
  passed on the final tree.
- The earlier slice-targeted release suites remained green:
  `test_hjmt_batch_proof`, `test_hjmt_adaptive_policy_proofs`,
  `test_hjmt_runtime_config`, `test_scenario_settlement`, and the
  `test_scenario1_stage_surface` heavy-only or cache-edge selectors.
- The earlier compile-only bench gates remained green:
  `cargo bench -p z00z_storage --bench settlement_proofs --no-run`,
  `settlement_hjmt --no-run`, `settlement_shard --no-run`,
  `settlement_nested --no-run`, and `adaptive_policy_bench --no-run`.
- The measured helper runs stayed green:
  `settlement_shard` validated the live `initial_shard_scaling/shards_{1,2,4,8,16}`
  matrix, `settlement_hjmt` validated the live
  `root_of_roots_publish/shards_{1,2,4,8,16}` matrix, and the final
  `settlement_hjmt_insert_single` helper run emitted both
  `settlement_hjmt_insert_single.md` and the sibling
  `settlement_hjmt_insert_single.timing.tsv` with the new
  `Internal Stage Timing Slices` section.
- Full `settlement_proofs` helper execution for the new heavy `1000` path was
  attempted earlier in the slice, but the runner remained computationally
  heavy enough that those full helper probes were stopped instead of being
  misreported as complete. The canonical proof-byte closure remains backed by
  the green targeted tests, compile gates, and the documented
  `128/1000/1024` lane contract.
- `cargo doc --no-deps` passed. It emitted only pre-existing rustdoc warnings
  in untouched crates and docs, including unresolved intra-doc links or
  invalid markup in `z00z_crypto`, `z00z_core`, `z00z_wallets`, and
  `z00z_simulator`.
- `cargo test --release` passed for the workspace on the final code tree.
- `git diff --check` is clean.

## Result

`058-05` is complete. Phase 058 advances to `058-06-PLAN.md` for dynamic
scope birth, wallet proof-before-ownership closure, historical playback, and
occupancy-privacy readiness.

This summary does not claim final dynamic-scope closure, final historical or
occupancy closure, or the final phase verdict; those remain owned by `058-06`
and `058-07`.
