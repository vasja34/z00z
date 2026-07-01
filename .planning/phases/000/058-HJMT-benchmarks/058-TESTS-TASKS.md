---
phase: 058-HJMT-benchmarks
artifact: tests-tasks
status: planned
source:
  - 058-TEST-SPEC.md
  - 058-TODO.md
  - 058-CONTEXT.md
  - 058-01-PLAN.md
  - 058-02-PLAN.md
  - 058-03-PLAN.md
  - 058-04-PLAN.md
  - 058-05-PLAN.md
  - 058-06-PLAN.md
  - 058-07-PLAN.md
updated: 2026-06-15
---

# Phase 058 Test Tasks

**Phase:** `058-HJMT-benchmarks`
**Status:** planned implementation-order artifact
**Companion spec:** `058-TEST-SPEC.md`

## Goal

This file turns the Phase 058 packet into an engineer-ready test and benchmark
work order. It exists so execution can proceed without guessing:

- which live file home owns each readiness scenario;
- which lanes are exact live, successor live, or still proposed;
- which artifacts prove score or readiness claims;
- which final packets must stay release-mode only;
- which verification waves must land before the final verdict is honest.

## Scope Inputs

- `058-TODO.md` is the canonical source for gates, tests, benches, execution
  profiles, artifacts, fixture families, and exit criteria.
- `058-CONTEXT.md` freezes the owner map, gate routing, and path honesty rules.
- `058-TEST-SPEC.md` freezes the scenario ledger, verification order, and
  live-versus-proposed test map.
- `058-01-PLAN.md` through `058-07-PLAN.md` freeze the execution order.

## Execution Strategy

- Wave 1 lands first because every later score or readiness row needs the
  evidence ledger.
- Wave 2 lands before Wave 3 because config realism and release-only trace
  ownership depend on the public simulator lane being frozen.
- Wave 3 lands before Wave 4 because final packet closure is not honest until
  YAML control, import/export, and startup reject behavior are real.
- Wave 4 lands before Wave 5 because measured reports must cite the final
  runtime and publication packets.
- Wave 5 lands before Wave 6 because wallet and dynamic-scope closeout depend on
  the real release lane and the honest benchmark/archive story.
- Wave 6 lands before Wave 7 because the final fixture matrix and repository
  verdict must aggregate all earlier readiness slices.

## Hard Rules

- Reuse existing live owner homes whenever they already own the behavior.
- Do not turn successor or proposed names into false exact-path claims.
- Do not create a second benchmark harness or a second release-lane simulator.
- Keep `SIM-BATCH-1000` heavy-only.
- Keep score and compression verdicts evidence-backed.
- Keep final verdict wording inside the allowed vocabulary only.
- When a commit is needed, use `/z00z-git-versioning`.

## Verify Block Template

Every Rust or test-affecting change in Phase 058 must verify in this order:

```bash
./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh
```

If bootstrap fails, stop, fix the regression, and rerun it before any broader
validation.

Then run the wave-specific targeted commands below, and close with:

```bash
cargo test --release
```

After the targeted commands, run
`/.github/prompts/gsd-review-tasks-execution.prompt.md`
(` /GSD-Review-Tasks-Execution `) in YOLO mode at least three times and
continue until at least two consecutive runs report no significant issues.

## Scenario To Plan Crosswalk

