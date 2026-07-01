---
phase: 031-refactor-architecture
plan: "09"
requirements_completed:
  - PH31-SIM
status: completed
task_commits:
  - 5dce4c30
  - fc17ab8e
review_surface_metrics:
  stage2_secret_contract_guards: 3
  sandbox_negative_guards: 1
  architecture_boundary_guards: 2
---

# Phase 031 Plan 09: Simulator Boundary And Secret Contract Summary

The simulator now stays on stable facades, keeps plaintext wallet-secret output off the default Stage 2 contract, and fail-closes recursive output cleanup outside the approved sandbox roots.

## Accomplishments

- Hardened [crates/z00z_simulator/src/scenario_1/stage_2.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_2.rs), [crates/z00z_simulator/src/scenario_1/stage_2_utils/artifacts.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_2_utils/artifacts.rs), and [crates/z00z_simulator/src/scenario_1/stage_2_utils/checks.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_2_utils/checks.rs) so the default Stage 2 lane no longer publishes `wlt_secrets_debug.md`, while the retained debug-only secret artifact is feature-gated, written under `wallets/private/`, and persisted with private-permission file writes.
- Updated [crates/z00z_simulator/tests/test_wallet_integration.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_wallet_integration.rs) and [crates/z00z_simulator/tests/test_scenario1_stage_surface.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_scenario1_stage_surface.rs) so the public simulator contract now proves the absence of plaintext secret artifacts by default, while the debug lane proves the retained private artifact exists only behind `wallet_debug_dump` and stays mode `0600`.
- Documented the simulator admission policy and scenario contract in [crates/z00z_simulator/README.md](/home/vadim/Projects/z00z/crates/z00z_simulator/README.md) so `z00z_simulator` is explicitly the integration harness instead of an accidental second owner of wallet, storage, or crypto business rules.
- Hardened [crates/z00z_simulator/src/scenario_1/runner.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/runner.rs) so `reset_outputs_dir()` validates paths against approved simulator sandbox roots before any recursive delete or directory creation occurs.
- Added [crates/z00z_simulator/tests/test_architecture_boundaries.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_architecture_boundaries.rs) as a persistent regression guard so deep wallet-service, wallet-db, and storage-internal imports cannot quietly return after this cleanup wave.

## Task Commits

| Task | Commit | Purpose |
| --- | --- | --- |
| Task 1 | `5dce4c30` | Remove default plaintext Stage 2 secret artifacts from the public contract, keep the debug artifact private and gated, and align the regression tests with the new contract. |
| Task 2 | `fc17ab8e` | Document the simulator harness boundary, sandbox `reset_outputs_dir()`, and add the persistent architecture-boundary guard. |

## Decisions Made

- The default public simulator contract does not include plaintext wallet-secret artifacts; any retained debugging surface must remain feature-gated, private-path-only, and private-permission-only.
- `reset_outputs_dir()` is a boundary enforcement seam, not a permissive helper, so sandbox validation must happen before delete-or-create side effects.
- The simulator boundary note belongs in the crate README and must be backed by a persistent failing architecture test instead of a one-off review instruction.

## Verification

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_simulator --release --features test-fast --test test_wallet_integration stage2_rpc_no_secrets -- --exact --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --test test_scenario1_stage_surface -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --test test_architecture_boundaries -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --test test_wallet_integration reset_outputs_dir_rejects_outside_sandbox -- --exact --nocapture`
- `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_wallet_integration stage2_debug_secret_artifact_is_private -- --exact --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump`

## Deviations From Plan

### Auto-fixed Issues

1. [Rule 1 - Bug] `stage2_rpc_no_secrets()` still treated the retained private debug artifact as forbidden in every feature lane, which caused a false release-style failure after the production Stage 2 contract had already moved the artifact behind `wallet_debug_dump`.
   Fix: updated the test to require the private debug artifact only when `wallet_debug_dump` is enabled, while keeping the legacy public path forbidden in all lanes.

2. [Rule 3 - Tooling substitution] The plan asked for repeated `/GSD-Review-Tasks-Execution` prompt runs, but that review prompt runner was not available in this executor session.
   Fix: replaced it with the exact release-style simulator contract checks, the persistent architecture-boundary test, the sandbox negative proof, the release-style scenario binary run, and the broader debug-lane simulator suite.

## Deferred Issues

- None from the simulator scope closed by this plan.

## Known Stubs

- None added by this plan.

## Threat Flags

- None.

## Self-Check

- PASSED
- Found summary artifact: `031-09-SUMMARY.md`
- Verified task commits: `5dce4c30`, `fc17ab8e`
- Verified bootstrap, targeted simulator contract checks, sandbox rejection proof, architecture-boundary guard, debug-lane scenario run, and the full debug-lane simulator release suite.
