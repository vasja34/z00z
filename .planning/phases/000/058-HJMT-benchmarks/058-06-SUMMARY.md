---
phase: 058-HJMT-benchmarks
plan: 058-06
status: complete
completed_at: 2026-06-15
next_plan: 058-07
requirements-completed:
  - 058-G11
summary_artifact_for: .planning/phases/058-HJMT-benchmarks/058-06-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 058-06 Summary: Dynamic Scope, Wallet Closure, Historical Playback, And Occupancy Privacy

## Completed Scope

`058-06` is complete for the dynamic-scope, wallet-closure, historical-playback,
and occupancy-privacy slice.

This slice did not create a second runtime lane, a second publication lane, or
a parallel proof packet. Instead it extended the existing release-mode simulator
observability contract with one canonical public release packet that now emits
`run_meta.json`, `wallet_scan.json`, `hist_flow.json`, `occ_flow.json`, and
`sim_summary.md`, while keeping `asset_flow.json` and `right_flow.json` honest
as `pending_exact_home` inventory rows instead of pretending that those exact
homes already exist.

The runtime config and packet vocabulary now match the live Phase 058 scope.
`SIM-BATCH-1000` remains live but heavy-only through `supported_profiles` plus
`heavy_only_profiles`, and the packet contract now requires explicit
emitted-versus-pending public inventory. `scope_flow.json` now carries
first-seen `definition_id` or `serial_id` or object-birth evidence, mixed
terminal/right creation rows, restart or failover owner homes, explicit wallet
proof-before-ownership promotions, negative wallet-scan summaries, and the
shared proof boundary that keeps final wallet promotion downstream of proof
validation instead of downstream of ad hoc tx-package guesses.

The release packet also closes the imported-artifact readiness story. Wallet
promotion rows are now bound to committed Stage 7 candidate outputs, and
`run_meta.json` plus `wallet_scan.json` keep Charlie's visible state aligned
with the shared Stage 7 checkpoint root. `hist_flow.json` and `occ_flow.json`
now replay imported Phase 057 publication artifacts on the same lineage,
record route migration plus old/new public and settlement roots, carry
historical-proof verdicts, occupancy-disclosure verdicts, and
imported-artifact validation verdicts, and fail closed if the live reject rows
disappear from the packet.

The stage-surface and settlement suites now prove the packet end to end:
heavy-only profile acceptance, planner/process packet consistency, public
artifact inventory, wallet proof-before-ownership closure, imported historical
replay, occupancy privacy guards, and tamper rejection for route-migration or
disclosure-guard drift. The shared Stage 13 cache root was also restabilized so
the new public packet files stay on the same canonical path without rebasing
`run_meta.json` or `wallet_scan.json` into a second authority lane. The same
proof-boundary slice now carries exact live acceptance homes for
`test_hjmt_transition_proofs.rs`, `test_hjmt_privacy_regression.rs`, and
`test_hjmt_e2e.rs` instead of scattering those closures across unrelated
packets.

## Files Changed

- `.planning/phases/058-HJMT-benchmarks/058-06-SUMMARY.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `crates/z00z_simulator/src/config.rs`
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
- `crates/z00z_simulator/src/test_support/stage13_shared_cases.rs`
- `crates/z00z_simulator/tests/test_hjmt_e2e.rs`
- `crates/z00z_simulator/tests/test_hjmt_runtime_config.rs`
- `crates/z00z_simulator/tests/test_scenario_settlement.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- `crates/z00z_storage/tests/test_hjmt_privacy_regression.rs`
- `crates/z00z_storage/tests/test_hjmt_transition_proofs.rs`

## Boundary Kept Intact

- Phase 058 still extends the inherited Phase 056 runtime and Phase 057
  publication evidence in place; it did not introduce a parallel runtime,
  publication, or proof authority path.
