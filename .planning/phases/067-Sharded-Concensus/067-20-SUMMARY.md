---
phase: 067
plan: 067-20
status: complete
completed_at: 2026-07-06
next_plan: 067-21
summary_artifact_for: .planning/phases/067-Sharded-Concensus/067-20-PLAN.md
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 067-20 Summary: Certificate-Bound Publication Resume And Recovered Primary Rejoin Addendum

## Outcome

`067-20` is complete.

`ADDENDUM-067-20` now closes on one canonical certificate-bound recovery path.
Post-quorum pre-DA crash recovery is bound to the exact persisted subject,
certificate, publication binding, and theorem evidence, and the recovered old
primary can no longer auto-failback after lawful takeover or planned rotation.

The closeout also removes the remaining alias drift in this slice. `scenario_11`
now carries a dedicated `old_primary_restart_after_takeover` fault row instead
of reusing the broader stale-membership restart case, and the runtime rejection
path is backed by explicit failover tests. `067-21` is now the next canonical
execution lane, while the `067-19` final rerun remains pending after
`067-21`.

## Files Changed

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/067-Sharded-Concensus/067-20-SUMMARY.md`
- `.planning/phases/067-Sharded-Concensus/067-21-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-FINAL-CONFORMANCE.md`
- `crates/z00z_runtime/aggregators/tests/test_recovery_failover.rs`
- `crates/z00z_simulator/src/scenario_11/mod.rs`
- `crates/z00z_simulator/tests/test_scenario_11.rs`

## Landed Changes

- Exact anti-failback runtime proof
  - `test_recovery_failover.rs` now includes
    `test_old_primary_failback_rejects` and
    `test_rotated_primary_reentry_rejects`.
  - These tests prove `RecoveryBoundary::resume(...)` rejects `RestartPrimary`
    after lawful secondary takeover or planned role rotation.
- Dedicated scenario evidence row
  - `scenario_11` now emits the explicit fault id
    `old_primary_restart_after_takeover`.
  - The row resolves to `rejected_as_expected` and no longer shares the
    broader `restart_reconnect_old_membership` alias.
- Canonical flow-alias cleanup
  - `067-FINAL-CONFORMANCE.md` and `067-21-PLAN.md` now point the
    `old_primary_restart_after_takeover` flow directly to the runtime failover
    tests plus the dedicated `scenario_11` row.
  - The stale-membership restart row remains a separate negative case instead
    of pretending to prove old-primary anti-failback.
- Canonical status sync
  - Phase `067` state and roadmap artifacts now record `067-20` as
    summary-backed complete, move the active lane to `067-21`, and keep the
    `067-19` final rerun explicitly open after the addendum packet.

## Validation

Commands green during the `067-20` closeout cycle:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_consensus_store -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_consensus_recovery_restart -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_failover_same_lineage -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_recovery_failover -- --nocapture`
- `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`
- `python3 scripts/audit/audit_067_claims.py`
- `cargo test --release`

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times in
YOLO mode, but the current runner again did not provide a usable automated
review path for this slice.

- Attempt 1
  - `timeout 90s gsd --no-session --extension .github --print '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-20-PLAN.md current_task="067-20-T1" --yolo'`
  - Result: exited with code `1` and returned `402 Prompt tokens limit exceeded: 85031 > 38936`.
- Attempt 2
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-20-PLAN.md current_task="067-20-T1" --yolo'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`.
- Attempt 3
  - `timeout 90s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-20-PLAN.md current_task="Certificate-Bound Publication Resume And Recovered Primary Rejoin Addendum" --yolo'`
  - Result: exited with code `1` and returned `402 Prompt tokens limit exceeded: 67847 > 38936`.

Equivalent workspace-first manual review was executed with the `/doublecheck`
posture against the same scope.

- Pass 1
  - Re-read `067-20-PLAN.md`, `test_recovery_failover.rs`, and the touched
    `scenario_11` surfaces against the addendum acceptance gates.
  - Result: found one material packet-to-evidence gap. The old-primary
    anti-failback claim still shared the stale-membership restart alias instead
    of having a dedicated scenario row. Fixed with an explicit
    `old_primary_restart_after_takeover` row plus matching tests.
- Pass 2
  - Re-checked the naming split between
    `old_primary_restart_after_takeover` and
    `restart_reconnect_old_membership` across runtime tests, `scenario_11`,
    and the final-packet docs.
  - Result: clean. The anti-failback proof path and the stale-membership proof
    path are now separate and explicit.
- Pass 3
  - Re-ran the targeted release tests and `python3 scripts/audit/audit_067_claims.py`
    after the fix.
  - Result: clean. The addendum stays aligned with the claim-audit contract.
- Pass 4
  - Ran `git diff --check` after the final phase-status edits.
  - Result: clean.
- Pass 5
  - Re-read `067-20-SUMMARY.md`, `.planning/STATE.md`, and `.planning/ROADMAP.md`
    after the final status sync.
  - Result: clean.

Passes 4 and 5 were consecutive clean manual review runs after the final
closeout sync.

## Closeout

`067-20` closes `ADDENDUM-067-20` by making certificate-bound publication
resume and recovered-primary anti-failback executable on one canonical local
path: persisted evidence, runtime recovery admission, dedicated simulator
fault evidence, and final flow-alias docs now agree.

`067-21` is now the next canonical execution lane. The `067-19` final rerun
remains open until the packet-closure addendum is complete.