| Plan slice | Test scenarios from `058-TEST-SPEC.md` | Main owner homes |
| --- | --- | --- |
| `058-01` | `058-SC-01` | `.planning/phases/058-HJMT-benchmarks/058-EVIDENCE-LEDGER.md`, `.planning/phases/058-HJMT-benchmarks/058-SOURCE-AUDIT.md`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_positive/manifest.json`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_negative/manifest.json`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/root_generation_migration/manifest.json`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_route_table_v1/manifest.json`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_root_leaf_v1/manifest.json`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/checkpoint_publication_v1/manifest.json`, `crates/z00z_runtime/aggregators/tests/fixtures/failover_v1/manifest.json`, and `crates/z00z_storage/benches/settlement_benches.md` |
| `058-02` | `058-SC-02` | `crates/z00z_simulator/tests/test_hjmt_runtime_config.rs`, `crates/z00z_simulator/tests/test_scenario_settlement.rs`, and `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` |
| `058-03` | `058-SC-03`, `058-SC-04` | `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs`, `crates/z00z_rollup_node/tests/test_hjmt_process.rs`, `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`, `crates/z00z_storage/tests/test_hjmt_import_export.rs`, `crates/z00z_storage/tests/test_hjmt_storage_boundary.rs`, `crates/z00z_storage/tests/test_hjmt_backend_conformance.rs`, `config/hjmt_runtime/sim_5a7s/manifest.json`, `config/hjmt_runtime/sim_5a7s/aggregators/agg-0/aggregator-config.yaml`, `config/hjmt_runtime/sim_5a7s/aggregators/agg-1/aggregator-config.yaml`, `config/hjmt_runtime/sim_5a7s/aggregators/agg-2/aggregator-config.yaml`, `config/hjmt_runtime/sim_5a7s/aggregators/agg-3/aggregator-config.yaml`, `config/hjmt_runtime/sim_5a7s/aggregators/agg-4/aggregator-config.yaml`, `config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml`, `config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml`, `crates/z00z_storage/tests/test_hjmt_batch_commit.rs`, and `crates/z00z_storage/tests/test_hjmt_batch_recovery.rs` |
| `058-04` | `058-SC-05`, `058-SC-06` | `crates/z00z_runtime/aggregators/tests/test_hjmt_publish.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_planner.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_split_brain_fencing.rs`, `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`, `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`, and `crates/z00z_simulator/tests/test_scenario_settlement.rs` / `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` |
| `058-05` | `058-SC-07`, `058-SC-08`, `058-SC-09` | `crates/z00z_storage/benches/settlement_proofs.rs`, `crates/z00z_storage/benches/settlement_hjmt.rs`, `crates/z00z_storage/benches/settlement_shard.rs`, `crates/z00z_storage/benches/settlement_nested.rs`, `crates/z00z_storage/benches/adaptive_policy_bench.rs`, `crates/z00z_storage/tests/test_bench_lanes.rs`, `crates/z00z_storage/scripts/run_storage_settlement_bench.py`, and `crates/z00z_storage/outputs/settlement/` |
| `058-06` | `058-SC-10`, `058-SC-11` | `crates/z00z_storage/tests/test_hjmt_scope_birth.rs`, `crates/z00z_storage/tests/test_hjmt_historical_proofs.rs`, `crates/z00z_storage/tests/test_hjmt_adaptive_policy_proofs.rs`, `crates/z00z_storage/tests/test_hjmt_transition_proofs.rs`, `crates/z00z_storage/tests/test_hjmt_privacy_regression.rs`, `crates/z00z_storage/tests/test_occupancy_privacy.rs`, `crates/z00z_storage/tests/test_occupancy_evidence.rs`, `crates/z00z_simulator/tests/test_stage7_jmt_wallet_scan.rs`, and `crates/z00z_simulator/tests/test_hjmt_e2e.rs` |
| `058-07` | `058-SC-12`, `058-SC-13` | the exact fixture manifest and README homes listed under Wave 7, `.planning/phases/058-HJMT-benchmarks/058-EVIDENCE-LEDGER.md`, `.planning/phases/058-HJMT-benchmarks/058-VALIDATION.md`, `.planning/phases/058-HJMT-benchmarks/058-SUMMARY.md`, `ROADMAP.md`, and `STATE.md` |

## Wave 1: `058-01` Evidence Ledger

**Purpose:** freeze the claim ledger and live/proposed acceptance map before
later slices claim closure.

**Files to create or extend**

- `.planning/phases/058-HJMT-benchmarks/058-EVIDENCE-LEDGER.md`
- `.planning/phases/058-HJMT-benchmarks/058-SOURCE-AUDIT.md`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_positive/manifest.json`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_negative/manifest.json`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/root_generation_migration/manifest.json`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_route_table_v1/manifest.json`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_root_leaf_v1/manifest.json`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/checkpoint_publication_v1/manifest.json`
- `crates/z00z_runtime/aggregators/tests/fixtures/failover_v1/manifest.json`
- `crates/z00z_storage/benches/settlement_benches.md`

**Implementation tasks**

- Map every gate, fixture family, evidence-gap class, report, and final verdict
  row to one command and one evidence pointer.
- Classify exact live, successor live, and proposed homes.
- Keep unsupported claims explicit.

