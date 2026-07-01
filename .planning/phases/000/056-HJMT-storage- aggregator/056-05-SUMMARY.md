---
phase: 056-HJMT-storage-aggregator
plan: 056-05
status: complete
completed_at: 2026-06-12
next_plan: 056-06
requirements-completed:
  - 056-G3
  - 056-G9
summary_artifact_for: .planning/phases/056-HJMT-storage- aggregator/056-05-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 056-05 Summary: YAML Materialization And Startup Preflight

## Completed Scope

`056-05` is complete for the live YAML-materialization and startup-preflight
slice.

The runtime now starts from one checked-in `config/hjmt_runtime/sim_5a7s/`
home instead of hidden topology constants. Aggregator, planner, storage, route,
and simulator-visible runtime inputs are loaded from YAML through the existing
repository config seams, rejected under `serde(deny_unknown_fields)`, and
recorded as exact config-digest evidence rather than treated as implied state.
The checked-in route table now has one canonical byte path and one canonical
`route_table_digest`
`000c78634c31e624c5e194378e6c7613e916e1975ca901e5d6416325c1d617e1` reused
across planner, storage, and per-aggregator config.

The live startup gate now fails closed before work begins if config load,
topology, route bytes, route digest, placement, journal lineage, backend mode
or generation, proof bytes, handoff metadata, or recovery tag assumptions are
wrong. The runtime rejects zero-shard topology, unknown standbys, impossible
placement coverage, wrong route digest binding, wrong expected journal
lineage, unsupported backend mode or generation, malformed proof bytes, and
unordered or drifted publication handoff metadata on the one existing startup
path rather than through a parallel validator or simulator layer.

The landed coverage also proves that runtime behavior materially changes when
ports, standby placement, planner mode, journal paths, backend selection, and
simulator runtime selection change in YAML. Config digests are now exported as
run evidence, but semantic truth remains on the existing runtime/storage seam:
planner truth stays runtime-owned, storage remains the only subtree and proof
owner, and startup preflight is only a fail-closed gate over those existing
authorities.

## Files Changed

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-05-SUMMARY.md`
- `Cargo.lock`
- `config/hjmt_runtime/sim_5a7s/`
- `crates/z00z_rollup_node/Cargo.toml`
- `crates/z00z_rollup_node/src/config.rs`
- `crates/z00z_rollup_node/src/lib.rs`
- `crates/z00z_rollup_node/src/runtime.rs`
- `crates/z00z_rollup_node/tests/support/test_hjmt_home.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs`
- `crates/z00z_simulator/Cargo.toml`
- `crates/z00z_simulator/src/config_accessors.rs`
- `crates/z00z_simulator/src/scenario_1/runner.rs`
- `crates/z00z_simulator/tests/test_hjmt_runtime_config.rs`
- `crates/z00z_storage/src/settlement/hjmt_config.rs`
- `crates/z00z_storage/src/settlement/mod.rs`
- `crates/z00z_storage/src/settlement/store.rs`

## Boundary Kept Intact

- YAML became the live config authority for runtime-visible topology and
  startup behavior, but no second orchestration or config-truth layer was
  introduced.
- Route bytes and `route_table_digest` stay bound to the runtime-owned shard
  routing contract; config digests are evidence only.
- Startup preflight validates placement, lineage, backend, proof, and handoff
  assumptions on the existing runtime/storage seam only; it does not mint new
  protocol truth.
- Storage remains the only owner of subtree lifecycle and proof truth, and the
  local durable backend contract remains the single live persistence authority.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md`
was used because the slash prompt is not a callable tool in this environment.

- Pass 1 found significant gaps: the config slice still lacked explicit TODO
  proof lanes for port drift, placement and backend-selection drift, scenario
  selection drift, and zero-shard topology rejection, and the broad release
  rerun exposed a release-only `recreate_storage_scope(...)` race in the
  simulator. The missing tests and the storage-scope lock fix were added.
- Pass 2 reran a repo-first residue scan against `056-TODO.md`,
  `056-CONTEXT.md`, `056-05-PLAN.md`, the config and preflight code, and the
  targeted tests. No significant issues remained.
- Pass 3 repeated a diff-focused review on the new preflight/test lanes and
  the storage-scope fix. No significant issues remained.
- Pass 4 reran a post-rename-fix residue scan for the helper-file rename,
  stale `hjmt_test_home.rs` references, and live `056-05` status strings in
  the active planning files. No significant issues remained.
- Pass 5 reran the same residue scan after `056-05-SUMMARY.md`, `STATE.md`,
  and `ROADMAP.md` were updated to make `056-06` the only live execution lane.
  No significant issues remained.

Two consecutive clean review passes were achieved on passes 4 and 5 after the
final helper-rename fix.

## Validation

Rust validation for this plan completed on the live tree before closeout.

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  as the mandatory fail-fast gate.
- `cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_preflight`
  passed.
- `cargo test -p z00z_rollup_node --release --features test-params-fast`
  passed.
- `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools`
  passed.
- `cargo test -p z00z_aggregators --release --features test-params-fast`
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast` passed.
- `cargo test -p z00z_wallets --release --test test_rename_guards test_test_file_prefix_guard -- --exact`
  passed after the helper-file rename.
- `cargo test --release` passed for the workspace.
- `git diff --check` is clean.

## Result

`056-05` is complete. Phase 056 now advances to `056-06-PLAN.md` for the
simulator stage-sync and runtime-evidence slice.

This summary does not claim simulator closeout, final fixture and benchmark
packet closure, or publication-lane completion; those remain owned by
`056-06` and `056-07`.
