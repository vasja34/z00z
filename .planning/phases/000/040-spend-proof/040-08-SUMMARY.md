# 040-08 Summary

## Audit Retraction

A repository-backed audit on 2026-04-27 reopened this summary. The live
default spend proof path was restored to the honest statement-bound suite
`regular_spend_statement_bound_v1` after confirming that the prior
theorem-branded artifact serialized witness bytes and replayed
`verify_spend_rules(...)` instead of verifying a zero-knowledge
theorem-carrying proof. Treat the remaining theorem-closeout language below as
historical overclaim, not current repository truth.

## ⚠️ Scope

This summary preserves the formerly claimed completion state for
`040-08-PLAN.md`. After the 2026-04-27 audit, it now records why final
theorem-closeout alignment is still not a truthful live claim.

## ⚠️ Repo-Backed Verdict

Plan 08 is not closed on final theorem-closeout alignment.

The current repository still runs the live spend proof path on
`regular_spend_statement_bound_v1`, and several Phase 040 ledgers had to be
reopened because they described theorem-closeout as if it were already shipped.

## ✅ Surfaces That Needed Truth Restoration

- `.planning/phases/040-spend-proof/040-UAT.md`
  - now needs to keep Test 6 reopened instead of passed
- `.planning/phases/040-spend-proof/040-INTEGRITY-GATES.md`
  - must describe `040-09` as audit-reopened rather than closed
- `.planning/phases/040-spend-proof/040-CLOSEOUT-GATES.md`
  - must keep closeout wording bounded to the already-landed statement-bound
    carrier and the existing checkpoint seam
- `.planning/phases/040-spend-proof/040-CONTEXT.md`, `040-TODO.md`, and
  `040-Spend-Proof-Spec.md`
  - must keep theorem-backend closure phrased as a target or blocker, not as
    current fact
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
  - must guard the reopened statement-bound truth instead of expecting a live
    theorem suite

## 🔍 Audit Basis

- `crates/z00z_wallets/src/core/tx/spend_proof_backend.rs`
  - default backend remains statement-bound
- `crates/z00z_wallets/src/core/tx/tx_wire_types.rs`
  - live proof suite remains `regular_spend_statement_bound_v1`
- `crates/z00z_wallets/tests/test_spend_proof_backend.rs`
  - statement-bound artifacts still verify on the backend seam
- `crates/z00z_wallets/tests/test_tx_proof_verifier.rs`
  - statement-bound artifacts still verify on the public verifier seam

## ⚠️ Current Boundary

Use `040-VALIDATION.md` and `040-UAT.md` for live Phase 040 status. Theorem
closure remains a reopened blocker, and Plan 08 cannot honestly close until the
supporting ledgers and the live proof seam agree on a real zero-knowledge
backend.