**Targeted commands**

```bash
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof_negative -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_root_generation -- --nocapture
cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_shard_routing -- --nocapture
```

## Wave 2: `058-02` Release-Mode Simulator And Stage Sync

**Purpose:** freeze the public release-lane simulator and stage-sync gate.

**Files to create or extend**

- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
- `crates/z00z_simulator/tests/test_hjmt_runtime_config.rs`
- `crates/z00z_simulator/tests/test_scenario_settlement.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`

**Implementation tasks**

- Run the public simulator path in `--release`.
- Keep `cfg_flow.json` through `watch_flow.json` authoritative and attached to
  inherited lineage.
- Keep `proc_flow.json` and `recovery_flow.json` attached to the same packet,
  keep config digests and process map explicit, and add new Phase 058 artifacts
  only when their exact homes are emitted and verified.
- Keep the literal observability inventory explicit:
  `run_meta.json`, `cfg_flow.json`, `tx_flow.json`, `route_flow.json`,
  `plan_flow.json`, `journal_flow.json`, `scope_flow.json`, `leaf_flow.json`,
  `proof_flow.json`, `pub_flow.json`, `val_flow.json`, `watch_flow.json`,
  `wallet_scan.json`, `asset_flow.json`, `right_flow.json`, `hist_flow.json`,
  `occ_flow.json`, `recovery_flow.json`, and `sim_summary.md`.
- Keep redaction compliance for public evidence explicit and keep private
  debug-only artifacts off the public gate.
- Continue the multi-aggregator simulation lane from Phases 056 and 057 on this
  one release-mode successor path instead of forking a second simulator story.

**Targeted commands**

```bash
cargo test -p z00z_simulator --release --features test-params-fast --test test_hjmt_runtime_config -- --nocapture
cargo test -p z00z_simulator --release --features test-params-fast --test test_scenario_settlement -- --nocapture
cargo test -p z00z_simulator --release --features test-params-fast --test test_scenario1_stage_surface -- --nocapture
cargo run --release -p z00z_simulator --bin scenario_1 --features test-params-fast
```

## Wave 3: `058-03` Config Realism, Import/Export, And Startup Rejects

**Purpose:** prove the checked runtime home is parameterized and fail-closed.

**Files to create or extend**

- `config/hjmt_runtime/sim_5a7s/manifest.json`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-0/aggregator-config.yaml`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-1/aggregator-config.yaml`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-2/aggregator-config.yaml`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-3/aggregator-config.yaml`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-4/aggregator-config.yaml`
- `config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml`
- `config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml`
- `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_process.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`
- `crates/z00z_simulator/tests/test_hjmt_runtime_config.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- `crates/z00z_storage/tests/test_hjmt_import_export.rs`
- `crates/z00z_storage/tests/test_hjmt_storage_boundary.rs`
- `crates/z00z_storage/tests/test_hjmt_backend_conformance.rs`
- `crates/z00z_storage/tests/test_hjmt_batch_commit.rs`
- `crates/z00z_storage/tests/test_hjmt_batch_recovery.rs`

**Implementation tasks**

- Prove YAML changes alter live behavior.
- Add one positive non-`5x7` topology from disk.
- Freeze exact import/export and startup fail-closed acceptance homes.
- Keep config-surface coverage, journal-baseline/WAL-boundary coverage,
  restart/persistence coverage, and the RedB-backed local journal baseline in
  this same slice.
- Treat ordered WAL and replicated-log directions as future `JournalBackend`
  adapters behind conformance/equal-durability gates, not as a second active
  runtime layer.

**Targeted commands**

```bash
cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_preflight -- --nocapture
cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_process -- --nocapture
cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_topology -- --nocapture
cargo test -p z00z_simulator --release --features test-params-fast --test test_hjmt_runtime_config -- --nocapture
cargo test -p z00z_simulator --release --features test-params-fast --test test_scenario1_stage_surface -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_commit -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_recovery -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_import_export -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_storage_boundary -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_backend_conformance -- --nocapture
```

## Wave 4: `058-04` Final Runtime And Publication Packets

**Purpose:** close `SIM-5A7S` and `SIM-5A7S-PUB` on the public release lane.

**Files to create or extend**

