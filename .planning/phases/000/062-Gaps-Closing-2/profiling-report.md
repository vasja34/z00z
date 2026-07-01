# Profiling Report

Date: 2026-06-25

Scope: `/z00z-full-verify-gate max-safe` with the goal of finding the small set of paths that dominate wall-clock time and should be optimized first.

Method: followed `.planning/phases/profiling-comprehensive.md` and combined baseline gate runs, targeted single-test runs, cargo timings, `strace`, `valgrind --tool=callgrind`, and `/usr/bin/time`.

## 🧪 Tooling

- Available: `time`, `perf`, `strace`, `valgrind`, `flamegraph`, `cargo-flamegraph`
- Missing: `hyperfine`, `heaptrack`
- Blocked by host policy: `perf`, `flamegraph`, and `cargo-flamegraph` could not collect samples because `perf_event_paranoid=4`
- Tool inventory artifact: `reports/profiler-tools-summary.json`

## 📊 Measured Results

| Area | Evidence | Wall time | Key note |
| --- | --- | ---: | --- |
| Full gate baseline | `reports/profiling-baseline-full_verify.log` | 2:18.75 | Not canonical: external cargo lock contention, then terminated before tests |
| Workspace build | `reports/profile-stage-build.log` | 1:43.59 | Main compile wave |
| Max-safe prebuild | `reports/profile-stage-maxsafe-prebuild.time` | 1:40.67 | Second compile wave via `cargo test --no-run` |
| Runnable targets | `reports/profile-stage-runnable-targets.log` | 0:05.07 | Not a top hotspot on warm cache |
| `test_hjmt_e2e::test_e2e_acceptance_homes_live` | `reports/profile-test-hjmt-e2e-acceptance.log` | 1:30.62 | Two Stage13 attempts, then runtime trace drift failure |
| `test_scenario_settlement::test_cover_mixed_fixture_scope` | `reports/profile-test-scenario-settlement.log` | 1:09.49 | Stage13-heavy, same drift failure |
| `test_scenario1_stage_surface::test_scenario1_stage_surface` | `reports/profile-test-scenario1-stage-surface.log` | 1:09.33 | Stage13-heavy, same drift failure |
| `test_stage4_digest_replay_contract_heavy` | `reports/profile-test-stage4-digest-heavy.log` | 0:25.54 | Real compute-heavy path, test passed |

Important stage-level timings inside the slow `scenario_1` runs:

- `claim_prepare` stage 3: `5.85s` to `7.14s`
- `tx_prepare` stage 6: `2.01s` to `2.15s`
- `checkpoint_apply_storage` stage 11: `4.29s` to `5.37s`
- `hjmt_settlement_examples` stage 13: `32.82s` to `33.01s` on successful attempts
- Total scenario body after a clean successful pass: about `46.2s`

## 🎯 Top 5% Hotspots

The practical top 5% set is the five paths below. They dominate both single-test latency and end-to-end gate runtime.

| Rank | Hotspot | Evidence | Why it dominates | Recommendation |
| --- | --- | --- | --- | --- |
| 1 | `crates/z00z_simulator/src/scenario_1/stage_13/shared_cases.rs` plus `runtime_observability.rs` validation | Three tests at `69s` to `91s`; Stage13 alone ~`33s`; first acceptance run retried after a failed Stage13 attempt | Shared Stage13 roots are rebuilt, copied, localized, then rejected by exact runtime-trace drift checks on dynamic fields like `tx_id` and `tx_history_digest_hex` | Stop exact full-payload equality on dynamic fields, build one canonical immutable Stage13 pack, and reuse it without full tree copies |
| 2 | `z00z_simulator` compile units | Cargo timing HTML: `47.54s` (`z00z_simulator`), `38.51s` (`z00z_simulator "lib" (test)`) | Simulator is one of the largest release and test compile units in the workspace | Split oversized test/support code behind narrower features, reduce all-features use for the gate, and trim simulator test dependency surface |
| 3 | `z00z_wallets` compile units | Cargo timing HTML: `43.15s` (`z00z_wallets "lib" (test)`), `26.31s` (`z00z_wallets`), plus several `9s` to `16.5s` test artifacts | Wallet test artifacts are the largest volume cluster in the workspace and the largest share of max-safe task fan-out | Remove unnecessary `--all-features` coverage from default gate paths, consolidate test helpers, and split expensive optional features from common test binaries |
| 4 | `z00z_storage` bench/test compile units | Cargo timing HTML: `25.44s` (`settlement_proofs` bench), `23.57s` (`z00z_storage "lib" (test)`), `20.75s` (`settlement_hjmt` bench), `12.02s` (`settlement_shard` bench) | Bench and test artifacts inflate compile time even before runtime work begins | Move heavyweight benches out of the default max-safe compile wave or gate them separately from routine verification |
| 5 | `test_stage4_digest_replay_contract_heavy` runtime path and crypto/proof kernels | Passed in `25.54s`; callgrind points to `curve25519_dalek` AVX2 field multiply as the top instruction sink | This is a genuinely compute-bound replay path, not just IO or lock overhead | Add a focused perf bench for the replay pipeline and optimize the hot crypto/hash/serialization path before trying broader harness changes |

