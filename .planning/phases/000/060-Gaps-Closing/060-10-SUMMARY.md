---
phase: 060-Gaps-Closing
plan: 060-10
status: complete
completed_at: 2026-06-21
next_plan: 060-11
summary_artifact_for: .planning/phases/060-Gaps-Closing/060-10-PLAN.md
---

# 060-10 Summary: HJMT Measurement Lanes And A/B Rerun Packet

## Completed Scope

`060-10` is complete for `B5` and `B6`.

This slice makes the HJMT evidence contract explicit instead of letting
Criterion timings, whole-command resource numbers, scenario stage-runtime
splits, and user-facing throughput claims drift into one mixed benchmark
story. The canonical storage bench docs and helper now name the measurement
lanes directly, keep `durable_root_published_tps` as the only throughput lane
allowed to back release claims, and record the selected `shard_mapping` plus
`shard_mapping_scope` in the generated packet.

The refreshed A/B packet lives in the canonical Phase 058 bench home at
`crates/z00z_storage/outputs/settlement/hjmt_mapping_ab.{md,json}`. It compares
`aggregator_owned` and `shard_process` under the required same-hardware,
same-release-profile, same-shard-count, same-cache-mode, same-persistence-mode,
and same-route-generation contract. The packet reports the required metrics:
`durable_root_published_tps`, `worker_local_tps`, `hjmt_journal_sync_p50`,
publication latency, blocked time, CPU, RSS, total stage time, restart time,
and failover recovery time.

The numbers do not justify a production-default flip. The refreshed packet
shows `shard_process` ahead on durable and worker-local throughput
(`6882.43` vs `6468.26` durable TPS; `6892.31` vs `6477.23` worker-local TPS)
and slightly ahead on publication latency and blocked time, but it is slightly
worse on `hjmt_journal_sync_p50` (`9.575 ms` vs `9.069 ms`) and still reports
`failover_recovery_time_us = not_measured` for both mappings. Because the
Phase 060 contract requires durable-path and recovery evidence before a default
promotion, the repository verdict stays unchanged: keep `aggregator_owned` as
the production default and keep `shard_process` as an opt-in mapping only.

The continuation also removed the last broad-only validation blocker on the
post-`060-12` tree. The mutating `scenario_1` fixture-cache contract tests now
run under isolated sandbox cache roots instead of the shared repo cache root,
which removes shared-state interference inside
`crates/z00z_simulator/tests/scenario_1/main.rs` without changing production
fixture-cache logic. With that isolation in place, the previously interrupted
long `scenario_1` tail now completes inside the full workspace gate.

## Files Changed

- `.planning/phases/060-Gaps-Closing/060-10-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`
- `crates/z00z_storage/benches/settlement_benches.md`
- `crates/z00z_storage/tests/test_bench_lanes.rs`
- `crates/z00z_storage/scripts/run_storage_settlement_bench.py`
- `crates/z00z_storage/benches/settlement_shard.rs`
- `config/hjmt_runtime/sim_5a7s/manifest.json`
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`
- `crates/z00z_simulator/tests/scenario_1/test_fixture_cache_contract.rs`

## Boundary Kept

- The existing Phase 058 bench home remains the only canonical measured archive
  home. No second benchmark archive or second authority layer was introduced.
- `aggregator_owned` remains the production default. `shard_process` is still
  a YAML-selectable option, not a promoted live default.
- The A/B verdict is still fail-closed on missing recovery evidence. The packet
  says `failover_recovery_time_us = not_measured` explicitly instead of
  implying a win from worker-local or partial durable-path wins.
- The validation-support fix is test-only. Production fixture-cache logic was
  not widened or forked.
- No parallel module, route, or publication path was introduced.

## Review Loop

Manual fallback for `/GSD-Review-Tasks-Execution` was used because the slash
prompt is not a callable tool in this environment.

- Pass 1 re-audited `060-10-PLAN.md`, the bench docs, the bench helper, and
  the refreshed `hjmt_mapping_ab` artifacts. It confirmed explicit lane
  separation, explicit `durable_root_published_tps` wording, and an honest
  no-promotion guard tied to the missing failover metric.
- Pass 2 re-audited the interrupted broad release gate and localized the real
  blocker to shared cache-root interference inside the broad
  `tests/scenario_1/main.rs` binary. The fix was narrowed to test sandboxing
  only.
- Pass 3 reran the scoped simulator and broad workspace release gates after the
  sandbox fix and confirmed the long `scenario_1` tail completes instead of
  stopping at the cache-contract cluster.
- Pass 4 rechecked the refreshed A/B numbers, summary wording, and
  `STATE`/`ROADMAP` status against the final green evidence. No significant
  issues remained.

Two consecutive clean review passes were achieved on passes 3 and 4 after the
test-isolation fix landed.

## Validation

- Mandatory bootstrap gate passed:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- Targeted bench-lane validation passed:
  `cargo test -p z00z_storage --release --test test_bench_lanes -- --nocapture`
- Refreshed A/B packet passed in the canonical output home:
  `KEEP=$(cd crates/z00z_storage/outputs/settlement && find . -mindepth 1 -maxdepth 1 -printf "%P;" | sed "s/;$//"); Z00Z_STORAGE_SETTLEMENT_BENCH_KEEP="$KEEP" ./crates/z00z_storage/scripts/run_storage_settlement_bench.py --bench hjmt_mapping_ab`
- Scoped simulator cache-contract validation passed:
  `cargo test -p z00z_simulator --release --test scenario_1 test_fixture_cache_contract -- --nocapture`
- Broad workspace release validation passed:
  `cargo test --release`
- The broad workspace rerun now completes the long default-feature
  `scenario_1` binary inside the green workspace gate:
  `tests/scenario_1/main.rs` finished with `252 passed; 0 failed; 1 ignored; finished in 1649.56s`

## Result

`060-10` is complete. Phase 060 now advances to `060-11-PLAN.md` for the
verification-pipeline performance and final closure rerun slice, while the
production-default HJMT verdict remains `aggregator_owned`.