- `crates/z00z_runtime/aggregators/tests/test_hjmt_publish.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_planner.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_split_brain_fencing.rs`
- `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`
- `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`
- `crates/z00z_simulator/tests/test_scenario_settlement.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`

**Implementation tasks**

- Close the final runtime packet with process and route evidence.
- Close the final publication packet with join/transfer/carry-forward and
  validator/watcher evidence.
- Keep one digest story only.
- Make deterministic re-encoding, five independent processes, planner
  equivalence, split-brain fencing, wrong-lineage rejection, and
  route-migration crash recovery explicit in the packet closure.

**Targeted commands**

```bash
cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_planner -- --nocapture
cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_publish -- --nocapture
cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_join -- --nocapture
cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_migrate -- --nocapture
cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_failover_same_lineage -- --nocapture
cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_split_brain_fencing -- --nocapture
cargo test -p z00z_validators --release --test test_hjmt_publication_contract -- --nocapture
cargo test -p z00z_watchers --release --test test_hjmt_publication_contract -- --nocapture
cargo test -p z00z_simulator --release --features test-params-fast --test test_scenario_settlement -- --nocapture
cargo test -p z00z_simulator --release --features test-params-fast --test test_scenario1_stage_surface -- --nocapture
```

## Wave 5: `058-05` Benchmarks, `SIM-BATCH-1000`, And Score Discipline

**Purpose:** close the measured readiness and score story.

**Files to create or extend**

- `crates/z00z_storage/benches/settlement_proofs.rs`
- `crates/z00z_storage/benches/settlement_hjmt.rs`
- `crates/z00z_storage/benches/settlement_shard.rs`
- `crates/z00z_storage/benches/settlement_nested.rs`
- `crates/z00z_storage/benches/adaptive_policy_bench.rs`
- `crates/z00z_storage/benches/settlement_benches.md`
- `crates/z00z_storage/tests/test_bench_lanes.rs`
- `crates/z00z_storage/scripts/run_storage_settlement_bench.py`

**Implementation tasks**

- Close query/search/shard-scaling and proof-size rows.
- Close `SIM-BATCH-1000` as heavy-only.
- Keep `SIM-SMALL` at `16-64`, `SIM-MEDIUM` at `128-256`, and
  `SIM-CACHE-EDGE` at `cap - 1/cap/cap + 1/2 * cap` as correctness profiles.
- Keep explicit cross-shard batch reject coverage inside the heavy profile.
- Archive reports and classify score or compression claims honestly.
- Preserve raw timing slices for planning, child commit, parent commit, journal
  sync, recovery replay, search/query time, and proof time.
- Resolve `outputs/settlement` vs `outputs/assets` explicitly.

**Targeted commands**

```bash
cargo test -p z00z_storage --release --features test-params-fast --test test_bench_lanes -- --nocapture
cargo bench -p z00z_storage --bench settlement_proofs --no-run
cargo bench -p z00z_storage --bench settlement_hjmt --no-run
cargo bench -p z00z_storage --bench settlement_shard --no-run
cargo bench -p z00z_storage --bench settlement_nested --no-run
cargo bench -p z00z_storage --bench adaptive_policy_bench --no-run
./crates/z00z_storage/scripts/run_storage_settlement_bench.py --bench settlement_hjmt -- --sample-size 10 --warm-up-time 0.01 --measurement-time 0.02
```

## Wave 6: `058-06` Dynamic Scope And Wallet Closure

**Purpose:** close the user-visible readiness story.

**Files to create or extend**

