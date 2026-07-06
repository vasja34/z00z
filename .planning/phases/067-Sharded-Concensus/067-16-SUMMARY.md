---
phase: 067
plan: 067-16
status: complete
completed_at: 2026-07-06
next_plan: 067-17
summary_artifact_for: .planning/phases/067-Sharded-Concensus/067-16-PLAN.md
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 067-16 Summary: Celestia Local Artifact Conformance

## Outcome

`067-16` is complete.

`VERDICT-LCS-07` now closes on one executable Celestia-local artifact path.
The local adapter stores and verifies raw blob bytes, namespace, blob
commitment, inclusion reference, retention horizon, blob height, and degraded
state on the same validator-facing artifact seam that already binds QC,
theorem digest, publication digest, and checkpoint-bearing validator
decisions.

The closeout also keeps the claim boundary explicit. `067-16` proves a local
simulated-full Celestia artifact contract only; it does not claim real
Celestia provider integration or real external finality. Those remain later
work, and `067-17` is now the next canonical execution lane.

## Files Changed

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/067-Sharded-Concensus/067-16-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-16-SUMMARY.md`
- `.planning/phases/067-Sharded-Concensus/067-COVERAGE.md`
- `.planning/phases/067-Sharded-Concensus/067-verdict.md`
- `crates/z00z_rollup_node/src/celestia_local.rs`
- `crates/z00z_rollup_node/src/da.rs`
- `crates/z00z_rollup_node/tests/test_celestia_local_binding.rs`
- `crates/z00z_simulator/src/scenario_11/mod.rs`
- `crates/z00z_simulator/src/scenario_11/report.rs`
- `crates/z00z_simulator/tests/test_scenario_11.rs`

## Landed Changes

- Canonical Celestia-local artifact path
  - `CelestiaLocalRecord` now carries raw blob bytes, inclusion reference,
    retention horizon, and degraded-mode metadata in addition to namespace,
    commitment, certificate digest, and checkpoint-bound bindings.
  - The local adapter now exposes real `retrieve_blob(...)` and
    `verify_blob(...)` paths over one runtime-owned record shape instead of a
    provider-name-only placeholder.
- Fail-closed artifact verification
  - `DaError` now distinguishes blob-bytes drift, inclusion-reference drift,
    and retention expiry, and the local adapter rejects mismatched record
    material instead of silently accepting detached payloads.
  - Targeted release tests now cover blob-bytes mismatch,
    inclusion-reference mismatch, degraded-mode behavior before the
    unanchored limit, and retention expiry on retrieval.
- Honest simulator claim surface
  - `scenario_11` now emits one explicit Celestia-local artifact contract
    report with `simulated-full` claim level plus namespace, commitment,
    inclusion-reference, height, retention, degraded-state, and payload
    availability fields.
  - The scenario honesty report now keeps real Celestia finality on the
    explicit `live-claim-removed` path while marking the local artifact
    contract as executable.
- Canonical planning/status sync
  - Phase `067` coverage, verdict, state, and roadmap artifacts now record
    `067-16` as complete and move the active lane to `067-17`.

## Validation

Commands green during the `067-16` closeout cycle:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_celestia_local_binding -- --nocapture`
- `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_da_local_sim -- --nocapture`
- `cargo test --release -p z00z_validators --test test_hjmt_publication_contract -- --nocapture`
- `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`
- `cargo test --release`
- `bash scripts/audit/audit_release_feature_guards.sh`

Broad release-gate note:

- The first current-cycle `cargo test --release` rerun exposed the existing
  wallet hardening guard expectation from
  `crates/z00z_wallets/tests/test_production_hardening.rs`; restoring the
  grouped crate-private `redb_store` debug-export re-export shape removed that
  drift before the final green broad rerun.
- `bash scripts/audit/audit_release_feature_guards.sh` reran green on the
  same final tree.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times in
YOLO mode, but the current runner again did not provide a usable automated
review path for this slice.

- Attempt 1
  - `timeout 90s gsd --no-session --extension .github --print '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-16-PLAN.md current_task="067-16-T1" --yolo'`
  - Result: exited with code `1` and produced no stdout or stderr.
- Attempt 2
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-16-PLAN.md current_task="067-16-T1" --yolo'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`.
- Attempt 3
  - `timeout 90s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-16-PLAN.md current_task="Celestia Local Artifact Conformance" --yolo'`
  - Result: exited with code `1` and produced no stdout or stderr.

Equivalent workspace-first manual review was executed with the
`/code-reviewer` checklist and `/doublecheck` three-layer posture against the
same scope.

- Pass 1
  - Re-ran anchored grep for the canonical Celestia-local artifact markers
    `CelestiaLocalAdapter`, `CelestiaLocalRecord`, `retrieve_blob`,
    `verify_blob`, `blob_bytes`, `inclusion_reference`,
    `retention_until_height`, and `degraded_mode` across the touched rollup,
    simulator, and test surfaces.
  - Result: clean. One canonical Celestia-local adapter and one canonical
    record shape remained under the reviewed scope.
- Pass 2
  - Re-ran anchored grep for the honesty markers `Celestia finality`,
    `Celestia-local artifact contract`, `simulated-full`,
    `live-claim-removed`, and `payload_available` across the touched
    simulator, report, and planning surfaces.
  - Result: clean. The code, tests, report JSON, and plan packet agree on one
    executable local artifact claim and one explicit real-finality non-claim.
- Pass 3
  - Ran `git diff --check` across the touched `067-16` code and planning
    artifacts after the final status sync.
  - Result: clean.
- Pass 4
  - Re-read `067-16-PLAN.md`, `067-16-SUMMARY.md`, `067-COVERAGE.md`,
    `067-verdict.md`, `.planning/STATE.md`, and `.planning/ROADMAP.md` after
    the final status sync.
  - Result: clean.

Passes 3 and 4 were consecutive clean manual review runs after the final
closeout sync.

## Closeout

`067-16` closes `VERDICT-LCS-07` by making the Celestia-local term an
executable artifact contract instead of a provider-shaped placeholder, while
keeping real external finality explicitly unclaimed.

`067-17` is now the next canonical execution lane.
