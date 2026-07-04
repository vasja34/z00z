---
phase: 067
plan: 067-05
status: complete
completed_at: 2026-07-04
next_plan: 067-06
summary_artifact_for: .planning/phases/067-Sharded-Concensus/067-05-PLAN.md
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 067-05 Summary: Scenario 11 End To End Harness

## Outcome

`067-05` is complete.

`PHASE-4` now closes on one independent `scenario_11` owner outside
`scenario_1`. The landed harness proves that one wallet-style package can flow
through live ingress normalization, route selection, placement ownership,
secondary replay, local `2-of-3` quorum certificate formation, local DA
publication, and validator verdict while keeping one canonical subject digest
bound across every step.

The closeout also hardens the required failure matrix beyond the initial happy
path. The live scenario now proves dual-primary per-shard isolation, all-shard
sweep coverage, one-secondary-offline behavior, stale-secondary rejection,
primary-offline-before-dispatch defer behavior, pre-quorum primary crash
non-publication, and post-quorum pre-DA crash resume from the exact same
certificate rather than a recomputed publication path.

## Files Changed

- `.planning/phases/067-Sharded-Concensus/067-05-SUMMARY.md`
- `.planning/phases/067-Sharded-Concensus/067-05-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-06-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-08-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-09-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-COVERAGE.md`
- `.planning/phases/067-Sharded-Concensus/067-PLAN-REVIEW.md`
- `.planning/phases/067-Sharded-Concensus/067-TESTS-TASKS.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `crates/z00z_runtime/aggregators/src/types.rs`
- `crates/z00z_rollup_node/Cargo.toml`
- `crates/z00z_rollup_node/src/da.rs`
- `crates/z00z_rollup_node/tests/support/test_theorem_fixture.rs`
- `crates/z00z_rollup_node/tests/test_da_local_sim.rs`
- `crates/z00z_simulator/Cargo.toml`
- `crates/z00z_simulator/src/lib.rs`
- `crates/z00z_simulator/src/scenario_11/mod.rs`
- `crates/z00z_simulator/src/scenario_11/report.rs`
- `crates/z00z_simulator/tests/test_scenario_11.rs`
- `crates/z00z_wallets/docs/egui_views.tar.gz`
- `crates/z00z_wallets/src/lib.rs`
- `crates/z00z_wallets/src/redb_store/mod.rs`

## Landed Changes

- Independent `scenario_11` owner
  - Added the dedicated `scenario_11` module, report schema, and explicit
    integration-test target outside `scenario_1`.
  - The simulator crate now exports `scenario_11` directly instead of
    piggybacking on a `scenario_1` stage or observability tail.
- End-to-end subject and route binding
  - `scenario_11` emits package ingress, route, placement, commit subject,
    replay vote, quorum certificate, local DA, validator verdict, fault
    matrix, and report-honesty artifacts from one live run.
  - `PublicationRequest` now carries the live `publication_route`, and the
    local DA adapter preserves that route snapshot through publication and
    resolution so the activation checkpoint and publication artifact stay bound
    to the same route authority.
- RAID-like continuity and fail-closed matrix
  - Added explicit evidence for `primary_offline_before_dispatch`, proving
    dispatch is deferred on the lawful owner path rather than silently
    rerouted.
  - The fault matrix also proves stale-secondary rejection,
    primary-crash-before-quorum no-publication behavior, and exact-certificate
    resume after a post-quorum pre-DA crash.
  - The dual-primary path proves that one process owning two shards does not
    merge shard-local quorums into a global committee.
- No parallel implementation layer
  - No new HJMT, crypto, or utility stack was introduced.
  - The slice reuses existing `z00z_core`, `z00z_crypto`, `z00z_storage`,
    `z00z_utils`, `z00z_aggregators`, `z00z_rollup_node`, and
    `z00z_validators` primitives only.
  - No new external crate was added for the `067-05` semantic slice; the only
    added dependency edge was the internal workspace `z00z_validators` link in
    the simulator.
- Broad release-gate unblockers
  - Renamed the simulator integration test home to
    `crates/z00z_simulator/tests/test_scenario_11.rs` and pinned it under the
    stable `scenario_11` test target to satisfy rename guards without changing
    the test invocation surface.
  - Restored `crates/z00z_wallets/docs/egui_views.tar.gz` to its canonical
    expected path so the existing wallet path-contract suite could pass again.
  - Kept release-only debug-gate behavior truthful by allowing the test-only
    `wallet_debug_tools` path only when paired with `test-params-fast`.

## Validation

Commands green during the `067-05` closeout cycle:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_wallets --test test_rename_guards -- --nocapture`
- `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_dispatch -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_shard_routing -- --nocapture`
- `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_da_local_sim -- --nocapture`
- `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_topology -- --nocapture`
- `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_preflight -- --nocapture`
- `cargo test --release -p z00z_validators --test test_hjmt_publication_contract -- --nocapture`
- `bash scripts/audit/audit_release_feature_guards.sh`
- `cargo test --release`

