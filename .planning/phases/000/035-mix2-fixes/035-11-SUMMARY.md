# 035-11 Summary

## Scope

This summary records the completion state for `035-11-PLAN.md`, covering task
`035-25 Legacy Builder Adapter Convergence`, task
`035-26 Replayable Bundle Adapter Convergence`, and task
`035-27 Stealth Export And Unit Coverage`.

## Outcome

Plan 11 is fully closed.

Phase 035 now has the second live sender-workflow slice closed on the
repository-backed stealth seam. Legacy builder and replayable bundle callers no
longer own shadow sender formulas; they are explicit stateless compatibility
adapters over the canonical helper/formula layer. The public sender surfaces now
honestly separate raw, serial-aware raw, request-bound validated, and dedicated
card-only validated lanes, while Stage 3 runtime wording stays aligned with the
wallet-owned sender policy.

## Repository Changes

- `crates/z00z_wallets/src/core/stealth/output_build.rs` now owns the helper
  topology used by the migrated sender adapters, including helper-owned full-leaf
  and replayable build-state paths, while keeping `select_r(...)` as the
  wallet-owned scalar policy and making the replayable/stateless lane explicit.
- `crates/z00z_wallets/src/core/stealth/output.rs` now routes full-leaf and
  replayable builders through canonical helper/formula ownership, exposes the
  serial-aware raw helper, and documents which surfaces are stateless
  compatibility lanes versus wallet-owned sender-policy lanes.
- `crates/z00z_wallets/src/core/tx/builder.rs` and
  `crates/z00z_wallets/src/core/tx/output_flow.rs` are now adapter-only
  compatibility surfaces instead of independent sender derivation owners.
- `crates/z00z_wallets/src/core/stealth/mod.rs` and `crates/z00z_wallets/src/lib.rs`
  now export the card-only validated seam on the stable sender surface and keep
  the legacy-bypass error wording aligned with the supported compatibility model.
- `crates/z00z_wallets/src/core/stealth/test_output.rs` and
  `crates/z00z_wallets/src/core/stealth/test_output_extra.rs` now freeze the
  approval-level split with success, fail-closed, and request-none compatibility
  coverage for the card-only and request-bound validated lanes.
- `crates/z00z_simulator/src/scenario_1/stage_3.rs` and
  `crates/z00z_simulator/src/scenario_1/stage_3_runtime.rs` now route claim
  outputs through the serial-aware raw sender seam, keep `SenderWallet` state
  per output rather than per actor wave, and state explicitly that `rng_mode`
  only drives asset distribution.
- `.planning/phases/035-mix2-fixes/035-11-PLAN.md`,
  `.planning/phases/035-mix2-fixes/035-4-fix-spec.md`, and
  `.planning/phases/035-mix2-fixes/035-TODO.md` now describe the narrowed honest
  contract instead of the older stronger convergence claim.
- `.planning/phases/035-mix2-fixes/035-11-REVIEW.md` records a clean scoped
  review result for tasks `035-25` through `035-27`.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`: passed.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump`:
  passed.
- Repeated YOLO read-only review loop: exceeded the minimum three-pass
  requirement and closed only after three consecutive clean passes on the final
  Plan 11 surface.

## Review Loop

The review loop exceeded the minimum three-pass requirement before closure was
accepted.

- Early passes blocked on remaining scalar-selection ownership under full-leaf
  and replayable helpers, per-output `SenderWallet` recreation in Stage 3, and
  contract-honesty drift in docs and runtime wording.
- Mid passes cleared the code-path issues after helper ownership moved into
  `output_build.rs`, Stage 3 sender state was scoped honestly to each output,
  and the stateless compatibility lanes were documented explicitly.
- Final passes blocked only on planning/spec truth drift; those were cleared by
  synchronizing Plan 11, the sender spec, the TODO matrix, the crate-level export
  surface, and the review artifact.
- The last three scoped review passes were clean, which satisfied the mandatory
  closure loop.

## Current Boundary

This summary closes only the second sender-workflow slice for `035-25` through
`035-27`. It does not claim completion of downstream sender regressions,
temporary-document correction, or acceptance-gate work reserved for Plans 12+
and specifically does not close the later documentation wave under `035-29` or
validation/acceptance waves under `035-30` and `035-31`.
