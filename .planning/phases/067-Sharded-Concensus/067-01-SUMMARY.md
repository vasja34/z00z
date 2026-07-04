---
phase: 067
plan: 067-01
status: complete
completed_at: 2026-07-03
next_plan: 067-02
summary_artifact_for: .planning/phases/067-Sharded-Concensus/067-01-PLAN.md
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 067-01 Summary: Terminology And Boundary Cleanup

## Outcome

`067-01` is complete.

`PHASE-0` now closes on one live shard-runtime vocabulary and one honest local
consensus statement. Active runtime, rollup-node, simulator, fixture, and test
surfaces no longer carry live `standby` naming, the shard placement seam uses
`secondary` terminology only, and the repo-owned runtime docs keep the current
implementation scoped to deterministic local quorum rather than overclaiming
external BFT or Celestia behavior.

The closeout also includes the required release-only verification contract for
the whole Phase 067 plan packet, plus the blocker fix for the wallet rename
guard that rejected a test filename containing `phase`.

## Files Changed

- `.planning/phases/067-Sharded-Concensus/067-01-SUMMARY.md`
- `.planning/phases/067-Sharded-Concensus/067-01-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-01-PLAN.md` through `067-09-PLAN.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `config/hjmt_runtime/sim_5a7s/manifest.json`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-0/aggregator-config.yaml`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-1/aggregator-config.yaml`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-2/aggregator-config.yaml`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-3/aggregator-config.yaml`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-4/aggregator-config.yaml`
- `crates/z00z_core/tests/test_live_guardrails.rs`
- `crates/z00z_rollup_node/src/config.rs`
- `crates/z00z_rollup_node/src/runtime.rs`
- `crates/z00z_rollup_node/tests/support/test_hjmt_home.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`
- `crates/z00z_runtime/aggregators/README.md`
- `crates/z00z_runtime/aggregators/src/consensus_adapter.rs`
- `crates/z00z_runtime/aggregators/src/dist_sim.rs`
- `crates/z00z_runtime/aggregators/src/lib.rs`
- `crates/z00z_runtime/aggregators/src/placement.rs`
- `crates/z00z_runtime/aggregators/src/recovery.rs`
- `crates/z00z_runtime/aggregators/src/shard_exec.rs`
- `crates/z00z_runtime/aggregators/tests/fixtures/failover_v1/README.md`
- `crates/z00z_runtime/aggregators/tests/fixtures/failover_v1/manifest.json`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_consensus.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_dist_journal.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_publish.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_route_rollout.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_shard_routing.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_topology_support.rs`
- `crates/z00z_runtime/aggregators/tests/test_live_guardrails.rs`
- `crates/z00z_runtime/aggregators/tests/test_recovery_common.rs`
- `crates/z00z_runtime/aggregators/tests/test_recovery_failover.rs`
- `crates/z00z_runtime/aggregators/tests/test_secondary_terminology_guard.rs`
- `crates/z00z_runtime/validators/tests/generated_kani_validator_checkpoint_flow.rs`
- `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`
- `crates/z00z_runtime/watchers/src/status.rs`
- `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`
- `crates/z00z_simulator/src/config.rs`
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`
- `crates/z00z_simulator/tests/scenario_1/test_claim_tx_pipeline.rs`
- `crates/z00z_simulator/tests/scenario_1/test_hjmt_runtime_config.rs`
- `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs`
- `crates/z00z_simulator/tests/scenario_1/test_scenario_settlement.rs`
- `crates/z00z_simulator/tests/scenario_1/test_stage2_secret_artifacts.rs`
- `crates/z00z_storage/scripts/run_storage_settlement_bench.py`

## Landed Changes

- Canonical terminology cleanup
  - Active runtime/config/test surfaces now use `secondary`, `secondary_ids`,
    `SecondaryState`, and `TakeoverSecondary`-aligned wording only.
  - The SIM-5A7S runtime manifest, per-aggregator YAML, rollup-node loaders,
    simulator observability, watcher surfaces, validator publication checks,
    and aggregator tests all follow the same single vocabulary.
