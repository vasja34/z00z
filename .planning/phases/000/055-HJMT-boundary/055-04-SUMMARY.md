---
phase: 055-HJMT-boundary
plan: 055-04
status: complete
completed_at: 2026-06-10
next_plan: none
requirements-completed: [PH55-04]
---

<!-- markdownlint-disable MD060 -->

# 055-04 Summary: Benchmarks, Scenario Evidence, And Closeout Guardrails

## Completed Scope

`055-04` is complete for the live Phase 055 benchmark-and-evidence slice.

The storage bench harness now owns the additive batch-proof evidence surface in
place. The logical lanes `hjmt_batch_proof_bytes` and `hjmt_batch_verify` live
inside the canonical `settlement_proofs.rs` home, remain compared against one
`ProofBlob` and the current `Vec<ProofBlob>` baseline, and do not introduce a
standalone proof bench or a second authority path. The batch-only helper path
now distinguishes the full `settlement_proofs` report from the canonical
`settlement_proofs_batch` evidence run, so the batch report no longer
recalculates the unrelated full proof-note matrix before it reaches the live
Criterion lanes. Live batch evidence is intentionally scoped to the lightweight
counts `{2,8,32}`; the heavier `128/1024` rows remain part of the full bench
and stress lanes only.

The scattered non-existence compare matrix is now truthful to the current live
V1 batch surface. Instead of pretending that every leaf-backed absence proof
can lower into the current batch witness envelope, the compare fixture selects
only live batch-compatible scattered gaps during setup and keeps the measured
runtime path unchanged.

Stage 13 remains the only simulator evidence authority and is extended in
place. The report pack now carries explicit comparison rows and proof-size rows
for `proof_blob_single`, `proof_blob_vec`, and `batch_proof_v1`, with live
`path_count`, `path_shape`, `canonical_order`, `atomic_verdict`,
`shard_context_mode`, and root-binding fields. The tamper pack now includes the
required batch-specific reject cases, and `runner_verify.rs` fails closed if
comparison coverage, batch shapes, batch counts, proof families, or
proof-size/example row parity drift.

The long simulator tail was also reduced at the correct architectural seam.
One canonical shared Stage 13 fixture is now built through the live
`scenario_1` runner and reused by the Stage 13 consumer tests via the precise
shared-fixture cache. The first live Stage 11/13 acceptance pass remains the
real runtime cost; repeated consumer tests no longer rebuild the full scenario
surface unnecessarily.

## Files Changed

- `.planning/phases/055-HJMT-boundary/055-04-SUMMARY.md`
- `.planning/phases/055-HJMT-boundary/055-SUMMARY.md`
- `.planning/phases/055-HJMT-boundary/055-CONTEXT.md`
- `.planning/phases/055-HJMT-boundary/055-SOURCE-AUDIT.md`
- `.planning/phases/055-HJMT-boundary/055-TEST-SPEC.md`
- `.planning/phases/055-HJMT-boundary/055-TESTS-TASKS.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `crates/z00z_storage/benches/settlement_hjmt.rs`
- `crates/z00z_storage/benches/settlement_proofs.rs`
- `crates/z00z_storage/benches/settlement_benches.md`
- `crates/z00z_storage/scripts/run_storage_settlement_bench.py`
- `crates/z00z_storage/src/fixture_support/settlement_bench_output.rs`
- `crates/z00z_storage/src/settlement/README.md`
- `crates/z00z_storage/src/settlement/hjmt_batch_proof.rs`
- `crates/z00z_storage/tests/test_bench_lanes.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_utils/report.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_utils/hjmt_examples.rs`
- `crates/z00z_simulator/src/scenario_1/runner_verify.rs`
- `crates/z00z_simulator/src/test_support/fixture_cache.rs`
- `crates/z00z_simulator/src/test_support/stage13_shared_cases.rs`
- `crates/z00z_simulator/src/test_support/stage4_shared_cases.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/test_tx_lane_runtime_suite.rs`
- `crates/z00z_simulator/tests/test_pipeline_genesis_tx.rs`
- `crates/z00z_simulator/tests/test_scenario_settlement.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`

## Boundary Kept Intact

- No standalone batch-proof bench crate, second bench helper, or parallel note
  authority was introduced.
- No second simulator lane or `Stage13B`-style artifact family was introduced;
  Stage 13 stayed the only scenario evidence authority.
- `ProofBlob` remained unchanged, `Vec<ProofBlob>` remained the independent
  baseline, and `BatchProofBlobV1` remained additive only.
- Measured timings and sizes stayed evidence only; they did not become protocol
  constants or proof-visible semantics.
- The live batch evidence path intentionally uses only the representative
  `{2,8,32}` counts, while heavier batch sizes remain isolated to the full
  benchmark/stress surface.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md`
was used because the slash prompt is not a callable tool in this environment.

- Pass 1 found one Phase 055 naming issue in the new batch-root unit test. The
  test name was shortened to the repository identifier-length boundary.
- Pass 2 re-audited the benchmark plumbing and the filtered batch evidence path
  against real measured scope. No significant performance or measurement-truth
  issues remained after the note-scope split.
- Pass 3 found one remaining docs-truth issue: the bench docs still described
  the scattered non-existence compare rows too loosely. The docs were updated
  to state explicitly that the fixture selects only live batch-compatible
  scattered gaps.
- Subsequent reruns of the same code/docs-truth audit stayed clean twice in a
  row after the final fixes.

## Validation

All Rust validation for this plan was rerun or rechecked after the final
Phase 055 changes.

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  as the mandatory fail-fast gate.
- `cargo test --release` passed for the full workspace.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_bench_lanes -- --nocapture`
  passed: 16 passed.
- `cargo test -p z00z_storage --release --features test-params-fast hjmt_batch_proof::tests::test_scattered_nonexistence_batch_root -- --nocapture`
  passed.
- `cargo bench -p z00z_storage --bench settlement_proofs --no-run` passed.
- `cargo bench -p z00z_storage --bench settlement_hjmt --no-run` passed.
- `./crates/z00z_storage/scripts/run_storage_settlement_bench.py --bench settlement_proofs --log-base settlement_proofs_batch -- hjmt_batch_ --quick --noplot --warm-up-time 0.01 --measurement-time 0.02`
  passed with the canonical batch-only evidence path. The completed run
  finished in `0:39.31` wall time with `163372 kB` maximum RSS.
- `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_fixture_cache_contract -- --nocapture`
  passed.
- `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario1_stage_surface -- --nocapture`
  passed.
- `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario_settlement -- --nocapture`
  passed.
- `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_pipeline_genesis_tx -- --nocapture`
  passed.
- `cargo run --release -p z00z_simulator --bin scenario_1 --features test-params-fast --features wallet_debug_tools`
  passed. The live acceptance run reported `stage11 elapsed_ms=20909`,
  `stage13 elapsed_ms=65861`, `scenario.profile_total stage_elapsed_ms=101952`,
  `2:17.96` wall time, and `2006032 kB` maximum RSS.

## Closeout

- `055-04` is complete.
- Phase 055 is now ready to close on `055-SUMMARY.md`.
- No active `055` execution lane remains after this summary.
