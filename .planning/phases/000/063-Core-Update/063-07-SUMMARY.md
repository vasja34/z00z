---
phase: 063-Core-Update
plan: 063-07
status: complete
completed_at: 2026-06-28
next_plan: 063-08
summary_artifact_for: .planning/phases/063-Core-Update/063-07-PLAN.md
---

# 063-07 Summary: Canonical `vouchers` Namespace Only

## Outcome

`063-07` is complete. `PLAN-063-G07` now closes `REC-063-P1-04` by removing
the misspelled live `vauchers` lane and keeping one canonical
`z00z_core::vouchers` owner path across core and downstream callers.

The live voucher owner now runs through `crates/z00z_core/src/vouchers/mod.rs`
plus the crate-root `pub mod vouchers;` export in `crates/z00z_core/src/lib.rs`.
Core docs and downstream imports across storage, wallets, and simulator now
name one path, and no compatibility alias or shim remains. The namespace
cleanup did not change voucher meaning: `VoucherBootstrapEntryV1` remains the
bootstrap-manifest concept rather than a runtime voucher object, and vouchers
are not reframed as reducible to `asset + right`.

The current Phase 063 authority packet was also truth-restored where it names
live voucher owner paths. After this slice, the planning packet points at the
live `vouchers` owner path when it references current code anchors, while
keeping `vauchers` only as the historical debt label or as a zero-hit scan
target.

The final release validation for this slice also exposed a pre-existing
simulator failpoint test leak outside the namespace rename itself:
`test_claim_persist.rs` could leave `Z00Z_FAIL_CLAIM_PUB` or
`Z00Z_FAIL_ASSET_SAVE` set after a panic. The validated tree now restores
those env vars through a local RAII guard so the broad release rerun stays
green on the renamed voucher path.

## Files Changed

- `crates/z00z_core/src/lib.rs`
- `crates/z00z_core/src/vouchers/{mod.rs,test_voucher_config.rs}`
- `crates/z00z_core/README.md`
- downstream voucher-import rewrites under `crates/z00z_storage/{src,tests}/**`
- downstream voucher-import rewrites under `crates/z00z_wallets/{src,tests}/**`
- downstream voucher-import rewrites under `crates/z00z_simulator/{src,tests}/**`
- `crates/z00z_simulator/tests/scenario_1/test_claim_persist.rs`
- `.planning/phases/063-Core-Update/{063-07-PLAN.md,063-11-PLAN.md,063-CONTEXT.md,063-TODO.md,063-TEST-SPEC.md}`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_core --test test_voucher_config -- --nocapture`
- `cargo test --release -p z00z_wallets object_rpc -- --nocapture`
- `cargo test --release -p z00z_validators test_object_policy_verdicts -- --nocapture`
- `cargo test --release -p z00z_simulator --test scenario_1 test_claim_persist::test_stage3_fail_claim_pub -- --nocapture`
- `cargo test --release`
- `rg -n "VoucherBootstrapEntryV1" crates/z00z_core crates/z00z_wallets crates/z00z_storage crates/z00z_simulator`
- `rg -n "\\bvauchers\\b" crates/z00z_core crates/z00z_storage crates/z00z_wallets crates/z00z_simulator`
- `rg -n "vauchers/mod.rs|src/vauchers/test_voucher_config.rs" .planning/phases/063-Core-Update/063-07-PLAN.md .planning/phases/063-Core-Update/063-11-PLAN.md .planning/phases/063-Core-Update/063-CONTEXT.md .planning/phases/063-Core-Update/063-TODO.md .planning/phases/063-Core-Update/063-TEST-SPEC.md`
- `git diff --check -- .planning/phases/063-Core-Update/063-07-SUMMARY.md .planning/phases/063-Core-Update/063-07-PLAN.md .planning/phases/063-Core-Update/063-11-PLAN.md .planning/phases/063-Core-Update/063-CONTEXT.md .planning/phases/063-Core-Update/063-TODO.md .planning/phases/063-Core-Update/063-TEST-SPEC.md .planning/STATE.md .planning/ROADMAP.md`
- Result: green

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times
against this slice, but the available runtime paths still did not produce a
review:

- Attempt 1
  - `timeout 30s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/063-Core-Update/063-07-PLAN.md current_task="PLAN-063-G07 final review in YOLO mode"'`
  - Result: timed out with no output
- Attempt 2
  - `timeout 30s gsd --no-session -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/063-Core-Update/063-07-PLAN.md current_task="PLAN-063-G07 second review in YOLO mode"'`
  - Result: timed out with no output
- Attempt 3
  - `timeout 30s gsd --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/063-Core-Update/063-07-PLAN.md current_task="PLAN-063-G07 third review in YOLO mode"'`
  - Result: timed out with no output

The repo-local launcher remains broken before prompt execution:

- `./.github/gsd-core/bin/gsd_run --help`
- Result: immediate `MODULE_NOT_FOUND` on missing `../../../package.json`
  from `.github/gsd-core/bin/lib/runtime-artifact-conversion.cjs`

Equivalent review passes were executed manually against the same slice.

- Pass 1
  - Reviewed `src/lib.rs`, `src/vouchers/mod.rs`, `README.md`, representative
    downstream voucher import sites, `063-TODO.md`, `063-CONTEXT.md`,
    `063-07-PLAN.md`, `063-11-PLAN.md`, and `063-TEST-SPEC.md` with the
    workspace-first `doublecheck` criteria plus the one-canonical-path rule
  - Result: no alias or shim remained, no current authority file pointed at
    the dead owner path, and no type-concept drift was found around
    `VoucherBootstrapEntryV1`
- Pass 2
  - `cargo test --release -p z00z_core --test test_voucher_config -- --nocapture`
  - `cargo test --release -p z00z_wallets object_rpc -- --nocapture`
  - `rg -n "VoucherBootstrapEntryV1" crates/z00z_core crates/z00z_wallets crates/z00z_storage crates/z00z_simulator`
  - `rg -n "\\bvauchers\\b" crates/z00z_core crates/z00z_storage crates/z00z_wallets crates/z00z_simulator`
  - `rg -n "vauchers/mod.rs|src/vauchers/test_voucher_config.rs" .planning/phases/063-Core-Update/063-07-PLAN.md .planning/phases/063-Core-Update/063-11-PLAN.md .planning/phases/063-Core-Update/063-CONTEXT.md .planning/phases/063-Core-Update/063-TODO.md .planning/phases/063-Core-Update/063-TEST-SPEC.md`
  - Result: clean
- Pass 3
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo test --release`
  - Result: clean

Passes 2 and 3 were consecutive clean runs.

## Completion Notes

- `063-07-SUMMARY.md` closes `PLAN-063-G07` and advances the execution lane to
  `063-08-PLAN.md`.
- `z00z_core::vouchers` is now the only live voucher namespace path.
- `VoucherBootstrapEntryV1` stays the bootstrap-manifest concept.
- The live Phase 063 authority packet now points at `vouchers` wherever it
  names the current voucher owner path.
- The final validated tree also restores simulator failpoint env vars
  fail-closed during broad release reruns.