## 🔬 Supporting Evidence

Compile-wave shape:

- `cargo build --workspace --release --all-targets --all-features --timings`: `1:43.59`, max RSS `3.41 GB`
- `cargo test --workspace --release --all-targets --all-features --no-run`: `1:40.67`, max RSS `3.41 GB`
- These two stages are nearly the same cost, which means the gate currently pays for two large compile waves

Max-safe sweep shape:

- Total tasks: `319`
- Test tasks: `292`
- Bench tasks: `21`
- Exec tasks: `6`
- Largest package counts: `z00z_wallets 149`, `z00z_storage 71`, `z00z_crypto 29`, `z00z_core 25`

System-call profile for `test_scenario_settlement::test_cover_mixed_fixture_scope` from `reports/profile-strace-scenario-settlement.txt`:

- `futex`: `75.48%` of syscall time
- `sched_yield`: `21.78%`
- `copy_file_range`: `220` calls
- `renameat`: `1410` calls
- `fdatasync`: `9643` calls
- `fsync`: `1618` calls
- `mkdir`: `3073` calls
- `flock`: `2308` calls

Interpretation:

- The slow Stage13-heavy tests are dominated by coordination and waiting, not by raw sequential disk throughput
- There is still substantial artifact churn from copying, renaming, locking, and syncing output trees

Callgrind evidence for `test_stage4_digest_replay_contract_heavy`:

- Top reported sink: `curve25519_dalek` AVX2 field multiplication at `37.78%` of retired instructions
- The callgrind run hit a different scenario failure mode under instrumentation, so use it as directional CPU evidence rather than a canonical pass/fail replay

## 🧱 Gate-Level Amplifiers

These are not the hottest single code functions, but they materially stretch total gate time and should be fixed after the top runtime path above.

1. Duplicate compile wave in the gate
- `full_verify.sh` runs a full workspace build, then max-safe prebuild effectively rebuilds the same graph with `cargo test --no-run`
- Removing one of these waves is worth roughly `100s` on a warm measured run

2. Forced serialization of `scenario_1`
- `crates/z00z_simulator/tests/scenario_1/main.rs` sets `RUST_TEST_THREADS=1`
- `crates/z00z_simulator/src/scenario_1/mod.rs` keeps a process-global scenario lock
- `fixture_cache.rs` uses `100ms` sleep-poll loops around shared fixture coordination

3. Artifact copying and localization churn in Stage13
- `shared_cases.rs` promotes shared roots, validates them, then copies trees into localized roots
- On failures it may retry up to `FULL_STAGE13_BUILD_RETRIES = 5`

4. Debug artifact generation on the hot path
- Stage 3 writes `export_wallet_debug_{alice,bob,charlie}.json` on each attempt
- That adds file churn inside already expensive scenario attempts

5. Runnable target direct-exec bypass is disabled for explicit-feature entries
- `run_runnable_targets.py` only direct-execs when there are no explicit features
- `z00z_wallet_egui` therefore runs via `cargo run --release --features test-params-fast,egui ...`
- This is a low-priority issue today, but it is unnecessary gate overhead

## 🚀 Recommended Optimization Order

1. Fix Stage13 cache validation semantics first
- Normalize or ignore dynamic runtime-trace fields before comparison
- Compare invariant structure, not the entire localized JSON payload

2. Remove Stage13 full-tree copy churn
- Replace localized root copies with an immutable shared artifact plus a writable overlay, hardlinks, or reflinks
- Avoid rebuilding the shared Stage13 pack after non-semantic drift

3. Collapse one compile wave from the max-safe gate
- Prefer a single prebuild strategy that satisfies both later test execution and runnable-target reuse
- Re-evaluate whether `cargo build --workspace --all-targets --all-features` is needed if `cargo test --no-run` already produces the required artifacts

4. Reduce simulator, wallets, and storage compile surface
- Pull heavyweight benches and optional GUI/debug features out of the default gate path
- Split large test-only modules and helpers so they do not force broad recompilation

5. Replace polling locks with blocking coordination
- Remove `sleep(Duration::from_millis(100))` loops in shared fixture/cache coordination
- Revisit whether all `scenario_1` tests must be globally serialized

6. Treat replay-contract work as a dedicated CPU optimization track
- Add a stable microbenchmark or Criterion-style bench around the replay/verify core
- Focus on crypto, hash, serialization, and proof verification kernels before broader harness changes

## ✅ Bottom Line

The single biggest optimization target is the Stage13 shared-case build and validation path in `z00z_simulator`. After that, the highest-return change is structural: stop paying for two nearly identical workspace compile waves inside the max-safe gate. The remaining large wins are reducing compile surface in `z00z_simulator`, `z00z_wallets`, and `z00z_storage`, then isolating the heavy replay-contract CPU path behind dedicated performance work.
