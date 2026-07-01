---
phase: 062-Gaps-Closing-2
plan: 062-09
status: complete
completed_at: 2026-06-25
next_plan: 062-10
summary_artifact_for: .planning/phases/062-Gaps-Closing-2/062-09-PLAN.md
---

# 062-09 Summary: Simulator Wallet Evidence Pack

## Outcome

`062-09` is complete. The grouped plan contract `PLAN-062-G09` now closes
through the renamed `062-09-PLAN.md` packet with one simulator evidence path
that stays downstream of the live wallet, storage, and publication primitives.

`scenario_1` runtime observability now records `wallet_scan_digest_hex` and a
`wallet_lifecycle_rows` matrix inside `hist_flow.json`. The matrix covers the
full required receive/import/history closure set: `imported`, `submitted`,
`admitted`, `confirmed`, `duplicate_import`, `conflicted`,
`already_spent`, `no_owned_output`, `wrong_chain`, `invalid_digest`, and
`unsupported_package_version`. Each row carries lifecycle, coarse status,
typed error code when relevant, wallet-scan digest binding, tx-history digest,
publication digest, and a restart proof bit.

The simulator wallet evidence is now path-invariant. Lifecycle simulation no
longer derives deterministic seeds from absolute output paths, so the shared
Stage 13 cache root and localized Stage 13 root produce the same tx ids and
tx-history digests for the same wallet-scan and publication inputs. Promoted
shared Stage 13 roots are also revalidated and rebuilt on drift instead of
trusting stale stable markers, which prevents cached evidence from surviving a
contract change incorrectly.

Phase-local closeout docs are now explicit. `Z00Z-IMPL-PHASES.md` records the
wallet receive/import/history authority closure in section `10` and the
simulator receive/import/history evidence table in section `21`, tying the
evidence back to the code-owned receive lane, tx-history authority, request
helper boundary, and simulator proof packet without creating a second truth
plane.

## Files Changed

- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13/shared_cases.rs`
- `.planning/phases/Z00Z-IMPL-PHASES.md`
- `.planning/phases/062-Gaps-Closing-2/062-09-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_simulator wallet_lifecycle_rows_ -- --nocapture`
- `cargo test --release -p z00z_simulator --test scenario_1 test_hjmt_e2e:: -- --nocapture`
- `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_stage_surface::test_scenario1_stage_surface -- --exact`
- `cargo test --release -p z00z_wallets --test test_rpc_logging_risk_policy`
- `cargo test --release -p z00z_wallets --test test_direct_tx_receive`
- `cargo test --release`
- `rg -n "## 10\\.|## 21\\.|wallet_lifecycle_rows|wallet_scan_digest_hex|restart_verification_passed|DuplicateConflict|AlreadySpent" .planning/phases/Z00Z-IMPL-PHASES.md crates/z00z_simulator/src/scenario_1/runtime_observability.rs crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs`
- `git diff --check -- .planning/phases/062-Gaps-Closing-2/062-09-SUMMARY.md .planning/phases/Z00Z-IMPL-PHASES.md .planning/STATE.md .planning/ROADMAP.md crates/z00z_simulator/src/scenario_1/runtime_observability.rs crates/z00z_simulator/src/scenario_1/stage_13/shared_cases.rs`
- Result: green

## Manual Review Passes

Because `/GSD-Review-Tasks-Execution` is not callable as a tool here, the
required review loop was executed manually against the same scope.

- Pass 1
  - Read `062-09-PLAN.md`, `GAPS.md` rows for `TASK-032`, `TASK-033`,
    `TASK-034`, and `TASK-039`, plus the diffs in
    `runtime_observability.rs` and `shared_cases.rs`.
  - Result: clean. The simulator remains evidence-only, wallet lifecycle truth
    still comes from live wallet and tx-history primitives, and the cache
    refresh path does not add a second authority plane.
- Pass 2
  - `cargo test --release -p z00z_simulator wallet_lifecycle_rows_ -- --nocapture`
  - `cargo test --release -p z00z_simulator --test scenario_1 test_hjmt_e2e:: -- --nocapture`
  - `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_stage_surface::test_scenario1_stage_surface -- --exact`
  - `cargo test --release -p z00z_wallets --test test_rpc_logging_risk_policy`
  - `cargo test --release -p z00z_wallets --test test_direct_tx_receive`
  - Result: clean
- Pass 3
  - `cargo test --release`
  - `rg -n "## 10\\.|## 21\\.|wallet_lifecycle_rows|wallet_scan_digest_hex|restart_verification_passed|DuplicateConflict|AlreadySpent" .planning/phases/Z00Z-IMPL-PHASES.md crates/z00z_simulator/src/scenario_1/runtime_observability.rs crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs`
  - `git diff --check -- .planning/phases/062-Gaps-Closing-2/062-09-SUMMARY.md .planning/phases/Z00Z-IMPL-PHASES.md .planning/STATE.md .planning/ROADMAP.md crates/z00z_simulator/src/scenario_1/runtime_observability.rs crates/z00z_simulator/src/scenario_1/stage_13/shared_cases.rs`
  - Result: clean

Passes 2 and 3 were consecutive clean runs.

## Task Closeout

- `TASK-032`
  - Closed by the live `hist_flow.json` wallet-lifecycle matrix, which now
    records the required success, duplicate, conflict, restart, and negative
    package cases on top of real wallet import, reconcile, asset, and
    tx-history primitives.
- `TASK-033`
  - Closed by restart verification that reopens wallet state, replays
    tx-history JSONL, re-reads owned assets, and proves pre-restart and
    post-restart lifecycle state equality for every required simulator case.
- `TASK-034`
  - Closed by the new section `10` closeout note in
    `Z00Z-IMPL-PHASES.md`, which explicitly lists the lifecycle projection
    type, receive status outcome type, advisory worker test anchors,
    asset/object authority boundaries, request-bound helper boundary, and the
    simulator evidence handoff.
- `TASK-039`
  - Closed by the new section `21` evidence table in
    `Z00Z-IMPL-PHASES.md`, which maps each simulator row to its expected
    lifecycle, coarse status, and typed error code, plus the live digest
    bindings carried in `wallet_lifecycle_rows`.