- Historical and occupancy closure stay bound to imported `leaf_flow.json`,
  `proof_flow.json`, `pub_flow.json`, `val_flow.json`, and `watch_flow.json`
  artifacts. No synthetic-root or current-config reinterpretation lane was
  introduced.
- `asset_flow.json` and `right_flow.json` remain explicit
  `pending_exact_home` rows. This slice did not overclaim those exact homes as
  landed.
- `test_hjmt_transition_proofs.rs`, `test_hjmt_privacy_regression.rs`, and
  `test_hjmt_e2e.rs` now stay on the same proof-boundary slice as exact live
  homes. No fake placeholder test suite or detached acceptance packet was
  added.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md`
was used because the slash prompt is not a callable tool in this environment.

- Pass 1 found two material issues before final validation: the new
  `sim_summary.md` headings introduced unnecessary non-ASCII output into the
  public packet, and `hist_flow.json` or `occ_flow.json` could still validate
  with missing live reject rows even though `058-TODO.md` requires explicit
  negative imported-flow coverage. Both issues were fixed.
- Pass 1 also used a workspace-first `/doublecheck` pass on those review
  findings: `rg` confirmed the touched packet files no longer contain
  non-ASCII headings, and the final source plus tests confirm that missing
  live reject rows now fail closed and that `wrong_root_generation` plus
  `stale_policy_transition_id` stay anchored in the release packet.
- Pass 2 re-audited `058-TODO.md`, `058-CONTEXT.md`, `058-06-PLAN.md`,
  `runtime_observability.rs`, `stage13_shared_cases.rs`, and the updated
  simulator tests against the final packet claims. No significant issues
  remained.
- Pass 3 repeated the same audit after the final validation wave and the
  `STATE.md` or `ROADMAP.md` closeout edits. No significant issues remained.

Two consecutive clean review passes were achieved on passes 2 and 3.

## Validation

Validation is green for this slice.

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  on the final code tree.
- Final-tree release validation passed for:
  `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_scope_birth -- --nocapture`,
  `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_historical_proofs -- --nocapture`,
  `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_adaptive_policy_proofs -- --nocapture`,
  `cargo test -p z00z_storage --release --features test-params-fast --test test_occupancy_privacy -- --nocapture`,
  `cargo test -p z00z_storage --release --features test-params-fast --test test_occupancy_evidence -- --nocapture`,
  `cargo test -p z00z_simulator --release --features test-params-fast --test test_hjmt_runtime_config -- --nocapture`,
  `cargo test -p z00z_simulator --release --features test-params-fast --test test_stage7_jmt_wallet_scan -- --nocapture`,
  `cargo test -p z00z_simulator --release --features test-params-fast --test test_scenario_settlement -- --nocapture`,
  and
  `cargo test -p z00z_simulator --release --features test-params-fast --test test_scenario1_stage_surface -- --nocapture`.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_transition_proofs -- --nocapture`
  passed during the 2026-06-15 phase-validation refresh that reconciled the
  slice summary with the exact proof-boundary homes.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_privacy_regression -- --nocapture`
  passed during the same validation refresh.
- `cargo test -p z00z_simulator --release --features test-params-fast --test test_hjmt_e2e -- --nocapture`
  passed during the same validation refresh.
- `cargo test --release` passed for the workspace earlier in this slice before
  the final low-risk simulator follow-up; the final tree was then revalidated
  with the full `058-06` targeted release packet above.
- `cargo doc --no-deps` passed. It emitted only pre-existing rustdoc warnings,
  including unresolved intra-doc links or invalid markup in untouched docs
  under `z00z_crypto`, `z00z_core`, `z00z_wallets`, and a pre-existing link
  warning in `z00z_simulator/src/lib.rs`.
- `git diff --check` is clean.

## Result

`058-06` is complete. Phase 058 advances to `058-07-PLAN.md` for fixture-family
closure, final verdict, and planning-state synchronization.

This summary does not claim the final fixture matrix, the final evidence-ledger
closeout, or the final repository readiness verdict; those remain owned by
`058-07`.