Additional release-validation note:

- The first broad workspace rerun in this closeout cycle exposed only two
  canonical-path drifts outside the core quorum logic: the simulator
  integration test home still used the pre-rename file naming shape that
  violated the rename-guard contract, and
  `crates/z00z_wallets/docs/egui_views.tar.gz` was absent from its canonical
  expected path.
- After renaming the simulator test home and restoring the wallet doc artifact,
  the isolated `test_rename_guards` suite turned green and the subsequent broad
  `cargo test --release` rerun completed without observed failures.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times in
YOLO mode, but the current runner again did not provide a usable automated
review path for this slice.

- Attempt 1
  - `timeout 90s gsd --bare --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-05-PLAN.md current_task="067-05-T1" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 83737 > 38936`
- Attempt 2
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-05-PLAN.md current_task="067-05-T1"'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 3
  - `timeout 90s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-05-PLAN.md current_task="Scenario 11 End To End Harness" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66677 > 38936`

Equivalent workspace-first manual review was executed against the same scope.

- Pass 1
  - Re-read `067-TODO.md` `13.2`, `14.5`, and `19`, then re-read
    `067-CONTEXT.md` and the live `scenario_11` harness.
  - Result: found one real closeout gap: the fault matrix did not yet carry
    explicit `primary_offline_before_dispatch` evidence. Added the live
    `DistDispatch::partition(...)`-backed defer path and a matching test
    assertion.
- Pass 2
  - Re-read the `publication_route` seam in
    `crates/z00z_runtime/aggregators/src/types.rs`,
    `crates/z00z_rollup_node/src/da.rs`,
    `crates/z00z_rollup_node/tests/support/test_theorem_fixture.rs`, and
    `crates/z00z_rollup_node/tests/test_da_local_sim.rs`.
  - Result: found and fixed one closeout issue: the local DA path still needed
    to preserve the request-carried route snapshot end to end so publication
    stayed bound to the same route authority as the planner and activation
    checkpoint.
- Pass 3
  - Re-read the full Phase 067 packet for stale source references.
  - Result: found packet drift from `069-New-Scenarios` to the live
    `090-New-Scenarios` source home and normalized the entire `067` packet.
- Pass 4
  - Ran `rg -n "069-New-Scenarios" .planning/phases/067-Sharded-Concensus .planning/STATE.md .planning/ROADMAP.md`.
  - Result: clean. No stale `069-New-Scenarios` references remained.
- Pass 5
  - Ran seam-level grep over the live end-to-end paths for
    `primary_offline_before_dispatch`, `resumed_same_certificate`, and
    `publication_route`.
  - Result: clean. The required live seams were present and no parallel
    HJMT/crypto/util path appeared.
- Pass 6
  - Ran a packet-wide grep for stale simulator-path strings and non-release
    cargo command forms.
  - Result: clean. The packet now uses the canonical
    `crates/z00z_simulator/tests/test_scenario_11.rs` path and release-only
    cargo commands.
- Pass 7
  - Re-read `067-05-SUMMARY.md`, `067-05-PLAN.md`, `067-COVERAGE.md`,
    `.planning/STATE.md`, and `.planning/ROADMAP.md` after the closeout sync.
  - Result: clean. `067-05` is summary-backed complete, `067-06` is the next
    active lane, and the packet keeps one canonical file, test, and status
    path.

Passes 6 and 7 were consecutive clean manual review runs after the final
closeout-doc sync.

## Closeout

`067-05` closes `PHASE-4` by giving the repository one canonical independent
local package-to-validator harness, one truthful local fault matrix, one
route-bound publication path, and one no-parallel-layer proof story for this
slice.

`067-06` is now the next canonical execution lane.
