---
phase: 060-Gaps-Closing
plan: 060-05
status: complete
completed_at: 2026-06-20
next_plan: 060-06
summary_artifact_for: .planning/phases/060-Gaps-Closing/060-05-PLAN.md
---

# 060-05 Summary: HJMT Decommission Coverage And `3A7S -> 2A7S -> 5A7S` Scenario

## Completed Scope

`060-05` is complete for the Phase 060 HJMT decommission and staged-topology
slice.

This slice closes `B3` by making aggregator removal explicit on the existing
HJMT lawfulness seam. The runtime tests now prove that decommissioning an
aggregator that owns multiple shards removes stale owner, standby, and route
references; redistributes affected shards only to lawful prior standbys; and
preserves same-lineage takeover semantics instead of creating a second recovery
story.

This slice also closes `B4` by adding one staged `3A7S -> 2A7S -> 5A7S`
scenario on the existing runtime and simulator evidence path. The new staged
fixtures and runtime-observability packet prove that all seven shards remain
owned through fail-down and re-expansion, route generations advance
monotonically, the removed aggregator does not reappear in owner or standby
tables, and publication continuity plus prior lineage remain intact across the
same canonical HJMT contract.

The implementation stayed on the existing test, simulator, and evidence seams.
No production recovery, placement, service, or shard-execution semantics were
rewritten for this slice.

## Files Changed

- `.planning/phases/060-Gaps-Closing/060-05-SUMMARY.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `crates/z00z_runtime/aggregators/tests/support/test_hjmt_topology_support.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_split_brain_fencing.rs`
- `crates/z00z_simulator/src/config.rs`
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`
- `crates/z00z_simulator/tests/test_hjmt_runtime_config.rs`
- `crates/z00z_simulator/tests/test_scenario_settlement.rs`

## Boundary Kept

- No second topology harness was introduced for decommission or staged
  topology work.
- No second publication truth path was introduced; the staged packet extends
  the existing runtime-observability and simulator evidence surface.
- `SIM-5A7S` remains the canonical default runtime profile; the staged
  `3A7S -> 2A7S -> 5A7S` flow is additive evidence and not a production-default
  topology switch.
- The removed aggregator is modeled explicitly as `agg-5` during decommission,
  while the final `5A7S` stage reuses the canonical `0..4` ownership set.
- `crates/z00z_runtime/aggregators/src/recovery.rs`,
  `crates/z00z_runtime/aggregators/src/placement.rs`,
  `crates/z00z_runtime/aggregators/src/service.rs`, and
  `crates/z00z_runtime/aggregators/src/shard_exec.rs` were left untouched, so
  this slice did not create a parallel live implementation path.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md` was
used because the slash prompt is not a callable tool in this environment.

- Pass 1 found a targeted failover compile gap: the new tests tried to use `?`
  on `validate_migration(...)` and `RecoveryBoundary::resume(...)`, but
  `RouteErr` and `RejectRecord` do not implement `StdError`. Those call sites
  were converted to explicit `expect(...)` assertions with phase-specific
  messages.
- Pass 2 reran the targeted release anchors and exposed simulator expectation
  drift: `hist_flow.route_migration_rows` still assumed two rows while the new
  staged packet emits three. The history assertions were expanded to include
  the staged `SIM-3A7S-2A7S-5A7S` row.
- Pass 3 reran the simulator and runtime anchors, then rechecked grep coverage
  for staged-topology, decommission, lineage, and stale-reference assertions.
  Scoped `git diff --check` on the changed slice files was clean.
- Pass 4 reran the full `cargo test --release` gate on the final tree and
  rechecked that the slice stayed on the existing HJMT and simulator seams
  without opening a second authority path.

Two consecutive clean review passes were achieved on passes 3 and 4.

## Validation

- Mandatory bootstrap gate passed on the final tree:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_aggregators --release --test test_hjmt_join -- --nocapture`
  passed.
- `cargo test -p z00z_aggregators --release --test test_hjmt_migrate -- --nocapture`
  passed.
- `cargo test -p z00z_aggregators --release --test test_hjmt_failover_same_lineage -- --nocapture`
  passed.
- `cargo test -p z00z_aggregators --release --test test_hjmt_split_brain_fencing -- --nocapture`
  passed.
- `cargo test -p z00z_simulator --release --test test_scenario_settlement -- --nocapture`
  passed.
- `cargo test --release` passed on the final tree.
- `git diff --check -- .planning/phases/060-Gaps-Closing/060-05-SUMMARY.md .planning/STATE.md .planning/ROADMAP.md crates/z00z_runtime/aggregators/tests/support/test_hjmt_topology_support.rs crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs crates/z00z_runtime/aggregators/tests/test_hjmt_split_brain_fencing.rs crates/z00z_simulator/src/config.rs crates/z00z_simulator/src/scenario_1/runtime_observability.rs crates/z00z_simulator/src/scenario_1/scenario_config.yaml crates/z00z_simulator/src/scenario_1/scenario_design.yaml crates/z00z_simulator/tests/test_hjmt_runtime_config.rs crates/z00z_simulator/tests/test_scenario_settlement.rs`
  is clean for this slice.
- `rg -n "SIM-3A7S-2A7S-5A7S|transition_stages|removed_aggregator_absent|publication_continuity_preserved|staged_fail_down_and_reexpand|decommissioned_aggregator_cannot_reenter|decommissioned_topology_keeps" crates/z00z_runtime/aggregators crates/z00z_simulator`
  confirms the staged packet and guards are wired into the live code and test
  homes.
- `rg -n "B3|B4|3A7S|2A7S|5A7S|decommission|same-lineage|split-brain|transition_stages" .planning/phases/060-Gaps-Closing/060-TODO.md crates/z00z_runtime/aggregators/tests crates/z00z_simulator/src/scenario_1 crates/z00z_simulator/tests/test_scenario_settlement.rs`
  confirms the implementation remains anchored to the Phase 060 authority
  packet.

## Result

`060-05` is complete. Phase 060 advances to `060-06-PLAN.md` for the
supply-chain review records and vet-trust closure slice.
