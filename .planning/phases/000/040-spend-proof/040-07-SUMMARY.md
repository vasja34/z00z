# 040-07 Summary

## Audit Retraction

A repository-backed audit on 2026-04-27 reopened this summary. The live
default spend proof path was restored to the honest statement-bound suite
`regular_spend_statement_bound_v1` after confirming that the prior
theorem-branded artifact serialized witness bytes and replayed
`verify_spend_rules(...)` instead of verifying a zero-knowledge
theorem-carrying proof. Treat the remaining theorem-closure language below as
historical overclaim, not current repository truth.

## ⚠️ Scope

This summary preserves the formerly claimed completion state for
`040-07-PLAN.md`. After the 2026-04-27 audit, it now records why that claim is
not valid as live repository truth.

## ⚠️ Repo-Backed Verdict

Plan 07 is not closed on a landed theorem-carrying regular-spend backend.

The current live code still returns `StatementBoundSpendProofBackend` from
`default_spend_proof_backend()`, freezes `SPEND_PROOF_SUITE` at
`regular_spend_statement_bound_v1`, and explicitly accepts statement-bound
artifacts on both the backend seam and the public verifier seam.

Typed witness plumbing, typed statement projection, and theorem-v2 artifact
framing exist in the working tree, but theorem proof verification is not live.

## ✅ Surfaces That Did Land

- `crates/z00z_wallets/src/core/tx/spend_verification.rs`
  - recomputes the canonical statement for verifier-side admission
  - passes explicit witness material into the backend prove path
- `crates/z00z_wallets/src/core/tx/spend_proof_backend.rs`
  - validates non-empty witness shape
  - can parse theorem-v2 framing
  - still emits and verifies only the statement-bound artifact on the live seam
- `crates/z00z_wallets/tests/test_spend_proof_backend.rs`
  - proves statement drift rejection, malformed-payload rejection, empty-witness
    rejection, theorem-shaped public-input alignment, and explicit
    statement-bound artifact acceptance
- `crates/z00z_wallets/tests/test_tx_proof_verifier.rs`
  - keeps the public verifier fail closed on malformed or drifted carrier data
  - still proves that the live verifier accepts the statement-bound artifact

## 🔍 Audit Basis

- `crates/z00z_wallets/src/core/tx/spend_proof_backend.rs`
  - `default_spend_proof_backend()` still returns
    `StatementBoundSpendProofBackend`
  - theorem-v2 artifacts still reject as `UnsupportedSuite`
- `crates/z00z_wallets/src/core/tx/tx_wire_types.rs`
  - `SPEND_PROOF_SUITE` remains `regular_spend_statement_bound_v1`
- `crates/z00z_wallets/tests/test_spend_proof_backend.rs`
  - `test_backend_accepts_statement_bound_artifact`
- `crates/z00z_wallets/tests/test_tx_proof_verifier.rs`
  - `test_verifier_accepts_statement_bound_artifact`

## ⚠️ Current Boundary

Use `040-VALIDATION.md` and `040-UAT.md` for live status. `040-09` and the
theorem-closeout portion of `040-CG` remain audit-reopened until a real
zero-knowledge theorem backend exists.
