# 035-12 Summary

## Scope

This summary records the completion state for `035-12-PLAN.md`, covering task
`035-28 Downstream Adapter Regression Sweep` and task
`035-29 Documentation Correction Wave`.

## Outcome

Plan 12 is fully closed.

Phase 035 now has the third sender-workflow slice closed on one repository-
backed truth surface. Downstream simulator-facing sender callers route through
the canonical Stage 3 wire helper and the serial-aware raw wallet seam, while
the temp sender notes and sender-facing comments now describe the live helper
ownership and approval boundaries without reviving obsolete adapter claims.

## Repository Changes

- `crates/z00z_simulator/src/scenario_1/stage_3.rs` now exposes the live
  `to_claim_wire(...)` production helper as the single simulator claim-wire
  assembly seam and keeps `SenderWallet` state per output.
- `crates/z00z_simulator/tests/test_stage3_nullifier_store.rs` and
  `crates/z00z_simulator/tests/test_claim_tx_pipeline.rs` now reuse the live
  `to_claim_wire(...)` seam instead of rebuilding sender-output glue inside
  test-local helpers.
- `crates/z00z_wallets/src/core/stealth/test_output.rs` now freezes the live
  serial-binding contract at two levels: fixed-input formula binding through
  `build_output_core(...)` and real public-wrapper validation through
  `build_tx_stealth_output_for(...)` plus reconstructed `SenderValidationCtx`.
- `crates/z00z_wallets/src/core/stealth/output.rs` keeps the real public
  serial-aware raw seam minimal and free of extra test-only wrapper logic.
- `.planning/temp/Z00Z-ECC-SPEC_part1.md` and
  `.planning/temp/Z00Z-ECC-IDEAS.md` now state the current sender ownership
  honestly: `output_build.rs` owns the tx-output raw sender helper/formula seam,
  while compatibility leaf assembly remains in `output.rs`.
- `.planning/phases/035-mix2-fixes/035-4-fix-spec.md` and
  `.planning/phases/035-mix2-fixes/035-TODO.md` now record the downstream
  adapter freeze and documentation-correction slice as closed repository-backed
  work.
- `.planning/phases/035-mix2-fixes/035-11-SUMMARY.md` now matches the live
  Stage 3 sender-state truth instead of the older per-actor-wave wording.

## Validation

- `cargo test -p z00z_wallets --lib core::stealth::output::tests::test_public_serial_aware_sender_seam_rejects_wrong_serial_validation_ctx -- --exact`:
  passed.
- `cargo test -p z00z_wallets --lib core::stealth::output::tests::test_serial_id_changes_leaf_binding_when_inputs_are_fixed -- --exact`:
  passed.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage3_nullifier_store test_restart_replay_path -- --exact`:
  passed.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_tx_pipeline test_portable_wire_preserves_requested_serial_lane -- --exact`:
  passed.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface test_scenario1_stage_surface -- --exact`:
  passed.
- Repeated independent review loop: closed only after two consecutive clean
  passes reported `no findings` on the final Plan 12 scope.

## Review Loop

The review loop stayed open until both downstream coverage and documentation
truth drift were resolved.

- Early passes blocked on brittle or misleading coverage, stale temp-doc claims
  about helper ownership, and Stage 3 wording that drifted from the live
  sender-state scope.
- Mid passes tightened the public serial-aware seam regression so it proves the
  real wrapper output fails closed under the wrong `serial_id` validation
  context instead of relying on incidental randomness.
- Later passes removed simulator-layer shadow glue by making integration tests
  consume the production `to_claim_wire(...)` helper directly.
- The last two independent passes returned `no findings`, which satisfied the
  mandatory closure loop for Plan 12.

## Current Boundary

This summary closes only the Phase 035 downstream-regression and documentation-
correction slice for `035-28` and `035-29`. It does not claim completion of the
full sender-workflow validation or acceptance-gate waves reserved for
`035-30` and `035-31`.
