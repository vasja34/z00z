# 040-06 Summary

## ⚠️ Scope

This summary records the current status of `040-06-PLAN.md`, covering the
`040-12 Checkpoint-Pipeline Reuse`, `040-13 Missing-Code Closure Tasks`, and
`040-14 Prohibited Shortcut Checklist` closeout slices.

## ✅ Outcome

Phase 040 closeout evidence now proves that the regular-spend path still reuses
the existing two-seam checkpoint pipeline, closes the missing-code ledger
against landed task owners or explicit bounded follow-up, and re-runs the
prohibited shortcut checklist against both the phase text surfaces and a live
persisted tx package.

The plan does not silently overclose the phase, however. The `2.12 Open
Questions / Missing Code Support` table in `040-Spend-Proof-Spec.md` still uses
older missing or partially-implemented wording for the prover and verifier
rows, so the completion-gate alignment clause remains fail-closed for separate
planning review before any full phase-close claim.

## ✅ Repository Changes

- `.planning/phases/040-spend-proof/040-CLOSEOUT-GATES.md`
  - recorded the final `040-12` checkpoint-reuse evidence, `040-13`
    missing-code closure matrix, `040-14` shortcut checklist, and completion-
    gate recheck
- `.planning/phases/040-spend-proof/040-CONTEXT.md`
  - updated the task-transfer matrix for `040-12` through `040-14` so context
    reflects the landed closeout state
- `.planning/phases/040-spend-proof/040-TODO.md`
  - marked the shortcut checklist as re-run, marked the mandatory-test and
    retired-draft completion-gate rows green, and kept the design-order
    alignment row fail-closed
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`
  - added expected Stage-4 `prev_root_hex` extraction for the package-coupled
    checkpoint adapter
- `crates/z00z_simulator/src/scenario_1/stage_11_apply.rs`
  - bound the accepted Stage-4 spend-proof root to the Stage-11 exec root
    before checkpoint apply
- `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs`
  - added explicit second-seam and root-drift regression coverage
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
  - locked the closeout wording and added a live tx-package shortcut guard for
    proof-suite, `receiver_cards`, and `C_fee` drift

## ✅ Validation

Focused validation passed on the final tree:

- `cargo test -p z00z_wallets --release --features test-fast --lib test_full_verifier_rejects_missing_public_spend_contract -- --nocapture`
- `cargo test -p z00z_wallets --release --features test-fast --lib test_public_spend_boundary_rejects_local_valid_package -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_checkpoint_acceptance -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump test_scenario1_stage_surface -- --nocapture`
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`

The required broad release gate also passed on the final tree:

- `cargo test --release --features test-fast --features wallet_debug_dump`

## ⚠️ Review Loop

The closeout review loop found three real issues before the final pass:

- `040-12` still lacked an explicit binding from the accepted Stage-4
  `prev_root_hex` to the Stage-11 exec root; that is now enforced before
  checkpoint apply and covered by `stage11_rejects_root_drift`
- `040-13` still reflected older `partial` wording in the missing-code matrix;
  it now closes the task-owned ledger honestly while recording the stale spec
  `2.12` table fail-closed for separate planning review
- `040-14` originally relied on circular text-only evidence and a weaker STARK
  shortcut discriminator; it now uses the stronger live boundary from context
  plus a live tx-package guard in `test_scenario1_stage_surface.rs`

The final review streak closed cleanly for `040-12`, `040-13`, and `040-14`.

## ⚠️ Current Boundary

All mandatory validation requested by `040-06-PLAN.md` now passes, the
checkpoint path remains package-coupled without a parallel regular-tx proof
layer, and no remaining implementation step depends on the retired legacy
draft.

The remaining explicit non-close condition is planning truth, not code or test
health: the spec `2.12` missing-code table still trails the landed task-owned
closure matrix. That drift stays recorded fail-closed in
`040-CLOSEOUT-GATES.md`, so this summary is ready for separate planning review
rather than a silent claim that Phase 040 is fully and textually closed.