- `crates/z00z_storage/tests/test_hjmt_scope_birth.rs`
- `crates/z00z_storage/tests/test_hjmt_historical_proofs.rs`
- `crates/z00z_storage/tests/test_hjmt_adaptive_policy_proofs.rs`
- `crates/z00z_storage/tests/test_occupancy_privacy.rs`
- `crates/z00z_storage/tests/test_occupancy_evidence.rs`
- `crates/z00z_simulator/tests/test_hjmt_e2e.rs`
- `crates/z00z_simulator/tests/test_stage7_jmt_wallet_scan.rs`
- `crates/z00z_simulator/tests/test_scenario_settlement.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- `crates/z00z_storage/tests/test_hjmt_transition_proofs.rs`
- `crates/z00z_storage/tests/test_hjmt_privacy_regression.rs`

**Implementation tasks**

- Prove first-seen scope birth under restart and failover.
- Prove proof-before-ownership and final wallet promotion.
- Prove split/merge transition closure and route migration near a
  scope-creating batch on the same proof-boundary slice.
- Keep positive and negative wallet proof-before-ownership cases explicit.
- Keep historical and occupancy replay bound to imported artifacts.
- Keep imported Phase 057 publication playback, `hist_flow.json` and
  `occ_flow.json` completeness, stale-route rejects, and lineage-disagreement
  rejects explicit in this wave.

**Targeted commands**

```bash
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_scope_birth -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_historical_proofs -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_adaptive_policy_proofs -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_occupancy_privacy -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_occupancy_evidence -- --nocapture
cargo test -p z00z_simulator --release --features test-params-fast --test test_stage7_jmt_wallet_scan -- --nocapture
cargo test -p z00z_simulator --release --features test-params-fast --test test_scenario_settlement -- --nocapture
cargo test -p z00z_simulator --release --features test-params-fast --test test_scenario1_stage_surface -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_transition_proofs -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_privacy_regression -- --nocapture
cargo test -p z00z_simulator --release --features test-params-fast --test test_hjmt_e2e -- --nocapture
```

## Wave 7: `058-07` Fixture Matrix And Final Verdict

**Purpose:** aggregate the full closure matrix and synchronize final state.

**Files to create or extend**

- `.planning/phases/058-HJMT-benchmarks/058-EVIDENCE-LEDGER.md`
- `.planning/phases/058-HJMT-benchmarks/058-VALIDATION.md`
- `.planning/phases/058-HJMT-benchmarks/058-SUMMARY.md`
- `ROADMAP.md`
- `STATE.md`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_positive/manifest.json`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_positive/README.md`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_negative/manifest.json`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_negative/README.md`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/root_generation_migration/manifest.json`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/root_generation_migration/README.md`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_route_table_v1/manifest.json`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_route_table_v1/README.md`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_root_leaf_v1/manifest.json`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_root_leaf_v1/README.md`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/checkpoint_publication_v1/manifest.json`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/checkpoint_publication_v1/README.md`
- `crates/z00z_runtime/aggregators/tests/fixtures/failover_v1/manifest.json`
- `crates/z00z_runtime/aggregators/tests/fixtures/failover_v1/README.md`

**Implementation tasks**

- Close every required fixture family and `12.1` evidence-gap class.
- Rate the repository honestly under the allowed verdict vocabulary only.
- Keep roadmap, state, and phase-local closeout artifacts synchronized.

**Targeted commands**

```bash
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof_negative -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_root_generation -- --nocapture
cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_shard_routing -- --nocapture
cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_failover_same_lineage -- --nocapture
```

## Task Ledger

| Task ID | Scenario IDs | File homes | Action | Done when |
| --- | --- | --- | --- | --- |
| `058-TT-01` | `058-SC-01` | `.planning/phases/058-HJMT-benchmarks/058-EVIDENCE-LEDGER.md`, `.planning/phases/058-HJMT-benchmarks/058-SOURCE-AUDIT.md`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_positive/manifest.json`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_negative/manifest.json`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/root_generation_migration/manifest.json`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_route_table_v1/manifest.json`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_root_leaf_v1/manifest.json`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/checkpoint_publication_v1/manifest.json`, `crates/z00z_runtime/aggregators/tests/fixtures/failover_v1/manifest.json`, and `crates/z00z_storage/benches/settlement_benches.md` | Freeze the evidence ledger, design-artifact rows, live/successor/proposed map, and unsupported-claim rows. | Every readiness row resolves to exact commands, artifacts, verdicts, and honest owner status. |
| `058-TT-02` | `058-SC-02` | `crates/z00z_simulator/tests/test_hjmt_runtime_config.rs`, `crates/z00z_simulator/tests/test_scenario_settlement.rs`, `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`, `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`, `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`, and `crates/z00z_simulator/src/scenario_1/runtime_observability.rs` | Extend the public `--release` lane, stage sync, trace lineage, config digests, process map, and redaction-safe packet. | One public release packet exists with complete lineage and no private-lane dependence. |
| `058-TT-03` | `058-SC-03`, `058-SC-04` | `config/hjmt_runtime/sim_5a7s/manifest.json`, `config/hjmt_runtime/sim_5a7s/aggregators/agg-0/aggregator-config.yaml`, `config/hjmt_runtime/sim_5a7s/aggregators/agg-1/aggregator-config.yaml`, `config/hjmt_runtime/sim_5a7s/aggregators/agg-2/aggregator-config.yaml`, `config/hjmt_runtime/sim_5a7s/aggregators/agg-3/aggregator-config.yaml`, `config/hjmt_runtime/sim_5a7s/aggregators/agg-4/aggregator-config.yaml`, `config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml`, `config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml`, `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs`, `crates/z00z_rollup_node/tests/test_hjmt_process.rs`, `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`, `crates/z00z_storage/tests/test_hjmt_import_export.rs`, `crates/z00z_storage/tests/test_hjmt_storage_boundary.rs`, `crates/z00z_storage/tests/test_hjmt_backend_conformance.rs`, `crates/z00z_storage/tests/test_hjmt_batch_commit.rs`, and `crates/z00z_storage/tests/test_hjmt_batch_recovery.rs` | Close YAML realism, non-`5x7` topology, journal baseline, restart, import/export, and startup fail-closed rows. | Behavior changes from disk only and every bad-state row rejects at the intended stage. |
| `058-TT-04` | `058-SC-05`, `058-SC-06` | `crates/z00z_runtime/aggregators/tests/test_hjmt_planner.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_publish.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_split_brain_fencing.rs`, `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`, `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`, `crates/z00z_simulator/tests/test_scenario_settlement.rs`, and `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` | Close final runtime and publication packets on one lineage with deterministic re-encoding and planner equivalence. | `SIM-5A7S` and `SIM-5A7S-PUB` are both explicit, replayable, and digest-consistent. |
| `058-TT-05` | `058-SC-07`, `058-SC-08`, `058-SC-09` | `crates/z00z_storage/benches/settlement_proofs.rs`, `crates/z00z_storage/benches/settlement_hjmt.rs`, `crates/z00z_storage/benches/settlement_shard.rs`, `crates/z00z_storage/benches/settlement_nested.rs`, `crates/z00z_storage/benches/adaptive_policy_bench.rs`, `crates/z00z_storage/tests/test_bench_lanes.rs`, `crates/z00z_storage/scripts/run_storage_settlement_bench.py`, and `crates/z00z_storage/outputs/settlement/` | Close heavy profile, measurement matrix, Design baseline lanes, archive-home honesty, and compression verdict. | All score rows are evidence-backed and the heavy profile stays heavy-only. |
| `058-TT-06` | `058-SC-10`, `058-SC-11` | `crates/z00z_storage/tests/test_hjmt_scope_birth.rs`, `crates/z00z_storage/tests/test_hjmt_historical_proofs.rs`, `crates/z00z_storage/tests/test_hjmt_adaptive_policy_proofs.rs`, `crates/z00z_storage/tests/test_hjmt_transition_proofs.rs`, `crates/z00z_storage/tests/test_hjmt_privacy_regression.rs`, `crates/z00z_storage/tests/test_occupancy_privacy.rs`, `crates/z00z_storage/tests/test_occupancy_evidence.rs`, `crates/z00z_simulator/tests/test_stage7_jmt_wallet_scan.rs`, `crates/z00z_simulator/tests/test_hjmt_e2e.rs`, `crates/z00z_simulator/tests/test_scenario_settlement.rs`, and `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` | Close first-seen scope birth, proof-before-ownership, historical replay, occupancy privacy, and lineage-disagreement rejects. | User-visible readiness is proven end to end and reinterpretation drift rejects. |
| `058-TT-07` | `058-SC-12`, `058-SC-13` | the exact fixture manifest and README homes listed under Wave 7, `.planning/phases/058-HJMT-benchmarks/058-EVIDENCE-LEDGER.md`, `.planning/phases/058-HJMT-benchmarks/058-VALIDATION.md`, `.planning/phases/058-HJMT-benchmarks/058-SUMMARY.md`, `ROADMAP.md`, and `STATE.md` | Aggregate fixture-family and evidence-gap closure and rate the repository honestly. | Final verdict, validation packet, roadmap, and state all describe the same evidence-backed state. |

## Required Fixture Tasks

| Fixture task | Owner home | Minimum content |
| --- | --- | --- |
| `SRT-*`, `SRL-*`, `CPP-*`, `FOV-*`, `BPB-*`, and `RGM-*` closure | `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_positive/manifest.json`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_positive/README.md`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_negative/manifest.json`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_negative/README.md`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/root_generation_migration/manifest.json`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/root_generation_migration/README.md`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_route_table_v1/manifest.json`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_route_table_v1/README.md`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_root_leaf_v1/manifest.json`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_root_leaf_v1/README.md`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/checkpoint_publication_v1/manifest.json`, `crates/z00z_storage/tests/fixtures/hjmt_upgrade/checkpoint_publication_v1/README.md`, `crates/z00z_runtime/aggregators/tests/fixtures/failover_v1/manifest.json`, and `crates/z00z_runtime/aggregators/tests/fixtures/failover_v1/README.md` | Canonical bytes where applicable, exact mutation point, exact reject stage, regeneration command, expected verdict, and evidence pointer. |
| `12.1` evidence-gap classes | `.planning/phases/058-HJMT-benchmarks/058-EVIDENCE-LEDGER.md` and `.planning/phases/058-HJMT-benchmarks/058-VALIDATION.md` | Exact artifact, owner home, command, verdict, and explicit open/closed status for `Current HJMT root set`, `Independent proof batch`, `Shared proof vector`, `Tampered shared proof set`, `Bucket commit fixture`, `Backend conformance fixture`, `Route migration fixture`, `Failover fixture`, `Historical proof fixture`, and `Occupancy fixture`. |
| Release evidence packet rows | `crates/z00z_simulator/tests/test_scenario_settlement.rs`, `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_publish.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`, and `crates/z00z_runtime/aggregators/tests/test_hjmt_split_brain_fencing.rs` | Config digests, process ids, ports, journal paths, route/publication digests, owner/standby assignments, restart/failover verdicts, split-brain fencing verdicts, and redaction-safe artifact paths. |
| Measurement report rows | `crates/z00z_storage/outputs/settlement/` | Query/search/lookups/scaling/proof-size fields, exact commands, profile id, and verdict class on the one canonical measured-report home. |

## Owned Fixture And Evidence Ledger

- `SRT-G-001..004`
- `SRT-T-001..008`
- `SRL-G-001..004`
- `SRL-T-001..006`
- `CPP-G-001..005`
- `CPP-T-001..007`
- `FOV-001`
- `FOV-T-001`
- `FOV-T-002`
- `FOV-G-002`
- `FOV-G-003`
- `FOV-G-004`
- `BPB-G-001..005`
- `BPB-T-001..008`
- `RGM-G-001`
- `RGM-T-001`
- `Current HJMT root set`
- `Independent proof batch`
- `Shared proof vector`
- `Tampered shared proof set`
- `Bucket commit fixture`
- `Backend conformance fixture`
- `Route migration fixture`
- `Failover fixture`
- `Historical proof fixture`
- `Occupancy fixture`

## Mandatory Review Loop

Every execution closeout for this packet must run
`/.github/prompts/gsd-review-tasks-execution.prompt.md`
(` /GSD-Review-Tasks-Execution `) in YOLO mode at least three times.

Do not stop the review loop until at least two consecutive runs report no
significant issues, and fix every material warning or finding before claiming
completion.

## Definition Of Done

Phase 058 test implementation is not done until all of the following are true:

1. Every gate `058-G1` through `058-G13` has at least one primary passing test
   or artifact owner and one explicit evidence pointer.
2. `058-TEST-SPEC.md` and `058-TESTS-TASKS.md` describe the same live,
   successor, and proposed homes with no speculative drift.
3. The public release lane proves config digests, process map, trace set,
   stage sync, and redaction-safe artifacts under `--release`.
4. YAML realism, extra topology, import/export, restart, journal baseline,
   startup fail-closed, wallet proof-before-ownership, historical replay, and
   occupancy replay all have explicit pass/fail assertions and owner homes.
5. `SIM-SMALL`, `SIM-MEDIUM`, and `SIM-CACHE-EDGE` remain correctness
   profiles, and `SIM-BATCH-1000` remains heavy-only.
6. Score and compression claims are verdict rows backed by measured reports,
   and unsupported claims remain explicit.
7. Final fixture closure and final repository verdict are synchronized across
   the phase packet, `ROADMAP.md`, and `STATE.md`.