- Honest scope wording
  - `crates/z00z_runtime/aggregators/README.md` now states the live seam as
    deterministic local quorum over real runtime/storage primitives and keeps
    external replicated-log or network bindings adapter-only.
  - Active docs and tests no longer claim unimplemented BFT or Celestia
    behavior as if it were already live runtime truth.
- Permanent terminology guard
  - Added `test_secondary_terminology_guard.rs` to scan the active runtime,
    rollup-node, simulator, fixture, watcher, validator, and touched script
    roots for forbidden `standby` residue.
  - Renamed the guard file away from `test_phase067_terminology_guard.rs`
    because `crates/z00z_wallets/tests/test_rename_guards.rs` intentionally
    fails on filenames containing `phase`.
- Release-only execution contract
  - `067-01-PLAN.md` through `067-09-PLAN.md` now require release-only cargo
    verification in acceptance, task, and verify sections.
- Broad validation unblocker
  - Fixed two unrelated `z00z_core` guardrail-path drifts so the mandatory
    workspace-wide `cargo test --release` gate could run honestly on the
    current tree instead of staying blocked by stale include paths.

## Validation

Commands green during the final `067-01` closeout:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release`
- `rg -n "standby|TakeoverStandby|standby_ids" crates/z00z_runtime crates/z00z_rollup_node crates/z00z_simulator config/hjmt_runtime/sim_5a7s --glob '!**/*.md'`

The green broad release rerun included the key `067-01` runtime proofs in the
same pass, including `test_hjmt_consensus`, `test_hjmt_preflight`,
`test_hjmt_topology`, `test_secondary_terminology_guard`, the long
`scenario_1` integration binary, `z00z_storage`, `z00z_wallets`, and the late
watcher publication contract tail.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times in
YOLO mode, but the current runner did not provide a usable automated review
path for this slice.

- Attempt 1
  - `timeout 45s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-01-PLAN.md --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 83731 > 38936`
- Attempt 2
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-01-PLAN.md current_task="Terminology And Boundary Cleanup"'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 3
  - `timeout 45s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-01-PLAN.md current_task="Terminology And Boundary Cleanup" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66677 > 38936`

Equivalent workspace-first manual review was executed against the same scope.

- Pass 1
  - Re-read `067-01-PLAN.md`, `067-TODO.md`, the touched runtime/config/test
    files, and the live guardrail surfaces named by `PHASE-0`.
  - Result: found one real closeout blocker outside the planned rename itself:
    the new terminology guard filename still contained `phase`, so the wallet
    rename-guard suite rejected it. Renamed the file to
    `test_secondary_terminology_guard.rs`, updated its self-skip literals, and
    refreshed the matching `STATE.md` reference.
- Pass 2
  - Re-ran `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`,
    reran the active-surface `rg` audit, and re-read
    `crates/z00z_runtime/aggregators/README.md`,
    `crates/z00z_runtime/aggregators/src/placement.rs`, and
    `crates/z00z_rollup_node/src/config.rs`.
  - Result: clean. Bootstrap completed, the `rg` audit returned no hits, live
    structs and config validators used only `secondary` terms, and the runtime
    README stayed scoped to deterministic local quorum.
- Pass 3
  - Re-read `config/hjmt_runtime/sim_5a7s/manifest.json`,
    `config/hjmt_runtime/sim_5a7s/aggregators/agg-0/aggregator-config.yaml`,
    `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs`, and
    `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`, then finished on the
    green broad `cargo test --release`.
  - Result: clean. Fixture JSON/YAML kept only `secondary_ids`, the preflight
    and topology tests enforced the fail-closed secondary invariants, and the
    full release workspace gate passed.

Passes 2 and 3 were consecutive clean manual review runs after the last
in-scope fix.

## Closeout

`067-01` closes `PHASE-0` by removing live vocabulary drift before the shard
quorum work grows more stateful. The runtime now has one canonical
`secondary`-based placement story, one honest deterministic-local-quorum story,
one permanent guard against terminology regression, one release-only
verification contract across the full Phase 067 plan packet, and one honest
broad workspace validation result.

`067-02` is now the next canonical execution lane.
